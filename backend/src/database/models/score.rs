use serde::{Deserialize, Serialize};
use sqlx::types::{
    Uuid,
    chrono::{DateTime, Utc},
};
use ts_rs::TS;

/// `v_leaderboard` view
#[derive(TS)]
#[ts(export)]
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
    #[ts(type = "number | null")]
    pub rank_global: Option<i64>, // RANK() returns i64
    #[ts(type = "number | null")]
    pub rank_server: Option<i64>,
    /// Rank change vs. the snapshot baseline for the requested movement interval.
    /// Positive = climbed. `None` when the caller didn't request movement data
    /// or no baseline snapshot exists for the user yet.
    #[sqlx(default)]
    #[ts(type = "number | null")]
    pub rank_delta: Option<i64>,
}

/// `user_scores` table
#[derive(TS)]
#[ts(export)]
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
    /// The base grade's stationing-utilization component (actual vs the
    /// optimizer's best on the built rooms). None on rows graded before the
    /// split was stored.
    pub base_utilization: Option<f64>,
    /// The base grade's infrastructure-completeness component (built rooms vs
    /// the same rooms at max level). None on pre-split rows.
    pub base_infrastructure: Option<f64>,
    pub skin_score: f64,
    pub grade: Option<String>,
    pub calculated_at: DateTime<Utc>,
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LeaderboardMover {
    pub uid: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub avatar_id: Option<String>,
    pub server: String,
    #[ts(type = "number")]
    pub current_rank: i64,
    #[ts(type = "number")]
    pub previous_rank: i64,
    #[ts(type = "number")]
    pub rank_delta: i64, // positive = climbed
    pub current_score: Option<f64>,
    pub score_delta: Option<f64>,
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServerShare {
    pub server: String,
    #[ts(type = "number")]
    pub players: i64,
}

/// One point of a user's leaderboard history - a snapshot they appeared in.
#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ScoreHistoryPoint {
    pub taken_at: DateTime<Utc>,
    pub total_score: Option<f64>,
    pub rank_global: i32,
    pub rank_server: i32,
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize)]
pub struct PlayerStanding {
    pub player: LeaderboardEntry,
    pub neighbors: Vec<LeaderboardEntry>,
    pub percentile: f64, // 0.0 = top, 1.0 = bottom
    #[ts(type = "number | null")]
    pub rank_delta: Option<i64>, // delta vs. the requested interval; None if no baseline
}
