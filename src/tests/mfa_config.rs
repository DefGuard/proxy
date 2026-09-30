//! Drives the MFA config flow through the HTTP API against a scripted fake Core.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use axum::http::StatusCode;
use serde_json::json;

use super::support::{app_with_fake_core, post_json};
use crate::proto::{
    CodeMfaSetupFinishResponse, CodeMfaSetupStartResponse, CoreError, MfaConfigAuthorizeResponse,
    MfaConfigFido2ChallengeResponse, MfaConfigSendCodeResponse, MfaConfigStartResponse, MfaMethod,
    core_request, core_response,
};

const SESSION_TOKEN: &str = "mfa-config-session";
const TOTP: i32 = MfaMethod::Totp as i32;
const EMAIL: i32 = MfaMethod::Email as i32;
const FIDO2: i32 = MfaMethod::Fido2 as i32;
const OIDC: i32 = MfaMethod::Oidc as i32;

fn authorize_response() -> core_response::Payload {
    core_response::Payload::MfaConfigAuthorize(MfaConfigAuthorizeResponse {
        deadline_timestamp: 1_800_003_600,
        recovery_codes: vec![],
    })
}

/// Fake Core for a user with no factor: the email fallback authorizes, then TOTP is set up.
fn fallback_then_totp(
    steps: Arc<AtomicUsize>,
) -> impl Fn(core_request::Payload) -> core_response::Payload {
    move |request| {
        steps.fetch_add(1, Ordering::Relaxed);
        match request {
            core_request::Payload::MfaConfigStart(req) => {
                assert_eq!(req.token, "polling-token");
                assert_eq!(req.pubkey, "device-pubkey");
                core_response::Payload::MfaConfigStart(MfaConfigStartResponse {
                    session_token: SESSION_TOKEN.into(),
                    available_methods: vec![],
                    email_fallback: true,
                    deadline_timestamp: 1_800_000_000,
                })
            }
            core_request::Payload::MfaConfigSendCode(req) => {
                assert_eq!(req.session_token, SESSION_TOKEN);
                core_response::Payload::MfaConfigSendCode(MfaConfigSendCodeResponse {})
            }
            core_request::Payload::MfaConfigAuthorize(req) => {
                assert_eq!(req.session_token, SESSION_TOKEN);
                assert_eq!(req.method, EMAIL);
                assert_eq!(req.code, "123456");
                authorize_response()
            }
            core_request::Payload::CodeMfaSetupStart(req) => {
                assert_eq!(req.token, SESSION_TOKEN);
                assert_eq!(req.method, TOTP);
                core_response::Payload::CodeMfaSetupStartResponse(CodeMfaSetupStartResponse {
                    totp_secret: Some("JBSWY3DPEHPK3PXP".into()),
                    fido2_creation_challenge: None,
                })
            }
            core_request::Payload::CodeMfaSetupFinish(req) => {
                assert_eq!(req.token, SESSION_TOKEN);
                assert_eq!(req.method, TOTP);
                assert_eq!(req.code, "654321");
                core_response::Payload::CodeMfaSetupFinishResponse(CodeMfaSetupFinishResponse {
                    recovery_codes: vec!["aaaa-bbbb".into(), "cccc-dddd".into()],
                })
            }
            core_request::Payload::MfaConfigEnd(req) => {
                assert_eq!(req.session_token, SESSION_TOKEN);
                core_response::Payload::Empty(())
            }
            _ => panic!("unexpected request to Core"),
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_mfa_config_flow_forwards_session_token() {
    let steps = Arc::new(AtomicUsize::new(0));
    let app = app_with_fake_core(fallback_then_totp(Arc::clone(&steps)));

    let (status, body) = post_json(
        &app,
        "/api/v1/mfa-config/start",
        &json!({ "token": "polling-token", "pubkey": "device-pubkey" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["session_token"], SESSION_TOKEN);
    assert_eq!(body["email_fallback"], true);
    assert_eq!(body["available_methods"], json!([]));
    let session_token = body["session_token"].as_str().unwrap().to_owned();

    let (status, body) = post_json(
        &app,
        "/api/v1/mfa-config/send-code",
        &json!({ "session_token": session_token }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, body) = post_json(
        &app,
        "/api/v1/mfa-config/authorize",
        &json!({ "session_token": session_token, "method": EMAIL, "code": "123456" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["deadline_timestamp"], 1_800_003_600);

    let (status, body) = post_json(
        &app,
        "/api/v1/mfa-config/setup/start",
        &json!({ "token": session_token, "method": TOTP }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["totp_secret"], "JBSWY3DPEHPK3PXP");

    let (status, body) = post_json(
        &app,
        "/api/v1/mfa-config/setup/finish",
        &json!({ "token": session_token, "method": TOTP, "code": "654321" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["recovery_codes"], json!(["aaaa-bbbb", "cccc-dddd"]));

    let (status, body) = post_json(
        &app,
        "/api/v1/mfa-config/end",
        &json!({ "session_token": session_token }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    assert_eq!(
        steps.load(Ordering::Relaxed),
        6,
        "Core must see every step once"
    );
}

#[tokio::test]
async fn test_mfa_setup_rejects_unsupported_method_before_core() {
    let app = app_with_fake_core(|_| panic!("Core must not be called"));

    for path in [
        "/api/v1/mfa-config/setup/start",
        "/api/v1/mfa-config/setup/finish",
    ] {
        let (status, _) = post_json(
            &app,
            path,
            &json!({ "token": SESSION_TOKEN, "method": OIDC, "code": "1" }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{path}");
    }
}

#[tokio::test]
async fn test_mfa_config_core_error_maps_to_http_status() {
    let app = app_with_fake_core(|_| {
        core_response::Payload::CoreError(CoreError {
            status_code: tonic::Code::Unauthenticated as i32,
            message: "invalid token".into(),
        })
    });

    let (status, body) = post_json(
        &app,
        "/api/v1/mfa-config/authorize",
        &json!({ "session_token": "stale", "method": EMAIL, "code": "000000" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");

    let app = app_with_fake_core(|_| {
        core_response::Payload::CoreError(CoreError {
            status_code: tonic::Code::FailedPrecondition as i32,
            message: "OIDC authentication not completed".into(),
        })
    });

    let (status, body) = post_json(
        &app,
        "/api/v1/mfa-config/authorize",
        &json!({ "session_token": SESSION_TOKEN, "method": OIDC, "code": "" }),
    )
    .await;
    assert_eq!(status, StatusCode::PRECONDITION_REQUIRED, "{body}");
    assert_eq!(body["error"], "OIDC authentication not completed");
}

#[tokio::test]
async fn test_mfa_config_fido2_challenge_forwards_session_token() {
    let app = app_with_fake_core(|request| match request {
        core_request::Payload::MfaConfigFido2Challenge(req) => {
            assert_eq!(req.session_token, SESSION_TOKEN);
            core_response::Payload::MfaConfigFido2Challenge(MfaConfigFido2ChallengeResponse {
                challenge: "challenge-json".into(),
                credential_ids: vec!["cred-a".into(), "cred-b".into()],
            })
        }
        _ => panic!("unexpected request to Core"),
    });

    let (status, body) = post_json(
        &app,
        "/api/v1/mfa-config/fido2-challenge",
        &json!({ "session_token": SESSION_TOKEN }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["challenge"], "challenge-json");
    assert_eq!(body["credential_ids"], json!(["cred-a", "cred-b"]));
}

#[tokio::test]
async fn test_mfa_config_authorize_forwards_fido2_assertion() {
    let app = app_with_fake_core(|request| match request {
        core_request::Payload::MfaConfigAuthorize(req) => {
            assert_eq!(req.session_token, SESSION_TOKEN);
            assert_eq!(req.method, FIDO2);
            assert_eq!(req.code, "");
            assert_eq!(req.signature.as_deref(), Some("c2lnbmF0dXJl"));
            assert_eq!(req.auth_data.as_deref(), Some(&[1, 2, 3][..]));
            assert_eq!(req.credential_id.as_deref(), Some(&[4, 5][..]));
            authorize_response()
        }
        _ => panic!("unexpected request to Core"),
    });

    let (status, body) = post_json(
        &app,
        "/api/v1/mfa-config/authorize",
        &json!({
            "session_token": SESSION_TOKEN,
            "method": FIDO2,
            "code": "",
            "signature": "c2lnbmF0dXJl",
            "auth_data": [1, 2, 3],
            "credential_id": [4, 5],
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["deadline_timestamp"], 1_800_003_600);
}

#[tokio::test]
async fn test_mfa_config_authorize_without_fido2_fields() {
    let app = app_with_fake_core(|request| match request {
        core_request::Payload::MfaConfigAuthorize(req) => {
            assert_eq!(req.method, EMAIL);
            assert_eq!(req.signature, None);
            assert_eq!(req.auth_data, None);
            assert_eq!(req.credential_id, None);
            authorize_response()
        }
        _ => panic!("unexpected request to Core"),
    });

    let (status, body) = post_json(
        &app,
        "/api/v1/mfa-config/authorize",
        &json!({ "session_token": SESSION_TOKEN, "method": EMAIL, "code": "123456" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}
