use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use tonic::{Code, Status, metadata::errors::InvalidMetadataValue};

use crate::proto::{CoreError, CoreErrorCode};

/// Core's message prefix for an OIDC MFA poll made before the browser round trip completes.
const OIDC_NOT_COMPLETED: &str = "OIDC authentication not completed";

/// Expected while clients poll OIDC, so not an error.
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
    #[error("{0}")]
    OidcNotCompleted(String),
    #[error("Bad request: {0}")]
    NotFound(String),
    #[error("PostureRejected: {0:?}")]
    PostureRejected(Vec<String>),
    #[error("Failed to get client IP address")]
    ClientIpError,
    #[error("{0}")]
    WithCode(Box<ApiError>, CoreErrorCode),
}

impl ApiError {
    fn status_and_message(self) -> (StatusCode, String) {
        match self {
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
            Self::WithCode(inner, _) => inner.status_and_message(),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        }
    }
}

/// The web UI matches on the lowercase name without the enum prefix, e.g. `oidc_account_mismatch`.
fn code_name(code: CoreErrorCode) -> String {
    code.as_str_name()
        .trim_start_matches("CORE_ERROR_CODE_")
        .to_ascii_lowercase()
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if matches!(self, Self::OidcNotCompleted(_)) {
            debug!("{self}");
        } else {
            error!("{self}");
        }
        let code = match &self {
            Self::WithCode(_, code) => Some(code_name(*code)),
            _ => None,
        };
        let (status, error_message) = self.status_and_message();

        (status, Json(json!({"error": error_message, "code": code}))).into_response()
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
        let code = core_error.code();
        // convert to tonic::Status first
        let status = Status::new(Code::from(core_error.status_code), core_error.message);
        let error = match status.code() {
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
        };
        match code {
            CoreErrorCode::Unspecified => error,
            code => ApiError::WithCode(Box::new(error), code),
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
            ..Default::default()
        }
        .into()
    }

    async fn response_json(error: ApiError) -> (StatusCode, serde_json::Value) {
        let response = error.into_response();
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn test_core_error_code_reaches_response_body() {
        let coded = CoreError {
            status_code: Code::Unauthenticated as i32,
            message: "unauthorized".into(),
            code: CoreErrorCode::OidcAccountMismatch.into(),
        };
        let (status, body) = response_json(coded.into()).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(
            body,
            json!({"error": "unauthorized", "code": "oidc_account_mismatch"})
        );

        // Older Core versions send no code.
        let uncoded = CoreError {
            status_code: Code::InvalidArgument as i32,
            message: "invalid state data".into(),
            ..Default::default()
        };
        let (status, body) = response_json(uncoded.into()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body, json!({"error": "invalid state data", "code": null}));
    }

    #[test]
    fn test_oidc_not_completed_maps_to_own_variant() {
        // Core's MFA config and client MFA wordings.
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
