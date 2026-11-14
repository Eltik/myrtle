#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::float_cmp)] // exact 0.0 for an empty pool is the claim under test
use myrtle_engine_grade::stages::pool::{PlayerPools, PoolCounts, weighted_score};
use myrtle_gamedata::model::stage_universe::{StageUniverse, UniverseEntry};
use myrtle_types::stage_clear::StageClear;
use std::collections::{HashMap, HashSet};

fn entry(stage_id: &str, optional: bool) -> UniverseEntry {
    UniverseEntry {
        stage_id: stage_id.to_string(),
        weight: 1.0,
        optional,
        live_only: false,
    }
}

fn live_only_entry(stage_id: &str) -> UniverseEntry {
    UniverseEntry {
        live_only: true,
        ..entry(stage_id, false)
    }
}

/// Three permanent stages of weight 1.0, the third behind an either/or ring.
fn universe() -> StageUniverse {
    StageUniverse {
        permanent: vec![
            entry("main_01-01", false),
            entry("main_01-02", false),
            entry("act21side_06_m", true),
        ],
        event: vec![],
        // The optional stage never belongs to the pool-wide maximum.
        permanent_max: 2.0,
    }
}

const fn record(state: i16) -> StageClear {
    StageClear {
        state,
        state_max: state,
        inferred: false,
        complete_times: 1,
        practice_times: 0,
    }
}

fn clears(states: &[(&str, i16)]) -> HashMap<String, StageClear> {
    states
        .iter()
        .map(|(id, state)| ((*id).to_string(), record(*state)))
        .collect()
}

fn permanent_ids<'a>(pools: &PlayerPools<'a>) -> Vec<&'a str> {
    pools.permanent().map(|s| s.stage_id).collect()
}

#[test]
fn an_uncleared_optional_stage_is_not_in_the_pool_at_all() {
    let universe = universe();
    let clears = clears(&[("main_01-01", 3)]);
    let pools = PlayerPools::new(&universe, &clears, None, 0, None);

    assert_eq!(permanent_ids(&pools), vec!["main_01-01", "main_01-02"]);
    assert_eq!(
        PoolCounts::of(pools.permanent()),
        PoolCounts {
            total: 2,
            cleared: 1,
            three_starred: 1
        }
    );
    // Denominator 2.0, not 3.0: one 3-starred stage out of the two the
    // player can actually reach is 1.0 / 2.0.
    let score = weighted_score(pools.permanent());
    assert!(
        (score - 0.5).abs() < 1e-9,
        "expected 1.0 / 2.0 = 0.5, got {score}"
    );
}

#[test]
fn a_cleared_optional_stage_counts_like_any_other() {
    let universe = universe();
    let clears = clears(&[("main_01-01", 3), ("act21side_06_m", 3)]);
    let pools = PlayerPools::new(&universe, &clears, None, 0, None);

    assert_eq!(
        permanent_ids(&pools),
        vec!["main_01-01", "main_01-02", "act21side_06_m"]
    );
    assert_eq!(
        PoolCounts::of(pools.permanent()),
        PoolCounts {
            total: 3,
            cleared: 2,
            three_starred: 2
        }
    );
    // The player took this arm, so the stage is back in both halves:
    // denominator 3.0, numerator 2.0.
    let score = weighted_score(pools.permanent());
    assert!(
        (score - 2.0 / 3.0).abs() < 1e-9,
        "expected 2.0 / 3.0 = 0.667, got {score}"
    );
}

#[test]
fn a_two_star_clear_earns_partial_credit() {
    let universe = universe();
    let clears = clears(&[("main_01-01", 2), ("main_01-02", 3)]);
    let pools = PlayerPools::new(&universe, &clears, None, 0, None);

    assert_eq!(
        PoolCounts::of(pools.permanent()),
        PoolCounts {
            total: 2,
            cleared: 2,
            three_starred: 1
        }
    );
    // 0.7 + 1.0 over 2.0.
    let score = weighted_score(pools.permanent());
    assert!(
        (score - 0.85).abs() < 1e-9,
        "expected 1.7 / 2.0 = 0.85, got {score}"
    );
}

#[test]
fn a_stage_the_server_does_not_ship_is_out_of_both_halves() {
    let universe = universe();
    let clears = clears(&[("main_01-01", 3)]);
    let allowed: HashSet<String> = ["main_01-01".to_string()].into();
    let pools = PlayerPools::new(&universe, &clears, Some(&allowed), 0, None);

    assert_eq!(permanent_ids(&pools), vec!["main_01-01"]);
    let score = weighted_score(pools.permanent());
    assert!(
        (score - 1.0).abs() < 1e-9,
        "expected 1.0 / 1.0, got {score}"
    );
}

#[test]
fn a_live_only_stage_counts_only_once_cleared_and_a_clear_is_three_stars() {
    // Saluzzo Estate: an Il Siracusano hub task, cleared at two stars
    // while the event ran and never replayable since.
    let universe = StageUniverse {
        permanent: vec![
            entry("main_01-01", false),
            live_only_entry("act21side_04_m1"),
            live_only_entry("act21side_02_t"),
        ],
        event: vec![],
        permanent_max: 1.0,
    };
    let clears = clears(&[("main_01-01", 3), ("act21side_04_m1", 2)]);
    let pools = PlayerPools::new(&universe, &clears, None, 0, None);

    // The untouched hub task is not a gap; the 2-star one is not a
    // 3-star gap.
    assert_eq!(permanent_ids(&pools), vec!["main_01-01", "act21side_04_m1"]);
    assert_eq!(
        PoolCounts::of(pools.permanent()),
        PoolCounts {
            total: 2,
            cleared: 2,
            three_starred: 2
        }
    );
    let score = weighted_score(pools.permanent());
    assert!(
        (score - 1.0).abs() < 1e-9,
        "expected 2.0 / 2.0 = 1.0, got {score}"
    );
}

#[test]
fn an_empty_pool_scores_zero() {
    let universe = StageUniverse::default();
    let clears = clears(&[]);
    let pools = PlayerPools::new(&universe, &clears, None, 0, None);
    assert_eq!(weighted_score(pools.permanent()), 0.0);
    assert_eq!(PoolCounts::of(pools.event()), PoolCounts::default());
}
