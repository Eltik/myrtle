use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::models::tier_lists::TierList;

#[derive(Deserialize)]
pub struct ListQuery {
    /// Filter by tier list type: "official", "community", or "all" (default)
    #[serde(rename = "type")]
    pub tier_list_type: Option<String>,
}

#[derive(Serialize)]
pub struct TierListSummary {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub tier_list_type: String,
    pub created_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct ListTierListsResponse {
    pub tier_lists: Vec<TierListSummary>,
}

/// GET /tier-lists
/// List all active tier lists (public)
/// Query params:
/// - type: "official", "community", or "all" (default: "all")
pub async fn list_tier_lists(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListTierListsResponse>, ApiError> {
    // Parse tier_list_type filter
    let type_filter = match query.tier_list_type.as_deref() {
        Some("official") => Some("official"),
        Some("community") => Some("community"),
        Some("all") | None => None,
        Some(other) => {
            return Err(ApiError::BadRequest(format!(
                "Invalid type filter '{}'. Use 'official', 'community', or 'all'",
                other
            )));
        }
    };

    let tier_lists = TierList::find_all_active_by_type(&state.db, type_filter)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Failed to fetch tier lists".into())
        })?;

    let summaries = tier_lists
        .into_iter()
        .map(|tl| TierListSummary {
            id: tl.id.to_string(),
            name: tl.name,
            slug: tl.slug,
            description: tl.description,
            is_active: tl.is_active,
            tier_list_type: tl.tier_list_type,
            created_by: tl.created_by.map(|id| id.to_string()),
            created_at: tl.created_at.to_rfc3339(),
            updated_at: tl.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(ListTierListsResponse {
        tier_lists: summaries,
    }))
}
