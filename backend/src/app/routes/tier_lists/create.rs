use axum::{Json, extract::State, http::HeaderMap};
use serde::{Deserialize, Serialize};

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::models::tier_lists::{CreateTierList, TierList};

use super::middleware::{require_auth, require_tier_list_admin};

#[derive(Deserialize)]
pub struct CreateTierListRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct CreateTierListResponse {
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
    pub created_at: String,
}

/// POST /tier-lists
/// Create a new tier list (requires TierListAdmin or SuperAdmin)
pub async fn create_tier_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateTierListRequest>,
) -> Result<Json<CreateTierListResponse>, ApiError> {
    // Authenticate and authorize
    let auth = require_auth(&headers, &state.jwt_secret)?;
    require_tier_list_admin(&auth)?;

    // Validate slug format (alphanumeric and hyphens only)
    if !body.slug.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(ApiError::BadRequest(
            "Slug must contain only alphanumeric characters and hyphens".into(),
        ));
    }

    // Check if slug is already taken
    if TierList::find_by_slug(&state.db, &body.slug)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Database error".into())
        })?
        .is_some()
    {
        return Err(ApiError::BadRequest("Slug is already taken".into()));
    }

    // Create tier list
    let input = CreateTierList {
        name: body.name,
        slug: body.slug,
        description: body.description,
    };

    let tier_list = TierList::create(&state.db, input, Some(auth.user_id))
        .await
        .map_err(|e| {
            eprintln!("Failed to create tier list: {e:?}");
            ApiError::Internal("Failed to create tier list".into())
        })?;

    Ok(Json(CreateTierListResponse {
        success: true,
        tier_list: TierListData {
            id: tier_list.id.to_string(),
            name: tier_list.name,
            slug: tier_list.slug,
            description: tier_list.description,
            is_active: tier_list.is_active,
            created_at: tier_list.created_at.to_rfc3339(),
        },
    }))
}
