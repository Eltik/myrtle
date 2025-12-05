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

#[derive(Deserialize)]
pub struct HandbookQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
    #[serde(flatten)]
    fields: FieldsParam,
}

/// GET /static/handbook
pub async fn get_all_handbook(
    State(state): State<AppState>,
    Query(params): Query<HandbookQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let limit = params.pagination.limit.unwrap_or(50);
    let fields = params.fields.to_set();

    let cache_key = format!(
        "static:handbook:all:limit:{}:cursor:{}:fields:{}",
        limit,
        params.pagination.cursor.as_deref().unwrap_or("start"),
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
            let all_handbook = state
                .game_data
                .handbook
                .handbook_dict
                .values()
                .collect::<Vec<_>>();

            // Cursor-based pagination
            let start_idx = params
                .pagination
                .cursor
                .and_then(|c| STANDARD.decode(&c).ok())
                .and_then(|b| String::from_utf8(b).ok())
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            let page: Vec<_> = all_handbook
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
                let filtered: Vec<_> = page.iter().map(|hb| filter_fields(hb, field_set)).collect();
                serde_json::json!({
                    "handbook": filtered,
                    "next_cursor": next_cursor,
                    "has_more": next_cursor.is_some(),
                    "total": all_handbook.len()
                })
            } else {
                serde_json::json!({
                    "handbook": page,
                    "next_cursor": next_cursor,
                    "has_more": next_cursor.is_some(),
                    "total": all_handbook.len()
                })
            };

            Some(response)
        },
    )
    .await
}

/// GET /static/handbook/:id
pub async fn get_handbook_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(fields): Query<FieldsParam>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let field_set = fields.to_set();
    let cache_key = format!(
        "static:handbook:{}:fields:{}",
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
            state.game_data.handbook.handbook_dict.get(&id).map(|hb| {
                if let Some(ref field_set) = field_set {
                    serde_json::json!({ "handbook": filter_fields(hb, field_set) })
                } else {
                    serde_json::json!({ "handbook": hb })
                }
            })
        },
    )
    .await
}
