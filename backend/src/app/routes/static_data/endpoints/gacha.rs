use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use serde::Deserialize;

use crate::app::routes::static_data::{
    fields::{FieldsParam, filter_fields},
    handler::cached_handler,
};
use crate::app::state::AppState;

#[derive(Deserialize)]
pub struct GachaQuery {
    #[serde(flatten)]
    fields: FieldsParam,
}

/// GET /static/gacha
/// Returns all gacha data
pub async fn get_all_gacha(
    State(state): State<AppState>,
    Query(params): Query<GachaQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let field_set = params.fields.to_set();
    let cache_key = format!(
        "static:gacha:all:fields:{}",
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
            if let Some(ref field_set) = field_set {
                Some(serde_json::json!({
                    "gacha": filter_fields(&state.game_data.gacha, field_set)
                }))
            } else {
                Some(serde_json::json!({
                    "gacha": &state.game_data.gacha
                }))
            }
        },
    )
    .await
}

/// GET /static/gacha/recruitment
/// Returns recruitment data (tags, pools)
pub async fn get_recruitment(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let cache_key = "static:gacha:recruitment".to_string();

    cached_handler(
        &mut state.redis.clone(),
        &cache_key,
        3600,
        &headers,
        || async {
            // Build tag maps
            let tag_map: std::collections::HashMap<i32, _> = state
                .game_data
                .gacha
                .gacha_tags
                .iter()
                .map(|tag| (tag.tag_id, tag.clone()))
                .collect();

            let tag_name_map: std::collections::HashMap<String, _> = state
                .game_data
                .gacha
                .gacha_tags
                .iter()
                .map(|tag| (tag.tag_name.clone(), tag.clone()))
                .collect();

            Some(serde_json::json!({
                "recruitment": {
                    "tags": &state.game_data.gacha.gacha_tags,
                    "tagMap": tag_map,
                    "tagNameMap": tag_name_map,
                    "recruitDetail": &state.game_data.gacha.recruit_detail,
                    "recruitPool": &state.game_data.gacha.recruit_pool
                }
            }))
        },
    )
    .await
}

/// GET /static/gacha/pools
/// Returns gacha pool information
pub async fn get_gacha_pools(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let cache_key = "static:gacha:pools".to_string();

    cached_handler(
        &mut state.redis.clone(),
        &cache_key,
        3600,
        &headers,
        || async {
            Some(serde_json::json!({
                "pools": &state.game_data.gacha.gacha_pool_client,
                "newbeePools": &state.game_data.gacha.newbee_gacha_pool_client,
                "total": state.game_data.gacha.gacha_pool_client.len()
            }))
        },
    )
    .await
}

/// GET /static/gacha/tags
/// Returns all gacha tags
pub async fn get_gacha_tags(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let cache_key = "static:gacha:tags".to_string();

    cached_handler(
        &mut state.redis.clone(),
        &cache_key,
        3600,
        &headers,
        || async {
            Some(serde_json::json!({
                "tags": &state.game_data.gacha.gacha_tags
            }))
        },
    )
    .await
}
