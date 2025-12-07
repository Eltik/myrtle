use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use serde::Deserialize;
use std::collections::HashSet;

use crate::app::routes::static_data::{
    fields::{FieldsParam, filter_fields},
    handler::cached_handler,
};
use crate::app::state::AppState;
use crate::core::local::types::operator::{OperatorPosition, OperatorProfession, OperatorRarity};

#[derive(Deserialize)]
pub struct GachaQuery {
    #[serde(flatten)]
    fields: FieldsParam,
}

#[derive(Deserialize)]
pub struct CalculateQuery {
    pub recruitment: Option<String>,
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

/// GET /static/gacha/calculate?recruitment=1,2,3
/// Calculates recruitment results based on selected tag IDs
pub async fn calculate_recruitment(
    State(state): State<AppState>,
    Query(params): Query<CalculateQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let recruitment = params.recruitment.ok_or(StatusCode::BAD_REQUEST)?;

    let tag_ids: HashSet<i32> = recruitment
        .split(',')
        .filter_map(|s| s.trim().parse::<i32>().ok())
        .collect();

    if tag_ids.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let cache_key = format!("static:gacha:calculate:{}", recruitment.replace(',', "_"));

    cached_handler(
        &mut state.redis.clone(),
        &cache_key,
        3600,
        &headers,
        || async {
            // Build tag map
            let tag_map: std::collections::HashMap<i32, _> = state
                .game_data
                .gacha
                .gacha_tags
                .iter()
                .map(|tag| (tag.tag_id, tag.clone()))
                .collect();

            // Get selected tags
            let selected_tags: Vec<_> = tag_ids
                .iter()
                .filter_map(|id| tag_map.get(id).cloned())
                .collect();

            if selected_tags.is_empty() {
                return Some(serde_json::json!({
                    "error": "No valid tags found",
                    "recruitment": []
                }));
            }

            // Filter operators that match ALL selected tags
            let matching_operators: Vec<_> = state
                .game_data
                .operators
                .values()
                .filter(|op| {
                    selected_tags.iter().all(|tag| {
                        match tag.tag_id {
                            // Position tags
                            9 => op.position == OperatorPosition::Melee,
                            10 => op.position == OperatorPosition::Ranged,
                            // Class tags
                            1 => op.profession == OperatorProfession::Guard, // WARRIOR -> Guard
                            2 => op.profession == OperatorProfession::Sniper,
                            3 => op.profession == OperatorProfession::Defender, // TANK -> Defender
                            4 => op.profession == OperatorProfession::Medic,
                            5 => op.profession == OperatorProfession::Supporter, // SUPPORT -> Supporter
                            6 => op.profession == OperatorProfession::Caster,
                            7 => op.profession == OperatorProfession::Specialist, // SPECIAL -> Specialist
                            8 => op.profession == OperatorProfession::Vanguard, // PIONEER -> Vanguard
                            // Rarity tags
                            11 => op.rarity == OperatorRarity::SixStar, // Top Operator
                            14 => op.rarity == OperatorRarity::FiveStar, // Senior Operator
                            17 => op.rarity == OperatorRarity::TwoStar, // Starter
                            28 => op.rarity == OperatorRarity::OneStar, // Robot
                            // Affix tags - check tagList
                            _ => op.tag_list.contains(&tag.tag_name),
                        }
                    })
                })
                .collect();

            Some(serde_json::json!({
                "recruitment": [{
                    "label": selected_tags.iter().map(|t| &t.tag_name).collect::<Vec<_>>(),
                    "operators": matching_operators
                }]
            }))
        },
    )
    .await
}

/// GET /static/gacha/calculate/{recruitment}
/// Path-based version
pub async fn calculate_recruitment_by_path(
    State(state): State<AppState>,
    Path(recruitment): Path<String>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    calculate_recruitment(
        State(state),
        Query(CalculateQuery {
            recruitment: Some(recruitment),
        }),
        headers,
    )
    .await
}
