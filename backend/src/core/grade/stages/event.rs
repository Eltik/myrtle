use std::collections::{HashMap, HashSet};

use crate::core::gamedata::types::stage_universe::StageUniverse;

use super::types::StageClear;

const DECAY_HORIZON_SECONDS: f64 = 5.0 * 365.25 * 86400.0;
const DECAY_FLOOR: f64 = 0.30;

/// Small grace window so an event that started moments before the user's sync
/// (or has clock skew) doesn't get filtered out.
pub const SYNC_GRACE_SECONDS: i64 = 12 * 60 * 60;

pub fn score_event_pool(
    universe: &StageUniverse,
    clears: &HashMap<String, StageClear>,
    now: i64,
    last_synced_ts: Option<i64>,
    allowed: Option<&HashSet<String>>,
) -> f64 {
    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for entry in &universe.event {
        if let Some(set) = allowed
            && !set.contains(&entry.stage_id)
        {
            continue;
        }
        if let (Some(start), Some(sync)) = (entry.start_time, last_synced_ts)
            && start > sync + SYNC_GRACE_SECONDS
        {
            continue;
        }

        let decay = decay_factor(entry.end_time, now);
        denominator += entry.weight * decay;

        if let Some(clear) = clears.get(&entry.stage_id) {
            numerator += clear.clear_score() * entry.weight * decay;
        }
    }

    if denominator <= 0.0 {
        return 0.0;
    }

    (numerator / denominator).min(1.0)
}

fn decay_factor(end_time: Option<i64>, now: i64) -> f64 {
    let Some(end) = end_time else {
        return 1.0;
    };
    if now <= end {
        return 1.0;
    }
    let age = (now - end) as f64;
    (1.0 - age / DECAY_HORIZON_SECONDS).max(DECAY_FLOOR)
}
