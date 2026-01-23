use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::core::user::gacha::GachaType;

// ============================================================================
// Database Models
// ============================================================================

/// Individual gacha pull record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GachaRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub pull_timestamp: i64,
    pub char_id: String,
    pub pool_id: String,
    pub char_name: String,
    pub rarity: i16,
    pub pool_name: String,
    pub gacha_type: String,
    pub type_name: String,
    pub pull_timestamp_str: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// User's gacha settings and sync tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserGachaSettings {
    pub user_id: Uuid,
    pub store_records: bool,
    pub share_anonymous_stats: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_sync_limited_at: Option<i64>,
    pub last_sync_regular_at: Option<i64>,
    pub last_sync_special_at: Option<i64>,
    pub total_pulls: i32,
    pub six_star_count: i32,
    pub five_star_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct UpdateGachaSettings {
    pub store_records: Option<bool>,
    pub share_anonymous_stats: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct GlobalPullRates {
    pub six_star_rate: Option<f64>,
    pub five_star_rate: Option<f64>,
    pub total_pulls: i64,
    pub contributing_users: i64,
}

// ============================================================================
// History Response Types
// ============================================================================

/// Individual pull entry for API responses
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GachaRecordEntry {
    pub id: Uuid,
    pub char_id: String,
    pub char_name: String,
    pub rarity: i16,
    pub pool_id: String,
    pub pool_name: String,
    pub gacha_type: String,
    pub pull_timestamp: i64,
    pub pull_timestamp_str: Option<String>,
}

impl From<GachaRecord> for GachaRecordEntry {
    fn from(r: GachaRecord) -> Self {
        Self {
            id: r.id,
            char_id: r.char_id,
            char_name: r.char_name,
            rarity: r.rarity,
            pool_id: r.pool_id,
            pool_name: r.pool_name,
            gacha_type: r.gacha_type,
            pull_timestamp: r.pull_timestamp,
            pull_timestamp_str: r.pull_timestamp_str,
        }
    }
}

/// Pagination metadata for paginated responses
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationInfo {
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
    pub has_more: bool,
}

/// Date range filter for history queries
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DateRange {
    pub from: Option<i64>,
    pub to: Option<i64>,
}

/// Filters applied to history query
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryFilters {
    pub rarity: Option<i16>,
    pub gacha_type: Option<String>,
    pub char_id: Option<String>,
    pub date_range: Option<DateRange>,
}

/// Response for user pull history endpoint
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryResponse {
    pub records: Vec<GachaRecordEntry>,
    pub pagination: PaginationInfo,
    pub filters_applied: HistoryFilters,
}

// ============================================================================
// Enhanced Statistics Types
// ============================================================================

/// Collective statistics for all consenting users
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectiveStats {
    pub total_pulls: i64,
    pub total_users: i64,
    pub total_six_stars: i64,
    pub total_five_stars: i64,
}

/// Pull rate percentages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRates {
    pub six_star_rate: f64,
    pub five_star_rate: f64,
}

/// Operator popularity statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorPopularity {
    pub char_id: String,
    pub char_name: String,
    pub rarity: i16,
    pub pull_count: i64,
    pub percentage: f64,
}

/// Row type for operator popularity query
#[derive(Debug, FromRow)]
pub struct OperatorPullCountRow {
    pub char_id: String,
    pub char_name: String,
    pub rarity: i16,
    pub pull_count: i64,
}

/// Hourly pull distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HourlyPullData {
    pub hour: i32,
    pub pull_count: i64,
    pub percentage: f64,
}

/// Row type for hourly pull query
#[derive(Debug, FromRow)]
pub struct HourlyPullRow {
    pub hour: i32,
    pub pull_count: i64,
}

/// Day of week pull distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayOfWeekPullData {
    pub day: i32,
    pub day_name: String,
    pub pull_count: i64,
    pub percentage: f64,
}

/// Row type for day of week query
#[derive(Debug, FromRow)]
pub struct DayOfWeekPullRow {
    pub day: i32,
    pub pull_count: i64,
}

/// Pull timing data for graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullTimingData {
    pub by_hour: Vec<HourlyPullData>,
    pub by_day_of_week: Vec<DayOfWeekPullData>,
}

/// Full enhanced statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnhancedStatsResponse {
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

/// Row type for collective stats query
#[derive(Debug, FromRow)]
pub struct CollectiveStatsRow {
    pub total_pulls: i64,
    pub total_users: i64,
    pub total_six_stars: i64,
    pub total_five_stars: i64,
}

// ============================================================================
// GachaRecord Implementations
// ============================================================================

impl GachaRecord {
    /// Parse rarity from star string (e.g., "6" or "TIER_6")
    fn parse_rarity(star: &str) -> i16 {
        star.chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .unwrap_or(3)
    }

    /// Store multiple gacha records with deduplication.
    /// The gacha_type for each record is derived from its pool_id prefix,
    /// ensuring consistency regardless of upstream categorization.
    pub async fn store_batch(
        pool: &PgPool,
        user_id: Uuid,
        records: &[crate::core::user::gacha::GachaItem],
    ) -> Result<usize, sqlx::Error> {
        if records.is_empty() {
            return Ok(0);
        }

        let mut inserted = 0;

        for chunk in records.chunks(100) {
            let user_ids: Vec<Uuid> = vec![user_id; chunk.len()];
            let timestamps: Vec<i64> = chunk.iter().map(|r| r.at).collect();
            let char_ids: Vec<&str> = chunk.iter().map(|r| r.char_id.as_str()).collect();
            let pool_ids: Vec<&str> = chunk.iter().map(|r| r.pool_id.as_str()).collect();
            let char_names: Vec<&str> = chunk.iter().map(|r| r.char_name.as_str()).collect();
            let rarities: Vec<i16> = chunk.iter().map(|r| Self::parse_rarity(&r.star)).collect();
            let pool_names: Vec<&str> = chunk.iter().map(|r| r.pool_name.as_str()).collect();

            let gacha_types: Vec<String> = chunk
                .iter()
                .map(|r| {
                    GachaType::from_pool_id(&r.pool_id)
                        .unwrap_or(GachaType::Limited)
                        .as_str()
                        .to_string()
                })
                .collect();
            let gacha_type_refs: Vec<&str> = gacha_types.iter().map(|s| s.as_str()).collect();
            let type_names: Vec<&str> = chunk.iter().map(|r| r.type_name.as_str()).collect();
            let timestamp_strs: Vec<&str> = chunk.iter().map(|r| r.at_str.as_str()).collect();

            let result = sqlx::query(
                r#"
                INSERT INTO gacha_records
                    (user_id, pull_timestamp, char_id, pool_id, char_name, rarity,
                     pool_name, gacha_type, type_name, pull_timestamp_str)
                SELECT * FROM UNNEST(
                    $1::uuid[], $2::bigint[], $3::varchar[], $4::varchar[],
                    $5::varchar[], $6::smallint[], $7::varchar[], $8::varchar[],
                    $9::varchar[], $10::varchar[]
                )
                ON CONFLICT (user_id, pull_timestamp, char_id, pool_id) DO NOTHING
                "#,
            )
            .bind(&user_ids)
            .bind(&timestamps)
            .bind(&char_ids)
            .bind(&pool_ids)
            .bind(&char_names)
            .bind(&rarities)
            .bind(&pool_names)
            .bind(&gacha_type_refs)
            .bind(&type_names)
            .bind(&timestamp_strs)
            .execute(pool)
            .await?;

            inserted += result.rows_affected() as usize;
        }

        Ok(inserted)
    }

    /// Get user's pull history with pagination and filters
    #[allow(clippy::too_many_arguments)]
    pub async fn get_user_history(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
        rarity: Option<i16>,
        gacha_type: Option<&str>,
        char_id: Option<&str>,
        from_timestamp: Option<i64>,
        to_timestamp: Option<i64>,
        descending: bool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let order = if descending { "DESC" } else { "ASC" };
        let query = format!(
            r#"
            SELECT * FROM gacha_records
            WHERE user_id = $1
              AND ($2::SMALLINT IS NULL OR rarity = $2)
              AND ($3::VARCHAR IS NULL OR gacha_type = $3)
              AND ($4::VARCHAR IS NULL OR char_id = $4)
              AND ($5::BIGINT IS NULL OR pull_timestamp >= $5)
              AND ($6::BIGINT IS NULL OR pull_timestamp <= $6)
            ORDER BY pull_timestamp {}
            LIMIT $7 OFFSET $8
            "#,
            order
        );

        sqlx::query_as::<_, Self>(&query)
            .bind(user_id)
            .bind(rarity)
            .bind(gacha_type)
            .bind(char_id)
            .bind(from_timestamp)
            .bind(to_timestamp)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    }

    /// Count user's pull history for pagination
    pub async fn count_user_history(
        pool: &PgPool,
        user_id: Uuid,
        rarity: Option<i16>,
        gacha_type: Option<&str>,
        char_id: Option<&str>,
        from_timestamp: Option<i64>,
        to_timestamp: Option<i64>,
    ) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM gacha_records
            WHERE user_id = $1
              AND ($2::SMALLINT IS NULL OR rarity = $2)
              AND ($3::VARCHAR IS NULL OR gacha_type = $3)
              AND ($4::VARCHAR IS NULL OR char_id = $4)
              AND ($5::BIGINT IS NULL OR pull_timestamp >= $5)
              AND ($6::BIGINT IS NULL OR pull_timestamp <= $6)
            "#,
        )
        .bind(user_id)
        .bind(rarity)
        .bind(gacha_type)
        .bind(char_id)
        .bind(from_timestamp)
        .bind(to_timestamp)
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }

    /// Get all pulls of a specific operator for a user
    pub async fn get_user_operator_history(
        pool: &PgPool,
        user_id: Uuid,
        char_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM gacha_records
            WHERE user_id = $1 AND char_id = $2
            ORDER BY pull_timestamp DESC
            "#,
        )
        .bind(user_id)
        .bind(char_id)
        .fetch_all(pool)
        .await
    }
}

// ============================================================================
// UserGachaSettings Implementations
// ============================================================================

impl UserGachaSettings {
    /// Find or create settings for a user (with opt-out defaults: store=true, share=true)
    pub async fn find_or_create(pool: &PgPool, user_id: Uuid) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO user_gacha_settings (user_id)
            VALUES ($1)
            ON CONFLICT (user_id) DO UPDATE SET updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
    }

    /// Update user's consent settings
    pub async fn update_settings(
        pool: &PgPool,
        user_id: Uuid,
        input: UpdateGachaSettings,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE user_gacha_settings
            SET store_records = COALESCE($1, store_records),
                share_anonymous_stats = COALESCE($2, share_anonymous_stats),
                updated_at = NOW()
            WHERE user_id = $3
            RETURNING *
            "#,
        )
        .bind(input.store_records)
        .bind(input.share_anonymous_stats)
        .bind(user_id)
        .fetch_one(pool)
        .await
    }

    /// Update sync timestamp for a specific gacha type
    pub async fn update_sync_timestamp(
        pool: &PgPool,
        user_id: Uuid,
        gacha_type: GachaType,
        timestamp: i64,
    ) -> Result<(), sqlx::Error> {
        let query = match gacha_type {
            GachaType::Limited => {
                "UPDATE user_gacha_settings SET last_sync_limited_at = $1, last_sync_at = NOW(), updated_at = NOW() WHERE user_id = $2"
            }
            GachaType::Regular => {
                "UPDATE user_gacha_settings SET last_sync_regular_at = $1, last_sync_at = NOW(), updated_at = NOW() WHERE user_id = $2"
            }
            GachaType::Special => {
                "UPDATE user_gacha_settings SET last_sync_special_at = $1, last_sync_at = NOW(), updated_at = NOW() WHERE user_id = $2"
            }
        };

        sqlx::query(query)
            .bind(timestamp)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Recalculate user's aggregate counts from records
    pub async fn recalculate_counts(pool: &PgPool, user_id: Uuid) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE user_gacha_settings
            SET
                total_pulls = (SELECT COUNT(*) FROM gacha_records WHERE user_id = $1),
                six_star_count = (SELECT COUNT(*) FROM gacha_records WHERE user_id = $1 AND rarity = 6),
                five_star_count = (SELECT COUNT(*) FROM gacha_records WHERE user_id = $1 AND rarity = 5),
                updated_at = NOW()
            WHERE user_id = $1
            RETURNING *
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
    }
}

// ============================================================================
// Global Statistics
// ============================================================================

/// Get global pull rates across all participating users
pub async fn get_global_pull_rates(pool: &PgPool) -> Result<GlobalPullRates, sqlx::Error> {
    sqlx::query_as::<_, GlobalPullRates>(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE rarity = 6)::FLOAT / NULLIF(COUNT(*), 0)::FLOAT as six_star_rate,
            COUNT(*) FILTER (WHERE rarity = 5)::FLOAT / NULLIF(COUNT(*), 0)::FLOAT as five_star_rate,
            COUNT(*) as total_pulls,
            COUNT(DISTINCT gr.user_id) as contributing_users
        FROM gacha_records gr
        JOIN user_gacha_settings ugs ON gr.user_id = ugs.user_id
        WHERE ugs.share_anonymous_stats = true
        "#,
    )
    .fetch_one(pool)
    .await
}

// ============================================================================
// Enhanced Global Statistics
// ============================================================================

/// Get most common operators pulled (only from consenting users)
pub async fn get_most_common_operators(
    pool: &PgPool,
    limit: i32,
) -> Result<Vec<OperatorPullCountRow>, sqlx::Error> {
    sqlx::query_as::<_, OperatorPullCountRow>(
        r#"
        SELECT
            gr.char_id,
            gr.char_name,
            gr.rarity,
            COUNT(*) as pull_count
        FROM gacha_records gr
        JOIN user_gacha_settings ugs ON gr.user_id = ugs.user_id
        WHERE ugs.share_anonymous_stats = true
        GROUP BY gr.char_id, gr.char_name, gr.rarity
        ORDER BY pull_count DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Get collective stats (total pulls, users, 6-stars, 5-stars)
pub async fn get_collective_stats(pool: &PgPool) -> Result<CollectiveStatsRow, sqlx::Error> {
    sqlx::query_as::<_, CollectiveStatsRow>(
        r#"
        SELECT
            COUNT(*) as total_pulls,
            COUNT(DISTINCT gr.user_id) as total_users,
            COUNT(*) FILTER (WHERE rarity = 6) as total_six_stars,
            COUNT(*) FILTER (WHERE rarity = 5) as total_five_stars
        FROM gacha_records gr
        JOIN user_gacha_settings ugs ON gr.user_id = ugs.user_id
        WHERE ugs.share_anonymous_stats = true
        "#,
    )
    .fetch_one(pool)
    .await
}

/// Calculate average pulls to reach a target rarity using pity calculation.
/// This analyzes pull sequences chronologically - each target rarity pull resets the counter.
pub async fn calculate_avg_pulls_to_rarity(
    pool: &PgPool,
    target_rarity: i16,
) -> Result<f64, sqlx::Error> {
    // Use window functions to calculate pity counts
    // For each user, we order pulls by timestamp and count how many pulls
    // occurred between each target rarity pull
    let result: Option<f64> = sqlx::query_scalar(
        r#"
        WITH ordered_pulls AS (
            SELECT
                gr.user_id,
                gr.pull_timestamp,
                gr.rarity,
                ROW_NUMBER() OVER (PARTITION BY gr.user_id ORDER BY gr.pull_timestamp) as pull_num
            FROM gacha_records gr
            JOIN user_gacha_settings ugs ON gr.user_id = ugs.user_id
            WHERE ugs.share_anonymous_stats = true
        ),
        target_pulls AS (
            SELECT
                user_id,
                pull_num,
                LAG(pull_num, 1, 0) OVER (PARTITION BY user_id ORDER BY pull_num) as prev_target_pull
            FROM ordered_pulls
            WHERE rarity = $1
        ),
        pity_counts AS (
            SELECT pull_num - prev_target_pull as pulls_to_target
            FROM target_pulls
            WHERE prev_target_pull > 0  -- Exclude first target pull per user (no previous reference)
        )
        SELECT AVG(pulls_to_target)::FLOAT8 FROM pity_counts
        "#,
    )
    .bind(target_rarity)
    .fetch_one(pool)
    .await?;

    Ok(result.unwrap_or(0.0))
}

/// Get pull timing data by hour of day (0-23)
pub async fn get_pull_timing_by_hour(pool: &PgPool) -> Result<Vec<HourlyPullRow>, sqlx::Error> {
    sqlx::query_as::<_, HourlyPullRow>(
        r#"
        SELECT
            EXTRACT(HOUR FROM to_timestamp(pull_timestamp / 1000))::INT as hour,
            COUNT(*) as pull_count
        FROM gacha_records gr
        JOIN user_gacha_settings ugs ON gr.user_id = ugs.user_id
        WHERE ugs.share_anonymous_stats = true
        GROUP BY hour
        ORDER BY hour
        "#,
    )
    .fetch_all(pool)
    .await
}

/// Get pull timing data by day of week (0-6, Sunday to Saturday)
pub async fn get_pull_timing_by_day(pool: &PgPool) -> Result<Vec<DayOfWeekPullRow>, sqlx::Error> {
    sqlx::query_as::<_, DayOfWeekPullRow>(
        r#"
        SELECT
            EXTRACT(DOW FROM to_timestamp(pull_timestamp / 1000))::INT as day,
            COUNT(*) as pull_count
        FROM gacha_records gr
        JOIN user_gacha_settings ugs ON gr.user_id = ugs.user_id
        WHERE ugs.share_anonymous_stats = true
        GROUP BY day
        ORDER BY day
        "#,
    )
    .fetch_all(pool)
    .await
}
