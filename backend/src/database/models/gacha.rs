use serde::{Deserialize, Serialize};
use sqlx::types::{
    Uuid,
    chrono::{DateTime, Utc},
};
use ts_rs::TS;

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GachaRecord {
    #[ts(type = "number")]
    pub id: i64,
    pub user_id: Uuid,
    pub char_id: String,
    pub pool_id: String,
    pub rarity: i16,
    #[ts(type = "number")]
    pub pull_timestamp: i64,
    pub pool_name: Option<String>,
    pub gacha_type: Option<String>,
    pub batch_index: i16,
    pub created_at: DateTime<Utc>,
}

/// `v_gacha_stats` view
#[derive(TS)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GachaStats {
    pub user_id: Uuid,
    #[ts(type = "number | null")]
    pub total_pulls: Option<i64>,
    #[ts(type = "number | null")]
    pub six_star_count: Option<i64>,
    #[ts(type = "number | null")]
    pub five_star_count: Option<i64>,
    #[ts(type = "number | null")]
    pub four_star_count: Option<i64>,
    #[ts(type = "number | null")]
    pub first_pull: Option<i64>,
    #[ts(type = "number | null")]
    pub last_pull: Option<i64>,
}
