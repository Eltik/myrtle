//! List available operators for DPS calculation

use axum::{Json, extract::State};
use serde::Serialize;

use crate::app::state::AppState;
use crate::core::dps_calculator::operators::{get_operator_metadata, get_supported_operator_names};

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

            // Get operator metadata (available skills, modules, defaults)
            let metadata = get_operator_metadata(&normalized_name);
            let (
                available_skills,
                available_modules,
                default_skill_index,
                default_potential,
                default_module_index,
            ) = match metadata {
                Some(m) => (
                    m.available_skills,
                    m.available_modules,
                    m.default_skill_index,
                    m.default_potential,
                    m.default_module_index,
                ),
                None => (vec![], vec![], 0, 1, 0),
            };

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

/// Normalize operator name to match calculator naming convention
fn normalize_operator_name(name: &str) -> String {
    let name = match name {
        n if n.contains("Alter") || n.ends_with(" the") => n.split(" the ").next().unwrap_or(n),
        _ => name,
    };

    let mut result = String::new();
    let mut capitalize_next = true;

    for c in name.chars() {
        if c.is_alphanumeric() {
            if capitalize_next {
                result.push(c.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        } else if c == ' ' || c == '-' {
            capitalize_next = true;
        }
    }

    result
}
