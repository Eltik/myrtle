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
pub struct SkinQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
    #[serde(flatten)]
    fields: FieldsParam,
}

/// GET /static/skins
pub async fn get_all_skins(
    State(state): State<AppState>,
    Query(params): Query<SkinQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let limit = params.pagination.limit.unwrap_or(50);
    let fields = params.fields.to_set();

    let cache_key = format!(
        "static:skins:all:limit:{}:cursor:{}:fields:{}",
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
            let all_skins = state
                .game_data
                .skins
                .enriched_skins
                .values()
                .collect::<Vec<_>>();

            let start_idx = params
                .pagination
                .cursor
                .and_then(|c| STANDARD.decode(&c).ok())
                .and_then(|b| String::from_utf8(b).ok())
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            let page: Vec<_> = all_skins
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

            let response = if let Some(ref field_set) = fields {
                let filtered: Vec<_> = page.iter().map(|s| filter_fields(s, field_set)).collect();
                serde_json::json!({
                    "skins": filtered,
                    "next_cursor": next_cursor,
                    "has_more": next_cursor.is_some(),
                    "total": all_skins.len()
                })
            } else {
                serde_json::json!({
                    "skins": page,
                    "next_cursor": next_cursor,
                    "has_more": next_cursor.is_some(),
                    "total": all_skins.len()
                })
            };

            Some(response)
        },
    )
    .await
}

/// GET /static/skins/{id}
pub async fn get_skin_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(fields): Query<FieldsParam>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let field_set = fields.to_set();
    let cache_key = format!(
        "static:skins:{}:fields:{}",
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
            state.game_data.skins.enriched_skins.get(&id).map(|skin| {
                if let Some(ref field_set) = field_set {
                    serde_json::json!({ "skin": filter_fields(skin, field_set) })
                } else {
                    serde_json::json!({ "skin": skin })
                }
            })
        },
    )
    .await
}

/// GET /static/skins/char/{char_id}
pub async fn get_skins_by_char_id(
    State(state): State<AppState>,
    Path(char_id): Path<String>,
    Query(fields): Query<FieldsParam>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let field_set = fields.to_set();
    let cache_key = format!(
        "static:skins:char:{}:fields:{}",
        char_id,
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
            let char_skins: Vec<_> = state
                .game_data
                .skins
                .enriched_skins
                .values()
                .filter(|skin| skin.skin.char_id == char_id)
                .collect();

            if char_skins.is_empty() {
                None
            } else if let Some(ref field_set) = field_set {
                let filtered: Vec<_> = char_skins
                    .iter()
                    .map(|s| filter_fields(*s, field_set))
                    .collect();
                Some(serde_json::json!({ "skins": filtered }))
            } else {
                Some(serde_json::json!({ "skins": char_skins }))
            }
        },
    )
    .await
}
