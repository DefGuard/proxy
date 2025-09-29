use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tonic::{metadata::errors::InvalidMetadataValue, Code, Status};

use crate::proto::CoreError;

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
    #[error("Bad request: {0}")]
    NotFound(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        error!("{self}");
        let (status, error_message) = match self {
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::PermissionDenied(msg) => (StatusCode::FORBIDDEN, msg),
            Self::EnterpriseNotEnabled => (
                StatusCode::PAYMENT_REQUIRED,
                "Enterprise features are not enabled".to_string(),
            ),
            Self::PreconditionRequired(msg) => (StatusCode::PRECONDITION_REQUIRED, msg),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
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
