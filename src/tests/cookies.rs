use std::{
    path::PathBuf,
    sync::{Arc, RwLock, atomic::AtomicBool},
};

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use axum_extra::extract::cookie::{Cookie, Key, SameSite};
use tokio::sync::{Mutex, broadcast, mpsc};
use tower::ServiceExt;

use crate::{
    grpc::ProxyServer,
    http::{AppState, ENROLLMENT_COOKIE_NAME, PASSWORD_RESET_COOKIE_NAME, build_router},
    proto::{EnrollmentStartResponse, PasswordResetStartResponse, PublicSettings, core_response},
};

fn test_proxy_server(cookie_key: Arc<RwLock<Option<Key>>>) -> ProxyServer {
    let (reset_tx, _) = broadcast::channel(1);
    let (https_cert_tx, _) = broadcast::channel(1);
    let (clear_https_tx, _) = broadcast::channel(1);
    let (_, logs_rx) = mpsc::channel(1);
    ProxyServer::new(
        cookie_key,
        PathBuf::new(),
        reset_tx,
        https_cert_tx,
        clear_https_tx,
        None,
        Arc::new(Mutex::new(logs_rx)),
        false,
    )
}

fn public_settings(public_url: &str) -> PublicSettings {
    PublicSettings {
        display_password_reset: true,
        display_download_step: true,
        public_url: Some(public_url.to_owned()),
    }
}

fn enrollment_response() -> core_response::Payload {
    core_response::Payload::EnrollmentStart(EnrollmentStartResponse {
        admin: None,
        user: None,
        deadline_timestamp: 4_000_000_000,
        final_page_content: String::new(),
        instance: None,
        settings: None,
    })
}

fn password_reset_response() -> core_response::Payload {
    core_response::Payload::PasswordResetStart(PasswordResetStartResponse {
        deadline_timestamp: 4_000_000_000,
    })
}

async fn request_cookie(
    public_url: Option<&str>,
    request_path: &'static str,
    response_payload: core_response::Payload,
) -> Cookie<'static> {
    let cookie_key = Arc::new(RwLock::new(Some(Key::generate())));
    let server = test_proxy_server(Arc::clone(&cookie_key));
    if let Some(public_url) = public_url {
        server.apply_test_public_settings(public_settings(public_url));
    }

    let mut core_requests = server.register_test_client();
    let response_server = server.clone();
    let state = AppState::for_test(server, cookie_key);
    let app = build_router(state, |router| router, Arc::new(AtomicBool::new(false)))
        .expect("Failed to build test router");
    let request = Request::builder()
        .method("POST")
        .uri(request_path)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-for", "127.0.0.1")
        .body(Body::from(r#"{"token":"test-token"}"#))
        .expect("Failed to build test request");

    let (response, ()) = tokio::join!(app.oneshot(request), async move {
        let Some(request) = core_requests.recv().await else {
            panic!("Test client channel closed");
        };
        let Ok(request) = request else {
            panic!("Test client received an error");
        };
        response_server.resolve_test_response(request.id, response_payload);
    });
    let response = response.expect("Test router request failed");
    assert_eq!(response.status(), StatusCode::OK);

    let set_cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .expect("Response did not contain Set-Cookie")
        .to_str()
        .expect("Set-Cookie was not valid ASCII");
    Cookie::parse(set_cookie.to_owned()).expect("Set-Cookie was not a valid cookie")
}

fn assert_cookie(cookie: &Cookie<'_>, name: &str, path: &str, secure: bool) {
    assert_eq!(cookie.name(), name);
    assert_eq!(cookie.http_only(), Some(true));
    assert_eq!(cookie.same_site(), Some(SameSite::Strict));
    assert_eq!(cookie.secure(), secure.then_some(true));
    assert_eq!(cookie.path(), Some(path));
}

#[tokio::test]
async fn test_enrollment_cookie_attributes_follow_core_public_url() {
    for (public_url, secure) in [
        (Some("https://proxy.example.com"), true),
        (Some("http://proxy.example.com"), false),
        (None, true),
        (Some("not a URL"), true),
    ] {
        let cookie = request_cookie(
            public_url,
            "/api/v1/enrollment/start",
            enrollment_response(),
        )
        .await;
        assert_cookie(
            &cookie,
            ENROLLMENT_COOKIE_NAME,
            "/api/v1/enrollment",
            secure,
        );
    }
}

#[tokio::test]
async fn test_password_reset_cookie_attributes_follow_core_public_url() {
    for (public_url, secure) in [
        (Some("https://proxy.example.com"), true),
        (Some("http://proxy.example.com"), false),
        (None, true),
        (Some("not a URL"), true),
    ] {
        let cookie = request_cookie(
            public_url,
            "/api/v1/password-reset/start",
            password_reset_response(),
        )
        .await;
        assert_cookie(
            &cookie,
            PASSWORD_RESET_COOKIE_NAME,
            "/api/v1/password-reset",
            secure,
        );
    }
}
