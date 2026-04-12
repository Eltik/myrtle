//! Acitivty table types, used for event start/end times

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::serde_helpers::deserialize_fb_map;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityBasicInfo {
    #[serde(alias = "Id")]
    pub id: String,

    #[serde(alias = "StartTime", default)]
    pub start_time: i64,

    #[serde(alias = "EndTime", default)]
    pub end_time: i64,

    #[serde(alias = "HasStage", default)]
    pub has_stage: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ActivityTableFile {
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub basic_info: HashMap<String, ActivityBasicInfo>,
    // Don't need rest of data
}
