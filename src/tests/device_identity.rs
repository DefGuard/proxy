use axum::http::StatusCode;
use prost::Message;

use super::support::{app_with_fake_core, post_json};
use crate::{
    generated::defguard::device_identity::v1::{
        AppleAssertion, AppleAttestationIdentity, AppleEvidence, AttestationIdentity,
        AttestationRequest, AttestationResponse, AttestationStatus, CredentialChallenge,
        IdentityErrorCode, IdentityState, PossessionChallenge, PossessionResponse, Signature,
        SignatureAlgorithm, TpmAttestationIdentity, TpmAttestationResponse, TpmEvidence,
        attestation_identity, attestation_request, attestation_response, possession_response,
    },
    proto::{
        DeviceConfigResponse, EnrollmentStartRequest, EnrollmentStartResponse, InstanceInfoRequest,
        InstanceInfoResponse, NewDevice, core_request, core_response, enrollment_start_request,
    },
};

fn assert_wire_equal<T: Message>(actual: &T, expected: &T) {
    assert_eq!(actual.encode_to_vec(), expected.encode_to_vec());
}

fn device_config() -> DeviceConfigResponse {
    DeviceConfigResponse {
        identity_state: Some(IdentityState {
            identity_id: "identity".into(),
            attestation_status: AttestationStatus::Attested as i32,
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

#[tokio::test]
async fn test_device_identity_enrollment_start_forwarding() {
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
                attestation_request::Challenge::Tpm(CredentialChallenge {
                    credential_blob: vec![0, 255],
                    encrypted_secret: vec![128, 1],
                })
            }
            attestation_identity::Backend::AppleAppAttest(_) => {
                attestation_request::Challenge::AppleAppAttest(())
            }
        };
        let request = EnrollmentStartRequest {
            token: "enrollment-token".into(),
            proposed_wireguard_pubkey: Some("wireguard-key".into()),
            device_identity: Some(
                enrollment_start_request::DeviceIdentity::AttestationIdentity(
                    AttestationIdentity {
                        backend: Some(backend),
                    },
                ),
            ),
        };
        let response = EnrollmentStartResponse {
            deadline_timestamp: 4_000_000_000,
            attestation_request: Some(AttestationRequest {
                session_id: "attestation-session".into(),
                data: vec![0, 255, 128, 1],
                challenge: Some(challenge),
                expires_at: 1_800_000_300,
            }),
            ..Default::default()
        };
        let expected_request = request.clone();
        let expected_response = response.clone();
        let app = app_with_fake_core(move |payload| {
            let core_request::Payload::EnrollmentStart(received) = payload else {
                panic!("expected enrollment start");
            };
            assert_wire_equal(&received, &expected_request);
            core_response::Payload::EnrollmentStart(expected_response.clone())
        });

        let (status, body) = post_json(&app, "/api/v1/enrollment/start", &request).await;
        assert_eq!(status, StatusCode::OK);
        let received: EnrollmentStartResponse = serde_json::from_value(body).unwrap();
        assert_wire_equal(&received, &response);
    }
}

#[tokio::test]
async fn test_device_identity_create_device_forwarding() {
    for result in [
        attestation_response::Result::Tpm(TpmAttestationResponse {
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
        attestation_response::Result::AppleAppAttest(AppleEvidence {
            attestation_object: vec![0, 255, 128],
        }),
        attestation_response::Result::Error(IdentityErrorCode::Unavailable as i32),
    ] {
        let request = NewDevice {
            name: "device".into(),
            pubkey: "wireguard-key".into(),
            token: None,
            attestation_response: Some(AttestationResponse {
                session_id: "attestation-session".into(),
                result: Some(result),
            }),
        };
        let expected_request = request.clone();
        let response = device_config();
        let expected_response = response.clone();
        let app = app_with_fake_core(move |payload| {
            let core_request::Payload::NewDevice(received) = payload else {
                panic!("expected new device");
            };
            assert_wire_equal(&received, &expected_request);
            core_response::Payload::DeviceConfig(expected_response.clone())
        });

        let (status, body) = post_json(&app, "/api/v1/enrollment/create_device", &request).await;
        assert_eq!(status, StatusCode::OK);
        let received: DeviceConfigResponse = serde_json::from_value(body).unwrap();
        assert_wire_equal(&received, &response);
    }
}

#[tokio::test]
async fn test_device_identity_poll_forwarding_and_challenge_recovery() {
    for proof in [
        Some(possession_response::Result::Signature(Signature {
            algorithm: SignatureAlgorithm::EcdsaSha256 as i32,
            signature: vec![0, 255, 128],
        })),
        Some(possession_response::Result::AppleAppAttest(
            AppleAssertion {
                assertion_object: vec![0, 255, 128],
            },
        )),
        Some(possession_response::Result::Error(
            IdentityErrorCode::IdentityLost as i32,
        )),
        None,
    ] {
        let request = InstanceInfoRequest {
            token: "polling-token".into(),
            possession_response: proof.map(|result| PossessionResponse {
                identity_id: "identity".into(),
                challenge_id: "previous-challenge".into(),
                result: Some(result),
            }),
        };
        let expected_request = request.clone();
        let response = InstanceInfoResponse {
            device_config: Some(device_config()),
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
async fn test_device_identity_enrollment_failure_forwarding() {
    let request = EnrollmentStartRequest {
        token: "enrollment-token".into(),
        device_identity: Some(enrollment_start_request::DeviceIdentity::IdentityError(
            IdentityErrorCode::Unavailable as i32,
        )),
        ..Default::default()
    };
    let expected_request = request.clone();
    let app = app_with_fake_core(move |payload| {
        let core_request::Payload::EnrollmentStart(received) = payload else {
            panic!("expected enrollment start");
        };
        assert_wire_equal(&received, &expected_request);
        core_response::Payload::EnrollmentStart(EnrollmentStartResponse {
            deadline_timestamp: 4_000_000_000,
            ..Default::default()
        })
    });
    let (status, body) = post_json(&app, "/api/v1/enrollment/start", &request).await;
    assert_eq!(status, StatusCode::OK);
    let response: EnrollmentStartResponse = serde_json::from_value(body).unwrap();
    assert!(response.attestation_request.is_none());
}
