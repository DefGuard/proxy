use std::time::Duration;

use axum::{
    Json, Router,
    extract::{
        Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
    routing::{any, post},
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{sync::oneshot::Receiver, task::JoinSet};

use crate::{
    error::ApiError,
    handlers::get_core_response,
    http::AppState,
    proto::{
        DeviceInfo, MfaFlowApproveRequest, MfaFlowRemoteRequest, MfaFlowRemoteResponse,
        MfaFlowStartRequest, MfaFlowStartResponse, MfaFlowStepFinishRequest,
        MfaFlowStepFinishResponse, MfaFlowStepStartRequest, MfaFlowStepStartResponse,
        MfaStepResult, core_request, core_response::Payload, mfa_step_result,
    },
};

const REMOTE_AUTH_TIMEOUT: Duration = Duration::from_secs(60);

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start_mfa_flow))
        .route("/step-start", post(step_start_mfa_flow))
        .route("/step-finish", post(step_finish_mfa_flow))
        .route("/approve", post(approve_mfa_flow))
        .route("/remote", any(remote_mfa_flow))
}

#[derive(Deserialize)]
pub(crate) struct RemoteMfaRequestQuery {
    pub token: String,
    pub step_attempt_id: String,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum MfaFlowSocketResponse<'a> {
    #[serde(rename = "mfa_result")]
    Result { result: &'a MfaStepResult },
}

fn mfa_flow_socket_response(response: &MfaFlowRemoteResponse) -> Option<MfaFlowSocketResponse<'_>> {
    let result = response.result.as_ref()?;
    match result.outcome.as_ref() {
        Some(mfa_step_result::Outcome::Advanced(_))
        | Some(mfa_step_result::Outcome::Completed(_)) => {
            Some(MfaFlowSocketResponse::Result { result })
        }
        Some(mfa_step_result::Outcome::AwaitingExternal(_)) | None => None,
    }
}

#[instrument(level = "debug", skip_all)]
async fn start_mfa_flow(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(request): Json<MfaFlowStartRequest>,
) -> Result<Json<MfaFlowStartResponse>, ApiError> {
    let rx = state
        .grpc_server
        .send(core_request::Payload::MfaFlowStart(request), device_info)?;
    let payload = get_core_response(rx, None).await?;
    if let Payload::MfaFlowStart(response) = payload {
        Ok(Json(response))
    } else {
        error!("Received invalid gRPC response type, expected MfaFlowStart");
        Err(ApiError::InvalidResponseType)
    }
}

#[instrument(level = "debug", skip_all)]
async fn step_start_mfa_flow(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(request): Json<MfaFlowStepStartRequest>,
) -> Result<Json<MfaFlowStepStartResponse>, ApiError> {
    let rx = state.grpc_server.send(
        core_request::Payload::MfaFlowStepStart(request),
        device_info,
    )?;
    let payload = get_core_response(rx, None).await?;
    if let Payload::MfaFlowStepStart(response) = payload {
        Ok(Json(response))
    } else {
        error!("Received invalid gRPC response type, expected MfaFlowStepStart");
        Err(ApiError::InvalidResponseType)
    }
}

#[instrument(level = "debug", skip_all)]
async fn step_finish_mfa_flow(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(request): Json<MfaFlowStepFinishRequest>,
) -> Result<Json<MfaFlowStepFinishResponse>, ApiError> {
    let rx = state.grpc_server.send(
        core_request::Payload::MfaFlowStepFinish(request),
        device_info,
    )?;
    let payload = get_core_response(rx, None).await?;
    if let Payload::MfaFlowStepFinish(response) = payload {
        Ok(Json(response))
    } else {
        error!("Received invalid gRPC response type, expected MfaFlowStepFinish");
        Err(ApiError::InvalidResponseType)
    }
}

#[instrument(level = "debug", skip_all)]
async fn approve_mfa_flow(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(request): Json<MfaFlowApproveRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let rx = state
        .grpc_server
        .send(core_request::Payload::MfaFlowApprove(request), device_info)?;
    let payload = get_core_response(rx, None).await?;
    if let Payload::Empty(()) = payload {
        Ok(Json(json!({})))
    } else {
        error!("Received invalid gRPC response type, expected empty response");
        Err(ApiError::InvalidResponseType)
    }
}

pub(crate) fn send_mfa_flow_remote(
    state: &AppState,
    request: MfaFlowRemoteRequest,
    device_info: DeviceInfo,
) -> Result<Receiver<Payload>, ApiError> {
    state
        .grpc_server
        .send(core_request::Payload::MfaFlowRemote(request), device_info)
}

#[instrument(level = "debug", skip_all)]
async fn remote_mfa_flow(
    ws: WebSocketUpgrade,
    Query(query): Query<RemoteMfaRequestQuery>,
    State(state): State<AppState>,
    device_info: DeviceInfo,
) -> Result<Response, ApiError> {
    let request = MfaFlowRemoteRequest {
        token: query.token,
        step_attempt_id: query.step_attempt_id,
    };
    let rx = send_mfa_flow_remote(&state, request, device_info)?;

    Ok(ws.on_upgrade(move |socket| handle_remote_mfa_flow_socket(socket, rx)))
}

async fn handle_remote_mfa_flow_socket(socket: WebSocket, rx: Receiver<Payload>) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let mut set = JoinSet::new();

    set.spawn(async move {
        match get_core_response(rx, Some(REMOTE_AUTH_TIMEOUT)).await {
            Ok(Payload::MfaFlowRemote(response)) => {
                if let Some(ws_response) = mfa_flow_socket_response(&response) {
                    match serde_json::to_string(&ws_response) {
                        Ok(serialized) => {
                            if ws_tx.send(Message::Text(serialized.into())).await.is_err() {
                                error!("Failed to send MFA flow result via WebSocket");
                            }
                        }
                        Err(_) => error!("Failed to serialize MFA flow result for WebSocket"),
                    }
                } else {
                    error!("Received invalid MFA flow remote result from Core");
                }
            }
            Ok(_) => error!("Received invalid gRPC response type, expected MfaFlowRemote"),
            Err(_) => error!("Failed to receive MFA flow remote result from Core"),
        }

        let _ = ws_tx.close().await;
    });

    set.spawn(async move {
        while let Some(message) = ws_rx.next().await {
            match message {
                Ok(Message::Close(_)) => break,
                Ok(_) => {}
                Err(_) => {
                    error!("MFA flow WebSocket client listen error");
                    break;
                }
            }
        }
    });

    let _ = set.join_next().await;
    set.shutdown().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{MfaAdvanced, MfaCompleted, mfa_step_result};

    fn response(outcome: mfa_step_result::Outcome) -> MfaFlowRemoteResponse {
        MfaFlowRemoteResponse {
            result: Some(MfaStepResult {
                outcome: Some(outcome),
            }),
        }
    }

    fn serialized(response: &MfaFlowRemoteResponse) -> serde_json::Value {
        serde_json::to_value(mfa_flow_socket_response(response).expect("valid remote result"))
            .expect("MFA flow response should serialize")
    }

    #[test]
    fn test_advanced_serializes_as_mfa_result() {
        let frame = serialized(&response(mfa_step_result::Outcome::Advanced(MfaAdvanced {
            next_step: 1,
        })));

        assert_eq!(
            frame,
            serde_json::json!({
                "type": "mfa_result",
                "result": {"outcome": {"Advanced": {"next_step": 1}}}
            })
        );
    }

    #[test]
    fn test_completed_nests_the_preshared_key() {
        let frame = serialized(&response(mfa_step_result::Outcome::Completed(
            MfaCompleted {
                preshared_key: "completed-psk".to_string(),
            },
        )));

        assert_eq!(
            frame,
            serde_json::json!({
                "type": "mfa_result",
                "result": {
                    "outcome": {"Completed": {"preshared_key": "completed-psk"}}
                }
            })
        );
        assert!(frame.get("preshared_key").is_none());
    }

    #[test]
    fn test_awaiting_external_is_not_a_remote_result() {
        let response = MfaFlowRemoteResponse {
            result: Some(MfaStepResult {
                outcome: Some(mfa_step_result::Outcome::AwaitingExternal(
                    crate::proto::MfaAwaitingExternal {},
                )),
            }),
        };

        assert!(mfa_flow_socket_response(&response).is_none());
    }
}
