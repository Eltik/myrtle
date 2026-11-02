use std::collections::{HashMap, HashSet};

use myrtle_gamedata_raw::tables::activity::ActivityBasicInfo;
use myrtle_gamedata_raw::tables::stage::StageDifficulty;
use myrtle_gamedata_raw::tables::zone::Zone;

use super::enemy_stages::relative_level_id;
use super::stage_class::{StageClassifier, StageInfo};

const fn indexed_difficulty(d: &StageDifficulty) -> StageDifficulty {
    match d {
        StageDifficulty::FourStar => StageDifficulty::FourStar,
        StageDifficulty::SixStar => StageDifficulty::SixStar,
        StageDifficulty::Normal | StageDifficulty::Unknown => StageDifficulty::Normal,
    }
}

fn zone_order(
    group: &str,
    zone_id: &str,
    zones: &HashMap<String, Zone>,
    activities: &HashMap<String, ActivityBasicInfo>,
) -> i64 {
    let zone = zones.get(zone_id);
    if group == "story"
        && let Some(z) = zone
        && let Some(n) = z
            .zone_name_title_current
            .as_deref()
            .and_then(|s| s.trim().parse::<i64>().ok())
    {
        return n;
    }
    if let Some(a) = activities.get(zone_id)
        && a.start_time > 0
    {
        return a.start_time;
    }
    zone.map_or(0, |z| i64::from(z.zone_index))
}

fn level_exists(level_files: &HashSet<&str>, level_id: Option<&str>) -> bool {
    let Some(level_id) = level_id else {
        return false;
    };
    let rel = level_id.to_lowercase().replace('\\', "/");
    level_files.contains(format!("{rel}.json").as_str())
}
