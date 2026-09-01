use axum::{Json, Router, extract::State, routing::post};
use axum_extra::extract::PrivateCookieJar;
use cookie::CookieBuilder;
use serde::{Deserialize, Serialize};
use time::Duration;

use crate::{
    enterprise::handlers::desktop_client_mfa::mfa_auth_callback,
    error::ApiError,
    handlers::get_core_response,
    http::{AppState, session_cookie},
    proto::{
        AuthCallbackRequest, AuthCallbackResponse, AuthFlowType, AuthInfoRequest, DeviceInfo,
        core_request, core_response,
    },
};

const COOKIE_MAX_AGE: Duration = Duration::days(1);
pub(super) const OIDC_CALLBACK_PATH: &str = "/api/v1/openid/callback";
pub(super) static CSRF_COOKIE_NAME: &str = "csrf_proxy";
pub(super) static NONCE_COOKIE_NAME: &str = "nonce_proxy";

pub(super) fn oidc_cookie(name: &'static str, value: String) -> CookieBuilder<'static> {
    session_cookie(name, value, OIDC_CALLBACK_PATH, true)
}

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

        let nonce_cookie = oidc_cookie(NONCE_COOKIE_NAME, response.nonce)
            // .domain(cookie_domain)
            .max_age(COOKIE_MAX_AGE)
            .build();
        let csrf_cookie = oidc_cookie(CSRF_COOKIE_NAME, response.csrf_token)
            // .domain(cookie_domain)
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
        .remove(oidc_cookie(NONCE_COOKIE_NAME, String::new()).build())
        .remove(oidc_cookie(CSRF_COOKIE_NAME, String::new()).build());

    let request = AuthCallbackRequest {
        code: payload.code,
        nonce,
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

#[cfg(test)]
mod tests {
    use axum_extra::extract::cookie::SameSite;
    use time::Duration;

    use super::*;

    #[test]
    fn test_oidc_cookie_attributes_preserve_security() {
        for name in [NONCE_COOKIE_NAME, CSRF_COOKIE_NAME] {
            let cookie = oidc_cookie(name, "value".to_owned())
                .max_age(COOKIE_MAX_AGE)
                .build();
            assert_eq!(cookie.secure(), Some(true));
            assert_eq!(cookie.http_only(), Some(true));
            assert_eq!(cookie.same_site(), Some(SameSite::Strict));
            assert_eq!(cookie.path(), Some(OIDC_CALLBACK_PATH));
            assert_eq!(cookie.max_age(), Some(COOKIE_MAX_AGE));

            let mut removal = oidc_cookie(name, String::new()).build();
            removal.make_removal();
            assert_eq!(removal.secure(), Some(true));
            assert_eq!(removal.http_only(), Some(true));
            assert_eq!(removal.same_site(), Some(SameSite::Strict));
            assert_eq!(removal.path(), Some(OIDC_CALLBACK_PATH));
            assert_eq!(removal.max_age(), Some(Duration::ZERO));
        }
    }
}
