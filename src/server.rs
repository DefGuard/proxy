use std::{
    fs::read_to_string,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use anyhow::Context;
use axum::{
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

pub async fn run_server(config: Config) -> anyhow::Result<()> {
    info!("Starting Defguard proxy server");

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
    let grpc_cert = config
        .grpc_cert
        .as_ref()
        .and_then(|path| read_to_string(path).ok());
    let grpc_key = config
        .grpc_key
        .as_ref()
        .and_then(|path| read_to_string(path).ok());

    // Start gRPC server.
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
            .context("Error running RPC server")
    });

    // Serve static frontend files.
    let serve_web_dir = ServeDir::new("web/dist").fallback(ServeFile::new("web/dist/index.html"));
    let serve_images =
        ServeDir::new("web/src/shared/images/svg").not_found_service(handle_404.into_service());
    let app = Router::new()
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
                .make_span_with(|request: &Request<_>| {
                    // extract client address
                    let addr = request
                        .extensions()
                        .get::<ConnectInfo<SocketAddr>>()
                        .map(|addr| addr.0.to_string())
                        .unwrap_or_else(|| "unknown".to_string());
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

    // Start web server.
    tasks.spawn(async move {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.http_port);
        let listener = TcpListener::bind(&addr).await?;
        info!("Web server is listening on {addr}");
        serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .context("Error running HTTP server")
    });

    while let Some(Ok(result)) = tasks.join_next().await {
        result?;
    }

    Ok(())
}
