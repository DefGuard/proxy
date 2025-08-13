use axum::{
    extract::{ws::WebSocket, Path, State, WebSocketUpgrade},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc;

use crate::{
    error::ApiError,
    handlers::get_core_response,
    http::AppState,
    proto::{
        core_request, core_response, ClientMfaFinishRequest, ClientMfaStartRequest,
        ClientMfaStartResponse, DeviceInfo,
    },
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start_client_mfa))
        .route("/finish", post(finish_client_mfa))
        .route("/remote/:token", post(await_remote_auth))
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RemoteMfaRequest {
    pub token: String,
}

#[instrument(level = "debug", skip(state))]
async fn await_remote_auth(
    ws: WebSocketUpgrade,
    //TODO: Validate this smth ?
    Path(token): Path<String>,
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(req): Json<RemoteMfaRequest>,
) -> Response {
    // TODO: validate token!
    // make sure no one else is listening already
    match state.remote_mfa_sessions.get(&token) {
        Some(_) => {
            return ApiError::Unauthorized("Session already exists".into()).into_response();
        }
        None => {}
    };
    ws.on_upgrade(move |socket| handle_remote_auth_socket(socket, state.clone(), token))
}

async fn handle_remote_auth_socket(socket: WebSocket, state: AppState, token: String) {
    let (tx, mut rx) = mpsc::channel::<String>(10);
    let (mut ws_tx, mut ws_rx) = socket.split();
    // include the current session in the state
    state.remote_mfa_sessions.insert(token.clone(), tx.clone());

    let forwarder = {
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let message = axum::extract::ws::Message::Text(msg);
                if ws_tx.send(message).await.is_err() {
                    break;
                }
            }
            let _ = ws_tx.close().await;
        })
    };
    while let Some(Ok(axum::extract::ws::Message::Close(_))) = ws_rx.next().await {
        break;
    }
    //cleanup
    state.remote_mfa_sessions.remove(&token);
    forwarder.abort();
}

#[instrument(level = "debug", skip(state))]
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
        error!("Received invalid gRPC response type: {payload:#?}");
        Err(ApiError::InvalidResponseType)
    }
}

#[instrument(level = "debug", skip(state))]
async fn finish_client_mfa(
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
        // check if this needs to be forwarded
        match response.token {
            // means mobile approve auth method was used
            Some(token) => {
                match state.remote_mfa_sessions.get(&token) {
                    Some(sender) => {
                        // send preshared key to the device that is awaiting this token
                        let _ = sender.send(response.preshared_key).await;
                    }
                    None => {
                        error!("Remote MFA approve finished but session was not found.");
                    }
                };
                info!("Finished desktop client authorization via mobile device");
                Ok(Json(json!({})))
            }
            None => {
                info!("Finished desktop authorization");
                Ok(Json(json!(response)))
            }
        }
    } else {
        error!("Received invalid gRPC response type: {payload:#?}");
        Err(ApiError::InvalidResponseType)
    }
}
