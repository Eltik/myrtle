use serde::Serialize;
use sqlx::types::{
    Uuid,
    chrono::{DateTime, Utc},
};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct GachaRecord {
    pub id: i64,
    pub user_id: Uuid,
    pub char_id: String,
    pub pool_id: String,
    pub rarity: i16,
    pub pull_timestamp: i64,
    pub pool_name: Option<String>,
    pub gacha_type: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// v_gacha_stats view
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct GachaStats {
    pub user_id: Uuid,
    pub total_pulls: Option<i64>,
    pub six_star_count: Option<i64>,
    pub five_star_count: Option<i64>,
    pub four_star_count: Option<i64>,
    pub first_pull: Option<i64>,
    pub last_pull: Option<i64>,
}
