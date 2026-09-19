use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

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

impl ApiError {
    /// An equivalent error for a second caller of the same work: the same
    /// status and message. `Internal` carries an `anyhow::Error`, which cannot
    /// be cloned, so its chain is flattened into the message.
    #[must_use]
    pub fn shared_copy(&self) -> Self {
        match self {
            Self::BadRequest(m) => Self::BadRequest(m.clone()),
            Self::Unauthorized => Self::Unauthorized,
            Self::Forbidden => Self::Forbidden,
            Self::NotFound => Self::NotFound,
            Self::RateLimited => Self::RateLimited,
            Self::Conflict(m) => Self::Conflict(m.clone()),
            Self::ValidationFailed(errors) => Self::ValidationFailed(errors.clone()),
            Self::Internal(e) => Self::Internal(anyhow::anyhow!("{e:#}")),
            Self::ServiceUnavailable => Self::ServiceUnavailable,
        }
    }
}

/// One field-level failure inside a `VALIDATION_FAILED` response.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FieldError {
    /// The offending field, as named in the request body.
    #[schema(example = "email")]
    pub field: String,
    #[schema(example = "must be a valid email address")]
    pub message: String,
}

/// The single error envelope every failing endpoint returns.
///
/// Public because it is the documented error schema: `openapi.rs` registers it
/// as a component and handlers reference it from their `responses(...)` lists.
#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorDetail {
    /// Stable machine-readable code. Clients should branch on this, not on
    /// `message`, which is prose and may change.
    #[schema(value_type = String, example = "NOT_FOUND")]
    pub code: &'static str,
    #[schema(example = "not found")]
    pub message: String,
    /// Present only on `VALIDATION_FAILED` (422).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<FieldError>>,
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
            // The pool is full and `acquire_timeout` elapsed. That is
            // backpressure, not an internal fault, so it is a 503: the code a
            // client, a proxy and a dashboard all know how to act on.
            sqlx::Error::PoolTimedOut => {
                tracing::warn!("database pool exhausted; shedding request");
                Self::ServiceUnavailable
            }
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
