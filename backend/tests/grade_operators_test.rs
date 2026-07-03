//! Consistency tests for the operator score breakdown: the per-dimension
//! decomposition must agree exactly with the collapsed subscore users see.

mod common;

use std::collections::HashSet;

use backend::core::grade::grade_operators::{grade_operators, operator_score_breakdown};
use backend::database::models::roster::RosterEntry;
use sqlx::types::Uuid;

/// A roster entry at the given promotion state; investment fields beyond
/// elite/level start at their fresh-pull defaults and are overridden per test.
fn entry(operator_id: &str, elite: i16, level: i16) -> RosterEntry {
    RosterEntry {
        user_id: Uuid::nil(),
        operator_id: operator_id.into(),
        elite,
        level,
        exp: 0,
        potential: 0,
        skill_level: 1,
        favor_point: 0,
        skin_id: None,
        default_skill: None,
        voice_lan: None,
        current_equip: None,
        current_tmpl: None,
        obtained_at: None,
        masteries: serde_json::json!([]),
        modules: serde_json::json!([]),
    }
}

#[test]
fn breakdown_sums_to_the_operator_grade() {
    let game_data = common::load_game_data();

    // A spread of investment levels across rarities. Operator ids are the
    // stable early-roster ones present in every gamedata dump.
    // 6★ mid-investment with one M3 and a leveled module.
    let mut kaltsit = entry("char_003_kalts", 2, 60);
    kaltsit.skill_level = 7;
    kaltsit.favor_point = 18000;
    kaltsit.masteries = serde_json::json!([{ "mastery": 3 }, { "mastery": 1 }, { "mastery": 0 }]);
    kaltsit.modules = serde_json::json!([{ "id": "uniequip_002_kalts", "level": 2 }]);
    // 5★ freshly promoted to E1.
    let mut ptilopsis = entry("char_128_plosis", 1, 40);
    ptilopsis.potential = 2;
    ptilopsis.skill_level = 6;
    ptilopsis.favor_point = 8000;
    // 3★ maxed-out (no masteries/modules exist at this rarity).
    let mut fang = entry("char_123_fang", 1, 55);
    fang.potential = 5;
    fang.skill_level = 7;
    fang.favor_point = 25000;
    let roster = vec![kaltsit, ptilopsis, fang];
    // Keep only ids that exist in the loaded gamedata so the test doesn't
    // silently degrade to an empty roster on a trimmed dump.
    let roster: Vec<RosterEntry> = roster
        .into_iter()
        .filter(|e| game_data.operators.contains_key(&e.operator_id))
        .collect();
    assert!(
        roster.len() >= 2,
        "expected the fixture operators to exist in gamedata"
    );

    let support_ids: HashSet<&str> = HashSet::from(["char_003_kalts"]);

    let grade = grade_operators(&roster, &game_data, &support_ids);
    let breakdown = operator_score_breakdown(&roster, &game_data, &support_ids);

    assert!(!breakdown.is_empty());
    assert!(grade > 0.0 && grade < 1.0, "grade = {grade}");

    let contribution_sum: f64 = breakdown.iter().map(|d| d.contribution).sum();
    let share_sum: f64 = breakdown.iter().map(|d| d.weight_share).sum();

    assert!(
        (contribution_sum - grade).abs() < 1e-9,
        "contributions ({contribution_sum}) must sum to the subscore ({grade})"
    );
    assert!(
        (share_sum - 1.0).abs() < 1e-9,
        "weight shares must sum to 1.0, got {share_sum}"
    );
    for dim in &breakdown {
        assert!(
            (0.0..=1.0).contains(&dim.completion),
            "{:?} completion out of range: {}",
            dim.kind,
            dim.completion
        );
        assert!(
            (dim.contribution - dim.weight_share * dim.completion).abs() < 1e-12,
            "{:?} contribution must equal share x completion",
            dim.kind
        );
    }
}

#[test]
fn breakdown_is_empty_for_an_uninvested_roster() {
    let game_data = common::load_game_data();
    // E0 L1 = no investment; such operators don't count toward the grade.
    let roster = vec![entry("char_123_fang", 0, 1)];
    let breakdown = operator_score_breakdown(&roster, &game_data, &HashSet::new());
    assert!(breakdown.is_empty());
}
