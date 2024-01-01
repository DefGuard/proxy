use axum::{extract::State, routing::post, Json, Router};
use axum_client_ip::{InsecureClientIp, LeftmostXForwardedFor};
use axum_extra::{
    extract::{cookie::Cookie, PrivateCookieJar},
    headers::UserAgent,
    TypedHeader,
};
use time::OffsetDateTime;
use tonic::Request;
use tracing::{debug, info};

use crate::{
    error::ApiError,
    grpc::password_reset::proto::{
        PasswordResetInitializeRequest, PasswordResetRequest, PasswordResetStartRequest,
        PasswordResetStartResponse,
    },
    handlers::shared::{add_auth_header, add_device_info_header},
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
    Json(req): Json<PasswordResetInitializeRequest>,
) -> Result<(), ApiError> {
    info!("Starting password reset request for {}", req.email);

    let mut password_reset_client = state.password_reset_client.lock().await;
    let mut request = Request::new(req);

    add_device_info_header(&mut request, forwarded_for_ip, insecure_ip, user_agent)?;

    password_reset_client
        .request_password_reset(request)
        .await?;

    Ok(())
}

pub async fn start_password_reset(
    State(state): State<AppState>,
    forwarded_for_ip: Option<LeftmostXForwardedFor>,
    InsecureClientIp(insecure_ip): InsecureClientIp,
    user_agent: Option<TypedHeader<UserAgent>>,
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

    let mut password_reset_client = state.password_reset_client.lock().await;
    let mut request = Request::new(req);

    add_device_info_header(&mut request, forwarded_for_ip, insecure_ip, user_agent)?;

    let response = password_reset_client
        .start_password_reset(request)
        .await?
        .into_inner();

    // set session cookie
    let cookie = Cookie::build((PASSWORD_RESET_COOKIE_NAME, token))
        .expires(OffsetDateTime::from_unix_timestamp(response.deadline_timestamp).unwrap());

    Ok((private_cookies.add(cookie), Json(response)))
}

pub async fn reset_password(
    State(state): State<AppState>,
    forwarded_for_ip: Option<LeftmostXForwardedFor>,
    InsecureClientIp(insecure_ip): InsecureClientIp,
    user_agent: Option<TypedHeader<UserAgent>>,
    mut private_cookies: PrivateCookieJar,
    Json(req): Json<PasswordResetRequest>,
) -> Result<PrivateCookieJar, ApiError> {
    info!("Resetting password");

    let mut password_reset_client = state.password_reset_client.lock().await;
    let mut request = Request::new(req);

    add_auth_header(&private_cookies, &mut request, PASSWORD_RESET_COOKIE_NAME)?;
    add_device_info_header(&mut request, forwarded_for_ip, insecure_ip, user_agent)?;

    password_reset_client.reset_password(request).await?;

    if let Some(cookie) = private_cookies.get(PASSWORD_RESET_COOKIE_NAME) {
        debug!("Password reset finished. Removing session cookie");
        private_cookies = private_cookies.remove(cookie);
    }

    Ok(private_cookies)
}
