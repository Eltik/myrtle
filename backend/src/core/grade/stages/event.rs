use std::collections::HashSet;

use crate::core::gamedata::types::stage_universe::EventEntry;

const DECAY_HORIZON_SECONDS: f64 = 5.0 * 365.25 * 86400.0;
const DECAY_FLOOR: f64 = 0.30;

/// Grace for an event that opened just before the sync, or clock skew.
pub const SYNC_GRACE_SECONDS: i64 = 12 * 60 * 60;

/// Whether an event stage counts toward the grade now.
///
/// Permanent events (`EventEntry::is_permanent`: rerun and in the retrospective
/// record) always count, subject to the per-server `allowed` cap. A limited event
/// that hasn't rerun counts only while open; after it ends it's unobtainable
/// until a rerun, so missing it isn't penalized. `last_synced_ts` keeps a
/// freshly opened event from reading "missing" before the user has synced clears.
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

/// Recency decay for a closed event: linear over five years down to a 0.30
/// floor, 1.0 while the event is open or has no end.
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
