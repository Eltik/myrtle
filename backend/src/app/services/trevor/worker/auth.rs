//! The scoped worker key: the header, the check, and the extractor that
//! guards every worker route.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use subtle::ConstantTimeEq;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::state::AppState;

/// The header the worker presents its key in.
pub const WORKER_KEY_HEADER: &str = "x-trevor-worker-key";

/// Proof the caller holds `TREVOR_WORKER_KEY`: the extractor every worker
/// route (and only those) takes.
///
/// An empty configured key turns the routes off with a 503. The global
/// service key is refused here, and nothing else accepts the worker key.
/// `TREVOR_SCOPED_WORKER_KEY=0` restores the old gate (service key only).
#[derive(Debug, Clone, Copy)]
pub struct WorkerKey;

/// The old gate: the service principal only, never a user token.
pub fn require_service(auth: &AuthUser) -> Result<(), ApiError> {
    if auth.user_id == "service" {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

/// The scoped check, pure: `presented` against `configured` in constant time.
pub fn check_worker_key(configured: &str, presented: Option<&[u8]>) -> Result<(), ApiError> {
    if configured.is_empty() {
        return Err(ApiError::ServiceUnavailableMessage(
            "the Trevor worker routes are off: TREVOR_WORKER_KEY is not set".into(),
        ));
    }
    match presented {
        Some(key) if !key.is_empty() && bool::from(key.ct_eq(configured.as_bytes())) => Ok(()),
        _ => Err(ApiError::Unauthorized),
    }
}

/// The request carries the valid scoped worker key (constant time): the rate
/// limiter lets it through, as it lets the service key. An invalid key, an
/// empty configured key or `TREVOR_SCOPED_WORKER_KEY=0` reads as false.
pub fn presents_worker_key(headers: &axum::http::HeaderMap, state: &AppState) -> bool {
    let cfg = state.trevor.config();
    cfg.scoped_worker_key
        && check_worker_key(
            &cfg.worker_key,
            headers.get(WORKER_KEY_HEADER).map(|v| v.as_bytes()),
        )
        .is_ok()
}

impl FromRequestParts<AppState> for WorkerKey {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cfg = state.trevor.config();
        if !cfg.scoped_worker_key {
            let auth = AuthUser::from_request_parts(parts, state).await?;
            require_service(&auth)?;
            return Ok(Self);
        }
        check_worker_key(
            &cfg.worker_key,
            parts.headers.get(WORKER_KEY_HEADER).map(|v| v.as_bytes()),
        )?;
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_worker_key_is_checked_and_an_empty_one_disables_the_routes() {
        assert!(matches!(
            check_worker_key("", Some(b"anything")),
            Err(ApiError::ServiceUnavailableMessage(_))
        ));
        assert!(matches!(
            check_worker_key("k-123", None),
            Err(ApiError::Unauthorized)
        ));
        assert!(matches!(
            check_worker_key("k-123", Some(b"")),
            Err(ApiError::Unauthorized)
        ));
        assert!(matches!(
            check_worker_key("k-123", Some(b"k-124")),
            Err(ApiError::Unauthorized)
        ));
        assert!(check_worker_key("k-123", Some(b"k-123")).is_ok());
    }
}
