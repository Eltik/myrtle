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
use crate::database::models::tier_lists::{CreatePlacement, Tier, TierChangeLog, TierPlacement};

use super::middleware::{check_permission, get_tier_list_by_slug, require_auth};

#[derive(Serialize)]
pub struct PlacementData {
    pub id: String,
    pub tier_id: String,
    pub tier_name: Option<String>,
    pub operator_id: String,
    pub sub_order: i32,
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_rarity: Option<String>,
}

#[derive(Serialize)]
pub struct ListPlacementsResponse {
    pub placements: Vec<PlacementData>,
}

#[derive(Deserialize)]
pub struct AddPlacementRequest {
    pub tier_id: String,
    pub operator_id: String,
    pub sub_order: Option<i32>,
    pub notes: Option<String>,
    pub reason: Option<String>,
}

#[derive(Serialize)]
pub struct AddPlacementResponse {
    pub success: bool,
    pub placement: PlacementData,
}

#[derive(Deserialize)]
pub struct UpdatePlacementRequest {
    pub notes: Option<String>,
    pub sub_order: Option<i32>,
}

#[derive(Serialize)]
pub struct UpdatePlacementResponse {
    pub success: bool,
    pub placement: PlacementData,
}

#[derive(Deserialize)]
pub struct MovePlacementRequest {
    pub new_tier_id: String,
    pub new_sub_order: Option<i32>,
    pub reason: Option<String>,
}

#[derive(Serialize)]
pub struct MovePlacementResponse {
    pub success: bool,
    pub placement: PlacementData,
}

#[derive(Serialize)]
pub struct RemovePlacementResponse {
    pub success: bool,
    pub message: String,
}

/// GET /tier-lists/{slug}/placements
/// List all placements in a tier list
pub async fn list_placements(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<ListPlacementsResponse>, ApiError> {
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    let placements = TierPlacement::find_by_tier_list(&state.db, tier_list.id)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Failed to fetch placements".into())
        })?;

    // Get tier info for each placement
    let tiers = Tier::find_by_tier_list(&state.db, tier_list.id)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Failed to fetch tiers".into())
        })?;

    let tier_map: std::collections::HashMap<Uuid, &Tier> =
        tiers.iter().map(|t| (t.id, t)).collect();

    let placement_data: Vec<PlacementData> = placements
        .into_iter()
        .map(|p| {
            let tier = tier_map.get(&p.tier_id);
            let operator = state.game_data.operators.get(&p.operator_id);

            PlacementData {
                id: p.id.to_string(),
                tier_id: p.tier_id.to_string(),
                tier_name: tier.map(|t| t.name.clone()),
                operator_id: p.operator_id,
                sub_order: p.sub_order,
                notes: p.notes,
                operator_name: operator.map(|o| o.name.clone()),
                operator_rarity: operator.and_then(|o| {
                    serde_json::to_value(&o.rarity)
                        .ok()
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                }),
            }
        })
        .collect();

    Ok(Json(ListPlacementsResponse {
        placements: placement_data,
    }))
}

/// POST /tier-lists/{slug}/placements
/// Add an operator to a tier (requires Edit permission)
pub async fn add_placement(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Json(body): Json<AddPlacementRequest>,
) -> Result<Json<AddPlacementResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    check_permission(&state, &auth, tier_list.id, Permission::Edit).await?;

    let tier_uuid = Uuid::parse_str(&body.tier_id)
        .map_err(|_| ApiError::BadRequest("Invalid tier ID".into()))?;

    // Verify tier belongs to this tier list
    let tier = Tier::find_by_id(&state.db, tier_uuid)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Database error".into())
        })?
        .ok_or_else(|| ApiError::NotFound("Tier not found".into()))?;

    if tier.tier_list_id != tier_list.id {
        return Err(ApiError::BadRequest(
            "Tier does not belong to this tier list".into(),
        ));
    }

    // Check if operator is already in the tier list
    if TierPlacement::find_by_operator(&state.db, tier_list.id, &body.operator_id)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Database error".into())
        })?
        .is_some()
    {
        return Err(ApiError::BadRequest(
            "Operator is already in this tier list".into(),
        ));
    }

    // Validate operator exists in game data
    if !state.game_data.operators.contains_key(&body.operator_id) {
        return Err(ApiError::BadRequest("Unknown operator ID".into()));
    }

    let input = CreatePlacement {
        tier_id: tier_uuid,
        operator_id: body.operator_id.clone(),
        sub_order: body.sub_order,
        notes: body.notes,
    };

    let placement = TierPlacement::create(&state.db, input).await.map_err(|e| {
        eprintln!("Failed to create placement: {e:?}");
        ApiError::Internal("Failed to add placement".into())
    })?;

    // Log the change
    let _ = TierChangeLog::create(
        &state.db,
        tier_list.id,
        "placement_add",
        Some(&body.operator_id),
        None,
        Some(tier_uuid),
        body.reason.as_deref(),
        Some(auth.user_id),
    )
    .await;

    let operator = state.game_data.operators.get(&placement.operator_id);

    Ok(Json(AddPlacementResponse {
        success: true,
        placement: PlacementData {
            id: placement.id.to_string(),
            tier_id: placement.tier_id.to_string(),
            tier_name: Some(tier.name),
            operator_id: placement.operator_id,
            sub_order: placement.sub_order,
            notes: placement.notes,
            operator_name: operator.map(|o| o.name.clone()),
            operator_rarity: operator.and_then(|o| {
                serde_json::to_value(&o.rarity)
                    .ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            }),
        },
    }))
}

/// PUT /tier-lists/{slug}/placements/{placement_id}
/// Update placement notes/order (requires Edit permission)
pub async fn update_placement(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((slug, placement_id)): Path<(String, String)>,
    Json(body): Json<UpdatePlacementRequest>,
) -> Result<Json<UpdatePlacementResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    check_permission(&state, &auth, tier_list.id, Permission::Edit).await?;

    let placement_uuid = Uuid::parse_str(&placement_id)
        .map_err(|_| ApiError::BadRequest("Invalid placement ID".into()))?;

    // Update notes if provided
    let placement = if let Some(notes) = body.notes {
        TierPlacement::update_notes(&state.db, placement_uuid, Some(notes))
            .await
            .map_err(|e| {
                eprintln!("Failed to update placement: {e:?}");
                ApiError::Internal("Failed to update placement".into())
            })?
            .ok_or_else(|| ApiError::NotFound("Placement not found".into()))?
    } else {
        // Just fetch the placement if no update
        return Err(ApiError::BadRequest("No updates provided".into()));
    };

    let operator = state.game_data.operators.get(&placement.operator_id);
    let tier = Tier::find_by_id(&state.db, placement.tier_id)
        .await
        .ok()
        .flatten();

    Ok(Json(UpdatePlacementResponse {
        success: true,
        placement: PlacementData {
            id: placement.id.to_string(),
            tier_id: placement.tier_id.to_string(),
            tier_name: tier.map(|t| t.name),
            operator_id: placement.operator_id,
            sub_order: placement.sub_order,
            notes: placement.notes,
            operator_name: operator.map(|o| o.name.clone()),
            operator_rarity: operator.and_then(|o| {
                serde_json::to_value(&o.rarity)
                    .ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            }),
        },
    }))
}

/// POST /tier-lists/{slug}/placements/{placement_id}/move
/// Move an operator to a different tier (requires Edit permission)
pub async fn move_placement(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((slug, placement_id)): Path<(String, String)>,
    Json(body): Json<MovePlacementRequest>,
) -> Result<Json<MovePlacementResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    check_permission(&state, &auth, tier_list.id, Permission::Edit).await?;

    let placement_uuid = Uuid::parse_str(&placement_id)
        .map_err(|_| ApiError::BadRequest("Invalid placement ID".into()))?;

    let new_tier_uuid = Uuid::parse_str(&body.new_tier_id)
        .map_err(|_| ApiError::BadRequest("Invalid tier ID".into()))?;

    // Verify new tier belongs to this tier list
    let new_tier = Tier::find_by_id(&state.db, new_tier_uuid)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Database error".into())
        })?
        .ok_or_else(|| ApiError::NotFound("Target tier not found".into()))?;

    if new_tier.tier_list_id != tier_list.id {
        return Err(ApiError::BadRequest(
            "Target tier does not belong to this tier list".into(),
        ));
    }

    // Get old tier for logging
    let old_placement = TierPlacement::find_by_tier_list(&state.db, tier_list.id)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Database error".into())
        })?
        .into_iter()
        .find(|p| p.id == placement_uuid);

    let old_tier_id = old_placement.as_ref().map(|p| p.tier_id);

    let placement =
        TierPlacement::move_to_tier(&state.db, placement_uuid, new_tier_uuid, body.new_sub_order)
            .await
            .map_err(|e| {
                eprintln!("Failed to move placement: {e:?}");
                ApiError::Internal("Failed to move placement".into())
            })?
            .ok_or_else(|| ApiError::NotFound("Placement not found".into()))?;

    // Log the change
    let _ = TierChangeLog::create(
        &state.db,
        tier_list.id,
        "placement_move",
        Some(&placement.operator_id),
        old_tier_id,
        Some(new_tier_uuid),
        body.reason.as_deref(),
        Some(auth.user_id),
    )
    .await;

    let operator = state.game_data.operators.get(&placement.operator_id);

    Ok(Json(MovePlacementResponse {
        success: true,
        placement: PlacementData {
            id: placement.id.to_string(),
            tier_id: placement.tier_id.to_string(),
            tier_name: Some(new_tier.name),
            operator_id: placement.operator_id,
            sub_order: placement.sub_order,
            notes: placement.notes,
            operator_name: operator.map(|o| o.name.clone()),
            operator_rarity: operator.and_then(|o| {
                serde_json::to_value(&o.rarity)
                    .ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            }),
        },
    }))
}

/// DELETE /tier-lists/{slug}/placements/{placement_id}
/// Remove an operator from the tier list (requires Edit permission)
pub async fn remove_placement(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((slug, placement_id)): Path<(String, String)>,
) -> Result<Json<RemovePlacementResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    check_permission(&state, &auth, tier_list.id, Permission::Edit).await?;

    let placement_uuid = Uuid::parse_str(&placement_id)
        .map_err(|_| ApiError::BadRequest("Invalid placement ID".into()))?;

    // Get placement info for logging before deletion
    let placement = TierPlacement::find_by_tier_list(&state.db, tier_list.id)
        .await
        .ok()
        .and_then(|placements| placements.into_iter().find(|p| p.id == placement_uuid));

    if let Some(ref p) = placement {
        // Log the change
        let _ = TierChangeLog::create(
            &state.db,
            tier_list.id,
            "placement_remove",
            Some(&p.operator_id),
            Some(p.tier_id),
            None,
            None,
            Some(auth.user_id),
        )
        .await;
    }

    let deleted = TierPlacement::delete(&state.db, placement_uuid)
        .await
        .map_err(|e| {
            eprintln!("Failed to delete placement: {e:?}");
            ApiError::Internal("Failed to remove placement".into())
        })?;

    if deleted {
        Ok(Json(RemovePlacementResponse {
            success: true,
            message: "Placement removed successfully".into(),
        }))
    } else {
        Err(ApiError::NotFound("Placement not found".into()))
    }
}
