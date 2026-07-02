use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::routes::ok_status;
use crate::app::services::tier_list::find_and_authorize;
use crate::app::services::tier_list::invalidate_detail;
use crate::app::state::AppState;
use crate::app::validation::{PLACEMENT_DESCRIPTION_MAX, validate_opt_length};
use crate::core::auth::permissions::Permission;
use crate::database::models::tier_list::TierPlacement;
use crate::database::queries::tier_lists::add_placement;
use crate::database::queries::tier_lists::get_placements;
use crate::database::queries::tier_lists::get_tiers;
use crate::database::queries::tier_lists::move_placement;
use crate::database::queries::tier_lists::remove_placement;
use crate::database::queries::tier_lists::set_placement_description;

#[derive(Deserialize)]
pub struct AddPlacementRequest {
    pub tier_id: Uuid,
    pub operator_id: String,
    pub sub_order: Option<i16>,
    pub description: Option<String>,
}

pub async fn add(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<AddPlacementRequest>,
) -> Result<Json<TierPlacement>, ApiError> {
    validate_opt_length(
        "placement description",
        body.description.as_deref(),
        PLACEMENT_DESCRIPTION_MAX,
    )?;
    let user_id: Uuid = auth.user_uuid()?;
    find_and_authorize(&state, &slug, user_id, auth.role, Permission::Edit).await?;

    let placement = add_placement(
        &state.db,
        body.tier_id,
        &body.operator_id,
        body.sub_order.unwrap_or(0),
        body.description.as_deref(),
    )
    .await?;
    invalidate_detail(&state, &slug).await;
    Ok(Json(placement))
}

#[derive(Deserialize)]
pub struct UpdateDescriptionRequest {
    pub description: Option<String>,
}

pub async fn update_description(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((slug, operator_id)): Path<(String, String)>,
    Json(body): Json<UpdateDescriptionRequest>,
) -> Result<Json<TierPlacement>, ApiError> {
    validate_opt_length(
        "placement description",
        body.description.as_deref(),
        PLACEMENT_DESCRIPTION_MAX,
    )?;
    let user_id: Uuid = auth.user_uuid()?;
    let list = find_and_authorize(&state, &slug, user_id, auth.role, Permission::Edit).await?;

    let placement = set_placement_description(
        &state.db,
        list.id,
        &operator_id,
        body.description.as_deref(),
    )
    .await?
    .ok_or(ApiError::NotFound)?;
    invalidate_detail(&state, &slug).await;
    Ok(Json(placement))
}

pub async fn remove(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((slug, operator_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let list = find_and_authorize(&state, &slug, user_id, auth.role, Permission::Edit).await?;

    // Find which tier this operator is in, then remove. Idempotent: if the
    // placement is already gone (e.g. cascade-deleted by a prior tier delete in
    // the same save batch), report success rather than NotFound.
    let tiers = get_tiers(&state.db, list.id).await?;
    for tier in &tiers {
        let placements = get_placements(&state.db, tier.id).await?;
        if placements.iter().any(|p| p.operator_id == operator_id) {
            remove_placement(&state.db, tier.id, &operator_id).await?;
            invalidate_detail(&state, &slug).await;
            break;
        }
    }
    Ok(ok_status())
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
    let user_id: Uuid = auth.user_uuid()?;
    let list = find_and_authorize(&state, &slug, user_id, auth.role, Permission::Edit).await?;

    let tiers = get_tiers(&state.db, list.id).await?;
    for tier in &tiers {
        let placements = get_placements(&state.db, tier.id).await?;
        if placements.iter().any(|p| p.operator_id == operator_id) {
            let result = move_placement(
                &state.db,
                tier.id,
                body.new_tier_id,
                &operator_id,
                body.sub_order.unwrap_or(0),
            )
            .await?;
            invalidate_detail(&state, &slug).await;
            return Ok(Json(result));
        }
    }
    Err(ApiError::NotFound)
}
