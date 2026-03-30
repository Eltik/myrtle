use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    app::{
        error::ApiError,
        extractors::{auth::AuthUser, pagination::Pagination},
        services,
        state::AppState,
    },
    database::models::gacha::{GachaRecord, GachaStats},
};

#[derive(Deserialize)]
pub struct HistoryParams {
    pub rarity: Option<i16>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

pub async fn fetch(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<services::gacha::FetchResult>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let result = services::gacha::fetch_and_store(&state, user_id, &auth.uid).await?;
    Ok(Json(result))
}

pub async fn global_stats(
    State(state): State<AppState>,
) -> Result<Json<services::gacha::GlobalGachaStats>, ApiError> {
    let stats = services::gacha::get_global_stats(&state).await?;
    Ok(Json(stats))
}

pub async fn history(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<HistoryParams>,
) -> Result<Json<Vec<GachaRecord>>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let records = services::gacha::get_history(
        &state,
        user_id,
        params.rarity,
        params.pagination.limit(),
        params.pagination.offset(),
    )
    .await?;
    Ok(Json(records))
}

pub async fn stats(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<GachaStats>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let stats = services::gacha::get_stats(&state, user_id).await?;
    Ok(Json(stats))
}
