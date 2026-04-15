use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::{
    app::{error::ApiError, extractors::pagination::Pagination, services, state::AppState},
    database::models::user::UserProfile,
};

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
    #[serde(flatten)]
    pub pagination: Pagination,
}

pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<UserProfile>>, ApiError> {
    let results = services::search::search_users(
        &state,
        &params.q,
        params.pagination.limit(),
        params.pagination.offset(),
    )
    .await?;
    Ok(Json(results))
}
