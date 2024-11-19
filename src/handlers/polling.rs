use axum::{extract::State, Json};

use crate::{
    error::ApiError,
    handlers::get_core_response,
    http::AppState,
    proto::{core_request, core_response, InstanceInfoRequest, InstanceInfoResponse},
};

#[instrument(level = "debug", skip(state))]
pub(crate) async fn info(
    State(state): State<AppState>,
    Json(req): Json<InstanceInfoRequest>,
) -> Result<Json<InstanceInfoResponse>, ApiError> {
    debug!("Retrieving info for polling request");
    let rx = state
        .grpc_server
        .send(Some(core_request::Payload::InstanceInfo(req.clone())), None)?;
    let payload = get_core_response(rx).await?;

    if let core_response::Payload::InstanceInfo(response) = payload {
        info!("Retrieved info for polling request");
        Ok(Json(response))
    } else {
        error!("Received invalid gRPC response type: {payload:#?}");
        Err(ApiError::InvalidResponseType)
    }
}
