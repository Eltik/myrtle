//! Search handler
//!
//! Provides the main handler for the `/search` endpoint with Redis caching.

use std::collections::HashSet;

use axum::{
    Json,
    extract::{Query, State},
};
use redis::AsyncCommands;
use sqlx::PgPool;

use crate::app::{error::ApiError, state::AppState};
use crate::database::models::user::User;

use super::filters::apply_filters;
use super::query_builder::{QueryParam, SearchQueryBuilder};
use super::types::*;

/// Handler for GET /search
///
/// Returns a paginated, filtered list of users based on various search criteria.
/// Results are cached in Redis for 2 minutes.
pub async fn search_users(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, ApiError> {
    // Validate and clamp parameters
    let limit = params.limit.clamp(1, 100);
    let offset = params.offset.max(0);
    let order = if params.order == "asc" { "ASC" } else { "DESC" };

    // Build cache key from query parameters
    let cache_key = build_cache_key(&params, limit, offset, order);

    // Check Redis cache first
    let mut redis = state.redis.clone();
    if let Ok(cached) = redis.get::<_, String>(&cache_key).await
        && let Ok(mut response) = serde_json::from_str::<SearchResponse>(&cached)
    {
        response.meta.cached = true;
        return Ok(Json(response));
    }

    // Build search query
    let mut builder = SearchQueryBuilder::new(params.logic);
    builder.add_privacy_filter(); // Filter out users with publicProfile: false
    let filters_applied = apply_filters(&mut builder, &params)?;

    builder.set_sort(params.sort_by.to_sql_expression(), order);
    builder.set_pagination(limit, offset);

    // Execute search query
    let (query, query_params) = builder.build();
    let users = execute_search_query(&state.db, &query, &query_params).await?;

    // Get total count for pagination
    let (count_query, count_params) = builder.build_count();
    let total = execute_count_query(&state.db, &count_query, &count_params).await?;

    // Parse fields selection
    let selected_fields: Option<HashSet<String>> = params.fields.as_ref().map(|f| {
        f.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    });

    // Transform to response entries
    let results: Vec<SearchResultEntry> = users
        .into_iter()
        .map(|user| transform_to_search_entry(user, &selected_fields))
        .collect();

    let response = SearchResponse {
        results,
        pagination: PaginationInfo {
            limit,
            offset,
            total,
            has_more: offset + limit < total,
        },
        meta: SearchMeta {
            query_logic: params.logic.as_str().to_string(),
            sort_by: params.sort_by.as_str().to_string(),
            order: order.to_lowercase(),
            filters_applied,
            cached: false,
        },
    };

    // Cache response for 2 minutes
    if let Ok(json) = serde_json::to_string(&response) {
        let _: Result<(), _> = redis.set_ex(&cache_key, &json, 120u64).await;
    }

    Ok(Json(response))
}

/// Build a deterministic cache key from query parameters
fn build_cache_key(params: &SearchQuery, limit: i64, offset: i64, order: &str) -> String {
    let mut parts = vec!["search".to_string()];

    if let Some(ref v) = params.nickname {
        parts.push(format!("nick:{}", v));
    }
    if let Some(ref v) = params.uid {
        parts.push(format!("uid:{}", v));
    }
    if let Some(ref v) = params.resume {
        parts.push(format!("resume:{}", v));
    }
    if let Some(ref v) = params.server {
        parts.push(format!("srv:{}", v));
    }
    if let Some(ref v) = params.grade {
        parts.push(format!("grd:{}", v));
    }
    if let Some(ref v) = params.secretary {
        parts.push(format!("sec:{}", v));
    }
    if let Some(ref v) = params.level {
        parts.push(format!("lvl:{}", v));
    }
    if let Some(ref v) = params.total_score {
        parts.push(format!("ts:{}", v));
    }
    if let Some(ref v) = params.composite_score {
        parts.push(format!("cs:{}", v));
    }
    if let Some(ref v) = params.operator_score {
        parts.push(format!("os:{}", v));
    }
    if let Some(ref v) = params.stage_score {
        parts.push(format!("ss:{}", v));
    }
    if let Some(ref v) = params.roguelike_score {
        parts.push(format!("rs:{}", v));
    }
    if let Some(ref v) = params.sandbox_score {
        parts.push(format!("sbs:{}", v));
    }
    if let Some(ref v) = params.medal_score {
        parts.push(format!("ms:{}", v));
    }
    if let Some(ref v) = params.base_score {
        parts.push(format!("bs:{}", v));
    }
    if let Some(ref v) = params.fields {
        parts.push(format!("fld:{}", v));
    }

    parts.push(format!("logic:{}", params.logic.as_str()));
    parts.push(format!("sort:{}", params.sort_by.as_str()));
    parts.push(format!("order:{}", order.to_lowercase()));
    parts.push(format!("lim:{}", limit));
    parts.push(format!("off:{}", offset));

    parts.join(":")
}

/// Execute the search query with bound parameters
async fn execute_search_query(
    pool: &PgPool,
    query: &str,
    params: &[QueryParam],
) -> Result<Vec<User>, ApiError> {
    // Build the query dynamically with sqlx
    let mut query_builder = sqlx::query_as::<_, User>(query);

    for param in params {
        query_builder = match param {
            QueryParam::String(s) => query_builder.bind(s.clone()),
            QueryParam::Int(i) => query_builder.bind(*i),
            QueryParam::Float(f) => query_builder.bind(*f),
        };
    }

    query_builder.fetch_all(pool).await.map_err(|e| {
        eprintln!("Database error executing search: {e:?}");
        ApiError::Internal("Failed to execute search".into())
    })
}

/// Execute the count query for pagination
async fn execute_count_query(
    pool: &PgPool,
    query: &str,
    params: &[QueryParam],
) -> Result<i64, ApiError> {
    let mut query_builder = sqlx::query_scalar::<_, i64>(query);

    for param in params {
        query_builder = match param {
            QueryParam::String(s) => query_builder.bind(s.clone()),
            QueryParam::Int(i) => query_builder.bind(*i),
            QueryParam::Float(f) => query_builder.bind(*f),
        };
    }

    query_builder.fetch_one(pool).await.map_err(|e| {
        eprintln!("Database error executing count: {e:?}");
        ApiError::Internal("Failed to count results".into())
    })
}

/// Transform a User database record into a SearchResultEntry
fn transform_to_search_entry(
    user: User,
    selected_fields: &Option<HashSet<String>>,
) -> SearchResultEntry {
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

    // Extract avatar from secretary skin ID (matches user profile behavior)
    let status = user.data.get("status");
    let secretary = status
        .and_then(|s| s.get("secretary"))
        .and_then(|s| s.as_str());
    let secretary_skin_id = status
        .and_then(|s| s.get("secretarySkinId"))
        .and_then(|s| s.as_str());

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

    // Keep secretary as separate field for filtering
    let secretary = secretary.map(|s| s.to_string());

    // Extract grade from score JSONB
    let grade = user
        .score
        .get("grade")
        .and_then(|g| g.get("grade"))
        .and_then(|g| g.as_str())
        .unwrap_or("F")
        .to_string();

    // Extract total score
    let total_score = user
        .score
        .get("totalScore")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;

    // Include full data only if requested via fields param
    let include_data = selected_fields.as_ref().is_some_and(|f| f.contains("data"));

    let include_score = selected_fields
        .as_ref()
        .is_some_and(|f| f.contains("score"));

    let include_settings = selected_fields
        .as_ref()
        .is_some_and(|f| f.contains("settings"));

    SearchResultEntry {
        uid: user.uid,
        server: user.server,
        nickname,
        level,
        avatar_id,
        secretary,
        grade,
        total_score,
        updated_at: user.updated_at.to_rfc3339(),
        data: if include_data { Some(user.data) } else { None },
        score: if include_score {
            Some(user.score)
        } else {
            None
        },
        settings: if include_settings {
            Some(user.settings)
        } else {
            None
        },
    }
}
