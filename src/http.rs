#[cfg(unix)]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};
use std::{
    future::Future,
    io::ErrorKind,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use anyhow::Context;
use axum::{
    Json, Router,
    body::Body,
    extract::{ConnectInfo, DefaultBodyLimit, FromRef, State},
    http::{HeaderName, HeaderValue, Request, Response, StatusCode, header},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::{
    PrivateCookieJar,
    cookie::{Cookie, Key, SameSite},
};
use axum_server::tls_rustls::RustlsConfig;
use clap::crate_version;
use cookie::CookieBuilder;
use defguard_version::{Version, server::DefguardVersionLayer};
use serde::Serialize;
use tokio::{
    sync::{broadcast, mpsc, oneshot, watch},
    task::JoinSet,
};
use tower_governor::{
    GovernorLayer, governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor,
};
use tower_http::{
    timeout::TimeoutLayer,
    trace::{self, TraceLayer},
};
use tracing::{Level, info_span};

use crate::{
    LogsReceiver, VERSION,
    assets::{index, web_asset},
    config::EnvConfig,
    enterprise::handlers::{desktop_client_posture, openid_login},
    error::ApiError,
    grpc::{ProxyServer, TlsConfig},
    handlers::{desktop_client_mfa, enrollment, mfa_config, mfa_flow, password_reset, polling},
    setup::ProxySetupServer,
};

pub(crate) static ENROLLMENT_COOKIE_NAME: &str = "defguard_proxy";
pub(crate) const ENROLLMENT_COOKIE_PATH: &str = "/api/v1/enrollment";
pub(crate) static PASSWORD_RESET_COOKIE_NAME: &str = "defguard_proxy_password_reset";
pub(crate) const PASSWORD_RESET_COOKIE_PATH: &str = "/api/v1/password-reset";

const DEFGUARD_CORE_CONNECTED_HEADER: &str = "defguard-core-connected";
const DEFGUARD_CORE_VERSION_HEADER: &str = "defguard-core-version";
const RATE_LIMITER_CLEANUP_PERIOD: Duration = Duration::from_secs(60);
// Drain active work within Docker's default 10-second stop window.
const SERVER_SHUTDOWN_GRACE_PERIOD: Duration = Duration::from_secs(5);
const SERVER_SHUTDOWN_FORCE_TIMEOUT: Duration = Duration::from_secs(6);
const SERVER_SHUTDOWN_DEADLINE: Duration = Duration::from_secs(7);
const X_FORWARDED_FOR: &str = "x-forwarded-for";
/// Default request body size limit applied globally to every route.
const REQUEST_BODY_LIMIT: usize = 256 * 1024; // 256 KB

async fn wait_for_server<F>(server: &mut F, timeout: Duration) -> anyhow::Result<bool>
where
    F: Future<Output = anyhow::Result<()>> + Unpin,
{
    match tokio::time::timeout(timeout, server).await {
        Ok(result) => {
            result?;
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

enum ServerWaitOutcome {
    Finished,
    Shutdown,
    TimedOut,
}

async fn receive_https_cert_update<T: Clone>(receiver: &mut broadcast::Receiver<T>) -> Option<T> {
    loop {
        match receiver.recv().await {
            Err(broadcast::error::RecvError::Lagged(_)) => {
                warn!("Missed HTTP server update; waiting for the latest one");
            }
            Ok(update) => return Some(update),
            Err(broadcast::error::RecvError::Closed) => return None,
        }
    }
}

async fn wait_for_server_or_shutdown<F>(
    server: &mut F,
    shutdown_rx: &mut watch::Receiver<bool>,
    restart_timeout: Duration,
) -> anyhow::Result<ServerWaitOutcome>
where
    F: Future<Output = anyhow::Result<()>> + Unpin,
{
    tokio::select! {
        result = &mut *server => {
            result?;
            Ok(ServerWaitOutcome::Finished)
        }
        _ = shutdown_rx.changed() => {
            if !wait_for_server(server, SERVER_SHUTDOWN_FORCE_TIMEOUT).await? {
                warn!("Server did not stop before the shutdown deadline");
            }
            Ok(ServerWaitOutcome::Shutdown)
        }
        _ = tokio::time::sleep(restart_timeout) => {
            warn!("Server did not stop before the restart deadline");
            Ok(ServerWaitOutcome::TimedOut)
        }
    }
}

/// Maximum time a single request may take before the server returns 408.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
// Header name constants not yet present in the `http` crate v1.x standard set.
const X_POWERED_BY: HeaderName = HeaderName::from_static("x-powered-by");
const PERMISSIONS_POLICY: HeaderName = HeaderName::from_static("permissions-policy");
const CROSS_ORIGIN_OPENER_POLICY: HeaderName =
    HeaderName::from_static("cross-origin-opener-policy");
const CROSS_ORIGIN_RESOURCE_POLICY: HeaderName =
    HeaderName::from_static("cross-origin-resource-policy");
pub use crate::setup::{CORE_CLIENT_CERT_NAME, GRPC_CA_CERT_NAME, GRPC_CERT_NAME, GRPC_KEY_NAME};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) grpc_server: ProxyServer,
    pub(crate) cookie_key: Arc<RwLock<Option<Key>>>,
}

impl AppState {
    pub(crate) fn new(grpc_server: ProxyServer, cookie_key: Arc<RwLock<Option<Key>>>) -> Self {
        Self {
            grpc_server,
            cookie_key,
        }
    }

    /// Whether session cookies must carry the `Secure` attribute. Derived by
    /// [`crate::grpc::PublicSettingsState`] from the public proxy URL Core sends, and defaulting
    /// to `true` until Core says otherwise.
    pub(crate) fn cookie_secure(&self) -> bool {
        self.grpc_server
            .public_settings
            .cookie_secure
            .load(Ordering::Relaxed)
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        let maybe_key = state.cookie_key.read().unwrap().clone();
        // We return the dummy key only to satisfy the `FromRef` trait, but it is never
        // used in practice because of the `ensure_configured` middleware.
        maybe_key.unwrap_or_else(|| Key::from(&[0; 64]))
    }
}

#[must_use]
pub(crate) fn session_cookie(
    name: &'static str,
    value: String,
    path: &'static str,
    secure: bool,
) -> CookieBuilder<'static> {
    Cookie::build((name, value))
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(secure)
        .path(path)
}

/// Build the removal counterpart of [`session_cookie`]. The attributes must match those the
/// cookie was set with, or the browser keeps the original.
// `PrivateCookieJar` is itself `#[must_use]`, so the return value cannot be dropped silently.
pub(crate) fn remove_session_cookie(
    cookies: PrivateCookieJar,
    name: &'static str,
    path: &'static str,
    secure: bool,
) -> PrivateCookieJar {
    cookies.remove(session_cookie(name, String::new(), path, secure).build())
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
    display_password_reset: bool,
    display_download_step: bool,
}

async fn app_info(State(state): State<AppState>) -> Result<Json<AppInfo>, ApiError> {
    let version = crate_version!();
    let server_state = ServerState::from(&state);

    Ok(Json(AppInfo {
        version,
        server_state,
        display_password_reset: state
            .grpc_server
            .public_settings
            .display_password_reset
            .load(Ordering::Relaxed),
        display_download_step: state
            .grpc_server
            .public_settings
            .display_download_step
            .load(Ordering::Relaxed),
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

/// Injects baseline security response headers on every response.
async fn security_headers_middleware(
    tls_active: Arc<AtomicBool>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let is_api = request.uri().path().starts_with("/api/");
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // `X-Powered-By: Defguard` - server identification header
    headers.insert(X_POWERED_BY, HeaderValue::from_static("Defguard"));

    // `X-Content-Type-Options: nosniff` - prevents MIME-type sniffing/confusion attacks
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    // `Referrer-Policy: strict-origin-when-cross-origin` - avoids leaking internal URLs via Referer to external sites
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // `Permissions-Policy: geolocation=(), camera=(), microphone=()` - disables unused browser APIs
    headers.insert(
        PERMISSIONS_POLICY,
        HeaderValue::from_static("geolocation=(), camera=(), microphone=()"),
    );

    // `Cross-Origin-Opener-Policy: same-origin` - severs window.opener references, preventing reverse tabnapping
    headers.insert(
        CROSS_ORIGIN_OPENER_POLICY,
        HeaderValue::from_static("same-origin"),
    );

    // `Cross-Origin-Resource-Policy: same-origin` - blocks cross-origin embedding of application resources
    headers.insert(
        CROSS_ORIGIN_RESOURCE_POLICY,
        HeaderValue::from_static("same-origin"),
    );

    // `X-Frame-Options: DENY` - clickjacking defense for browsers without CSP frame-ancestors support
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));

    // `Content-Security-Policy: frame-ancestors 'none'` - prevents framing/clickjacking
    // Use entry/or_insert so individual handlers can override CSP (e.g. per-request nonces)
    headers
        .entry(header::CONTENT_SECURITY_POLICY)
        .or_insert(HeaderValue::from_static("frame-ancestors 'none';"));

    // `Strict-Transport-Security` - only sent over TLS; ignored and potentially harmful over plain HTTP (RFC 6797 §7.2)
    let tls = tls_active.load(Ordering::Relaxed);
    if tls {
        headers.insert(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000"),
        );
    }

    // `Cache-Control: no-store` - prevents browsers and caches from storing sensitive API responses
    if is_api {
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }

    response
}

pub async fn run_setup(env_config: &EnvConfig, logs_rx: LogsReceiver) -> anyhow::Result<TlsConfig> {
    let cert_dir = Path::new(&env_config.cert_dir);
    if cert_dir.exists() {
        // verify write access before starting the setup server
        let test_path = cert_dir.join(".write_test");
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&test_path)
            .await
        {
            Ok(_) => {
                let _ = tokio::fs::remove_file(&test_path).await;
            }
            Err(err) if err.kind() == ErrorKind::PermissionDenied => {
                anyhow::bail!(
                    "Certificate directory {} is not writable. Permission denied.",
                    cert_dir.display()
                );
            }
            Err(err) => return Err(err.into()),
        }
    } else {
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

    info!(
        "No gRPC TLS certificates found at {}, new certificates will be obtained during setup",
        cert_dir.display()
    );
    let setup_server = ProxySetupServer::new(logs_rx, cert_dir.to_path_buf());
    let tls_config = setup_server
        .await_initial_setup(
            SocketAddr::new(
                env_config
                    .grpc_bind_address
                    .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
                env_config.grpc_port,
            ),
            env_config,
        )
        .await?;
    info!("Generated new gRPC TLS certificates and signed by Defguard Core");

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
    if state
        .cookie_key
        .read()
        .expect("cookie_key lock poisoned")
        .is_none()
    {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }

    next.run(request).await
}

async fn build_tls_config(cert_pem: &str, key_pem: &str) -> anyhow::Result<RustlsConfig> {
    RustlsConfig::from_pem(cert_pem.as_bytes().to_vec(), key_pem.as_bytes().to_vec())
        .await
        .context("Failed to build HTTPS TLS configuration from PEM")
}

/// All `/api/v1` routes without middleware, so tests can drive them with a fake Core.
pub(crate) fn api_router() -> Router<AppState> {
    Router::new().nest(
        "/api/v1",
        Router::new()
            .nest("/enrollment", enrollment::router())
            .nest("/password-reset", password_reset::router())
            .nest("/client-mfa", desktop_client_mfa::router())
            .nest("/mfa-flow", mfa_flow::router())
            .nest("/mfa-config", mfa_config::router())
            .nest("/openid", openid_login::router())
            .nest("/posture", desktop_client_posture::router())
            .route("/poll", post(polling::info))
            .route("/health", get(healthcheck))
            .route("/health-grpc", get(healthcheckgrpc))
            .route("/info", get(app_info)),
    )
}

pub(crate) fn build_router(
    state: AppState,
    apply_rate_limit: impl FnOnce(Router<AppState>) -> Router<AppState>,
    tls_active: Arc<AtomicBool>,
) -> anyhow::Result<Router> {
    // Collect all API routes into a separate router to scope API-only middleware.
    let mut api_router = apply_rate_limit(api_router());
    api_router = api_router.layer(middleware::from_fn_with_state(
        state.clone(),
        ensure_configured,
    ));

    // Build axum app
    let mut app = Router::new()
        .route("/", get(index))
        .route("/{*path}", get(index))
        .route("/fonts/{*path}", get(web_asset))
        .route("/assets/{*path}", get(web_asset))
        .merge(api_router)
        .fallback_service(get(handle_404))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            core_version_middleware,
        ))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ))
        .with_state(state)
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

    // Global request body size limit; all proxy endpoints have small payloads.
    app = app.layer(DefaultBodyLimit::max(REQUEST_BODY_LIMIT));
    // Security headers and version are the outermost layers so that ALL short-circuit
    // responses (408 timeout, 413 body-too-large, 429 rate-limited) also carry the
    // baseline security headers and the server version header.
    let tls_for_headers = Arc::clone(&tls_active);
    app = app
        .layer(middleware::from_fn(move |req, next| {
            security_headers_middleware(Arc::clone(&tls_for_headers), req, next)
        }))
        .layer(DefguardVersionLayer::new(Version::parse(VERSION)?));
    debug!("Configured API server routing: {app:?}");

    Ok(app)
}

pub async fn run_server(
    env_config: EnvConfig,
    tls_config: Option<TlsConfig>,
    logs_rx: Option<LogsReceiver>,
    shutdown_signal: impl Future<Output = anyhow::Result<()>>,
) -> anyhow::Result<()> {
    info!("Starting Defguard Proxy server");
    debug!("Using config: {env_config:?}");

    let mut tasks = JoinSet::new();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
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
    let mut grpc_shutdown_rx = shutdown_rx.clone();

    // Start gRPC server.
    debug!("Spawning gRPC server task");
    tasks.spawn(async move {
        let mut proxy_tls_config = tls_config;
        let keepalive_interval =
            Duration::from_secs(env_config.grpc_keepalive_interval_seconds.get());
        let keepalive_timeout =
            Duration::from_secs(env_config.grpc_keepalive_timeout_seconds.get());

        loop {
            if *grpc_shutdown_rx.borrow() {
                break;
            }

            let configuration = if let Some(conf) = proxy_tls_config.clone() {
                debug!("Using existing gRPC certificates, skipping setup process");
                conf
            } else {
                info!("gRPC certificates not found, running setup process");
                let setup = run_setup(&env_config_clone, Arc::clone(&logs_rx));
                tokio::pin!(setup);
                let configuration = tokio::select! {
                    result = &mut setup => result?,
                    _ = grpc_shutdown_rx.changed() => return Ok(()),
                };
                info!("Setup process completed successfully");
                configuration
            };
            proxy_tls_config = Some(configuration.clone());
            server_clone.configure(configuration);

            info!("Starting gRPC server...");
            let server_to_run = server_clone.clone();
            let addr = SocketAddr::new(
                env_config
                    .grpc_bind_address
                    .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
                env_config.grpc_port,
            );
            let mut reset_server_rx = reset_rx.resubscribe();
            let mut shutdown_server_rx = grpc_shutdown_rx.clone();
            let server =
                server_to_run.run(addr, keepalive_interval, keepalive_timeout, async move {
                    tokio::select! {
                        _ = reset_server_rx.recv() => {},
                        _ = shutdown_server_rx.changed() => {},
                    }
                });
            tokio::pin!(server);

            tokio::select! {
                result = &mut server => {
                    if let Err(err) = result {
                        error!("gRPC server error: {err:?}, restarting...");
                    }
                }
                result = reset_rx.recv() => {
                    if result.is_ok() {
                        info!("Reset requested, restarting setup process");
                        proxy_tls_config = None;
                    } else {
                        error!("Reset channel closed; gRPC server will keep running");
                    }
                    if !wait_for_server(&mut server, SERVER_SHUTDOWN_FORCE_TIMEOUT).await? {
                        warn!("gRPC server did not stop after reset; dropping it");
                    }
                }
                _ = grpc_shutdown_rx.changed() => {
                    info!("Shutdown signal received, draining gRPC server");
                    if !wait_for_server(&mut server, SERVER_SHUTDOWN_FORCE_TIMEOUT).await? {
                        warn!("gRPC server did not drain before the shutdown deadline");
                    }
                    return Ok(());
                }
            }

            if *grpc_shutdown_rx.borrow() {
                return Ok(());
            }
        }

        Ok(())
    });

    // build application
    debug!("Setting up API server");
    let tls_active = Arc::new(AtomicBool::new(false));
    let shared_state = AppState::new(grpc_server, cookie_key);

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
        let mut rate_limit_shutdown_rx = shutdown_rx.clone();
        tasks.spawn(async move {
            loop {
                tokio::select! {
                    _ = rate_limit_shutdown_rx.changed() => break,
                    _ = tokio::time::sleep(RATE_LIMITER_CLEANUP_PERIOD) => {
                        debug!(
                            "Cleaning-up rate limiter storage, current size: {}",
                            governor_limiter.len()
                        );
                        governor_limiter.retain_recent();
                    }
                }
            }
            Ok(())
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

    let app = build_router(
        shared_state,
        move |router| match governor_conf {
            Some(conf) => router.layer(GovernorLayer::new(conf)),
            None => router,
        },
        Arc::clone(&tls_active),
    )?;

    // Start web server.
    debug!("Spawning API web server");
    let mut http_shutdown_rx = shutdown_rx.clone();
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
            if *http_shutdown_rx.borrow() {
                break;
            }

            let handle = axum_server::Handle::new();
            let handle_clone = handle.clone();
            tls_active.store(current_tls.is_some(), Ordering::Relaxed);
            let app_service = app.clone().into_make_service_with_connect_info::<SocketAddr>();
            let tls_certs = current_tls.clone();

            let server = async move {
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
            };
            tokio::pin!(server);

            tokio::select! {
                result = &mut server => {
                    result?;
                    break;
                }
                result = receive_https_cert_update(&mut https_cert_rx) => {
                    match result {
                        Some(certs) => {
                            info!("Received new HTTPS certificates, restarting web server with TLS");
                            handle.graceful_shutdown(Some(Duration::from_secs(30)));
                            if matches!(
                                wait_for_server_or_shutdown(
                                    &mut server,
                                    &mut http_shutdown_rx,
                                    Duration::from_secs(31),
                                ).await?,
                                ServerWaitOutcome::Shutdown
                            ) {
                                return Ok(());
                            }
                            current_tls = Some(certs);
                        }
                        None => {
                            error!("HTTPS certificate channel closed unexpectedly");
                            handle.graceful_shutdown(Some(SERVER_SHUTDOWN_GRACE_PERIOD));
                            if !wait_for_server(&mut server, SERVER_SHUTDOWN_FORCE_TIMEOUT).await? {
                                warn!("HTTP server did not drain after certificate channel closed");
                            }
                            return Ok(());
                        }
                    }
                }
                result = receive_https_cert_update(&mut clear_https_rx) => {
                    match result {
                        Some(()) => {
                            info!("Received ClearHttpsCerts, restarting web server without TLS");
                            handle.graceful_shutdown(Some(Duration::from_secs(30)));
                            if matches!(
                                wait_for_server_or_shutdown(
                                    &mut server,
                                    &mut http_shutdown_rx,
                                    Duration::from_secs(31),
                                ).await?,
                                ServerWaitOutcome::Shutdown
                            ) {
                                return Ok(());
                            }
                            current_tls = None;
                        }
                        None => {
                            error!("ClearHttpsCerts channel closed unexpectedly");
                            handle.graceful_shutdown(Some(SERVER_SHUTDOWN_GRACE_PERIOD));
                            if !wait_for_server(&mut server, SERVER_SHUTDOWN_FORCE_TIMEOUT).await? {
                                warn!("HTTP server did not drain after certificate channel closed");
                            }
                            return Ok(());
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
                    match wait_for_server_or_shutdown(
                        &mut server,
                        &mut http_shutdown_rx,
                        Duration::from_secs(11),
                    ).await? {
                        ServerWaitOutcome::Finished => {}
                        ServerWaitOutcome::Shutdown => return Ok(()),
                        ServerWaitOutcome::TimedOut => {
                            return Err(anyhow::anyhow!("HTTP server did not stop for ACME hand-off"));
                        }
                    }
                    if *http_shutdown_rx.borrow() {
                        return Ok(());
                    }
                    let _ = ready_tx.send(());
                    tokio::select! {
                        _ = done_rx => {},
                        _ = http_shutdown_rx.changed() => return Ok(()),
                    }
                    if *http_shutdown_rx.borrow() {
                        return Ok(());
                    }
                    info!("ACME task released port 80; restarting HTTP server");
                }
                _ = http_shutdown_rx.changed() => {
                    info!("Shutdown signal received, draining HTTP server");
                    handle.graceful_shutdown(Some(SERVER_SHUTDOWN_GRACE_PERIOD));
                    if !wait_for_server(&mut server, SERVER_SHUTDOWN_FORCE_TIMEOUT).await? {
                        warn!("HTTP server did not drain before the shutdown deadline");
                    }
                    return Ok(());
                }
            }
        }

        Ok(())
    });

    info!("Defguard Proxy HTTP server initialization complete");
    tokio::pin!(shutdown_signal);
    loop {
        tokio::select! {
            result = &mut shutdown_signal => {
                info!("Shutdown signal received, draining server tasks");
                let _ = shutdown_tx.send(true);
                let all_tasks_stopped = tokio::time::timeout(
                    SERVER_SHUTDOWN_DEADLINE,
                    async {
                        while let Some(result) = tasks.join_next().await {
                            match result {
                                Ok(Ok(())) => {}
                                Ok(Err(err)) => error!("Server task failed during shutdown: {err:?}"),
                                Err(err) if err.is_cancelled() => {},
                                Err(err) => error!("Server task join failed during shutdown: {err}"),
                            }
                        }
                    },
                )
                .await
                .is_ok();
                if !all_tasks_stopped {
                    warn!("Server tasks exceeded the shutdown deadline; aborting remaining tasks");
                    tasks.abort_all();
                    while tasks.join_next().await.is_some() {}
                }
                result?;
                break;
            }
            result = tasks.join_next() => match result {
                Some(Ok(result)) => result?,
                Some(Err(_)) | None => break,
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
    };

    #[tokio::test]
    async fn test_lagged_update_drains_active_http_request() {
        let addr = std::net::TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
            .unwrap()
            .local_addr()
            .unwrap();
        let request_started = Arc::new(tokio::sync::Notify::new());
        let release_request = Arc::new(tokio::sync::Notify::new());
        let app = Router::new().route(
            "/",
            get({
                let request_started = Arc::clone(&request_started);
                let release_request = Arc::clone(&release_request);
                move || {
                    let request_started = Arc::clone(&request_started);
                    let release_request = Arc::clone(&release_request);
                    async move {
                        request_started.notify_one();
                        release_request.notified().await;
                        "request drained"
                    }
                }
            }),
        );
        let handle = axum_server::Handle::new();
        let server_handle = handle.clone();
        let mut server = Box::pin(async move {
            axum_server::bind(addr)
                .handle(server_handle)
                .serve(app.into_make_service())
                .await
                .map_err(anyhow::Error::from)
        });
        let (update_tx, mut update_rx) = broadcast::channel(1);
        let mut update = Box::pin(receive_https_cert_update(&mut update_rx));
        let mut request = tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr).await?;
            stream
                .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                .await?;
            let mut response = Vec::new();
            stream.read_to_end(&mut response).await?;
            Ok::<_, std::io::Error>(response)
        });

        tokio::select! {
            _ = request_started.notified() => {},
            result = &mut server => panic!("HTTP server stopped before the request started: {result:?}"),
            _ = &mut update => panic!("update arrived before the test sent one"),
            _ = tokio::time::sleep(Duration::from_secs(3)) => panic!("HTTP request did not start"),
        }
        assert!(!request.is_finished(), "handler should still be waiting");

        update_tx
            .send(("old cert".to_owned(), "old key".to_owned()))
            .unwrap();
        update_tx
            .send(("latest cert".to_owned(), "latest key".to_owned()))
            .unwrap();
        let latest = tokio::select! {
            result = &mut server => panic!("HTTP server stopped while the request was active: {result:?}"),
            update = &mut update => update.expect("latest certificate update should arrive"),
        };
        assert_eq!(latest.0, "latest cert");
        assert!(
            !request.is_finished(),
            "lagged update must not cancel the handler"
        );

        handle.graceful_shutdown(Some(Duration::from_secs(2)));
        release_request.notify_one();
        assert!(
            tokio::time::timeout(
                Duration::from_secs(3),
                wait_for_server(&mut server, Duration::from_secs(2))
            )
            .await
            .expect("server should finish after the handler drains")
            .expect("server should stop cleanly")
        );
        let response = tokio::time::timeout(Duration::from_secs(2), &mut request)
            .await
            .expect("HTTP response should finish")
            .expect("request task should not panic")
            .expect("HTTP request should succeed");
        assert!(response.starts_with(b"HTTP/1.1 200"));
    }

    #[test]
    fn test_session_cookie_sets_security_attributes_in_both_modes() {
        for secure in [true, false] {
            let cookie =
                session_cookie("session", "value".to_owned(), "/api/v1/session", secure).build();

            assert_eq!(cookie.http_only(), Some(true));
            assert_eq!(cookie.same_site(), Some(SameSite::Strict));
            assert_eq!(cookie.secure(), Some(secure));
            assert_eq!(cookie.path(), Some("/api/v1/session"));
        }
    }
}
