use axum::{extract::State, routing::post, Json, Router};
use axum_extra::extract::{cookie::Cookie, PrivateCookieJar};
use time::OffsetDateTime;

use crate::{
    error::ApiError,
    handlers::get_core_response,
    http::{AppState, PASSWORD_RESET_COOKIE_NAME},
    proto::{
        core_request, core_response, DeviceInfo, PasswordResetInitializeRequest,
        PasswordResetRequest, PasswordResetStartRequest, PasswordResetStartResponse,
    },
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/request", post(request_password_reset))
        .route("/start", post(start_password_reset))
        .route("/reset", post(reset_password))
}

#[instrument(level = "debug", skip(state))]
async fn request_password_reset(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(req): Json<PasswordResetInitializeRequest>,
) -> Result<(), ApiError> {
    info!("Starting password reset request for {}", req.email);

    let rx = state.grpc_server.send(
        core_request::Payload::PasswordResetInit(req.clone()),
        device_info,
    )?;
    let payload = get_core_response(rx).await?;
    if let core_response::Payload::Empty(()) = payload {
        info!("Started password reset request for {}", req.email);
        Ok(())
    } else {
        error!("Received invalid gRPC response type");
        Err(ApiError::InvalidResponseType)
    }
}

#[instrument(level = "debug", skip(state))]
async fn start_password_reset(
    State(state): State<AppState>,
    device_info: DeviceInfo,
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

    let rx = state
        .grpc_server
        .send(core_request::Payload::PasswordResetStart(req), device_info)?;
    let payload = get_core_response(rx).await?;
    if let core_response::Payload::PasswordResetStart(response) = payload {
        // set session cookie
        let cookie = Cookie::build((PASSWORD_RESET_COOKIE_NAME, token))
            .expires(OffsetDateTime::from_unix_timestamp(response.deadline_timestamp).unwrap());

        info!("Started password reset process");
        Ok((private_cookies.add(cookie), Json(response)))
    } else {
        error!("Received invalid gRPC response type");
        Err(ApiError::InvalidResponseType)
    }
}

#[instrument(level = "debug", skip(state, req))]
async fn reset_password(
    State(state): State<AppState>,
    device_info: DeviceInfo,
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
        .send(core_request::Payload::PasswordReset(req), device_info)?;
    let payload = get_core_response(rx).await?;
    if let core_response::Payload::Empty(()) = payload {
        if let Some(cookie) = private_cookies.get(PASSWORD_RESET_COOKIE_NAME) {
            info!("Password reset finished. Removing session cookie");
            private_cookies = private_cookies.remove(cookie);
        }
        Ok(private_cookies)
    } else {
        error!("Received invalid gRPC response type");
        Err(ApiError::InvalidResponseType)
    }
}
