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

    /// Store multiple gacha records with deduplication
    pub async fn store_batch(
        pool: &PgPool,
        user_id: Uuid,
        records: &[crate::core::user::gacha::GachaItem],
        gacha_type: GachaType,
    ) -> Result<usize, sqlx::Error> {
        if records.is_empty() {
            return Ok(0);
        }

        let gacha_type_str = gacha_type.as_str();
        let mut inserted = 0;

        for chunk in records.chunks(100) {
            let user_ids: Vec<Uuid> = vec![user_id; chunk.len()];
            let timestamps: Vec<i64> = chunk.iter().map(|r| r.at).collect();
            let char_ids: Vec<&str> = chunk.iter().map(|r| r.char_id.as_str()).collect();
            let pool_ids: Vec<&str> = chunk.iter().map(|r| r.pool_id.as_str()).collect();
            let char_names: Vec<&str> = chunk.iter().map(|r| r.char_name.as_str()).collect();
            let rarities: Vec<i16> = chunk.iter().map(|r| Self::parse_rarity(&r.star)).collect();
            let pool_names: Vec<&str> = chunk.iter().map(|r| r.pool_name.as_str()).collect();
            let gacha_types: Vec<&str> = vec![gacha_type_str; chunk.len()];
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
            .bind(&gacha_types)
            .bind(&type_names)
            .bind(&timestamp_strs)
            .execute(pool)
            .await?;

            inserted += result.rows_affected() as usize;
        }

        Ok(inserted)
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
