//! Type definitions for leaderboard endpoint
//!
//! Provides query parameters for sorting/filtering and response types
//! for the leaderboard API.

use serde::{Deserialize, Serialize};

/// Activity metrics breakdown for grade calculation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityMetricsResponse {
    /// Days since last login
    pub days_since_login: i64,
    /// Login recency score (0-100)
    pub login_recency_score: f32,
    /// Login frequency score (0-100)
    pub login_frequency_score: f32,
    /// Mission completion consistency score (0-100)
    pub consistency_score: f32,
    /// Combined activity score (0-100)
    pub total_activity_score: f32,
}

/// Engagement metrics breakdown for grade calculation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngagementMetricsResponse {
    /// Content variety score (0-100)
    pub content_variety_score: f32,
    /// Roguelike engagement depth (0-100)
    pub roguelike_depth_score: f32,
    /// Stage completion diversity (0-100)
    pub stage_diversity_score: f32,
    /// Account progression depth (0-100)
    pub progression_depth_score: f32,
    /// Combined engagement score (0-100)
    pub total_engagement_score: f32,
    /// Number of content types engaged (out of 6)
    pub content_types_engaged: i32,
}

/// Detailed grade breakdown showing how the grade was calculated
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GradeBreakdown {
    /// Account age in days
    pub account_age_days: i64,
    /// Normalized score component (0-100) - 50% weight
    pub normalized_score: f32,
    /// Activity metrics (30% weight)
    pub activity_metrics: ActivityMetricsResponse,
    /// Engagement metrics (20% weight)
    pub engagement_metrics: EngagementMetricsResponse,
    /// Estimated percentile ranking
    pub percentile_estimate: f32,
}

/// Sorting options for the leaderboard
#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    #[default]
    TotalScore,
    OperatorScore,
    StageScore,
    RoguelikeScore,
    SandboxScore,
    MedalScore,
    BaseScore,
    CompositeScore,
    Grade,
}

impl SortBy {
    /// Convert to SQL expression for ORDER BY clause
    pub fn to_sql_expression(&self) -> &'static str {
        match self {
            Self::TotalScore => "COALESCE((score->>'totalScore')::FLOAT, 0)",
            Self::OperatorScore => "COALESCE((score->>'operatorScore')::FLOAT, 0)",
            Self::StageScore => "COALESCE((score->>'stageScore')::FLOAT, 0)",
            Self::RoguelikeScore => "COALESCE((score->>'roguelikeScore')::FLOAT, 0)",
            Self::SandboxScore => "COALESCE((score->>'sandboxScore')::FLOAT, 0)",
            Self::MedalScore => "COALESCE((score->>'medalScore')::FLOAT, 0)",
            Self::BaseScore => "COALESCE((score->>'baseScore')::FLOAT, 0)",
            Self::CompositeScore => "COALESCE((score->'grade'->>'compositeScore')::FLOAT, 0)",
            Self::Grade => {
                "CASE score->'grade'->>'grade' WHEN 'S' THEN 6 WHEN 'A' THEN 5 WHEN 'B' THEN 4 WHEN 'C' THEN 3 WHEN 'D' THEN 2 ELSE 1 END"
            }
        }
    }

    /// Get display name for the sort option
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TotalScore => "total_score",
            Self::OperatorScore => "operator_score",
            Self::StageScore => "stage_score",
            Self::RoguelikeScore => "roguelike_score",
            Self::SandboxScore => "sandbox_score",
            Self::MedalScore => "medal_score",
            Self::BaseScore => "base_score",
            Self::CompositeScore => "composite_score",
            Self::Grade => "grade",
        }
    }
}

/// Query parameters for the leaderboard endpoint
#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    /// Field to sort by (default: total_score)
    #[serde(default)]
    pub sort_by: SortBy,
    /// Sort order: "asc" or "desc" (default: desc)
    #[serde(default = "default_order")]
    pub order: String,
    /// Optional server filter (en, jp, cn, kr, tw)
    pub server: Option<String>,
    /// Results per page (default: 25, max: 100)
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Pagination offset (default: 0)
    #[serde(default)]
    pub offset: i64,
}

fn default_order() -> String {
    "desc".to_string()
}

fn default_limit() -> i64 {
    25
}

/// Full leaderboard response
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardResponse {
    pub entries: Vec<LeaderboardEntry>,
    pub pagination: PaginationInfo,
    pub meta: LeaderboardMeta,
}

/// Single leaderboard entry
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub uid: String,
    pub server: String,
    pub nickname: String,
    pub level: i64,
    pub avatar_id: Option<String>,
    pub total_score: f32,
    pub operator_score: f32,
    pub stage_score: f32,
    pub roguelike_score: f32,
    pub sandbox_score: f32,
    pub medal_score: f32,
    pub base_score: f32,
    pub composite_score: f32,
    pub grade: String,
    pub grade_breakdown: GradeBreakdown,
    pub updated_at: String,
}

/// Pagination information
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationInfo {
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
    pub has_more: bool,
}

/// Metadata about the leaderboard query
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardMeta {
    pub sort_by: String,
    pub order: String,
    pub server_filter: Option<String>,
}
