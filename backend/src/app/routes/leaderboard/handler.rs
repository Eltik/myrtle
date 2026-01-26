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

use super::types::{
    ActivityMetricsResponse, EngagementMetricsResponse, GradeBreakdown, LeaderboardEntry,
    LeaderboardFields, LeaderboardMeta, LeaderboardQuery, LeaderboardResponse, LeaderboardUser,
    PaginationInfo,
};

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

    // Build cache key (includes fields parameter for different cached responses)
    let cache_key = format!(
        "leaderboard:{}:{}:{}:{}:{}:{}",
        params.sort_by.as_str(),
        order.to_lowercase(),
        params.server.as_deref().unwrap_or("all"),
        limit,
        offset,
        params.fields.as_str()
    );

    // Check Redis cache
    let mut redis = state.redis.clone();
    if let Ok(cached) = redis.get::<_, String>(&cache_key).await
        && let Ok(response) = serde_json::from_str::<LeaderboardResponse>(&cached)
    {
        return Ok(Json(response));
    }

    let is_full = params.fields == LeaderboardFields::Full;
    let users = User::find_for_leaderboard_optimized(
        &state.db,
        params.sort_by.to_sql_expression(),
        order,
        params.server.as_deref(),
        limit,
        offset,
        is_full,
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
        .map(|(i, user)| transform_to_entry(user, offset + i as i64 + 1, is_full))
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
            fields: params.fields.as_str().to_string(),
        },
    };

    // Cache response for 5 minutes
    if let Ok(json) = serde_json::to_string(&response) {
        let _: Result<(), _> = redis.set_ex(&cache_key, &json, 300u64).await;
    }

    Ok(Json(response))
}

/// Transform a LeaderboardUser database record into a LeaderboardEntry
fn transform_to_entry(user: LeaderboardUser, rank: i64, is_full: bool) -> LeaderboardEntry {
    let nickname = user.nickname.unwrap_or_else(|| "Unknown".to_string());
    let level = user.level.unwrap_or(0);
    let secretary = user.secretary.as_deref();
    let secretary_skin_id = user.secretary_skin_id.as_deref();

    // Use secretarySkinId for avatar, falling back to secretary base ID for default skins
    let avatar_id = match (secretary, secretary_skin_id) {
        (Some(sec), Some(skin_id)) => {
            // If skin doesn't contain @ and ends with #1, use base secretary ID
            if !skin_id.contains('@') && skin_id.ends_with("#1") {
                Some(sec.to_string())
            } else {
                Some(skin_id.to_string())
            }
        }
        (Some(sec), None) => Some(sec.to_string()),
        _ => None,
    };

    LeaderboardEntry {
        // Core fields (always present)
        rank,
        uid: user.uid,
        server: user.server,
        nickname,
        level,
        avatar_id,
        total_score: user.total_score.unwrap_or(0.0) as f32,
        grade: user.grade.unwrap_or_else(|| "F".to_string()),
        updated_at: user.updated_at.to_rfc3339(),

        operator_score: if is_full {
            user.operator_score.map(|s| s as f32)
        } else {
            None
        },
        stage_score: if is_full {
            user.stage_score.map(|s| s as f32)
        } else {
            None
        },
        roguelike_score: if is_full {
            user.roguelike_score.map(|s| s as f32)
        } else {
            None
        },
        sandbox_score: if is_full {
            user.sandbox_score.map(|s| s as f32)
        } else {
            None
        },
        medal_score: if is_full {
            user.medal_score.map(|s| s as f32)
        } else {
            None
        },
        base_score: if is_full {
            user.base_score.map(|s| s as f32)
        } else {
            None
        },
        composite_score: if is_full {
            user.composite_score.map(|s| s as f32)
        } else {
            None
        },
        grade_breakdown: if is_full {
            Some(extract_grade_breakdown(user.grade_data.as_ref()))
        } else {
            None
        },
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
