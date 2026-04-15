#[cfg(unix)]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};
use std::{
    io::ErrorKind,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path,
    sync::{Arc, RwLock, atomic::Ordering},
    time::Duration,
};

use anyhow::Context;
use axum::{
    Json, Router,
    body::Body,
    extract::{ConnectInfo, FromRef, State},
    http::{Request, Response, StatusCode, header::HeaderValue},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::cookie::Key;
use axum_server::tls_rustls::RustlsConfig;
use clap::crate_version;
use defguard_version::{Version, server::DefguardVersionLayer};
use serde::Serialize;
use tokio::{
    fs::OpenOptions,
    io::AsyncWriteExt,
    sync::{broadcast, mpsc, oneshot},
    task::JoinSet,
};
use tower_governor::{
    GovernorLayer, governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor,
};
use tower_http::trace::{self, TraceLayer};
use tracing::{Level, info_span};

use crate::{
    LogsReceiver, VERSION,
    assets::{index, web_asset},
    config::EnvConfig,
    enterprise::handlers::openid_login,
    error::ApiError,
    grpc::{ProxyServer, TlsConfig},
    handlers::{desktop_client_mfa, enrollment, password_reset, polling},
    setup::ProxySetupServer,
};

pub(crate) static ENROLLMENT_COOKIE_NAME: &str = "defguard_proxy";
pub(crate) static PASSWORD_RESET_COOKIE_NAME: &str = "defguard_proxy_password_reset";
const DEFGUARD_CORE_CONNECTED_HEADER: &str = "defguard-core-connected";
const DEFGUARD_CORE_VERSION_HEADER: &str = "defguard-core-version";
const RATE_LIMITER_CLEANUP_PERIOD: Duration = Duration::from_secs(60);
const X_FORWARDED_FOR: &str = "x-forwarded-for";
const X_POWERED_BY: &str = "x-powered-by";
pub const GRPC_CERT_NAME: &str = "proxy_grpc_cert.pem";
pub const GRPC_KEY_NAME: &str = "proxy_grpc_key.pem";
pub const GRPC_CA_CERT_NAME: &str = "grpc_ca_cert.pem";
pub const CORE_CLIENT_CERT_NAME: &str = "core_client_cert.pem";

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) grpc_server: ProxyServer,
    cookie_key: Arc<RwLock<Option<Key>>>,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        let maybe_key = state.cookie_key.read().unwrap().clone();
        // We return the dummy key only to satisfy the `FromRef` trait, but it is never
        // used in practice because of the `ensure_configured` middleware.
        maybe_key.unwrap_or_else(|| Key::from(&[0; 64]))
    }
}

async fn handle_404() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found")
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum ServerState {
    Setup,
    Disconnected,
    Connected,
}

impl From<&AppState> for ServerState {
    fn from(state: &AppState) -> Self {
        if !state.grpc_server.setup_completed() {
            Self::Setup
        } else if state.grpc_server.connected.load(Ordering::Relaxed) {
            Self::Connected
        } else {
            Self::Disconnected
        }
    }
}

#[derive(Serialize)]
struct AppInfo {
    version: &'static str,
    server_state: ServerState,
}

async fn app_info(State(state): State<AppState>) -> Result<Json<AppInfo>, ApiError> {
    let version = crate_version!();
    let server_state = ServerState::from(&state);

    Ok(Json(AppInfo {
        version,
        server_state,
    }))
}

async fn healthcheck() -> &'static str {
    "alive"
}

async fn healthcheckgrpc(State(state): State<AppState>) -> (StatusCode, &'static str) {
    if state.grpc_server.connected.load(Ordering::Relaxed) {
        (StatusCode::OK, "alive")
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Not connected to Defguard Core",
        )
    }
}

// Retrieves client address from the request. Uses either the left most x-forwarded-for
// header value, or socket address if the header is not present.
fn get_client_addr(request: &Request<Body>) -> String {
    request
        .headers()
        .get(X_FORWARDED_FOR)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.split(',').next())
        .map_or_else(
            || {
                request
                    .extensions()
                    .get::<ConnectInfo<SocketAddr>>()
                    .map_or_else(|| "unknown".to_string(), |addr| addr.0.to_string())
            },
            |ip| ip.trim().to_string(),
        )
}

async fn core_version_middleware(
    State(app_state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let mut response = next.run(request).await;

    if let Some(core_version) = app_state.grpc_server.core_version.lock().unwrap().as_ref()
        && let Ok(core_version_header) = HeaderValue::from_str(&core_version.to_string())
    {
        response
            .headers_mut()
            .insert(DEFGUARD_CORE_VERSION_HEADER, core_version_header);
    }

    let core_connected = app_state.grpc_server.connected.load(Ordering::Relaxed);
    let core_connected_header =
        HeaderValue::from_static(if core_connected { "true" } else { "false" });

    response
        .headers_mut()
        .insert(DEFGUARD_CORE_CONNECTED_HEADER, core_connected_header);

    response
}

async fn powered_by_header<B>(mut response: Response<B>) -> Response<B> {
    response
        .headers_mut()
        .insert(X_POWERED_BY, HeaderValue::from_static("Defguard"));
    response
}

pub async fn run_setup(env_config: &EnvConfig, logs_rx: LogsReceiver) -> anyhow::Result<TlsConfig> {
    let setup_server = ProxySetupServer::new(logs_rx);
    let cert_dir = Path::new(&env_config.cert_dir);
    if !cert_dir.exists() {
        tokio::fs::create_dir_all(cert_dir).await.map_err(|err| {
            if err.kind() == ErrorKind::PermissionDenied {
                anyhow::anyhow!(
                    "Cannot create certificate directory {}. Permission denied.",
                    cert_dir.display()
                )
            } else {
                err.into()
            }
        })?;
        #[cfg(unix)]
        tokio::fs::set_permissions(cert_dir, Permissions::from_mode(0o700)).await?;
    }

    // Only attempt setup if not already configured
    info!(
        "No gRPC TLS certificates found at {}, new certificates will be obtained during setup",
        cert_dir.display()
    );
    let tls_config = setup_server
        .await_initial_setup(SocketAddr::new(
            env_config
                .grpc_bind_address
                .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            env_config.grpc_port,
        ))
        .await?;
    info!("Generated new gRPC TLS certificates and signed by Defguard Core");

    let TlsConfig {
        grpc_cert_pem,
        grpc_key_pem,
        grpc_ca_cert_pem,
        core_client_cert_der,
    } = &tls_config;

    let cert_path = cert_dir.join(GRPC_CERT_NAME);
    let key_path = cert_dir.join(GRPC_KEY_NAME);
    // Certificate and its key will be accessed only to this process's user.
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    options.mode(0o600); // rw-------

    // Write certificate to a file.
    options
        .clone()
        .open(&cert_path)
        .await?
        .write_all(grpc_cert_pem.as_bytes())
        .await.map_err(|err| {
            if err.kind() == ErrorKind::PermissionDenied {
                anyhow::anyhow!(
                    "Cannot write certificate file {}. Permission denied for certificate directory {}.",
                    cert_path.display(),
                    cert_dir.display()
                )
            } else {
                err.into()
            }
        })?;
    // Write key to a file.
    options
        .clone()
        .open(&key_path)
        .await?
        .write_all(grpc_key_pem.as_bytes())
        .await
        .map_err(|err| {
            if err.kind() == ErrorKind::PermissionDenied {
                anyhow::anyhow!(
                    "Cannot write key file {}. Permission denied for certificate directory {}.",
                    key_path.display(),
                    cert_dir.display()
                )
            } else {
                err.into()
            }
        })?;
    // Write CA certificate to a file.
    options
        .clone()
        .open(cert_dir.join(GRPC_CA_CERT_NAME))
        .await?
        .write_all(grpc_ca_cert_pem.as_bytes())
        .await?;
    // Write Core client certificate (PEM-encoded) to a file for serial pinning on restart.
    let core_client_cert_pem =
        defguard_certs::der_to_pem(core_client_cert_der, defguard_certs::PemLabel::Certificate)
            .map_err(|err| {
                anyhow::anyhow!("Failed to PEM-encode Core client certificate: {err}")
            })?;
    options
        .open(cert_dir.join(CORE_CLIENT_CERT_NAME))
        .await?
        .write_all(core_client_cert_pem.as_bytes())
        .await?;

    Ok(tls_config)
}

/// Middleware that gates all HTTP endpoints except health checks until the proxy
/// is fully configured.
///
/// The proxy cannot safely handle requests that rely on encrypted cookies
/// (e.g. OpenID / MFA flows) until it receives the cookie encryption key from
/// the core. This key is provided asynchronously after the core connects.
///
/// Until the key is available, only health check endpoints are served and all
/// other requests return HTTP 503 (Service Unavailable). Once the key is set,
/// the middleware becomes a no-op and all routes are enabled.
async fn ensure_configured(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    // Allow healthchecks and info even before core connects and gives us the cookie key.
    let path = request.uri().path();
    if matches!(
        path,
        "/api/v1/health" | "/api/v1/health-grpc" | "/api/v1/info"
    ) {
        return next.run(request).await;
    }

    // Block all other requests until cookie key is configured.
    if state.cookie_key.read().unwrap().is_none() {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }

    next.run(request).await
}

async fn build_tls_config(cert_pem: &str, key_pem: &str) -> anyhow::Result<RustlsConfig> {
    RustlsConfig::from_pem(cert_pem.as_bytes().to_vec(), key_pem.as_bytes().to_vec())
        .await
        .context("Failed to build HTTPS TLS configuration from PEM")
}

pub async fn run_server(
    env_config: EnvConfig,
    tls_config: Option<TlsConfig>,
    logs_rx: Option<LogsReceiver>,
) -> anyhow::Result<()> {
    info!("Starting Defguard Proxy server");
    debug!("Using config: {env_config:?}");

    let mut tasks = JoinSet::new();
    let cookie_key = Arc::default();
    let (reset_tx, mut reset_rx) = tokio::sync::broadcast::channel(1);
    let (https_cert_tx, https_cert_rx) = broadcast::channel::<(String, String)>(1);
    let (clear_https_tx, clear_https_rx) = broadcast::channel::<()>(1);

    // When the main HTTP server is on port 80, create a channel so the ACME task can request
    // a graceful hand-off of port 80 before binding its temporary challenge listener.
    let (port80_pause_tx, port80_pause_rx) = if env_config.http_port == 80 {
        let (tx, rx) = mpsc::channel::<(oneshot::Sender<()>, oneshot::Receiver<()>)>(1);
        (Some(tx), Some(rx))
    } else {
        (None, None)
    };

    // Both the gRPC setup loop and the permanent ProxyServer
    // share the same Arc<Mutex<Receiver<LogEntry>>> so they draw from the same channel.
    let logs_rx = logs_rx
        .ok_or_else(|| anyhow::anyhow!("Log receiver not available; cannot start server"))?;

    // connect to upstream gRPC server
    let grpc_server = ProxyServer::new(
        Arc::clone(&cookie_key),
        env_config.cert_dir.clone(),
        reset_tx,
        https_cert_tx,
        clear_https_tx,
        port80_pause_tx,
        Arc::clone(&logs_rx),
        env_config.acme_staging,
    );

    // Preload existing TLS configuration so /api/v1/info can report "disconnected"
    // immediately on startup
    if let Some(existing_tls_config) = tls_config.clone() {
        grpc_server.configure(existing_tls_config);
    }

    let server_clone = grpc_server.clone();
    let env_config_clone = env_config.clone();

    // Start gRPC server.
    debug!("Spawning gRPC server task");
    tasks.spawn(async move {
        let mut proxy_tls_config = tls_config;

        loop {
            let configuration = if let Some(conf) = proxy_tls_config.clone() {
                debug!("Using existing gRPC certificates, skipping setup process");
                conf
            } else {
                info!("gRPC certificates not found, running setup process");
                let conf = run_setup(&env_config_clone, Arc::clone(&logs_rx)).await?;
                info!("Setup process completed successfully");
                proxy_tls_config = Some(conf.clone());
                conf
            };

            server_clone.configure(configuration);
            info!("Starting gRPC server...");
            let server_to_run = server_clone.clone();
            let addr = SocketAddr::new(
                env_config
                    .grpc_bind_address
                    .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
                env_config.grpc_port,
            );
            let mut shutdown_rx = reset_rx.resubscribe();
            let mut server_task = tokio::spawn(async move {
                server_to_run
                    .run(addr, async move {
                        let _ = shutdown_rx.recv().await;
                    })
                    .await
            });

            tokio::select! {
                result = &mut server_task => {
                    match result {
                        Ok(Ok(())) => {}
                        Ok(Err(err)) => error!("gRPC server error: {err:?}, restarting..."),
                        Err(err) => error!("gRPC server task error: {err:?}, restarting..."),
                    }
                }
                result = reset_rx.recv() => {
                    if result.is_ok() {
                        info!("Reset requested, restarting setup process");
                        proxy_tls_config = None;
                    } else {
                        error!("Reset channel closed; gRPC server will keep running");
                    }
                    let _ = server_task.await;
                }
            }
        }
    });

    // build application
    debug!("Setting up API server");
    let shared_state = AppState {
        grpc_server,
        cookie_key,
    };

    // Setup tower_governor rate-limiter
    debug!(
        "Configuring rate limiter, per_second: {}, burst: {}",
        env_config.rate_limit_per_second, env_config.rate_limit_burst
    );
    let governor_conf = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(env_config.rate_limit_per_second)
        .burst_size(env_config.rate_limit_burst)
        .finish();

    let governor_conf = if let Some(conf) = governor_conf {
        let governor_limiter = conf.limiter().clone();

        // Start background task to cleanup rate-limiter data
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(RATE_LIMITER_CLEANUP_PERIOD).await;
                debug!(
                    "Cleaning-up rate limiter storage, current size: {}",
                    governor_limiter.len()
                );
                governor_limiter.retain_recent();
            }
        });
        info!(
            "Configured rate limiter, per_second: {}, burst: {}",
            env_config.rate_limit_per_second, env_config.rate_limit_burst
        );
        Some(conf)
    } else {
        info!("Skipping rate limiter setup");
        None
    };

    // Build axum app
    let mut app = Router::new()
        .route("/", get(index))
        .route("/{*path}", get(index))
        .route("/fonts/{*path}", get(web_asset))
        .route("/assets/{*path}", get(web_asset))
        .nest(
            "/api/v1",
            Router::new()
                .nest("/enrollment", enrollment::router())
                .nest("/password-reset", password_reset::router())
                .nest("/client-mfa", desktop_client_mfa::router())
                .nest("/openid", openid_login::router())
                .route("/poll", post(polling::info))
                .route("/health", get(healthcheck))
                .route("/health-grpc", get(healthcheckgrpc))
                .route("/info", get(app_info)),
        )
        .fallback_service(get(handle_404))
        .layer(middleware::from_fn_with_state(
            shared_state.clone(),
            ensure_configured,
        ))
        .layer(middleware::map_response(powered_by_header))
        .layer(middleware::from_fn_with_state(
            shared_state.clone(),
            core_version_middleware,
        ))
        .layer(DefguardVersionLayer::new(Version::parse(VERSION)?))
        .with_state(shared_state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    let addr = get_client_addr(request);
                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        path = ?request.uri(),
                        // TODO: headers only in debug logs
                        // headers = ?request.headers(),
                        client_addr = addr,
                    )
                })
                .on_response(trace::DefaultOnResponse::new().level(Level::DEBUG)),
        );
    if let Some(conf) = governor_conf {
        app = app.layer(GovernorLayer::new(conf));
    }
    debug!("Configured API server routing: {app:?}");

    // Start web server.
    debug!("Spawning API web server");
    tasks.spawn(async move {
        let http_addr = SocketAddr::new(
            env_config
                .http_bind_address
                .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            env_config.http_port,
        );
        let https_addr = SocketAddr::new(
            env_config
                .http_bind_address
                .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            env_config.https_port,
        );
        let mut current_tls: Option<(String, String)> = None;
        let mut https_cert_rx = https_cert_rx;
        let mut clear_https_rx = clear_https_rx;
        let mut port80_pause_rx = port80_pause_rx;

        loop {
            let handle = axum_server::Handle::new();
            let handle_clone = handle.clone();
            let app_service = app.clone().into_make_service_with_connect_info::<SocketAddr>();
            let tls_certs = current_tls.clone();

            let mut server_task = tokio::spawn(async move {
                if let Some((cert_pem, key_pem)) = tls_certs {
                    let tls_config = build_tls_config(&cert_pem, &key_pem).await?;
                    info!("API web server is listening on https://{https_addr}");
                    axum_server::bind_rustls(https_addr, tls_config)
                        .handle(handle_clone)
                        .serve(app_service)
                        .await
                        .context("Error running HTTPS server")
                } else {
                    info!("API web server is listening on http://{http_addr}");
                    axum_server::bind(http_addr)
                        .handle(handle_clone)
                        .serve(app_service)
                        .await
                        .context("Error running HTTP server")
                }
            });

            tokio::select! {
                result = &mut server_task => {
                    match result {
                        Ok(Ok(())) => {}
                        Ok(Err(err)) => return Err(err),
                        Err(join_err) => {
                            return Err(anyhow::anyhow!("Web server task panicked: {join_err}"));
                        }
                    }
                    break;
                }
                result = https_cert_rx.recv() => {
                    match result {
                        Ok(certs) => {
                            info!("Received new HTTPS certificates, restarting web server with TLS");
                            current_tls = Some(certs);
                            handle.graceful_shutdown(Some(Duration::from_secs(30)));
                            let _ = server_task.await;
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            warn!("Missed HTTPS certificate update; will apply next one");
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            error!("HTTPS certificate channel closed unexpectedly");
                            break;
                        }
                    }
                }
                result = clear_https_rx.recv() => {
                    match result {
                        Ok(()) => {
                            info!("Received ClearHttpsCerts, restarting web server without TLS");
                            current_tls = None;
                            handle.graceful_shutdown(Some(Duration::from_secs(30)));
                            let _ = server_task.await;
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            warn!("Missed ClearHttpsCerts update; will apply next one");
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            error!("ClearHttpsCerts channel closed unexpectedly");
                            break;
                        }
                    }
                }
                // An ACME task needs port 80: gracefully stop the current HTTP server,
                // signal the task that port 80 is free, wait until it's done, then let
                // the loop restart the server.
                Some((ready_tx, done_rx)) = async {
                    if let Some(rx) = port80_pause_rx.as_mut() {
                        rx.recv().await
                    } else {
                        std::future::pending().await
                    }
                } => {
                    info!("ACME task requested port 80; pausing HTTP server");
                    handle.graceful_shutdown(Some(Duration::from_secs(10)));
                    let _ = server_task.await;
                    // Port 80 is now free - tell the ACME task to proceed.
                    let _ = ready_tx.send(());
                    // Wait for the ACME task to finish with port 80.
                    let _ = done_rx.await;
                    info!("ACME task released port 80; restarting HTTP server");
                    // Loop will restart the server automatically.
                }
            }
        }

        Ok(())
    });

    // TODO: Possibly switch to using select! macro
    info!("Defguard Proxy HTTP server initialization complete");
    while let Some(Ok(result)) = tasks.join_next().await {
        result?;
    }

    Ok(())
}
