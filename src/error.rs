use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use tonic::{Code, Status, metadata::errors::InvalidMetadataValue};

use crate::proto::CoreError;

/// Prefix of Core's answer to an OIDC MFA poll made before the browser round trip completes.
const OIDC_NOT_COMPLETED: &str = "OIDC authentication not completed";

/// Whether Core reports that OIDC authentication is still pending. Clients poll until it
/// completes, so this is expected and not an error.
pub(crate) fn is_oidc_not_completed(core_error: &CoreError) -> bool {
    core_error.status_code == Code::FailedPrecondition as i32
        && core_error.message.starts_with(OIDC_NOT_COMPLETED)
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Unexpected error: {0}")]
    Unexpected(String),
    #[error(transparent)]
    InvalidMetadata(#[from] InvalidMetadataValue),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Core gRPC response timeout")]
    CoreTimeout,
    #[error("Invalid core gRPC response type received")]
    InvalidResponseType,
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Enterprise not enabled")]
    EnterpriseNotEnabled,
    #[error("Precondition required: {0}")]
    PreconditionRequired(String),
    #[error("OIDC authentication pending: {0}")]
    OidcNotCompleted(String),
    #[error("Bad request: {0}")]
    NotFound(String),
    #[error("PostureRejected: {0:?}")]
    PostureRejected(Vec<String>),
    #[error("Failed to get client IP address")]
    ClientIpError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if let Self::OidcNotCompleted(_) = self {
            debug!("{self}");
        } else {
            error!("{self}");
        }
        let (status, error_message) = match self {
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::PermissionDenied(msg) => (StatusCode::FORBIDDEN, msg),
            Self::EnterpriseNotEnabled => (
                StatusCode::PAYMENT_REQUIRED,
                "Enterprise features are not enabled".to_string(),
            ),
            Self::PreconditionRequired(msg) | Self::OidcNotCompleted(msg) => {
                (StatusCode::PRECONDITION_REQUIRED, msg)
            }
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::PostureRejected(reasons) => (StatusCode::FORBIDDEN, reasons.join(", ")),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        let body = Json(json!({"error": error_message}));

        (status, body).into_response()
    }
}

impl From<base64::DecodeError> for ApiError {
    fn from(value: base64::DecodeError) -> Self {
        Self::BadRequest(format!(
            "Failed to decode base64 from request data. {value}"
        ))
    }
}

impl From<CoreError> for ApiError {
    fn from(core_error: CoreError) -> Self {
        if is_oidc_not_completed(&core_error) {
            return ApiError::OidcNotCompleted(core_error.message);
        }
        // convert to tonic::Status first
        let status = Status::new(Code::from(core_error.status_code), core_error.message);
        match status.code() {
            Code::Unauthenticated => ApiError::Unauthorized(status.message().to_string()),
            Code::InvalidArgument => ApiError::BadRequest(status.message().to_string()),
            Code::PermissionDenied => ApiError::PermissionDenied(status.message().to_string()),
            Code::FailedPrecondition => match status.message().to_lowercase().as_str() {
                // TODO: find a better way than matching on the error message
                "no valid license" => ApiError::EnterpriseNotEnabled,
                _ => ApiError::PreconditionRequired(status.message().to_string()),
            },
            Code::Unavailable => ApiError::CoreTimeout,
            Code::NotFound => ApiError::NotFound(status.to_string()),
            _ => ApiError::Unexpected(status.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn precondition(message: &str) -> ApiError {
        CoreError {
            status_code: Code::FailedPrecondition as i32,
            message: message.into(),
        }
        .into()
    }

    #[test]
    fn test_oidc_not_completed_maps_to_own_variant() {
        // MFA configuration and VPN client MFA polls respectively.
        for message in [
            "OIDC authentication not completed",
            "OIDC authentication not completed yet",
        ] {
            assert!(
                matches!(precondition(message), ApiError::OidcNotCompleted(msg) if msg == message),
                "{message}"
            );
        }
        assert!(matches!(
            precondition("something else"),
            ApiError::PreconditionRequired(_)
        ));
    }
}
