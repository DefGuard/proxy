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
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use base64::{Engine, prelude::BASE64_STANDARD};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{sync::broadcast, time::sleep};

use crate::{
    error::ApiError,
    grpc::VerificationWaiter,
    handlers::get_core_response,
    http::AppState,
    proto::{
        DeviceInfo, DeviceVerificationOutcome, DeviceVerificationStartRequest,
        DeviceVerificationStatus, DeviceVerificationStatusRequest, TotpAlgorithm, core_request,
        core_response,
    },
};

/// Absorbs clock skew between Edge and Core when asking for the final status at expiry.
const EXPIRY_GRACE: Duration = Duration::from_secs(5);
/// Core does not push anything when the device is deleted, so waiters ask again periodically.
const RECHECK_INTERVAL: Duration = Duration::from_secs(5 * 60);

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/start", post(start))
        .route("/status", post(status))
        .route("/await", any(await_decision))
}

#[derive(Deserialize)]
pub(crate) struct StartBody {
    token: String,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Algorithm {
    Sha1,
}

#[derive(Serialize)]
pub(crate) struct StartResponse {
    session_id: i64,
    /// raw RFC 6238 secret in base64, not the base32 used by authenticator apps
    secret: String,
    algorithm: Algorithm,
    digits: u32,
    period: u32,
    expires_at: i64,
}

#[derive(Deserialize)]
pub(crate) struct StatusBody {
    token: String,
    session_id: i64,
}

#[derive(Deserialize)]
pub(crate) struct AwaitQuery {
    session_id: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Outcome {
    Pending,
    Verified,
    Blocked,
    Cancelled,
    Expired,
    /// an outcome from a newer Core, waited on like pending
    Unknown,
}

impl Outcome {
    fn is_final(self) -> bool {
        !matches!(self, Self::Pending | Self::Unknown)
    }
}

impl From<i32> for Outcome {
    fn from(value: i32) -> Self {
        match DeviceVerificationOutcome::try_from(value) {
            Ok(DeviceVerificationOutcome::Pending) => Self::Pending,
            Ok(DeviceVerificationOutcome::Verified) => Self::Verified,
            Ok(DeviceVerificationOutcome::Blocked) => Self::Blocked,
            Ok(DeviceVerificationOutcome::Cancelled) => Self::Cancelled,
            Ok(DeviceVerificationOutcome::Expired) => Self::Expired,
            Ok(DeviceVerificationOutcome::Unspecified) | Err(_) => Self::Unknown,
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub(crate) struct VerificationStatusBody {
    session_id: i64,
    outcome: Outcome,
    expires_at: i64,
    /// set only for an administrator decision
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved_at: Option<i64>,
}

impl From<DeviceVerificationStatus> for VerificationStatusBody {
    fn from(status: DeviceVerificationStatus) -> Self {
        Self {
            session_id: status.session_id,
            outcome: status.outcome.into(),
            expires_at: status.expires_at,
            resolved_at: status.resolved_at,
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SocketMessage {
    Status(VerificationStatusBody),
    Error(serde_json::Value),
}

#[instrument(level = "debug", skip_all)]
async fn start(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(body): Json<StartBody>,
) -> Result<Json<StartResponse>, ApiError> {
    debug!("Starting device verification");
    let rx = state.grpc_server.send(
        core_request::Payload::DeviceVerificationStart(DeviceVerificationStartRequest {
            token: body.token,
        }),
        device_info,
    )?;
    let core_response::Payload::DeviceVerificationStart(response) =
        get_core_response(rx, None).await?
    else {
        error!("Received invalid gRPC response type, expected DeviceVerificationStart");
        return Err(ApiError::InvalidResponseType);
    };
    let algorithm = match TotpAlgorithm::try_from(response.algorithm) {
        Ok(TotpAlgorithm::Sha1) => Algorithm::Sha1,
        _ => {
            error!(
                "Core sent an unsupported TOTP algorithm {} for device verification {}",
                response.algorithm, response.session_id
            );
            return Err(ApiError::InvalidResponseType);
        }
    };
    info!("Device verification {} started", response.session_id);

    Ok(Json(StartResponse {
        session_id: response.session_id,
        secret: BASE64_STANDARD.encode(&response.secret),
        algorithm,
        digits: response.digits,
        period: response.period,
        expires_at: response.expires_at,
    }))
}

#[instrument(level = "debug", skip_all)]
async fn status(
    State(state): State<AppState>,
    device_info: DeviceInfo,
    Json(body): Json<StatusBody>,
) -> Result<Json<VerificationStatusBody>, ApiError> {
    let status = fetch_status(&state, &body.token, body.session_id, device_info).await?;
    Ok(Json(status.into()))
}

pub(crate) async fn fetch_status(
    state: &AppState,
    token: &str,
    session_id: i64,
    device_info: DeviceInfo,
) -> Result<DeviceVerificationStatus, ApiError> {
    let rx = state.grpc_server.send(
        core_request::Payload::DeviceVerificationStatus(DeviceVerificationStatusRequest {
            token: token.to_owned(),
            session_id,
        }),
        device_info,
    )?;
    match get_core_response(rx, None).await? {
        core_response::Payload::DeviceVerificationStatus(status) => Ok(status),
        _ => {
            error!("Received invalid gRPC response type, expected DeviceVerificationStatus");
            Err(ApiError::InvalidResponseType)
        }
    }
}

/// The token comes in the Authorization header to keep it out of URLs and access logs.
#[instrument(level = "debug", skip_all)]
async fn await_decision(
    ws: WebSocketUpgrade,
    Query(query): Query<AwaitQuery>,
    State(state): State<AppState>,
    device_info: DeviceInfo,
    authorization: Option<TypedHeader<Authorization<Bearer>>>,
) -> Result<Response, ApiError> {
    let Some(TypedHeader(Authorization(bearer))) = authorization else {
        return Err(ApiError::Unauthorized("missing bearer token".into()));
    };
    let token = bearer.token().to_owned();
    let session_id = query.session_id;

    // subscribe before asking so a decision pushed in between is not missed
    let waiter = state.grpc_server.verification_waiters.subscribe(session_id);
    let core_connected = state.grpc_server.subscribe_core_connected();
    // token and session errors are answered over HTTP, before the upgrade
    let current = fetch_status(&state, &token, session_id, device_info.clone()).await?;
    debug!("Client awaits the outcome of device verification {session_id}");

    Ok(ws.on_upgrade(move |socket| async move {
        let decision =
            wait_for_decision(&state, &token, device_info, current, waiter, core_connected);
        handle_socket(socket, decision).await;
    }))
}

async fn handle_socket(
    socket: WebSocket,
    decision: impl Future<Output = Result<DeviceVerificationStatus, ApiError>>,
) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let client_gone = async {
        while let Some(message) = ws_rx.next().await {
            match message {
                Ok(Message::Close(_)) => break,
                Ok(_) => {}
                Err(err) => {
                    debug!("Device verification websocket client error: {err}");
                    break;
                }
            }
        }
    };

    let message = tokio::select! {
        result = decision => match result {
            Ok(status) => SocketMessage::Status(status.into()),
            Err(err) => {
                warn!("Stopped waiting for device verification: {err}");
                SocketMessage::Error(err.status_and_body().1)
            }
        },
        () = client_gone => {
            debug!("Client stopped waiting for device verification");
            return;
        }
    };
    match serde_json::to_string(&message) {
        Ok(serialized) => {
            if let Err(err) = ws_tx.send(Message::Text(serialized.into())).await {
                error!("Failed to send device verification outcome via websocket: {err}");
            }
        }
        Err(err) => error!("Failed to serialize device verification outcome: {err}"),
    }
    let _ = ws_tx.close().await;
}

/// Core being away or slow surfaces as these, so they are retried.
fn is_transient(err: &ApiError) -> bool {
    matches!(err, ApiError::CoreTimeout | ApiError::Unexpected(_))
}

/// Core pushes are best effort, so Core is also asked on reconnect, at expiry and periodically.
pub(crate) async fn wait_for_decision(
    state: &AppState,
    token: &str,
    device_info: DeviceInfo,
    mut current: DeviceVerificationStatus,
    mut waiter: VerificationWaiter,
    mut core_connected: broadcast::Receiver<()>,
) -> Result<DeviceVerificationStatus, ApiError> {
    let session_id = current.session_id;
    loop {
        if Outcome::from(current.outcome).is_final() {
            return Ok(current);
        }
        let until_expiry = u64::try_from(current.expires_at - chrono::Utc::now().timestamp())
            .ok()
            .map(|secs| Duration::from_secs(secs) + EXPIRY_GRACE);

        tokio::select! {
            pushed = waiter.recv() => {
                current = pushed;
                continue;
            }
            received = core_connected.recv() => {
                if let Err(broadcast::error::RecvError::Closed) = received {
                    // the server is shutting down, no push or reconnect can come any more
                    return Err(ApiError::Unexpected("Edge is shutting down".into()));
                }
                debug!("Core reconnected, checking device verification {session_id}");
            }
            () = sleep(until_expiry.unwrap_or(RECHECK_INTERVAL)), if until_expiry.is_some() => {
                debug!("Device verification {session_id} should have expired, checking");
            }
            () = sleep(RECHECK_INTERVAL) => {
                debug!("Checking device verification {session_id}");
            }
        }

        match fetch_status(state, token, session_id, device_info.clone()).await {
            Ok(status) => current = status,
            Err(err) if is_transient(&err) => {
                warn!("Failed to check device verification {session_id}, will retry: {err}");
            }
            Err(err) => return Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outcome_maps_every_proto_value() {
        for (value, outcome) in [
            (DeviceVerificationOutcome::Unspecified, Outcome::Unknown),
            (DeviceVerificationOutcome::Pending, Outcome::Pending),
            (DeviceVerificationOutcome::Verified, Outcome::Verified),
            (DeviceVerificationOutcome::Blocked, Outcome::Blocked),
            (DeviceVerificationOutcome::Cancelled, Outcome::Cancelled),
            (DeviceVerificationOutcome::Expired, Outcome::Expired),
        ] {
            assert_eq!(Outcome::from(value as i32), outcome);
        }
        assert_eq!(Outcome::from(99), Outcome::Unknown);
        assert!(!Outcome::Pending.is_final());
        assert!(!Outcome::Unknown.is_final());
        assert!(Outcome::Verified.is_final());
        assert!(Outcome::Expired.is_final());
    }

    #[test]
    fn test_socket_messages_are_tagged() {
        let status = SocketMessage::Status(VerificationStatusBody {
            session_id: 3,
            outcome: Outcome::Blocked,
            expires_at: 100,
            resolved_at: Some(50),
        });
        assert_eq!(
            serde_json::to_value(&status).unwrap(),
            serde_json::json!({
                "type": "status",
                "session_id": 3,
                "outcome": "blocked",
                "expires_at": 100,
                "resolved_at": 50,
            })
        );

        let error = SocketMessage::Error(
            ApiError::Unauthorized("invalid token".into())
                .status_and_body()
                .1,
        );
        assert_eq!(
            serde_json::to_value(&error).unwrap(),
            serde_json::json!({"type": "error", "error": "invalid token", "code": null})
        );
    }
}
