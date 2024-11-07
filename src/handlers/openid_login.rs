use axum::{extract::State, routing::get, Json, Router};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    PrivateCookieJar,
};
use serde::Serialize;
use time::Duration;

use crate::{
    error::ApiError,
    handlers::get_core_response,
    http::AppState,
    proto::{core_request, core_response},
};

const COOKIE_MAX_AGE: Duration = Duration::days(1);
static CSRF_COOKIE_NAME: &str = "csrf";
static NONCE_COOKIE_NAME: &str = "nonce";

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/auth_info", get(auth_info))
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

    let rx = state
        .grpc_server
        .send(Some(core_request::Payload::AuthInfo(())), None)?;
    let payload = get_core_response(rx).await?;
    if let core_response::Payload::AuthInfo(response) = payload {
        debug!("Got auth info {response:?}");

        let nonce_cookie = Cookie::build((NONCE_COOKIE_NAME, response.nonce))
            // .domain(cookie_domain)
            // .path("/api/v1/openid/callback")
            .http_only(true)
            .same_site(SameSite::Strict)
            .secure(true)
            .max_age(COOKIE_MAX_AGE)
            .build();
        let csrf_cookie = Cookie::build((CSRF_COOKIE_NAME, response.csrf_token))
            // .domain(cookie_domain)
            // .path("/api/v1/openid/callback")
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
