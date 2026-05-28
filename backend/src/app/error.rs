use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;

use crate::core::hypergryph::fetch::FetchError;

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
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", None),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", None),
            Self::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", None),
            Self::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", None),
            Self::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED", None),
            Self::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT", None),
            Self::ValidationFailed(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDATION_FAILED",
                Some(errors.clone()),
            ),
            Self::Internal(e) => {
                tracing::error!(error = %e, "internal server error");
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", None)
            }
            Self::ServiceUnavailable => {
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
        match &e {
            sqlx::Error::RowNotFound => Self::NotFound,
            sqlx::Error::Database(db) => match db.code().as_deref() {
                Some("23505") => Self::Conflict("resource already exists".into()),
                Some("22001") => Self::BadRequest("value too long".into()),
                _ => Self::Internal(e.into()),
            },
            _ => Self::Internal(e.into()),
        }
    }
}

impl From<redis::RedisError> for ApiError {
    fn from(e: redis::RedisError) -> Self {
        tracing::error!(error = %e, "redis error");
        Self::ServiceUnavailable
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(_: jsonwebtoken::errors::Error) -> Self {
        Self::Unauthorized
    }
}

impl From<FetchError> for ApiError {
    fn from(e: FetchError) -> Self {
        match e {
            FetchError::NotLoggedIn => Self::Unauthorized,
            FetchError::DomainNotFound(_, _) => Self::BadRequest("unsupported server".into()),
            FetchError::RequestFailed(e) => Self::Internal(e.into()),
            FetchError::ParseError(msg) => Self::BadRequest(msg),
            FetchError::Upstream(err) => Self::BadRequest(format!(
                "{} ({})",
                if err.msg.is_empty() {
                    &err.message
                } else {
                    &err.msg
                },
                err.code
            )),
        }
    }
}
