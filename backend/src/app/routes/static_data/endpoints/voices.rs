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
pub struct VoiceQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
    #[serde(flatten)]
    fields: FieldsParam,
}

/// GET /static/voices
pub async fn get_all_voices(
    State(state): State<AppState>,
    Query(params): Query<VoiceQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let limit = params.pagination.limit.unwrap_or(50);
    let fields = params.fields.to_set();

    let cache_key = format!(
        "static:voices:all:limit:{}:cursor:{}:fields:{}",
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
            let all_voices = state
                .game_data
                .voices
                .char_words
                .values()
                .collect::<Vec<_>>();

            let start_idx = params
                .pagination
                .cursor
                .and_then(|c| STANDARD.decode(&c).ok())
                .and_then(|b| String::from_utf8(b).ok())
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            let page: Vec<_> = all_voices
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
                let filtered: Vec<_> = page.iter().map(|v| filter_fields(v, field_set)).collect();
                serde_json::json!({
                    "voices": filtered,
                    "next_cursor": next_cursor,
                    "has_more": next_cursor.is_some(),
                    "total": all_voices.len()
                })
            } else {
                serde_json::json!({
                    "voices": page,
                    "next_cursor": next_cursor,
                    "has_more": next_cursor.is_some(),
                    "total": all_voices.len()
                })
            };

            Some(response)
        },
    )
    .await
}

/// GET /static/voices/{id}
pub async fn get_voice_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(fields): Query<FieldsParam>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let field_set = fields.to_set();
    let cache_key = format!(
        "static:voices:{}:fields:{}",
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
            state.game_data.voices.char_words.get(&id).map(|voice| {
                if let Some(ref field_set) = field_set {
                    serde_json::json!({ "voice": filter_fields(voice, field_set) })
                } else {
                    serde_json::json!({ "voice": voice })
                }
            })
        },
    )
    .await
}

/// GET /static/voices/char/{char_id}
pub async fn get_voices_by_char_id(
    State(state): State<AppState>,
    Path(char_id): Path<String>,
    Query(fields): Query<FieldsParam>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let field_set = fields.to_set();
    let cache_key = format!(
        "static:voices:char:{}:fields:{}",
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
            let char_voices: Vec<_> = state
                .game_data
                .voices
                .char_words
                .values()
                .filter(|voice| voice.char_id == char_id)
                .collect();

            if char_voices.is_empty() {
                None
            } else if let Some(ref field_set) = field_set {
                let filtered: Vec<_> = char_voices
                    .iter()
                    .map(|v| filter_fields(*v, field_set))
                    .collect();
                Some(serde_json::json!({ "voices": filtered }))
            } else {
                Some(serde_json::json!({ "voices": char_voices }))
            }
        },
    )
    .await
}
