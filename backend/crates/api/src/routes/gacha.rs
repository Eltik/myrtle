use axum::Json;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use myrtle_app::AppContext;
use myrtle_app::services::gacha::stats::{BannerPullStat, GachaEnhancedStats, GlobalGachaStats};
use myrtle_app::services::gacha::{
    self, FetchResult, GachaHistoryEnvelopeDto, GachaRecordEntryDto, GachaRecordsDto,
    GachaSettingsDto, GachaStats, HistoryQuery,
};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extract::{self, Caller, MaybeCaller, Paging};
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/gacha/fetch", post(fetch))
        .route("/gacha/global-stats", get(global_stats))
        .route("/gacha/stats/enhanced", get(enhanced_stats))
        .route("/gacha/stats/per-banner", get(per_banner_stats))
        .route("/gacha/history", get(history))
        .route("/gacha/history/{char_id}", get(history_by_char))
        .route("/gacha/stored-records", get(stored_records))
        .route("/gacha/public-records", get(public_records))
        .route("/gacha/stats", get(stats))
        .route("/gacha/settings", get(get_settings).post(update_settings))
}

async fn fetch(State(ctx): State<AppContext>, caller: Caller) -> ApiResult<Json<FetchResult>> {
    Ok(Json(gacha::fetch_and_store(&ctx, &caller).await?))
}

async fn global_stats(State(ctx): State<AppContext>) -> ApiResult<Json<GlobalGachaStats>> {
    Ok(Json(gacha::global_stats(&ctx).await?))
}

#[derive(Deserialize)]
struct EnhancedStatsParams {
    #[serde(alias = "topN")]
    top_n: Option<u32>,
    #[serde(alias = "includeTiming")]
    include_timing: Option<bool>,
}

async fn enhanced_stats(
    State(ctx): State<AppContext>,
    Query(params): Query<EnhancedStatsParams>,
) -> ApiResult<Json<GachaEnhancedStats>> {
    let include_timing = params.include_timing.unwrap_or(false);
    Ok(Json(
        gacha::enhanced_stats(&ctx, params.top_n, include_timing).await?,
    ))
}

async fn per_banner_stats(State(ctx): State<AppContext>) -> ApiResult<Json<Vec<BannerPullStat>>> {
    Ok(Json(gacha::per_banner_stats(&ctx).await?))
}

#[derive(Deserialize)]
struct HistoryParams {
    rarity: Option<i16>,
    #[serde(rename = "gachaType", alias = "gacha_type", default)]
    gacha_type: Option<String>,
    #[serde(rename = "charId", alias = "char_id", default)]
    char_id: Option<String>,
    #[serde(default)]
    from: Option<i64>,
    #[serde(default)]
    to: Option<i64>,
    #[serde(default)]
    order: Option<String>,
}

async fn history(
    State(ctx): State<AppContext>,
    caller: Caller,
    Query(params): Query<HistoryParams>,
    Paging(page): Paging,
) -> ApiResult<Json<GachaHistoryEnvelopeDto>> {
    let query = HistoryQuery {
        rarity: params.rarity,
        gacha_type: params.gacha_type,
        char_id: params.char_id,
        from: params.from,
        to: params.to,
        newest_first: params.order.as_deref() != Some("asc"),
        limit: page.limit(),
        offset: page.offset(),
    };
    Ok(Json(gacha::history(&ctx, &caller, query).await?))
}

async fn history_by_char(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(char_id): Path<String>,
) -> ApiResult<Json<Vec<GachaRecordEntryDto>>> {
    Ok(Json(
        gacha::history_for_char(&ctx, &caller, &char_id).await?,
    ))
}

async fn stored_records(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<GachaRecordsDto>> {
    Ok(Json(gacha::stored_records(&ctx, &caller).await?))
}

#[derive(Deserialize)]
struct PublicRecordsParams {
    uid: String,
    #[serde(default)]
    pools: Option<String>,
}

async fn public_records(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(params): Query<PublicRecordsParams>,
) -> ApiResult<Json<GachaRecordsDto>> {
    let records =
        gacha::public_records(&ctx, &caller, &params.uid, params.pools.as_deref()).await?;
    Ok(Json(records))
}

async fn stats(State(ctx): State<AppContext>, caller: Caller) -> ApiResult<Json<GachaStats>> {
    Ok(Json(gacha::user_stats(&ctx, &caller).await?))
}

async fn get_settings(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<GachaSettingsDto>> {
    Ok(Json(gacha::settings(&ctx, &caller).await?))
}

#[derive(Deserialize)]
struct UpdateSettingsBody {
    store_records: Option<bool>,
    share_anonymous_stats: Option<bool>,
}

async fn update_settings(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<UpdateSettingsBody>,
) -> ApiResult<Json<GachaSettingsDto>> {
    let settings = gacha::update_settings(
        &ctx,
        &caller,
        body.store_records,
        body.share_anonymous_stats,
    )
    .await?;
    Ok(Json(settings))
}
