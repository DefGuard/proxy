use std::time::Duration;

use axum::{
    Json, Router,
    extract::{
        Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::{IntoResponse, Response},
    routing::{any, post},
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::task::JoinSet;

use crate::{
    error::ApiError,
    handlers::get_core_response,
    http::AppState,
    proto::{
        AwaitRemoteMfaFinishRequest, AwaitRemoteMfaFinishResponse, ClientMfaFinishRequest,
        ClientMfaFinishResponse, ClientMfaStartRequest, ClientMfaStartResponse,
        ClientMfaStepStartRequest, ClientMfaStepStartResponse, DeviceInfo, MfaStepResult,
        core_request,
        core_response::{self, Payload},
    },
};

// How much time the user has to approve remote MFA with mobile device
const REMOTE_AUTH_TIMEOUT: Duration = Duration::from_secs(60);

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start_client_mfa))
        .route("/step-start", post(step_start_client_mfa))
        .route("/finish", post(finish_client_mfa))
        .route("/remote", any(await_remote_auth))
        .route("/finish-remote", post(finish_remote_mfa))
}

#[derive(Deserialize)]
pub(crate) struct RemoteMfaRequestQuery {
    pub token: String,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum RemoteMfaResponse<'a> {
    #[serde(rename = "mfa_success")]
    Legacy { preshared_key: &'a str },
    #[serde(rename = "mfa_result")]
    Result { result: &'a MfaStepResult },
}

fn remote_mfa_response(response: &AwaitRemoteMfaFinishResponse) -> Option<RemoteMfaResponse<'_>> {
    match response.result.as_ref() {
        // New-protocol outcome: preserve the result and avoid the legacy success type.
        Some(result) if result.outcome.is_some() => Some(RemoteMfaResponse::Result { result }),
        // A present but empty result is malformed. Do not downgrade it to legacy behavior.
        Some(_) => None,
        // Legacy response: preserve the deprecated envelope for deployed clients.
        None => {
            #[allow(deprecated)]
            let preshared_key = response.preshared_key.as_str();
            Some(RemoteMfaResponse::Legacy { preshared_key })
        }
    }
}

// Allows desktop client to await for another device to complete MFA for it via mobile client.
#[instrument(level = "debug", skip(state, req))]
async fn await_remote_auth(
    ws: WebSocketUpgrade,
    Query(req): Query<RemoteMfaRequestQuery>,
    State(state): State<AppState>,
    device_info: DeviceInfo,
) -> Result<Response, impl IntoResponse> {
    let token = req.token;
    // let core validate token first
    let rx = state.grpc_server.send(
        core_request::Payload::ClientMfaTokenValidation(
            crate::proto::ClientMfaTokenValidationRequest {
                token: token.clone(),
            },
        ),
        device_info.clone(),
    )?;
    let payload = get_core_response(rx, Some(REMOTE_AUTH_TIMEOUT)).await?;
    if let core_response::Payload::ClientMfaTokenValidation(response) = payload {
        if !response.token_valid {
            return Err(ApiError::Unauthorized(String::new()));
        }

        Ok(ws.on_upgrade(move |socket| {
            handle_remote_auth_socket(socket, state.clone(), token, device_info)
        }))
    } else {
        Err(ApiError::InvalidResponseType)
    }
}

/// Handle axum web socket upgrade for `await_remote_auth`.
async fn handle_remote_auth_socket(
    socket: WebSocket,
    state: AppState,
    token: String,
    device_info: DeviceInfo,
) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let mut set = JoinSet::new();

    let request = AwaitRemoteMfaFinishRequest { token };
    let rx = match state.grpc_server.send(
        core_request::Payload::AwaitRemoteMfaFinish(request),
        device_info,
    ) {
        Ok(rx) => rx,
        Err(err) => {
            error!("Failed to send AwaitRemoteMfaFinishRequest: {err:?}");
            return;
        }
    };

    // Response to AwaitRemoteMfaFinish comes once the user concludes MFA with mobile device.
    // This task then sends the outcome to the WebSocket where the desktop client awaits it.
    set.spawn(async move {
        match rx.await {
            Ok(Payload::AwaitRemoteMfaFinish(response)) => {
                if let Some(ws_response) = remote_mfa_response(&response) {
                    match serde_json::to_string(&ws_response) {
                        Ok(serialized) => {
                            let message = Message::Text(serialized.into());
                            if let Err(err) = ws_tx.send(message).await {
                                error!("Failed to send MFA result via ws: {err:?}");
                            }
                        }
                        Err(err) => {
                            error!("Failed to serialize MFA result for ws: {err:?}");
                        }
                    }
                } else {
                    error!("Received malformed MFA result from Core");
                }
            }
            Ok(_) => {
                error!("Received wrong response type, expected AwaitRemoteMfaFinish");
            }
            Err(err) => {
                error!("Failed to receive MFA result from receiver: {err:?}");
            }
        }

        // Close the websocket once we're done.
        let _ = ws_tx.close().await;
    });

    // Another task to monitor the websocket connection in case desktop client disconnects
    // or the connection errors-out.
    set.spawn(async move {
        while let Some(msg_result) = ws_rx.next().await {
            match msg_result {
                Ok(msg) => {
                    if let Message::Close(_) = msg {
                        break;
                    }
                }
                Err(e) => {
                    error!("Remote desktop mfa WS client listen error {e}");
                    break;
                }
            }
        }
    });

    // Wait for whichever task finishes first and kill the other one.
    let _ = set.join_next().await;
    set.shutdown().await;
}

#[instrument(level = "debug", skip(state, req))]
async fn start_client_mfa(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(req): Json<ClientMfaStartRequest>,
) -> Result<Json<ClientMfaStartResponse>, ApiError> {
    info!("Starting desktop client authorization {req:?}");
    let rx = state.grpc_server.send(
        core_request::Payload::ClientMfaStart(req.clone()),
        device_info,
    )?;
    let payload = get_core_response(rx, None).await?;

    match payload {
        core_response::Payload::ClientMfaStart(response) => {
            info!("Started desktop client authorization {req:?}");
            Ok(Json(response))
        }
        core_response::Payload::DevicePostureRejected(response) => {
            info!("Desktop client failed posture check {response:?}");
            Err(ApiError::PostureRejected(response.failed_posture_checks))
        }
        _ => {
            error!("Received invalid gRPC response type");
            Err(ApiError::InvalidResponseType)
        }
    }
}

#[instrument(level = "debug", skip(state, req))]
async fn step_start_client_mfa(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(req): Json<ClientMfaStepStartRequest>,
) -> Result<Json<ClientMfaStepStartResponse>, ApiError> {
    info!("Starting MFA step for desktop client authorization");
    let rx = state
        .grpc_server
        .send(core_request::Payload::ClientMfaStepStart(req), device_info)?;
    let payload = get_core_response(rx, None).await?;
    if let core_response::Payload::ClientMfaStepStart(response) = payload {
        Ok(Json(response))
    } else {
        error!("Received invalid gRPC response type, expected ClientMfaStepStart");
        Err(ApiError::InvalidResponseType)
    }
}

#[instrument(level = "debug", skip(state, req))]
async fn finish_client_mfa(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(req): Json<ClientMfaFinishRequest>,
) -> Result<Json<ClientMfaFinishResponse>, ApiError> {
    info!("Finishing desktop client authorization");
    let rx = state
        .grpc_server
        .send(core_request::Payload::ClientMfaFinish(req), device_info)?;
    let payload = get_core_response(rx, None).await?;
    if let core_response::Payload::ClientMfaFinish(response) = payload {
        Ok(Json(response))
    } else {
        error!("Received invalid gRPC response type");
        Err(ApiError::InvalidResponseType)
    }
}

#[instrument(level = "debug", skip(state, req))]
async fn finish_remote_mfa(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(req): Json<ClientMfaFinishRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!("Finishing desktop client authorization");
    let rx = state
        .grpc_server
        .send(core_request::Payload::ClientMfaFinish(req), device_info)?;
    if let core_response::Payload::ClientMfaFinish(_response) = get_core_response(rx, None).await? {
        Ok(Json(json!({})))
    } else {
        error!("Received invalid gRPC response type, expected ClientMfaFinish");
        Err(ApiError::InvalidResponseType)
    }
}

#[cfg(test)]
mod tests {
    use super::remote_mfa_response;
    use crate::proto::{
        AwaitRemoteMfaFinishResponse, MfaAdvanced, MfaAwaitingExternal, MfaCompleted,
        MfaStepResult, mfa_step_result,
    };

    #[allow(deprecated)]
    fn response(result: Option<MfaStepResult>) -> AwaitRemoteMfaFinishResponse {
        AwaitRemoteMfaFinishResponse {
            preshared_key: "legacy-psk".to_string(),
            result,
        }
    }

    fn result(outcome: mfa_step_result::Outcome) -> MfaStepResult {
        MfaStepResult {
            outcome: Some(outcome),
        }
    }

    fn serialized(response: &AwaitRemoteMfaFinishResponse) -> serde_json::Value {
        serde_json::from_str(
            &serde_json::to_string(&remote_mfa_response(response).expect("valid MFA result"))
                .expect("MFA response should serialize"),
        )
        .expect("MFA response should be valid JSON")
    }

    #[test]
    fn test_completed_uses_result_envelope() {
        let result = result(mfa_step_result::Outcome::Completed(MfaCompleted {
            preshared_key: "completed-psk".to_string(),
        }));
        let frame = serialized(&response(Some(result.clone())));

        assert_eq!(frame["type"], serde_json::json!("mfa_result"));
        assert_eq!(frame["result"], serde_json::to_value(&result).unwrap());
        assert!(frame.get("preshared_key").is_none());
    }

    #[test]
    fn test_advanced_uses_result_envelope() {
        let result = result(mfa_step_result::Outcome::Advanced(MfaAdvanced {
            next_step: 1,
        }));
        let frame = serialized(&response(Some(result.clone())));

        assert_eq!(frame["type"], serde_json::json!("mfa_result"));
        assert_eq!(frame["result"], serde_json::to_value(&result).unwrap());
        assert!(frame.get("preshared_key").is_none());
    }

    #[test]
    fn test_awaiting_external_uses_result_envelope() {
        let result = result(mfa_step_result::Outcome::AwaitingExternal(
            MfaAwaitingExternal {},
        ));
        let frame = serialized(&response(Some(result.clone())));

        assert_eq!(frame["type"], serde_json::json!("mfa_result"));
        assert_eq!(frame["result"], serde_json::to_value(&result).unwrap());
        assert!(frame.get("preshared_key").is_none());
    }

    #[test]
    fn test_legacy_response_preserves_success_envelope() {
        let response = response(None);
        let serialized = serde_json::to_string(&remote_mfa_response(&response).expect("legacy"))
            .expect("legacy response should serialize");

        assert_eq!(
            serialized,
            r#"{"type":"mfa_success","preshared_key":"legacy-psk"}"#
        );
    }

    #[test]
    fn test_empty_result_fails_closed() {
        let response = response(Some(MfaStepResult { outcome: None }));

        assert!(remote_mfa_response(&response).is_none());
    }
}
