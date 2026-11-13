use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use prost::Message;
use serde::Serialize;
use serde_json::{Value, json};
use tower::ServiceExt;

use super::support::{app_with_fake_core, post_json};
use crate::{
    generated::defguard::device_identity::v2::{
        AppleAssertion, AppleAttestationIdentity, AppleEvidence, AttestationChallenge,
        AttestationChallengeSolution, AttestationIdentity, AttestationStatus, CredentialChallenge,
        IdentityErrorCode, IdentityState, PossessionChallenge, PossessionChallengeSolution,
        Signature, SignatureAlgorithm, TpmAttestationIdentity, TpmAttestationResponse, TpmEvidence,
        attestation_challenge, attestation_challenge_solution, attestation_identity,
        possession_challenge_solution,
    },
    http::ENROLLMENT_COOKIE_NAME,
    proto::{
        AttestationFinishRequest, AttestationFinishResponse, AttestationStartRequest,
        AttestationStartResponse, CoreError, DeviceConfigResponse, EnrollmentStartResponse,
        InstanceInfoRequest, InstanceInfoResponse, core_request, core_response,
    },
};

fn assert_wire_equal<T: Message>(actual: &T, expected: &T) {
    assert_eq!(actual.encode_to_vec(), expected.encode_to_vec());
}

fn device_config(attestation_status: AttestationStatus) -> DeviceConfigResponse {
    DeviceConfigResponse {
        identity_state: Some(IdentityState {
            identity_id: "identity".into(),
            attestation_status: attestation_status as i32,
            key_fingerprint: vec![0, 255, 1],
            attested_at: Some(1_800_000_000),
            last_possession_verified_at: Some(1_800_000_030),
            ..Default::default()
        }),
        possession_challenge: Some(PossessionChallenge {
            challenge_id: "next-challenge".into(),
            data: vec![0, 255, 128, 1],
            expires_at: 1_800_000_300,
        }),
        ..Default::default()
    }
}

fn app_with_enrollment<F>(core: F) -> Router
where
    F: Fn(core_request::Payload) -> core_response::Payload + Send + 'static,
{
    app_with_fake_core(move |payload| match payload {
        core_request::Payload::EnrollmentStart(request) => {
            assert_eq!(request.token, "enrollment-token");
            core_response::Payload::EnrollmentStart(EnrollmentStartResponse {
                deadline_timestamp: 4_000_000_000,
                ..Default::default()
            })
        }
        payload => core(payload),
    })
}

async fn enrollment_cookie(app: &Router) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/enrollment/start")
                .header(header::CONTENT_TYPE, "application/json")
                .header("X-Forwarded-For", "10.0.0.1")
                .body(Body::from(r#"{"token":"enrollment-token"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    response.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .into()
}

async fn post_with_cookie<T: Serialize>(
    app: &Router,
    path: &str,
    cookie: &str,
    body: &T,
) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, cookie)
                .header("X-Forwarded-For", "10.0.0.1")
                .body(Body::from(serde_json::to_vec(body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn test_device_identity_attestation_start_forwarding() {
    for backend in [
        attestation_identity::Backend::Tpm(TpmAttestationIdentity {
            ek_public: vec![0, 255],
            ek_certificate_chain_der: vec![vec![128, 1], vec![2, 0]],
            ak_public: vec![3, 255],
        }),
        attestation_identity::Backend::AppleAppAttest(AppleAttestationIdentity {
            key_id: "apple-key-id".into(),
        }),
    ] {
        let challenge = match &backend {
            attestation_identity::Backend::Tpm(_) => {
                attestation_challenge::Challenge::Tpm(CredentialChallenge {
                    credential_blob: vec![0, 255],
                    encrypted_secret: vec![128, 1],
                })
            }
            attestation_identity::Backend::AppleAppAttest(_) => {
                attestation_challenge::Challenge::AppleAppAttest(())
            }
        };
        let request = AttestationStartRequest {
            token: "body-token-must-be-ignored".into(),
            device_id: 7,
            attestation_identity: Some(AttestationIdentity {
                backend: Some(backend),
            }),
        };
        let response = AttestationStartResponse {
            challenge: Some(AttestationChallenge {
                session_id: "attestation-session".into(),
                data: vec![0, 255, 128, 1],
                challenge: Some(challenge),
                expires_at: 1_800_000_300,
            }),
        };
        let mut expected_request = request.clone();
        expected_request.token = "enrollment-token".into();
        let expected_response = response.clone();
        let app = app_with_enrollment(move |payload| {
            let core_request::Payload::AttestationStart(received) = payload else {
                panic!("expected attestation start");
            };
            assert_wire_equal(&received, &expected_request);
            core_response::Payload::AttestationStart(expected_response.clone())
        });

        let cookie = enrollment_cookie(&app).await;
        let (status, body) = post_with_cookie(
            &app,
            "/api/v1/enrollment/attestation/start",
            &cookie,
            &request,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let received: AttestationStartResponse = serde_json::from_value(body).unwrap();
        assert_wire_equal(&received, &response);
    }
}

#[tokio::test]
async fn test_device_identity_attestation_finish_forwarding() {
    for result in [
        attestation_challenge_solution::Result::Tpm(TpmAttestationResponse {
            activation_secret: vec![0, 255, 128],
            evidence: Some(TpmEvidence {
                ek_public: vec![1, 255],
                ek_certificate_chain_der: vec![vec![2, 128]],
                ak_public: vec![3, 0],
                identity_public: vec![4, 255],
                attest: vec![5, 128],
                signature: Some(Signature {
                    algorithm: SignatureAlgorithm::EcdsaSha256 as i32,
                    signature: vec![6, 0],
                }),
            }),
        }),
        attestation_challenge_solution::Result::AppleAppAttest(AppleEvidence {
            attestation_object: vec![0, 255, 128],
        }),
        attestation_challenge_solution::Result::Error(IdentityErrorCode::Unavailable as i32),
    ] {
        let status = if matches!(&result, attestation_challenge_solution::Result::Error(_)) {
            AttestationStatus::AttestationFailed
        } else {
            AttestationStatus::Attested
        };
        let request = AttestationFinishRequest {
            token: "body-token-must-be-ignored".into(),
            device_id: 7,
            solution: Some(AttestationChallengeSolution {
                session_id: "attestation-session".into(),
                result: Some(result),
            }),
        };
        let mut expected_request = request.clone();
        expected_request.token = "enrollment-token".into();
        let response = AttestationFinishResponse {
            identity_state: device_config(status).identity_state,
        };
        let expected_response = response.clone();
        let app = app_with_enrollment(move |payload| {
            let core_request::Payload::AttestationFinish(received) = payload else {
                panic!("expected attestation finish");
            };
            assert_wire_equal(&received, &expected_request);
            core_response::Payload::AttestationFinish(expected_response.clone())
        });

        let cookie = enrollment_cookie(&app).await;
        let (status, body) = post_with_cookie(
            &app,
            "/api/v1/enrollment/attestation/finish",
            &cookie,
            &request,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let received: AttestationFinishResponse = serde_json::from_value(body).unwrap();
        assert_wire_equal(&received, &response);
    }
}

#[tokio::test]
async fn test_device_identity_poll_forwarding_and_challenge_recovery() {
    for proof in [
        Some(possession_challenge_solution::Result::Signature(
            Signature {
                algorithm: SignatureAlgorithm::EcdsaSha256 as i32,
                signature: vec![0, 255, 128],
            },
        )),
        Some(possession_challenge_solution::Result::AppleAppAttest(
            AppleAssertion {
                assertion_object: vec![0, 255, 128],
            },
        )),
        Some(possession_challenge_solution::Result::Error(
            IdentityErrorCode::IdentityLost as i32,
        )),
        None,
    ] {
        let request = InstanceInfoRequest {
            token: "polling-token".into(),
            possession_challenge_solution: proof.map(|result| PossessionChallengeSolution {
                identity_id: "identity".into(),
                challenge_id: "previous-challenge".into(),
                result: Some(result),
            }),
        };
        let expected_request = request.clone();
        let response = InstanceInfoResponse {
            device_config: Some(device_config(AttestationStatus::AttestationFailed)),
        };
        let expected_response = response.clone();
        let app = app_with_fake_core(move |payload| {
            let core_request::Payload::InstanceInfo(received) = payload else {
                panic!("expected polling request");
            };
            assert_wire_equal(&received, &expected_request);
            core_response::Payload::InstanceInfo(expected_response.clone())
        });

        let (status, body) = post_json(&app, "/api/v1/poll", &request).await;
        assert_eq!(status, StatusCode::OK);
        let received: InstanceInfoResponse = serde_json::from_value(body).unwrap();
        assert_wire_equal(&received, &response);
    }
}

#[tokio::test]
async fn test_attestation_requires_a_private_enrollment_cookie() {
    let app = app_with_fake_core(|_| panic!("unauthenticated request reached Core"));
    for path in [
        "/api/v1/enrollment/attestation/start",
        "/api/v1/enrollment/attestation/finish",
    ] {
        let body = json!({"device_id": 7, "token": "body-token"});
        let (status, _) = post_json(&app, path, &body).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        let (status, _) = post_with_cookie(
            &app,
            path,
            &format!("{ENROLLMENT_COOKIE_NAME}=unencrypted-token"),
            &body,
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
}

#[tokio::test]
async fn test_attestation_forwards_core_denials_and_rejects_unexpected_responses() {
    for (payload, expected_status) in [
        (
            core_response::Payload::CoreError(CoreError {
                status_code: tonic::Code::PermissionDenied.into(),
                message: "device belongs to another user".into(),
                ..Default::default()
            }),
            StatusCode::FORBIDDEN,
        ),
        (
            core_response::Payload::Empty(()),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    ] {
        let app = app_with_enrollment(move |_| payload.clone());
        let cookie = enrollment_cookie(&app).await;
        for path in [
            "/api/v1/enrollment/attestation/start",
            "/api/v1/enrollment/attestation/finish",
        ] {
            let (status, _) = post_with_cookie(&app, path, &cookie, &json!({"device_id":7})).await;
            assert_eq!(status, expected_status);
        }
    }
}
