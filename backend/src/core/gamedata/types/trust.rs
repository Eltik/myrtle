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
