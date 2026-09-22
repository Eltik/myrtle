//! Consistency tests for the operator score breakdown: the per-dimension
//! decomposition must agree exactly with the collapsed subscore users see.

mod common;

use std::collections::HashSet;

use backend::core::grade::grade_operators::{
    DimensionKind, ScoreModel, grade_operators, grade_operators_in, operator_score_breakdown,
    operator_score_breakdown_in,
};
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

    let grade = grade_operators(&roster, game_data, &support_ids);
    let breakdown = operator_score_breakdown(&roster, game_data, &support_ids);

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

/// An owned operator is in the average from the moment it is pulled: an E0 L1
/// roster has a full set of weight shares and earns nothing on them. Under the
/// `GRADE_INVESTED_ONLY` kill switch the same roster is outside the grade.
#[test]
fn an_unraised_pull_is_in_the_denominator_unless_invested_only() {
    let game_data = common::load_game_data();
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

/// The default entry points read the switch from the environment; with it
/// unset they must be the all-owned grader bit for bit.
#[test]
fn default_entry_points_grade_every_owned_operator() {
    let game_data = common::load_game_data();
    let roster = vec![entry("char_123_fang", 0, 1), entry("char_003_kalts", 2, 60)];
    let support_ids: HashSet<&str> = HashSet::new();
    assert_eq!(
        grade_operators(&roster, game_data, &support_ids),
        grade_operators_in(&roster, game_data, &support_ids, ScoreModel::SHIPPED)
    );
    assert_eq!(
        operator_score_breakdown(&roster, game_data, &support_ids).len(),
        operator_score_breakdown_in(&roster, game_data, &support_ids, ScoreModel::SHIPPED).len()
    );
}

#[test]
fn every_skill_at_m3_completes_the_mastery_dimension() {
    let game_data = common::load_game_data();

    // Most 4★/5★ operators only ever have two skills, so "all skills at M3" is
    // M6 for them, not M9. Scoring the milestone off the raw M3 count used to
    // cap them at 0.75 with nothing left to buy - see the ladder in
    // `mastery_milestone_from_levels`.
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
        let breakdown = operator_score_breakdown(&[op], game_data, &HashSet::new());

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

/// The "operators below milestone" card must price gains the score can pay.
/// Only operators inside `grade_operators`' average may be listed, and the
/// advertised gains, summed, must fit inside the subscore's remaining headroom.
/// Every owned operator is in the average, so an unraised pull is listed with
/// an ELITE gain the score really pays.
#[test]
fn below_milestone_lists_every_graded_operator_and_fits_the_headroom() {
    use backend::app::services::improvements::build_operator_improvements;

    let game_data = common::load_game_data();

    let mut kaltsit = entry("char_003_kalts", 2, 60);
    kaltsit.skill_level = 7;
    kaltsit.favor_point = 18000;
    kaltsit.masteries = serde_json::json!([{ "mastery": 3 }, { "mastery": 1 }, { "mastery": 0 }]);
    kaltsit.modules = serde_json::json!([{ "id": "uniequip_002_kalts", "level": 2 }]);
    let mut ptilopsis = entry("char_128_plosis", 1, 40);
    ptilopsis.skill_level = 6;
    // A 6★ still at its pull state: owned, unraised, inside the average.
    let unraised = entry("char_010_chen", 0, 1);
    let roster: Vec<RosterEntry> = vec![kaltsit, ptilopsis, unraised]
        .into_iter()
        .filter(|e| game_data.operators.contains_key(&e.operator_id))
        .collect();
    assert_eq!(
        roster.len(),
        3,
        "expected the fixture operators to exist in gamedata"
    );

    let support_ids: HashSet<&str> = HashSet::new();
    let grade = grade_operators(&roster, game_data, &support_ids);
    let improvements = build_operator_improvements(&roster, game_data, &support_ids);

    let listed: Vec<&str> = improvements
        .below_milestone
        .iter()
        .map(|g| g.operator_id.as_str())
        .collect();
    assert!(
        listed.contains(&"char_010_chen"),
        "an E0 L1 pull is in the grade's denominator and must be priced: {listed:?}"
    );
    assert!(
        listed.contains(&"char_003_kalts") && listed.contains(&"char_128_plosis"),
        "raised operators with open milestones must be listed: {listed:?}"
    );
    let chen = improvements
        .below_milestone
        .iter()
        .find(|g| g.operator_id == "char_010_chen")
        .expect("chen listed");
    assert!(
        chen.deltas
            .iter()
            .any(|d| d.tag == "ELITE" && d.operator_grade_delta > 0.0),
        "promoting an unraised pull must now raise the subscore: {:?}",
        chen.deltas
    );

    let headroom = 1.0 - grade;
    let advertised: f64 = improvements
        .below_milestone
        .iter()
        .map(|g| g.subscore_potential_gain)
        .sum();
    assert!(
        advertised <= headroom + 1e-12,
        "advertised gain {advertised} exceeds the subscore headroom {headroom}"
    );
    // Per-tag sums are what the card prints per upgrade type; each must fit too.
    let mut per_tag: std::collections::HashMap<&str, f64> = std::collections::HashMap::new();
    for gap in &improvements.below_milestone {
        for d in &gap.deltas {
            *per_tag.entry(d.tag).or_default() += d.operator_grade_delta;
        }
    }
    for (tag, sum) in per_tag {
        assert!(
            sum <= headroom + 1e-12,
            "tag {tag} advertises {sum} on {headroom} of headroom"
        );
    }
}

/// The rarity weights follow the cost of a full build (census in
/// `rarity_to_weight_in`), and the `GRADE_RARITY_WEIGHTS=legacy` switch
/// restores the pre-2026-09-22 numbers exactly.
#[test]
fn rarity_weights_follow_build_cost_and_legacy_restores_the_old_numbers() {
    use backend::core::gamedata::types::operator::OperatorRarity as R;
    use backend::core::grade::grade_operators::rarity_to_weight_in;
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

/// The 2026-09-22 report: a Mod3 on Viviana (6★, two advanced module slots)
/// showed +0.07 and on Conviction (4★, one slot) +0.06, at a quarter of the
/// materials (every 4★ module costs 3 blocks / 15 sticks / 5 instruments /
/// 75k LMD against a 6★'s 12 / 60 / 20 / 300k). Per operator weight the
/// model priced Conviction at 0.800 of Viviana; per slot with cost-based
/// rarity weights it is 0.3625, against a cost ratio of 0.25 (the residual
/// is the gating premium in the dimension constants).
#[test]
fn a_mod3_is_priced_per_slot_and_per_rarity_cost() {
    use backend::core::grade::grade_operators::{grade_operator_in, rarity_to_weight_in};
    let game_data = common::load_game_data();
    let pre_09_22 = ScoreModel {
        invested_only: false,
        legacy_rarity_weights: true,
        module_per_slot: false,
    };

    // Grade contribution per unit of roster weight of the first Mod3, from
    // an otherwise module-less E2 max operator.
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

/// Price the 2026-09-22 reweight on every local account: the Operators
/// subscore under the model deployed that morning (all-owned denominator,
/// legacy rarity weights, one module weight per operator) against the
/// shipped model. Prints the distribution; the numbers belong in the commit.
#[tokio::test]
#[ignore = "reads the local Postgres roster"]
async fn real_data_reweight_before_after() {
    use std::collections::HashMap;
    let game_data = common::load_game_data();
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("connect to local postgres");
    let rows: Vec<RosterEntry> = sqlx::query_as("SELECT * FROM v_user_roster")
        .fetch_all(&pool)
        .await
        .expect("select roster");
    let supports: Vec<(Uuid, String)> =
        sqlx::query_as("SELECT user_id, operator_id FROM user_support_units")
            .fetch_all(&pool)
            .await
            .expect("select supports");
    let mut by_user: HashMap<Uuid, Vec<RosterEntry>> = HashMap::new();
    for r in rows {
        by_user.entry(r.user_id).or_default().push(r);
    }
    let mut support_by_user: HashMap<Uuid, HashSet<String>> = HashMap::new();
    for (u, op) in supports {
        support_by_user.entry(u).or_default().insert(op);
    }

    let before_model = ScoreModel {
        invested_only: false,
        legacy_rarity_weights: true,
        module_per_slot: false,
    };
    let mut before = Vec::new();
    let mut after = Vec::new();
    for (user, roster) in &by_user {
        let sup = support_by_user.get(user);
        let ids: HashSet<&str> = sup
            .map(|s| s.iter().map(String::as_str).collect())
            .unwrap_or_default();
        before.push(grade_operators_in(roster, game_data, &ids, before_model));
        after.push(grade_operators_in(
            roster,
            game_data,
            &ids,
            ScoreModel::SHIPPED,
        ));
    }
    let median = |v: &[f64]| {
        let mut s = v.to_vec();
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        s[s.len() / 2]
    };
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let deltas: Vec<f64> = before.iter().zip(&after).map(|(b, a)| a - b).collect();
    let mut sorted = deltas.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |p: f64| sorted[((p * sorted.len() as f64) as usize).min(sorted.len() - 1)];
    println!(
        "accounts={} before median={:.4} mean={:.4} | after median={:.4} mean={:.4} | delta median={:+.4} p10={:+.4} p90={:+.4} min={:+.4} max={:+.4} | up>0.5pt={} down>0.5pt={}",
        before.len(),
        median(&before),
        mean(&before),
        median(&after),
        mean(&after),
        median(&deltas),
        q(0.1),
        q(0.9),
        sorted[0],
        sorted[sorted.len() - 1],
        deltas.iter().filter(|d| **d > 0.005).count(),
        deltas.iter().filter(|d| **d < -0.005).count(),
    );
    assert!(
        after
            .iter()
            .all(|g| g.is_finite() && (0.0..=1.0).contains(g))
    );
}
