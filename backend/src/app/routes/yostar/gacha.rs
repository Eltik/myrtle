use crate::{
    app::{error::ApiError, state::AppState},
    core::{
        authentication::jwt,
        user::gacha::{self, GachaRecords, GachaType, GachaTypeRecords},
    },
    database::models::{
        gacha::{
            CollectiveStats, DateRange, DayOfWeekPullData, EnhancedStatsResponse, GachaRecord,
            GachaRecordEntry, GlobalPullRates, HistoryFilters, HistoryResponse, HourlyPullData,
            OperatorPopularity, PaginationInfo, PullRates, PullTimingData, UpdateGachaSettings,
            UserGachaSettings, calculate_avg_pulls_to_rarity, get_collective_stats,
            get_global_pull_rates, get_most_common_operators, get_pull_timing_by_day,
            get_pull_timing_by_hour,
        },
        user::User,
    },
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use redis::AsyncCommands;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GachaQuery {
    pub yssid: Option<String>,
    pub yssid_sig: Option<String>,
    #[serde(rename = "type")]
    pub gacha_type: Option<GachaType>,
    /// JWT token for authenticated storage (optional - if not provided, records won't be stored)
    pub token: Option<String>,
}

/// GET /gacha?yssid={yssid}&yssid_sig={yssid_sig}&token={token}
/// Returns all gacha types (limited, regular, special) in parallel.
/// If a valid JWT token is provided and user hasn't opted out, records are automatically stored.
pub async fn get_all_gacha(
    State(state): State<AppState>,
    Query(params): Query<GachaQuery>,
) -> Result<Json<GachaRecords>, ApiError> {
    let yssid = params
        .yssid
        .ok_or(ApiError::BadRequest("Missing 'yssid' parameter.".into()))?;
    let yssid_sig = params.yssid_sig.ok_or(ApiError::BadRequest(
        "Missing 'yssid_sig' parameter.".into(),
    ))?;

    // Fetch records from Yostar API
    let records = gacha::get_gacha(&state.client, &yssid, &yssid_sig)
        .await
        .map_err(|e| {
            eprintln!("Gacha fetch error: {e:?}");
            ApiError::Internal("Failed to fetch gacha records.".into())
        })?;

    // If token provided, attempt to store records (opt-out model)
    if let Some(token) = &params.token
        && let Ok(claims) = jwt::verify_token(&state.jwt_secret, token)
        && let Ok(Some(user)) = User::find_by_uid(&state.db, &claims.uid).await
    {
        // Get or create settings (defaults to store=true)
        if let Ok(settings) = UserGachaSettings::find_or_create(&state.db, user.id).await {
            // Only store if user hasn't opted out
            if settings.store_records {
                store_records_for_user(&state, user.id, &records, &settings).await;
            }
        }
    }

    Ok(Json(records))
}

/// GET /gacha/type?yssid={yssid}&yssid_sig={yssid_sig}&type={type}&token={token}
/// Returns gacha records for a specific type only.
/// If a valid JWT token is provided and user hasn't opted out, records are automatically stored.
pub async fn get_gacha_by_type(
    State(state): State<AppState>,
    Query(params): Query<GachaQuery>,
) -> Result<Json<GachaTypeRecords>, ApiError> {
    let yssid = params
        .yssid
        .ok_or(ApiError::BadRequest("Missing 'yssid' parameter.".into()))?;
    let yssid_sig = params.yssid_sig.ok_or(ApiError::BadRequest(
        "Missing 'yssid_sig' parameter.".into(),
    ))?;
    let gacha_type = params
        .gacha_type
        .ok_or(ApiError::BadRequest("Missing 'type' parameter.".into()))?;

    // Fetch records from Yostar API
    let records = gacha::get_gacha_by_type(&state.client, &yssid, &yssid_sig, gacha_type)
        .await
        .map_err(|e| {
            eprintln!("Gacha fetch error: {e:?}");
            ApiError::Internal("Failed to fetch gacha records.".into())
        })?;

    // If token provided, attempt to store records (opt-out model)
    if let Some(token) = &params.token
        && let Ok(claims) = jwt::verify_token(&state.jwt_secret, token)
        && let Ok(Some(user)) = User::find_by_uid(&state.db, &claims.uid).await
        && let Ok(settings) = UserGachaSettings::find_or_create(&state.db, user.id).await
        && settings.store_records
    {
        store_type_records_for_user(&state, user.id, &records, gacha_type, &settings).await;
    }

    Ok(Json(records))
}

/// Internal: Store all gacha records for a user
async fn store_records_for_user(
    state: &AppState,
    user_id: uuid::Uuid,
    records: &GachaRecords,
    settings: &UserGachaSettings,
) {
    // Store each type, filtering by last sync timestamp
    for (gacha_type, type_records, last_sync) in [
        (
            GachaType::Limited,
            &records.limited.records,
            settings.last_sync_limited_at,
        ),
        (
            GachaType::Regular,
            &records.regular.records,
            settings.last_sync_regular_at,
        ),
        (
            GachaType::Special,
            &records.special.records,
            settings.last_sync_special_at,
        ),
    ] {
        // Filter to only new records (newer than last sync timestamp)
        let new_records: Vec<_> = type_records
            .iter()
            .filter(|r| last_sync.is_none() || r.at > last_sync.unwrap())
            .cloned()
            .collect();

        if new_records.is_empty() {
            continue;
        }

        // Store records - gacha_type is derived from pool_id in store_batch
        if GachaRecord::store_batch(&state.db, user_id, &new_records).await.is_ok() {
            // Update sync timestamp to prevent re-processing
            if let Some(max_ts) = new_records.iter().map(|r| r.at).max() {
                let _ = UserGachaSettings::update_sync_timestamp(
                    &state.db, user_id, gacha_type, max_ts,
                )
                .await;
            }
        }
    }

    // Recalculate aggregate counts
    let _ = UserGachaSettings::recalculate_counts(&state.db, user_id).await;
}

/// Internal: Store single type gacha records for a user
async fn store_type_records_for_user(
    state: &AppState,
    user_id: uuid::Uuid,
    records: &GachaTypeRecords,
    gacha_type: GachaType,
    settings: &UserGachaSettings,
) {
    let last_sync = match gacha_type {
        GachaType::Limited => settings.last_sync_limited_at,
        GachaType::Regular => settings.last_sync_regular_at,
        GachaType::Special => settings.last_sync_special_at,
    };

    // Filter to only new records
    let new_records: Vec<_> = records
        .records
        .iter()
        .filter(|r| last_sync.is_none() || r.at > last_sync.unwrap())
        .cloned()
        .collect();

    if !new_records.is_empty()
        && let Ok(_) = GachaRecord::store_batch(&state.db, user_id, &new_records).await
        && let Some(max_ts) = new_records.iter().map(|r| r.at).max()
    {
        let _ =
            UserGachaSettings::update_sync_timestamp(&state.db, user_id, gacha_type, max_ts).await;
    }

    let _ = UserGachaSettings::recalculate_counts(&state.db, user_id).await;
}

// ============================================================================
// Settings & Stats Endpoints
// ============================================================================

#[derive(Deserialize)]
pub struct SettingsQuery {
    pub token: String,
}

#[derive(Deserialize)]
pub struct UpdateSettingsRequest {
    pub token: String,
    pub store_records: Option<bool>,
    pub share_anonymous_stats: Option<bool>,
}

/// GET /gacha/settings?token={token}
/// Gets user's gacha storage settings
pub async fn get_gacha_settings(
    State(state): State<AppState>,
    Query(params): Query<SettingsQuery>,
) -> Result<Json<UserGachaSettings>, ApiError> {
    let claims = jwt::verify_token(&state.jwt_secret, &params.token)
        .map_err(|_| ApiError::BadRequest("Invalid token.".into()))?;

    let user = User::find_by_uid(&state.db, &claims.uid)
        .await?
        .ok_or(ApiError::NotFound("User not found.".into()))?;

    let settings = UserGachaSettings::find_or_create(&state.db, user.id).await?;
    Ok(Json(settings))
}

/// POST /gacha/settings
/// Updates user's gacha storage settings (opt-out)
pub async fn update_gacha_settings(
    State(state): State<AppState>,
    Json(body): Json<UpdateSettingsRequest>,
) -> Result<Json<UserGachaSettings>, ApiError> {
    let claims = jwt::verify_token(&state.jwt_secret, &body.token)
        .map_err(|_| ApiError::BadRequest("Invalid token.".into()))?;

    let user = User::find_by_uid(&state.db, &claims.uid)
        .await?
        .ok_or(ApiError::NotFound("User not found.".into()))?;

    // Ensure settings exist
    UserGachaSettings::find_or_create(&state.db, user.id).await?;

    let updated = UserGachaSettings::update_settings(
        &state.db,
        user.id,
        UpdateGachaSettings {
            store_records: body.store_records,
            share_anonymous_stats: body.share_anonymous_stats,
        },
    )
    .await?;

    Ok(Json(updated))
}

/// GET /gacha/stats
/// Gets global anonymous pull rate statistics
pub async fn get_global_stats(
    State(state): State<AppState>,
) -> Result<Json<GlobalPullRates>, ApiError> {
    let rates = get_global_pull_rates(&state.db).await?;
    Ok(Json(rates))
}

// ============================================================================
// User History Endpoints
// ============================================================================

fn default_limit() -> i64 {
    25
}

fn default_order() -> String {
    "desc".to_string()
}

/// Query parameters for pull history endpoint
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub token: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    /// Filter by rarity (1-6)
    pub rarity: Option<i16>,
    /// Filter by gacha type (limited, regular, special)
    pub gacha_type: Option<String>,
    /// Filter by operator ID
    pub char_id: Option<String>,
    /// Filter by date range (start timestamp in milliseconds)
    pub from: Option<i64>,
    /// Filter by date range (end timestamp in milliseconds)
    pub to: Option<i64>,
    /// Sort order: "asc" or "desc" (default: desc)
    #[serde(default = "default_order")]
    pub order: String,
}

/// GET /gacha/history?token={token}&limit={limit}&offset={offset}&...
/// Returns the user's pull history with pagination and filters
pub async fn get_user_history(
    State(state): State<AppState>,
    Query(params): Query<HistoryQuery>,
) -> Result<Json<HistoryResponse>, ApiError> {
    // Validate token and get user
    let claims = jwt::verify_token(&state.jwt_secret, &params.token)
        .map_err(|_| ApiError::BadRequest("Invalid token.".into()))?;

    let user = User::find_by_uid(&state.db, &claims.uid)
        .await?
        .ok_or(ApiError::NotFound("User not found.".into()))?;

    // Validate and clamp parameters
    let limit = params.limit.clamp(1, 100);
    let offset = params.offset.max(0);
    let descending = params.order != "asc";

    // Query database
    let records = GachaRecord::get_user_history(
        &state.db,
        user.id,
        limit,
        offset,
        params.rarity,
        params.gacha_type.as_deref(),
        params.char_id.as_deref(),
        params.from,
        params.to,
        descending,
    )
    .await?;

    let total = GachaRecord::count_user_history(
        &state.db,
        user.id,
        params.rarity,
        params.gacha_type.as_deref(),
        params.char_id.as_deref(),
        params.from,
        params.to,
    )
    .await?;

    // Transform to response entries
    let entries: Vec<GachaRecordEntry> = records.into_iter().map(GachaRecordEntry::from).collect();

    let response = HistoryResponse {
        records: entries,
        pagination: PaginationInfo {
            limit,
            offset,
            total,
            has_more: offset + limit < total,
        },
        filters_applied: HistoryFilters {
            rarity: params.rarity,
            gacha_type: params.gacha_type,
            char_id: params.char_id,
            date_range: if params.from.is_some() || params.to.is_some() {
                Some(DateRange {
                    from: params.from,
                    to: params.to,
                })
            } else {
                None
            },
        },
    };

    Ok(Json(response))
}

/// Query parameters for operator history endpoint
#[derive(Debug, Deserialize)]
pub struct OperatorHistoryQuery {
    pub token: String,
}

/// GET /gacha/history/operator/{char_id}?token={token}
/// Returns all pulls of a specific operator for the authenticated user
pub async fn get_user_operator_history(
    State(state): State<AppState>,
    Path(char_id): Path<String>,
    Query(params): Query<OperatorHistoryQuery>,
) -> Result<Json<Vec<GachaRecordEntry>>, ApiError> {
    let claims = jwt::verify_token(&state.jwt_secret, &params.token)
        .map_err(|_| ApiError::BadRequest("Invalid token.".into()))?;

    let user = User::find_by_uid(&state.db, &claims.uid)
        .await?
        .ok_or(ApiError::NotFound("User not found.".into()))?;

    let records = GachaRecord::get_user_operator_history(&state.db, user.id, &char_id).await?;

    let entries: Vec<GachaRecordEntry> = records.into_iter().map(GachaRecordEntry::from).collect();

    Ok(Json(entries))
}

// ============================================================================
// Enhanced Statistics Endpoints
// ============================================================================

fn default_top_n() -> i32 {
    10
}

/// Query parameters for enhanced stats endpoint
#[derive(Debug, Deserialize)]
pub struct EnhancedStatsQuery {
    /// Number of top operators to return (default: 10, max: 50)
    #[serde(default = "default_top_n")]
    pub top_n: i32,
    /// Include pull timing data (default: false)
    #[serde(default)]
    pub include_timing: bool,
}

/// GET /gacha/stats/enhanced?top_n={top_n}&include_timing={bool}
/// Returns comprehensive global statistics with caching
pub async fn get_enhanced_stats(
    State(state): State<AppState>,
    Query(params): Query<EnhancedStatsQuery>,
) -> Result<Json<EnhancedStatsResponse>, ApiError> {
    let top_n = params.top_n.clamp(1, 50);
    let include_timing = params.include_timing;

    // Build cache key
    let cache_key = format!("gacha:stats:enhanced:{}:{}", top_n, include_timing);

    // Check Redis cache (5 minute TTL for global stats)
    let mut redis = state.redis.clone();
    if let Ok(cached) = redis.get::<_, String>(&cache_key).await
        && let Ok(mut response) = serde_json::from_str::<EnhancedStatsResponse>(&cached)
    {
        response.cached = true;
        return Ok(Json(response));
    }

    // Compute statistics
    let collective = get_collective_stats(&state.db).await.map_err(|e| {
        eprintln!("Failed to get collective stats: {e:?}");
        ApiError::Internal("Failed to compute statistics".into())
    })?;

    let operators = get_most_common_operators(&state.db, top_n)
        .await
        .map_err(|e| {
            eprintln!("Failed to get operator popularity: {e:?}");
            ApiError::Internal("Failed to compute statistics".into())
        })?;

    let avg_to_six = calculate_avg_pulls_to_rarity(&state.db, 6)
        .await
        .unwrap_or(0.0);
    let avg_to_five = calculate_avg_pulls_to_rarity(&state.db, 5)
        .await
        .unwrap_or(0.0);

    // Calculate operator percentages
    let total_pulls_f64 = collective.total_pulls as f64;
    let operator_popularity: Vec<OperatorPopularity> = operators
        .into_iter()
        .map(|op| OperatorPopularity {
            char_id: op.char_id,
            char_name: op.char_name,
            rarity: op.rarity,
            pull_count: op.pull_count,
            percentage: if total_pulls_f64 > 0.0 {
                (op.pull_count as f64 / total_pulls_f64) * 100.0
            } else {
                0.0
            },
        })
        .collect();

    // Optionally compute timing data
    let pull_timing = if include_timing {
        let hourly = get_pull_timing_by_hour(&state.db).await.unwrap_or_default();
        let daily = get_pull_timing_by_day(&state.db).await.unwrap_or_default();

        let total_hourly: i64 = hourly.iter().map(|h| h.pull_count).sum();
        let total_daily: i64 = daily.iter().map(|d| d.pull_count).sum();

        let day_names = [
            "Sunday",
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
        ];

        Some(PullTimingData {
            by_hour: hourly
                .into_iter()
                .map(|h| HourlyPullData {
                    hour: h.hour,
                    pull_count: h.pull_count,
                    percentage: if total_hourly > 0 {
                        (h.pull_count as f64 / total_hourly as f64) * 100.0
                    } else {
                        0.0
                    },
                })
                .collect(),
            by_day_of_week: daily
                .into_iter()
                .map(|d| DayOfWeekPullData {
                    day: d.day,
                    day_name: day_names
                        .get(d.day as usize)
                        .unwrap_or(&"Unknown")
                        .to_string(),
                    pull_count: d.pull_count,
                    percentage: if total_daily > 0 {
                        (d.pull_count as f64 / total_daily as f64) * 100.0
                    } else {
                        0.0
                    },
                })
                .collect(),
        })
    } else {
        None
    };

    let six_star_rate = if collective.total_pulls > 0 {
        (collective.total_six_stars as f64 / collective.total_pulls as f64) * 100.0
    } else {
        0.0
    };

    let five_star_rate = if collective.total_pulls > 0 {
        (collective.total_five_stars as f64 / collective.total_pulls as f64) * 100.0
    } else {
        0.0
    };

    let response = EnhancedStatsResponse {
        collective_stats: CollectiveStats {
            total_pulls: collective.total_pulls,
            total_users: collective.total_users,
            total_six_stars: collective.total_six_stars,
            total_five_stars: collective.total_five_stars,
        },
        pull_rates: PullRates {
            six_star_rate,
            five_star_rate,
        },
        most_common_operators: operator_popularity,
        average_pulls_to_six_star: avg_to_six,
        average_pulls_to_five_star: avg_to_five,
        pull_timing,
        computed_at: chrono::Utc::now().to_rfc3339(),
        cached: false,
    };

    // Cache for 5 minutes (300 seconds)
    if let Ok(json) = serde_json::to_string(&response) {
        let _: Result<(), _> = redis.set_ex::<_, _, ()>(&cache_key, &json, 300).await;
    }

    Ok(Json(response))
}
