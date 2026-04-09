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
    let mut index = 0;

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

        index += page_count as i64;
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

    let total = stats.total_pulls.max(1) as f64;
    let result = GlobalGachaStats {
        total_pulls: stats.total_pulls,
        total_users: stats.total_users,
        six_star_rate: (stats.six_star_count as f64 / total) * 100.0,
        five_star_rate: (stats.five_star_count as f64 / total) * 100.0,
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
