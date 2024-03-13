use crate::{
    error::ApiError,
    handlers::get_core_response,
    proto::{
        core_request, core_response, ClientMfaFinishRequest, ClientMfaFinishResponse,
        ClientMfaStartRequest, ClientMfaStartResponse, DeviceInfo,
    },
    server::AppState,
};
use axum::{extract::State, routing::post, Json, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start_client_mfa))
        .route("/finish", post(finish_client_mfa))
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
    match payload {
        core_response::Payload::ClientMfaStart(response) => Ok(Json(response)),
        _ => {
            error!("Received invalid gRPC response type: {payload:#?}");
            Err(ApiError::InvalidResponseType)
        }
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
    match payload {
        core_response::Payload::ClientMfaFinish(response) => Ok(Json(response)),
        _ => {
            error!("Received invalid gRPC response type: {payload:#?}");
            Err(ApiError::InvalidResponseType)
        }
    }
}
