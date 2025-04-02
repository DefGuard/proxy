use std::time::Duration;

use axum::{extract::FromRequestParts, http::request::Parts};
use axum_client_ip::{InsecureClientIp, LeftmostXForwardedFor};
use axum_extra::{headers::UserAgent, TypedHeader};
use defguard_protos::proto::proxy::{core_response::Payload, DeviceInfo};
use tokio::{sync::oneshot::Receiver, time::timeout};
use tonic::Code;

use crate::{error::ApiError, http::AppState};

pub(crate) mod desktop_client_mfa;
pub(crate) mod enrollment;
pub(crate) mod password_reset;
pub(crate) mod polling;

// Timeout for awaiting response from Defguard Core.
const CORE_RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
struct LocalDeviceInfo(DeviceInfo);

// #[tonic::async_trait]
// impl<S> FromRequestParts<S> for DeviceInfo
// where
//     S: Send + Sync,
// {
//     type Rejection = ();

//     async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
//         let forwarded_for_ip = LeftmostXForwardedFor::from_request_parts(parts, state)
//             .await
//             .map(|v| v.0);
//         let insecure_ip = InsecureClientIp::from_request_parts(parts, state)
//             .await
//             .map(|v| v.0);
//         let user_agent = TypedHeader::<UserAgent>::from_request_parts(parts, state)
//             .await
//             .map(|v| v.to_string())
//             .ok();

//         let ip_address = forwarded_for_ip.or(insecure_ip).map(|v| v.to_string()).ok();

//         Ok(DeviceInfo {
//             ip_address,
//             user_agent,
//         })
//     }
// }

#[tonic::async_trait]
impl<S> FromRequestParts<S> for LocalDeviceInfo
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

        Ok(Self(DeviceInfo {
            ip_address,
            user_agent,
        }))
    }
}

impl From<LocalDeviceInfo> for DeviceInfo {
    fn from(value: LocalDeviceInfo) -> Self {
        value.0
    }
}

impl From<DeviceInfo> for LocalDeviceInfo {
    fn from(value: DeviceInfo) -> Self {
        Self(value)
    }
}

/// Helper which awaits core response
///
/// Waits for core response with a given timeout and returns the response payload.
pub(crate) async fn get_core_response(rx: Receiver<Payload>) -> Result<Payload, ApiError> {
    debug!("Fetching core response...");
    if let Ok(core_response) = timeout(CORE_RESPONSE_TIMEOUT, rx).await {
        debug!("Got gRPC response from Defguard core: {core_response:?}");
        if let Ok(Payload::CoreError(core_error)) = core_response {
            if core_error.status_code == Code::FailedPrecondition as i32
                && core_error.message == "no valid license"
            {
                debug!("Tried to get core response related to an enterprise feature but the enterprise is not enabled, ignoring it...");
                return Err(ApiError::EnterpriseNotEnabled);
            }
            error!(
                "Received an error response from the core service. | status code: {} message: {}",
                core_error.status_code, core_error.message
            );
            return Err(core_error.into());
        };
        core_response
            .map_err(|err| ApiError::Unexpected(format!("Failed to receive core response: {err}")))
    } else {
        error!("Did not receive core response within {CORE_RESPONSE_TIMEOUT:?}");
        Err(ApiError::CoreTimeout)
    }
}
