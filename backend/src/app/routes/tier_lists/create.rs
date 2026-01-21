use axum::{Json, extract::State, http::HeaderMap};
use serde::{Deserialize, Serialize};

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::models::tier_lists::{CreateTierList, TierList, TierListPermission};

use super::middleware::{check_community_limit, require_any_admin_role, require_auth};

#[derive(Deserialize)]
pub struct CreateTierListRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    #[serde(default = "default_community")]
    pub tier_list_type: String,
}

fn default_community() -> String {
    "community".to_string()
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
    pub tier_list_type: String,
    pub created_at: String,
}

/// POST /tier-lists
/// Create a new tier list
/// - Any authenticated user can create "community" tier lists (max 10)
/// - Only admins can create "official" tier lists
pub async fn create_tier_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateTierListRequest>,
) -> Result<Json<CreateTierListResponse>, ApiError> {
    // Authenticate
    let auth = require_auth(&headers, &state.jwt_secret)?;

    // Validate tier_list_type
    let tier_list_type = body.tier_list_type.to_lowercase();
    if tier_list_type != "community" && tier_list_type != "official" {
        return Err(ApiError::BadRequest(
            "tier_list_type must be 'community' or 'official'".into(),
        ));
    }

    // Authorization based on tier list type
    if tier_list_type == "official" {
        // Only admins can create official tier lists
        require_any_admin_role(&auth)?;
    } else {
        // Any authenticated user can create community tier lists
        // Check the 10 tier list limit for community lists
        check_community_limit(&state, auth.user_id).await?;
    }

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
        tier_list_type: tier_list_type.clone(),
    };

    let tier_list = TierList::create(&state.db, input, Some(auth.user_id))
        .await
        .map_err(|e| {
            eprintln!("Failed to create tier list: {e:?}");
            ApiError::Internal("Failed to create tier list".into())
        })?;

    // Grant Admin permission to the creator for official tier lists
    // (Community tier lists use ownership-based authorization, not permissions)
    if tier_list_type == "official" && !auth.role.is_tier_list_admin() {
        TierListPermission::grant(
            &state.db,
            tier_list.id,
            auth.user_id,
            "admin",
            Some(auth.user_id),
        )
        .await
        .map_err(|e| {
            eprintln!("Failed to grant permission to creator: {e:?}");
            // Don't fail the request, just log the error
        })
        .ok();
    }

    Ok(Json(CreateTierListResponse {
        success: true,
        tier_list: TierListData {
            id: tier_list.id.to_string(),
            name: tier_list.name,
            slug: tier_list.slug,
            description: tier_list.description,
            is_active: tier_list.is_active,
            tier_list_type: tier_list.tier_list_type,
            created_at: tier_list.created_at.to_rfc3339(),
        },
    }))
}
