use crate::proto::CoreError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tonic::metadata::errors::InvalidMetadataValue;
use tonic::{Code, Status};

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Unauthorized")]
    Unauthorized,
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
    #[error("Permission denied")]
    PermissionDenied,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        error!("{}", self);
        let (status, error_message) = match self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::PermissionDenied => (StatusCode::FORBIDDEN, self.to_string()),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl From<CoreError> for ApiError {
    fn from(core_error: CoreError) -> Self {
        // convert to tonic::Status first
        let status = Status::new(Code::from(core_error.status_code), core_error.message);
        match status.code() {
            Code::Unauthenticated => ApiError::Unauthorized,
            Code::InvalidArgument => ApiError::BadRequest(status.message().to_string()),
            Code::PermissionDenied => ApiError::PermissionDenied,
            _ => ApiError::Unexpected(status.to_string()),
        }
    }
}
