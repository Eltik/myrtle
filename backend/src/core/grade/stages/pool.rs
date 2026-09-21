//! One player's gradeable stage pools.
//!
//! The stage universe is the same for everyone; what a given player can be
//! graded on is narrower, and three gates decide it:
//!
//!   - the server gate: a stage no user on the player's server has ever seen
//!     is not on that server yet (the bundled tables can run ahead of EN);
//!   - the event window: a limited event that has not rerun counts only while
//!     it is open (`event::event_is_gradeable`);
//!   - the either/or gate: an optional stage (one arm of a hub choice that
//!     locks the other arm for good) counts only once the player cleared it.
//!
//! [`PlayerPools`] applies all three once, and every consumer walks the result:
//! the weighted grade, the Score tab counts and the improvements gap lists all
//! see the same [`PoolStage`] rows, so a gate is written in exactly one place.

use std::collections::{HashMap, HashSet};

use crate::core::gamedata::types::stage_universe::StageUniverse;

use super::event::{decay_factor, event_is_gradeable};
use super::types::StageClear;

/// One stage of a player's pool, after every gate, with the player's record.
#[derive(Debug, Clone, Copy)]
pub struct PoolStage<'a> {
    pub stage_id: &'a str,
    /// The universe weight (zone weight times difficulty multiplier).
    pub weight: f64,
    /// Recency decay for an event that has closed; 1.0 for everything else.
    pub decay: f64,
    /// The player's dungeon record, `None` when they never touched the stage.
    pub clear: Option<&'a StageClear>,
}

impl PoolStage<'_> {
    /// The raw dungeon state, 0 when the player has no record.
    pub fn state(&self) -> i16 {
        self.clear.map_or(0, |c| c.state)
    }

    pub fn is_cleared(&self) -> bool {
        self.clear.is_some_and(StageClear::is_cleared)
    }

    pub fn is_three_starred(&self) -> bool {
        self.clear.is_some_and(StageClear::is_three_starred)
    }

    /// Weighted credit toward the pool grade: 1.0 for three stars, 0.7 for a
    /// clear, 0 otherwise, scaled by weight and decay.
    fn credit(&self) -> f64 {
        self.clear.map_or(0.0, StageClear::clear_score) * self.weight * self.decay
    }
}

/// The universe as one player sees it.
pub struct PlayerPools<'a> {
    universe: &'a StageUniverse,
    clears: &'a HashMap<String, StageClear>,
    /// Stage ids the player's server ships; `None` accepts everything.
    allowed: Option<&'a HashSet<String>>,
    now: i64,
    last_synced_ts: Option<i64>,
}

impl<'a> PlayerPools<'a> {
    pub fn new(
        universe: &'a StageUniverse,
        clears: &'a HashMap<String, StageClear>,
        allowed: Option<&'a HashSet<String>>,
        now: i64,
        last_synced_ts: Option<i64>,
    ) -> Self {
        Self {
            universe,
            clears,
            allowed,
            now,
            last_synced_ts,
        }
    }

    /// The permanent pool: server gate and either/or gate.
    pub fn permanent(&self) -> impl Iterator<Item = PoolStage<'a>> + '_ {
        self.universe
            .permanent
            .iter()
            .filter(|e| self.on_server(&e.stage_id))
            .filter_map(|e| self.admit(&e.stage_id, e.weight, 1.0, e.optional))
    }

    /// The event pool: server gate, event window, either/or gate, with
    /// recency decay on closed events.
    pub fn event(&self) -> impl Iterator<Item = PoolStage<'a>> + '_ {
        self.universe
            .event
            .iter()
            .filter(|e| event_is_gradeable(e, self.now, self.last_synced_ts, self.allowed))
            .filter_map(|e| {
                let decay = decay_factor(e.end_time, self.now);
                self.admit(&e.stage_id, e.weight, decay, e.optional)
            })
    }

    fn on_server(&self, stage_id: &str) -> bool {
        self.allowed.is_none_or(|set| set.contains(stage_id))
    }

    /// The either/or gate. An optional stage sits behind a choice the player
    /// made once and cannot undo, so it joins the pool (numerator and
    /// denominator alike) only once cleared, and is never a gap.
    fn admit(
        &self,
        stage_id: &'a str,
        weight: f64,
        decay: f64,
        optional: bool,
    ) -> Option<PoolStage<'a>> {
        let stage = PoolStage {
            stage_id,
            weight,
            decay,
            clear: self.clears.get(stage_id),
        };
        (!optional || stage.is_cleared()).then_some(stage)
    }
}

/// How much of a pool the player has: cleared credit over the pool's weight,
/// clamped to 1.0; 0 for an empty pool.
pub fn weighted_score<'a>(stages: impl Iterator<Item = PoolStage<'a>>) -> f64 {
    let (credit, weight) = stages.fold((0.0, 0.0), |(credit, weight), s| {
        (credit + s.credit(), weight + s.weight * s.decay)
    });
    if weight <= 0.0 {
        return 0.0;
    }
    (credit / weight).min(1.0)
}

/// Headline counts for one pool.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PoolCounts {
    pub total: usize,
    pub cleared: usize,
    pub three_starred: usize,
}

impl PoolCounts {
    pub fn of<'a>(stages: impl Iterator<Item = PoolStage<'a>>) -> Self {
        stages.fold(Self::default(), |mut counts, s| {
            counts.total += 1;
            counts.cleared += usize::from(s.is_cleared());
            counts.three_starred += usize::from(s.is_three_starred());
            counts
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{PlayerPools, PoolCounts, weighted_score};
    use crate::core::gamedata::types::stage_universe::{StageUniverse, UniverseEntry};
    use crate::core::grade::stages::types::StageClear;
    use std::collections::{HashMap, HashSet};

    fn entry(stage_id: &str, optional: bool) -> UniverseEntry {
        UniverseEntry {
            stage_id: stage_id.to_string(),
            weight: 1.0,
            optional,
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

    fn record(state: i16) -> StageClear {
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
    fn an_empty_pool_scores_zero() {
        let universe = StageUniverse::default();
        let clears = clears(&[]);
        let pools = PlayerPools::new(&universe, &clears, None, 0, None);
        assert_eq!(weighted_score(pools.permanent()), 0.0);
        assert_eq!(PoolCounts::of(pools.event()), PoolCounts::default());
    }
}
