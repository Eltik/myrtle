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

/// Search operators, stages and players in one call.
#[utoipa::path(
    get,
    path = "/search",
    tag = "search",
    params(
        ("q" = Option<String>, Query, description = "Query text. An empty or missing value returns an empty page."),
        ("limit" = Option<u32>, Query, description = "Page size. Defaults to 20, capped at 100."),
        ("offset" = Option<u32>, Query, description = "Rows to skip. Defaults to 0.")
    ),
    responses(
        (status = 200, description = "One page of mixed results.", body = SearchPage),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
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
