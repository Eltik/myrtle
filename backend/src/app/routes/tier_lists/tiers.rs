use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::services;
use crate::app::state::AppState;
use crate::core::auth::permissions::Permission;
use crate::database::models::tier_list::Tier;
use crate::database::queries::tier_lists as queries;

#[derive(Deserialize)]
pub struct CreateTierRequest {
    pub name: String,
    pub display_order: i16,
    pub color: Option<String>,
    pub description: Option<String>,
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<CreateTierRequest>,
) -> Result<Json<Tier>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    services::tier_list::check_permission(&state, &list, user_id, auth.role, Permission::Edit)
        .await?;

    let tier = queries::create_tier(
        &state.db,
        list.id,
        &body.name,
        body.display_order,
        body.color.as_deref(),
        body.description.as_deref(),
    )
    .await?;
    Ok(Json(tier))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((slug, tier_id)): Path<(String, Uuid)>,
    Json(body): Json<CreateTierRequest>,
) -> Result<Json<Tier>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    services::tier_list::check_permission(&state, &list, user_id, auth.role, Permission::Edit)
        .await?;

    let tier = queries::update_tier(
        &state.db,
        tier_id,
        &body.name,
        body.display_order,
        body.color.as_deref(),
        body.description.as_deref(),
    )
    .await?;
    Ok(Json(tier))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((slug, tier_id)): Path<(String, Uuid)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    services::tier_list::check_permission(&state, &list, user_id, auth.role, Permission::Admin)
        .await?;

    queries::delete_tier(&state.db, tier_id).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}
