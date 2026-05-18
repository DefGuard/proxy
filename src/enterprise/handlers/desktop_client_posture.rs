use axum::{Json, Router, extract::State, routing::post};

use crate::{
    error::ApiError,
    handlers::get_core_response,
    http::AppState,
    proto::{
        DeviceInfo, DevicePostureCheckRequest, DevicePostureCheckResponse, core_request,
        core_response,
    },
};

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/connect", post(connect_with_posture_check))
}

#[instrument(level = "debug", skip(state, req))]
async fn connect_with_posture_check(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(req): Json<DevicePostureCheckRequest>,
) -> Result<Json<DevicePostureCheckResponse>, ApiError> {
    info!("Starting desktop client posture check {req:?}");
    let rx = state.grpc_server.send(
        core_request::Payload::DevicePostureCheck(req.clone()),
        device_info,
    )?;
    let payload = get_core_response(rx, None).await?;

    match payload {
        core_response::Payload::DevicePostureCheck(response) => {
            info!("Desktop client passed posture check {req:?}");
            Ok(Json(response))
        }
        core_response::Payload::DevicePostureRejected(response) => {
            info!("Desktop client failed posture check {response:?}");
            Err(ApiError::PostureRejected(response.failed_posture_checks))
        }
        _ => {
            error!("Received invalid gRPC response type");
            Err(ApiError::InvalidResponseType)
        }
    }
}
