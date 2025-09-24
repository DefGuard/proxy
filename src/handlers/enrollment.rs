use axum::{extract::State, routing::post, Json, Router};
use axum_extra::extract::{cookie::Cookie, PrivateCookieJar};
use time::OffsetDateTime;

use super::register_mfa::router as register_mfa_router;

use crate::{
    error::ApiError,
    handlers::{get_core_response, mobile_client::register_mobile_auth},
    http::{AppState, ENROLLMENT_COOKIE_NAME},
    proto::{
        core_request, core_response, ActivateUserRequest, DeviceConfigResponse, DeviceInfo,
        EnrollmentStartRequest, EnrollmentStartResponse, ExistingDevice, NewDevice,
    },
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .nest("/register-mfa", register_mfa_router())
        .route("/start", post(start_enrollment_process))
        .route("/activate_user", post(activate_user))
        .route("/create_device", post(create_device))
        .route("/network_info", post(get_network_info))
        .route("/register_mobile", post(register_mobile_auth))
}

#[instrument(level = "debug", skip(state))]
async fn start_enrollment_process(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    mut private_cookies: PrivateCookieJar,
    Json(req): Json<EnrollmentStartRequest>,
) -> Result<(PrivateCookieJar, Json<EnrollmentStartResponse>), ApiError> {
    info!("Starting enrollment process");

    // clear session cookies if already populated
    debug!("Trying to remove previous session cookie if it still exists.");
    if let Some(cookie) = private_cookies.get(ENROLLMENT_COOKIE_NAME) {
        debug!("Removing previous session cookie");
        private_cookies = private_cookies.remove(cookie);
    }

    let token = req.token.clone();

    debug!("Sending the enrollment process request to core service.");
    let rx = state
        .grpc_server
        .send(core_request::Payload::EnrollmentStart(req), device_info)?;
    let payload = get_core_response(rx).await?;
    debug!("Receving payload from the core service. Try to set private cookie for starting enrollment process.");
    if let core_response::Payload::EnrollmentStart(response) = payload {
        info!(
            "Started enrollment process for user {:?} by admin {:?}",
            response.user, response.admin
        );
        // set session cookie
        let cookie = Cookie::build((ENROLLMENT_COOKIE_NAME, token))
            .expires(OffsetDateTime::from_unix_timestamp(response.deadline_timestamp).unwrap());

        Ok((private_cookies.add(cookie), Json(response)))
    } else {
        error!("Received invalid gRPC response type");
        Err(ApiError::InvalidResponseType)
    }
}

#[instrument(level = "debug", skip(state, req))]
async fn activate_user(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    mut private_cookies: PrivateCookieJar,
    Json(mut req): Json<ActivateUserRequest>,
) -> Result<PrivateCookieJar, ApiError> {
    let phone = req.phone_number.clone();
    info!("Activating user - phone number {phone:?}");

    // set auth info
    debug!("Set private cookie for the request.");
    req.token = private_cookies
        .get(ENROLLMENT_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string());

    debug!("Sending the activate user request to core service.");
    let rx = state
        .grpc_server
        .send(core_request::Payload::ActivateUser(req), device_info)?;
    let payload = get_core_response(rx).await?;
    debug!("Receiving payload from the core service. Trying to remove private cookie...");
    if let core_response::Payload::Empty(()) = payload {
        info!("Activated user - phone number {phone:?}");
        if let Some(cookie) = private_cookies.get(ENROLLMENT_COOKIE_NAME) {
            debug!("Enrollment finished. Removing session cookie");
            private_cookies = private_cookies.remove(cookie);
        }
        Ok(private_cookies)
    } else {
        error!("Received invalid gRPC response type");
        Err(ApiError::InvalidResponseType)
    }
}

#[instrument(level = "debug", skip(state, req))]
async fn create_device(
    State(state): State<AppState>,
    device_info: DeviceInfo,
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
        .send(core_request::Payload::NewDevice(req), device_info)?;
    let payload = get_core_response(rx).await?;
    if let core_response::Payload::DeviceConfig(response) = payload {
        info!("Added new device {name} {pubkey}");
        Ok(Json(response))
    } else {
        error!("Received invalid gRPC response type");
        Err(ApiError::InvalidResponseType)
    }
}

#[instrument(level = "debug", skip(state))]
async fn get_network_info(
    State(state): State<AppState>,
    device_info: DeviceInfo,
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
        .send(core_request::Payload::ExistingDevice(req), device_info)?;
    let payload = get_core_response(rx).await?;
    if let core_response::Payload::DeviceConfig(response) = payload {
        info!("Got network info for device {pubkey}");
        Ok(Json(response))
    } else {
        error!("Received invalid gRPC response type");
        Err(ApiError::InvalidResponseType)
    }
}
