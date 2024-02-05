use axum::{extract::State, routing::post, Json, Router};
use axum_extra::extract::{cookie::Cookie, PrivateCookieJar};
use time::OffsetDateTime;
use tracing::{debug, error, info};

use crate::{
    error::ApiError,
    handlers::get_core_response,
    proto::{
        core_request, core_response, DeviceInfo, PasswordResetInitializeRequest,
        PasswordResetRequest, PasswordResetStartRequest, PasswordResetStartResponse,
    },
    server::{AppState, PASSWORD_RESET_COOKIE_NAME},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/request", post(request_password_reset))
        .route("/start", post(start_password_reset))
        .route("/reset", post(reset_password))
}

pub async fn request_password_reset(
    State(state): State<AppState>,
    device_info: Option<DeviceInfo>,
    Json(req): Json<PasswordResetInitializeRequest>,
) -> Result<(), ApiError> {
    info!("Starting password reset request for {}", req.email);

    let rx = state.grpc_server.send(
        Some(core_request::Payload::PasswordResetInit(req)),
        device_info,
    )?;
    let payload = get_core_response(rx).await?;
    if let core_response::Payload::Empty(()) = payload {
        Ok(())
    } else {
        error!("Received invalid gRPC response type: {payload:#?}");
        Err(ApiError::InvalidResponseType)
    }
}

pub async fn start_password_reset(
    State(state): State<AppState>,
    device_info: Option<DeviceInfo>,
    mut private_cookies: PrivateCookieJar,
    Json(req): Json<PasswordResetStartRequest>,
) -> Result<(PrivateCookieJar, Json<PasswordResetStartResponse>), ApiError> {
    info!("Starting password reset process");

    // clear session cookies if already populated
    if let Some(cookie) = private_cookies.get(PASSWORD_RESET_COOKIE_NAME) {
        debug!("Removing previous session cookie");
        private_cookies = private_cookies.remove(cookie);
    }

    let token = req.clone().token.clone();

    let rx = state.grpc_server.send(
        Some(core_request::Payload::PasswordResetStart(req)),
        device_info,
    )?;
    let payload = get_core_response(rx).await?;
    if let core_response::Payload::PasswordResetStart(response) = payload {
        // set session cookie
        let cookie = Cookie::build((PASSWORD_RESET_COOKIE_NAME, token))
            .expires(OffsetDateTime::from_unix_timestamp(response.deadline_timestamp).unwrap());

        Ok((private_cookies.add(cookie), Json(response)))
    } else {
        error!("Received invalid gRPC response type: {payload:#?}");
        Err(ApiError::InvalidResponseType)
    }
}

pub async fn reset_password(
    State(state): State<AppState>,
    device_info: Option<DeviceInfo>,
    mut private_cookies: PrivateCookieJar,
    Json(mut req): Json<PasswordResetRequest>,
) -> Result<PrivateCookieJar, ApiError> {
    info!("Resetting password");

    // set auth info
    req.token = private_cookies
        .get(PASSWORD_RESET_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string());

    let rx = state
        .grpc_server
        .send(Some(core_request::Payload::PasswordReset(req)), device_info)?;
    let payload = get_core_response(rx).await?;
    if let core_response::Payload::Empty(()) = payload {
        if let Some(cookie) = private_cookies.get(PASSWORD_RESET_COOKIE_NAME) {
            debug!("Password reset finished. Removing session cookie");
            private_cookies = private_cookies.remove(cookie);
        }
        Ok(private_cookies)
    } else {
        error!("Received invalid gRPC response type: {payload:#?}");
        Err(ApiError::InvalidResponseType)
    }
}
