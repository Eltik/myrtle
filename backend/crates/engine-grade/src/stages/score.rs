use std::collections::HashSet;

use crate::{
    GradeData, StageClears,
    stages::{PlayerPools, PoolCounts, weighted_score},
};

const PERMANENT_POOL_WEIGHT: f64 = 0.70;
const EVENT_POOL_WEIGHT: f64 = 0.30;

#[derive(Debug, Clone)]
pub struct StageGradeDetail {
    pub total: f64,
    pub permanent_score: f64,
    pub event_score: f64,
    pub permanent: PoolCounts,
    pub event: PoolCounts,
}

#[must_use]
pub fn grade_stages(
    game_data: &GradeData,
    data: &StageClears,
    known_stage_ids: &HashSet<String>,
    now: i64,
) -> f64 {
    grade_stages_detail(game_data, data, known_stage_ids, now).total
}

#[must_use]
pub fn grade_stages_detail(
    game_data: &GradeData,
    data: &StageClears,
    known_stage_ids: &HashSet<String>,
    now: i64,
) -> StageGradeDetail {
    let pools = PlayerPools::new(
        &game_data.stage_universe,
        &data.clears,
        Some(known_stage_ids),
        now,
        data.last_synced_ts,
    );

    let permanent_score = weighted_score(pools.permanent());
    let event_score = weighted_score(pools.event());
    let total = EVENT_POOL_WEIGHT.mul_add(event_score, PERMANENT_POOL_WEIGHT * permanent_score)
        .clamp(0.0, 1.0);

    StageGradeDetail {
        total,
        permanent_score,
        event_score,
        permanent: PoolCounts::of(pools.permanent()),
        event: PoolCounts::of(pools.event()),
    }
}
