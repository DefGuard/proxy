pub(crate) mod enrollment;
pub(crate) mod password_reset;

use axum::{extract::FromRequestParts, http::request::Parts};
use axum_client_ip::{InsecureClientIp, LeftmostXForwardedFor};
use axum_extra::{headers::UserAgent, TypedHeader};

use super::proto::DeviceInfo;

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
