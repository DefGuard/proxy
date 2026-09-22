use std::sync::Arc;

use axum::http::StatusCode;
use serde_json::json;

use crate::{
    handlers::mfa_flow::send_mfa_flow_remote,
    http::AppState,
    proto::{
        ClientMfaFinishRequest, ClientMfaFinishResponse, ClientMfaStartRequest,
        ClientMfaStartResponse, DeviceInfo, MfaAdvanced, MfaAwaitingExternal, MfaCodeCredential,
        MfaCompleted, MfaFlowApproveRequest, MfaFlowRemoteRequest, MfaFlowRemoteResponse,
        MfaFlowStartAccepted, MfaFlowStartRequest, MfaFlowStartResponse, MfaFlowStepFinishRequest,
        MfaFlowStepFinishResponse, MfaFlowStepStartRequest, MfaFlowStepStartResponse, MfaMethod,
        MfaMobileApprovalProof, MfaSignatureChallenge, MfaStepResult, MfaStepStarted, core_request,
        core_response, mfa_flow_start_response, mfa_flow_step_finish_request, mfa_step_result,
        mfa_step_started,
    },
    tests::support::{app_with_fake_core, cookie_key, post_json, test_proxy_server},
};

fn first_step() -> MfaStepStarted {
    MfaStepStarted {
        step_attempt_id: "attempt-id".to_string(),
        challenge: Some(mfa_step_started::Challenge::Signature(
            MfaSignatureChallenge {
                challenge: "challenge".to_string(),
            },
        )),
    }
}

fn start_response() -> MfaFlowStartResponse {
    MfaFlowStartResponse {
        outcome: Some(mfa_flow_start_response::Outcome::Accepted(
            MfaFlowStartAccepted {
                token: "flow-token".to_string(),
                first_step: Some(first_step()),
            },
        )),
    }
}

#[tokio::test]
async fn test_start_dispatches_to_mfa_flow_start() {
    let response = start_response();
    let expected = serde_json::to_value(&response).unwrap();
    let app = app_with_fake_core(move |payload| match payload {
        core_request::Payload::MfaFlowStart(request) => {
            assert!(request.location_id == 7);
            assert!(request.pubkey == "device-public-key");
            assert!(request.selected_methods == vec![MfaMethod::Totp as i32]);
            core_response::Payload::MfaFlowStart(response.clone())
        }
        _ => panic!("unexpected Core request"),
    });

    let request = MfaFlowStartRequest {
        location_id: 7,
        pubkey: "device-public-key".to_string(),
        posture_data: None,
        selected_methods: vec![MfaMethod::Totp as i32],
    };
    let (status, body) = post_json(&app, "/api/v1/mfa-flow/start", &request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body == expected);
}

#[tokio::test]
async fn test_step_start_dispatches_to_mfa_flow_step_start() {
    let response = MfaFlowStepStartResponse {
        started: Some(first_step()),
    };
    let expected = serde_json::to_value(&response).unwrap();
    let app = app_with_fake_core(move |payload| match payload {
        core_request::Payload::MfaFlowStepStart(request) => {
            assert!(request.token == "flow-token");
            assert!(request.method == MfaMethod::Email as i32);
            core_response::Payload::MfaFlowStepStart(response.clone())
        }
        _ => panic!("unexpected Core request"),
    });

    let request = MfaFlowStepStartRequest {
        token: "flow-token".to_string(),
        method: MfaMethod::Email as i32,
    };
    let (status, body) = post_json(&app, "/api/v1/mfa-flow/step-start", &request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body == expected);
}

#[tokio::test]
async fn test_step_finish_dispatches_and_preserves_all_result_arms() {
    let outcomes = [
        mfa_step_result::Outcome::Advanced(MfaAdvanced { next_step: 1 }),
        mfa_step_result::Outcome::Completed(MfaCompleted {
            preshared_key: "flow-psk".to_string(),
        }),
        mfa_step_result::Outcome::AwaitingExternal(MfaAwaitingExternal {}),
    ];

    for outcome in outcomes {
        let response = MfaFlowStepFinishResponse {
            result: Some(MfaStepResult {
                outcome: Some(outcome),
            }),
        };
        let expected = serde_json::to_value(&response).unwrap();
        let app = app_with_fake_core(move |payload| match payload {
            core_request::Payload::MfaFlowStepFinish(request) => {
                assert!(request.token == "flow-token");
                assert!(request.step_attempt_id == "attempt-id");
                assert!(matches!(
                    request.submission,
                    Some(mfa_flow_step_finish_request::Submission::Code(_))
                ));
                core_response::Payload::MfaFlowStepFinish(response.clone())
            }
            _ => panic!("unexpected Core request"),
        });

        let request = MfaFlowStepFinishRequest {
            token: "flow-token".to_string(),
            step_attempt_id: "attempt-id".to_string(),
            submission: Some(mfa_flow_step_finish_request::Submission::Code(
                MfaCodeCredential {
                    code: "123456".to_string(),
                },
            )),
        };
        let (status, body) = post_json(&app, "/api/v1/mfa-flow/step-finish", &request).await;

        assert_eq!(status, StatusCode::OK);
        assert!(body == expected);
    }
}

#[tokio::test]
async fn test_approve_dispatches_and_returns_empty_json() {
    let app = app_with_fake_core(|payload| match payload {
        core_request::Payload::MfaFlowApprove(request) => {
            assert!(request.token == "flow-token");
            assert!(request.step_attempt_id == "attempt-id");
            assert!(matches!(request.proof, Some(MfaMobileApprovalProof { .. })));
            core_response::Payload::Empty(())
        }
        _ => panic!("unexpected Core request"),
    });

    let request = MfaFlowApproveRequest {
        token: "flow-token".to_string(),
        step_attempt_id: "attempt-id".to_string(),
        proof: Some(MfaMobileApprovalProof {
            signature: "signature".to_string(),
            auth_pub_key: "auth-public-key".to_string(),
        }),
    };
    let (status, body) = post_json(&app, "/api/v1/mfa-flow/approve", &request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body == json!({}));
}

#[tokio::test]
async fn test_remote_dispatches_to_mfa_flow_remote() {
    let cookie_key = cookie_key();
    let server = test_proxy_server(Arc::clone(&cookie_key));
    let mut requests = server.register_test_client();
    let state = AppState {
        grpc_server: server.clone(),
        cookie_key,
    };
    let receiver = send_mfa_flow_remote(
        &state,
        MfaFlowRemoteRequest {
            token: "flow-token".to_string(),
            step_attempt_id: "attempt-id".to_string(),
        },
        DeviceInfo {
            ip_address: "10.0.0.1".to_string(),
            user_agent: None,
            version: None,
            platform: None,
        },
    )
    .unwrap();

    let request = requests.recv().await.unwrap().unwrap();
    let request_id = request.id;
    match request.payload {
        Some(core_request::Payload::MfaFlowRemote(request)) => {
            assert!(request.token == "flow-token");
            assert!(request.step_attempt_id == "attempt-id");
        }
        _ => panic!("unexpected Core request"),
    }
    server.resolve_test_response(
        request_id,
        core_response::Payload::MfaFlowRemote(MfaFlowRemoteResponse {
            result: Some(MfaStepResult {
                outcome: Some(mfa_step_result::Outcome::Advanced(MfaAdvanced {
                    next_step: 1,
                })),
            }),
        }),
    );

    assert!(matches!(
        receiver.await.unwrap(),
        core_response::Payload::MfaFlowRemote(_)
    ));
}

#[tokio::test]
async fn test_legacy_http_routes_keep_their_response_shapes() {
    let app = app_with_fake_core(|payload| match payload {
        core_request::Payload::ClientMfaStart(_) => {
            core_response::Payload::ClientMfaStart(ClientMfaStartResponse {
                token: "legacy-token".to_string(),
                challenge: None,
            })
        }
        core_request::Payload::ClientMfaFinish(_) => {
            core_response::Payload::ClientMfaFinish(ClientMfaFinishResponse {
                preshared_key: "legacy-psk".to_string(),
                token: None,
            })
        }
        _ => panic!("unexpected Core request"),
    });

    let (status, body) = post_json(
        &app,
        "/api/v1/client-mfa/start",
        &ClientMfaStartRequest {
            location_id: 7,
            pubkey: "device-public-key".to_string(),
            method: MfaMethod::Totp as i32,
            posture_data: None,
        },
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body == json!({"token": "legacy-token", "challenge": null}));

    let finish_request = ClientMfaFinishRequest {
        token: "legacy-token".to_string(),
        code: Some("123456".to_string()),
        auth_pub_key: None,
    };
    let (status, body) = post_json(&app, "/api/v1/client-mfa/finish", &finish_request).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body == json!({"preshared_key": "legacy-psk", "token": null}));

    let (status, body) = post_json(&app, "/api/v1/client-mfa/finish-remote", &finish_request).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body == json!({}));
}

#[tokio::test]
async fn test_legacy_step_start_is_an_unknown_route() {
    let app = app_with_fake_core(|_| panic!("unknown route reached Core"));
    let (step_status, step_body) =
        post_json(&app, "/api/v1/client-mfa/step-start", &json!({})).await;
    let (unknown_status, unknown_body) =
        post_json(&app, "/api/v1/client-mfa/unknown", &json!({})).await;

    assert_eq!(step_status, unknown_status);
    assert!(step_body == unknown_body);
}

#[tokio::test]
async fn test_malformed_step_finish_is_rejected_before_core_dispatch() {
    let app = app_with_fake_core(|_| panic!("malformed request reached Core"));
    let (status, _) = post_json(
        &app,
        "/api/v1/mfa-flow/step-finish",
        &json!({"token": "flow-token"}),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}
