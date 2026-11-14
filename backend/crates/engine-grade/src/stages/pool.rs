use std::collections::{HashMap, HashSet};

use myrtle_gamedata::model::stage_universe::StageUniverse;
use myrtle_types::stage_clear::StageClear;

use crate::stages::{event::decay_factor, event_is_gradeable};

#[derive(Debug, Clone, Copy)]
pub struct PoolStage<'a> {
    pub stage_id: &'a str,
    /// Zone weight times difficulty multiplier
    pub weight: f64,
    /// Recency decay for an event that has ended
    pub decay: f64,
    pub clear: Option<&'a StageClear>,
    pub live_only: bool,
}

impl PoolStage<'_> {
    #[must_use]
    pub fn state(&self) -> i16 {
        self.clear.map_or(0, |c| c.state)
    }

    pub fn is_cleared(&self) -> bool {
        self.clear.is_some_and(StageClear::is_cleared)
    }

    pub fn is_three_starred(&self) -> bool {
        if self.live_only {
            return self.is_cleared();
        }
        self.clear.is_some_and(StageClear::is_three_starred)
    }

    /// Weighted credit toward the pool grade
    /// 1.0 for three stars, 0.7 for a clear, 0 otherwise
    fn credit(&self) -> f64 {
        let score = if self.is_three_starred() {
            1.0
        } else {
            self.clear.map_or(0.0, StageClear::clear_score)
        };
        score * self.weight * self.decay
    }
}

pub struct PlayerPools<'a> {
    universe: &'a StageUniverse,
    clears: &'a HashMap<String, StageClear>,
    allowed: Option<&'a HashSet<String>>,
    now: i64,
    last_synced_ts: Option<i64>,
}

impl<'a> PlayerPools<'a> {
    #[must_use]
    pub const fn new(
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

    pub fn permanent(&self) -> impl Iterator<Item = PoolStage<'a>> + '_ {
        self.universe
            .permanent
            .iter()
            .filter(|e| self.on_server(&e.stage_id))
            .filter_map(|e| self.admit(&e.stage_id, e.weight, 1.0, e.optional, e.live_only))
    }

    pub fn event(&self) -> impl Iterator<Item = PoolStage<'a>> + '_ {
        self.universe
            .event
            .iter()
            .filter(|e| event_is_gradeable(e, self.now, self.last_synced_ts, self.allowed))
            .filter_map(|e| {
                let decay = decay_factor(e.end_time, self.now);
                self.admit(&e.stage_id, e.weight, decay, e.optional, e.live_only)
            })
    }

    fn on_server(&self, stage_id: &str) -> bool {
        self.allowed.is_none_or(|set| set.contains(stage_id))
    }

    fn admit(
        &self,
        stage_id: &'a str,
        weight: f64,
        decay: f64,
        optional: bool,
        live_only: bool,
    ) -> Option<PoolStage<'a>> {
        let stage = PoolStage {
            stage_id,
            weight,
            decay,
            clear: self.clears.get(stage_id),
            live_only,
        };
        (!(optional || live_only) || stage.is_cleared()).then_some(stage)
    }
}

pub fn weighted_score<'a>(stages: impl Iterator<Item = PoolStage<'a>>) -> f64 {
    let (credit, weight) = stages.fold((0.0, 0.0), |(credit, weight), s| {
        (credit + s.credit(), s.weight.mul_add(s.decay, weight))
    });
    if weight <= 0.0 {
        return 0.0;
    }
    (credit / weight).min(1.0)
}

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
