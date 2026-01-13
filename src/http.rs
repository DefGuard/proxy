use std::{
    collections::HashMap,
    fs::read_to_string,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path,
    sync::{atomic::Ordering, Arc, LazyLock},
    time::Duration,
};

use anyhow::Context;
use axum::{
    body::Body,
    extract::{ConnectInfo, FromRef, State},
    http::{header::HeaderValue, Request, Response, StatusCode},
    middleware::{self, Next},
    routing::{get, post},
    serve, Json, Router,
};
use axum_extra::extract::cookie::Key;
use clap::crate_version;
use defguard_version::{server::DefguardVersionLayer, Version};
use serde::Serialize;
use tokio::{
    net::TcpListener,
    sync::{mpsc, oneshot, Mutex},
    task::JoinSet,
};
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tower_http::trace::{self, TraceLayer};
use tracing::{info_span, Level};
use url::Url;

use crate::{
    assets::{index, web_asset},
    config::Config,
    enterprise::handlers::openid_login::{self, FlowType},
    error::ApiError,
    grpc::{Configuration, ProxyServer},
    handlers::{desktop_client_mfa, enrollment, password_reset, polling},
    setup::ProxySetupServer,
    CommsChannel, VERSION,
};

pub(crate) static ENROLLMENT_COOKIE_NAME: &str = "defguard_proxy";
pub(crate) static PASSWORD_RESET_COOKIE_NAME: &str = "defguard_proxy_password_reset";
const DEFGUARD_CORE_CONNECTED_HEADER: &str = "defguard-core-connected";
const DEFGUARD_CORE_VERSION_HEADER: &str = "defguard-core-version";
const RATE_LIMITER_CLEANUP_PERIOD: Duration = Duration::from_secs(60);
const X_FORWARDED_FOR: &str = "x-forwarded-for";
const X_POWERED_BY: &str = "x-powered-by";
const GRPC_CERT_NAME: &str = "proxy_grpc_cert.pem";
const GRPC_KEY_NAME: &str = "proxy_grpc_key.pem";

pub static GRPC_SERVER_RESTART_CHANNEL: LazyLock<CommsChannel<()>> = LazyLock::new(|| {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    (Arc::new(Mutex::new(tx)), Arc::new(Mutex::new(rx)))
});

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) grpc_server: ProxyServer,
    pub(crate) remote_mfa_sessions:
        Arc<tokio::sync::Mutex<HashMap<String, oneshot::Sender<String>>>>,
    key: Key,
    url: Url,
}

impl AppState {
    /// Returns configured URL with "auth/callback" appended to the path.
    #[must_use]
    pub(crate) fn callback_url(&self, flow_type: &FlowType) -> Url {
        let mut url = self.url.clone();
        // Append "/openid/callback" to the URL.
        if let Ok(mut path_segments) = url.path_segments_mut() {
            match flow_type {
                FlowType::Enrollment => path_segments.extend(&["openid", "callback"]),
                FlowType::Mfa => path_segments.extend(&["openid", "mfa", "callback"]),
            };
        }
        url
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

async fn handle_404() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found")
}

#[derive(Serialize)]
struct AppInfo<'a> {
    version: &'a str,
}

async fn app_info<'a>() -> Result<Json<AppInfo<'a>>, ApiError> {
    let version = crate_version!();
    Ok(Json(AppInfo { version }))
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

    if let Some(core_version) = app_state.grpc_server.core_version.lock().unwrap().as_ref() {
        if let Ok(core_version_header) = HeaderValue::from_str(&core_version.to_string()) {
            response
                .headers_mut()
                .insert(DEFGUARD_CORE_VERSION_HEADER, core_version_header);
        }
    }

    let core_connected = app_state.grpc_server.connected.load(Ordering::Relaxed);
    let core_connected_header = if core_connected {
        HeaderValue::from_static("true")
    } else {
        HeaderValue::from_static("false")
    };

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

pub async fn run_server(config: Config) -> anyhow::Result<()> {
    info!("Starting Defguard Proxy server");
    debug!("Using config: {config:?}");

    let mut tasks = JoinSet::new();

    // Prepare the channel for gRPC -> http server communication.
    // The channel sends private cookies key once core connects to gRPC.
    let (tx, mut rx) = mpsc::unbounded_channel::<Key>();

    // connect to upstream gRPC server
    let grpc_server = ProxyServer::new(tx);

    let server_clone = grpc_server.clone();

    let setup_server = ProxySetupServer::new();

    // Start gRPC server.
    // TODO: Wait with spawning the HTTP server until gRPC server is ready.
    debug!("Spawning gRPC server");
    tasks.spawn(async move {
        let cert_dir = Path::new(&config.cert_dir);
        if !cert_dir.exists() {
            tokio::fs::create_dir_all(cert_dir).await?;
        }

        loop {
            let server_to_run = server_clone.clone();

            if let (Some(cert), Some(key)) = (
                read_to_string(cert_dir.join(GRPC_CERT_NAME)).ok(),
                read_to_string(cert_dir.join(GRPC_KEY_NAME)).ok(),
            ) {
                info!(
                    "Using existing gRPC TLS certificates from {}",
                    cert_dir.display()
                );
                server_clone.set_tls_config(cert, key)?;
            } else if !server_clone.setup_completed() {
                // Only attempt setup if not already configured
                info!(
                    "No gRPC TLS certificates found at {}, new certificates will be generated",
                    cert_dir.display()
                );
                let configuration = setup_server
                    .await_setup(SocketAddr::new(
                        config
                            .grpc_bind_address
                            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
                        config.grpc_port,
                    ))
                    .await?;
                info!("Generated new gRPC TLS certificates and signed by Defguard Core");

                let Configuration {
                    grpc_cert_pem,
                    grpc_key_pem,
                    ..
                } = &configuration;

                let cert_path = cert_dir.join(GRPC_CERT_NAME);
                let key_path = cert_dir.join(GRPC_KEY_NAME);
                tokio::fs::write(&cert_path, grpc_cert_pem).await?;
                tokio::fs::write(&key_path, grpc_key_pem).await?;

                server_to_run.configure(configuration);
            } else {
                info!("Proxy already configured, skipping setup phase");
            }

            let addr = SocketAddr::new(
                config
                    .grpc_bind_address
                    .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
                config.grpc_port,
            );

            if let Err(e) = server_to_run.run(addr).await {
                error!("gRPC server error: {e:?}, restarting...");
            }
        }
    });

    // Wait for core to connect to gRPC and send private cookies encryption key.
    let Some(key) = rx.recv().await else {
        return Err(anyhow::Error::msg("http channel closed"));
    };

    // build application
    debug!("Setting up API server");
    let shared_state = AppState {
        key,
        grpc_server: grpc_server,
        remote_mfa_sessions: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        url: config.url.clone(),
    };

    // Setup tower_governor rate-limiter
    debug!(
        "Configuring rate limiter, per_second: {}, burst: {}",
        config.rate_limit_per_second, config.rate_limit_burst
    );
    let governor_conf = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(config.rate_limit_per_second)
        .burst_size(config.rate_limit_burst)
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
            config.rate_limit_per_second, config.rate_limit_burst
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
        let addr = SocketAddr::new(
            config
                .http_bind_address
                .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            config.http_port,
        );
        let listener = TcpListener::bind(&addr).await?;
        info!("API web server is listening on {addr}");
        serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .context("Error running HTTP server")
    });

    // TODO: Possibly switch to using select! macro
    info!("Defguard Proxy HTTP server initialization complete");
    while let Some(Ok(result)) = tasks.join_next().await {
        result?;
    }

    Ok(())
}
