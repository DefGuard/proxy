use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tonic::metadata::errors::InvalidMetadataValue;
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
    #[error("Core gRPC response timeout")]
    CoreTimeout,
    #[error("Invalid core gRPC response type received")]
    InvalidResponseType,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        error!("{}", self);
        let (status, error_message) = match self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            // Self::CoreTimeout => (StatusCode::REQUEST_TIMEOUT, self.to_string()),
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
