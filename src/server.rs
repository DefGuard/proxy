use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use anyhow::Context;
use axum::{
    extract::FromRef,
    handler::HandlerWithoutStateExt,
    http::{Request, StatusCode},
    routing::get,
    serve, Json, Router,
};
use axum_extra::extract::cookie::Key;
use clap::crate_version;
use serde::Serialize;
use tokio::{net::TcpListener, sync::Mutex};
use tonic::transport::Channel;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::{self, TraceLayer},
};
use tracing::{debug, info, info_span, Level};

use crate::{
    config::Config,
    grpc::{
        enrollment::proto::enrollment_service_client::EnrollmentServiceClient,
        password_reset::proto::password_reset_service_client::PasswordResetServiceClient,
        setup_client, setup_password_reset_client,
    },
    handlers::{enrollment, password_reset, ApiResult},
};

pub static ENROLLMENT_COOKIE_NAME: &str = "defguard_proxy";
pub static PASSWORD_RESET_COOKIE_NAME: &str = "defguard_proxy_password_reset";

#[derive(Clone)]
pub(crate) struct AppState {
    // pub config: Arc<Config>,
    pub enrollment_client: Arc<Mutex<EnrollmentServiceClient<Channel>>>,
    pub password_reset_client: Arc<Mutex<PasswordResetServiceClient<Channel>>>,
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
struct AppInfo {
    version: String,
}

async fn app_info() -> ApiResult<Json<AppInfo>> {
    let version = crate_version!().to_string();
    Ok(Json(AppInfo { version }))
}

async fn healthcheck() -> &'static str {
    "I'm alive!"
}

pub async fn run_server(config: Config) -> anyhow::Result<()> {
    info!("Starting Defguard proxy server");

    // connect to upstream gRPC server
    let client = setup_client(&config).context("Failed to setup gRPC client")?;
    let password_reset_client = setup_password_reset_client(&config)
        .context("Failed to setup password reset gRPC client")?;

    // store port before moving config
    let http_port = config.http_port;

    // build application
    debug!("Setting up API server");
    let shared_state = AppState {
        // config: Arc::new(config),
        enrollment_client: Arc::new(Mutex::new(client)),
        password_reset_client: Arc::new(Mutex::new(password_reset_client)),
        // generate secret key for encrypting cookies
        key: Key::generate(),
    };
    // serving static frontend files
    let serve_web_dir = ServeDir::new("web/dist").fallback(ServeFile::new("web/dist/index.html"));
    let serve_images =
        ServeDir::new("web/src/shared/images/svg").not_found_service(handle_404.into_service());
    let app = Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .nest("/enrollment", enrollment::router())
                .nest("/password-reset", password_reset::router())
                .route("/health", get(healthcheck))
                .route("/info", get(app_info)),
        )
        .nest_service("/svg", serve_images)
        .fallback_service(serve_web_dir)
        .with_state(shared_state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        path = ?request.uri(),
                    )
                })
                .on_response(trace::DefaultOnResponse::new().level(Level::DEBUG)),
        );

    // run server
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), http_port);
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on {addr}");
    serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .context("Error running HTTP server")
}
