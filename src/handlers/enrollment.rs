use axum::{extract::State, routing::post, Json, Router};
use axum_extra::extract::{cookie::Cookie, PrivateCookieJar};
use time::OffsetDateTime;
use tracing::{debug, info};

use crate::{
    error::ApiError,
    proto::{
        core_request, core_response, ActivateUserRequest, DeviceConfigResponse, DeviceInfo,
        EnrollmentStartRequest, EnrollmentStartResponse, ExistingDevice, NewDevice,
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

    if let Some(rx) = state
        .grpc_server
        .send(Some(core_request::Payload::EnrollmentStart(req)), None)
    {
        if let Ok(core_response::Payload::EnrollmentStart(response)) = rx.await {
            // set session cookie
            let cookie = Cookie::build((ENROLLMENT_COOKIE_NAME, token))
                .expires(OffsetDateTime::from_unix_timestamp(response.deadline_timestamp).unwrap());

            return Ok((private_cookies.add(cookie), Json(response)));
        }
    }

    Err(ApiError::Unexpected(
        "failed to communicate with Defguard core".into(),
    ))
}

pub async fn activate_user(
    State(state): State<AppState>,
    device_info: Option<DeviceInfo>,
    mut private_cookies: PrivateCookieJar,
    Json(mut req): Json<ActivateUserRequest>,
) -> Result<PrivateCookieJar, ApiError> {
    info!("Activating user");

    // set auth info
    req.token = private_cookies
        .get(ENROLLMENT_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string());

    if let Some(rx) = state
        .grpc_server
        .send(Some(core_request::Payload::ActivateUser(req)), device_info)
    {
        if let Ok(core_response::Payload::Empty(())) = rx.await {
            if let Some(cookie) = private_cookies.get(ENROLLMENT_COOKIE_NAME) {
                debug!("Enrollment finished. Removing session cookie");
                private_cookies = private_cookies.remove(cookie);
            }

            return Ok(private_cookies);
        }
    }

    Err(ApiError::Unexpected(
        "failed to communicate with Defguard core".into(),
    ))
}

pub async fn create_device(
    State(state): State<AppState>,
    device_info: Option<DeviceInfo>,
    private_cookies: PrivateCookieJar,
    Json(mut req): Json<NewDevice>,
) -> Result<Json<DeviceConfigResponse>, ApiError> {
    info!("Adding new device");

    // set auth info
    req.token = private_cookies
        .get(ENROLLMENT_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string());

    if let Some(rx) = state
        .grpc_server
        .send(Some(core_request::Payload::NewDevice(req)), device_info)
    {
        if let Ok(core_response::Payload::DeviceConfig(response)) = rx.await {
            return Ok(Json(response));
        }
    }

    Err(ApiError::Unexpected(
        "failed to communicate with Defguard core".into(),
    ))
}

pub async fn get_network_info(
    State(state): State<AppState>,
    private_cookies: PrivateCookieJar,
    Json(mut req): Json<ExistingDevice>,
) -> Result<Json<DeviceConfigResponse>, ApiError> {
    info!("Getting network info");

    // set auth info
    req.token = private_cookies
        .get(ENROLLMENT_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string());

    if let Some(rx) = state
        .grpc_server
        .send(Some(core_request::Payload::ExistingDevice(req)), None)
    {
        if let Ok(core_response::Payload::DeviceConfig(response)) = rx.await {
            return Ok(Json(response));
        }
    }

    Err(ApiError::Unexpected(
        "failed to communicate with Defguard core".into(),
    ))
}
