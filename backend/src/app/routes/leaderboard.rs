use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::app::{
    error::ApiError,
    extractors::pagination::Pagination,
    services::{self, leaderboard::LeaderboardPage},
    state::AppState,
};

#[derive(Deserialize)]
pub struct LeaderboardParams {
    pub sort: Option<String>,   // defaults to "total_score"
    pub server: Option<String>, // optional server filter
    #[serde(flatten)]
    pub pagination: Pagination,
}

pub async fn leaderboard(
    State(state): State<AppState>,
    Query(params): Query<LeaderboardParams>,
) -> Result<Json<LeaderboardPage>, ApiError> {
    let sort = params.sort.as_deref().unwrap_or("total_score");
    let page = services::leaderboard::get_leaderboard(
        &state,
        sort,
        params.server.as_deref(),
        params.pagination.limit(),
        params.pagination.offset(),
    )
    .await?;
    Ok(Json(page))
}
