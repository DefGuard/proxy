use axum::{extract::State, routing::post, Json, Router};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    PrivateCookieJar,
};
use serde::{Deserialize, Serialize};
use time::Duration;

use crate::{
    enterprise::handlers::desktop_client_mfa::mfa_auth_callback,
    error::ApiError,
    handlers::get_core_response,
    http::AppState,
    proto::{
        core_request, core_response, AuthCallbackRequest, AuthCallbackResponse, AuthFlowType,
        AuthInfoRequest, DeviceInfo,
    },
};

const COOKIE_MAX_AGE: Duration = Duration::days(1);
pub(super) static CSRF_COOKIE_NAME: &str = "csrf_proxy";
pub(super) static NONCE_COOKIE_NAME: &str = "nonce_proxy";

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/auth_info", post(auth_info))
        .route("/callback", post(auth_callback))
        .route("/callback/mfa", post(mfa_auth_callback))
}

#[derive(Serialize)]
struct AuthInfo {
    url: String,
    button_display_name: Option<String>,
}

impl AuthInfo {
    #[must_use]
    fn new(url: String, button_display_name: Option<String>) -> Self {
        Self {
            url,
            button_display_name,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum FlowType {
    Enrollment,
    Mfa,
}

#[derive(Deserialize, Debug)]
pub(crate) struct RequestData {
    state: Option<String>,
    #[serde(rename = "type")]
    flow_type: FlowType,
}

/// Request external OAuth2/OpenID provider details from Defguard Core.
#[instrument(level = "debug", skip(state))]
async fn auth_info(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    private_cookies: PrivateCookieJar,
    Json(request_data): Json<RequestData>,
) -> Result<(PrivateCookieJar, Json<AuthInfo>), ApiError> {
    debug!("Getting auth info for OAuth2/OpenID login");

    let auth_flow_type = match request_data.flow_type {
        FlowType::Enrollment => AuthFlowType::Enrollment as i32,
        FlowType::Mfa => AuthFlowType::Mfa as i32,
    };
    let request = AuthInfoRequest {
        #[allow(deprecated)]
        redirect_url: String::new(),
        state: request_data.state,
        auth_flow_type,
    };

    let rx = state
        .grpc_server
        .send(core_request::Payload::AuthInfo(request), device_info)?;
    let payload = get_core_response(rx, None).await?;
    if let core_response::Payload::AuthInfo(response) = payload {
        debug!("Received auth info response");

        let nonce_cookie = Cookie::build((NONCE_COOKIE_NAME, response.nonce))
            // .domain(cookie_domain)
            .path("/api/v1/openid/callback")
            .http_only(true)
            .same_site(SameSite::Strict)
            .secure(true)
            .max_age(COOKIE_MAX_AGE)
            .build();
        let csrf_cookie = Cookie::build((CSRF_COOKIE_NAME, response.csrf_token))
            // .domain(cookie_domain)
            .path("/api/v1/openid/callback")
            .http_only(true)
            .same_site(SameSite::Strict)
            .secure(true)
            .max_age(COOKIE_MAX_AGE)
            .build();
        let private_cookies = private_cookies.add(nonce_cookie).add(csrf_cookie);

        let auth_info = AuthInfo::new(response.url, response.button_display_name);
        Ok((private_cookies, Json(auth_info)))
    } else {
        error!("Received invalid gRPC response type");
        Err(ApiError::InvalidResponseType)
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct AuthenticationResponse {
    pub(super) code: String,
    pub(super) state: String,
    #[serde(rename = "type")]
    pub(super) flow_type: FlowType,
}

#[derive(Serialize)]
struct CallbackResponseData {
    url: String,
    token: String,
}

#[instrument(level = "debug", skip(state))]
async fn auth_callback(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    mut private_cookies: PrivateCookieJar,
    Json(payload): Json<AuthenticationResponse>,
) -> Result<(PrivateCookieJar, Json<CallbackResponseData>), ApiError> {
    match payload.flow_type {
        FlowType::Enrollment => (),
        FlowType::Mfa => {
            return Err(ApiError::BadRequest(
                "Invalid flow type for OpenID enrollment callback".into(),
            ));
        }
    }

    let nonce = private_cookies
        .get(NONCE_COOKIE_NAME)
        .ok_or(ApiError::Unauthorized("Nonce cookie not found".into()))?
        .value_trimmed()
        .to_string();
    let csrf = private_cookies
        .get(CSRF_COOKIE_NAME)
        .ok_or(ApiError::Unauthorized("CSRF cookie not found".into()))?
        .value_trimmed()
        .to_string();

    if payload.state != csrf {
        return Err(ApiError::Unauthorized("CSRF token mismatch".into()));
    }

    private_cookies = private_cookies
        .remove(Cookie::from(NONCE_COOKIE_NAME))
        .remove(Cookie::from(CSRF_COOKIE_NAME));

    let request = AuthCallbackRequest {
        code: payload.code,
        nonce,
        #[allow(deprecated)]
        callback_url: String::new(),
    };

    let rx = state
        .grpc_server
        .send(core_request::Payload::AuthCallback(request), device_info)?;
    let payload = get_core_response(rx, None).await?;

    if let core_response::Payload::AuthCallback(AuthCallbackResponse { url, token }) = payload {
        debug!("Received auth callback response {url:?} {token:?}");
        Ok((private_cookies, Json(CallbackResponseData { url, token })))
    } else {
        error!(
            "Received invalid gRPC response type during handling the OpenID authentication \
            callback"
        );
        Err(ApiError::InvalidResponseType)
    }
}
