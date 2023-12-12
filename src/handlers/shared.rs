use std::net::IpAddr;

use axum::{headers::UserAgent, TypedHeader};
use axum_client_ip::LeftmostXForwardedFor;
use tonic::metadata::MetadataValue;
use tower_cookies::Cookies;
use tracing::{debug, error};

use crate::{error::ApiError, server::SECRET_KEY};

// extract token from session cookies and add it to gRPC request auth header
pub fn add_auth_header<T>(
    cookies: Cookies,
    request: &mut tonic::Request<T>,
    cookie_name: &str,
) -> Result<(), ApiError> {
    debug!("Adding auth header to gRPC request");
    let key = SECRET_KEY.get().unwrap();
    let private_cookies = cookies.private(key);

    match private_cookies.get(cookie_name) {
        Some(cookie) => {
            let token = MetadataValue::try_from(cookie.value())?;
            request.metadata_mut().insert("authorization", token);
        }
        None => {
            error!("Enrollment session cookie not found");
            return Err(ApiError::CookieNotFound);
        }
    }

    Ok(())
}

pub fn add_device_info_header<T>(
    request: &mut tonic::Request<T>,
    forwarded_for_ip: Option<LeftmostXForwardedFor>,
    insecure_ip: IpAddr,
    user_agent: Option<TypedHeader<UserAgent>>,
) -> Result<(), ApiError> {
    let ip_address: String = forwarded_for_ip.map_or(insecure_ip, |v| v.0).to_string();
    let user_agent_string: String = user_agent.map(|v| v.to_string()).unwrap_or_default();

    request
        .metadata_mut()
        .insert("ip_address", MetadataValue::try_from(ip_address)?);
    request
        .metadata_mut()
        .insert("user_agent", MetadataValue::try_from(user_agent_string)?);

    Ok(())
}

pub fn create_request<T>(req: T) -> tonic::Request<T> {
    tonic::Request::new(req)
}
