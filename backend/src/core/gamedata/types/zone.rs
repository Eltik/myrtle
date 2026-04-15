//! Zone table types for stage randomization.
//!
//! Zones represent chapters/regions in the game (e.g., Episode 1, Side Stories, etc.)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::serde_helpers::deserialize_fb_map;

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ZoneType {
    #[default]
    Mainline,
    Sidestory,
    Branchline,
    Activity,
    Weekly,
    Campaign,
    ClimbTower,
    Roguelike,
    Guide,
    Evolve,
    MainlineActivity,
    MainlineRetro,
    Special,
    #[serde(other)]
    Unknown,
}

// ============================================================================
// Zone
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Zone {
    #[serde(alias = "ZoneID")]
    pub zone_id: String,

    #[serde(alias = "ZoneIndex", default)]
    pub zone_index: i32,

    #[serde(alias = "Type_", rename = "type")]
    pub zone_type: ZoneType,

    #[serde(alias = "ZoneNameFirst")]
    pub zone_name_first: Option<String>,

    #[serde(alias = "ZoneNameSecond")]
    pub zone_name_second: Option<String>,

    #[serde(alias = "ZoneNameTitleCurrent")]
    pub zone_name_title_current: Option<String>,

    #[serde(alias = "ZoneNameTitleUnCurrent")]
    pub zone_name_title_un_current: Option<String>,

    #[serde(alias = "ZoneNameTitleEx")]
    pub zone_name_title_ex: Option<String>,

    #[serde(alias = "ZoneNameThird")]
    pub zone_name_third: Option<String>,

    #[serde(alias = "LockedText")]
    pub locked_text: Option<String>,

    #[serde(alias = "CanPreview", default)]
    pub can_preview: bool,

    #[serde(alias = "HasAdditionalPanel", default)]
    pub has_additional_panel: bool,
}

// ============================================================================
// Container Types
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoneData {
    pub zones: HashMap<String, Zone>,
}

// ============================================================================
// Table File Wrapper (for loading from FlatBuffer JSON)
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ZoneTableFile {
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub zones: HashMap<String, Zone>,
    // Other fields like WeeklyAdditionInfo, ZoneRecordRewardData, etc.
    // are not needed for the randomizer
}
