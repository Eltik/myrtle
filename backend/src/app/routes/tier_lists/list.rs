use axum::{Json, extract::State};
use serde::Serialize;

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::models::tier_lists::TierList;

#[derive(Serialize)]
pub struct TierListSummary {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct ListTierListsResponse {
    pub tier_lists: Vec<TierListSummary>,
}

/// GET /tier-lists
/// List all active tier lists (public)
pub async fn list_tier_lists(
    State(state): State<AppState>,
) -> Result<Json<ListTierListsResponse>, ApiError> {
    let tier_lists = TierList::find_all_active(&state.db).await.map_err(|e| {
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
            created_at: tl.created_at.to_rfc3339(),
            updated_at: tl.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(ListTierListsResponse {
        tier_lists: summaries,
    }))
}
