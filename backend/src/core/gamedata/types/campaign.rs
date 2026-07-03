//! Annihilation (campaign) rotation schedule from `campaign_table.json`.
//!
//! Rotating Annihilation maps (`camp_r_*`) are only playable during their
//! scheduled window - only one is active at a time, alongside the three
//! permanent maps (`camp_01/02/03`, which never appear in this table and are
//! always playable). `CampaignRotateStageOpenTimes` gives each rotating map a
//! definite open/close window, which we use to tell "you can max this now"
//! apart from "this rotated out and is currently locked."

use serde::Deserialize;
use std::collections::HashMap;

use super::serde_helpers::deserialize_fb_map_or_default;

/// Root of `campaign_table.json` - the rotation schedule plus per-map kill
/// ladders (for the full-clear kill count).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CampaignTableFile {
    #[serde(default)]
    pub campaign_rotate_stage_open_times: Vec<CampaignRotateOpenTime>,
    #[serde(default, deserialize_with = "deserialize_fb_map_or_default")]
    pub campaigns: HashMap<String, CampaignStage>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CampaignStage {
    #[serde(default)]
    pub break_ladders: Vec<KillLadder>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KillLadder {
    #[serde(default)]
    pub kill_cnt: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CampaignRotateOpenTime {
    pub stage_id: String,
    #[serde(rename = "StartTs")]
    pub start_ts: i64,
    #[serde(rename = "EndTs")]
    pub end_ts: i64,
}

/// Open/close window for one rotating Annihilation map.
#[derive(Debug, Clone, Copy)]
pub struct RotationWindow {
    pub start_ts: i64,
    pub end_ts: i64,
}

/// Where a rotating Annihilation map sits relative to a reference time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationStatus {
    /// Window contains `now` - currently playable.
    Active,
    /// Window already closed - rotated out, not currently playable.
    Past,
    /// Window hasn't opened yet.
    Future,
}

/// Rotation schedule indexed by stage id. Only rotating maps (`camp_r_*`)
/// appear here; permanent maps and non-Annihilation stages return `None`.
#[derive(Debug, Clone, Default)]
pub struct CampaignRotations {
    windows: HashMap<String, RotationWindow>,
    kill_max: HashMap<String, i32>,
}

impl CampaignRotations {
    pub fn from_table(table: CampaignTableFile) -> Self {
        let windows = table
            .campaign_rotate_stage_open_times
            .into_iter()
            .filter(|w| w.end_ts > 0)
            .map(|w| {
                (
                    w.stage_id,
                    RotationWindow {
                        start_ts: w.start_ts,
                        end_ts: w.end_ts,
                    },
                )
            })
            .collect();
        let kill_max = table
            .campaigns
            .into_iter()
            .filter_map(|(stage_id, c)| {
                let max = c.break_ladders.iter().map(|l| l.kill_cnt).max()?;
                (max > 0).then_some((stage_id, max))
            })
            .collect();
        Self { windows, kill_max }
    }

    /// Kill count for a full clear of an Annihilation map - the highest
    /// `BreakLadders` milestone (400 for every map to date). `None` for
    /// stages not in the campaign table.
    pub fn kill_max(&self, stage_id: &str) -> Option<i32> {
        self.kill_max.get(stage_id).copied()
    }

    /// The scheduled window for a rotating map, if it is one.
    pub fn window(&self, stage_id: &str) -> Option<RotationWindow> {
        self.windows.get(stage_id).copied()
    }

    /// Classify a rotating map's availability at `now`. Returns `None` for
    /// stages that aren't rotating maps (permanent Annihilation, other stages).
    pub fn status(&self, stage_id: &str, now: i64) -> Option<RotationStatus> {
        let w = self.windows.get(stage_id)?;
        Some(if now < w.start_ts {
            RotationStatus::Future
        } else if now <= w.end_ts {
            RotationStatus::Active
        } else {
            RotationStatus::Past
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guard against the real table's shape drifting away from the parser
    /// (kill ladders live in `Campaigns[].BreakLadders[].KillCnt`). Skips
    /// when the gamedata assets aren't checked out.
    #[test]
    fn kill_max_parses_from_real_campaign_table() {
        let path = std::path::Path::new("../assets/output/en/gamedata/excel/campaign_table.json");
        let Ok(text) = std::fs::read_to_string(path) else {
            return;
        };
        let table: CampaignTableFile = serde_json::from_str(&text).unwrap();
        let rotations = CampaignRotations::from_table(table);
        assert_eq!(rotations.kill_max("camp_01"), Some(400));
        assert_eq!(rotations.kill_max("camp_r_01"), Some(400));
        assert_eq!(rotations.kill_max("main_00-01"), None);
    }
}
