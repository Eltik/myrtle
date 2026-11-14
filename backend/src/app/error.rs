use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

use crate::core::hypergryph::fetch::FetchError;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    /// No game session in cache or durable store; only a fresh email-code sign-in
    /// recovers. 400 with code `GAME_LOGIN_REQUIRED` so clients can branch on it.
    #[error("no game session - login again")]
    GameLoginRequired,
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    /// A 404 that says WHAT is missing (`NOT_FOUND` code, prose message), for
    /// the cases where the id is known but its backing file is not there.
    #[error("{0}")]
    NotFoundMessage(String),
    #[error("rate limited")]
    RateLimited,
    #[error("{0}")]
    Conflict(String),
    #[error("validation failed")]
    ValidationFailed(Vec<FieldError>),

    #[error("internal error")]
    Internal(#[from] anyhow::Error),
    #[error("service unavailable")]
    ServiceUnavailable,
    /// The game server is down for maintenance, so nothing can be synced from
    /// it. 503 with code `GAME_MAINTENANCE`; `ends_at` is the announced end
    /// (unix seconds) when one is known.
    #[error("the game is down for maintenance")]
    GameMaintenance { ends_at: Option<i64> },
}

impl ApiError {
    /// An equivalent error for a second caller of the same work: the same
    /// status and message. `Internal` carries an `anyhow::Error`, which cannot
    /// be cloned, so its chain is flattened into the message.
    #[must_use]
    pub fn shared_copy(&self) -> Self {
        match self {
            Self::BadRequest(m) => Self::BadRequest(m.clone()),
            Self::GameLoginRequired => Self::GameLoginRequired,
            Self::Unauthorized => Self::Unauthorized,
            Self::Forbidden => Self::Forbidden,
            Self::NotFound => Self::NotFound,
            Self::NotFoundMessage(m) => Self::NotFoundMessage(m.clone()),
            Self::RateLimited => Self::RateLimited,
            Self::Conflict(m) => Self::Conflict(m.clone()),
            Self::ValidationFailed(errors) => Self::ValidationFailed(errors.clone()),
            Self::Internal(e) => Self::Internal(anyhow::anyhow!("{e:#}")),
            Self::ServiceUnavailable => Self::ServiceUnavailable,
            Self::GameMaintenance { ends_at } => Self::GameMaintenance { ends_at: *ends_at },
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
    /// Present only on `GAME_MAINTENANCE` (503) when the announced end is
    /// known: unix seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<i64>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, details) = match &self {
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", None),
            Self::GameLoginRequired => (StatusCode::BAD_REQUEST, "GAME_LOGIN_REQUIRED", None),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", None),
            Self::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", None),
            Self::NotFound | Self::NotFoundMessage(_) => (StatusCode::NOT_FOUND, "NOT_FOUND", None),
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
            Self::GameMaintenance { .. } => {
                (StatusCode::SERVICE_UNAVAILABLE, "GAME_MAINTENANCE", None)
            }
        };
        let ends_at = match &self {
            Self::GameMaintenance { ends_at } => *ends_at,
            _ => None,
        };

        let body = ErrorBody {
            error: ErrorDetail {
                code,
                message: self.to_string(),
                details,
                ends_at,
            },
        };

        (status, axum::Json(body)).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        match &e {
            sqlx::Error::RowNotFound => Self::NotFound,
            // Pool full and `acquire_timeout` elapsed: backpressure, not a fault, so 503.
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
            // Every server this backend knows has a network config to load, so
            // a missing domain means that load has not succeeded yet (it is
            // retried in the background, `loaders::spawn_retry`), not that the
            // server is unsupported. As a 400 the login dialog read it as the
            // player's code being turned down and told them to request another.
            FetchError::DomainNotFound(server, domain) => {
                tracing::warn!(
                    server = server.as_str(),
                    domain = ?domain,
                    "request needs a domain that has not loaded"
                );
                Self::ServiceUnavailable
            }
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

/// A failure of one of the OAuth protocol endpoints (`/oauth/token`,
/// `/oauth/revoke`, `/oauth/userinfo`), answered in the format RFC 6749 5.2
/// gives instead of [`ErrorBody`]: the caller is an OAuth client library, and
/// `{"error": "invalid_grant"}` is the shape every one of them reads.
///
/// The site's own OAuth routes (registering an application, the consent page)
/// are called by this site's web client and keep [`ApiError`].
#[derive(Debug)]
pub struct OAuthError {
    status: StatusCode,
    error: &'static str,
    description: String,
    /// The `WWW-Authenticate` challenge a 401 carries.
    challenge: Option<&'static str>,
}

impl OAuthError {
    fn new(status: StatusCode, error: &'static str, description: impl Into<String>) -> Self {
        Self {
            status,
            error,
            description: description.into(),
            challenge: None,
        }
    }

    /// A parameter is missing, repeated or malformed.
    pub fn invalid_request(description: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "invalid_request", description)
    }

    /// The application did not authenticate: unknown id or wrong secret. The
    /// two are not told apart, so a response never confirms that an id exists.
    pub fn invalid_client() -> Self {
        Self {
            challenge: Some(r#"Basic realm="myrtle.moe""#),
            ..Self::new(
                StatusCode::UNAUTHORIZED,
                "invalid_client",
                "client authentication failed: unknown client_id or wrong client_secret",
            )
        }
    }

    /// The code or refresh token is invalid, expired, spent, or was issued to
    /// another application or for another redirect URI.
    pub fn invalid_grant(description: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "invalid_grant", description)
    }

    pub fn unsupported_grant_type() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "unsupported_grant_type",
            "grant_type must be authorization_code or refresh_token",
        )
    }

    pub fn invalid_scope(description: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "invalid_scope", description)
    }

    /// The access token is missing, unknown, expired or revoked (RFC 6750 3.1).
    pub fn invalid_token() -> Self {
        Self {
            challenge: Some(r#"Bearer error="invalid_token""#),
            ..Self::new(
                StatusCode::UNAUTHORIZED,
                "invalid_token",
                "the access token is missing, expired or revoked",
            )
        }
    }

    /// The access token is valid but was not granted what this endpoint needs.
    pub fn insufficient_scope(scope: &str) -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "insufficient_scope",
            format!("the access token lacks the {scope} scope"),
        )
    }

    pub const fn status(&self) -> StatusCode {
        self.status
    }

    /// The RFC 6749 error code, e.g. `invalid_grant`.
    pub const fn code(&self) -> &'static str {
        self.error
    }
}

impl IntoResponse for OAuthError {
    fn into_response(self) -> Response {
        let body = crate::database::models::oauth::OAuthErrorBody {
            error: self.error,
            error_description: self.description,
        };
        let mut response = (self.status, axum::Json(body)).into_response();
        let headers = response.headers_mut();
        // RFC 6749 5.1: nothing from these endpoints may be stored by a cache.
        headers.insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-store"),
        );
        if let Some(challenge) = self.challenge {
            headers.insert(
                axum::http::header::WWW_AUTHENTICATE,
                axum::http::HeaderValue::from_static(challenge),
            );
        }
        response
    }
}

impl From<sqlx::Error> for OAuthError {
    fn from(e: sqlx::Error) -> Self {
        if matches!(e, sqlx::Error::PoolTimedOut) {
            tracing::warn!("database pool exhausted; shedding OAuth request");
            return Self::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "temporarily_unavailable",
                "the server is busy; retry shortly",
            );
        }
        tracing::error!(error = %e, "OAuth endpoint failed");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "server_error",
            "unexpected server-side failure",
        )
    }
}
