//! Retro table types, used to map permanent (retro) events to their linked activities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::serde_helpers::deserialize_fb_map_or_default;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetroAct {
    #[serde(alias = "RetroId")]
    pub retro_id: String,

    #[serde(alias = "Name", default)]
    pub name: String,

    #[serde(alias = "Index", default)]
    pub index: i32,

    #[serde(alias = "StartTime", default)]
    pub start_time: i64,

    #[serde(alias = "Type_", alias = "Type", default)]
    pub r#type: String,

    #[serde(alias = "LinkedActId", default)]
    pub linked_act_id: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RetroTableFile {
    #[serde(deserialize_with = "deserialize_fb_map_or_default", default)]
    pub retro_act_list: HashMap<String, RetroAct>,
    // Don't need the rest of the table
}
