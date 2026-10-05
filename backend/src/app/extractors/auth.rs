use axum::extract::FromRequestParts;
use axum::http::HeaderMap;
use axum::http::request::Parts;
use subtle::ConstantTimeEq;

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::auth::jwt::verify_token;
use crate::core::auth::permissions::GlobalRole;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub uid: String,
    pub server: String,
    pub role: GlobalRole,
}

impl AuthUser {
    /// The subject as a `Uuid`; `Unauthorized` for a non-UUID subject such as the
    /// internal "service" principal.
    pub fn user_uuid(&self) -> Result<uuid::Uuid, ApiError> {
        self.user_id.parse().map_err(|_| ApiError::Unauthorized)
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    // `async fn` is the idiomatic spelling of this trait method even though nothing awaits.
    #[allow(clippy::unused_async_trait_impl)]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Service key first (internal SSR calls), compared in constant time.
        // Both sides must be non-empty: `ct_eq` on two empty slices is equal, so an
        // empty key would match an empty header. `AppConfig::require_secret` also
        // refuses an empty key at boot.
        if let Some(key) = parts.headers.get("x-service-key")
            && !key.is_empty()
            && !state.config.service_key.is_empty()
            && key
                .as_bytes()
                .ct_eq(state.config.service_key.as_bytes())
                .into()
        {
            return Ok(Self {
                user_id: "service".to_owned(),
                uid: "service".to_owned(),
                server: "internal".to_owned(),
                role: GlobalRole::SuperAdmin,
            });
        }

        let token = extract_bearer(&parts.headers)?;
        let claims = verify_token(&state.config.jwt_secret, token)?;

        let role = parse_role(&claims.role);

        Ok(Self {
            user_id: claims.sub,
            uid: claims.uid,
            server: claims.server,
            role,
        })
    }
}

pub struct MaybeAuthUser(pub Option<AuthUser>);

impl FromRequestParts<AppState> for MaybeAuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(Self(Some(user))),
            Err(_) => Ok(Self(None)),
        }
    }
}

fn extract_bearer(headers: &HeaderMap) -> Result<&str, ApiError> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(ApiError::Unauthorized)
}

/// Unknown or absent role degrades to `User`: a token minted before the role
/// existed still authenticates, minus the privilege. `GlobalRole::from_str` owns
/// the spellings so this can't drift from `Display`.
fn parse_role(role: &str) -> GlobalRole {
    GlobalRole::from_str(role).unwrap_or_default()
}
