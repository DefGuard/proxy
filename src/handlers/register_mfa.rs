use axum::Router;
use serde::Deserialize;

use axum::{extract::State, response::IntoResponse, routing::post, Json};
use axum_extra::extract::PrivateCookieJar;

use crate::http::ENROLLMENT_COOKIE_NAME;
use crate::proto::MfaMethod;
use crate::{
    error::ApiError,
    handlers::get_core_response,
    http::AppState,
    proto::{
        core_request, core_response, CodeMfaSetupFinishRequest, CodeMfaSetupFinishResponse,
        CodeMfaSetupStartRequest, CodeMfaSetupStartResponse, DeviceInfo,
    },
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(register_code_mfa_start))
        .route("/finish", post(register_code_mfa_finish))
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
) -> Result<Json<CodeMfaSetupStartResponse>, impl IntoResponse> {
    let token = cookie_jar
        .get(ENROLLMENT_COOKIE_NAME)
        .ok_or_else(|| ApiError::Unauthorized(String::new()))?
        .value()
        .to_string();

    if req.method != MfaMethod::Email && req.method != MfaMethod::Totp {
        return Err(ApiError::BadRequest("Method not supported.".to_string()));
    }

    let rx = state.grpc_server.send(
        core_request::Payload::CodeMfaSetupStartRequest(CodeMfaSetupStartRequest {
            token,
            method: req.method.into(),
        }),
        device_info,
    )?;
    let payload = get_core_response(rx).await?;
    match payload {
        core_response::Payload::CodeMfaSetupStartResponse(response) => Ok(Json(response)),
        _ => Err(ApiError::InvalidResponseType),
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RegisterMfaCodeFinishRequest {
    pub code: String,
}

#[instrument(level = "debug", skip(state, req))]
async fn register_code_mfa_finish(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    cookie_jar: PrivateCookieJar,
    Json(req): Json<RegisterMfaCodeFinishRequest>,
) -> Result<Json<CodeMfaSetupFinishResponse>, impl IntoResponse> {
    let token = cookie_jar
        .get(ENROLLMENT_COOKIE_NAME)
        .ok_or_else(|| ApiError::Unauthorized(String::new()))?
        .value()
        .to_string();

    let rx = state.grpc_server.send(
        core_request::Payload::CodeMfaSetupFinishRequest(CodeMfaSetupFinishRequest {
            token,
            code: req.code,
        }),
        device_info,
    )?;
    let payload = get_core_response(rx).await?;
    match payload {
        core_response::Payload::CodeMfaSetupFinishResponse(response) => Ok(Json(response)),
        _ => Err(ApiError::InvalidResponseType),
    }
}
