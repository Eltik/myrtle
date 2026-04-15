use serde::{Deserialize, Serialize};
use sqlx::types::{
    Uuid,
    chrono::{DateTime, Utc},
};

/// v_leaderboard view
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LeaderboardEntry {
    pub id: Uuid,
    pub uid: String,
    pub nickname: Option<String>,
    pub level: Option<i16>,
    pub avatar_id: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
    pub server: String,
    pub total_score: Option<f64>,
    pub grade: Option<String>,
    pub operator_score: Option<f64>,
    pub stage_score: Option<f64>,
    pub roguelike_score: Option<f64>,
    pub sandbox_score: Option<f64>,
    pub medal_score: Option<f64>,
    pub base_score: Option<f64>,
    pub skin_score: Option<f64>,
    pub rank_global: Option<i64>, // RANK() returns i64
    pub rank_server: Option<i64>,
}

/// user_scores table
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UserScore {
    pub user_id: Uuid,
    pub total_score: f64,
    pub operator_score: f64,
    pub stage_score: f64,
    pub roguelike_score: f64,
    pub sandbox_score: f64,
    pub medal_score: f64,
    pub base_score: f64,
    pub skin_score: f64,
    pub grade: Option<String>,
    pub calculated_at: DateTime<Utc>,
}
