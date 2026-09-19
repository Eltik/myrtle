use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::services::gacha::fetch_and_store;
use crate::app::services::gacha::get_enhanced_stats;
use crate::app::services::gacha::get_gacha_settings;
use crate::app::services::gacha::get_global_stats;
use crate::app::services::gacha::get_history_envelope;
use crate::app::services::gacha::get_history_for_char;
use crate::app::services::gacha::get_per_banner_stats;
use crate::app::services::gacha::get_stats;
use crate::app::services::gacha::get_stored_records;
use crate::app::services::gacha::update_gacha_settings;
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

/// Pull the caller's newest gacha records from the game servers and store them.
///
/// Needs stored game credentials, so a player who has disconnected must log in
/// again first.
#[utoipa::path(
    post,
    path = "/gacha/fetch",
    tag = "gacha",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "What the sync found and stored.", body = crate::app::services::gacha::FetchResult),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn fetch(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<services::gacha::FetchResult>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let result = fetch_and_store(&state, user_id, &auth.uid).await?;
    Ok(Json(result))
}

/// Pull statistics across every player who opted into sharing.
#[utoipa::path(
    get,
    path = "/gacha/global-stats",
    tag = "gacha",
    responses(
        (status = 200, description = "Community-wide pull statistics.", body = crate::app::services::gacha::GlobalGachaStats),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn global_stats(
    State(state): State<AppState>,
) -> Result<Json<services::gacha::GlobalGachaStats>, ApiError> {
    let stats = get_global_stats(&state).await?;
    Ok(Json(stats))
}

#[derive(Deserialize)]
pub struct EnhancedStatsParams {
    #[serde(alias = "topN")]
    pub top_n: Option<u32>,
    #[serde(alias = "includeTiming")]
    pub include_timing: Option<bool>,
}

/// Community pull statistics with per-operator detail.
#[utoipa::path(
    get,
    path = "/gacha/stats/enhanced",
    tag = "gacha",
    params(
        ("top_n" = Option<u32>, Query, description = "How many operators to include. Also accepted as `topN`."),
        ("include_timing" = Option<bool>, Query, description = "Include pull-timing breakdowns. Also accepted as `includeTiming`.")
    ),
    responses(
        (status = 200, description = "Enhanced community statistics.", body = crate::app::services::gacha::GachaEnhancedStats),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn enhanced_stats(
    State(state): State<AppState>,
    Query(params): Query<EnhancedStatsParams>,
) -> Result<Json<services::gacha::GachaEnhancedStats>, ApiError> {
    let top_n = params.top_n.unwrap_or(20).clamp(1, 50);
    let include_timing = params.include_timing.unwrap_or(false);
    let stats = get_enhanced_stats(&state, top_n, include_timing).await?;
    Ok(Json(stats))
}

/// Community pull statistics broken down by banner.
#[utoipa::path(
    get,
    path = "/gacha/stats/per-banner",
    tag = "gacha",
    responses(
        (status = 200, description = "One entry per banner.", body = Vec<crate::app::services::gacha::BannerPullStat>),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn per_banner_stats(
    State(state): State<AppState>,
) -> Result<Json<Vec<services::gacha::BannerPullStat>>, ApiError> {
    let stats = get_per_banner_stats(&state).await?;
    Ok(Json(stats))
}

/// The caller's own pull history, filtered and paged.
#[utoipa::path(
    get,
    path = "/gacha/history",
    tag = "gacha",
    params(
        ("rarity" = Option<i16>, Query, description = "Keep only pulls of this rarity, 0 to 5 (a 6-star is rarity 5)."),
        ("gachaType" = Option<String>, Query, description = "Keep only this banner type. Also accepted as `gacha_type`."),
        ("charId" = Option<String>, Query, description = "Keep only pulls of this operator. Also accepted as `char_id`."),
        ("from" = Option<i64>, Query, description = "Unix seconds; keep pulls at or after this time."),
        ("to" = Option<i64>, Query, description = "Unix seconds; keep pulls at or before this time."),
        ("order" = Option<String>, Query, description = "`asc` or `desc` by pull time."),
        ("limit" = Option<u32>, Query, description = "Page size. Defaults to 20, capped at 100."),
        ("offset" = Option<u32>, Query, description = "Rows to skip. Defaults to 0.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "One page of pulls, plus the totals for the filter.", body = crate::app::services::gacha::GachaHistoryEnvelopeDto),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn history(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<HistoryParams>,
) -> Result<Json<services::gacha::GachaHistoryEnvelopeDto>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let order_desc = !matches!(params.order.as_deref(), Some("asc"));
    let envelope = get_history_envelope(
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

/// Every one of the caller's pulls of a single operator.
#[utoipa::path(
    get,
    path = "/gacha/history/{char_id}",
    tag = "gacha",
    params(
        ("char_id" = String, Path, description = "Operator id.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The caller's pulls of that operator.", body = Vec<crate::app::services::gacha::GachaRecordEntryDto>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn history_by_char(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(char_id): Path<String>,
) -> Result<Json<Vec<services::gacha::GachaRecordEntryDto>>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let rows = get_history_for_char(&state, user_id, &char_id).await?;
    Ok(Json(rows))
}

/// The caller's raw stored pull records.
#[utoipa::path(
    get,
    path = "/gacha/stored-records",
    tag = "gacha",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Stored records.", body = crate::app::services::gacha::GachaRecordsDto),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn stored_records(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<services::gacha::GachaRecordsDto>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let records = get_stored_records(&state, user_id).await?;
    Ok(Json(records))
}

/// The caller's own pull statistics: pity, rates and totals.
#[utoipa::path(
    get,
    path = "/gacha/stats",
    operation_id = "gacha_stats",
    tag = "gacha",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The caller's statistics.", body = GachaStats),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn stats(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<GachaStats>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let stats = get_stats(&state, user_id).await?;
    Ok(Json(stats))
}

/// Whether the caller stores pull records and shares anonymous statistics.
#[utoipa::path(
    get,
    path = "/gacha/settings",
    tag = "gacha",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Current settings.", body = crate::app::services::gacha::GachaSettingsDto),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_settings(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<services::gacha::GachaSettingsDto>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let settings = get_gacha_settings(&state, user_id).await?;
    Ok(Json(settings))
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct UpdateSettingsBody {
    pub store_records: Option<bool>,
    pub share_anonymous_stats: Option<bool>,
}

/// Change the caller's gacha storage and sharing settings.
#[utoipa::path(
    post,
    path = "/gacha/settings",
    operation_id = "gacha_update_settings",
    tag = "gacha",
    request_body = UpdateSettingsBody,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The settings as stored.", body = crate::app::services::gacha::GachaSettingsDto),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn update_settings(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpdateSettingsBody>,
) -> Result<Json<services::gacha::GachaSettingsDto>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let settings = update_gacha_settings(
        &state,
        user_id,
        body.store_records,
        body.share_anonymous_stats,
    )
    .await?;
    Ok(Json(settings))
}
