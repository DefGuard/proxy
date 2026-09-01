use std::sync::atomic::Ordering;

use axum::{Json, Router, extract::State, routing::post};
use axum_extra::extract::PrivateCookieJar;
use time::OffsetDateTime;

use crate::{
    error::ApiError,
    handlers::get_core_response,
    http::{
        AppState, PASSWORD_RESET_COOKIE_NAME, PASSWORD_RESET_COOKIE_PATH, remove_session_cookie,
        session_cookie,
    },
    proto::{
        DeviceInfo, PasswordResetInitializeRequest, PasswordResetRequest,
        PasswordResetStartRequest, PasswordResetStartResponse, core_request, core_response,
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
    let payload = get_core_response(rx, None).await?;
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
    if private_cookies.get(PASSWORD_RESET_COOKIE_NAME).is_some() {
        debug!("Removing previous session cookie");
        private_cookies = remove_session_cookie(
            private_cookies,
            PASSWORD_RESET_COOKIE_NAME,
            PASSWORD_RESET_COOKIE_PATH,
            state.cookie_secure.load(Ordering::Relaxed),
        );
    }

    let token = req.clone().token.clone();

    let rx = state
        .grpc_server
        .send(core_request::Payload::PasswordResetStart(req), device_info)?;
    let payload = get_core_response(rx, None).await?;
    if let core_response::Payload::PasswordResetStart(response) = payload {
        // set session cookie
        let cookie = session_cookie(
            PASSWORD_RESET_COOKIE_NAME,
            token,
            PASSWORD_RESET_COOKIE_PATH,
            state.cookie_secure.load(Ordering::Relaxed),
        )
        .expires(
            OffsetDateTime::from_unix_timestamp(response.deadline_timestamp).map_err(|_| {
                ApiError::Unexpected("Invalid password reset deadline timestamp".into())
            })?,
        );

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
    let payload = get_core_response(rx, None).await?;
    if let core_response::Payload::Empty(()) = payload {
        if private_cookies.get(PASSWORD_RESET_COOKIE_NAME).is_some() {
            info!("Password reset finished. Removing session cookie");
            private_cookies = remove_session_cookie(
                private_cookies,
                PASSWORD_RESET_COOKIE_NAME,
                PASSWORD_RESET_COOKIE_PATH,
                state.cookie_secure.load(Ordering::Relaxed),
            );
        }
        Ok(private_cookies)
    } else {
        error!("Received invalid gRPC response type");
        Err(ApiError::InvalidResponseType)
    }
}
