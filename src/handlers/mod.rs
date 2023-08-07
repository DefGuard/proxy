use crate::error::ApiError;

pub mod enrollment;

pub type ApiResult<T> = Result<T, ApiError>;
