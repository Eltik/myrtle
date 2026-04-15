use serde::{Deserialize, Serialize};
use sqlx::types::{
    Uuid,
    chrono::{DateTime, Utc},
};

/// v_user_profile view
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserProfile {
    pub id: Uuid,
    pub uid: String,
    pub nickname: Option<String>,
    pub level: Option<i16>,
    pub avatar_id: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
    pub resume_id: Option<String>,
    pub role: String,
    pub server: String, // FROM servers.code
    // Scores (LEFT JOIN)
    pub total_score: Option<f64>,
    pub grade: Option<String>,
    // Settings (LEFT JOIN)
    pub public_profile: Option<bool>,
    pub store_gacha: Option<bool>,
    pub share_stats: Option<bool>,
    // Status (LEFT JOIN)
    pub exp: Option<i32>,
    pub orundum: Option<i32>,
    pub lmd: Option<i32>,
    pub sanity: Option<i16>,
    pub max_sanity: Option<i16>,
    pub gacha_tickets: Option<i32>,
    pub ten_pull_tickets: Option<i32>,
    pub monthly_sub_end: Option<i64>,
    pub register_ts: Option<i64>,
    pub last_online_ts: Option<i64>,
    pub resume: Option<String>,
    pub friend_num_limit: Option<i16>,
    // Counts
    pub operator_count: Option<i64>,
    pub item_count: Option<i64>,
    pub skin_count: Option<i64>,
}

/// users table
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub uid: String,
    pub server_id: i16,
    pub nickname: Option<String>,
    pub level: Option<i16>,
    pub role: String,
    pub avatar_id: Option<String>,
    pub resume_id: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// user_settings table
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UserSettings {
    pub user_id: Uuid,
    pub public_profile: bool,
    pub store_gacha: bool,
    pub share_stats: bool,
    pub updated_at: DateTime<Utc>,
}
