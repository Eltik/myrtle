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
#[serde(default)]
pub struct SkillFormula {
    #[serde(rename = "type")]
    pub formula_type: String,
    // Indices into skill_parameters/talent_parameters for param resolution
    pub atk_scale_idx: Option<usize>, // skill_params[idx] for ATK scale
    pub hits_idx: Option<usize>,      // skill_params[idx] for hit count
    pub hits: Option<f64>,            // literal hit count
    pub targets: Option<f64>,         // literal target count
    pub aspd_talent_idx: Option<usize>, // talent1_params[idx] for ASPD bonus
    pub def_ignore_idx: Option<usize>, // talent1_params[idx] for DEF ignore
    pub res_ignore_idx: Option<usize>, // talent1_params[idx] for RES ignore
    // Extended fields for generic_dps
    pub scale_on_skill_only: Option<bool>, // only apply atk_scale when skill is active
    pub cycle_average: Option<bool>,       // average skill + basic attack over SP cost
    pub talent_atk_idx: Option<usize>,     // talent1_params[idx] for ATK buff
    pub module_atk_scale: Option<f64>,     // module multiplier on final ATK (e.g. 1.1)
    pub override_interval: Option<f64>,    // skill overrides attack interval
    pub damage_type: Option<String>,       // "arts" or "physical" to override is_physical
}

impl Default for SkillFormula {
    fn default() -> Self {
        Self {
            formula_type: "custom".to_string(),
            atk_scale_idx: None,
            hits_idx: None,
            hits: None,
            targets: None,
            aspd_talent_idx: None,
            def_ignore_idx: None,
            res_ignore_idx: None,
            scale_on_skill_only: None,
            cycle_average: None,
            talent_atk_idx: None,
            module_atk_scale: None,
            override_interval: None,
            damage_type: None,
        }
    }
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

pub fn calculate_skill_dps(
    unit: &OperatorUnit,
    _formula: &SkillFormula,
    enemy: &EnemyStats,
) -> f64 {
    super::custom::dispatch(unit, enemy).unwrap_or(0.0)
}

static FORMULAS: LazyLock<HashMap<String, OperatorFormula>> = LazyLock::new(load_formulas);

pub fn get_formula(op_id: &str) -> Option<&OperatorFormula> {
    FORMULAS.get(op_id)
}

pub fn supported_operators() -> &'static HashMap<String, OperatorFormula> {
    &FORMULAS
}

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
        formula.available_modules.clone(),
    );

    // Apply shreds to enemy
    let shredded = formulas::apply_shreds(enemy, &unit.shreds);

    // Get the skill formula for current skill index
    let skill_key = unit.skill_index.to_string();
    let skill_formula = formula.skills.get(&skill_key)?;

    // Calculate skill DPS
    // Save and zero out buff_fragile before calling skill_dps — operators should not
    // see external fragile (matches Python behavior where buff_fragile=0 during skill_dps).
    // Fragile is applied externally after the call.
    let external_fragile = unit.buff_fragile;
    let mut unit = unit;
    unit.buff_fragile = 0.0;
    let skill_dps = calculate_skill_dps(&unit, skill_formula, &shredded);

    // Apply fragile externally (always last)
    let skill_dps = skill_dps * (1.0 + external_fragile);

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
