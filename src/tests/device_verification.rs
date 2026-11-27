use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::http::{StatusCode, header};
use futures_util::StreamExt;
use serde_json::{Value, json};
use tokio::{
    net::TcpListener,
    sync::mpsc::UnboundedReceiver,
    task::JoinHandle,
    time::{sleep, timeout},
};
use tokio_tungstenite::tungstenite::{self, client::IntoClientRequest};
use tonic::{Code, Status};

use super::support::{app_with_fake_core, cookie_key, post_json, test_proxy_server};
use crate::{
    enterprise::handlers::device_verification::wait_for_decision,
    error::ApiError,
    grpc::ProxyServer,
    http::{AppState, api_router},
    proto::{
        CoreError, CoreErrorCode, CoreRequest, DeviceInfo, DeviceVerificationOutcome,
        DeviceVerificationStartRequest, DeviceVerificationStartResponse, DeviceVerificationStatus,
        DeviceVerificationStatusRequest, TotpAlgorithm, core_request, core_response,
    },
};

const TOKEN: &str = "polling-token";
const SESSION_ID: i64 = 42;

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

fn status(session_id: i64, outcome: DeviceVerificationOutcome) -> DeviceVerificationStatus {
    DeviceVerificationStatus {
        session_id,
        outcome: outcome.into(),
        expires_at: now() + 3600,
        resolved_at: None,
    }
}

fn core_error(code: Code, message: &str, error_code: CoreErrorCode) -> core_response::Payload {
    core_response::Payload::CoreError(CoreError {
        status_code: code as i32,
        message: message.into(),
        code: error_code.into(),
    })
}

fn device_info() -> DeviceInfo {
    DeviceInfo {
        ip_address: "10.0.0.1".into(),
        user_agent: None,
        version: None,
        platform: None,
    }
}

type CoreRequests = UnboundedReceiver<Result<CoreRequest, Status>>;

fn manual_core() -> (ProxyServer, AppState, CoreRequests) {
    let cookie_key = cookie_key();
    let server = test_proxy_server(Arc::clone(&cookie_key));
    let requests = server.register_test_client();
    let state = AppState {
        grpc_server: server.clone(),
        cookie_key,
    };
    (server, state, requests)
}

async fn expect_status_request(requests: &mut CoreRequests) -> u64 {
    expect_status_request_within(requests, Duration::from_secs(5)).await
}

async fn expect_status_request_within(requests: &mut CoreRequests, limit: Duration) -> u64 {
    let request = timeout(limit, requests.recv())
        .await
        .expect("Edge did not ask Core for the status")
        .unwrap()
        .unwrap();
    match request.payload {
        Some(core_request::Payload::DeviceVerificationStatus(
            DeviceVerificationStatusRequest { token, session_id },
        )) => {
            assert_eq!(token, TOKEN);
            assert_eq!(session_id, SESSION_ID);
        }
        _ => panic!("unexpected Core request"),
    }
    request.id
}

fn spawn_wait(
    server: &ProxyServer,
    state: AppState,
    current: DeviceVerificationStatus,
) -> JoinHandle<Result<DeviceVerificationStatus, ApiError>> {
    let waiter = server.verification_waiters.subscribe(SESSION_ID);
    let core_connected = server.subscribe_core_connected();
    tokio::spawn(async move {
        wait_for_decision(
            &state,
            TOKEN,
            device_info(),
            current,
            waiter,
            core_connected,
        )
        .await
    })
}

async fn finished<T>(handle: JoinHandle<T>) -> T {
    timeout(Duration::from_secs(5), handle)
        .await
        .expect("waiting did not finish")
        .unwrap()
}

#[tokio::test]
async fn test_start_forwards_token_and_encodes_secret() {
    let app = app_with_fake_core(|payload| match payload {
        core_request::Payload::DeviceVerificationStart(DeviceVerificationStartRequest {
            token,
        }) => {
            assert_eq!(token, TOKEN);
            core_response::Payload::DeviceVerificationStart(DeviceVerificationStartResponse {
                session_id: SESSION_ID,
                secret: vec![0, 1, 2, 255],
                algorithm: TotpAlgorithm::Sha1.into(),
                digits: 6,
                period: 60,
                expires_at: 1_800_000_000,
            })
        }
        _ => panic!("unexpected Core request"),
    });

    let (status, body) = post_json(
        &app,
        "/api/v1/client/verification/start",
        &json!({"token": TOKEN}),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        json!({
            "session_id": SESSION_ID,
            "secret": "AAEC/w==",
            "algorithm": "sha1",
            "digits": 6,
            "period": 60,
            "expires_at": 1_800_000_000,
        })
    );
}

#[tokio::test]
async fn test_start_rejects_unsupported_algorithm() {
    let app = app_with_fake_core(|_| {
        core_response::Payload::DeviceVerificationStart(DeviceVerificationStartResponse {
            session_id: SESSION_ID,
            secret: vec![1],
            algorithm: TotpAlgorithm::Unspecified.into(),
            digits: 6,
            period: 60,
            expires_at: 1_800_000_000,
        })
    });

    let (status, _) = post_json(
        &app,
        "/api/v1/client/verification/start",
        &json!({"token": TOKEN}),
    )
    .await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_status_forwards_session_and_names_outcomes() {
    let app = app_with_fake_core(|payload| match payload {
        core_request::Payload::DeviceVerificationStatus(DeviceVerificationStatusRequest {
            token,
            session_id,
        }) => {
            assert_eq!(token, TOKEN);
            core_response::Payload::DeviceVerificationStatus(DeviceVerificationStatus {
                session_id,
                outcome: DeviceVerificationOutcome::Verified.into(),
                expires_at: 1_800_000_000,
                resolved_at: Some(1_799_999_000),
            })
        }
        _ => panic!("unexpected Core request"),
    });

    let (status, body) = post_json(
        &app,
        "/api/v1/client/verification/status",
        &json!({"token": TOKEN, "session_id": SESSION_ID}),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        json!({
            "session_id": SESSION_ID,
            "outcome": "verified",
            "expires_at": 1_800_000_000,
            "resolved_at": 1_799_999_000,
        })
    );
}

#[tokio::test]
async fn test_core_errors_reach_the_client_with_their_codes() {
    for (error, expected_status, expected_code) in [
        (
            core_error(
                Code::Unauthenticated,
                "invalid token",
                CoreErrorCode::DeviceTokenInvalid,
            ),
            StatusCode::UNAUTHORIZED,
            json!("device_token_invalid"),
        ),
        (
            core_error(
                Code::PermissionDenied,
                "user inactive",
                CoreErrorCode::UserInactive,
            ),
            StatusCode::FORBIDDEN,
            json!("user_inactive"),
        ),
        (
            core_error(
                Code::PermissionDenied,
                "device blocked",
                CoreErrorCode::DeviceBlocked,
            ),
            StatusCode::FORBIDDEN,
            json!("device_blocked"),
        ),
        (
            core_error(
                Code::NotFound,
                "verification session not found",
                CoreErrorCode::VerificationSessionNotFound,
            ),
            StatusCode::NOT_FOUND,
            json!("verification_session_not_found"),
        ),
        (
            core_error(
                Code::FailedPrecondition,
                "no valid license",
                CoreErrorCode::LicenseRequired,
            ),
            StatusCode::PAYMENT_REQUIRED,
            json!("license_required"),
        ),
    ] {
        let app = app_with_fake_core(move |_| error.clone());
        for (path, body) in [
            ("/api/v1/client/verification/start", json!({"token": TOKEN})),
            (
                "/api/v1/client/verification/status",
                json!({"token": TOKEN, "session_id": SESSION_ID}),
            ),
        ] {
            let (status, body) = post_json(&app, path, &body).await;
            assert_eq!(status, expected_status, "{path}");
            assert_eq!(body["code"], expected_code, "{path}");
        }
    }
}

#[tokio::test]
async fn test_resolved_push_ends_the_wait() {
    let (server, state, _requests) = manual_core();
    let wait = spawn_wait(
        &server,
        state,
        status(SESSION_ID, DeviceVerificationOutcome::Pending),
    );

    assert!(
        !server
            .verification_waiters
            .notify(status(SESSION_ID + 1, DeviceVerificationOutcome::Verified))
    );
    sleep(Duration::from_millis(50)).await;
    assert!(!wait.is_finished());

    assert!(
        server
            .verification_waiters
            .notify(status(SESSION_ID, DeviceVerificationOutcome::Blocked))
    );
    let outcome = finished(wait).await.unwrap();
    assert_eq!(
        outcome.outcome,
        i32::from(DeviceVerificationOutcome::Blocked)
    );
    assert!(!server.verification_waiters.is_waiting(SESSION_ID));
}

/// Edge subscribes before asking Core, so a decision pushed before the answer must be kept.
#[tokio::test]
async fn test_push_before_the_status_answer_is_not_lost() {
    let (server, state, _requests) = manual_core();
    let waiter = server.verification_waiters.subscribe(SESSION_ID);
    let core_connected = server.subscribe_core_connected();
    server
        .verification_waiters
        .notify(status(SESSION_ID, DeviceVerificationOutcome::Verified));

    let outcome = timeout(
        Duration::from_secs(5),
        wait_for_decision(
            &state,
            TOKEN,
            device_info(),
            status(SESSION_ID, DeviceVerificationOutcome::Pending),
            waiter,
            core_connected,
        ),
    )
    .await
    .expect("the early push was lost")
    .unwrap();

    assert_eq!(
        outcome.outcome,
        i32::from(DeviceVerificationOutcome::Verified)
    );
}

#[tokio::test]
async fn test_core_reconnect_asks_for_the_missed_decision() {
    let (server, state, mut requests) = manual_core();
    let wait = spawn_wait(
        &server,
        state,
        status(SESSION_ID, DeviceVerificationOutcome::Pending),
    );

    server.announce_test_core_connected();
    let id = expect_status_request(&mut requests).await;
    server.resolve_test_response(
        id,
        core_response::Payload::DeviceVerificationStatus(status(
            SESSION_ID,
            DeviceVerificationOutcome::Pending,
        )),
    );
    sleep(Duration::from_millis(50)).await;
    assert!(!wait.is_finished());

    server.announce_test_core_connected();
    let id = expect_status_request(&mut requests).await;
    server.resolve_test_response(
        id,
        core_response::Payload::DeviceVerificationStatus(status(
            SESSION_ID,
            DeviceVerificationOutcome::Cancelled,
        )),
    );

    let outcome = finished(wait).await.unwrap();
    assert_eq!(
        outcome.outcome,
        i32::from(DeviceVerificationOutcome::Cancelled)
    );
}

#[tokio::test]
async fn test_transient_errors_keep_waiting_and_final_errors_end_it() {
    let (server, state, mut requests) = manual_core();
    let wait = spawn_wait(
        &server,
        state,
        status(SESSION_ID, DeviceVerificationOutcome::Pending),
    );

    server.announce_test_core_connected();
    let id = expect_status_request(&mut requests).await;
    server.resolve_test_response(
        id,
        core_error(
            Code::Internal,
            "unexpected error",
            CoreErrorCode::Unspecified,
        ),
    );
    sleep(Duration::from_millis(50)).await;
    assert!(!wait.is_finished());

    // the device was deleted while its owner waited
    server.announce_test_core_connected();
    let id = expect_status_request(&mut requests).await;
    server.resolve_test_response(
        id,
        core_error(
            Code::Unauthenticated,
            "invalid token",
            CoreErrorCode::DeviceTokenInvalid,
        ),
    );

    let error = finished(wait).await.unwrap_err();
    assert!(matches!(
        error,
        ApiError::WithCode(_, CoreErrorCode::DeviceTokenInvalid)
    ));
}

#[tokio::test(start_paused = true)]
async fn test_expiry_asks_core_for_the_final_status() {
    let (server, state, mut requests) = manual_core();
    let mut current = status(SESSION_ID, DeviceVerificationOutcome::Pending);
    current.expires_at = now() + 1;
    let wait = spawn_wait(&server, state, current);

    // paused time jumps to the expiry once idle, which comes before the periodic recheck
    let id = expect_status_request_within(&mut requests, Duration::from_secs(60)).await;
    server.resolve_test_response(
        id,
        core_response::Payload::DeviceVerificationStatus(status(
            SESSION_ID,
            DeviceVerificationOutcome::Expired,
        )),
    );

    let outcome = finished(wait).await.unwrap();
    assert_eq!(
        outcome.outcome,
        i32::from(DeviceVerificationOutcome::Expired)
    );
}

#[tokio::test]
async fn test_registry_entry_lives_while_anyone_waits() {
    let server = test_proxy_server(cookie_key());
    let waiters = &server.verification_waiters;
    let first = waiters.subscribe(SESSION_ID);
    let second = waiters.subscribe(SESSION_ID);

    drop(first);
    assert!(waiters.is_waiting(SESSION_ID));
    drop(second);
    assert!(!waiters.is_waiting(SESSION_ID));
}

async fn serve(state: AppState) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = api_router().with_state(state);
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    addr
}

fn await_request(addr: SocketAddr, token: Option<&str>) -> tungstenite::handshake::client::Request {
    let mut request =
        format!("ws://{addr}/api/v1/client/verification/await?session_id={SESSION_ID}")
            .into_client_request()
            .unwrap();
    if let Some(token) = token {
        request.headers_mut().insert(
            header::AUTHORIZATION,
            format!("Bearer {token}").parse().unwrap(),
        );
    }
    request
}

async fn next_json(
    socket: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) -> Value {
    let message = timeout(Duration::from_secs(5), socket.next())
        .await
        .expect("no websocket message")
        .unwrap()
        .unwrap();
    serde_json::from_str(message.to_text().unwrap()).unwrap()
}

#[tokio::test]
async fn test_websocket_delivers_the_pushed_decision() {
    let (server, state, mut requests) = manual_core();
    let addr = serve(state).await;

    let responder = server.clone();
    let answer = tokio::spawn(async move {
        let id = expect_status_request(&mut requests).await;
        responder.resolve_test_response(
            id,
            core_response::Payload::DeviceVerificationStatus(status(
                SESSION_ID,
                DeviceVerificationOutcome::Pending,
            )),
        );
    });
    let (mut socket, _) = tokio_tungstenite::connect_async(await_request(addr, Some(TOKEN)))
        .await
        .unwrap();
    answer.await.unwrap();
    assert!(server.verification_waiters.is_waiting(SESSION_ID));

    let mut decision = status(SESSION_ID, DeviceVerificationOutcome::Verified);
    decision.resolved_at = Some(now());
    server.verification_waiters.notify(decision);

    assert_eq!(
        next_json(&mut socket).await,
        json!({
            "type": "status",
            "session_id": SESSION_ID,
            "outcome": "verified",
            "expires_at": decision.expires_at,
            "resolved_at": decision.resolved_at,
        })
    );
    assert!(matches!(
        timeout(Duration::from_secs(5), socket.next())
            .await
            .unwrap(),
        Some(Ok(tungstenite::Message::Close(_))) | None
    ));
}

#[tokio::test]
async fn test_websocket_answers_a_decided_session_at_once() {
    let app_core = |payload| match payload {
        core_request::Payload::DeviceVerificationStatus(_) => {
            core_response::Payload::DeviceVerificationStatus(DeviceVerificationStatus {
                session_id: SESSION_ID,
                outcome: DeviceVerificationOutcome::Blocked.into(),
                expires_at: 1_800_000_000,
                resolved_at: Some(1_799_999_000),
            })
        }
        _ => panic!("unexpected Core request"),
    };
    let (server, state, mut requests) = manual_core();
    let addr = serve(state).await;
    let responder = server.clone();
    tokio::spawn(async move {
        while let Some(Ok(request)) = requests.recv().await {
            responder.resolve_test_response(request.id, app_core(request.payload.unwrap()));
        }
    });

    let (mut socket, _) = tokio_tungstenite::connect_async(await_request(addr, Some(TOKEN)))
        .await
        .unwrap();

    assert_eq!(next_json(&mut socket).await["outcome"], "blocked");
}

#[tokio::test]
async fn test_websocket_errors_are_answered_before_the_upgrade() {
    let (server, state, mut requests) = manual_core();
    let addr = serve(state).await;

    let error = tokio_tungstenite::connect_async(await_request(addr, None))
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        tungstenite::Error::Http(response) if response.status() == StatusCode::UNAUTHORIZED
    ));
    assert!(requests.try_recv().is_err());

    let responder = server.clone();
    tokio::spawn(async move {
        let id = expect_status_request(&mut requests).await;
        responder.resolve_test_response(
            id,
            core_error(
                Code::NotFound,
                "verification session not found",
                CoreErrorCode::VerificationSessionNotFound,
            ),
        );
    });
    let error = tokio_tungstenite::connect_async(await_request(addr, Some(TOKEN)))
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        tungstenite::Error::Http(response) if response.status() == StatusCode::NOT_FOUND
    ));
    assert!(!server.verification_waiters.is_waiting(SESSION_ID));
}
