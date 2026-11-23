use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use myrtle_app::AppError;
use myrtle_hypergryph::UpstreamError;
use myrtle_types::FieldError;
use serde::Serialize;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    BadRequest,
    GameLoginRequired,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    ValidationFailed,
    RateLimited,
    Internal,
    ServiceUnavailable,
    GameMaintenance,
    Timeout,
}

impl ErrorCode {
    pub const ALL: [Self; 12] = [
        Self::BadRequest,
        Self::GameLoginRequired,
        Self::Unauthorized,
        Self::Forbidden,
        Self::NotFound,
        Self::Conflict,
        Self::ValidationFailed,
        Self::RateLimited,
        Self::Internal,
        Self::ServiceUnavailable,
        Self::GameMaintenance,
        Self::Timeout,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BadRequest => "BAD_REQUEST",
            Self::GameLoginRequired => "GAME_LOGIN_REQUIRED",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Forbidden => "FORBIDDEN",
            Self::NotFound => "NOT_FOUND",
            Self::Conflict => "CONFLICT",
            Self::ValidationFailed => "VALIDATION_FAILED",
            Self::RateLimited => "RATE_LIMITED",
            Self::Internal => "INTERNAL_ERROR",
            Self::ServiceUnavailable => "SERVICE_UNAVAILABLE",
            Self::GameMaintenance => "GAME_MAINTENANCE",
            Self::Timeout => "TIMEOUT",
        }
    }

    #[must_use]
    pub const fn status(self) -> StatusCode {
        match self {
            Self::BadRequest | Self::GameLoginRequired => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict => StatusCode::CONFLICT,
            Self::ValidationFailed => StatusCode::UNPROCESSABLE_ENTITY,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceUnavailable | Self::GameMaintenance => StatusCode::SERVICE_UNAVAILABLE,
            Self::Timeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }

    pub fn respond(self, message: impl Into<String>) -> Response {
        let detail = ErrorDetail {
            code: self.as_str(),
            message: message.into(),
            details: None,
            ends_at: None,
        };
        (self.status(), Json(ErrorBody { error: detail })).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<FieldError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<i64>,
}

#[derive(Debug)]
pub struct ApiError(pub AppError);

impl<E: Into<AppError>> From<E> for ApiError {
    fn from(e: E) -> Self {
        Self(e.into())
    }
}

impl ApiError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self(AppError::BadRequest(message.into()))
    }

    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        match &self.0 {
            AppError::BadRequest(_) => ErrorCode::BadRequest,
            AppError::Validation(_) => ErrorCode::ValidationFailed,
            AppError::Unauthorized => ErrorCode::Unauthorized,
            AppError::Forbidden => ErrorCode::Forbidden,
            AppError::NotFound(_) => ErrorCode::NotFound,
            AppError::Conflict(_) => ErrorCode::Conflict,
            AppError::RateLimited => ErrorCode::RateLimited,
            AppError::GameLoginRequired => ErrorCode::GameLoginRequired,
            AppError::GameMaintenance { .. } => ErrorCode::GameMaintenance,
            AppError::Unavailable => ErrorCode::ServiceUnavailable,
            AppError::Upstream(e) => upstream_code(e),
            AppError::Internal(_) => ErrorCode::Internal,
        }
    }
}

const fn upstream_code(e: &UpstreamError) -> ErrorCode {
    match e {
        UpstreamError::NotLoggedIn => ErrorCode::Unauthorized,
        UpstreamError::Maintenance(_) => ErrorCode::GameMaintenance,
        UpstreamError::ServiceAccountRefused { .. } => ErrorCode::ServiceUnavailable,
        UpstreamError::DomainNotFound { .. }
        | UpstreamError::Unsupported { .. }
        | UpstreamError::Parse { .. }
        | UpstreamError::ApiCode { .. }
        | UpstreamError::Envelope(_)
        | UpstreamError::LoginRejected { .. }
        | UpstreamError::RiskControl { .. } => ErrorCode::BadRequest,
        UpstreamError::Transport(_)
        | UpstreamError::Timeout(_)
        | UpstreamError::HttpStatus { .. }
        | UpstreamError::Store(_) => ErrorCode::Internal,
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let code = self.code();
        let message = match (&self.0, code) {
            (e, ErrorCode::Internal) => {
                tracing::error!(error = %e, "internal server error");
                "internal server error".to_owned()
            }
            (AppError::Upstream(UpstreamError::Envelope(e)), _) => {
                let text = if e.msg.is_empty() { &e.message } else { &e.msg };
                format!("{text} ({})", e.code)
            }
            (AppError::Upstream(UpstreamError::ApiCode { code, message, .. }), _) => {
                format!("{message} ({code})")
            }
            (AppError::Upstream(UpstreamError::DomainNotFound { .. }), _) => {
                "unsupported server".to_owned()
            }
            (AppError::Upstream(_), ErrorCode::Unauthorized) => AppError::Unauthorized.to_string(),
            (AppError::Upstream(_), ErrorCode::ServiceUnavailable) => {
                AppError::Unavailable.to_string()
            }
            (AppError::Upstream(_), ErrorCode::GameMaintenance) => {
                AppError::GameMaintenance { ends_at: None }.to_string()
            }
            (e, _) => e.to_string(),
        };
        let (details, ends_at) = match self.0 {
            AppError::Validation(details) => (Some(details), None),
            AppError::GameMaintenance { ends_at } => (None, ends_at),
            _ => (None, None),
        };
        let detail = ErrorDetail {
            code: code.as_str(),
            message,
            details,
            ends_at,
        };
        (code.status(), Json(ErrorBody { error: detail })).into_response()
    }
}
