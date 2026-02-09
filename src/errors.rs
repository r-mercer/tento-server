use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use async_graphql::ErrorExtensions;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl AppError {
    fn error_code(&self) -> &'static str {
        match self {
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::AlreadyExists(_) => "ALREADY_EXISTS",
            AppError::ValidationError(_) => "VALIDATION_ERROR",
            AppError::DatabaseError(_) => "DATABASE_ERROR",
            AppError::Unauthorized(_) => "UNAUTHORIZED",
            AppError::InternalError(_) => "INTERNAL_ERROR",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::AlreadyExists(_) => StatusCode::CONFLICT,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error: self.to_string(),
            code: self.status_code().as_u16(),
        })
    }
}
impl From<mongodb::error::Error> for AppError {
    fn from(err: mongodb::error::Error) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}
impl From<mongodb::bson::ser::Error> for AppError {
    fn from(err: mongodb::bson::ser::Error) -> Self {
        AppError::InternalError(format!("BSON serialization error: {}", err))
    }
}
impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        AppError::ValidationError(err.to_string())
    }
}
impl From<async_graphql::Error> for AppError {
    fn from(err: async_graphql::Error) -> Self {
        AppError::InternalError(err.message)
    }
}
impl ErrorExtensions for AppError {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(self.to_string()).extend_with(|_err, e| {
            e.set("code", self.error_code());
        })
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            AppError::NotFound("test".into()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::AlreadyExists("test".into()).status_code(),
            StatusCode::CONFLICT
        );
        assert_eq!(
            AppError::ValidationError("test".into()).status_code(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn test_error_messages() {
        let err = AppError::NotFound("user".into());
        assert_eq!(err.to_string(), "Not found: user");
    }
}
