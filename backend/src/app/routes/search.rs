use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::app::services::search::search_users;
use crate::app::{
    error::ApiError, extractors::pagination::Pagination, services::search::SearchPage,
    state::AppState,
};

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchPage>, ApiError> {
    let q = params.q.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let page = search_users(
        &state,
        q,
        params.pagination.limit(),
        params.pagination.offset(),
    )
    .await?;
    Ok(Json(page))
}
