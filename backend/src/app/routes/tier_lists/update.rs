use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::authentication::permissions::Permission;
use crate::database::models::tier_lists::{TierList, UpdateTierList};

use super::middleware::{check_permission, get_tier_list_by_slug, require_auth};

#[derive(Deserialize)]
pub struct UpdateTierListRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Serialize)]
pub struct UpdateTierListResponse {
    pub success: bool,
    pub tier_list: TierListData,
}

#[derive(Serialize)]
pub struct TierListData {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct DeleteResponse {
    pub success: bool,
    pub message: String,
}

/// PUT /tier-lists/{slug}
/// Update a tier list (requires Admin permission)
pub async fn update_tier_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Json(body): Json<UpdateTierListRequest>,
) -> Result<Json<UpdateTierListResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    // Check admin permission
    check_permission(&state, &auth, tier_list.id, Permission::Admin).await?;

    let input = UpdateTierList {
        name: body.name,
        description: body.description,
        is_active: body.is_active,
    };

    let updated = TierList::update(&state.db, tier_list.id, input)
        .await
        .map_err(|e| {
            eprintln!("Failed to update tier list: {e:?}");
            ApiError::Internal("Failed to update tier list".into())
        })?
        .ok_or_else(|| ApiError::NotFound("Tier list not found".into()))?;

    Ok(Json(UpdateTierListResponse {
        success: true,
        tier_list: TierListData {
            id: updated.id.to_string(),
            name: updated.name,
            slug: updated.slug,
            description: updated.description,
            is_active: updated.is_active,
            updated_at: updated.updated_at.to_rfc3339(),
        },
    }))
}

/// DELETE /tier-lists/{slug}
/// Delete a tier list (requires Admin permission)
pub async fn delete_tier_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> Result<Json<DeleteResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;
    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    // Check admin permission
    check_permission(&state, &auth, tier_list.id, Permission::Admin).await?;

    let deleted = TierList::delete(&state.db, tier_list.id)
        .await
        .map_err(|e| {
            eprintln!("Failed to delete tier list: {e:?}");
            ApiError::Internal("Failed to delete tier list".into())
        })?;

    if deleted {
        Ok(Json(DeleteResponse {
            success: true,
            message: format!("Tier list '{slug}' deleted successfully"),
        }))
    } else {
        Err(ApiError::NotFound("Tier list not found".into()))
    }
}
