//! Search route types
//!
//! Query parameters, response types, and enums for the `/search` endpoint.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Main search query parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    // Text searches (fuzzy by default using ILIKE)
    /// Search nickname (fuzzy match with ILIKE %term%)
    pub nickname: Option<String>,

    /// Search by exact UID
    pub uid: Option<String>,

    /// Search profile bio/resume
    pub resume: Option<String>,

    // Exact match filters
    /// Filter by server (en, jp, cn, kr, tw)
    pub server: Option<String>,

    /// Filter by grade (S, A, B, C, D, F)
    pub grade: Option<String>,

    /// Filter by secretary character ID
    pub secretary: Option<String>,

    // Range queries (format: "min,max" or just "min" for >= or ",max" for <=)
    /// Level range (e.g., "100,120" or "100" for >= 100)
    pub level: Option<String>,

    /// Total score range
    pub total_score: Option<String>,

    /// Composite score range
    pub composite_score: Option<String>,

    /// Operator score range
    pub operator_score: Option<String>,

    /// Stage score range
    pub stage_score: Option<String>,

    /// Roguelike score range
    pub roguelike_score: Option<String>,

    /// Sandbox score range
    pub sandbox_score: Option<String>,

    /// Medal score range
    pub medal_score: Option<String>,

    /// Base score range
    pub base_score: Option<String>,

    // Advanced query options
    /// Query logic: "and" (default) or "or"
    #[serde(default)]
    pub logic: QueryLogic,

    // Pagination
    #[serde(default = "default_limit")]
    pub limit: i64,

    #[serde(default)]
    pub offset: i64,

    // Sorting
    #[serde(default)]
    pub sort_by: SortField,

    #[serde(default = "default_order")]
    pub order: String,

    // Field selection
    /// Comma-separated list of fields to return (minimal by default)
    /// Supported: "data", "score", "settings"
    pub fields: Option<String>,
}

fn default_limit() -> i64 {
    25
}
fn default_order() -> String {
    "desc".to_string()
}

/// Query logic for combining filters
#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryLogic {
    #[default]
    And,
    Or,
}

impl QueryLogic {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::And => "and",
            Self::Or => "or",
        }
    }

    pub fn sql_connector(&self) -> &'static str {
        match self {
            Self::And => " AND ",
            Self::Or => " OR ",
        }
    }
}

/// Sort field options
#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    #[default]
    TotalScore,
    CompositeScore,
    OperatorScore,
    StageScore,
    RoguelikeScore,
    SandboxScore,
    MedalScore,
    BaseScore,
    Level,
    Nickname,
    CreatedAt,
    UpdatedAt,
    /// When the user started playing Arknights (from game data)
    RegisterTs,
}

impl SortField {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TotalScore => "total_score",
            Self::CompositeScore => "composite_score",
            Self::OperatorScore => "operator_score",
            Self::StageScore => "stage_score",
            Self::RoguelikeScore => "roguelike_score",
            Self::SandboxScore => "sandbox_score",
            Self::MedalScore => "medal_score",
            Self::BaseScore => "base_score",
            Self::Level => "level",
            Self::Nickname => "nickname",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
            Self::RegisterTs => "register_ts",
        }
    }

    pub fn to_sql_expression(&self) -> &'static str {
        match self {
            Self::TotalScore => "COALESCE((score->>'totalScore')::FLOAT, 0)",
            Self::CompositeScore => "COALESCE((score->'grade'->>'compositeScore')::FLOAT, 0)",
            Self::OperatorScore => "COALESCE((score->>'operatorScore')::FLOAT, 0)",
            Self::StageScore => "COALESCE((score->>'stageScore')::FLOAT, 0)",
            Self::RoguelikeScore => "COALESCE((score->>'roguelikeScore')::FLOAT, 0)",
            Self::SandboxScore => "COALESCE((score->>'sandboxScore')::FLOAT, 0)",
            Self::MedalScore => "COALESCE((score->>'medalScore')::FLOAT, 0)",
            Self::BaseScore => "COALESCE((score->>'baseScore')::FLOAT, 0)",
            Self::Level => "COALESCE((data->'status'->>'level')::INT, 0)",
            Self::Nickname => "LOWER(data->'status'->>'nickName')",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
            Self::RegisterTs => "COALESCE((data->'status'->>'registerTs')::BIGINT, 0)",
        }
    }
}

/// Search response with pagination
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub results: Vec<SearchResultEntry>,
    pub pagination: PaginationInfo,
    pub meta: SearchMeta,
}

/// Individual search result (minimal by default)
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultEntry {
    pub uid: String,
    pub server: String,
    pub nickname: String,
    pub level: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secretary: Option<String>,
    pub grade: String,
    pub total_score: f32,
    pub updated_at: String,

    /// Only included if fields param requests it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    /// Only included if fields param requests it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<serde_json::Value>,

    /// Only included if fields param requests it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<serde_json::Value>,
}

/// Pagination info
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationInfo {
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
    pub has_more: bool,
}

/// Search metadata
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchMeta {
    pub query_logic: String,
    pub sort_by: String,
    pub order: String,
    pub filters_applied: Vec<String>,
    pub cached: bool,
}

/// Parsed range value for range queries
#[derive(Debug, Clone)]
pub struct RangeFilter {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl RangeFilter {
    /// Parse a range string like "100,200", "100", or ",200"
    pub fn parse(value: &str) -> Option<Self> {
        let parts: Vec<&str> = value.split(',').collect();
        match parts.len() {
            1 => {
                // Single value means >= min
                let min = parts[0].trim().parse().ok();
                if min.is_some() {
                    Some(Self { min, max: None })
                } else {
                    None
                }
            }
            2 => {
                let min = if parts[0].trim().is_empty() {
                    None
                } else {
                    parts[0].trim().parse().ok()
                };
                let max = if parts[1].trim().is_empty() {
                    None
                } else {
                    parts[1].trim().parse().ok()
                };
                if min.is_some() || max.is_some() {
                    Some(Self { min, max })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_filter_parse_single_value() {
        let range = RangeFilter::parse("100").unwrap();
        assert_eq!(range.min, Some(100.0));
        assert_eq!(range.max, None);
    }

    #[test]
    fn test_range_filter_parse_full_range() {
        let range = RangeFilter::parse("100,200").unwrap();
        assert_eq!(range.min, Some(100.0));
        assert_eq!(range.max, Some(200.0));
    }

    #[test]
    fn test_range_filter_parse_max_only() {
        let range = RangeFilter::parse(",200").unwrap();
        assert_eq!(range.min, None);
        assert_eq!(range.max, Some(200.0));
    }

    #[test]
    fn test_range_filter_parse_invalid() {
        assert!(RangeFilter::parse("abc").is_none());
        assert!(RangeFilter::parse(",").is_none());
    }
}

/// Lightweight user struct for search results
#[derive(Debug, Clone, FromRow)]
pub struct SearchUser {
    pub uid: String,
    pub server: String,
    pub updated_at: DateTime<Utc>,
    pub nickname: Option<String>,
    pub level: Option<i64>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
    pub total_score: Option<f64>,
    pub grade: Option<String>,
    // Only populated if explicitly requested via fields param
    #[sqlx(default)]
    pub data: Option<serde_json::Value>,
    #[sqlx(default)]
    pub score: Option<serde_json::Value>,
    #[sqlx(default)]
    pub settings: Option<serde_json::Value>,
}
