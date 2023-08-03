use crate::{
    config::Config,
    grpc::{enrollment::proto::enrollment_service_client::EnrollmentServiceClient, setup_client},
    handlers::enrollment,
};
use anyhow::Context;
use axum::{
    handler::HandlerWithoutStateExt,
    http::{Request, StatusCode},
    Router,
};
use once_cell::sync::OnceCell;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tower_cookies::{CookieManagerLayer, Key};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::{self, TraceLayer},
};
use tracing::{debug, info, info_span, Level};

pub const COOKIE_NAME: &str = "defguard_proxy";
pub static SECRET_KEY: OnceCell<Key> = OnceCell::new();

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub client: Arc<Mutex<EnrollmentServiceClient<Channel>>>,
}

async fn handle_404() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found")
}

pub async fn run_server(config: Config) -> anyhow::Result<()> {
    info!("Starting Defguard proxy server");

    // connect to upstream gRPC server
    let client = setup_client(&config).context("Failed to setup gRPC client")?;

    // store port before moving config
    let http_port = config.http_port;

    // generate secret key for encrypting cookies
    SECRET_KEY.set(Key::generate()).ok();

    // build application
    debug!("Setting up API server");
    let shared_state = AppState {
        config: Arc::new(config),
        client: Arc::new(Mutex::new(client)),
    };
    // serving static frontend files
    let serve_web_dir =
        ServeDir::new("web/dist").not_found_service(ServeFile::new("web/dist/index.html"));
    let serve_images =
        ServeDir::new("web/src/shared/images/svg").not_found_service(handle_404.into_service());
    let app = Router::new()
        .nest(
            "/api/v1",
            Router::new().nest("/enrollment", enrollment::router()),
        )
        .nest_service("/svg", serve_images)
        .fallback_service(serve_web_dir)
        .with_state(shared_state)
        .layer(CookieManagerLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        path = ?request.uri(),
                        some_other_field = tracing::field::Empty,
                    )
                })
                .on_response(trace::DefaultOnResponse::new().level(Level::DEBUG)),
        );

    // run server
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), http_port);
    info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("Error running HTTP server")
}
