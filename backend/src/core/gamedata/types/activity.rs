//! Acitivty table types, used for event start/end times

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::serde_helpers::deserialize_fb_map;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityBasicInfo {
    #[serde(alias = "Id")]
    pub id: String,

    #[serde(alias = "Name", default)]
    pub name: String,

    #[serde(alias = "StartTime", default)]
    pub start_time: i64,

    #[serde(alias = "EndTime", default)]
    pub end_time: i64,

    #[serde(alias = "HasStage", default)]
    pub has_stage: bool,

    #[serde(alias = "IsReplicate", default)]
    pub is_replicate: bool,

    /// Activity category (e.g. `TYPE_ACT46SIDE`, `MULTIPLAY_V3`, `BOSS_RUSH`).
    /// Used by the stage universe to filter out one-time competitive events
    /// (Contingency Contract, Boss Rush, Vector Breakthrough, etc.) that can't
    /// be cleared after they end and aren't rebroadcast.
    #[serde(alias = "Type_", rename = "type", default)]
    pub activity_type: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ActivityTableFile {
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub basic_info: HashMap<String, ActivityBasicInfo>,
    // Don't need rest of data
}
