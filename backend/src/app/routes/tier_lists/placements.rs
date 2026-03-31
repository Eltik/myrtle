use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::services;
use crate::app::state::AppState;
use crate::core::auth::permissions::Permission;
use crate::database::models::tier_list::TierPlacement;
use crate::database::queries::tier_lists as queries;

#[derive(Deserialize)]
pub struct AddPlacementRequest {
    pub tier_id: Uuid,
    pub operator_id: String,
    pub sub_order: Option<i16>,
    pub notes: Option<String>,
}

pub async fn add(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<AddPlacementRequest>,
) -> Result<Json<TierPlacement>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    services::tier_list::check_permission(&state, &list, user_id, auth.role, Permission::Edit)
        .await?;

    let placement = queries::add_placement(
        &state.db,
        body.tier_id,
        &body.operator_id,
        body.sub_order.unwrap_or(0),
        body.notes.as_deref(),
    )
    .await?;
    Ok(Json(placement))
}

pub async fn remove(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((slug, operator_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    services::tier_list::check_permission(&state, &list, user_id, auth.role, Permission::Edit)
        .await?;

    // Find which tier this operator is in, then remove
    let tiers = queries::get_tiers(&state.db, list.id).await?;
    for tier in &tiers {
        let placements = queries::get_placements(&state.db, tier.id).await?;
        if placements.iter().any(|p| p.operator_id == operator_id) {
            queries::remove_placement(&state.db, tier.id, &operator_id).await?;
            return Ok(Json(serde_json::json!({ "status": "ok" })));
        }
    }
    Err(ApiError::NotFound)
}

#[derive(Deserialize)]
pub struct MovePlacementRequest {
    pub new_tier_id: Uuid,
    pub sub_order: Option<i16>,
}

pub async fn move_to(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((slug, operator_id)): Path<(String, String)>,
    Json(body): Json<MovePlacementRequest>,
) -> Result<Json<TierPlacement>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    services::tier_list::check_permission(&state, &list, user_id, auth.role, Permission::Edit)
        .await?;

    // Find current tier
    let tiers = queries::get_tiers(&state.db, list.id).await?;
    for tier in &tiers {
        let placements = queries::get_placements(&state.db, tier.id).await?;
        if placements.iter().any(|p| p.operator_id == operator_id) {
            let result = queries::move_placement(
                &state.db,
                tier.id,
                body.new_tier_id,
                &operator_id,
                body.sub_order.unwrap_or(0),
            )
            .await?;
            return Ok(Json(result));
        }
    }
    Err(ApiError::NotFound)
}
