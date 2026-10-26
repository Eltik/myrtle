use std::collections::HashMap;

use serde::Deserialize;

use crate::serde_helpers::deserialize_fb_map_or_default;

/// `campaign_table.json`: rotation schedule and per-map kill ladders
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

/// Open/close window for one rotating Annihilation map
#[derive(Debug, Clone, Copy)]
pub struct RotationWindow {
    pub start_ts: i64,
    pub end_ts: i64,
}

/// Rotating Annihilation map's status relative to a reference time
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationStatus {
    /// Window contains `now`
    Active,
    /// Window closed
    Past,
    /// Window not yet open
    Future,
}

/// Rotation schedule by stage id, rotating maps (`camp_r_*`) only
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

    /// Full-clear kill count (highest `BreakLadders` milestone), `None` if not in the table
    pub fn kill_max(&self, stage_id: &str) -> Option<i32> {
        self.kill_max.get(stage_id).copied()
    }

    pub fn window(&self, stage_id: &str) -> Option<RotationWindow> {
        self.windows.get(stage_id).copied()
    }

    /// Rotating map's availability at `now`, `None` for non-rotating stages
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
