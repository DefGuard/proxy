use axum::{Json, Router, extract::State, routing::post};

use super::register_mfa::{code_mfa_setup_finish, code_mfa_setup_start};
use crate::{
    error::ApiError,
    handlers::get_core_response,
    http::AppState,
    proto::{
        CodeMfaSetupFinishRequest, CodeMfaSetupFinishResponse, CodeMfaSetupStartRequest,
        CodeMfaSetupStartResponse, DeviceInfo, MfaConfigAuthorizeRequest,
        MfaConfigAuthorizeResponse, MfaConfigSendCodeRequest, MfaConfigStartRequest,
        MfaConfigStartResponse, core_request, core_response,
    },
};

/// MFA factor configuration for an enrolled desktop client.
///
/// The client is not a browser, so every request carries its token in the JSON body.
pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start_mfa_config))
        .route("/send-code", post(send_mfa_config_code))
        .route("/authorize", post(authorize_mfa_config))
        .route("/setup/start", post(start_mfa_setup))
        .route("/setup/finish", post(finish_mfa_setup))
}

#[instrument(level = "debug", skip(state, req))]
async fn start_mfa_config(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(req): Json<MfaConfigStartRequest>,
) -> Result<Json<MfaConfigStartResponse>, ApiError> {
    info!("Starting MFA configuration for device {}", req.pubkey);
    let rx = state
        .grpc_server
        .send(core_request::Payload::MfaConfigStart(req), device_info)?;
    let payload = get_core_response(rx, None).await?;
    if let core_response::Payload::MfaConfigStart(response) = payload {
        Ok(Json(response))
    } else {
        error!("Received invalid gRPC response type, expected MfaConfigStart");
        Err(ApiError::InvalidResponseType)
    }
}

#[instrument(level = "debug", skip(state, req))]
async fn send_mfa_config_code(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(req): Json<MfaConfigSendCodeRequest>,
) -> Result<(), ApiError> {
    info!("Sending MFA configuration email code");
    let rx = state
        .grpc_server
        .send(core_request::Payload::MfaConfigSendCode(req), device_info)?;
    let payload = get_core_response(rx, None).await?;
    if let core_response::Payload::MfaConfigSendCode(_) = payload {
        Ok(())
    } else {
        error!("Received invalid gRPC response type, expected MfaConfigSendCode");
        Err(ApiError::InvalidResponseType)
    }
}

#[instrument(level = "debug", skip(state, req))]
async fn authorize_mfa_config(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(req): Json<MfaConfigAuthorizeRequest>,
) -> Result<Json<MfaConfigAuthorizeResponse>, ApiError> {
    info!("Authorizing MFA configuration session");
    let rx = state
        .grpc_server
        .send(core_request::Payload::MfaConfigAuthorize(req), device_info)?;
    let payload = get_core_response(rx, None).await?;
    if let core_response::Payload::MfaConfigAuthorize(response) = payload {
        Ok(Json(response))
    } else {
        error!("Received invalid gRPC response type, expected MfaConfigAuthorize");
        Err(ApiError::InvalidResponseType)
    }
}

#[instrument(level = "debug", skip(state, req))]
async fn start_mfa_setup(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(req): Json<CodeMfaSetupStartRequest>,
) -> Result<Json<CodeMfaSetupStartResponse>, ApiError> {
    code_mfa_setup_start(&state, device_info, req).await
}

#[instrument(level = "debug", skip(state, req))]
async fn finish_mfa_setup(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(req): Json<CodeMfaSetupFinishRequest>,
) -> Result<Json<CodeMfaSetupFinishResponse>, ApiError> {
    code_mfa_setup_finish(&state, device_info, req).await
}
