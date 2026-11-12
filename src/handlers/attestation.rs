use axum::{Json, Router, extract::State, routing::post};
use axum_extra::extract::PrivateCookieJar;
use serde::Deserialize;

use crate::{
    error::ApiError,
    generated::defguard::device_identity::v2::AttestationChallengeSolution,
    handlers::{enrollment_token, get_core_response},
    http::AppState,
    proto::{
        AttestationFinishRequest, AttestationFinishResponse, AttestationStartRequest,
        AttestationStartResponse, DeviceInfo, attestation_start_request, core_request,
        core_response,
    },
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start))
        .route("/finish", post(finish))
}

#[derive(Deserialize)]
struct AttestationStartBody {
    device_id: i64,
    device_identity: Option<attestation_start_request::DeviceIdentity>,
}

#[derive(Deserialize)]
struct AttestationFinishBody {
    device_id: i64,
    solution: Option<AttestationChallengeSolution>,
}

#[instrument(level = "debug", skip(state, cookie_jar, req))]
async fn start(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    cookie_jar: PrivateCookieJar,
    Json(req): Json<AttestationStartBody>,
) -> Result<Json<AttestationStartResponse>, ApiError> {
    let token = enrollment_token(&cookie_jar)?;
    let rx = state.grpc_server.send(
        core_request::Payload::AttestationStart(AttestationStartRequest {
            token,
            device_id: req.device_id,
            device_identity: req.device_identity,
        }),
        device_info,
    )?;
    match get_core_response(rx, None).await? {
        core_response::Payload::AttestationStart(response) => Ok(Json(response)),
        _ => Err(ApiError::InvalidResponseType),
    }
}

#[instrument(level = "debug", skip(state, cookie_jar, req))]
async fn finish(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    cookie_jar: PrivateCookieJar,
    Json(req): Json<AttestationFinishBody>,
) -> Result<Json<AttestationFinishResponse>, ApiError> {
    let token = enrollment_token(&cookie_jar)?;
    let rx = state.grpc_server.send(
        core_request::Payload::AttestationFinish(AttestationFinishRequest {
            token,
            device_id: req.device_id,
            solution: req.solution,
        }),
        device_info,
    )?;
    match get_core_response(rx, None).await? {
        core_response::Payload::AttestationFinish(response) => Ok(Json(response)),
        _ => Err(ApiError::InvalidResponseType),
    }
}
