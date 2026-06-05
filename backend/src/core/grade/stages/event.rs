use std::collections::{HashMap, HashSet};

use crate::core::gamedata::types::stage_universe::{EventEntry, StageUniverse};

use super::types::StageClear;

const DECAY_HORIZON_SECONDS: f64 = 5.0 * 365.25 * 86400.0;
const DECAY_FLOOR: f64 = 0.30;

/// Small grace window so an event that started moments before the user's sync
/// (or has clock skew) doesn't get filtered out.
pub const SYNC_GRACE_SECONDS: i64 = 12 * 60 * 60;

/// Whether an event stage should count toward a user's grade right now.
///
/// Permanent events (`EventEntry::is_permanent` - those that have rerun and
/// entered the retrospective record) always count, subject to the per-server
/// `allowed` cap. A limited event that has yet to rerun only counts while its
/// run window is currently open; once it ends it becomes unobtainable until it
/// reruns or goes permanent, so penalizing a player for missing it would be
/// unfair. The `last_synced_ts` guard keeps a freshly-opened event from showing
/// as "missing" before the user has synced clears for it.
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
        if !event_is_gradeable(entry, now, last_synced_ts, allowed) {
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
