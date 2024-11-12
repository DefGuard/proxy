use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    PrivateCookieJar,
};
use serde::{Deserialize, Serialize};
use time::Duration;

use crate::{
    error::ApiError,
    handlers::get_core_response,
    http::AppState,
    proto::{core_request, core_response, AuthCallbackRequest, AuthInfoRequest},
};

const COOKIE_MAX_AGE: Duration = Duration::days(1);
static CSRF_COOKIE_NAME: &str = "csrf";
static NONCE_COOKIE_NAME: &str = "nonce";

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/auth_info", get(auth_info))
        .route("/callback", post(auth_callback))
}

#[derive(Serialize)]
struct AuthInfo {
    url: String,
}

impl AuthInfo {
    #[must_use]
    fn new(url: String) -> Self {
        Self { url }
    }
}

/// Request external OAuth2/OpenID provider details from Defguard Core.
#[instrument(level = "debug", skip(state))]
async fn auth_info(
    State(state): State<AppState>,
    private_cookies: PrivateCookieJar,
) -> Result<(PrivateCookieJar, Json<AuthInfo>), ApiError> {
    debug!("Getting auth info for OAuth2/OpenID login");

    let request = AuthInfoRequest {
        redirect_url: state.callback_url().to_string(),
    };

    let rx = state
        .grpc_server
        .send(Some(core_request::Payload::AuthInfo(request)), None)?;
    let payload = get_core_response(rx).await?;
    if let core_response::Payload::AuthInfo(response) = payload {
        debug!("Received auth info {response:?}");

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

        let auth_info = AuthInfo::new(response.url);
        Ok((private_cookies, Json(auth_info)))
    } else {
        error!("Received invalid gRPC response type: {payload:#?}");
        Err(ApiError::InvalidResponseType)
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthenticationResponse {
    id_token: String,
    state: String,
}

#[instrument(level = "debug", skip(state))]
async fn auth_callback(
    State(state): State<AppState>,
    mut private_cookies: PrivateCookieJar,
    Json(payload): Json<AuthenticationResponse>,
) -> Result<PrivateCookieJar, ApiError> {
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
        id_token: payload.id_token,
        nonce,
        callback_url: state.callback_url().to_string(),
    };

    let rx = state
        .grpc_server
        .send(Some(core_request::Payload::AuthCallback(request)), None)?;
    let payload = get_core_response(rx).await?;
    if let core_response::Payload::Empty(()) = payload {
        debug!("Received auth callback response (empty message)");
        Ok(private_cookies)
    } else {
        error!("Received invalid gRPC response type: {payload:#?}");
        Err(ApiError::InvalidResponseType)
    }
}
