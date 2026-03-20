use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::core::gamedata::types::operator::Operator;
use crate::dps::operator_data::OperatorData;
use crate::dps::operator_unit::OperatorParams;

use super::formulas;
use super::operator_unit::{EnemyStats, OperatorUnit};

const FORMULAS_JSON: &str = include_str!("config/operator_formulas.json");

pub fn load_formulas() -> HashMap<String, OperatorFormula> {
    serde_json::from_str(FORMULAS_JSON).expect("Invalid operator_formulas.json")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DpsResult {
    pub skill_dps: f64,
    pub total_damage: f64,
    pub average_dps: f64,
}

#[derive(Debug, Deserialize)]
pub struct SkillFormula {
    #[serde(rename = "type")]
    pub formula_type: String,
    // Indices into skill_parameters/talent_parameters for param resolution
    pub atk_scale_idx: Option<usize>,   // skill_params[idx]
    pub hits_idx: Option<usize>,        // skill_params[idx] for hit count
    pub hits: Option<f64>,              // literal hit count
    pub targets: Option<f64>,           // literal target count
    pub aspd_talent_idx: Option<usize>, // talent1_params[idx] for ASPD
    pub def_ignore_idx: Option<usize>,  // talent1_params[idx] for DEF ignore
    pub res_ignore_idx: Option<usize>,  // talent1_params[idx] for RES ignore
}

#[derive(Debug, Deserialize)]
pub struct OperatorFormula {
    pub name: String,
    pub class_name: String,
    pub available_skills: Vec<i32>,
    pub available_modules: Vec<i32>,
    pub default_skill: i32,
    pub default_potential: i32,
    pub default_module: i32,
    pub skills: HashMap<String, SkillFormula>,
    pub conditionals: Vec<ConditionalConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ConditionalConfig {
    #[serde(rename = "type")]
    pub cond_type: String,
    pub name: String,
    pub default: bool,
    pub skills: Vec<i32>,
    pub modules: Vec<i32>,
}

pub fn calculate_skill_dps(unit: &OperatorUnit, formula: &SkillFormula, enemy: &EnemyStats) -> f64 {
    // Resolve parameters from config indices
    let atk_scale = formula
        .atk_scale_idx
        .and_then(|i| unit.skill_parameters.get(i))
        .copied()
        .unwrap_or(0.0);

    let hits = formula
        .hits_idx
        .and_then(|i| unit.skill_parameters.get(i))
        .copied()
        .or(formula.hits)
        .unwrap_or(1.0);

    let targets = formula.targets.unwrap_or(1.0);

    match formula.formula_type.as_str() {
        "atk_buff" => formulas::atk_buff(unit, enemy, atk_scale, hits),
        "multi_hit" => formulas::multi_hit(unit, enemy, atk_scale, hits, targets),
        "atk_buff_aspd" => {
            let extra_aspd = formula
                .aspd_talent_idx
                .and_then(|i| unit.talent1_parameters.get(i))
                .copied()
                .unwrap_or(0.0);
            formulas::atk_buff_with_aspd(unit, enemy, atk_scale, extra_aspd)
        }
        "true_damage" => formulas::true_damage(unit, atk_scale),
        "aoe" => {
            let target_cap = formula.targets.unwrap_or(unit.targets as f64);
            formulas::aoe(unit, enemy, atk_scale, target_cap)
        }
        "def_ignore" => {
            let def_ignore = formula
                .def_ignore_idx
                .and_then(|i| unit.talent1_parameters.get(i))
                .copied()
                .unwrap_or(0.0);
            let res_ignore = formula
                .res_ignore_idx
                .and_then(|i| unit.talent1_parameters.get(i))
                .copied()
                .unwrap_or(0.0);
            formulas::damage_with_ignore(unit, enemy, atk_scale, def_ignore, res_ignore)
        }
        "res_ignore" => {
            let res_ignore = formula
                .res_ignore_idx
                .and_then(|i| unit.talent1_parameters.get(i))
                .copied()
                .unwrap_or(0.0);
            formulas::damage_with_ignore(unit, enemy, atk_scale, 0.0, res_ignore)
        }
        "custom" => super::custom::dispatch(unit, enemy),
        _ => unit.normal_attack(enemy, None, None, None),
    }
}

static FORMULAS: LazyLock<HashMap<String, OperatorFormula>> = LazyLock::new(load_formulas);

pub fn calculate_dps(
    operator: &Operator,
    params: OperatorParams,
    enemy: &EnemyStats,
) -> Option<DpsResult> {
    let op_id = operator.id.as_deref()?;
    let formula = FORMULAS.get(op_id)?;

    // Create OperatorData from game data
    let data = OperatorData::new(operator.clone());

    // Create OperatorUnit with resolved stats
    let unit = OperatorUnit::new(
        data,
        params,
        formula.default_skill,
        formula.default_potential,
        formula.default_module,
        formula.available_skills.clone(),
    );

    // Apply shreds to enemy
    let shredded = formulas::apply_shreds(enemy, &unit.shreds);

    // Get the skill formula for current skill index
    let skill_key = unit.skill_index.to_string();
    let skill_formula = formula.skills.get(&skill_key)?;

    // Calculate skill DPS
    let skill_dps = calculate_skill_dps(&unit, skill_formula, &shredded);

    // Apply fragile (always last)
    let skill_dps = skill_dps * (1.0 + unit.buff_fragile);

    // Total damage
    let total_damage = if unit.skill_duration > 0.0 {
        skill_dps * unit.skill_duration
    } else {
        skill_dps
    };

    // Average DPS (cycle including downtime)
    let average_dps = if unit.skill_duration > 0.0 && unit.skill_cost > 0 {
        // Calculate off-skill DPS (basic attack)
        let off_skill_dps =
            unit.normal_attack(&shredded, None, None, None) * (1.0 + unit.buff_fragile);
        let sp_time = unit.skill_cost as f64 / (1.0 + unit.sp_boost as f64);
        let cycle_dmg = skill_dps * unit.skill_duration + off_skill_dps * sp_time;
        cycle_dmg / (unit.skill_duration + sp_time)
    } else {
        skill_dps
    };

    Some(DpsResult {
        skill_dps,
        total_damage,
        average_dps,
    })
}
