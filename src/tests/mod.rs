use std::{
    env::temp_dir,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, RwLock},
};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use axum_extra::extract::cookie::Key;
use serde::Serialize;
use tokio::sync::{Mutex, broadcast, mpsc};
use tower::ServiceExt;

use crate::{
    grpc::ProxyServer,
    http::{AppState, api_router},
    proto::{CoreError, core_request, core_response},
};

mod mfa_config;
mod mtls;

pub(super) fn cookie_key() -> Arc<RwLock<Option<Key>>> {
    Arc::new(RwLock::new(Some(Key::generate())))
}

pub(super) fn build_proxy_server(cookie_key: Arc<RwLock<Option<Key>>>) -> ProxyServer {
    let (reset_tx, _) = broadcast::channel(1);
    let (https_cert_tx, _) = broadcast::channel(1);
    let (clear_https_tx, _) = broadcast::channel(1);
    let (_, logs_rx) = mpsc::channel(1);
    ProxyServer::new(
        cookie_key,
        temp_dir(),
        reset_tx,
        https_cert_tx,
        clear_https_tx,
        None,
        Arc::new(Mutex::new(logs_rx)),
        false,
    )
}

/// The API router backed by a fake Core that answers every request with `core`.
///
/// `core` receives each request payload and returns the response payload. It runs on a
/// separate task, so a handler can await the response like in production. A panic inside
/// `core` would only kill that task, so it is turned into an internal `CoreError` that
/// carries the panic message back through the handler under test.
pub(super) fn app_with_fake_core<F>(core: F) -> Router
where
    F: Fn(core_request::Payload) -> core_response::Payload + Send + 'static,
{
    let cookie_key = cookie_key();
    let server = build_proxy_server(Arc::clone(&cookie_key));
    let mut requests = server.connect_fake_core();
    let responder = server.clone();
    tokio::spawn(async move {
        while let Some(Ok(request)) = requests.recv().await {
            let payload = request.payload.expect("request without payload");
            let response = catch_unwind(AssertUnwindSafe(|| core(payload))).unwrap_or_else(|err| {
                let message = err
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| err.downcast_ref::<&str>().map(ToString::to_string))
                    .unwrap_or_default();
                core_response::Payload::CoreError(CoreError {
                    status_code: tonic::Code::Internal as i32,
                    message: format!("fake Core panicked: {message}"),
                })
            });
            responder.respond(request.id, response);
        }
    });
    api_router().with_state(AppState {
        grpc_server: server,
        cookie_key,
    })
}

/// Sends a JSON POST like a desktop client and returns the status with the JSON body.
///
/// The `X-Forwarded-For` header satisfies the `DeviceInfo` extractor without a socket.
pub(super) async fn post_json<T: Serialize>(
    app: &Router,
    path: &str,
    body: &T,
) -> (StatusCode, serde_json::Value) {
    let request = Request::builder()
        .method("POST")
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-Forwarded-For", "10.0.0.1")
        .body(Body::from(serde_json::to_vec(body).unwrap()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json = if bytes.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::String(
            String::from_utf8_lossy(&bytes).into_owned(),
        ))
    };
    (status, json)
}
