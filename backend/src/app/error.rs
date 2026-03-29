use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    // Client errors
    #[error("{0}")]
    BadRequest(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("rate limited")]
    RateLimited,
    #[error("{0}")]
    Conflict(String),
    #[error("validation failed")]
    ValidationFailed(Vec<FieldError>),

    // Server errors
    #[error("internal error")]
    Internal(#[from] anyhow::Error),
    #[error("service unavailable")]
    ServiceUnavailable,
}

#[derive(Debug, Clone, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Vec<FieldError>>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, details) = match &self {
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", None),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", None),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", None),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", None),
            ApiError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED", None),
            ApiError::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT", None),
            ApiError::ValidationFailed(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDATION_FAILED",
                Some(errors.clone()),
            ),
            ApiError::Internal(e) => {
                tracing::error!(error = %e, "internal server error");
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", None)
            }
            ApiError::ServiceUnavailable => {
                (StatusCode::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE", None)
            }
        };

        let body = ErrorBody {
            error: ErrorDetail {
                code,
                message: self.to_string(), // uses thiserror Display
                details,
            },
        };

        (status, axum::Json(body)).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => ApiError::NotFound,
            other => ApiError::Internal(other.into()),
        }
    }
}

impl From<redis::RedisError> for ApiError {
    fn from(e: redis::RedisError) -> Self {
        tracing::error!(error = %e, "redis error");
        ApiError::ServiceUnavailable
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(_: jsonwebtoken::errors::Error) -> Self {
        ApiError::Unauthorized
    }
}
