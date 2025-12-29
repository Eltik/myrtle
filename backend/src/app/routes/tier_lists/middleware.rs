use axum::http::HeaderMap;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::authentication::jwt;
use crate::core::authentication::permissions::{AuthContext, GlobalRole, Permission};
use crate::database::models::tier_lists::{TierList, TierListPermission};

/// Extract authentication context from request headers
pub fn extract_auth_context(
    headers: &HeaderMap,
    jwt_secret: &str,
) -> Result<AuthContext, ApiError> {
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::BadRequest("Missing authorization header".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::BadRequest("Invalid authorization format".into()))?;

    let claims = jwt::verify_token(jwt_secret, token)
        .map_err(|_| ApiError::BadRequest("Invalid or expired token".into()))?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| ApiError::Internal("Invalid user ID in token".into()))?;

    let role = claims
        .role
        .parse::<GlobalRole>()
        .unwrap_or(GlobalRole::User);

    Ok(AuthContext::new(user_id, role))
}

/// Check if user has required permission on a tier list
pub async fn check_permission(
    state: &AppState,
    auth: &AuthContext,
    tier_list_id: Uuid,
    required: Permission,
) -> Result<(), ApiError> {
    // Super admins and tier list admins have full access
    if auth.role.is_tier_list_admin() {
        return Ok(());
    }

    // Check if user has the specific permission
    let has_permission = TierListPermission::has_permission(
        &state.db,
        tier_list_id,
        auth.user_id,
        &required.to_string(),
    )
    .await
    .map_err(|e| {
        eprintln!("Database error checking permission: {e:?}");
        ApiError::Internal("Failed to check permissions".into())
    })?;

    if has_permission {
        return Ok(());
    }

    // Check if user has a higher permission that grants the required one
    for perm in Permission::all() {
        if perm.grants(required) {
            let has_higher = TierListPermission::has_permission(
                &state.db,
                tier_list_id,
                auth.user_id,
                &perm.to_string(),
            )
            .await
            .unwrap_or(false);

            if has_higher {
                return Ok(());
            }
        }
    }

    Err(ApiError::BadRequest("Insufficient permissions".into()))
}

/// Helper to get tier list by slug and verify it exists
pub async fn get_tier_list_by_slug(state: &AppState, slug: &str) -> Result<TierList, ApiError> {
    TierList::find_by_slug(&state.db, slug)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Database error".into())
        })?
        .ok_or_else(|| ApiError::NotFound("Tier list not found".into()))
}

/// Require authentication and return auth context
pub fn require_auth(headers: &HeaderMap, jwt_secret: &str) -> Result<AuthContext, ApiError> {
    extract_auth_context(headers, jwt_secret)
}

/// Require user to be a tier list admin (global)
pub fn require_tier_list_admin(auth: &AuthContext) -> Result<(), ApiError> {
    if auth.role.is_tier_list_admin() {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "This action requires tier list admin privileges".into(),
        ))
    }
}
