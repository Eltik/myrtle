use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use myrtle_app::services::oauth::error::{OAuthErrorCode, ProtocolError};
use serde::Serialize;

pub type OAuthResult<T> = Result<T, OAuthError>;

#[derive(Debug)]
pub struct OAuthError(pub ProtocolError);

impl<E: Into<ProtocolError>> From<E> for OAuthError {
    fn from(e: E) -> Self {
        Self(e.into())
    }
}

#[derive(Debug, Serialize)]
pub struct OAuthErrorBody {
    pub error: &'static str,
    pub error_description: String,
}

#[must_use]
pub const fn status(code: OAuthErrorCode) -> StatusCode {
    match code {
        OAuthErrorCode::InvalidRequest
        | OAuthErrorCode::InvalidGrant
        | OAuthErrorCode::UnsupportedGrantType
        | OAuthErrorCode::InvalidScope => StatusCode::BAD_REQUEST,
        OAuthErrorCode::InvalidClient | OAuthErrorCode::InvalidToken => StatusCode::UNAUTHORIZED,
        OAuthErrorCode::InsufficientScope => StatusCode::FORBIDDEN,
        OAuthErrorCode::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
        OAuthErrorCode::TemporarilyUnavailable => StatusCode::SERVICE_UNAVAILABLE,
    }
}

const fn challenge(code: OAuthErrorCode) -> Option<&'static str> {
    match code {
        OAuthErrorCode::InvalidClient => Some(r#"Basic realm="myrtle.moe""#),
        OAuthErrorCode::InvalidToken => Some(r#"Bearer error="invalid_token""#),
        _ => None,
    }
}

impl IntoResponse for OAuthError {
    fn into_response(self) -> Response {
        let ProtocolError { code, description } = self.0;
        let body = OAuthErrorBody {
            error: code.as_str(),
            error_description: description,
        };
        let mut response = (status(code), Json(body)).into_response();
        let headers = response.headers_mut();
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store")); // RFC 6749 5.1
        if let Some(challenge) = challenge(code) {
            headers.insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static(challenge),
            );
        }
        response
    }
}
