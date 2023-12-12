use crate::error::ApiError;

pub mod enrollment;
pub mod password_reset;
pub mod shared;

pub type ApiResult<T> = Result<T, ApiError>;
