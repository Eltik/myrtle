use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
pub struct FavorFrame {
    #[serde(alias = "Level")]
    pub level: i32,
    #[serde(alias = "Data")]
    pub data: FavorFrameData,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Favor {
    #[serde(alias = "MaxFavor")]
    pub max_favor: i32,
    #[serde(alias = "FavorFrames")]
    pub favor_frames: Vec<FavorFrame>,
}

impl Favor {
    /// Raw favor points -> trust percent (0-200), 0.0 if table is empty
    pub fn trust_pct(&self, favor_point: i32) -> f64 {
        self.favor_frames
            .iter()
            .rev()
            .find(|f| f.level <= favor_point)
            .map_or(0.0, |f| f.data.percent)
    }

    /// e.g. 200.0
    pub fn max_trust_pct(&self) -> f64 {
        self.favor_frames.last().map_or(0.0, |f| f.data.percent)
    }
}
