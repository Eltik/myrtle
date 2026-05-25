use std::collections::{HashMap, HashSet};

use crate::core::gamedata::types::stage_universe::StageUniverse;

use super::types::StageClear;

/// Score the permanent pool, optionally restricting to a server-specific set
/// of stage IDs. Entries not in `allowed` (None means accept-all) contribute
/// to neither numerator nor denominator, so a server that doesn't have the
/// stage at all isn't penalized.
pub fn score_permanent_pool(
    universe: &StageUniverse,
    clears: &HashMap<String, StageClear>,
    allowed: Option<&HashSet<String>>,
) -> f64 {
    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for entry in &universe.permanent {
        if let Some(set) = allowed
            && !set.contains(&entry.stage_id)
        {
            continue;
        }
        denominator += entry.weight;
        if let Some(clear) = clears.get(&entry.stage_id) {
            numerator += clear.clear_score() * entry.weight;
        }
    }

    if denominator <= 0.0 {
        return 0.0;
    }
    (numerator / denominator).min(1.0)
}
