use axum::{extract::State, routing::post, Json, Router};
use axum_extra::extract::{cookie::Cookie, PrivateCookieJar};
use time::OffsetDateTime;

use crate::{
    error::ApiError,
    handlers::get_core_response,
    http::{AppState, ENROLLMENT_COOKIE_NAME},
    proto::{
        core_request, core_response, ActivateUserRequest, DeviceConfigResponse, DeviceInfo,
        EnrollmentStartRequest, EnrollmentStartResponse, ExistingDevice, NewDevice,
    },
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start_enrollment_process))
        .route("/activate_user", post(activate_user))
        .route("/create_device", post(create_device))
        .route("/network_info", post(get_network_info))
}

#[instrument(level = "debug", skip(state))]
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

    let rx = state
        .grpc_server
        .send(Some(core_request::Payload::EnrollmentStart(req)), None)?;
    let payload = get_core_response(rx).await?;
    match payload {
        core_response::Payload::EnrollmentStart(response) => {
            info!(
                "Started enrollment process for user {:?} by admin {:?}",
                response.user, response.admin
            );
            // set session cookie
            let cookie = Cookie::build((ENROLLMENT_COOKIE_NAME, token))
                .expires(OffsetDateTime::from_unix_timestamp(response.deadline_timestamp).unwrap());

            Ok((private_cookies.add(cookie), Json(response)))
        }
        _ => {
            error!("Received invalid gRPC response type: {payload:?}");
            Err(ApiError::InvalidResponseType)
        }
    }
}

#[instrument(level = "debug", skip(state))]
pub async fn activate_user(
    State(state): State<AppState>,
    device_info: Option<DeviceInfo>,
    mut private_cookies: PrivateCookieJar,
    Json(mut req): Json<ActivateUserRequest>,
) -> Result<PrivateCookieJar, ApiError> {
    let phone = req.phone_number.clone();
    info!("Activating user - phone number {phone:?}");

    // set auth info
    req.token = private_cookies
        .get(ENROLLMENT_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string());

    let rx = state
        .grpc_server
        .send(Some(core_request::Payload::ActivateUser(req)), device_info)?;
    let payload = get_core_response(rx).await?;
    match payload {
        core_response::Payload::Empty(_) => {
            if let Some(cookie) = private_cookies.get(ENROLLMENT_COOKIE_NAME) {
                info!("Activated user - phone number {phone:?}");
                debug!("Enrollment finished. Removing session cookie");
                private_cookies = private_cookies.remove(cookie);
            }

            Ok(private_cookies)
        }
        _ => {
            error!("Received invalid gRPC response type: {payload:?}");
            Err(ApiError::InvalidResponseType)
        }
    }
}

#[instrument(level = "debug", skip(state))]
pub async fn create_device(
    State(state): State<AppState>,
    device_info: Option<DeviceInfo>,
    private_cookies: PrivateCookieJar,
    Json(mut req): Json<NewDevice>,
) -> Result<Json<DeviceConfigResponse>, ApiError> {
    let (name, pubkey) = (req.name.clone(), req.pubkey.clone());
    info!("Adding new device {name} {pubkey}");

    // set auth info
    req.token = private_cookies
        .get(ENROLLMENT_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string());

    let rx = state
        .grpc_server
        .send(Some(core_request::Payload::NewDevice(req)), device_info)?;
    let payload = get_core_response(rx).await?;
    match payload {
        core_response::Payload::DeviceConfig(response) => {
            info!("Added new device {name} {pubkey}");
            Ok(Json(response))
        }
        _ => {
            error!("Received invalid gRPC response type: {payload:?}");
            Err(ApiError::InvalidResponseType)
        }
    }
}

#[instrument(level = "debug", skip(state))]
pub async fn get_network_info(
    State(state): State<AppState>,
    private_cookies: PrivateCookieJar,
    Json(mut req): Json<ExistingDevice>,
) -> Result<Json<DeviceConfigResponse>, ApiError> {
    let pubkey = req.pubkey.clone();
    info!("Getting network info for device {pubkey}");

    // set auth info
    req.token = private_cookies
        .get(ENROLLMENT_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string());

    let rx = state
        .grpc_server
        .send(Some(core_request::Payload::ExistingDevice(req)), None)?;
    let payload = get_core_response(rx).await?;
    match payload {
        core_response::Payload::DeviceConfig(response) => {
            info!("Got network info for device {pubkey}");
            Ok(Json(response))
        }
        _ => {
            error!("Received invalid gRPC response type: {payload:?}");
            Err(ApiError::InvalidResponseType)
        }
    }
}
