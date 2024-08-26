use crate::{error::ApiError, proto::core_response::Payload};
use axum::{extract::FromRequestParts, http::request::Parts};
use axum_client_ip::{InsecureClientIp, LeftmostXForwardedFor};
use axum_extra::{headers::UserAgent, TypedHeader};
use std::time::Duration;
use tokio::{sync::oneshot::Receiver, time::timeout};

use super::proto::DeviceInfo;

pub(crate) mod desktop_client_mfa;
pub(crate) mod enrollment;
pub(crate) mod info;
pub(crate) mod password_reset;

// timeout in seconds for awaiting core response
const CORE_RESPONSE_TIMEOUT: u64 = 5;

#[tonic::async_trait]
impl<S> FromRequestParts<S> for DeviceInfo
where
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let forwarded_for_ip = LeftmostXForwardedFor::from_request_parts(parts, state)
            .await
            .map(|v| v.0);
        let insecure_ip = InsecureClientIp::from_request_parts(parts, state)
            .await
            .map(|v| v.0);
        let user_agent = TypedHeader::<UserAgent>::from_request_parts(parts, state)
            .await
            .map(|v| v.to_string())
            .ok();

        let ip_address = forwarded_for_ip.or(insecure_ip).map(|v| v.to_string()).ok();

        Ok(DeviceInfo {
            ip_address,
            user_agent,
        })
    }
}

/// Helper which awaits core response
///
/// Waits for core response with a given timeout and returns the response payload.
async fn get_core_response(rx: Receiver<Payload>) -> Result<Payload, ApiError> {
    debug!("Fetching core response...");
    if let Ok(core_response) = timeout(Duration::from_secs(CORE_RESPONSE_TIMEOUT), rx).await {
        debug!("Got gRPC response from Defguard core: {core_response:?}");
        if let Ok(Payload::CoreError(core_error)) = core_response {
            error!(
                "Received an error response from the core service. | status code: {} message: {}",
                core_error.status_code, core_error.message
            );
            return Err(core_error.into());
        };
        core_response
            .map_err(|err| ApiError::Unexpected(format!("Failed to receive core response: {err}")))
    } else {
        error!("Did not receive core response within {CORE_RESPONSE_TIMEOUT} seconds");
        Err(ApiError::CoreTimeout)
    }
}
