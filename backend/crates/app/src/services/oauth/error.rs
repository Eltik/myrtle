use myrtle_db::DbError;

use super::protocol::Scope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OAuthErrorCode {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnsupportedGrantType,
    InvalidScope,
    InvalidToken,
    InsufficientScope,
    ServerError,
    TemporarilyUnavailable,
}

impl OAuthErrorCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::InvalidClient => "invalid_client",
            Self::InvalidGrant => "invalid_grant",
            Self::UnsupportedGrantType => "unsupported_grant_type",
            Self::InvalidScope => "invalid_scope",
            Self::InvalidToken => "invalid_token",
            Self::InsufficientScope => "insufficient_scope",
            Self::ServerError => "server_error",
            Self::TemporarilyUnavailable => "temporarily_unavailable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{}: {description}", .code.as_str())]
pub struct ProtocolError {
    pub code: OAuthErrorCode,
    pub description: String,
}

impl ProtocolError {
    pub fn new(code: OAuthErrorCode, description: impl Into<String>) -> Self {
        Self {
            code,
            description: description.into(),
        }
    }

    pub fn invalid_request(description: impl Into<String>) -> Self {
        Self::new(OAuthErrorCode::InvalidRequest, description)
    }

    #[must_use]
    pub fn invalid_client() -> Self {
        Self::new(
            OAuthErrorCode::InvalidClient,
            "client authentication failed: unknown client_id or wrong client_secret",
        )
    }

    pub fn invalid_grant(description: impl Into<String>) -> Self {
        Self::new(OAuthErrorCode::InvalidGrant, description)
    }

    #[must_use]
    pub fn unsupported_grant_type() -> Self {
        Self::new(
            OAuthErrorCode::UnsupportedGrantType,
            "grant_type must be authorization_code or refresh_token",
        )
    }

    pub fn invalid_scope(description: impl Into<String>) -> Self {
        Self::new(OAuthErrorCode::InvalidScope, description)
    }

    #[must_use]
    pub fn invalid_token() -> Self {
        Self::new(
            OAuthErrorCode::InvalidToken,
            "the access token is missing, expired or revoked",
        )
    }

    #[must_use]
    pub fn insufficient_scope(scope: Scope) -> Self {
        Self::new(
            OAuthErrorCode::InsufficientScope,
            format!("the access token lacks the {scope} scope"),
        )
    }
}

impl From<DbError> for ProtocolError {
    fn from(e: DbError) -> Self {
        if matches!(e, DbError::PoolTimedOut) {
            tracing::warn!("database pool exhausted; shedding OAuth request");
            return Self::new(
                OAuthErrorCode::TemporarilyUnavailable,
                "the server is busy; retry shortly",
            );
        }
        tracing::error!(error = %e, "OAuth endpoint failed");
        Self::new(
            OAuthErrorCode::ServerError,
            "unexpected server-side failure",
        )
    }
}
