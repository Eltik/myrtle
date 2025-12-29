use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::authentication::permissions::Permission;
use crate::database::models::tier_lists::TierListPermission;
use crate::database::models::user::User;

use super::middleware::{check_permission, get_tier_list_by_slug, require_auth};

#[derive(Serialize)]
pub struct PermissionData {
    pub id: String,
    pub user_id: String,
    pub user_uid: Option<String>,
    pub permission: String,
    pub granted_by: Option<String>,
    pub granted_at: String,
}

#[derive(Serialize)]
pub struct ListPermissionsResponse {
    pub permissions: Vec<PermissionData>,
}

#[derive(Deserialize)]
pub struct GrantPermissionRequest {
    pub user_id: String,
    pub permission: String,
}

#[derive(Serialize)]
pub struct GrantPermissionResponse {
    pub success: bool,
    pub permission: PermissionData,
}

#[derive(Serialize)]
pub struct RevokePermissionResponse {
    pub success: bool,
    pub message: String,
}

/// GET /tier-lists/{slug}/permissions
/// List all permissions for a tier list (requires Admin permission)
pub async fn list_permissions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> Result<Json<ListPermissionsResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    check_permission(&state, &auth, tier_list.id, Permission::Admin).await?;

    let permissions = TierListPermission::find_all_for_tier_list(&state.db, tier_list.id)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Failed to fetch permissions".into())
        })?;

    // Get user info for each permission
    let mut permission_data = Vec::new();
    for p in permissions {
        let user = User::find_by_id(&state.db, p.user_id).await.ok().flatten();

        permission_data.push(PermissionData {
            id: p.id.to_string(),
            user_id: p.user_id.to_string(),
            user_uid: user.map(|u| u.uid),
            permission: p.permission,
            granted_by: p.granted_by.map(|id| id.to_string()),
            granted_at: p.granted_at.to_rfc3339(),
        });
    }

    Ok(Json(ListPermissionsResponse {
        permissions: permission_data,
    }))
}

/// POST /tier-lists/{slug}/permissions
/// Grant a permission to a user (requires Admin permission)
pub async fn grant_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Json(body): Json<GrantPermissionRequest>,
) -> Result<Json<GrantPermissionResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    check_permission(&state, &auth, tier_list.id, Permission::Admin).await?;

    let user_uuid = Uuid::parse_str(&body.user_id)
        .map_err(|_| ApiError::BadRequest("Invalid user ID".into()))?;

    // Validate permission string
    let _perm: Permission = body
        .permission
        .parse()
        .map_err(|_| ApiError::BadRequest("Invalid permission type".into()))?;

    // Verify user exists
    let user = User::find_by_id(&state.db, user_uuid)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Database error".into())
        })?
        .ok_or_else(|| ApiError::NotFound("User not found".into()))?;

    let permission = TierListPermission::grant(
        &state.db,
        tier_list.id,
        user_uuid,
        &body.permission,
        Some(auth.user_id),
    )
    .await
    .map_err(|e| {
        eprintln!("Failed to grant permission: {e:?}");
        ApiError::Internal("Failed to grant permission".into())
    })?;

    Ok(Json(GrantPermissionResponse {
        success: true,
        permission: PermissionData {
            id: permission.id.to_string(),
            user_id: permission.user_id.to_string(),
            user_uid: Some(user.uid),
            permission: permission.permission,
            granted_by: permission.granted_by.map(|id| id.to_string()),
            granted_at: permission.granted_at.to_rfc3339(),
        },
    }))
}

/// DELETE /tier-lists/{slug}/permissions/{user_id}/{permission}
/// Revoke a permission from a user (requires Admin permission)
pub async fn revoke_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((slug, user_id, permission)): Path<(String, String, String)>,
) -> Result<Json<RevokePermissionResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    check_permission(&state, &auth, tier_list.id, Permission::Admin).await?;

    let user_uuid =
        Uuid::parse_str(&user_id).map_err(|_| ApiError::BadRequest("Invalid user ID".into()))?;

    let revoked = TierListPermission::revoke(&state.db, tier_list.id, user_uuid, &permission)
        .await
        .map_err(|e| {
            eprintln!("Failed to revoke permission: {e:?}");
            ApiError::Internal("Failed to revoke permission".into())
        })?;

    if revoked {
        Ok(Json(RevokePermissionResponse {
            success: true,
            message: format!("Permission '{permission}' revoked successfully"),
        }))
    } else {
        Err(ApiError::NotFound("Permission not found".into()))
    }
}
