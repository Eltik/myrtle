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
use crate::core::local::types::zone::ZoneType;

#[derive(Deserialize)]
pub struct ZoneQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
    #[serde(flatten)]
    fields: FieldsParam,
    /// Comma-separated list of zone types to include (e.g., "MAINLINE,SIDESTORY,BRANCHLINE")
    types: Option<String>,
}

/// GET /static/zones
pub async fn get_all_zones(
    State(state): State<AppState>,
    Query(params): Query<ZoneQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let limit = params.pagination.limit.unwrap_or(100);
    let fields = params.fields.to_set();
    let type_filter = params.types.clone();

    let cache_key = format!(
        "static:zones:all:limit:{}:cursor:{}:types:{}:fields:{}",
        limit,
        params.pagination.cursor.as_deref().unwrap_or("start"),
        type_filter.as_deref().unwrap_or("all"),
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
            // Parse type filter if provided
            let type_set: Option<Vec<ZoneType>> = type_filter.as_ref().map(|types| {
                types
                    .split(',')
                    .filter_map(|t| {
                        let trimmed = t.trim().to_uppercase();
                        match trimmed.as_str() {
                            "MAINLINE" => Some(ZoneType::Mainline),
                            "SIDESTORY" => Some(ZoneType::Sidestory),
                            "BRANCHLINE" => Some(ZoneType::Branchline),
                            "ACTIVITY" => Some(ZoneType::Activity),
                            "WEEKLY" => Some(ZoneType::Weekly),
                            "CAMPAIGN" => Some(ZoneType::Campaign),
                            "CLIMB_TOWER" => Some(ZoneType::ClimbTower),
                            "ROGUELIKE" => Some(ZoneType::Roguelike),
                            "GUIDE" => Some(ZoneType::Guide),
                            _ => None,
                        }
                    })
                    .collect()
            });

            // Filter zones by type if specified
            let all_zones: Vec<_> = state
                .game_data
                .zones
                .values()
                .filter(|zone| {
                    type_set
                        .as_ref()
                        .is_none_or(|types| types.contains(&zone.zone_type))
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

            let page: Vec<_> = all_zones
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
                    .map(|zone| filter_fields(zone, field_set))
                    .collect();
                serde_json::json!({
                    "zones": filtered,
                    "next_cursor": next_cursor,
                    "has_more": next_cursor.is_some(),
                    "total": all_zones.len()
                })
            } else {
                serde_json::json!({
                    "zones": page,
                    "next_cursor": next_cursor,
                    "has_more": next_cursor.is_some(),
                    "total": all_zones.len()
                })
            };

            Some(response)
        },
    )
    .await
}

/// GET /static/zones/:id
pub async fn get_zone_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(fields): Query<FieldsParam>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let field_set = fields.to_set();
    let cache_key = format!(
        "static:zones:{}:fields:{}",
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
            state.game_data.zones.get(&id).map(|zone| {
                if let Some(ref field_set) = field_set {
                    serde_json::json!({ "zone": filter_fields(zone, field_set) })
                } else {
                    serde_json::json!({ "zone": zone })
                }
            })
        },
    )
    .await
}
