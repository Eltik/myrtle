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
use crate::database::models::tier_lists::{CreateTier, Tier, TierChangeLog};

use super::middleware::{check_tier_list_permission, get_tier_list_by_slug, require_auth};

#[derive(Serialize)]
pub struct TierData {
    pub id: String,
    pub name: String,
    pub display_order: i32,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct ListTiersResponse {
    pub tiers: Vec<TierData>,
}

#[derive(Deserialize)]
pub struct CreateTierRequest {
    pub name: String,
    pub display_order: i32,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct CreateTierResponse {
    pub success: bool,
    pub tier: TierData,
}

#[derive(Deserialize)]
pub struct UpdateTierRequest {
    pub name: Option<String>,
    pub display_order: Option<i32>,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct UpdateTierResponse {
    pub success: bool,
    pub tier: TierData,
}

#[derive(Serialize)]
pub struct DeleteTierResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize)]
pub struct ReorderTiersRequest {
    pub order: Vec<TierOrder>,
}

#[derive(Deserialize)]
pub struct TierOrder {
    pub tier_id: String,
    pub display_order: i32,
}

#[derive(Serialize)]
pub struct ReorderTiersResponse {
    pub success: bool,
}

/// GET /tier-lists/{slug}/tiers
/// List all tiers in a tier list
pub async fn list_tiers(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<ListTiersResponse>, ApiError> {
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    // Check if tier list is deleted
    if tier_list.is_deleted {
        return Err(ApiError::NotFound("Tier list not found".into()));
    }

    let tiers = Tier::find_by_tier_list(&state.db, tier_list.id)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Failed to fetch tiers".into())
        })?;

    let tier_data: Vec<TierData> = tiers
        .into_iter()
        .map(|t| TierData {
            id: t.id.to_string(),
            name: t.name,
            display_order: t.display_order,
            color: t.color,
            description: t.description,
        })
        .collect();

    Ok(Json(ListTiersResponse { tiers: tier_data }))
}

/// POST /tier-lists/{slug}/tiers
/// Create a new tier (requires Edit permission)
pub async fn create_tier(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Json(body): Json<CreateTierRequest>,
) -> Result<Json<CreateTierResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    // Check if tier list is deleted
    if tier_list.is_deleted {
        return Err(ApiError::NotFound("Tier list not found".into()));
    }

    check_tier_list_permission(&state, &auth, &tier_list, Permission::Edit).await?;

    let input = CreateTier {
        name: body.name,
        display_order: body.display_order,
        color: body.color,
        description: body.description,
    };

    let tier = Tier::create(&state.db, tier_list.id, input)
        .await
        .map_err(|e| {
            eprintln!("Failed to create tier: {e:?}");
            ApiError::Internal("Failed to create tier".into())
        })?;

    // Log the change
    let _ = TierChangeLog::create(
        &state.db,
        tier_list.id,
        "tier_add",
        None,
        None,
        Some(tier.id),
        None,
        Some(auth.user_id),
    )
    .await;

    Ok(Json(CreateTierResponse {
        success: true,
        tier: TierData {
            id: tier.id.to_string(),
            name: tier.name,
            display_order: tier.display_order,
            color: tier.color,
            description: tier.description,
        },
    }))
}

/// PUT /tier-lists/{slug}/tiers/{tier_id}
/// Update a tier (requires Edit permission)
pub async fn update_tier(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((slug, tier_id)): Path<(String, String)>,
    Json(body): Json<UpdateTierRequest>,
) -> Result<Json<UpdateTierResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    // Check if tier list is deleted
    if tier_list.is_deleted {
        return Err(ApiError::NotFound("Tier list not found".into()));
    }

    check_tier_list_permission(&state, &auth, &tier_list, Permission::Edit).await?;

    let tier_uuid =
        Uuid::parse_str(&tier_id).map_err(|_| ApiError::BadRequest("Invalid tier ID".into()))?;

    let tier = Tier::update(
        &state.db,
        tier_uuid,
        body.name,
        body.display_order,
        body.color,
        body.description,
    )
    .await
    .map_err(|e| {
        eprintln!("Failed to update tier: {e:?}");
        ApiError::Internal("Failed to update tier".into())
    })?
    .ok_or_else(|| ApiError::NotFound("Tier not found".into()))?;

    Ok(Json(UpdateTierResponse {
        success: true,
        tier: TierData {
            id: tier.id.to_string(),
            name: tier.name,
            display_order: tier.display_order,
            color: tier.color,
            description: tier.description,
        },
    }))
}

/// DELETE /tier-lists/{slug}/tiers/{tier_id}
/// Delete a tier (requires Edit permission)
pub async fn delete_tier(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((slug, tier_id)): Path<(String, String)>,
) -> Result<Json<DeleteTierResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    // Check if tier list is deleted
    if tier_list.is_deleted {
        return Err(ApiError::NotFound("Tier list not found".into()));
    }

    check_tier_list_permission(&state, &auth, &tier_list, Permission::Edit).await?;

    let tier_uuid =
        Uuid::parse_str(&tier_id).map_err(|_| ApiError::BadRequest("Invalid tier ID".into()))?;

    // Log the change before deletion
    let _ = TierChangeLog::create(
        &state.db,
        tier_list.id,
        "tier_delete",
        None,
        Some(tier_uuid),
        None,
        None,
        Some(auth.user_id),
    )
    .await;

    let deleted = Tier::delete(&state.db, tier_uuid).await.map_err(|e| {
        eprintln!("Failed to delete tier: {e:?}");
        ApiError::Internal("Failed to delete tier".into())
    })?;

    if deleted {
        Ok(Json(DeleteTierResponse {
            success: true,
            message: "Tier deleted successfully".into(),
        }))
    } else {
        Err(ApiError::NotFound("Tier not found".into()))
    }
}

/// POST /tier-lists/{slug}/tiers/reorder
/// Reorder tiers (requires Edit permission)
pub async fn reorder_tiers(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Json(body): Json<ReorderTiersRequest>,
) -> Result<Json<ReorderTiersResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    // Check if tier list is deleted
    if tier_list.is_deleted {
        return Err(ApiError::NotFound("Tier list not found".into()));
    }

    check_tier_list_permission(&state, &auth, &tier_list, Permission::Edit).await?;

    let tier_orders: Result<Vec<(Uuid, i32)>, _> = body
        .order
        .into_iter()
        .map(|o| Uuid::parse_str(&o.tier_id).map(|id| (id, o.display_order)))
        .collect();

    let tier_orders = tier_orders.map_err(|_| ApiError::BadRequest("Invalid tier ID".into()))?;

    Tier::reorder(&state.db, tier_list.id, tier_orders)
        .await
        .map_err(|e| {
            eprintln!("Failed to reorder tiers: {e:?}");
            ApiError::Internal("Failed to reorder tiers".into())
        })?;

    Ok(Json(ReorderTiersResponse { success: true }))
}
