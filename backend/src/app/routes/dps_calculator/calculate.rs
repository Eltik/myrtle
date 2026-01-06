//! DPS calculation endpoint handler

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::app::{error::ApiError, state::AppState};
use crate::core::dps_calculator::{
    operator_data::OperatorData,
    operator_unit::{
        EnemyStats, OperatorBaseBuffs, OperatorBuffs, OperatorConditionals, OperatorParams,
        OperatorShred,
    },
    operators::create_operator,
};

/// Request body for DPS calculation
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DpsCalculateRequest {
    /// Operator ID (e.g., "char_017_huang" for Blaze)
    pub operator_id: String,

    /// Operator configuration parameters
    #[serde(default)]
    pub params: DpsParams,

    /// Enemy stats to calculate against
    #[serde(default)]
    pub enemy: EnemyRequest,

    /// Optional: Calculate DPS across a range of defense/resistance values
    #[serde(default)]
    pub range: Option<DpsRangeRequest>,
}

/// Operator configuration parameters
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DpsParams {
    /// Potential rank (1-6), defaults to 6
    pub potential: Option<i32>,

    /// Elite/Promotion level (0-2), defaults to max for rarity
    pub promotion: Option<i32>,

    /// Operator level, defaults to max for elite
    pub level: Option<i32>,

    /// Trust level (0-100), defaults to 100
    pub trust: Option<i32>,

    /// Skill index (0=basic attack, 1=S1, 2=S2, 3=S3), defaults to operator's best skill
    pub skill_index: Option<i32>,

    /// Mastery level (0-3), defaults to 3 for skill calculation
    pub mastery_level: Option<i32>,

    /// Module index (0=none, 1-3 for different modules), defaults to operator's default
    pub module_index: Option<i32>,

    /// Module level (1-3), defaults to 3
    pub module_level: Option<i32>,

    /// External buffs from allies
    #[serde(default)]
    pub buffs: BuffsRequest,

    /// Base ATK buffs (applied before other modifiers)
    #[serde(default)]
    pub base_buffs: BaseBuffsRequest,

    /// SP regeneration boost per second
    pub sp_boost: Option<f32>,

    /// Number of targets the operator is hitting
    pub targets: Option<i32>,

    /// Conditional damage toggles
    #[serde(default)]
    pub conditionals: ConditionalsRequest,

    /// Enable/disable all conditionals at once
    pub all_cond: Option<bool>,

    /// Defense/resistance shred from allies
    #[serde(default)]
    pub shred: ShredRequest,
}

/// External buff parameters
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuffsRequest {
    /// ATK buff as decimal (e.g., 0.4 = +40% ATK)
    pub atk: Option<f32>,
    /// Flat ATK bonus
    pub flat_atk: Option<i32>,
    /// ASPD buff (e.g., 30 = +30 ASPD)
    pub aspd: Option<i32>,
    /// Fragile debuff as decimal (e.g., 0.3 = +30% damage taken)
    pub fragile: Option<f32>,
}

/// Base ATK buffs (applied to base stat before skill multipliers)
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseBuffsRequest {
    /// Base ATK multiplier (e.g., 1.2 = +20% base ATK)
    pub atk: Option<f32>,
    /// Flat base ATK bonus
    pub flat_atk: Option<i32>,
}

/// Conditional damage toggles
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalsRequest {
    /// Enable trait damage bonuses
    pub trait_damage: Option<bool>,
    /// Enable talent 1 damage bonuses
    pub talent_damage: Option<bool>,
    /// Enable talent 2 damage bonuses
    pub talent2_damage: Option<bool>,
    /// Enable skill damage bonuses
    pub skill_damage: Option<bool>,
    /// Enable module damage bonuses
    pub module_damage: Option<bool>,
}

/// Defense/resistance shred parameters
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShredRequest {
    /// DEF shred as percentage (e.g., 40 = -40% DEF)
    pub def: Option<i32>,
    /// Flat DEF reduction
    pub def_flat: Option<i32>,
    /// RES shred as percentage (e.g., 20 = -20 RES)
    pub res: Option<i32>,
    /// Flat RES reduction
    pub res_flat: Option<i32>,
}

/// Enemy stats for DPS calculation
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnemyRequest {
    /// Enemy defense value
    #[serde(default)]
    pub defense: f64,
    /// Enemy resistance value (0-100+)
    #[serde(default)]
    pub res: f64,
}

/// Range parameters for batch DPS calculation
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DpsRangeRequest {
    /// Minimum defense value (default 0)
    #[serde(default)]
    pub min_def: i32,
    /// Maximum defense value (default 3000)
    #[serde(default = "default_max_def")]
    pub max_def: i32,
    /// Defense step size (default 100)
    #[serde(default = "default_step")]
    pub def_step: i32,
    /// Minimum resistance value (default 0)
    #[serde(default)]
    pub min_res: i32,
    /// Maximum resistance value (default 120)
    #[serde(default = "default_max_res")]
    pub max_res: i32,
    /// Resistance step size (default 10)
    #[serde(default = "default_res_step")]
    pub res_step: i32,
}

fn default_max_def() -> i32 {
    3000
}
fn default_step() -> i32 {
    100
}
fn default_max_res() -> i32 {
    120
}
fn default_res_step() -> i32 {
    10
}

/// DPS calculation response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DpsCalculateResponse {
    /// The calculated DPS value(s)
    pub dps: DpsResult,

    /// Operator metadata after applying configuration
    pub operator: OperatorMetadata,
}

/// DPS result - either a single value or a range
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum DpsResult {
    /// Single DPS calculation
    Single(SingleDps),
    /// DPS across defense/resistance ranges
    Range(RangeDps),
}

/// Single DPS calculation result
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleDps {
    /// DPS while skill is active
    pub skill_dps: f64,
    /// Total damage during skill duration
    pub total_damage: f64,
    /// Average DPS over full skill cycle (including downtime)
    pub average_dps: f64,
}

/// DPS across a range of defense/resistance values
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RangeDps {
    /// DPS values at different defense values (fixed resistance)
    pub by_defense: Vec<DpsPoint>,
    /// DPS values at different resistance values (fixed defense)
    pub by_resistance: Vec<DpsPoint>,
}

/// A single point in the DPS curve
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DpsPoint {
    /// Defense or resistance value
    pub value: i32,
    /// DPS at this value
    pub dps: f64,
}

/// Operator metadata returned with DPS calculation
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorMetadata {
    /// Operator ID
    pub id: String,
    /// Operator name
    pub name: String,
    /// Rarity (1-6)
    pub rarity: i32,
    /// Elite level (0-2)
    pub elite: i32,
    /// Current level
    pub level: i32,
    /// Potential rank (1-6)
    pub potential: i32,
    /// Trust level (0-100)
    pub trust: i32,
    /// Active skill index
    pub skill_index: i32,
    /// Skill level (1-10)
    pub skill_level: i32,
    /// Module index (0 = none)
    pub module_index: i32,
    /// Module level (1-3)
    pub module_level: i32,
    /// Final calculated ATK stat
    pub atk: f64,
    /// Attack interval in seconds
    pub attack_interval: f32,
    /// Attack speed percentage (100 = base)
    pub attack_speed: f64,
    /// Whether operator deals physical damage
    pub is_physical: bool,
    /// Skill duration in seconds
    pub skill_duration: f64,
    /// Skill SP cost
    pub skill_cost: i32,
    /// Current skill parameters
    pub skill_parameters: Vec<f64>,
    /// Talent 1 parameters
    pub talent1_parameters: Vec<f64>,
    /// Talent 2 parameters (if available)
    pub talent2_parameters: Vec<f64>,
}

/// POST /dps-calculator
///
/// Calculate DPS for an operator with given configuration against enemy stats.
pub async fn calculate_dps(
    State(state): State<AppState>,
    Json(request): Json<DpsCalculateRequest>,
) -> Result<Json<DpsCalculateResponse>, ApiError> {
    // Find the operator in game data
    let operator = state
        .game_data
        .operators
        .get(&request.operator_id)
        .ok_or_else(|| {
            ApiError::NotFound(format!("Operator not found: {}", request.operator_id))
        })?;

    // Convert operator name to PascalCase for create_operator lookup
    let operator_name = normalize_operator_name(&operator.name);

    // Create OperatorData from the game data
    let operator_data = OperatorData::new(operator.clone());

    // Convert request params to OperatorParams
    let params = convert_params(&request.params);

    // Create the operator calculator
    let mut calculator =
        create_operator(&operator_name, operator_data, params).ok_or_else(|| {
            ApiError::BadRequest(format!(
                "DPS calculator not implemented for operator: {} ({})",
                operator.name, operator_name
            ))
        })?;

    // Get the unit for metadata
    let unit = calculator.unit();

    // Build operator metadata
    let operator_metadata = OperatorMetadata {
        id: request.operator_id.clone(),
        name: operator.name.clone(),
        rarity: unit.rarity,
        elite: unit.elite,
        level: unit.level,
        potential: unit.potential,
        trust: unit.trust,
        skill_index: unit.skill_index,
        skill_level: unit.skill_level,
        module_index: unit.module_index,
        module_level: unit.module_level,
        atk: unit.atk,
        attack_interval: unit.attack_interval,
        attack_speed: unit.attack_speed,
        is_physical: unit.is_physical,
        skill_duration: unit.skill_duration,
        skill_cost: unit.skill_cost,
        skill_parameters: unit.skill_parameters.clone(),
        talent1_parameters: unit.talent1_parameters.clone(),
        talent2_parameters: unit.talent2_parameters.clone(),
    };

    // Calculate DPS
    let dps = if let Some(range) = request.range {
        // Calculate across a range
        let mut by_defense = Vec::new();
        let mut by_resistance = Vec::new();

        // Fixed resistance, varying defense
        let fixed_res = request.enemy.res;
        let mut def = range.min_def;
        while def <= range.max_def {
            let enemy = EnemyStats {
                defense: def as f64,
                res: fixed_res,
            };
            let dps_value = calculator.skill_dps(&enemy);
            by_defense.push(DpsPoint {
                value: def,
                dps: dps_value,
            });
            def += range.def_step;
        }

        // Fixed defense, varying resistance
        let fixed_def = request.enemy.defense;
        let mut res = range.min_res;
        while res <= range.max_res {
            let enemy = EnemyStats {
                defense: fixed_def,
                res: res as f64,
            };
            let dps_value = calculator.skill_dps(&enemy);
            by_resistance.push(DpsPoint {
                value: res,
                dps: dps_value,
            });
            res += range.res_step;
        }

        DpsResult::Range(RangeDps {
            by_defense,
            by_resistance,
        })
    } else {
        // Single calculation
        let enemy = EnemyStats {
            defense: request.enemy.defense,
            res: request.enemy.res,
        };

        let skill_dps = calculator.skill_dps(&enemy);
        let total_damage = calculator.unit().total_dmg(&enemy);
        let average_dps = calculator.unit_mut().average_dps(&enemy);

        DpsResult::Single(SingleDps {
            skill_dps,
            total_damage,
            average_dps,
        })
    };

    Ok(Json(DpsCalculateResponse {
        dps,
        operator: operator_metadata,
    }))
}

/// Convert request params to OperatorParams
fn convert_params(params: &DpsParams) -> OperatorParams {
    OperatorParams {
        potential: params.potential,
        promotion: params.promotion,
        level: params.level,
        trust: params.trust,
        skill_index: params.skill_index,
        mastery_level: params.mastery_level,
        module_index: params.module_index,
        module_level: params.module_level,
        buffs: OperatorBuffs {
            atk: params.buffs.atk,
            flat_atk: params.buffs.flat_atk,
            aspd: params.buffs.aspd,
            fragile: params.buffs.fragile,
        },
        base_buffs: OperatorBaseBuffs {
            atk: params.base_buffs.atk,
            flat_atk: params.base_buffs.flat_atk,
        },
        sp_boost: params.sp_boost,
        targets: params.targets,
        conditionals: Some(OperatorConditionals {
            trait_damage: params.conditionals.trait_damage,
            talent_damage: params.conditionals.talent_damage,
            talent2_damage: params.conditionals.talent2_damage,
            skill_damage: params.conditionals.skill_damage,
            module_damage: params.conditionals.module_damage,
        }),
        all_cond: params.all_cond,
        shred: Some(OperatorShred {
            def: params.shred.def,
            def_flat: params.shred.def_flat,
            res: params.shred.res,
            res_flat: params.shred.res_flat,
        }),
        ..Default::default()
    }
}

/// Normalize operator name to PascalCase format expected by create_operator
///
/// Examples:
/// - "Blue Poison" -> "BluePoison"
/// - "Ch'en" -> "Chen"
/// - "Exusiai the Sankta" -> "Exusiai"
/// - "SilverAsh" -> "SilverAsh"
/// - "12F" -> "12F" (numbers preserved)
fn normalize_operator_name(name: &str) -> String {
    // Handle special cases with alternate forms
    let name = match name {
        // Known alternates with different naming conventions
        n if n.contains("Alter") || n.ends_with(" the") => {
            // Handle "Operator the Title" format - take just the name
            n.split(" the ").next().unwrap_or(n)
        }
        _ => name,
    };

    // Remove special characters and convert to PascalCase
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
            // Space and hyphen: capitalize next character
            capitalize_next = true;
        }
        // Apostrophes and other special characters are just removed (don't capitalize next)
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_operator_name() {
        assert_eq!(normalize_operator_name("Blue Poison"), "BluePoison");
        assert_eq!(normalize_operator_name("Ch'en"), "Chen");
        assert_eq!(normalize_operator_name("SilverAsh"), "SilverAsh");
        assert_eq!(normalize_operator_name("12F"), "12F");
        assert_eq!(normalize_operator_name("Projekt Red"), "ProjektRed");
    }
}
