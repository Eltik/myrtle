use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use regex::Regex;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use crate::app::routes::static_data::{
    fields::{FieldsParam, filter_fields},
    handler::cached_handler,
};
use crate::app::state::AppState;
use crate::core::local::types::operator::{
    Operator, OperatorPosition, OperatorProfession, OperatorRarity,
};

#[derive(Deserialize)]
pub struct GachaQuery {
    #[serde(flatten)]
    fields: FieldsParam,
}

#[derive(Deserialize)]
pub struct CalculateQuery {
    pub recruitment: Option<String>,
}

/// Name substitutions for operators with localization differences
fn get_name_substitutions() -> HashMap<String, String> {
    let mut subs = HashMap::new();
    subs.insert("justice knight".to_string(), "'justice knight'".to_string());
    subs.insert("サーマル-ex".to_string(), "thrm-ex".to_string());
    subs.insert("샤미르".to_string(), "샤마르".to_string());
    subs
}

/// Parse the recruitDetail string to extract recruitable operator names
/// Returns a HashSet of lowercase operator names that are in the recruitment pool
fn parse_recruit_pool(
    recruit_detail: &str,
    operators: &HashMap<String, Operator>,
) -> HashSet<String> {
    let mut recruit_names: HashSet<String> = HashSet::new();

    // Build a name -> operator id map (lowercase names)
    let name_to_id: HashMap<String, String> = operators
        .iter()
        .map(|(id, op)| (op.name.to_lowercase(), id.clone()))
        .collect();

    let substitutions = get_name_substitutions();

    // Pattern 1: <@rc.eml>Name</> - recruitment-only operators (robots, starters)
    let re_eml = Regex::new(r"(?i)<@rc\.eml>([^<]+)</>").unwrap();

    // Pattern 2: / Name - operators separated by slashes
    // We split by common separators and filter out non-operator content
    let re_slash = Regex::new(r"(?i)(?:/\s*|\n\s*|\\n\s*)([^\r\n/★<>]+)").unwrap();

    // Extract recruitment-only operators (Pattern 1)
    for cap in re_eml.captures_iter(recruit_detail) {
        if let Some(m) = cap.get(1) {
            let mut name = m.as_str().trim().to_lowercase();

            // Apply substitutions
            if let Some(sub) = substitutions.get(&name) {
                name = sub.clone();
            }

            if name_to_id.contains_key(&name) {
                recruit_names.insert(name);
            }
        }
    }

    // Extract regular operators (Pattern 2)
    for cap in re_slash.captures_iter(recruit_detail) {
        if let Some(m) = cap.get(1) {
            let mut name = m.as_str().trim().to_lowercase();

            // Skip empty names, star ratings, and dashes
            if name.is_empty()
                || name.starts_with('-')
                || name.ends_with('-')
                || name.chars().all(|c| c == '★' || c.is_whitespace())
            {
                continue;
            }

            // Apply substitutions
            if let Some(sub) = substitutions.get(&name) {
                name = sub.clone();
            }

            if name_to_id.contains_key(&name) {
                recruit_names.insert(name);
            }
        }
    }

    recruit_names
}

/// Get the set of operator IDs that are in the recruitment pool
fn get_recruitable_operator_ids(
    recruit_detail: &str,
    operators: &HashMap<String, Operator>,
) -> HashSet<String> {
    let recruit_names = parse_recruit_pool(recruit_detail, operators);

    // Map names back to IDs
    operators
        .iter()
        .filter(|(_, op)| recruit_names.contains(&op.name.to_lowercase()))
        .map(|(id, _)| id.clone())
        .collect()
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

            // Get the recruitment pool - only operators listed in recruitDetail
            let recruitable_ids = get_recruitable_operator_ids(
                &state.game_data.gacha.recruit_detail,
                &state.game_data.operators,
            );

            // Check if Top Operator tag is selected (allows 6-star operators)
            let has_top_operator = tag_ids.contains(&11);

            // Filter operators that:
            // 1. Are in the recruitment pool
            // 2. Match ALL selected tags
            // 3. Are <= 5 star unless Top Operator tag is selected
            let matching_operators: Vec<_> = state
                .game_data
                .operators
                .iter()
                .filter(|(id, op)| {
                    // Must be in recruitment pool
                    if !recruitable_ids.contains(*id) {
                        return false;
                    }

                    // Unless Top Operator tag is selected, exclude 6-star operators
                    if !has_top_operator && op.rarity == OperatorRarity::SixStar {
                        return false;
                    }

                    // Must match all selected tags
                    selected_tags.iter().all(|tag| {
                        match tag.tag_id {
                            // Position tags
                            9 => op.position == OperatorPosition::Melee,
                            10 => op.position == OperatorPosition::Ranged,
                            // Class tags
                            1 => op.profession == OperatorProfession::Guard,
                            2 => op.profession == OperatorProfession::Sniper,
                            3 => op.profession == OperatorProfession::Defender,
                            4 => op.profession == OperatorProfession::Medic,
                            5 => op.profession == OperatorProfession::Supporter,
                            6 => op.profession == OperatorProfession::Caster,
                            7 => op.profession == OperatorProfession::Specialist,
                            8 => op.profession == OperatorProfession::Vanguard,
                            // Rarity tags
                            11 => op.rarity == OperatorRarity::SixStar,
                            14 => op.rarity == OperatorRarity::FiveStar,
                            17 => op.rarity == OperatorRarity::TwoStar,
                            28 => op.rarity == OperatorRarity::OneStar,
                            // Affix tags - check tagList
                            _ => op.tag_list.contains(&tag.tag_name),
                        }
                    })
                })
                .map(|(_, op)| op)
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

/// GET /static/gacha/recruitable
/// Returns ALL operators in the recruitment pool with their tag data
/// This endpoint is designed for client-side recruitment calculation
pub async fn get_recruitable_operators(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let cache_key = "static:gacha:recruitable".to_string();

    cached_handler(
        &mut state.redis.clone(),
        &cache_key,
        3600,
        &headers,
        || async {
            // Get the recruitment pool - only operators listed in recruitDetail
            let recruitable_ids = get_recruitable_operator_ids(
                &state.game_data.gacha.recruit_detail,
                &state.game_data.operators,
            );

            // Build list of recruitable operators with minimal data needed for client-side calculation
            let recruitable_operators: Vec<_> = state
                .game_data
                .operators
                .iter()
                .filter(|(id, _)| recruitable_ids.contains(*id))
                .map(|(_, op)| {
                    serde_json::json!({
                        "id": op.id,
                        "name": op.name,
                        "rarity": op.rarity,
                        "profession": op.profession,
                        "position": op.position,
                        "tagList": op.tag_list
                    })
                })
                .collect();

            Some(serde_json::json!({
                "operators": recruitable_operators,
                "total": recruitable_operators.len()
            }))
        },
    )
    .await
}
