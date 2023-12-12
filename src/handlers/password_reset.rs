use axum::{extract::State, headers::UserAgent, routing::post, Json, Router, TypedHeader};
use axum_client_ip::{InsecureClientIp, LeftmostXForwardedFor};
use tower_cookies::{cookie::time::OffsetDateTime, Cookie, Cookies};
use tracing::{debug, info};

use crate::{
    grpc::password_reset::proto::{
        PasswordResetRequest, PasswordResetStartRequest, PasswordResetStartResponse,
        PasswordResetVerifyRequest,
    },
    handlers::shared::{add_auth_header, add_device_info_header},
    server::{AppState, COOKIE_NAME, SECRET_KEY},
};

use super::ApiResult;

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
    Json(req): Json<PasswordResetStartRequest>,
) -> ApiResult<()> {
    info!(
        "Starting password reset request for {}",
        req.email.to_string()
    );

    let mut password_reset_client = state.password_reset_client.lock().await;
    let mut request = tonic::Request::new(req);

    let ip_address = forwarded_for_ip.map_or(insecure_ip, |v| v.0).to_string();
    add_device_info_header(&mut request, ip_address, user_agent)?;

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
    cookies: Cookies,
    Json(req): Json<PasswordResetVerifyRequest>,
) -> ApiResult<Json<PasswordResetStartResponse>> {
    info!("Starting password reset process");

    // clear session cookies if already populated
    let key = SECRET_KEY.get().unwrap();
    let private_cookies = cookies.private(key);
    if let Some(cookie) = private_cookies.get(COOKIE_NAME) {
        debug!("Removing previous session cookie");
        cookies.remove(cookie)
    };

    let token = req.clone().token.clone();

    let mut password_reset_client = state.password_reset_client.lock().await;
    let mut request = tonic::Request::new(req);

    let ip_address = forwarded_for_ip.map_or(insecure_ip, |v| v.0).to_string();
    add_device_info_header(&mut request, ip_address, user_agent)?;

    let response = password_reset_client
        .start_password_reset(request)
        .await?
        .into_inner();

    // TODO: consider changing name to that it does not collide with?
    // set session cookie
    let cookie = Cookie::build(COOKIE_NAME, token)
        .expires(OffsetDateTime::from_unix_timestamp(response.deadline_timestamp).unwrap())
        .finish();
    private_cookies.add(cookie);

    Ok(Json(response))
}

pub async fn reset_password(
    State(state): State<AppState>,
    forwarded_for_ip: Option<LeftmostXForwardedFor>,
    InsecureClientIp(insecure_ip): InsecureClientIp,
    user_agent: Option<TypedHeader<UserAgent>>,
    cookies: Cookies,
    Json(req): Json<PasswordResetRequest>,
) -> ApiResult<()> {
    info!("Resetting password");

    let ip_address = forwarded_for_ip.map_or(insecure_ip, |v| v.0).to_string();

    let mut password_reset_client = state.password_reset_client.lock().await;
    let mut request = tonic::Request::new(req);

    add_auth_header(cookies, &mut request)?;
    add_device_info_header(&mut request, ip_address, user_agent)?;

    password_reset_client.reset_password(request).await?;

    Ok(())
}
