//! `zone_table` types: chapters and regions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

use super::serde_helpers::{deserialize_fb_map, deserialize_fb_map_or_default};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
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
    /// Main zone -> chapter link, read by the story reader.
    #[serde(deserialize_with = "deserialize_fb_map_or_default", default)]
    pub mainline_addition_info: HashMap<String, super::chapter::MainlineAdditionInfo>,
    // Other fields like WeeklyAdditionInfo, ZoneRecordRewardData, etc.
    // are not needed for the randomizer
}
