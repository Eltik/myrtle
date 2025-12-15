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
pub struct ChibiQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
    #[serde(flatten)]
    fields: FieldsParam,
}

/// GET /static/chibis
/// Returns all chibi character data with pagination
pub async fn get_all_chibis(
    State(state): State<AppState>,
    Query(params): Query<ChibiQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let limit = params.pagination.limit.unwrap_or(50);
    let fields = params.fields.to_set();

    let cache_key = format!(
        "static:chibis:all:limit:{}:cursor:{}:fields:{}",
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
            let all_chibis = &state.game_data.chibis.characters;

            let start_idx = params
                .pagination
                .cursor
                .and_then(|c| STANDARD.decode(&c).ok())
                .and_then(|b| String::from_utf8(b).ok())
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            let page: Vec<_> = all_chibis.iter().skip(start_idx).take(limit).collect();

            let next_cursor = if page.len() == limit {
                Some(STANDARD.encode((start_idx + limit).to_string()))
            } else {
                None
            };

            let response = if let Some(ref field_set) = fields {
                let filtered: Vec<_> = page.iter().map(|c| filter_fields(*c, field_set)).collect();
                serde_json::json!({
                    "chibis": filtered,
                    "next_cursor": next_cursor,
                    "has_more": next_cursor.is_some(),
                    "total": all_chibis.len()
                })
            } else {
                serde_json::json!({
                    "chibis": page,
                    "next_cursor": next_cursor,
                    "has_more": next_cursor.is_some(),
                    "total": all_chibis.len()
                })
            };

            Some(response)
        },
    )
    .await
}

/// GET /static/chibis/{operator_code}
/// Returns chibi data for a specific operator
pub async fn get_chibi_by_operator(
    State(state): State<AppState>,
    Path(operator_code): Path<String>,
    Query(fields): Query<FieldsParam>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let field_set = fields.to_set();
    let cache_key = format!(
        "static:chibis:{}:fields:{}",
        operator_code,
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
            state
                .game_data
                .chibis
                .get_by_operator(&operator_code)
                .map(|chibi| {
                    if let Some(ref field_set) = field_set {
                        serde_json::json!({ "chibi": filter_fields(chibi, field_set) })
                    } else {
                        serde_json::json!({ "chibi": chibi })
                    }
                })
        },
    )
    .await
}

/// GET /static/chibis/operators
/// Returns a list of all operator codes that have chibi data
pub async fn get_chibi_operators(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let cache_key = "static:chibis:operators";

    cached_handler(
        &mut state.redis.clone(),
        cache_key,
        3600,
        &headers,
        || async {
            let operators: Vec<&String> = state
                .game_data
                .chibis
                .characters
                .iter()
                .map(|c| &c.operator_code)
                .collect();

            Some(serde_json::json!({
                "operators": operators,
                "total": operators.len()
            }))
        },
    )
    .await
}
