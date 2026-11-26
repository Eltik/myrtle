#![allow(
    clippy::unwrap_used,
    clippy::float_cmp,
    clippy::cast_possible_truncation
)]

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use myrtle_engine_grade::GradeData;
use myrtle_engine_grade::operators::{
    DimensionKind, ScoreModel, grade_operators_in, operator_score_breakdown_in,
};
use myrtle_gamedata::compile::{CompileOptions, compile};
use myrtle_gamedata_raw::tables::sandbox_universe::SandboxUniverse as RawSandboxUniverse;
use myrtle_gamedata_raw::{DirSource, load};
use myrtle_types::Server;
use myrtle_types::roster::RosterEntry;

fn game_data() -> Option<&'static GradeData> {
    static DATA: OnceLock<Option<GradeData>> = OnceLock::new();
    DATA.get_or_init(|| {
        let root = PathBuf::from(std::env::var("GAMEDATA_ROOT").ok()?);
        let server = match std::env::var("GAMEDATA_SERVER").as_deref() {
            Ok("cn") => Server::CN,
            _ => Server::EN,
        };
        let raw = load(&DirSource { root }, server).unwrap();
        let sandbox_universe = RawSandboxUniverse::build(&raw.sandbox.perm);
        let m = compile(raw, &CompileOptions::new(server)).unwrap().model;
        Some(GradeData {
            operators: m.operators,
            favor: m.favor,
            roguelike: m.roguelike,
            medals: m.medals,
            stage_universe: m.stage_universe,
            sandbox_universe,
        })
    })
    .as_ref()
}

macro_rules! some_or_skip {
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => {
                println!("GAMEDATA_ROOT unset, skipping");
                return;
            }
        }
    };
}

const fn uuid_nil() -> uuid::Uuid {
    uuid::Uuid::nil()
}

fn entry(operator_id: &str, elite: i16, level: i16) -> RosterEntry {
    RosterEntry {
        user_id: uuid_nil(),
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
    let game_data = some_or_skip!(game_data());

    let mut kaltsit = entry("char_003_kalts", 2, 60);
    kaltsit.skill_level = 7;
    kaltsit.favor_point = 18000;
    kaltsit.masteries = serde_json::json!([{ "mastery": 3 }, { "mastery": 1 }, { "mastery": 0 }]);
    kaltsit.modules = serde_json::json!([{ "id": "uniequip_002_kalts", "level": 2 }]);
    let mut ptilopsis = entry("char_128_plosis", 1, 40);
    ptilopsis.potential = 2;
    ptilopsis.skill_level = 6;
    ptilopsis.favor_point = 8000;
    let mut fang = entry("char_123_fang", 1, 55);
    fang.potential = 5;
    fang.skill_level = 7;
    fang.favor_point = 25000;
    let roster = vec![kaltsit, ptilopsis, fang];
    let roster: Vec<RosterEntry> = roster
        .into_iter()
        .filter(|e| game_data.operators.contains_key(&e.operator_id))
        .collect();
    assert!(
        roster.len() >= 2,
        "expected the fixture operators to exist in gamedata"
    );

    let support_ids: HashSet<&str> = HashSet::from(["char_003_kalts"]);

    let grade = grade_operators_in(&roster, game_data, &support_ids, ScoreModel::SHIPPED);
    let breakdown =
        operator_score_breakdown_in(&roster, game_data, &support_ids, ScoreModel::SHIPPED);

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
            dim.weight_share
                .mul_add(-dim.completion, dim.contribution)
                .abs()
                < 1e-12,
            "{:?} contribution must equal share x completion",
            dim.kind
        );
    }
}

#[test]
fn an_unraised_pull_is_in_the_denominator_unless_invested_only() {
    let game_data = some_or_skip!(game_data());
    let roster = vec![entry("char_123_fang", 0, 1)];
    let support_ids: HashSet<&str> = HashSet::new();

    let breakdown =
        operator_score_breakdown_in(&roster, game_data, &support_ids, ScoreModel::SHIPPED);
    assert!(
        !breakdown.is_empty(),
        "an owned E0 L1 operator must be graded"
    );
    let share: f64 = breakdown.iter().map(|d| d.weight_share).sum();
    assert!(
        (share - 1.0).abs() < 1e-12,
        "shares must sum to 1.0, got {share}"
    );
    let earned: f64 = breakdown.iter().map(|d| d.contribution).sum();
    let grade = grade_operators_in(&roster, game_data, &support_ids, ScoreModel::SHIPPED);
    assert!(
        (earned - grade).abs() < 1e-12,
        "earned {earned} vs grade {grade}"
    );
    assert!(
        grade < 0.05,
        "a fresh pull must earn next to nothing, got {grade}"
    );

    let legacy = operator_score_breakdown_in(
        &roster,
        game_data,
        &support_ids,
        ScoreModel {
            invested_only: true,
            ..ScoreModel::SHIPPED
        },
    );
    assert!(
        legacy.is_empty(),
        "invested-only must drop the unraised pull"
    );
    assert_eq!(
        grade_operators_in(
            &roster,
            game_data,
            &support_ids,
            ScoreModel {
                invested_only: true,
                ..ScoreModel::SHIPPED
            }
        ),
        0.0
    );
}

#[test]
fn every_skill_at_m3_completes_the_mastery_dimension() {
    let game_data = some_or_skip!(game_data());

    for (operator_id, masteries) in [
        (
            "char_151_myrtle",
            serde_json::json!([{ "mastery": 3 }, { "mastery": 3 }]),
        ),
        (
            "char_003_kalts",
            serde_json::json!([{ "mastery": 3 }, { "mastery": 3 }, { "mastery": 3 }]),
        ),
    ] {
        let Some(static_op) = game_data.operators.get(operator_id) else {
            continue;
        };
        assert_eq!(
            static_op.skills.len(),
            masteries.as_array().map_or(0, Vec::len),
            "{operator_id}: fixture must master every skill the operator has"
        );

        let mut op = entry(operator_id, 2, 1);
        op.masteries = masteries;
        let breakdown =
            operator_score_breakdown_in(&[op], game_data, &HashSet::new(), ScoreModel::SHIPPED);

        let mastery = breakdown
            .iter()
            .find(|d| matches!(d.kind, DimensionKind::Mastery))
            .unwrap_or_else(|| panic!("{operator_id}: expected a mastery dimension"));
        assert!(
            (mastery.completion - 1.0).abs() < 1e-9,
            "{operator_id}: fully mastered but completion = {}",
            mastery.completion
        );
    }
}

#[test]
fn rarity_weights_follow_build_cost_and_legacy_restores_the_old_numbers() {
    use myrtle_engine_grade::operators::rarity_to_weight_in;
    use myrtle_gamedata_raw::tables::operator::OperatorRarity as R;
    let ladder = [
        R::SixStar,
        R::FiveStar,
        R::FourStar,
        R::ThreeStar,
        R::TwoStar,
        R::OneStar,
    ];
    let shipped: Vec<f64> = ladder
        .iter()
        .map(|r| rarity_to_weight_in(r, false))
        .collect();
    let legacy: Vec<f64> = ladder
        .iter()
        .map(|r| rarity_to_weight_in(r, true))
        .collect();
    assert_eq!(shipped, [1.0, 0.5, 0.3, 0.05, 0.02, 0.01]);
    assert_eq!(legacy, [1.0, 0.7, 0.4, 0.15, 0.1, 0.05]);
}

#[test]
fn a_mod3_is_priced_per_slot_and_per_rarity_cost() {
    use myrtle_engine_grade::operators::{grade_operator_in, rarity_to_weight_in};
    let game_data = some_or_skip!(game_data());
    let pre_09_22 = ScoreModel {
        invested_only: false,
        legacy_rarity_weights: true,
        module_per_slot: false,
    };

    let mod3_gain = |op_id: &str, module_id: &str, model: ScoreModel| -> f64 {
        let op = game_data
            .operators
            .get(op_id)
            .unwrap_or_else(|| panic!("{op_id} in gamedata"));
        let max_level = op.phases.last().map_or(1, |p| p.max_level as i16);
        let before = entry(op_id, 2, max_level);
        let mut after = entry(op_id, 2, max_level);
        after.modules = serde_json::json!([{ "id": module_id, "level": 3 }]);
        let favor = &game_data.favor;
        let delta = grade_operator_in(&after, op, favor, false, &model)
            - grade_operator_in(&before, op, favor, false, &model);
        delta * rarity_to_weight_in(&op.rarity, model.legacy_rarity_weights)
    };
    let close = |a: f64, b: f64| (a - b).abs() < 1e-6;

    let viviana_old = mod3_gain("char_4098_vvana", "uniequip_002_vvana", pre_09_22);
    let conviction_old = mod3_gain("char_159_peacok", "uniequip_002_peacok", pre_09_22);
    assert!(close(viviana_old, 0.5 * 25.0 / 120.0), "{viviana_old}");
    assert!(
        close(conviction_old, 0.4 * 25.0 / 120.0),
        "{conviction_old}"
    );
    assert!(close(conviction_old / viviana_old, 0.8));

    let viviana = mod3_gain("char_4098_vvana", "uniequip_002_vvana", ScoreModel::SHIPPED);
    let conviction = mod3_gain(
        "char_159_peacok",
        "uniequip_002_peacok",
        ScoreModel::SHIPPED,
    );
    assert!(close(viviana, 0.5 * 50.0 / 145.0), "{viviana}");
    assert!(close(conviction, 0.3 * 25.0 / 120.0), "{conviction}");
    assert!(
        close(conviction / viviana, 0.3625),
        "{}",
        conviction / viviana
    );
}

#[test]
fn every_mod3_of_a_three_slot_operator_is_worth_one_slot() {
    use myrtle_engine_grade::operators::grade_operator_in;
    let game_data = some_or_skip!(game_data());
    let op = game_data
        .operators
        .get("char_4046_ebnhlz")
        .expect("Ebenholz in gamedata");
    let max_level = op.phases.last().map_or(1, |p| p.max_level as i16);
    let favor = &game_data.favor;
    let modules = [
        "uniequip_002_ebnhlz",
        "uniequip_003_ebnhlz",
        "uniequip_004_ebnhlz",
    ];
    let grades: Vec<f64> = (0..=modules.len())
        .map(|built| {
            let mut e = entry("char_4046_ebnhlz", 2, max_level);
            e.modules = serde_json::Value::Array(
                modules[..built]
                    .iter()
                    .map(|id| serde_json::json!({ "id": id, "level": 3 }))
                    .collect(),
            );
            grade_operator_in(&e, op, favor, false, &ScoreModel::SHIPPED)
        })
        .collect();
    for step in grades.windows(2) {
        let gain = step[1] - step[0];
        assert!((gain - 25.0 / 170.0).abs() < 1e-6, "{grades:?}");
    }
}
