use crate::{config::Config, handlers::enrollment};
use anyhow::Context;
use axum::{extract::MatchedPath, http::Request, Router};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tower_http::trace::{self, TraceLayer};
use tracing::{info, info_span, Level};

pub async fn run_server(config: Config) -> anyhow::Result<()> {
    // build application
    let app = Router::new()
        .nest(
            "/api/v1",
            Router::new().nest("/enrollment", enrollment::router()),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    // Log the matched route's path (with placeholders not filled in).
                    // Use request.uri() or OriginalUri if you want the real path.
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);

                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        matched_path,
                        some_other_field = tracing::field::Empty,
                    )
                })
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    // run server
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), config.http_port);
    info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("Error running HTTP server")
}
