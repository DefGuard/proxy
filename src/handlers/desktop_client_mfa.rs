use std::time::Duration;

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
use tokio::task::JoinSet;

use crate::{
    error::ApiError,
    handlers::get_core_response,
    http::AppState,
    proto::{
        core_request,
        core_response::{self, Payload},
        ClientMfaFinishRequest, ClientMfaFinishResponse, ClientMfaStartRequest,
        ClientMfaStartResponse, ClientRemoteMfaFinishRequest, DeviceInfo,
    },
};

// How much time the user has to approve remote MFA with mobile device
const REMOTE_AUTH_TIMEOUT: Duration = Duration::from_secs(60);

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

    let request = ClientRemoteMfaFinishRequest { token };
    let rx = match state.grpc_server.send(
        core_request::Payload::ClientRemoteMfaFinish(request),
        device_info,
    ) {
        Ok(rx) => rx,
        Err(err) => {
            error!("Failed to send ClientRemoteMfaFinishRequest: {err:?}");
            return;
        }
    };

	// Response to ClientRemoteMfaFinishRequest comes once the user concludes MFA with mobile device.
	// This task then sends the preshared key to the WebSocket where desktop client awaits for it.
    set.spawn(async move {
        match rx.await {
            Ok(Payload::ClientRemoteMfaFinish(response)) => {
                let ws_response = json!({
                    "type": "mfa_success",
                    "preshared_key": &response.preshared_key,
                });
                if let Ok(serialized) = serde_json::to_string(&ws_response) {
                    let message = Message::Text(serialized.into());
                    if let Err(err) = ws_tx.send(message).await {
                        error!("Failed to send preshared key via ws: {err:?}");
                    }
                }
            }
            Ok(_) => {
                error!("Received wrong response type, expected ClientRemoteMfaFinish");
            }
            Err(err) => {
                error!("Failed to receive preshared key from receiver: {err:?}");
            }
        };

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
    // TODO(jck) can we make the response proto::Empty here?
    if let core_response::Payload::ClientMfaFinish(_response) = get_core_response(rx, None).await? {
        Ok(Json(json!({})))
    } else {
        error!("Received invalid gRPC response type");
        Err(ApiError::InvalidResponseType)
    }
}
