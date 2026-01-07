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
use crate::core::local::types::stage::StageType;

#[derive(Deserialize)]
pub struct StageQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
    #[serde(flatten)]
    fields: FieldsParam,
    /// Comma-separated list of stage types to exclude (e.g., "GUIDE")
    #[serde(rename = "excludeTypes")]
    exclude_types: Option<String>,
    /// Comma-separated list of stage types to include (e.g., "MAIN,SUB")
    types: Option<String>,
    /// Filter by zone ID
    #[serde(rename = "zoneId")]
    zone_id: Option<String>,
}

fn parse_stage_type(s: &str) -> Option<StageType> {
    let trimmed = s.trim().to_uppercase();
    match trimmed.as_str() {
        "MAIN" => Some(StageType::Main),
        "SUB" => Some(StageType::Sub),
        "ACTIVITY" => Some(StageType::Activity),
        "DAILY" => Some(StageType::Daily),
        "CAMPAIGN" => Some(StageType::Campaign),
        "CLIMB_TOWER" => Some(StageType::ClimbTower),
        "GUIDE" => Some(StageType::Guide),
        "SPECIAL_STORY" => Some(StageType::SpecialStory),
        _ => None,
    }
}

/// GET /static/stages
pub async fn get_all_stages(
    State(state): State<AppState>,
    Query(params): Query<StageQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let limit = params.pagination.limit.unwrap_or(100);
    let fields = params.fields.to_set();
    let exclude_types = params.exclude_types.clone();
    let include_types = params.types.clone();
    let zone_id_filter = params.zone_id.clone();

    let cache_key = format!(
        "static:stages:all:limit:{}:cursor:{}:exclude:{}:types:{}:zone:{}:fields:{}",
        limit,
        params.pagination.cursor.as_deref().unwrap_or("start"),
        exclude_types.as_deref().unwrap_or("none"),
        include_types.as_deref().unwrap_or("all"),
        zone_id_filter.as_deref().unwrap_or("all"),
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
            // Parse exclude types if provided
            let exclude_set: Option<Vec<StageType>> = exclude_types
                .as_ref()
                .map(|types| types.split(',').filter_map(parse_stage_type).collect());

            // Parse include types if provided
            let include_set: Option<Vec<StageType>> = include_types
                .as_ref()
                .map(|types| types.split(',').filter_map(parse_stage_type).collect());

            // Filter stages
            let all_stages: Vec<_> = state
                .game_data
                .stages
                .values()
                .filter(|stage| {
                    // Exclude types filter
                    if let Some(ref excludes) = exclude_set
                        && excludes.contains(&stage.stage_type)
                    {
                        return false;
                    }
                    // Include types filter
                    if let Some(ref includes) = include_set
                        && !includes.contains(&stage.stage_type)
                    {
                        return false;
                    }
                    // Zone ID filter
                    if let Some(ref zone_id) = zone_id_filter
                        && &stage.zone_id != zone_id
                    {
                        return false;
                    }
                    true
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

            let page: Vec<_> = all_stages
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
                    .map(|stage| filter_fields(stage, field_set))
                    .collect();
                serde_json::json!({
                    "stages": filtered,
                    "next_cursor": next_cursor,
                    "has_more": next_cursor.is_some(),
                    "total": all_stages.len()
                })
            } else {
                serde_json::json!({
                    "stages": page,
                    "next_cursor": next_cursor,
                    "has_more": next_cursor.is_some(),
                    "total": all_stages.len()
                })
            };

            Some(response)
        },
    )
    .await
}

/// GET /static/stages/:id
pub async fn get_stage_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(fields): Query<FieldsParam>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let field_set = fields.to_set();
    let cache_key = format!(
        "static:stages:{}:fields:{}",
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
            state.game_data.stages.get(&id).map(|stage| {
                if let Some(ref field_set) = field_set {
                    serde_json::json!({ "stage": filter_fields(stage, field_set) })
                } else {
                    serde_json::json!({ "stage": stage })
                }
            })
        },
    )
    .await
}

/// GET /static/stages/zone/:zone_id
pub async fn get_stages_by_zone(
    State(state): State<AppState>,
    Path(zone_id): Path<String>,
    Query(params): Query<StageQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let limit = params.pagination.limit.unwrap_or(100);
    let fields = params.fields.to_set();
    let exclude_types = params.exclude_types.clone();

    let cache_key = format!(
        "static:stages:zone:{}:limit:{}:cursor:{}:exclude:{}:fields:{}",
        zone_id,
        limit,
        params.pagination.cursor.as_deref().unwrap_or("start"),
        exclude_types.as_deref().unwrap_or("none"),
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
            // Parse exclude types if provided
            let exclude_set: Option<Vec<StageType>> = exclude_types
                .as_ref()
                .map(|types| types.split(',').filter_map(parse_stage_type).collect());

            // Filter stages by zone ID
            let zone_stages: Vec<_> = state
                .game_data
                .stages
                .values()
                .filter(|stage| {
                    // Zone ID filter
                    if stage.zone_id != zone_id {
                        return false;
                    }
                    // Exclude types filter
                    if let Some(ref excludes) = exclude_set
                        && excludes.contains(&stage.stage_type)
                    {
                        return false;
                    }
                    true
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

            let page: Vec<_> = zone_stages
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
                    .map(|stage| filter_fields(stage, field_set))
                    .collect();
                serde_json::json!({
                    "stages": filtered,
                    "next_cursor": next_cursor,
                    "has_more": next_cursor.is_some(),
                    "total": zone_stages.len()
                })
            } else {
                serde_json::json!({
                    "stages": page,
                    "next_cursor": next_cursor,
                    "has_more": next_cursor.is_some(),
                    "total": zone_stages.len()
                })
            };

            Some(response)
        },
    )
    .await
}
