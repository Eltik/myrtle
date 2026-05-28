use serde::{Deserialize, Serialize};

// ============================================================================
// Nested Structs
// ============================================================================

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

// ============================================================================
// Favor
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Favor {
    #[serde(alias = "MaxFavor")]
    pub max_favor: i32,
    #[serde(alias = "FavorFrames")]
    pub favor_frames: Vec<FavorFrame>,
}

impl Favor {
    /// Resolves a raw favor-point count to its trust percent (0–200) using the
    /// frame thresholds shipped with the game. Returns 0.0 if the table is
    /// empty (e.g. game data not loaded).
    ///
    /// Frames are sorted ascending by level in the source data; we scan in
    /// reverse to grab the highest threshold the player has reached.
    pub fn trust_pct(&self, favor_point: i32) -> f64 {
        self.favor_frames
            .iter()
            .rev()
            .find(|f| f.level <= favor_point)
            .map_or(0.0, |f| f.data.percent)
    }

    /// Maximum trust percent defined by the table (typically 200.0).
    pub fn max_trust_pct(&self) -> f64 {
        self.favor_frames
            .last()
            .map_or(0.0, |f| f.data.percent)
    }
}
