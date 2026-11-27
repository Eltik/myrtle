use std::collections::HashMap;
use std::sync::LazyLock;

use serde::Deserialize;

pub(crate) const FORMULAS_JSON: &str = include_str!("../data/operator_formulas.json");
pub(crate) const HEAL_FORMULAS_JSON: &str = include_str!("../data/heal_formulas.json");
const POLARITY_JSON: &str = include_str!("../data/conditional_polarity.json");

type PolarityTable = HashMap<String, HashMap<String, HashMap<String, Option<bool>>>>;

static POLARITY: LazyLock<PolarityTable> = LazyLock::new(|| {
    serde_json::from_str(POLARITY_JSON).expect("Invalid conditional_polarity.json")
});

pub fn conditional_inverted(table: &str, op_id: &str, cond_type: &str, name: &str) -> bool {
    POLARITY
        .get(table)
        .and_then(|t| t.get(op_id))
        .and_then(|m| m.get(&format!("{cond_type}|{name}")))
        .copied()
        .flatten()
        .unwrap_or(false)
}

pub fn polarity_counts() -> (usize, usize, usize) {
    let mut counts = (0, 0, 0);
    for v in POLARITY
        .values()
        .flat_map(HashMap::values)
        .flat_map(HashMap::values)
    {
        match v {
            Some(false) => counts.0 += 1,
            Some(true) => counts.1 += 1,
            None => counts.2 += 1,
        }
    }
    counts
}

#[derive(Deserialize)]
pub(crate) struct HandFormulas {
    pub(crate) dps: HashMap<String, OperatorFormula>,
    pub(crate) hps: HashMap<String, OperatorFormula>,
}

pub(crate) fn load_hand_formulas() -> HandFormulas {
    serde_json::from_str(crate::custom::hand::FORMULAS_JSON)
        .expect("Invalid custom/hand/formulas.json")
}

#[must_use]
pub fn load_formulas() -> HashMap<String, OperatorFormula> {
    let mut formulas: HashMap<String, OperatorFormula> =
        serde_json::from_str(FORMULAS_JSON).expect("Invalid operator_formulas.json");
    formulas.extend(load_hand_formulas().dps);
    for formula in formulas.values_mut() {
        normalize_default_skill(formula);
    }
    formulas
}

pub(crate) fn normalize_default_skill(formula: &mut OperatorFormula) {
    match formula.available_skills.iter().max() {
        None => formula.default_skill = 0,
        Some(&highest) if !formula.available_skills.contains(&formula.default_skill) => {
            formula.default_skill = highest;
        }
        Some(_) => {}
    }
}

#[must_use]
pub fn load_heal_formulas() -> HashMap<String, OperatorFormula> {
    let mut formulas: HashMap<String, OperatorFormula> =
        serde_json::from_str(HEAL_FORMULAS_JSON).expect("Invalid heal_formulas.json");
    formulas.extend(load_hand_formulas().hps);
    formulas
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct SkillFormula {
    #[serde(rename = "type")]
    pub formula_type: String,
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

static FORMULAS: LazyLock<HashMap<String, OperatorFormula>> = LazyLock::new(load_formulas);
static HEAL_FORMULAS: LazyLock<HashMap<String, OperatorFormula>> =
    LazyLock::new(load_heal_formulas);

pub fn get_formula(op_id: &str) -> Option<&'static OperatorFormula> {
    FORMULAS.get(op_id)
}

#[must_use]
pub fn supported_operators() -> &'static HashMap<String, OperatorFormula> {
    &FORMULAS
}

pub fn get_heal_formula(op_id: &str) -> Option<&'static OperatorFormula> {
    HEAL_FORMULAS.get(op_id)
}

#[must_use]
pub fn supported_healers() -> &'static HashMap<String, OperatorFormula> {
    &HEAL_FORMULAS
}
