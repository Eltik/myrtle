use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::core::gamedata::types::operator::Operator;
use crate::dps::operator_data::OperatorData;
use crate::dps::operator_unit::OperatorParams;

use super::custom::apply_init;
use super::custom::dispatch;
use super::custom::dispatch_hps;
use super::formulas::apply_shreds;
use super::operator_unit::{EnemyStats, OperatorUnit};
use utoipa::ToSchema;

const FORMULAS_JSON: &str = include_str!("config/operator_formulas.json");
const HEAL_FORMULAS_JSON: &str = include_str!("config/heal_formulas.json");

pub fn load_formulas() -> HashMap<String, OperatorFormula> {
    let mut formulas: HashMap<String, OperatorFormula> =
        serde_json::from_str(FORMULAS_JSON).expect("Invalid operator_formulas.json");
    for formula in formulas.values_mut() {
        normalize_default_skill(formula);
    }
    formulas
}

/// Makes `default_skill` one the table can actually compute.
///
/// The client opens an operator on `default_skill`, so a default with no formula
/// shows no DPS until the user picks another skill. Two shapes reach that today:
/// a default outside `available_skills` (`char_1044_hsgma2` defaults to S2,
/// but only S1 and S3 are transpiled), which moves to the highest available
/// skill; and a table with no skills at all (1 and 2 stars, and operators whose
/// skills were not transpiled), which moves to 0, basic attack, the index the
/// engine already forces for rarity 2 and below. DPS tables only: `calculate_hps`
/// never reads the skill map, so the healer defaults are left as shipped.
fn normalize_default_skill(formula: &mut OperatorFormula) {
    match formula.available_skills.iter().max() {
        None => formula.default_skill = 0,
        Some(&highest) if !formula.available_skills.contains(&formula.default_skill) => {
            formula.default_skill = highest;
        }
        Some(_) => {}
    }
}

/// The skill entry for basic attack (`skill_index` 0), which no table carries.
/// Every entry is the `custom` kind and `calculate_skill_dps` dispatches on the
/// operator id, so a default entry computes exactly what the operator's function
/// does with no skill active.
static BASIC_ATTACK: LazyLock<SkillFormula> = LazyLock::new(SkillFormula::default);

pub fn load_heal_formulas() -> HashMap<String, OperatorFormula> {
    serde_json::from_str(HEAL_FORMULAS_JSON).expect("Invalid heal_formulas.json")
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DpsResult {
    pub skill_dps: f64,
    pub total_damage: f64,
    pub average_dps: f64,
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema)]
pub struct HpsResult {
    /// Heal-per-second while the active skill is up.
    pub skill_hps: f64,
    /// Heal-per-second during the SP-charge phase (0 for burst healers).
    pub base_hps: f64,
    /// Cycle-averaged HPS including charge time.
    pub avg_hps: f64,
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
    dispatch(unit, enemy).unwrap_or(0.0)
}

static FORMULAS: LazyLock<HashMap<String, OperatorFormula>> = LazyLock::new(load_formulas);
static HEAL_FORMULAS: LazyLock<HashMap<String, OperatorFormula>> =
    LazyLock::new(load_heal_formulas);

pub fn get_formula(op_id: &str) -> Option<&OperatorFormula> {
    FORMULAS.get(op_id)
}

pub fn supported_operators() -> &'static HashMap<String, OperatorFormula> {
    &FORMULAS
}

pub fn get_heal_formula(op_id: &str) -> Option<&OperatorFormula> {
    HEAL_FORMULAS.get(op_id)
}

pub fn supported_healers() -> &'static HashMap<String, OperatorFormula> {
    &HEAL_FORMULAS
}

pub fn calculate_dps(
    operator: &Operator,
    params: OperatorParams,
    enemy: &EnemyStats,
) -> Option<DpsResult> {
    let op_id = operator.id.as_deref()?;
    let formula = FORMULAS.get(op_id)?;

    let data = OperatorData::new(operator.clone());

    let mut unit = OperatorUnit::new(
        data,
        params,
        formula.default_skill,
        formula.default_potential,
        formula.default_module,
        formula.available_skills.clone(),
        formula.available_modules.clone(),
    );

    if unit.module_unavailable {
        return None;
    }

    apply_init(&mut unit);

    let shredded = apply_shreds(enemy, &unit.shreds);

    let skill_key = unit.skill_index.to_string();
    let skill_formula = match formula.skills.get(&skill_key) {
        Some(skill_formula) => skill_formula,
        None if unit.skill_index == 0 => &BASIC_ATTACK,
        None => return None,
    };

    // buff_fragile is 0 during skill_dps in the Python reference: the operator never
    // sees external fragile, it is multiplied on after the call.
    let external_fragile = unit.buff_fragile;

    unit.buff_fragile = 0.0;
    let skill_dps = calculate_skill_dps(&unit, skill_formula, &shredded);

    let skill_dps = skill_dps * (1.0 + external_fragile);

    let total_damage = if unit.skill_duration > 0.0 {
        skill_dps * unit.skill_duration
    } else {
        skill_dps
    };

    let average_dps = if unit.skill_duration > 0.0 && unit.skill_cost > 0 {
        let off_skill_dps =
            unit.normal_attack(&shredded, None, None, None) * (1.0 + unit.buff_fragile);
        let sp_time = f64::from(unit.skill_cost) / (1.0 + f64::from(unit.sp_boost));
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

/// Returns `None` for operators without a transpiled HPS implementation.
pub fn calculate_hps(operator: &Operator, params: OperatorParams) -> Option<HpsResult> {
    let op_id = operator.id.as_deref()?;
    let formula = HEAL_FORMULAS.get(op_id)?;

    let data = OperatorData::new(operator.clone());
    let mut unit = OperatorUnit::new(
        data,
        params,
        formula.default_skill,
        formula.default_potential,
        formula.default_module,
        formula.available_skills.clone(),
        formula.available_modules.clone(),
    );

    if unit.module_unavailable {
        return None;
    }

    apply_init_fixups(&mut unit, op_id);

    dispatch_hps(&unit)
}

/// Per-operator init side effects that the transpiler can't derive from the
/// skill formula alone (e.g. values set in the operator's Python `__init__`).
fn apply_init_fixups(unit: &mut OperatorUnit, op_id: &str) {
    // Warfarin S1 heals for a fraction of the target's max HP; default it the
    // way the reference does when no explicit target HP is supplied.
    if op_id == "char_171_bldsk" && unit.skill_index == 1 {
        unit.target_hp = (1000.0 * (f64::from(unit.elite) + 1.0)).max(100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Operators whose `skills` map has no entry for their own `default_skill`.
    fn default_skill_gaps(formulas: &HashMap<String, OperatorFormula>) -> Vec<String> {
        let mut gaps: Vec<String> = formulas
            .iter()
            .filter(|(_, f)| !f.skills.contains_key(&f.default_skill.to_string()))
            .map(|(id, _)| id.clone())
            .collect();
        gaps.sort();
        gaps
    }

    #[test]
    fn bundled_formula_tables_parse() {
        assert_eq!(load_formulas().len(), 260);
        assert_eq!(load_heal_formulas().len(), 61);
        assert_eq!(supported_operators().len(), 260);
        assert_eq!(supported_healers().len(), 61);
    }

    #[test]
    fn every_available_skill_has_a_formula() {
        for (id, f) in supported_operators().iter().chain(supported_healers()) {
            for skill in &f.available_skills {
                assert!(
                    f.skills.contains_key(&skill.to_string()),
                    "{id}: S{skill} listed but has no formula"
                );
            }
        }
    }

    #[test]
    fn every_formula_skill_is_the_custom_kind() {
        for (id, f) in supported_operators().iter().chain(supported_healers()) {
            for (key, skill) in &f.skills {
                assert_eq!(skill.formula_type, "custom", "{id} S{key}");
            }
        }
    }

    #[test]
    fn lookups_hit_and_miss() {
        assert!(get_formula("char_1044_hsgma2").is_some());
        assert!(get_formula("char_does_not_exist").is_none());
        assert!(get_heal_formula("char_473_mberry").is_some());
        assert!(get_heal_formula("char_does_not_exist").is_none());
    }

    #[test]
    fn every_dps_default_skill_is_computable() {
        // The client opens an operator on its default skill. Every default is
        // either a skill with a formula or 0, basic attack, which `calculate_dps`
        // serves without one. Before `normalize_default_skill` these eight had
        // neither and showed no DPS when first opened.
        for (id, f) in supported_operators() {
            assert!(
                f.default_skill == 0 || f.skills.contains_key(&f.default_skill.to_string()),
                "{id}: default S{} has no formula",
                f.default_skill
            );
        }
        let basic_attack_defaults: Vec<String> = default_skill_gaps(supported_operators());
        assert_eq!(
            basic_attack_defaults,
            vec![
                "char_009_12fce",
                "char_286_cast3",
                "char_347_jaksel",
                "char_4000_jnight",
                "char_4077_palico",
                "char_501_durin",
                "char_503_rang",
            ]
        );
        assert_eq!(get_formula("char_1044_hsgma2").map(|f| f.default_skill), Some(3));
        assert_eq!(get_formula("char_347_jaksel").map(|f| f.default_skill), Some(0));
    }

    #[test]
    fn normalize_default_skill_shapes() {
        let mut f: OperatorFormula = serde_json::from_value(serde_json::json!({
            "name": "x", "class_name": "x", "available_skills": [1, 3],
            "available_modules": [], "default_skill": 2, "default_potential": 1,
            "default_module": 0, "skills": {}, "conditionals": []
        }))
        .unwrap();
        normalize_default_skill(&mut f);
        assert_eq!(f.default_skill, 3, "outside the list moves to the highest");
        f.default_skill = 1;
        normalize_default_skill(&mut f);
        assert_eq!(f.default_skill, 1, "a listed default is kept");
        f.available_skills.clear();
        f.default_skill = 3;
        normalize_default_skill(&mut f);
        assert_eq!(f.default_skill, 0, "no skills means basic attack");
    }

    #[test]
    fn healers_with_no_default_skill_formula() {
        // `calculate_hps` never consults `skills`, so for healers this gap is
        // harmless; pinned so a regenerated table that changes it is noticed.
        assert_eq!(
            default_skill_gaps(supported_healers()),
            vec![
                "char_181_flower",
                "char_275_breeze",
                "char_285_medic2",
                "char_4091_ulika",
                "char_4163_rosesa",
                "char_449_glider",
                "char_473_mberry",
            ]
        );
    }

    #[test]
    fn skill_formula_default_is_custom_with_no_overrides() {
        let f = SkillFormula::default();
        assert_eq!(f.formula_type, "custom");
        assert!(f.atk_scale_idx.is_none() && f.hits.is_none() && f.damage_type.is_none());
    }
}
