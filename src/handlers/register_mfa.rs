use axum::{Json, Router, extract::State, routing::post};
use axum_extra::extract::PrivateCookieJar;
use serde::Deserialize;

use crate::{
    error::ApiError,
    handlers::{enrollment_token, get_core_response},
    http::AppState,
    proto::{
        CodeMfaSetupFinishRequest, CodeMfaSetupFinishResponse, CodeMfaSetupStartRequest,
        CodeMfaSetupStartResponse, DeviceInfo, MfaMethod, core_request, core_response,
    },
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/code/start", post(register_code_mfa_start))
        .route("/code/finish", post(register_code_mfa_finish))
}

/// Forwards an MFA factor setup start to Core.
///
/// `req.token` is either an enrollment token or an authorized MFA config session token.
pub(super) async fn code_mfa_setup_start(
    state: &AppState,
    device_info: DeviceInfo,
    req: CodeMfaSetupStartRequest,
) -> Result<Json<CodeMfaSetupStartResponse>, ApiError> {
    debug!("MFA factor setup started");
    reject_unsupported_method(req.method)?;

    let rx = state
        .grpc_server
        .send(core_request::Payload::CodeMfaSetupStart(req), device_info)?;
    let payload = get_core_response(rx, None).await?;
    match payload {
        core_response::Payload::CodeMfaSetupStartResponse(response) => Ok(Json(response)),
        _ => Err(ApiError::InvalidResponseType),
    }
}

/// Forwards an MFA factor setup finish to Core. See [`code_mfa_setup_start`] for the token.
pub(super) async fn code_mfa_setup_finish(
    state: &AppState,
    device_info: DeviceInfo,
    req: CodeMfaSetupFinishRequest,
) -> Result<Json<CodeMfaSetupFinishResponse>, ApiError> {
    reject_unsupported_method(req.method)?;

    let rx = state
        .grpc_server
        .send(core_request::Payload::CodeMfaSetupFinish(req), device_info)?;
    let payload = get_core_response(rx, None).await?;
    match payload {
        core_response::Payload::CodeMfaSetupFinishResponse(response) => Ok(Json(response)),
        _ => Err(ApiError::InvalidResponseType),
    }
}

/// The factors Core's MFA setup can enable: a code by email or TOTP, or a FIDO2
/// security key, whose proof is an attestation rather than a code.
fn reject_unsupported_method(method: i32) -> Result<(), ApiError> {
    if matches!(
        MfaMethod::try_from(method),
        Ok(MfaMethod::Email | MfaMethod::Totp | MfaMethod::Fido2)
    ) {
        Ok(())
    } else {
        error!("Requested method not supported");
        Err(ApiError::BadRequest("Method not supported.".to_string()))
    }
}

/// The enrollment routes below carry neither a key name nor an attestation, so they
/// can only ever set up a code factor; FIDO2 has to go through MFA configuration.
fn reject_non_code_method(method: MfaMethod) -> Result<(), ApiError> {
    if matches!(method, MfaMethod::Email | MfaMethod::Totp) {
        Ok(())
    } else {
        error!("Requested method not supported during enrollment");
        Err(ApiError::BadRequest("Method not supported.".to_string()))
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RegisterMfaCodeStartRequest {
    pub method: MfaMethod,
}

#[instrument(level = "debug", skip(state, req))]
async fn register_code_mfa_start(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    cookie_jar: PrivateCookieJar,
    Json(req): Json<RegisterMfaCodeStartRequest>,
) -> Result<Json<CodeMfaSetupStartResponse>, ApiError> {
    let token = enrollment_token(&cookie_jar)?;
    reject_non_code_method(req.method)?;
    code_mfa_setup_start(
        &state,
        device_info,
        CodeMfaSetupStartRequest {
            token,
            method: req.method.into(),
        },
    )
    .await
}

#[derive(Debug, Clone, Deserialize)]
struct RegisterMfaCodeFinishRequest {
    pub code: String,
    pub method: MfaMethod,
}

#[instrument(level = "debug", skip(state, req))]
async fn register_code_mfa_finish(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    cookie_jar: PrivateCookieJar,
    Json(req): Json<RegisterMfaCodeFinishRequest>,
) -> Result<Json<CodeMfaSetupFinishResponse>, ApiError> {
    let token = enrollment_token(&cookie_jar)?;
    reject_non_code_method(req.method)?;
    code_mfa_setup_finish(
        &state,
        device_info,
        CodeMfaSetupFinishRequest {
            token,
            code: req.code,
            method: req.method as i32,
            // FIDO2 only, and rejected above.
            name: None,
            fido2_attestation: None,
        },
    )
    .await
}
