//! Stationary Security Service (climbing tower) schedule from
//! `climb_tower_table.json`.
//!
//! SSS runs in ~4-month seasons, each opening a couple of towers; towers also
//! get replicated into later seasons. A tower is only playable while one of its
//! seasons is live, so medals that require completing a tower (`PassTower`) are
//! seasonal even though the medal table leaves their `ExpireTimes` empty (which
//! would otherwise read as permanently available).

use serde::Deserialize;
use std::collections::HashMap;

/// Root of `climb_tower_table.json` - only the season schedule is parsed.
/// `SeasonInfos` is a FlatBuffer-style `[{key, value}]` map.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ClimbTowerTableFile {
    #[serde(default)]
    pub season_infos: Vec<ClimbTowerSeasonEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClimbTowerSeasonEntry {
    pub value: ClimbTowerSeason,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ClimbTowerSeason {
    pub start_ts: i64,
    pub end_ts: i64,
    /// Towers introduced this season.
    #[serde(default)]
    pub towers: Vec<String>,
    /// Towers brought back from earlier seasons.
    #[serde(default)]
    pub replicated_towers: Vec<String>,
}

impl ClimbTowerTableFile {
    /// `tower_id -> [(season_start, season_end)]` across every season the tower
    /// appears in (introduced or replicated).
    pub fn tower_windows(&self) -> HashMap<String, Vec<(i64, i64)>> {
        let mut windows: HashMap<String, Vec<(i64, i64)>> = HashMap::new();
        for entry in &self.season_infos {
            let season = &entry.value;
            for tower_id in season.towers.iter().chain(season.replicated_towers.iter()) {
                windows
                    .entry(tower_id.clone())
                    .or_default()
                    .push((season.start_ts, season.end_ts));
            }
        }
        windows
    }
}
