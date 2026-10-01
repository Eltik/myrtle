use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct FavorFrameData {
    #[serde(alias = "FavorPoint")]
    pub favor_point: i32,
    #[serde(alias = "Percent")]
    pub percent: f64,
    #[serde(alias = "BattlePhase")]
    pub battle_phase: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct FavorFrame {
    #[serde(alias = "Level")]
    pub level: i32,
    #[serde(alias = "Data")]
    pub data: FavorFrameData,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct Favor {
    #[serde(alias = "MaxFavor")]
    pub max_favor: i32,
    #[serde(alias = "FavorFrames")]
    pub favor_frames: Vec<FavorFrame>,
}

impl Favor {
    /// Raw favor-point count -> trust percent (0-200) using the game's frame
    /// thresholds; 0.0 when the table is empty (game data not loaded). Frames
    /// ascend by level, so scan in reverse for the highest one reached.
    pub fn trust_pct(&self, favor_point: i32) -> f64 {
        self.favor_frames
            .iter()
            .rev()
            .find(|f| f.level <= favor_point)
            .map_or(0.0, |f| f.data.percent)
    }

    /// Maximum trust percent defined by the table (typically 200.0).
    pub fn max_trust_pct(&self) -> f64 {
        self.favor_frames.last().map_or(0.0, |f| f.data.percent)
    }
}
