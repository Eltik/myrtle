use std::collections::HashMap;
use std::sync::LazyLock;

use serde::Deserialize;

const FORMULAS_JSON: &str = include_str!("../data/operator_formulas.json");
const HEAL_FORMULAS_JSON: &str = include_str!("../data/heal_formulas.json");
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
struct HandFormulas {
    dps: HashMap<String, OperatorFormula>,
    hps: HashMap<String, OperatorFormula>,
}

fn load_hand_formulas() -> HandFormulas {
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

fn normalize_default_skill(formula: &mut OperatorFormula) {
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

    /// Hand ids that are ALSO in a generated table. An id lands here
    /// only after a review decided to keep the hand formula over upstream's.
    const REVIEWED_HAND_OVERRIDES: &[&str] = &[
        // Manticore: the transpiled formula re-adds the reference's hand-coded module
        // step to a talent `OperatorUnit` already upgraded (custom/hand/manticore.rs).
        // Ported from main a6f95021 (#69).
        "char_215_mantic",
    ];

    fn dispatch_arm_ids(source: &str) -> Vec<String> {
        let body = source
            .split("pub fn dispatch(")
            .nth(1)
            .and_then(|rest| rest.split("\n}\n").next())
            .expect("no `pub fn dispatch(` in the generated dispatch file");
        let mut ids: Vec<String> = body
            .lines()
            .filter_map(|line| line.trim().strip_prefix('"')?.split_once("\" =>"))
            .map(|(id, _)| id.to_string())
            .collect();
        ids.sort();
        ids
    }

    #[test]
    fn bundled_formula_tables_parse() {
        let hand = load_hand_formulas();
        let check = |name: &str,
                     generated_json: &str,
                     dispatch_source: &str,
                     hand_table: &HashMap<String, OperatorFormula>,
                     merged: &HashMap<String, OperatorFormula>| {
            let generated: HashMap<String, OperatorFormula> =
                serde_json::from_str(generated_json).expect("generated table parses");
            let mut generated_ids: Vec<String> = generated.keys().cloned().collect();
            generated_ids.sort();
            assert_eq!(
                generated_ids,
                dispatch_arm_ids(dispatch_source),
                "{name}: generated table vs generated dispatch arms"
            );
            let mut overlap: Vec<&str> = hand_table
                .keys()
                .filter(|id| generated.contains_key(*id))
                .map(String::as_str)
                .collect();
            overlap.sort_unstable();
            let reviewed: Vec<&str> = REVIEWED_HAND_OVERRIDES
                .iter()
                .copied()
                .filter(|id| hand_table.contains_key(*id))
                .collect();
            assert_eq!(
                overlap, reviewed,
                "{name}: hand ids that shadow a generated formula; delete the hand file \
                 or list the id in REVIEWED_HAND_OVERRIDES"
            );
            assert_eq!(
                merged.len(),
                generated.len() + hand_table.len() - overlap.len(),
                "{name}: merged = generated + hand - overlap"
            );
            println!(
                "{name}: {} generated + {} hand - {} overlap = {}",
                generated.len(),
                hand_table.len(),
                overlap.len(),
                merged.len()
            );
        };
        check(
            "dps",
            FORMULAS_JSON,
            include_str!("custom/dispatch.rs"),
            &hand.dps,
            &load_formulas(),
        );
        check(
            "hps",
            HEAL_FORMULAS_JSON,
            include_str!("custom/dispatch_hps.rs"),
            &hand.hps,
            &load_heal_formulas(),
        );
        assert_eq!(supported_operators().len(), load_formulas().len());
        assert_eq!(supported_healers().len(), load_heal_formulas().len());
    }

    #[test]
    fn hand_registry_matches_hand_metadata() {
        use crate::custom::hand;
        let meta = load_hand_formulas();
        let check = |fns: Vec<&str>,
                     table: &HashMap<String, OperatorFormula>,
                     merged: &HashMap<String, OperatorFormula>| {
            let mut fns = fns;
            fns.sort_unstable();
            let mut ids: Vec<&str> = table.keys().map(String::as_str).collect();
            ids.sort_unstable();
            assert_eq!(fns, ids, "hand registry vs custom/hand/formulas.json");
            for id in ids {
                assert!(
                    merged.contains_key(id),
                    "{id} missing from the merged table"
                );
            }
        };
        check(
            hand::DPS.iter().map(|(id, _)| *id).collect(),
            &meta.dps,
            &load_formulas(),
        );
        check(
            hand::HPS.iter().map(|(id, _)| *id).collect(),
            &meta.hps,
            &load_heal_formulas(),
        );
        assert!(hand::dps("char_1015_aglna2").is_some());
        assert!(hand::dps("char_002_amiya").is_none());

        for (id, _) in hand::SKILL_DURATION {
            assert!(
                hand::dps(id).is_some() || hand::hps(id).is_some(),
                "{id}: in hand::SKILL_DURATION but in neither hand::DPS nor hand::HPS"
            );
        }
        let mut duration_ids: Vec<&str> = hand::SKILL_DURATION.iter().map(|(id, _)| *id).collect();
        let listed = duration_ids.len();
        duration_ids.sort_unstable();
        duration_ids.dedup();
        assert_eq!(
            duration_ids.len(),
            listed,
            "duplicate hand::SKILL_DURATION row"
        );
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
        assert_eq!(
            get_formula("char_1044_hsgma2").map(|f| f.default_skill),
            Some(3)
        );
        assert_eq!(
            get_formula("char_347_jaksel").map(|f| f.default_skill),
            Some(0)
        );
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
