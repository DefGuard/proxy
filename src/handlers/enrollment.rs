use std::net::SocketAddr;

use crate::error::ApiError;
use crate::server::{COOKIE_NAME, SECRET_KEY};
use crate::{
    grpc::enrollment::proto::{
        ActivateUserRequest, CreateDeviceResponse, EnrollmentStartRequest, EnrollmentStartResponse,
        NewDevice,
    },
    handlers::ApiResult,
    server::AppState,
};
use axum::extract::ConnectInfo;
use axum::headers::UserAgent;
use axum::TypedHeader;
use axum::{extract::State, routing::post, Json, Router};
use tonic::metadata::MetadataValue;
use tower_cookies::{cookie::time::OffsetDateTime, Cookie, Cookies};
use tracing::{debug, error, info};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start_enrollment_process))
        .route("/activate_user", post(activate_user))
        .route("/create_device", post(create_device))
}

// extract token from session cookies and add it to gRPC request auth header
fn add_auth_header<T>(
    cookies: Cookies,
    request: &mut tonic::Request<T>,
    ip_address: String,
    user_agent: Option<TypedHeader<UserAgent>>,
) -> Result<(), ApiError> {
    debug!("Adding auth header to gRPC request");
    let key = SECRET_KEY.get().unwrap();
    let private_cookies = cookies.private(key);

    match private_cookies.get(COOKIE_NAME) {
        Some(cookie) => {
            let token = MetadataValue::try_from(cookie.value())?;
            let user_agent_string = match user_agent {
                Some(value) => value.to_string(),
                None => String::new(),
            };
            request.metadata_mut().insert("authorization", token);
            request
                .metadata_mut()
                .insert("ip_address", MetadataValue::try_from(ip_address)?);
            request
                .metadata_mut()
                .insert("user_agent", MetadataValue::try_from(user_agent_string)?);
        }
        None => {
            error!("Enrollment session cookie not found");
            return Err(ApiError::CookieNotFound);
        }
    }

    Ok(())
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
    if let Some(cookie) = private_cookies.get(COOKIE_NAME) {
        debug!("Removing previous session cookie");
        cookies.remove(cookie)
    };

    let token = req.token.clone();

    let mut client = state.client.lock().await;
    let response = client.start_enrollment(req).await?.into_inner();

    // set session cookie
    let cookie = Cookie::build(COOKIE_NAME, token)
        .expires(OffsetDateTime::from_unix_timestamp(response.deadline_timestamp).unwrap())
        .finish();
    private_cookies.add(cookie);

    Ok(Json(response))
}

pub async fn activate_user(
    State(state): State<AppState>,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    user_agent: Option<TypedHeader<UserAgent>>,
    cookies: Cookies,
    Json(req): Json<ActivateUserRequest>,
) -> ApiResult<()> {
    info!("Activating user");

    let ip_address = connect_info.ip().to_string();
    let mut client = state.client.lock().await;
    let mut request = tonic::Request::new(req);
    add_auth_header(cookies, &mut request, ip_address, user_agent)?;
    client.activate_user(request).await?;

    Ok(())
}

pub async fn create_device(
    State(state): State<AppState>,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    user_agent: Option<TypedHeader<UserAgent>>,
    cookies: Cookies,
    Json(req): Json<NewDevice>,
) -> ApiResult<Json<CreateDeviceResponse>> {
    info!("Adding new device");

    let ip_address = connect_info.ip().to_string();
    let mut client = state.client.lock().await;
    let mut request = tonic::Request::new(req);
    add_auth_header(cookies, &mut request, ip_address, user_agent)?;
    let response = client.create_device(request).await?;

    Ok(Json(response.into_inner()))
}
