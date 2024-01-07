use axum::{extract::State, routing::post, Json, Router};
use axum_client_ip::{InsecureClientIp, LeftmostXForwardedFor};
use axum_extra::{
    extract::{cookie::Cookie, PrivateCookieJar},
    headers::UserAgent,
    TypedHeader,
};
use time::OffsetDateTime;
use tracing::{debug, info};

use crate::{
    error::ApiError,
    proto::{
        proxy_request, proxy_response, PasswordResetInitializeRequest, PasswordResetRequest,
        PasswordResetStartRequest, PasswordResetStartResponse,
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
    forwarded_for_ip: Option<LeftmostXForwardedFor>,
    InsecureClientIp(insecure_ip): InsecureClientIp,
    user_agent: Option<TypedHeader<UserAgent>>,
    Json(mut req): Json<PasswordResetInitializeRequest>,
) -> Result<(), ApiError> {
    info!("Starting password reset request for {}", req.email);

    // let mut password_reset_client = state.password_reset_client.lock().await;
    // let mut request = Request::new(req);

    // set device info
    req.ip_address = forwarded_for_ip
        .map(|v| v.0)
        .or(Some(insecure_ip))
        .map(|v| v.to_string());
    req.user_agent = user_agent.map(|v| v.to_string());

    if let Some(rx) = state
        .grpc_server
        .send(Some(proxy_response::Payload::PasswordResetInit(req)))
    {
        if rx.await.is_ok() {
            return Ok(());
        }
    }

    Err(ApiError::Unexpected(
        "failed to communicate with Defguard core".into(),
    ))
}

pub async fn start_password_reset(
    State(state): State<AppState>,
    forwarded_for_ip: Option<LeftmostXForwardedFor>,
    InsecureClientIp(insecure_ip): InsecureClientIp,
    user_agent: Option<TypedHeader<UserAgent>>,
    mut private_cookies: PrivateCookieJar,
    Json(mut req): Json<PasswordResetStartRequest>,
) -> Result<(PrivateCookieJar, Json<PasswordResetStartResponse>), ApiError> {
    info!("Starting password reset process");

    // clear session cookies if already populated
    if let Some(cookie) = private_cookies.get(PASSWORD_RESET_COOKIE_NAME) {
        debug!("Removing previous session cookie");
        private_cookies = private_cookies.remove(cookie);
    }

    let token = req.clone().token.clone();

    // set device info
    req.ip_address = forwarded_for_ip
        .map(|v| v.0)
        .or(Some(insecure_ip))
        .map(|v| v.to_string());
    req.user_agent = user_agent.map(|v| v.to_string());

    if let Some(rx) = state
        .grpc_server
        .send(Some(proxy_response::Payload::PasswordResetStart(req)))
    {
        if let Ok(proxy_request::Payload::PasswordResetStart(response)) = rx.await {
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
    forwarded_for_ip: Option<LeftmostXForwardedFor>,
    InsecureClientIp(insecure_ip): InsecureClientIp,
    user_agent: Option<TypedHeader<UserAgent>>,
    mut private_cookies: PrivateCookieJar,
    Json(mut req): Json<PasswordResetRequest>,
) -> Result<PrivateCookieJar, ApiError> {
    info!("Resetting password");

    // set device info
    req.ip_address = forwarded_for_ip
        .map(|v| v.0)
        .or(Some(insecure_ip))
        .map(|v| v.to_string());
    req.user_agent = user_agent.map(|v| v.to_string());

    if let Some(rx) = state
        .grpc_server
        .send(Some(proxy_response::Payload::PasswordReset(req)))
    {
        if rx.await.is_ok() {
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
