use axum::{Json, extract::State, http::HeaderMap};
use serde::Serialize;

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::models::tier_lists::TierList;

use super::middleware::require_auth;

#[derive(Serialize)]
pub struct MyTierListSummary {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub tier_list_type: String,
    pub is_deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct ListMyTierListsResponse {
    pub tier_lists: Vec<MyTierListSummary>,
    pub count: usize,
    pub limit: i64,
}

/// GET /tier-lists/mine
/// List current user's community tier lists (requires authentication)
/// Includes soft-deleted lists so users can see their deleted content
pub async fn list_mine(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ListMyTierListsResponse>, ApiError> {
    // Authenticate
    let auth = require_auth(&headers, &state.jwt_secret)?;

    let tier_lists = TierList::find_community_by_owner(&state.db, auth.user_id)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Failed to fetch tier lists".into())
        })?;

    let count = tier_lists.len();

    let summaries = tier_lists
        .into_iter()
        .map(|tl| MyTierListSummary {
            id: tl.id.to_string(),
            name: tl.name,
            slug: tl.slug,
            description: tl.description,
            is_active: tl.is_active,
            tier_list_type: tl.tier_list_type,
            is_deleted: tl.is_deleted,
            created_at: tl.created_at.to_rfc3339(),
            updated_at: tl.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(ListMyTierListsResponse {
        tier_lists: summaries,
        count,
        limit: 10, // Maximum allowed community tier lists
    }))
}
