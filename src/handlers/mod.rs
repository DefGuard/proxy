use std::time::Duration;

use axum::{extract::FromRequestParts, http::request::Parts};
use axum_client_ip::{InsecureClientIp, LeftmostXForwardedFor};
use axum_extra::{headers::UserAgent, TypedHeader};
use tokio::{sync::oneshot::Receiver, time::timeout};
use tonic::Code;

use super::proto::DeviceInfo;
use crate::{error::ApiError, proto::core_response::Payload};

pub(crate) mod desktop_client_mfa;
pub(crate) mod enrollment;
pub(crate) mod mobile_client;
pub(crate) mod password_reset;
pub(crate) mod polling;
pub(crate) mod register_mfa;

// Timeout for awaiting response from Defguard Core.
const CORE_RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

impl<S> FromRequestParts<S> for DeviceInfo
where
    S: Send + Sync,
{
    type Rejection = ApiError;

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
            .ok()
            // sanitize user-agent
            .filter(|agent| !ammonia::is_html(agent));

        let ip_address = forwarded_for_ip
            .or(insecure_ip)
            .map(|v| v.to_string())
            .map_err(|_| ApiError::Unexpected("Missing client IP".to_string()))?;

        Ok(DeviceInfo {
            ip_address,
            user_agent,
        })
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
        }
        core_response
            .map_err(|err| ApiError::Unexpected(format!("Failed to receive core response: {err}")))
    } else {
        error!("Did not receive core response within {CORE_RESPONSE_TIMEOUT:?}");
        Err(ApiError::CoreTimeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};

    static VALID_USER_AGENTS: &[&str] = &[
        // desktop
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.10 Safari/605.1.1	43.03",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.3	21.05",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.3	17.34",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.3	3.72",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36 Trailer/93.3.8652.5	2.48",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36 Edg/134.0.0.	2.48",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.	2.48",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36 OPR/117.0.0.	2.48",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36 Edg/132.0.0.	1.24",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/70.0.3538.102 Safari/537.36 Edge/18.1958	1.24",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:136.0) Gecko/20100101 Firefox/136.	1.24",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0 Safari/537.3	1.24",

        // mobile
        "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Mobile Safari/537.3	63.11",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 18_3_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.3.1 Mobile/15E148 Safari/604.	8.25",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 18_3_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/360.1.737798518 Mobile/15E148 Safari/604.	5.83",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 18_3_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/134.0.6998.99 Mobile/15E148 Safari/604.	4.85",
        "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) SamsungBrowser/27.0 Chrome/125.0.0.0 Mobile Safari/537.3	3.88",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 17_5_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Mobile/15E148 Safari/604.	3.4",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 18_3_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.3 Mobile/15E148 Safari/604.	1.94",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1.1 Mobile/15E148 Safari/604.	1.94",
        "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Mobile Safari/537.3	1.46",
        "Mozilla/5.0 (Android 14; Mobile; rv:136.0) Gecko/136.0 Firefox/136.	0.97",
        "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Mobile Safari/537.3	0.97",
        "Mozilla/5.0 (Linux; Android 10; JNY-LX1; HMSCore 6.15.0.302) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.5735.196 HuaweiBrowser/15.0.4.312 Mobile Safari/537.3	0.97",
        "Mozilla/5.0 (Android 15; Mobile; rv:136.0) Gecko/136.0 Firefox/136.	0.49",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 10_3 like Mac OS X) AppleWebKit/602.1.50 (KHTML, like Gecko) CriOS/56.0.2924.75 Mobile/14E5239e YisouSpider/5.0 Safari/602.	0.49",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 16_7_10 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.	0.49",
        "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Mobile Safari/537.3	0.49",
        "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Mobile Safari/537.3	0.49",
    ];

    static INVALID_USER_AGENTS: &[&str] = &[
        "<h1><a href=\"//isec.pl\">CLICK HERE</a></h1>",
        "<html><script>alert(\"test\")</script></html>",
        "<h1><a href=\"//isec.pl\">CLICK HERE",
    ];

    struct DummyState;

    #[tokio::test]
    async fn test_user_agent_sanitization_dg25_16() {
        let state = DummyState;

        // valid user agents
        for agent in VALID_USER_AGENTS {
            let req = Request::builder()
                .header("User-Agent", *agent)
                .header("X-Forwarded-For", "10.0.0.1")
                .body(Body::empty())
                .unwrap();
            let (parts, _) = req.into_parts();
            let mut parts = parts;

            let device_info = DeviceInfo::from_request_parts(&mut parts, &state)
                .await
                .expect("should succeed");

            assert_eq!(device_info.user_agent, Some(agent.to_string()));
        }

        // invalid user agents
        for agent in INVALID_USER_AGENTS {
            let req = Request::builder()
                .header("User-Agent", *agent)
                .header("X-Forwarded-For", "10.0.0.1")
                .body(Body::empty())
                .unwrap();
            let (parts, _) = req.into_parts();
            let mut parts = parts;

            let device_info = DeviceInfo::from_request_parts(&mut parts, &state)
                .await
                .expect("should succeed");

            assert!(device_info.user_agent.is_none());
        }

        // no user agent
        let req = Request::builder()
            .header("X-Forwarded-For", "10.0.0.1")
            .body(Body::empty())
            .unwrap();
        let (parts, _) = req.into_parts();
        let mut parts = parts;

        let device_info = DeviceInfo::from_request_parts(&mut parts, &state)
            .await
            .expect("should succeed");

        assert!(device_info.user_agent.is_none());
    }
}
