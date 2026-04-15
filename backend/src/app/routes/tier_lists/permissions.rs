use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::services;
use crate::app::state::AppState;
use crate::core::auth::permissions::Permission;
use crate::database::models::tier_list::TierListPermission;
use crate::database::queries::tier_lists as queries;

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<Vec<TierListPermission>>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let tier_list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    services::tier_list::check_permission(
        &state,
        &tier_list,
        user_id,
        auth.role,
        Permission::Admin,
    )
    .await?;

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
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let tier_list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    services::tier_list::check_permission(
        &state,
        &tier_list,
        user_id,
        auth.role,
        Permission::Admin,
    )
    .await?;

    queries::grant_permission(
        &state.db,
        tier_list.id,
        body.user_id,
        &body.permission,
        user_id,
    )
    .await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn revoke(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((slug, target_user_id)): Path<(String, Uuid)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let tier_list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    services::tier_list::check_permission(
        &state,
        &tier_list,
        user_id,
        auth.role,
        Permission::Admin,
    )
    .await?;

    queries::revoke_permission(&state.db, tier_list.id, target_user_id).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}
