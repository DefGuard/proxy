use axum::{extract::State, routing::post, Json, Router};
use axum_extra::extract::{cookie::Cookie, PrivateCookieJar};
use time::OffsetDateTime;
use tracing::{debug, info};

use crate::{
    error::ApiError,
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

    if let Some(rx) = state.grpc_server.send(
        Some(core_request::Payload::PasswordResetInit(req)),
        device_info,
    ) {
        if let Ok(core_response::Payload::Empty(())) = rx.await {
            return Ok(());
        }
    }

    Err(ApiError::Unexpected(
        "failed to communicate with Defguard core".into(),
    ))
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

    if let Some(rx) = state.grpc_server.send(
        Some(core_request::Payload::PasswordResetStart(req)),
        device_info,
    ) {
        if let Ok(core_response::Payload::PasswordResetStart(response)) = rx.await {
            // set session cookie
            let cookie = Cookie::build((PASSWORD_RESET_COOKIE_NAME, token))
                .expires(OffsetDateTime::from_unix_timestamp(response.deadline_timestamp).unwrap());

            return Ok((private_cookies.add(cookie), Json(response)));
        }
    }

    Err(ApiError::Unexpected(
        "failed to communicate with Defguard core".into(),
    ))
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

    if let Some(rx) = state
        .grpc_server
        .send(Some(core_request::Payload::PasswordReset(req)), device_info)
    {
        if let Ok(core_response::Payload::Empty(())) = rx.await {
            if let Some(cookie) = private_cookies.get(PASSWORD_RESET_COOKIE_NAME) {
                debug!("Password reset finished. Removing session cookie");
                private_cookies = private_cookies.remove(cookie);
            }
            return Ok(private_cookies);
        }
    }

    Err(ApiError::Unexpected(
        "failed to communicate with Defguard core".into(),
    ))
}
