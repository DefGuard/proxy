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
    grpc::enrollment::proto::{
        ActivateUserRequest, DeviceConfigResponse, EnrollmentStartRequest, EnrollmentStartResponse,
        ExistingDevice, NewDevice,
    },
    handlers::{
        shared::{add_auth_header, add_device_info_header},
        ApiResult,
    },
    server::{AppState, ENROLLMENT_COOKIE_NAME},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start_enrollment_process))
        .route("/activate_user", post(activate_user))
        .route("/create_device", post(create_device))
        .route("/network_info", post(get_network_info))
}

pub async fn start_enrollment_process(
    State(state): State<AppState>,
    mut private_cookies: PrivateCookieJar,
    Json(req): Json<EnrollmentStartRequest>,
) -> Result<(PrivateCookieJar, Json<EnrollmentStartResponse>), ApiError> {
    info!("Starting enrollment process");

    // clear session cookies if already populated
    if let Some(cookie) = private_cookies.get(ENROLLMENT_COOKIE_NAME) {
        debug!("Removing previous session cookie");
        private_cookies = private_cookies.remove(cookie);
    }

    let token = req.token.clone();

    let mut client = state.enrollment_client.lock().await;
    let response = client.start_enrollment(req).await?.into_inner();

    // set session cookie
    let cookie = Cookie::build((ENROLLMENT_COOKIE_NAME, token))
        .expires(OffsetDateTime::from_unix_timestamp(response.deadline_timestamp).unwrap());

    Ok((private_cookies.add(cookie), Json(response)))
}

pub async fn activate_user(
    State(state): State<AppState>,
    forwarded_for_ip: Option<LeftmostXForwardedFor>,
    InsecureClientIp(insecure_ip): InsecureClientIp,
    user_agent: Option<TypedHeader<UserAgent>>,
    mut private_cookies: PrivateCookieJar,
    Json(req): Json<ActivateUserRequest>,
) -> ApiResult<PrivateCookieJar> {
    info!("Activating user");

    let mut client = state.enrollment_client.lock().await;
    let mut request = Request::new(req);
    add_auth_header(&private_cookies, &mut request, ENROLLMENT_COOKIE_NAME)?;
    add_device_info_header(&mut request, forwarded_for_ip, insecure_ip, user_agent)?;
    client.activate_user(request).await?;

    if let Some(cookie) = private_cookies.get(ENROLLMENT_COOKIE_NAME) {
        debug!("Enrollment finished. Removing session cookie");
        private_cookies = private_cookies.remove(cookie);
    }

    Ok(private_cookies)
}

pub async fn create_device(
    State(state): State<AppState>,
    forwarded_for_ip: Option<LeftmostXForwardedFor>,
    InsecureClientIp(insecure_ip): InsecureClientIp,
    user_agent: Option<TypedHeader<UserAgent>>,
    private_cookies: PrivateCookieJar,
    Json(req): Json<NewDevice>,
) -> ApiResult<Json<DeviceConfigResponse>> {
    info!("Adding new device");

    let mut client = state.enrollment_client.lock().await;
    let mut request = Request::new(req);
    add_auth_header(&private_cookies, &mut request, ENROLLMENT_COOKIE_NAME)?;
    add_device_info_header(&mut request, forwarded_for_ip, insecure_ip, user_agent)?;
    let response = client.create_device(request).await?;

    Ok(Json(response.into_inner()))
}
pub async fn get_network_info(
    State(state): State<AppState>,
    private_cookies: PrivateCookieJar,
    Json(req): Json<ExistingDevice>,
) -> ApiResult<Json<DeviceConfigResponse>> {
    info!("Getting network info");

    let mut client = state.enrollment_client.lock().await;
    let mut request = Request::new(req);
    add_auth_header(&private_cookies, &mut request, ENROLLMENT_COOKIE_NAME)?;
    let response = client.get_network_info(request).await?;

    Ok(Json(response.into_inner()))
}
