use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::hypergryph::yostar::AccountPortalSession;
use crate::database::models::gacha::{GachaRecord, GachaStats};
use crate::database::queries::gacha;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct GachaApiResponse {
    pub data: Option<GachaApiData>,
}

#[derive(Deserialize)]
pub struct GachaApiData {
    pub rows: Vec<GachaApiItem>,
    pub count: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GachaApiItem {
    pub char_id: String,
    pub char_name: Option<String>,
    pub star: String,
    pub pool_id: String,
    pub pool_name: Option<String>,
    pub type_name: Option<String>,
    pub at: i64,
}

impl GachaApiItem {
    /// Parse the "star" string ("6", "5", etc.) to i16 rarity
    fn rarity(&self) -> i16 {
        self.star.parse().unwrap_or(3)
    }

    /// Derive gacha type from pool_id prefix
    fn gacha_type(&self) -> &'static str {
        if self.pool_id.starts_with("LIMITED_") {
            "limited"
        } else if self.pool_id.starts_with("LINKAGE_") {
            "linkage"
        } else if self.pool_id.starts_with("CLASSIC_") {
            "classic"
        } else if self.pool_id.starts_with("SINGLE_") {
            "single"
        } else if self.pool_id.starts_with("BOOT_") {
            "boot"
        } else {
            "normal"
        }
    }

    /// Convert to the JSONB shape that sp_insert_gacha_batch expects
    fn to_record_json(&self) -> serde_json::Value {
        serde_json::json!({
            "char_id": self.char_id,
            "pool_id": self.pool_id,
            "rarity": self.rarity(),
            "pull_timestamp": self.at,
            "pool_name": self.pool_name,
            "gacha_type": self.gacha_type(),
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchResult {
    pub total_fetched: usize,
    pub new_records: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalGachaStats {
    pub total_pulls: i64,
    pub total_users: i64,
    pub six_star_rate: f64,
    pub five_star_rate: f64,
}

const GACHA_API_URL: &str = "https://account.yo-star.com/api/game/gachas";
const PAGE_SIZE: i64 = 50;

/// Fetch gacha records from Yostar API and store them
pub async fn fetch_and_store(
    state: &AppState,
    user_id: Uuid,
    uid: &str,
) -> Result<FetchResult, ApiError> {
    let portal_json: Option<String> = state.cache.get(&CacheKey::PortalSession { uid }).await;

    let portal_json = portal_json.ok_or(ApiError::BadRequest(
        "no portal session — login again".into(),
    ))?;

    let portal: AccountPortalSession = serde_json::from_str(&portal_json)
        .map_err(|_| ApiError::BadRequest("invalid portal session".into()))?;

    let mut all_items: Vec<GachaApiItem> = Vec::new();
    let mut index: i64 = 1;

    loop {
        let url = format!(
            "{}?key=ark&index={}&size={}",
            GACHA_API_URL, index, PAGE_SIZE
        );

        let response = state
            .http_client
            .get(&url)
            .header(
                "Cookie",
                format!("YSSID={}; YSSID.sig={}", portal.yssid, portal.yssid_sig),
            )
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e: reqwest::Error| ApiError::Internal(e.into()))?;

        let page: GachaApiResponse = response
            .json()
            .await
            .map_err(|e: reqwest::Error| ApiError::Internal(e.into()))?;

        let Some(data) = page.data else {
            break;
        };

        if data.rows.is_empty() {
            break;
        }

        let page_count = data.rows.len();
        all_items.extend(data.rows);

        if (page_count as i64) < PAGE_SIZE {
            break;
        }

        index += 1;
    }

    let total_fetched = all_items.len();

    if total_fetched == 0 {
        return Ok(FetchResult {
            total_fetched: 0,
            new_records: 0,
        });
    }

    let records_json: Vec<serde_json::Value> =
        all_items.iter().map(|item| item.to_record_json()).collect();

    let records_value =
        serde_json::to_value(&records_json).map_err(|e| ApiError::Internal(e.into()))?;

    gacha::insert_batch(&state.db, user_id, &records_value).await?;

    Ok(FetchResult {
        total_fetched,
        new_records: total_fetched,
    })
}

#[derive(sqlx::FromRow)]
struct GlobalGachaStatsRow {
    total_pulls: i64,
    total_users: i64,
    six_star_count: i64,
    five_star_count: i64,
}

/// Global anonymous stats
pub async fn get_global_stats(state: &AppState) -> Result<GlobalGachaStats, ApiError> {
    let key = CacheKey::GachaGlobalStats;
    if let Some(cached) = state.cache.get(&key).await {
        return Ok(cached);
    }

    let stats = sqlx::query_as::<_, GlobalGachaStatsRow>(
        r#"
        SELECT
            COUNT(*) AS total_pulls,
            COUNT(DISTINCT gr.user_id) AS total_users,
            COUNT(*) FILTER (WHERE rarity = 6) AS six_star_count,
            COUNT(*) FILTER (WHERE rarity = 5) AS five_star_count
        FROM gacha_records gr
        JOIN user_settings us ON us.user_id = gr.user_id
        WHERE us.share_stats = true
        "#,
    )
    .fetch_one(&state.db)
    .await?;

    // Rates are fractions (0.0–1.0). The frontend renders them as percentages
    // by multiplying by 100 at display time.
    let total = stats.total_pulls.max(1) as f64;
    let result = GlobalGachaStats {
        total_pulls: stats.total_pulls,
        total_users: stats.total_users,
        six_star_rate: stats.six_star_count as f64 / total,
        five_star_rate: stats.five_star_count as f64 / total,
    };

    state.cache.set(&key, &result).await;
    Ok(result)
}

// ============================================
// Enhanced global stats (port from old backend)
// ============================================

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CollectiveStats {
    pub total_pulls: i64,
    pub total_users: i64,
    pub total_six_stars: i64,
    pub total_five_stars: i64,
    pub total_four_stars: i64,
    pub total_three_stars: i64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PullRates {
    pub six_star_rate: f64,
    pub five_star_rate: f64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OperatorPopularity {
    pub char_id: String,
    pub char_name: String,
    pub rarity: i16,
    pub pull_count: i64,
    pub percentage: f64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HourlyPullData {
    pub hour: i32,
    pub pull_count: i64,
    pub percentage: f64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DayOfWeekPullData {
    pub day: i32,
    pub day_name: String,
    pub pull_count: i64,
    pub percentage: f64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DatePullData {
    pub date: String,
    pub pull_count: i64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PullTimingData {
    pub by_hour: Vec<HourlyPullData>,
    pub by_day_of_week: Vec<DayOfWeekPullData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by_date: Option<Vec<DatePullData>>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GachaEnhancedStats {
    pub collective_stats: CollectiveStats,
    pub pull_rates: PullRates,
    pub most_common_operators: Vec<OperatorPopularity>,
    pub average_pulls_to_six_star: f64,
    pub average_pulls_to_five_star: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_timing: Option<PullTimingData>,
    pub computed_at: String,
    pub cached: bool,
}

pub async fn get_enhanced_stats(
    state: &AppState,
    top_n: u32,
    include_timing: bool,
) -> Result<GachaEnhancedStats, ApiError> {
    let key = CacheKey::GachaEnhancedStats {
        top_n,
        include_timing,
    };
    if let Some(mut cached) = state.cache.get::<GachaEnhancedStats>(&key).await {
        cached.cached = true;
        return Ok(cached);
    }

    // Collective stats
    #[derive(sqlx::FromRow)]
    struct CollectiveRow {
        total_pulls: i64,
        total_users: i64,
        total_six_stars: i64,
        total_five_stars: i64,
        total_four_stars: i64,
        total_three_stars: i64,
    }

    let collective = sqlx::query_as::<_, CollectiveRow>(
        r#"
        SELECT
            COUNT(*) AS total_pulls,
            COUNT(DISTINCT gr.user_id) AS total_users,
            COUNT(*) FILTER (WHERE rarity = 6) AS total_six_stars,
            COUNT(*) FILTER (WHERE rarity = 5) AS total_five_stars,
            COUNT(*) FILTER (WHERE rarity = 4) AS total_four_stars,
            COUNT(*) FILTER (WHERE rarity = 3) AS total_three_stars
        FROM gacha_records gr
        JOIN user_settings us ON us.user_id = gr.user_id
        WHERE us.share_stats = true
        "#,
    )
    .fetch_one(&state.db)
    .await?;

    let total = collective.total_pulls.max(1) as f64;
    let collective_stats = CollectiveStats {
        total_pulls: collective.total_pulls,
        total_users: collective.total_users,
        total_six_stars: collective.total_six_stars,
        total_five_stars: collective.total_five_stars,
        total_four_stars: collective.total_four_stars,
        total_three_stars: collective.total_three_stars,
    };
    let pull_rates = PullRates {
        six_star_rate: collective.total_six_stars as f64 / total,
        five_star_rate: collective.total_five_stars as f64 / total,
    };

    // Most common operators (top N overall)
    #[derive(sqlx::FromRow)]
    struct OpRow {
        char_id: String,
        rarity: i16,
        pull_count: i64,
    }

    // Top-N **per rarity**, windowed. A single global top-N is dominated by
    // 3-stars (which drop far more often than higher rarities) so the 6/5/4-star
    // buckets would always be empty. Windowing fixes that.
    let op_rows = sqlx::query_as::<_, OpRow>(
        r#"
        SELECT char_id, rarity, pull_count FROM (
            SELECT gr.char_id, gr.rarity, COUNT(*) AS pull_count,
                   ROW_NUMBER() OVER (PARTITION BY gr.rarity ORDER BY COUNT(*) DESC) AS rn
            FROM gacha_records gr
            JOIN user_settings us ON us.user_id = gr.user_id
            WHERE us.share_stats = true
            GROUP BY gr.char_id, gr.rarity
        ) t
        WHERE rn <= $1
        ORDER BY rarity DESC, pull_count DESC
        "#,
    )
    .bind(top_n as i64)
    .fetch_all(&state.db)
    .await?;

    let most_common_operators: Vec<OperatorPopularity> = op_rows
        .into_iter()
        .map(|r| OperatorPopularity {
            char_id: r.char_id,
            char_name: String::new(),
            rarity: r.rarity,
            pull_count: r.pull_count,
            percentage: if collective.total_pulls > 0 {
                r.pull_count as f64 / collective.total_pulls as f64
            } else {
                0.0
            },
        })
        .collect();

    // Average pulls to a 6-star / 5-star (simple estimate: total pulls / count)
    let average_pulls_to_six_star = if collective.total_six_stars > 0 {
        collective.total_pulls as f64 / collective.total_six_stars as f64
    } else {
        0.0
    };
    let average_pulls_to_five_star = if collective.total_five_stars > 0 {
        collective.total_pulls as f64 / collective.total_five_stars as f64
    } else {
        0.0
    };

    // Optional pull timing data
    let pull_timing = if include_timing {
        #[derive(sqlx::FromRow)]
        struct HourRow {
            hour: i32,
            pull_count: i64,
        }
        #[derive(sqlx::FromRow)]
        struct DowRow {
            day: i32,
            pull_count: i64,
        }
        #[derive(sqlx::FromRow)]
        struct DateRow {
            date: String,
            pull_count: i64,
        }

        let hours = sqlx::query_as::<_, HourRow>(
            r#"
            SELECT EXTRACT(HOUR FROM to_timestamp(gr.pull_timestamp))::int AS hour,
                   COUNT(*) AS pull_count
            FROM gacha_records gr
            JOIN user_settings us ON us.user_id = gr.user_id
            WHERE us.share_stats = true
            GROUP BY hour
            ORDER BY hour
            "#,
        )
        .fetch_all(&state.db)
        .await?;

        let dows = sqlx::query_as::<_, DowRow>(
            r#"
            SELECT EXTRACT(DOW FROM to_timestamp(gr.pull_timestamp))::int AS day,
                   COUNT(*) AS pull_count
            FROM gacha_records gr
            JOIN user_settings us ON us.user_id = gr.user_id
            WHERE us.share_stats = true
            GROUP BY day
            ORDER BY day
            "#,
        )
        .fetch_all(&state.db)
        .await?;

        let dates = sqlx::query_as::<_, DateRow>(
            r#"
            SELECT to_char(to_timestamp(gr.pull_timestamp), 'YYYY-MM-DD') AS date,
                   COUNT(*) AS pull_count
            FROM gacha_records gr
            JOIN user_settings us ON us.user_id = gr.user_id
            WHERE us.share_stats = true
            GROUP BY date
            ORDER BY date
            "#,
        )
        .fetch_all(&state.db)
        .await?;

        let day_names = [
            "Sunday",
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
        ];

        let by_hour: Vec<HourlyPullData> = hours
            .into_iter()
            .map(|r| HourlyPullData {
                hour: r.hour,
                pull_count: r.pull_count,
                percentage: if collective.total_pulls > 0 {
                    r.pull_count as f64 / collective.total_pulls as f64
                } else {
                    0.0
                },
            })
            .collect();

        let by_day_of_week: Vec<DayOfWeekPullData> = dows
            .into_iter()
            .map(|r| DayOfWeekPullData {
                day: r.day,
                day_name: day_names
                    .get(r.day as usize)
                    .copied()
                    .unwrap_or("")
                    .to_string(),
                pull_count: r.pull_count,
                percentage: if collective.total_pulls > 0 {
                    r.pull_count as f64 / collective.total_pulls as f64
                } else {
                    0.0
                },
            })
            .collect();

        let by_date: Vec<DatePullData> = dates
            .into_iter()
            .map(|r| DatePullData {
                date: r.date,
                pull_count: r.pull_count,
            })
            .collect();

        Some(PullTimingData {
            by_hour,
            by_day_of_week,
            by_date: Some(by_date),
        })
    } else {
        None
    };

    let result = GachaEnhancedStats {
        collective_stats,
        pull_rates,
        most_common_operators,
        average_pulls_to_six_star,
        average_pulls_to_five_star,
        pull_timing,
        computed_at: chrono::Utc::now().to_rfc3339(),
        cached: false,
    };

    state.cache.set(&key, &result).await;
    Ok(result)
}

pub async fn get_history(
    state: &AppState,
    user_id: Uuid,
    rarity: Option<i16>,
    limit: u32,
    offset: u32,
) -> Result<Vec<GachaRecord>, ApiError> {
    let records =
        gacha::get_history(&state.db, user_id, rarity, limit as i64, offset as i64).await?;
    Ok(records)
}

pub async fn get_stats(state: &AppState, user_id: Uuid) -> Result<GachaStats, ApiError> {
    gacha::get_stats(&state.db, user_id)
        .await?
        .ok_or(ApiError::NotFound)
}

// ============================================
// Port: GachaRecords (grouped) + history envelope + settings
// ============================================

#[derive(Serialize, Clone)]
pub struct GachaRecordEntryDto {
    pub id: String,
    #[serde(rename = "charId")]
    pub char_id: String,
    #[serde(rename = "charName")]
    pub char_name: String,
    pub rarity: i16,
    #[serde(rename = "poolId")]
    pub pool_id: String,
    #[serde(rename = "poolName")]
    pub pool_name: String,
    #[serde(rename = "gachaType")]
    pub gacha_type: String,
    #[serde(rename = "pullTimestamp")]
    pub pull_timestamp: i64,
    #[serde(rename = "pullTimestampStr")]
    pub pull_timestamp_str: Option<String>,
}

impl From<GachaRecord> for GachaRecordEntryDto {
    fn from(r: GachaRecord) -> Self {
        Self {
            id: r.id.to_string(),
            char_id: r.char_id,
            char_name: String::new(),
            rarity: r.rarity,
            pool_id: r.pool_id,
            pool_name: r.pool_name.unwrap_or_default(),
            gacha_type: r.gacha_type.unwrap_or_else(|| "normal".to_owned()),
            pull_timestamp: r.pull_timestamp,
            pull_timestamp_str: None,
        }
    }
}

/// `GachaItem` shape expected by the frontend (camelCase/star-as-string).
#[derive(Serialize, Clone)]
pub struct GachaItemDto {
    #[serde(rename = "charId")]
    pub char_id: String,
    #[serde(rename = "charName")]
    pub char_name: String,
    pub star: String,
    pub color: String,
    #[serde(rename = "poolId")]
    pub pool_id: String,
    #[serde(rename = "poolName")]
    pub pool_name: String,
    #[serde(rename = "typeName")]
    pub type_name: String,
    pub at: i64,
    #[serde(rename = "atStr")]
    pub at_str: String,
}

impl From<&GachaRecord> for GachaItemDto {
    fn from(r: &GachaRecord) -> Self {
        let gt = r.gacha_type.clone().unwrap_or_else(|| "normal".to_owned());
        Self {
            char_id: r.char_id.clone(),
            char_name: String::new(),
            star: r.rarity.to_string(),
            color: String::new(),
            pool_id: r.pool_id.clone(),
            pool_name: r.pool_name.clone().unwrap_or_default(),
            type_name: gt,
            at: r.pull_timestamp,
            at_str: String::new(),
        }
    }
}

#[derive(Serialize)]
pub struct GachaTypeRecordsDto {
    pub gacha_type: &'static str,
    pub records: Vec<GachaItemDto>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct GachaRecordsDto {
    pub limited: GachaTypeRecordsDto,
    pub regular: GachaTypeRecordsDto,
    pub special: GachaTypeRecordsDto,
}

fn classify_gacha_group(gt: &str) -> &'static str {
    match gt {
        "limited" | "linkage" => "limited",
        "single" | "boot" => "special",
        _ => "regular",
    }
}

pub async fn get_stored_records(
    state: &AppState,
    user_id: Uuid,
) -> Result<GachaRecordsDto, ApiError> {
    let rows = gacha::get_all_for_user(&state.db, user_id).await?;

    let mut limited = Vec::new();
    let mut regular = Vec::new();
    let mut special = Vec::new();

    for r in &rows {
        let gt = r.gacha_type.as_deref().unwrap_or("normal");
        let item = GachaItemDto::from(r);
        match classify_gacha_group(gt) {
            "limited" => limited.push(item),
            "special" => special.push(item),
            _ => regular.push(item),
        }
    }

    Ok(GachaRecordsDto {
        limited: GachaTypeRecordsDto {
            gacha_type: "limited",
            total: limited.len(),
            records: limited,
        },
        regular: GachaTypeRecordsDto {
            gacha_type: "regular",
            total: regular.len(),
            records: regular,
        },
        special: GachaTypeRecordsDto {
            gacha_type: "special",
            total: special.len(),
            records: special,
        },
    })
}

#[derive(Serialize)]
pub struct GachaPaginationInfoDto {
    pub limit: u32,
    pub offset: u32,
    pub total: i64,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
}

#[derive(Serialize)]
pub struct HistoryFiltersAppliedDto {
    pub rarity: Option<i16>,
    #[serde(rename = "gachaType")]
    pub gacha_type: Option<String>,
    #[serde(rename = "charId")]
    pub char_id: Option<String>,
    #[serde(rename = "dateRange")]
    pub date_range: Option<DateRangeDto>,
}

#[derive(Serialize)]
pub struct DateRangeDto {
    pub from: Option<i64>,
    pub to: Option<i64>,
}

#[derive(Serialize)]
pub struct GachaHistoryEnvelopeDto {
    pub records: Vec<GachaRecordEntryDto>,
    pub pagination: GachaPaginationInfoDto,
    #[serde(rename = "filtersApplied")]
    pub filters_applied: HistoryFiltersAppliedDto,
}

#[allow(clippy::too_many_arguments)]
pub async fn get_history_envelope(
    state: &AppState,
    user_id: Uuid,
    rarity: Option<i16>,
    gacha_type: Option<String>,
    char_id: Option<String>,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    order_desc: bool,
    limit: u32,
    offset: u32,
) -> Result<GachaHistoryEnvelopeDto, ApiError> {
    let (rows, total) = gacha::get_history_filtered(
        &state.db,
        user_id,
        rarity,
        gacha_type.clone(),
        char_id.clone(),
        from_ts,
        to_ts,
        order_desc,
        limit as i64,
        offset as i64,
    )
    .await?;

    let date_range = if from_ts.is_some() || to_ts.is_some() {
        Some(DateRangeDto {
            from: from_ts,
            to: to_ts,
        })
    } else {
        None
    };

    let records: Vec<GachaRecordEntryDto> = rows.into_iter().map(Into::into).collect();
    let has_more = (offset as i64 + records.len() as i64) < total;

    Ok(GachaHistoryEnvelopeDto {
        records,
        pagination: GachaPaginationInfoDto {
            limit,
            offset,
            total,
            has_more,
        },
        filters_applied: HistoryFiltersAppliedDto {
            rarity,
            gacha_type,
            char_id,
            date_range,
        },
    })
}

pub async fn get_history_for_char(
    state: &AppState,
    user_id: Uuid,
    char_id: &str,
) -> Result<Vec<GachaRecordEntryDto>, ApiError> {
    let rows = gacha::get_by_char_for_user(&state.db, user_id, char_id).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

// ============================================
// Settings
// ============================================

#[derive(Serialize)]
pub struct GachaSettingsDto {
    pub user_id: String,
    pub store_records: bool,
    pub share_anonymous_stats: bool,
    pub total_pulls: i64,
    pub six_star_count: i64,
    pub five_star_count: i64,
    pub last_sync_at: Option<String>,
}

pub async fn get_gacha_settings(
    state: &AppState,
    user_id: Uuid,
) -> Result<GachaSettingsDto, ApiError> {
    let settings = gacha::get_or_create_settings(&state.db, user_id).await?;
    let stats = gacha::get_stats(&state.db, user_id).await?;
    Ok(GachaSettingsDto {
        user_id: user_id.to_string(),
        store_records: settings.store_gacha,
        share_anonymous_stats: settings.share_stats,
        total_pulls: stats.as_ref().and_then(|s| s.total_pulls).unwrap_or(0),
        six_star_count: stats.as_ref().and_then(|s| s.six_star_count).unwrap_or(0),
        five_star_count: stats.as_ref().and_then(|s| s.five_star_count).unwrap_or(0),
        last_sync_at: None,
    })
}

pub async fn update_gacha_settings(
    state: &AppState,
    user_id: Uuid,
    store_records: Option<bool>,
    share_anonymous_stats: Option<bool>,
) -> Result<GachaSettingsDto, ApiError> {
    gacha::update_gacha_flags(&state.db, user_id, store_records, share_anonymous_stats).await?;
    get_gacha_settings(state, user_id).await
}
