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
use tonic::Status;
use tower::ServiceExt;

use crate::{
    grpc::ProxyServer,
    http::{AppState, ENROLLMENT_COOKIE_NAME, PASSWORD_RESET_COOKIE_NAME, build_router},
    proto::{
        AuthInfoResponse, CoreRequest, EnrollmentStartResponse, PasswordResetStartResponse,
        PublicSettings, core_response,
    },
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

fn auth_info_response() -> core_response::Payload {
    core_response::Payload::AuthInfo(AuthInfoResponse {
        url: "https://idp.example.com/authorize".to_owned(),
        csrf_token: "csrf-token".to_owned(),
        nonce: "nonce".to_owned(),
        button_display_name: None,
    })
}

async fn request_with_core_response(
    app: &axum::Router,
    core_requests: &mut mpsc::UnboundedReceiver<Result<CoreRequest, Status>>,
    response_server: &ProxyServer,
    request: Request<Body>,
    response_payload: core_response::Payload,
) -> axum::response::Response {
    let (response, ()) = tokio::join!(app.clone().oneshot(request), async {
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
    response
}

fn parse_set_cookies(response: &axum::response::Response) -> Vec<Cookie<'static>> {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|set_cookie| {
            let set_cookie = set_cookie
                .to_str()
                .expect("Set-Cookie was not valid ASCII")
                .to_owned();
            Cookie::parse(set_cookie).expect("Set-Cookie was not a valid cookie")
        })
        .collect()
}

fn cookie_header(cookies: &[Cookie<'_>]) -> String {
    cookies
        .iter()
        .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
        .collect::<Vec<_>>()
        .join("; ")
}

async fn request_cookie_removal(
    public_url: &'static str,
    start_path: &'static str,
    start_body: &'static str,
    start_payload: core_response::Payload,
    finish_path: &'static str,
    finish_body: &'static str,
    finish_payload: core_response::Payload,
) -> Vec<Cookie<'static>> {
    let cookie_key = Arc::new(RwLock::new(Some(Key::generate())));
    let server = test_proxy_server(Arc::clone(&cookie_key));
    server.apply_test_public_settings(public_settings(public_url));

    let mut core_requests = server.register_test_client();
    let response_server = server.clone();
    let state = AppState::for_test(server, cookie_key);
    let app = build_router(state, |router| router, Arc::new(AtomicBool::new(false)))
        .expect("Failed to build test router");

    let start_request = Request::builder()
        .method("POST")
        .uri(start_path)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-for", "127.0.0.1")
        .body(Body::from(start_body))
        .expect("Failed to build start request");
    let start_response = request_with_core_response(
        &app,
        &mut core_requests,
        &response_server,
        start_request,
        start_payload,
    )
    .await;
    let start_cookies = parse_set_cookies(&start_response);
    assert_eq!(start_cookies.len(), 1);

    let finish_request = Request::builder()
        .method("POST")
        .uri(finish_path)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-for", "127.0.0.1")
        .header(header::COOKIE, cookie_header(&start_cookies))
        .body(Body::from(finish_body))
        .expect("Failed to build finish request");
    let finish_response = request_with_core_response(
        &app,
        &mut core_requests,
        &response_server,
        finish_request,
        finish_payload,
    )
    .await;

    parse_set_cookies(&finish_response)
}

async fn request_cookies(
    public_url: Option<&str>,
    request_path: &'static str,
    request_body: &'static str,
    response_payload: core_response::Payload,
) -> Vec<Cookie<'static>> {
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
        .body(Body::from(request_body))
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

    parse_set_cookies(&response)
}

async fn request_cookie(
    public_url: Option<&str>,
    request_path: &'static str,
    response_payload: core_response::Payload,
) -> Cookie<'static> {
    request_cookies(
        public_url,
        request_path,
        r#"{"token":"test-token"}"#,
        response_payload,
    )
    .await
    .into_iter()
    .next()
    .expect("Response did not contain Set-Cookie")
}

fn assert_cookie(cookie: &Cookie<'_>, name: &str, path: &str, secure: bool) {
    assert_eq!(cookie.name(), name);
    assert_eq!(cookie.http_only(), Some(true));
    assert_eq!(cookie.same_site(), Some(SameSite::Strict));
    assert_eq!(cookie.secure(), secure.then_some(true));
    assert_eq!(cookie.path(), Some(path));
}

fn assert_removed_cookie(cookie: &Cookie<'_>, name: &str, path: &str, secure: bool) {
    assert_cookie(cookie, name, path, secure);
    assert_eq!(cookie.value(), "");
    assert_eq!(cookie.max_age(), Some(time::Duration::ZERO));
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

#[tokio::test]
async fn test_oidc_cookies_remain_secure_over_http() {
    let cookies = request_cookies(
        Some("http://proxy.example.com"),
        "/api/v1/openid/auth_info",
        r#"{"type":"enrollment"}"#,
        auth_info_response(),
    )
    .await;

    assert_eq!(cookies.len(), 2);
    let nonce_cookie = cookies
        .iter()
        .find(|cookie| cookie.name() == "nonce_proxy")
        .expect("Nonce cookie was not set");
    let csrf_cookie = cookies
        .iter()
        .find(|cookie| cookie.name() == "csrf_proxy")
        .expect("CSRF cookie was not set");
    assert_cookie(nonce_cookie, "nonce_proxy", "/api/v1/openid/callback", true);
    assert_cookie(csrf_cookie, "csrf_proxy", "/api/v1/openid/callback", true);
    assert_eq!(nonce_cookie.max_age(), Some(time::Duration::days(1)));
    assert_eq!(csrf_cookie.max_age(), Some(time::Duration::days(1)));
}

#[tokio::test]
async fn test_enrollment_cookie_removal_attributes_follow_core_public_url() {
    for (public_url, secure) in [
        ("https://proxy.example.com", true),
        ("http://proxy.example.com", false),
    ] {
        let cookies = request_cookie_removal(
            public_url,
            "/api/v1/enrollment/start",
            r#"{"token":"test-token"}"#,
            enrollment_response(),
            "/api/v1/enrollment/activate_user",
            r#"{"password":"test-password"}"#,
            core_response::Payload::Empty(()),
        )
        .await;

        assert_eq!(cookies.len(), 1);
        assert_removed_cookie(
            &cookies[0],
            ENROLLMENT_COOKIE_NAME,
            "/api/v1/enrollment",
            secure,
        );
    }
}

#[tokio::test]
async fn test_password_reset_cookie_removal_attributes_follow_core_public_url() {
    for (public_url, secure) in [
        ("https://proxy.example.com", true),
        ("http://proxy.example.com", false),
    ] {
        let cookies = request_cookie_removal(
            public_url,
            "/api/v1/password-reset/start",
            r#"{"token":"test-token"}"#,
            password_reset_response(),
            "/api/v1/password-reset/reset",
            r#"{"password":"new-password"}"#,
            core_response::Payload::Empty(()),
        )
        .await;

        assert_eq!(cookies.len(), 1);
        assert_removed_cookie(
            &cookies[0],
            PASSWORD_RESET_COOKIE_NAME,
            "/api/v1/password-reset",
            secure,
        );
    }
}
