use axum::{extract::State, Json};
use axum_extra::extract::PrivateCookieJar;
use base64::{prelude::BASE64_STANDARD, Engine};
use serde::Deserialize;

use crate::{
    error::ApiError,
    handlers::get_core_response,
    http::{AppState, ENROLLMENT_COOKIE_NAME},
    proto::{core_request, core_response, DeviceInfo, RegisterMobileAuthRequest},
};

fn validate_register_request_data(data: &RegisterMobileAuth) -> Result<(), ApiError> {
    let decoded_device_key = BASE64_STANDARD.decode(&data.device_pub_key)?;
    let decoded_biometry_key = BASE64_STANDARD.decode(&data.auth_pub_key)?;
    if decoded_biometry_key.len() != 32 {
        return Err(ApiError::BadRequest("Invalid biometric public key.".into()));
    }
    if decoded_device_key.len() != 32 {
        return Err(ApiError::BadRequest("Invalid device public key.".into()));
    }
    Ok(())
}

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct RegisterMobileAuth {
    pub auth_pub_key: String,
    pub device_pub_key: String,
}

#[instrument(level = "debug", skip(state))]
pub(crate) async fn register_mobile_auth(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    private_cookies: PrivateCookieJar,
    Json(req): Json<RegisterMobileAuth>,
) -> Result<(), ApiError> {
    debug!("Register mobile auth started");
    // set auth info
    let Some(token) = private_cookies
        .get(ENROLLMENT_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string())
    else {
        return Err(ApiError::BadRequest("No token present".into()));
    };
    validate_register_request_data(&req)?;
    let send_data = RegisterMobileAuthRequest {
        token,
        auth_pub_key: req.auth_pub_key,
        device_pub_key: req.device_pub_key,
    };
    let rx = state.grpc_server.send(
        core_request::Payload::RegisterMobileAuth(send_data),
        device_info,
    )?;
    let payload = get_core_response(rx).await?;
    if let core_response::Payload::Empty(()) = payload {
        info!("Registered mobile device for auth");
        Ok(())
    } else {
        error!("Received invalid gRPC response type: {payload:#?}");
        Err(ApiError::InvalidResponseType)
    }
}
