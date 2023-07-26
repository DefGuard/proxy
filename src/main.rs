use axum::{routing::{get, post}, Router};
use tower_http::trace::{self, TraceLayer};
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use defguard_proxy::handlers::enrollment;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // build application
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest(
            "/api/v1",
            Router::new().nest(
                "/enrollment",
                Router::new()
                    .route("/start", post(enrollment::start_enrollment_process))
                    .route("/activate_user", post(enrollment::activate_user))
                    .route("/create_device", post(enrollment::create_device)),
            ),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    // run server
    info!("Starting server");
    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
