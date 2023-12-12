use crate::handlers::shared::{add_auth_header, add_device_info_header, create_request};
use crate::server::{ENROLLMENT_COOKIE_NAME, SECRET_KEY};
use crate::{
    grpc::enrollment::proto::{
        ActivateUserRequest, DeviceConfigResponse, EnrollmentStartRequest, EnrollmentStartResponse,
        ExistingDevice, NewDevice,
    },
    handlers::ApiResult,
    server::AppState,
};
use axum::{extract::State, headers::UserAgent, routing::post, Json, Router, TypedHeader};
use axum_client_ip::{InsecureClientIp, LeftmostXForwardedFor};
use tower_cookies::{cookie::time::OffsetDateTime, Cookie, Cookies};
use tracing::{debug, info};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start_enrollment_process))
        .route("/activate_user", post(activate_user))
        .route("/create_device", post(create_device))
        .route("/network_info", post(get_network_info))
}

pub async fn start_enrollment_process(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(req): Json<EnrollmentStartRequest>,
) -> ApiResult<Json<EnrollmentStartResponse>> {
    info!("Starting enrollment process");

    // clear session cookies if already populated
    let key = SECRET_KEY.get().unwrap();
    let private_cookies = cookies.private(key);
    if let Some(cookie) = private_cookies.get(ENROLLMENT_COOKIE_NAME) {
        debug!("Removing previous session cookie");
        cookies.remove(cookie)
    };

    let token = req.token.clone();

    let mut client = state.client.lock().await;
    let response = client.start_enrollment(req).await?.into_inner();

    // set session cookie
    let cookie = Cookie::build(ENROLLMENT_COOKIE_NAME, token)
        .expires(OffsetDateTime::from_unix_timestamp(response.deadline_timestamp).unwrap())
        .finish();
    private_cookies.add(cookie);

    Ok(Json(response))
}

pub async fn activate_user(
    State(state): State<AppState>,
    forwarded_for_ip: Option<LeftmostXForwardedFor>,
    InsecureClientIp(insecure_ip): InsecureClientIp,
    user_agent: Option<TypedHeader<UserAgent>>,
    cookies: Cookies,
    Json(req): Json<ActivateUserRequest>,
) -> ApiResult<()> {
    info!("Activating user");

    let mut client = state.client.lock().await;
    let mut request = create_request(req);
    add_auth_header(cookies.clone(), &mut request, ENROLLMENT_COOKIE_NAME)?;
    add_device_info_header(&mut request, forwarded_for_ip, insecure_ip, user_agent)?;
    client.activate_user(request).await?;

    let key = SECRET_KEY.get().unwrap();
    let private_cookies = cookies.private(key);
    if let Some(cookie) = private_cookies.get(ENROLLMENT_COOKIE_NAME) {
        debug!("Enrollment finished. Removing session cookie");
        cookies.remove(cookie)
    };

    Ok(())
}

pub async fn create_device(
    State(state): State<AppState>,
    forwarded_for_ip: Option<LeftmostXForwardedFor>,
    InsecureClientIp(insecure_ip): InsecureClientIp,
    user_agent: Option<TypedHeader<UserAgent>>,
    cookies: Cookies,
    Json(req): Json<NewDevice>,
) -> ApiResult<Json<DeviceConfigResponse>> {
    info!("Adding new device");

    let mut client = state.client.lock().await;
    let mut request = create_request(req);
    add_auth_header(cookies, &mut request, ENROLLMENT_COOKIE_NAME)?;
    add_device_info_header(&mut request, forwarded_for_ip, insecure_ip, user_agent)?;
    let response = client.create_device(request).await?;

    Ok(Json(response.into_inner()))
}
pub async fn get_network_info(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(req): Json<ExistingDevice>,
) -> ApiResult<Json<DeviceConfigResponse>> {
    info!("Getting network info");

    let mut client = state.client.lock().await;
    let mut request = create_request(req);
    add_auth_header(cookies, &mut request, ENROLLMENT_COOKIE_NAME)?;
    let response = client.get_network_info(request).await?;

    Ok(Json(response.into_inner()))
}
