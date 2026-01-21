use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::models::tier_lists::TierList;

use super::middleware::{can_moderate, get_tier_list_by_slug, require_auth};

#[derive(Deserialize)]
pub struct ModerateRequest {
    /// Reason for the moderation action
    pub reason: String,
}

#[derive(Serialize)]
pub struct ModerateResponse {
    pub success: bool,
    pub message: String,
}

/// POST /admin/tier-lists/{slug}/moderate
/// Soft delete a tier list (admin moderation action)
/// This is for moderators to hide inappropriate content without permanently deleting it
pub async fn moderate_tier_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Json(body): Json<ModerateRequest>,
) -> Result<Json<ModerateResponse>, ApiError> {
    let auth = require_auth(&headers, &state.jwt_secret)?;

    // Require moderator role
    if !can_moderate(&auth) {
        return Err(ApiError::BadRequest(
            "This action requires moderator privileges".into(),
        ));
    }

    let tier_list = get_tier_list_by_slug(&state, &slug).await?;

    // Check if already deleted
    if tier_list.is_deleted {
        return Err(ApiError::BadRequest("Tier list is already deleted".into()));
    }

    // Can only moderate community tier lists
    if !tier_list.is_community() {
        return Err(ApiError::BadRequest(
            "Only community tier lists can be moderated. Use DELETE for official tier lists."
                .into(),
        ));
    }

    // Validate reason is not empty
    if body.reason.trim().is_empty() {
        return Err(ApiError::BadRequest("Moderation reason is required".into()));
    }

    // Soft delete the tier list
    let deleted = TierList::soft_delete(&state.db, tier_list.id, auth.user_id, &body.reason)
        .await
        .map_err(|e| {
            eprintln!("Failed to moderate tier list: {e:?}");
            ApiError::Internal("Failed to moderate tier list".into())
        })?;

    if deleted {
        Ok(Json(ModerateResponse {
            success: true,
            message: format!(
                "Tier list '{}' has been hidden from public view. Reason: {}",
                slug, body.reason
            ),
        }))
    } else {
        Err(ApiError::NotFound("Tier list not found".into()))
    }
}
