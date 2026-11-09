use std::collections::HashSet;

use myrtle_gamedata::model::stage_universe::EventEntry;

const DECAY_HORIZON_SECONDS: f64 = 5.0 * 365.25 * 86400.0;
const DECAY_FLOOR: f64 = 0.30;

/// Grace for an event that opened just before the sync
pub const SYNC_GRACE_SECONDS: i64 = 12 * 60 * 60;

#[must_use]
pub fn event_is_gradeable(
    entry: &EventEntry,
    now: i64,
    last_synced_ts: Option<i64>,
    allowed: Option<&HashSet<String>>,
) -> bool {
    if let Some(set) = allowed
        && !set.contains(&entry.stage_id)
    {
        return false;
    }
    if entry.is_permanent {
        return true;
    }
    match (entry.start_time, entry.end_time) {
        (Some(start), Some(end)) => {
            start <= now
                && now <= end
                && last_synced_ts.is_none_or(|sync| start <= sync + SYNC_GRACE_SECONDS)
        }
        _ => false,
    }
}

pub(super) fn decay_factor(end_time: Option<i64>, now: i64) -> f64 {
    let Some(end) = end_time else {
        return 1.0;
    };
    if now <= end {
        return 1.0;
    }
    let age = (now - end) as f64;
    (1.0 - age / DECAY_HORIZON_SECONDS).max(DECAY_FLOOR)
}
