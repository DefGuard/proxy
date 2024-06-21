use std::{
    fs::read_to_string,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use anyhow::Context;
use axum::{
    body::Body,
    extract::{ConnectInfo, FromRef},
    handler::HandlerWithoutStateExt,
    http::{Request, StatusCode},
    routing::get,
    serve, Json, Router,
};
use axum_extra::extract::cookie::Key;
use clap::crate_version;
use serde::Serialize;
use tokio::{net::TcpListener, task::JoinSet};
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::{self, TraceLayer},
};
use tracing::{info_span, Level};

use crate::{
    config::Config,
    error::ApiError,
    grpc::ProxyServer,
    handlers::{desktop_client_mfa, enrollment, password_reset},
    proto::proxy_server,
};

pub(crate) static ENROLLMENT_COOKIE_NAME: &str = "defguard_proxy";
pub(crate) static PASSWORD_RESET_COOKIE_NAME: &str = "defguard_proxy_password_reset";
const RATE_LIMITER_CLEANUP_PERIOD: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) grpc_server: ProxyServer,
    key: Key,
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
    "I'm alive!"
}

// Retrieves client address from the request. Uses either the left most x-forwarded-for
// header value, or socket address if the header is not present.
fn get_client_addr(request: &Request<Body>) -> String {
    request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.split(',').next())
        .map(|ip| ip.trim().to_string())
        .unwrap_or_else(|| {
            request
                .extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map_or("unknown".to_string(), |addr| addr.0.to_string())
        })
}

pub async fn run_server(config: Config) -> anyhow::Result<()> {
    info!("Starting Defguard proxy server");
    debug!("Using config: {config:?}");

    let mut tasks = JoinSet::new();

    // connect to upstream gRPC server
    let grpc_server = ProxyServer::new();

    // build application
    debug!("Setting up API server");
    let shared_state = AppState {
        grpc_server: grpc_server.clone(),
        // Generate secret key for encrypting cookies.
        key: Key::generate(),
    };

    // read gRPC TLS cert and key
    debug!("Configuring grpc certificates");
    let grpc_cert = config
        .grpc_cert
        .as_ref()
        .and_then(|path| read_to_string(path).ok());
    let grpc_key = config
        .grpc_key
        .as_ref()
        .and_then(|path| read_to_string(path).ok());
    debug!("Configured grpc certificates, cert: {grpc_cert:?}, key: {grpc_key:?}");

    // Start gRPC server.
    debug!("Spawning gRPC server");
    tasks.spawn(async move {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.grpc_port);
        info!("gRPC server is listening on {addr}");
        let mut builder = if let (Some(cert), Some(key)) = (grpc_cert, grpc_key) {
            let identity = Identity::from_pem(cert, key);
            Server::builder().tls_config(ServerTlsConfig::new().identity(identity))?
        } else {
            Server::builder()
        };
        builder
            .add_service(proxy_server::ProxyServer::new(grpc_server))
            .serve(addr)
            .await
            .context("Error running gRPC server")
    });

    // Serve static frontend files.
    debug!("Configuring API server routing");
    let serve_web_dir = ServeDir::new("web/dist").fallback(ServeFile::new("web/dist/index.html"));
    let serve_images =
        ServeDir::new("web/src/shared/images/svg").not_found_service(handle_404.into_service());

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
                tracing::debug!(
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
        .nest(
            "/api/v1",
            Router::new()
                .nest("/enrollment", enrollment::router())
                .nest("/password-reset", password_reset::router())
                .nest("/client-mfa", desktop_client_mfa::router())
                .route("/health", get(healthcheck))
                .route("/info", get(app_info)),
        )
        .nest_service("/svg", serve_images)
        .fallback_service(serve_web_dir)
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
        app = app.layer(GovernorLayer {
            config: conf.into(),
        });
    }
    debug!("Configured API server routing: {app:?}");

    // Start web server.
    debug!("Spawning API web server");
    tasks.spawn(async move {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.http_port);
        let listener = TcpListener::bind(&addr).await?;
        info!("API web server is listening on {addr}");
        serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .context("Error running HTTP server")
    });

    info!("Defguard proxy server initialization complete");
    while let Some(Ok(result)) = tasks.join_next().await {
        result?;
    }

    Ok(())
}
