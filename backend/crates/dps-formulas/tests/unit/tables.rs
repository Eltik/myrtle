use std::collections::HashMap;

use crate::tables::{
    FORMULAS_JSON, HEAL_FORMULAS_JSON, OperatorFormula, load_formulas, load_hand_formulas,
    load_heal_formulas, normalize_default_skill, supported_healers, supported_operators,
};

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
        include_str!("../../src/custom/dispatch.rs"),
        &hand.dps,
        &load_formulas(),
    );
    check(
        "hps",
        HEAL_FORMULAS_JSON,
        include_str!("../../src/custom/dispatch_hps.rs"),
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
