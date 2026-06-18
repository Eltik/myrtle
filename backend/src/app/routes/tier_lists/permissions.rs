use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::routes::ok_status;
use crate::app::state::AppState;
use crate::database::models::tier_list::TierListPermission;
use crate::database::queries::tier_lists::get_permissions;
use crate::database::queries::tier_lists::grant_permission;
use crate::database::queries::tier_lists::revoke_permission;

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<Vec<TierListPermission>>, ApiError> {
    if !auth.role.is_tier_list_admin() {
        return Err(ApiError::Forbidden);
    }
    let tier_list = super::load_tier_list(&state, &slug).await?;

    let perms = get_permissions(&state.db, tier_list.id).await?;
    Ok(Json(perms))
}

#[derive(Deserialize)]
pub struct GrantRequest {
    pub user_id: Uuid,
    pub permission: String,
}

pub async fn grant(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<GrantRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !auth.role.is_tier_list_admin() {
        return Err(ApiError::Forbidden);
    }
    let granter_id: Uuid = auth.user_uuid()?;
    let tier_list = super::load_tier_list(&state, &slug).await?;

    grant_permission(
        &state.db,
        tier_list.id,
        body.user_id,
        &body.permission,
        granter_id,
    )
    .await?;
    Ok(ok_status())
}

pub async fn revoke(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((slug, target_user_id, permission)): Path<(String, Uuid, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !auth.role.is_tier_list_admin() {
        return Err(ApiError::Forbidden);
    }
    let tier_list = super::load_tier_list(&state, &slug).await?;

    revoke_permission(&state.db, tier_list.id, target_user_id, &permission).await?;
    Ok(ok_status())
}
