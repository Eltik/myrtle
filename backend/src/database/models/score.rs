use serde::{Deserialize, Serialize};
use sqlx::types::{
    Uuid,
    chrono::{DateTime, Utc},
};

/// `v_leaderboard` view
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LeaderboardEntry {
    pub id: Uuid,
    pub uid: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
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
    /// Rank change vs. the snapshot baseline for the requested movement interval.
    /// Positive = climbed. `None` when the caller didn't request movement data
    /// or no baseline snapshot exists for the user yet.
    #[sqlx(default)]
    pub rank_delta: Option<i64>,
}

/// `user_scores` table
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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LeaderboardMover {
    pub uid: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub avatar_id: Option<String>,
    pub server: String,
    pub current_rank: i64,
    pub previous_rank: i64,
    pub rank_delta: i64, // positive = climbed
    pub current_score: Option<f64>,
    pub score_delta: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServerShare {
    pub server: String,
    pub players: i64,
}

#[derive(Debug, Serialize)]
pub struct PlayerStanding {
    pub player: LeaderboardEntry,
    pub neighbors: Vec<LeaderboardEntry>,
    pub percentile: f64,         // 0.0 = top, 1.0 = bottom
    pub rank_delta: Option<i64>, // delta vs. the requested interval; None if no baseline
}
