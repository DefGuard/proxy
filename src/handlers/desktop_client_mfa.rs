use std::collections::hash_map::Entry;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    response::{IntoResponse, Response},
    routing::{any, post},
    Json, Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::{sync::oneshot, task::JoinSet};

use crate::{
    error::ApiError,
    handlers::get_core_response,
    http::AppState,
    proto::{
        core_request, core_response, ClientMfaFinishRequest, ClientMfaFinishResponse,
        ClientMfaStartRequest, ClientMfaStartResponse, DeviceInfo,
    },
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start_client_mfa))
        .route("/finish", post(finish_client_mfa))
        .route("/remote", any(await_remote_auth))
        .route("/finish-remote", post(finish_remote_mfa))
}

#[derive(Deserialize)]
pub(crate) struct RemoteMfaRequestQuery {
    pub token: String,
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
        device_info,
    )?;
    let payload = get_core_response(rx).await?;
    if let core_response::Payload::ClientMfaTokenValidation(response) = payload {
        if !response.token_valid {
            return Err(ApiError::Unauthorized(String::new()));
        }
        // check if its already in the map
        let contains_key = {
            let sessions = state.remote_mfa_sessions.lock().await;
            sessions.contains_key(&token)
        };
        if contains_key {
            return Err(ApiError::Unauthorized(String::new()));
        }
        Ok(ws.on_upgrade(move |socket| handle_remote_auth_socket(socket, state.clone(), token)))
    } else {
        Err(ApiError::InvalidResponseType)
    }
}

/// Handle axum web socket upgrade for `await_remote_auth`.
async fn handle_remote_auth_socket(socket: WebSocket, state: AppState, token: String) {
    let (tx, rx) = oneshot::channel();

    {
        let mut sessions = state.remote_mfa_sessions.lock().await;
        match sessions.entry(token.clone()) {
            Entry::Occupied(_) => {
                return;
            }
            Entry::Vacant(v) => {
                v.insert(tx);
            }
        }
    }

    let (mut ws_tx, mut ws_rx) = socket.split();
    let mut set = JoinSet::new();

    set.spawn(async move {
        if let Ok(msg) = rx.await {
            let payload = json!({
                "type": "mfa_success",
                "preshared_key": &msg,
            });
            if let Ok(serialized) = serde_json::to_string(&payload) {
                let message = Message::Text(serialized.into());
                if ws_tx.send(message).await.is_err() {
                    error!("Failed to send preshared key via ws");
                }
            } else {
                error!("Failed to serialize remote mfa ws client response message");
            }
        } else {
            error!("Failed to receive preshared key from receiver");
        }
        let _ = ws_tx.close().await;
    });

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

    let _ = set.join_next().await;
    set.shutdown().await;
    // This will remove token, if it's still there.
    state.remote_mfa_sessions.lock().await.remove(&token);
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
    let payload = get_core_response(rx).await?;

    if let core_response::Payload::ClientMfaStart(response) = payload {
        info!("Started desktop client authorization {req:?}");
        Ok(Json(response))
    } else {
        error!("Received invalid gRPC response type");
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
    let payload = get_core_response(rx).await?;
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
    let payload = get_core_response(rx).await?;
    if let core_response::Payload::ClientMfaFinish(response) = payload {
        // Check if this needs to be forwarded.
        if let Some(token) = response.token {
            let sender_option = {
                let mut sessions = state.remote_mfa_sessions.lock().await;
                sessions.remove(&token)
            };
            if let Some(sender) = sender_option {
                let _ = sender.send(response.preshared_key);
            }
            // If desktop stopped listening for the result, there will be no place to send the
            // result.
            else {
                error!("Remote MFA approve finished but session was not found.");
                return Err(ApiError::Unexpected(String::new()));
            }

            info!("Finished desktop client authorization via mobile device");
            Ok(Json(json!({})))
        } else {
            error!("Remote MFA Unexpected core response, token was not returned");
            Err(ApiError::Unexpected(String::new()))
        }
    } else {
        error!("Received invalid gRPC response type");
        Err(ApiError::InvalidResponseType)
    }
}
