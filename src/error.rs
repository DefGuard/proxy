use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tonic::{metadata::errors::InvalidMetadataValue, Code};
use tracing::error;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Session cookie not found")]
    CookieNotFound,
    #[error("Unexpected error: {0}")]
    Unexpected(String),
    #[error(transparent)]
    InvalidMetadata(#[from] InvalidMetadataValue),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        error!("{}", self);
        let (status, error_message) = match self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
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

// implement more granular mapping of gRPC errors
impl From<tonic::Status> for ApiError {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            Code::Unauthenticated => ApiError::Unauthorized,
            Code::InvalidArgument => ApiError::BadRequest(status.message().to_string()),
            _ => ApiError::Unexpected(status.to_string()),
        }
    }
}
