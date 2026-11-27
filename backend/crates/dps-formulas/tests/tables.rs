//! The bundled formula tables through the public lookups.

use std::collections::HashMap;

use myrtle_dps_formulas::tables::{
    OperatorFormula, SkillFormula, get_formula, get_heal_formula, supported_healers,
    supported_operators,
};

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
