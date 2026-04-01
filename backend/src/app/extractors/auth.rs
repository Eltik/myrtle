use axum::extract::FromRequestParts;
use axum::http::HeaderMap;
use axum::http::request::Parts;

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::auth::jwt;
use crate::core::auth::permissions::GlobalRole;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub uid: String,
    pub server: String,
    pub role: GlobalRole,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Try service key first (for internal SSR calls)
        if let Some(key) = parts.headers.get("x-service-key")
            && key.as_bytes() == state.config.service_key.as_bytes()
        {
            return Ok(AuthUser {
                user_id: "service".to_owned(),
                uid: "service".to_owned(),
                server: "internal".to_owned(),
                role: GlobalRole::SuperAdmin,
            });
        }

        // Otherwise, require Bearer token
        let token = extract_bearer(&parts.headers)?;
        let claims = jwt::verify_token(&state.config.jwt_secret, token)?;

        let role = parse_role(&claims.role);

        Ok(AuthUser {
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
            Ok(user) => Ok(MaybeAuthUser(Some(user))),
            Err(_) => Ok(MaybeAuthUser(None)),
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

fn parse_role(role: &str) -> GlobalRole {
    match role {
        "super_admin" => GlobalRole::SuperAdmin,
        "tier_list_admin" => GlobalRole::TierListAdmin,
        "tier_list_editor" => GlobalRole::TierListEditor,
        _ => GlobalRole::User,
    }
}
