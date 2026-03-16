//! Stage table types for stage randomization.
//!
//! Stages represent individual playable levels in the game.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::serde_helpers::deserialize_fb_map;

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StageType {
    #[default]
    Main,
    Sub,
    Activity,
    Daily,
    Campaign,
    ClimbTower,
    Guide,
    SpecialStory,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StageDifficulty {
    #[default]
    Normal,
    FourStar,
    SixStar,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppearanceStyle {
    #[default]
    MainNormal,
    Sub,
    Training,
    SpecialStory,
    HighDifficulty,
    MainPredefined,
    MistOps,
    #[serde(other)]
    Unknown,
}

// ============================================================================
// Nested Structs
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockCondition {
    #[serde(alias = "StageId")]
    pub stage_id: String,

    #[serde(alias = "CompleteState")]
    pub complete_state: String,
}

// ============================================================================
// Stage
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stage {
    #[serde(alias = "StageId")]
    pub stage_id: String,

    #[serde(alias = "LevelId")]
    pub level_id: Option<String>,

    #[serde(alias = "ZoneId")]
    pub zone_id: String,

    #[serde(alias = "Code")]
    pub code: String,

    #[serde(alias = "Name")]
    pub name: Option<String>,

    #[serde(alias = "Description")]
    pub description: Option<String>,

    #[serde(alias = "StageType")]
    pub stage_type: StageType,

    #[serde(alias = "Difficulty")]
    pub difficulty: StageDifficulty,

    #[serde(alias = "ApCost", default)]
    pub ap_cost: i32,

    #[serde(alias = "CanPractice", default)]
    pub can_practice: bool,

    #[serde(alias = "CanBattleReplay", default)]
    pub can_battle_replay: bool,

    #[serde(alias = "CanMultipleBattle", default)]
    pub can_multiple_battle: bool,

    #[serde(alias = "IsStoryOnly", default)]
    pub is_story_only: bool,

    #[serde(alias = "IsPredefined", default)]
    pub is_predefined: bool,

    #[serde(alias = "DangerLevel")]
    pub danger_level: Option<String>,

    #[serde(alias = "DangerPoint", default)]
    pub danger_point: f64,

    #[serde(alias = "ExpGain", default)]
    pub exp_gain: i32,

    #[serde(alias = "GoldGain", default)]
    pub gold_gain: i32,

    #[serde(alias = "AppearanceStyle")]
    pub appearance_style: Option<AppearanceStyle>,

    #[serde(alias = "HardStagedId")]
    pub hard_staged_id: Option<String>,

    #[serde(alias = "MainStageId")]
    pub main_stage_id: Option<String>,

    #[serde(alias = "UnlockCondition", default)]
    pub unlock_condition: Vec<UnlockCondition>,

    #[serde(alias = "LoadingPicId")]
    pub loading_pic_id: Option<String>,

    #[serde(alias = "BossMark", default)]
    pub boss_mark: bool,
}

// ============================================================================
// Container Types
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageData {
    pub stages: HashMap<String, Stage>,
}

// ============================================================================
// Table File Wrapper (for loading from FlatBuffer JSON)
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StageTableFile {
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub stages: HashMap<String, Stage>,
    // Other fields like TileInfo, MapThemes, etc.
    // are not needed for the randomizer
}
