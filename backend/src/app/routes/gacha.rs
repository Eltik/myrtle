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
use crate::app::services::gacha::get_stored_records_in_pools;
use crate::app::services::gacha::update_gacha_settings;
use crate::{
    app::{
        error::ApiError,
        extractors::{
            auth::{AuthUser, MaybeAuthUser},
            pagination::Pagination,
        },
        routes::resolve_public_profile,
        services,
        state::AppState,
    },
    database::{models::gacha::GachaStats, queries::gacha::public_history_enabled},
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
/// again first. A player who turned off "Store gacha history" (Settings >
/// Privacy, `store_gacha`) gets a 409 and the game servers are not called;
/// pulls stored before the switch was turned off are kept.
#[utoipa::path(
    post,
    path = "/gacha/fetch",
    tag = "gacha",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "What the sync found and stored.", body = crate::app::services::gacha::FetchResult),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 409, description = "Gacha history storage is turned off for this account (`store_gacha`), so nothing was fetched or stored.", body = crate::app::error::ErrorBody),
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
        ("top_n" = Option<u32>, Query, description = "How many operators to include: 10, 20 (the default) or 50. Other values round up to the next of these, and anything above 50 is 50. Also accepted as `topN`."),
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
    let top_n = snap_top_n(params.top_n);
    let include_timing = params.include_timing.unwrap_or(false);
    let stats = get_enhanced_stats(&state, top_n, include_timing).await?;
    Ok(Json(stats))
}

/// The sizes `enhanced_stats` serves. Each is its own cache entry and every
/// miss runs two full aggregations, so the set is kept small: 3 sizes x 2
/// timing flags is 6 keys, where clamping to 1..=50 allowed 100.
const TOP_N_SIZES: [u32; 3] = [10, 20, 50];

/// Rounds a requested size up to the next served size; above the largest,
/// the largest. Absent is the default, 20.
fn snap_top_n(requested: Option<u32>) -> u32 {
    let n = requested.unwrap_or(20);
    TOP_N_SIZES
        .into_iter()
        .find(|&size| n <= size)
        .unwrap_or(TOP_N_SIZES[TOP_N_SIZES.len() - 1])
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
    security(("bearer_auth" = [])),
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
    security(("bearer_auth" = [])),
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
    security(("bearer_auth" = [])),
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

#[derive(Deserialize)]
pub struct PublicRecordsParams {
    pub uid: String,
    /// Comma-separated banner (pool) ids; absent or empty is every banner.
    #[serde(default)]
    pub pools: Option<String>,
}

/// The most banners one request may name. A player has pulled on 40 at most
/// (local Postgres, 2026-10-28), so this is room, not a limit anyone meets.
const MAX_POOL_FILTER: usize = 64;

/// The longest pool id accepted. The longest stored is 24 characters.
const MAX_POOL_ID_LEN: usize = 64;

/// `pools` as a list of distinct ids in the order given, `None` for absent or
/// blank (every banner). A pool id is letters, digits and `_`
/// (`LIMITED_EN_40_0_4`); anything else, or more than [`MAX_POOL_FILTER`]
/// ids, is a 400 rather than a silently empty page.
fn parse_pool_filter(raw: Option<&str>) -> Result<Option<Vec<String>>, ApiError> {
    let mut pools: Vec<String> = Vec::new();
    for id in raw.unwrap_or("").split(',').map(str::trim) {
        if id.is_empty() {
            continue;
        }
        if id.len() > MAX_POOL_ID_LEN || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(ApiError::BadRequest(format!(
                "`pools` holds an invalid banner id: {id:?}"
            )));
        }
        if !pools.iter().any(|p| p == id) {
            pools.push(id.to_owned());
        }
    }
    if pools.len() > MAX_POOL_FILTER {
        return Err(ApiError::BadRequest(format!(
            "`pools` names {} banners; at most {MAX_POOL_FILTER} are allowed",
            pools.len()
        )));
    }
    Ok((!pools.is_empty()).then_some(pools))
}

/// A player's stored pulls, grouped as `/gacha/stored-records` groups the
/// caller's own, optionally limited to some banners: the profile's Pulls tab
/// and the links the history page's Share button copies.
///
/// Two gates, the owner passing both: the profile must be public (403
/// otherwise, as for every tab's data), and the owner must have opted in to
/// a public pull history (`public_gacha`, off by default; 403 otherwise).
/// An unknown `uid` is 404. The records carry no account identifier beyond
/// the `uid` the caller already named.
#[utoipa::path(
    get,
    path = "/gacha/public-records",
    tag = "gacha",
    params(
        ("uid" = String, Query, description = "Player to read."),
        ("pools" = Option<String>, Query, description = "Comma-separated banner (pool) ids, such as `LIMITED_EN_40_0_4,SINGLE_EN_40_0_1`, to keep only those banners' pulls. Absent or empty is every banner. At most 64 ids of letters, digits and `_`.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "The player's pulls, newest first, grouped by banner type.", body = crate::app::services::gacha::GachaRecordsDto),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn public_records(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<PublicRecordsParams>,
) -> Result<Json<services::gacha::GachaRecordsDto>, ApiError> {
    let (profile, own) = resolve_public_profile(&state, &auth, &params.uid).await?;
    if !own && !public_history_enabled(&state.db, profile.id).await? {
        return Err(ApiError::Forbidden);
    }
    let pools = parse_pool_filter(params.pools.as_deref())?;
    let records = get_stored_records_in_pools(&state, profile.id, pools.as_deref()).await?;
    Ok(Json(records))
}

/// The caller's own pull statistics: pity, rates and totals.
#[utoipa::path(
    get,
    path = "/gacha/stats",
    operation_id = "gacha_stats",
    tag = "gacha",
    security(("bearer_auth" = [])),
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
    security(("bearer_auth" = [])),
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
    security(("bearer_auth" = [])),
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

#[cfg(test)]
mod tests {
    use super::{MAX_POOL_FILTER, parse_pool_filter, snap_top_n};

    #[test]
    fn pool_filter_keeps_distinct_ids_in_order() {
        assert_eq!(parse_pool_filter(None).unwrap(), None);
        assert_eq!(parse_pool_filter(Some("")).unwrap(), None);
        assert_eq!(parse_pool_filter(Some(" , ,")).unwrap(), None);
        assert_eq!(
            parse_pool_filter(Some(
                "LIMITED_EN_40_0_4, SINGLE_EN_40_0_1,LIMITED_EN_40_0_4"
            ))
            .unwrap(),
            Some(vec![
                "LIMITED_EN_40_0_4".to_owned(),
                "SINGLE_EN_40_0_1".to_owned()
            ])
        );
    }

    #[test]
    fn pool_filter_refuses_what_no_pool_id_looks_like() {
        for bad in ["LIMITED'; --", "a b", "pool%", "x".repeat(65).as_str()] {
            assert!(parse_pool_filter(Some(bad)).is_err(), "{bad}");
        }
        let at_cap = (0..MAX_POOL_FILTER)
            .map(|i| format!("P_{i}"))
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(
            parse_pool_filter(Some(&at_cap)).unwrap().map(|p| p.len()),
            Some(MAX_POOL_FILTER)
        );
        let over = format!("{at_cap},P_X");
        assert!(parse_pool_filter(Some(&over)).is_err());
    }

    #[test]
    fn top_n_rounds_up_to_a_served_size() {
        assert_eq!(snap_top_n(None), 20);
        assert_eq!(snap_top_n(Some(0)), 10);
        assert_eq!(snap_top_n(Some(1)), 10);
        assert_eq!(snap_top_n(Some(10)), 10);
        assert_eq!(snap_top_n(Some(11)), 20);
        assert_eq!(snap_top_n(Some(20)), 20);
        assert_eq!(snap_top_n(Some(21)), 50);
        assert_eq!(snap_top_n(Some(50)), 50);
        assert_eq!(snap_top_n(Some(51)), 50);
        assert_eq!(snap_top_n(Some(u32::MAX)), 50);
    }

    #[test]
    fn every_request_lands_on_one_of_three_sizes() {
        let mut sizes: Vec<u32> = (0..=200).map(|n| snap_top_n(Some(n))).collect();
        sizes.push(snap_top_n(None));
        sizes.sort_unstable();
        sizes.dedup();
        assert_eq!(sizes, [10, 20, 50]);
    }
}
