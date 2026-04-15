use std::collections::HashMap;

use crate::core::gamedata::types::stage_universe::StageUniverse;

use super::types::StageClear;

pub fn score_permanent_pool(universe: &StageUniverse, clears: &HashMap<String, StageClear>) -> f64 {
    if universe.permanent_max <= 0.0 {
        return 0.0;
    }

    let mut numerator = 0.0;
    for entry in &universe.permanent {
        if let Some(clear) = clears.get(&entry.stage_id) {
            numerator += clear.clear_score() * entry.weight;
        }
    }

    (numerator / universe.permanent_max).min(1.0)
}
