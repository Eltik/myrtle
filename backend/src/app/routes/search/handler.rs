//! Search handler
//!
//! Provides the main handler for the `/search` endpoint with Redis caching.

use axum::{
    Json,
    extract::{Query, State},
};
use redis::AsyncCommands;
use sqlx::PgPool;

use crate::app::{error::ApiError, state::AppState};

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

    // Parse fields selection to determine what to fetch from database
    let fields_str = params.fields.as_deref().unwrap_or("");
    let include_data = fields_str.contains("data");
    let include_score = fields_str.contains("score");
    let include_settings = fields_str.contains("settings");

    // Build search query
    let mut builder = SearchQueryBuilder::new(params.logic);
    builder.add_privacy_filter(); // Filter out users with publicProfile: false
    let filters_applied = apply_filters(&mut builder, &params)?;

    builder.set_sort(params.sort_by.to_sql_expression(), order);
    builder.set_pagination(limit, offset);

    let (query, query_params) =
        builder.build_with_fields(include_data, include_score, include_settings);
    let users = execute_search_query(&state.db, &query, &query_params).await?;

    // Get total count for pagination
    let (count_query, count_params) = builder.build_count();
    let total = execute_count_query(&state.db, &count_query, &count_params).await?;

    // Transform to response entries
    let results: Vec<SearchResultEntry> =
        users.into_iter().map(transform_to_search_entry).collect();

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
) -> Result<Vec<SearchUser>, ApiError> {
    // Build the query dynamically with sqlx
    let mut query_builder = sqlx::query_as::<_, SearchUser>(query);

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

/// Transform a SearchUser database record into a SearchResultEntry
fn transform_to_search_entry(user: SearchUser) -> SearchResultEntry {
    let nickname = user.nickname.unwrap_or_else(|| "Unknown".to_string());
    let level = user.level.unwrap_or(0);
    let secretary = user.secretary.clone();
    let secretary_skin_id = user.secretary_skin_id.as_deref();

    // Use secretarySkinId for avatar, falling back to secretary base ID for default skins
    let avatar_id = match (secretary.as_deref(), secretary_skin_id) {
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

    let grade = user.grade.unwrap_or_else(|| "F".to_string());
    let total_score = user.total_score.unwrap_or(0.0) as f32;

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
        data: user.data,
        score: user.score,
        settings: user.settings,
    }
}
