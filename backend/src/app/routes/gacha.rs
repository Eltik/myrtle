use axum::{
    Json,
    extract::{Path, Query, State},
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
    database::models::gacha::GachaStats,
};

#[derive(Deserialize)]
pub struct HistoryParams {
    pub rarity: Option<i16>,
    #[serde(alias = "gacha_type")]
    #[serde(rename = "gachaType", default)]
    pub gacha_type: Option<String>,
    #[serde(alias = "char_id")]
    #[serde(rename = "charId", default)]
    pub char_id: Option<String>,
    #[serde(default)]
    pub from: Option<i64>,
    #[serde(default)]
    pub to: Option<i64>,
    #[serde(default)]
    pub order: Option<String>,
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

#[derive(Deserialize)]
pub struct EnhancedStatsParams {
    #[serde(alias = "topN")]
    pub top_n: Option<u32>,
    #[serde(alias = "includeTiming")]
    pub include_timing: Option<bool>,
}

pub async fn enhanced_stats(
    State(state): State<AppState>,
    Query(params): Query<EnhancedStatsParams>,
) -> Result<Json<services::gacha::GachaEnhancedStats>, ApiError> {
    let top_n = params.top_n.unwrap_or(20).clamp(1, 50);
    let include_timing = params.include_timing.unwrap_or(false);
    let stats = services::gacha::get_enhanced_stats(&state, top_n, include_timing).await?;
    Ok(Json(stats))
}

pub async fn history(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<HistoryParams>,
) -> Result<Json<services::gacha::GachaHistoryEnvelopeDto>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let order_desc = !matches!(params.order.as_deref(), Some("asc"));
    let envelope = services::gacha::get_history_envelope(
        &state,
        user_id,
        params.rarity,
        params.gacha_type,
        params.char_id,
        params.from,
        params.to,
        order_desc,
        params.pagination.limit(),
        params.pagination.offset(),
    )
    .await?;
    Ok(Json(envelope))
}

pub async fn history_by_char(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(char_id): Path<String>,
) -> Result<Json<Vec<services::gacha::GachaRecordEntryDto>>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let rows = services::gacha::get_history_for_char(&state, user_id, &char_id).await?;
    Ok(Json(rows))
}

pub async fn stored_records(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<services::gacha::GachaRecordsDto>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let records = services::gacha::get_stored_records(&state, user_id).await?;
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

pub async fn get_settings(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<services::gacha::GachaSettingsDto>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let settings = services::gacha::get_gacha_settings(&state, user_id).await?;
    Ok(Json(settings))
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateSettingsBody {
    pub store_records: Option<bool>,
    pub share_anonymous_stats: Option<bool>,
}

pub async fn update_settings(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpdateSettingsBody>,
) -> Result<Json<services::gacha::GachaSettingsDto>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let settings = services::gacha::update_gacha_settings(
        &state,
        user_id,
        body.store_records,
        body.share_anonymous_stats,
    )
    .await?;
    Ok(Json(settings))
}
