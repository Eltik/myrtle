use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::state::AppState;
use crate::database::models::tier_list::TierListPermission;
use crate::database::queries::tier_lists as queries;

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<Vec<TierListPermission>>, ApiError> {
    if !auth.role.is_tier_list_admin() {
        return Err(ApiError::Forbidden);
    }
    let tier_list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;

    let perms = queries::get_permissions(&state.db, tier_list.id).await?;
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
    let tier_list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;

    queries::grant_permission(
        &state.db,
        tier_list.id,
        body.user_id,
        &body.permission,
        granter_id,
    )
    .await?;
    Ok(crate::app::routes::ok_status())
}

pub async fn revoke(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((slug, target_user_id, permission)): Path<(String, Uuid, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !auth.role.is_tier_list_admin() {
        return Err(ApiError::Forbidden);
    }
    let tier_list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;

    queries::revoke_permission(&state.db, tier_list.id, target_user_id, &permission).await?;
    Ok(crate::app::routes::ok_status())
}
