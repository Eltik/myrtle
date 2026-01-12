//! Leaderboard handler
//!
//! Provides the main handler for the `/leaderboard` endpoint with Redis caching.

use axum::{
    Json,
    extract::{Query, State},
};
use redis::AsyncCommands;

use crate::app::{error::ApiError, state::AppState};
use crate::database::models::user::User;

use super::types::*;

/// Handler for GET /leaderboard
///
/// Returns a paginated, sorted list of users based on various score metrics.
/// Results are cached in Redis for 5 minutes.
pub async fn get_leaderboard(
    State(state): State<AppState>,
    Query(params): Query<LeaderboardQuery>,
) -> Result<Json<LeaderboardResponse>, ApiError> {
    // Validate and clamp parameters
    let limit = params.limit.clamp(1, 100);
    let offset = params.offset.max(0);
    let order = if params.order == "asc" { "ASC" } else { "DESC" };

    // Build cache key
    let cache_key = format!(
        "leaderboard:{}:{}:{}:{}:{}",
        params.sort_by.as_str(),
        order.to_lowercase(),
        params.server.as_deref().unwrap_or("all"),
        limit,
        offset
    );

    // Check Redis cache
    let mut redis = state.redis.clone();
    if let Ok(cached) = redis.get::<_, String>(&cache_key).await
        && let Ok(response) = serde_json::from_str::<LeaderboardResponse>(&cached)
    {
        return Ok(Json(response));
    }

    // Query database
    let users = User::find_for_leaderboard(
        &state.db,
        params.sort_by.to_sql_expression(),
        order,
        params.server.as_deref(),
        limit,
        offset,
    )
    .await
    .map_err(|e| {
        eprintln!("Database error fetching leaderboard: {e:?}");
        ApiError::Internal("Failed to fetch leaderboard".into())
    })?;

    let total = User::count_for_leaderboard(&state.db, params.server.as_deref())
        .await
        .unwrap_or(0);

    // Transform to response entries
    let entries: Vec<LeaderboardEntry> = users
        .into_iter()
        .enumerate()
        .map(|(i, user)| transform_to_entry(user, offset + i as i64 + 1))
        .collect();

    let response = LeaderboardResponse {
        entries,
        pagination: PaginationInfo {
            limit,
            offset,
            total,
            has_more: offset + limit < total,
        },
        meta: LeaderboardMeta {
            sort_by: params.sort_by.as_str().to_string(),
            order: order.to_lowercase(),
            server_filter: params.server,
        },
    };

    // Cache response for 5 minutes
    if let Ok(json) = serde_json::to_string(&response) {
        let _: Result<(), _> = redis.set_ex(&cache_key, &json, 300u64).await;
    }

    Ok(Json(response))
}

/// Transform a User database record into a LeaderboardEntry
fn transform_to_entry(user: User, rank: i64) -> LeaderboardEntry {
    // Extract nickname from user.data JSONB
    let nickname = user
        .data
        .get("status")
        .and_then(|s| s.get("nickName"))
        .and_then(|n| n.as_str())
        .unwrap_or("Unknown")
        .to_string();

    // Extract level from user.data JSONB
    let level = user
        .data
        .get("status")
        .and_then(|s| s.get("level"))
        .and_then(|l| l.as_i64())
        .unwrap_or(0);

    // Extract avatar ID from user.data JSONB
    let avatar_id = user
        .data
        .get("status")
        .and_then(|s| s.get("avatar"))
        .and_then(|a| a.get("id"))
        .and_then(|id| id.as_str())
        .map(|s| s.to_string());

    let grade_data = user.score.get("grade");

    LeaderboardEntry {
        rank,
        uid: user.uid,
        server: user.server,
        nickname,
        level,
        avatar_id,
        total_score: extract_f32(&user.score, "totalScore"),
        operator_score: extract_f32(&user.score, "operatorScore"),
        stage_score: extract_f32(&user.score, "stageScore"),
        roguelike_score: extract_f32(&user.score, "roguelikeScore"),
        sandbox_score: extract_f32(&user.score, "sandboxScore"),
        medal_score: extract_f32(&user.score, "medalScore"),
        base_score: extract_f32(&user.score, "baseScore"),
        composite_score: grade_data
            .and_then(|g| g.get("compositeScore"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32,
        grade: grade_data
            .and_then(|g| g.get("grade"))
            .and_then(|v| v.as_str())
            .unwrap_or("F")
            .to_string(),
        grade_breakdown: extract_grade_breakdown(grade_data),
        updated_at: user.updated_at.to_rfc3339(),
    }
}

/// Extract grade breakdown from the grade JSONB object
fn extract_grade_breakdown(grade_data: Option<&serde_json::Value>) -> GradeBreakdown {
    let Some(grade) = grade_data else {
        return GradeBreakdown::default();
    };

    let activity = grade.get("activityMetrics");
    let engagement = grade.get("engagementMetrics");

    GradeBreakdown {
        account_age_days: grade
            .get("accountAgeDays")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        normalized_score: grade
            .get("normalizedScore")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32,
        activity_metrics: ActivityMetricsResponse {
            days_since_login: activity
                .and_then(|a| a.get("daysSinceLogin"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            login_recency_score: activity
                .and_then(|a| a.get("loginRecencyScore"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32,
            login_frequency_score: activity
                .and_then(|a| a.get("loginFrequencyScore"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32,
            consistency_score: activity
                .and_then(|a| a.get("consistencyScore"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32,
            total_activity_score: activity
                .and_then(|a| a.get("totalActivityScore"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32,
        },
        engagement_metrics: EngagementMetricsResponse {
            content_variety_score: engagement
                .and_then(|e| e.get("contentVarietyScore"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32,
            roguelike_depth_score: engagement
                .and_then(|e| e.get("roguelikeDepthScore"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32,
            stage_diversity_score: engagement
                .and_then(|e| e.get("stageDiversityScore"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32,
            progression_depth_score: engagement
                .and_then(|e| e.get("progressionDepthScore"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32,
            total_engagement_score: engagement
                .and_then(|e| e.get("totalEngagementScore"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32,
            content_types_engaged: engagement
                .and_then(|e| e.get("contentTypesEngaged"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
        },
        percentile_estimate: grade
            .get("percentileEstimate")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32,
    }
}

/// Helper to extract f32 from JSONB value
fn extract_f32(json: &serde_json::Value, key: &str) -> f32 {
    json.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32
}
