//! List available operators for DPS calculation

use axum::{Json, extract::State};
use serde::Serialize;

use crate::app::state::AppState;
use crate::core::dps_calculator::operators::{get_operator_metadata, get_supported_operator_names};

use super::utils::normalize_operator_name;

/// Response for listing available DPS calculator operators
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOperatorsResponse {
    /// Total count of supported operators
    pub count: usize,
    /// List of operators with their IDs and names
    pub operators: Vec<SupportedOperator>,
}

/// A supported operator for DPS calculations
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportedOperator {
    /// Operator ID (e.g., "char_017_huang")
    pub id: String,
    /// Operator display name (e.g., "Blaze")
    pub name: String,
    /// Normalized name used by DPS calculator (e.g., "Blaze")
    pub calculator_name: String,
    /// Operator rarity (1-6 stars)
    pub rarity: i32,
    /// Operator profession/class
    pub profession: String,
    /// Available skill indices (e.g., [1, 2] or [1, 2, 3])
    pub available_skills: Vec<i32>,
    /// Available module indices (e.g., [1, 2] or [])
    pub available_modules: Vec<i32>,
    /// Default skill index for this operator
    pub default_skill_index: i32,
    /// Default potential for this operator
    pub default_potential: i32,
    /// Default module index for this operator
    pub default_module_index: i32,
    /// Maximum promotion/elite level (0, 1, or 2) based on rarity
    pub max_promotion: i32,
    /// Potential rank descriptions (for UI tooltips)
    pub potential_ranks: Vec<PotentialRankInfo>,
    /// Available conditionals for this operator
    pub conditionals: Vec<ConditionalInfoResponse>,
}

/// Lightweight potential rank info for API response
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PotentialRankInfo {
    /// Description of what this potential does (e.g., "Deploy Cost -1")
    pub description: String,
}

/// Conditional info for API response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalInfoResponse {
    /// Conditional type: "trait", "talent", "talent2", "skill", "module"
    pub conditional_type: String,
    /// Display name (e.g., "aerialTarget", "maxStacks")
    pub name: String,
    /// If true, this label applies when conditional is disabled
    pub inverted: bool,
    /// Which skills this applies to (empty = all)
    pub applicable_skills: Vec<i32>,
    /// Which modules this applies to (empty = all)
    pub applicable_modules: Vec<i32>,
    /// Minimum elite level required
    pub min_elite: i32,
    /// Minimum module level required
    pub min_module_level: i32,
}

/// GET /dps-calculator/operators
///
/// Returns a list of all operators that have DPS calculator implementations.
pub async fn list_operators(State(state): State<AppState>) -> Json<ListOperatorsResponse> {
    let supported_names = get_supported_operator_names();
    let mut operators = Vec::new();

    // Match supported calculator names against game data operators
    for (id, operator) in &state.game_data.operators {
        let normalized_name = normalize_operator_name(&operator.name);

        if supported_names.contains(&normalized_name.as_str()) {
            let rarity = rarity_to_number(&operator.rarity);
            let max_promotion = rarity_to_max_promotion(rarity);

            // Get operator metadata (available skills, modules, defaults, conditionals)
            let metadata = get_operator_metadata(&normalized_name);
            let (
                available_skills,
                available_modules,
                default_skill_index,
                default_potential,
                default_module_index,
                conditionals,
            ) = match metadata {
                Some(m) => {
                    let conditionals: Vec<ConditionalInfoResponse> = m
                        .conditionals
                        .iter()
                        .map(|c| ConditionalInfoResponse {
                            conditional_type: c.conditional_type.as_str().to_string(),
                            name: c.name.clone(),
                            inverted: c.inverted,
                            applicable_skills: c.availability.skills.clone(),
                            applicable_modules: c.availability.modules.clone(),
                            min_elite: c.availability.min_elite,
                            min_module_level: c.availability.min_module_level,
                        })
                        .collect();
                    (
                        m.available_skills,
                        m.available_modules,
                        m.default_skill_index,
                        m.default_potential,
                        m.default_module_index,
                        conditionals,
                    )
                }
                None => (vec![], vec![], 0, 1, 0, vec![]),
            };

            // Convert potential ranks to lightweight format
            let potential_ranks: Vec<PotentialRankInfo> = operator
                .potential_ranks
                .iter()
                .map(|rank| PotentialRankInfo {
                    description: rank.description.clone(),
                })
                .collect();

            operators.push(SupportedOperator {
                id: id.clone(),
                name: operator.name.clone(),
                calculator_name: normalized_name,
                rarity,
                profession: format!("{:?}", operator.profession),
                available_skills,
                available_modules,
                default_skill_index,
                default_potential,
                default_module_index,
                max_promotion,
                potential_ranks,
                conditionals,
            });
        }
    }

    // Sort by name for consistent output
    operators.sort_by(|a, b| a.name.cmp(&b.name));

    Json(ListOperatorsResponse {
        count: operators.len(),
        operators,
    })
}

/// Convert rarity enum to number
fn rarity_to_number(rarity: &crate::core::local::types::operator::OperatorRarity) -> i32 {
    use crate::core::local::types::operator::OperatorRarity;
    match rarity {
        OperatorRarity::OneStar => 1,
        OperatorRarity::TwoStar => 2,
        OperatorRarity::ThreeStar => 3,
        OperatorRarity::FourStar => 4,
        OperatorRarity::FiveStar => 5,
        OperatorRarity::SixStar => 6,
    }
}

/// Convert rarity to maximum promotion level
/// 1-2 star: E0 max, 3 star: E1 max, 4-6 star: E2 max
fn rarity_to_max_promotion(rarity: i32) -> i32 {
    match rarity {
        1 | 2 => 0,
        3 => 1,
        4..=6 => 2,
        _ => 0,
    }
}
