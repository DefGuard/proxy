use axum::{extract::State, routing::get, Json, Router};
use axum_extra::extract::{cookie::Cookie, PrivateCookieJar};
use time::OffsetDateTime;

use crate::{
    error::ApiError,
    handlers::get_core_response,
    http::{AppState, ENROLLMENT_COOKIE_NAME},
    proto::{
        core_request, core_response, ActivateUserRequest, DeviceConfigResponse, DeviceInfo,
        EnrollmentStartRequest, EnrollmentStartResponse, ExistingDevice, NewDevice,
    },
};

// pub fn router() -> Router<AppState> {
//     Router::new()
//         .route("/info", get(location_info))
// }

// #[instrument(level = "debug", skip(state))]
// pub async fn location_info(
//     State(state): State<AppState>,
//     private_cookies: PrivateCookieJar,
// ) -> Result<Json<DeviceConfigResponse>, ApiError> {
//     // let pubkey = req.pubkey.clone();
//     info!("Getting network info for device {pubkey}");

//     // set auth info
//     req.token = private_cookies
//         .get(ENROLLMENT_COOKIE_NAME)
//         .map(|cookie| cookie.value().to_string());

//     let rx = state
//         .grpc_server
//         .send(Some(core_request::Payload::ExistingDevice(req)), None)?;
//     let payload = get_core_response(rx).await?;
//     if let core_response::Payload::DeviceConfig(response) = payload {
//         info!("Got network info for device {pubkey}");
//         Ok(Json(response))
//     } else {
//         error!("Received invalid gRPC response type: {payload:#?}");
//         Err(ApiError::InvalidResponseType)
//     }
// }
