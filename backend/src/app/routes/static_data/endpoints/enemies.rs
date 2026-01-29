use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Deserialize;

use crate::app::routes::static_data::{
    fields::{FieldsParam, filter_fields},
    handler::cached_handler,
    pagination::PaginationParams,
};
use crate::app::state::AppState;
use crate::core::local::types::enemy::EnemyLevel;

#[derive(Deserialize)]
pub struct EnemyQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
    #[serde(flatten)]
    fields: FieldsParam,
    /// Comma-separated list of enemy levels to include (e.g., "NORMAL,ELITE,BOSS")
    levels: Option<String>,
}

/// GET /static/enemies
pub async fn get_all_enemies(
    State(state): State<AppState>,
    Query(params): Query<EnemyQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let limit = params.pagination.limit.unwrap_or(50);
    let fields = params.fields.to_set();
    let level_filter = params.levels.clone();

    let cache_key = format!(
        "static:enemies:all:limit:{}:cursor:{}:levels:{}:fields:{}",
        limit,
        params.pagination.cursor.as_deref().unwrap_or("start"),
        level_filter.as_deref().unwrap_or("all"),
        fields
            .as_ref()
            .map(|f| f.iter().cloned().collect::<Vec<_>>().join(","))
            .unwrap_or("all".into())
    );

    cached_handler(
        &mut state.redis.clone(),
        &cache_key,
        3600,
        &headers,
        || async {
            // Parse level filter if provided
            let level_set: Option<Vec<EnemyLevel>> = level_filter.as_ref().map(|levels| {
                levels
                    .split(',')
                    .filter_map(|l| {
                        let trimmed = l.trim().to_uppercase();
                        match trimmed.as_str() {
                            "NORMAL" => Some(EnemyLevel::Normal),
                            "ELITE" => Some(EnemyLevel::Elite),
                            "BOSS" => Some(EnemyLevel::Boss),
                            _ => None,
                        }
                    })
                    .collect()
            });

            // Filter enemies by level if specified
            let all_enemies: Vec<_> = state
                .game_data
                .enemies
                .enemy_data
                .values()
                .filter(|enemy| {
                    level_set
                        .as_ref()
                        .is_none_or(|levels| levels.contains(&enemy.enemy_level))
                })
                .collect();

            // Cursor-based pagination
            let start_idx = params
                .pagination
                .cursor
                .and_then(|c| STANDARD.decode(&c).ok())
                .and_then(|b| String::from_utf8(b).ok())
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            let page: Vec<_> = all_enemies
                .iter()
                .skip(start_idx)
                .take(limit)
                .cloned()
                .collect();

            let next_cursor = if page.len() == limit {
                Some(STANDARD.encode((start_idx + limit).to_string()))
            } else {
                None
            };

            // Apply field filtering if requested
            let response = if let Some(ref field_set) = fields {
                let filtered: Vec<_> = page
                    .iter()
                    .map(|enemy| filter_fields(enemy, field_set))
                    .collect();
                serde_json::json!({
                    "enemies": filtered,
                    "next_cursor": next_cursor,
                    "has_more": next_cursor.is_some(),
                    "total": all_enemies.len()
                })
            } else {
                serde_json::json!({
                    "enemies": page,
                    "next_cursor": next_cursor,
                    "has_more": next_cursor.is_some(),
                    "total": all_enemies.len()
                })
            };

            Some(response)
        },
    )
    .await
}

/// GET /static/enemies/:id
pub async fn get_enemy_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(fields): Query<FieldsParam>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let field_set = fields.to_set();
    let cache_key = format!(
        "static:enemies:{}:fields:{}",
        id,
        field_set
            .as_ref()
            .map(|f| f.iter().cloned().collect::<Vec<_>>().join(","))
            .unwrap_or("all".into())
    );

    cached_handler(
        &mut state.redis.clone(),
        &cache_key,
        3600,
        &headers,
        || async {
            state.game_data.enemies.enemy_data.get(&id).map(|enemy| {
                if let Some(ref field_set) = field_set {
                    serde_json::json!({ "enemy": filter_fields(enemy, field_set) })
                } else {
                    serde_json::json!({ "enemy": enemy })
                }
            })
        },
    )
    .await
}

/// GET /static/enemies/races
pub async fn get_all_races(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let cache_key = "static:enemies:races".to_string();

    cached_handler(
        &mut state.redis.clone(),
        &cache_key,
        3600,
        &headers,
        || async {
            let races: Vec<_> = state.game_data.enemies.race_data.values().collect();
            Some(serde_json::json!({
                "races": races,
                "total": races.len()
            }))
        },
    )
    .await
}

/// GET /static/enemies/levels
pub async fn get_level_info(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let cache_key = "static:enemies:levels".to_string();

    cached_handler(
        &mut state.redis.clone(),
        &cache_key,
        3600,
        &headers,
        || async {
            Some(serde_json::json!({
                "levels": state.game_data.enemies.level_info_list
            }))
        },
    )
    .await
}
