use std::time::Duration;

use axum::{
    error_handling::HandleErrorLayer, extract::State, http::StatusCode, routing::post, BoxError,
    Json, Router,
};
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};
use tracing::{error, info};

use crate::{
    error::ApiError,
    handlers::get_core_response,
    proto::{
        core_request, core_response, ClientMfaFinishRequest, ClientMfaFinishResponse,
        ClientMfaStartRequest, ClientMfaStartResponse, DeviceInfo,
    },
    server::AppState,
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start_client_mfa))
        .route("/finish", post(finish_client_mfa))
        // `RateLimitLayer` does not implement `Clone`.
        // See: https://github.com/tokio-rs/axum/discussions/987
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled error: {err}"),
                    )
                }))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(5, Duration::from_secs(10))),
        )
}

async fn start_client_mfa(
    State(state): State<AppState>,
    device_info: Option<DeviceInfo>,
    Json(req): Json<ClientMfaStartRequest>,
) -> Result<Json<ClientMfaStartResponse>, ApiError> {
    info!("Starting desktop client authorization");
    let rx = state.grpc_server.send(
        Some(core_request::Payload::ClientMfaStart(req)),
        device_info,
    )?;
    let payload = get_core_response(rx).await?;
    if let core_response::Payload::ClientMfaStart(response) = payload {
        Ok(Json(response))
    } else {
        error!("Received invalid gRPC response type: {payload:#?}");
        Err(ApiError::InvalidResponseType)
    }
}

async fn finish_client_mfa(
    State(state): State<AppState>,
    device_info: Option<DeviceInfo>,
    Json(req): Json<ClientMfaFinishRequest>,
) -> Result<Json<ClientMfaFinishResponse>, ApiError> {
    info!("Finishing desktop client authorization");
    let rx = state.grpc_server.send(
        Some(core_request::Payload::ClientMfaFinish(req)),
        device_info,
    )?;
    let payload = get_core_response(rx).await?;
    if let core_response::Payload::ClientMfaFinish(response) = payload {
        Ok(Json(response))
    } else {
        error!("Received invalid gRPC response type: {payload:#?}");
        Err(ApiError::InvalidResponseType)
    }
}
