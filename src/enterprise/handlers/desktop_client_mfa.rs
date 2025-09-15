use axum::{extract::State, Json};
use axum_extra::extract::{cookie::Cookie, PrivateCookieJar};
use tracing::{debug, error, info, warn};

use crate::{
    enterprise::handlers::openid_login::{
        AuthenticationResponse, FlowType, CSRF_COOKIE_NAME, NONCE_COOKIE_NAME,
    },
    error::ApiError,
    handlers::get_core_response,
    http::AppState,
    proto::{core_request, core_response, ClientMfaOidcAuthenticateRequest, DeviceInfo},
};

#[instrument(level = "debug", skip(state))]
pub(super) async fn mfa_auth_callback(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    mut private_cookies: PrivateCookieJar,
    Json(payload): Json<AuthenticationResponse>,
) -> Result<PrivateCookieJar, ApiError> {
    info!("Processing MFA authentication callback");
    debug!(
        "Received payload: state={}, flow_type={}",
        payload.state, payload.flow_type
    );

    let flow_type = payload.flow_type.parse::<FlowType>().map_err(|err| {
        warn!("Failed to parse flow type '{}': {err:?}", payload.flow_type);
        ApiError::BadRequest("Invalid flow type".into())
    })?;

    if flow_type != FlowType::Mfa {
        warn!("Invalid flow type for MFA callback: {flow_type:?}");
        return Err(ApiError::BadRequest(
            "Invalid flow type for MFA callback".into(),
        ));
    }

    debug!("Flow type validation passed: {flow_type:?}");

    let nonce = private_cookies
        .get(NONCE_COOKIE_NAME)
        .ok_or_else(|| {
            warn!("Nonce cookie not found in request");
            ApiError::Unauthorized("Nonce cookie not found".into())
        })?
        .value_trimmed()
        .to_string();

    let csrf = private_cookies
        .get(CSRF_COOKIE_NAME)
        .ok_or_else(|| {
            warn!("CSRF cookie not found in request");
            ApiError::Unauthorized("CSRF cookie not found".into())
        })?
        .value_trimmed()
        .to_string();

    debug!("Retrieved cookies successfully");

    if payload.state != csrf {
        warn!(
            "CSRF token mismatch: expected={csrf}, received={}",
            payload.state
        );
        return Err(ApiError::Unauthorized("CSRF token mismatch".into()));
    }

    debug!("CSRF token validation passed");

    private_cookies = private_cookies
        .remove(Cookie::from(NONCE_COOKIE_NAME))
        .remove(Cookie::from(CSRF_COOKIE_NAME));

    debug!("Removed security cookies");

    let request = ClientMfaOidcAuthenticateRequest {
        code: payload.code,
        nonce,
        callback_url: state.callback_url(&flow_type).to_string(),
        state: payload.state,
    };

    debug!("Sending MFA OIDC authenticate request to core service");

    let rx = state.grpc_server.send(
        core_request::Payload::ClientMfaOidcAuthenticate(request),
        device_info,
    )?;

    let payload = get_core_response(rx).await?;

    if let core_response::Payload::Empty(()) = payload {
        info!("MFA authentication callback completed successfully");
        Ok(private_cookies)
    } else {
        error!("Received invalid gRPC response type during handling the MFA OpenID authentication callback: {payload:#?}");
        Err(ApiError::InvalidResponseType)
    }
}
