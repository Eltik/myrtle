use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::serde_helpers::deserialize_fb_map_or_default;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    #[serde(alias = "ChapterId", default)]
    pub chapter_id: String,
    #[serde(alias = "ChapterName", default)]
    pub chapter_name: String,
    #[serde(alias = "ChapterName2", default)]
    pub chapter_name2: Option<String>,
    #[serde(alias = "ChapterIndex", default)]
    pub chapter_index: i32,
    #[serde(alias = "StartZoneId", default)]
    pub start_zone_id: String,
    #[serde(alias = "EndZoneId", default)]
    pub end_zone_id: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ChapterTableFile {
    #[serde(
        alias = "Chapters",
        deserialize_with = "deserialize_fb_map_or_default",
        default
    )]
    pub chapters: HashMap<String, Chapter>,
}

/// `zone_table.MainlineAdditionInfo[zone]`, chapter a main zone belongs to
/// and when it opened (`ZoneOpenTime` unix time, `-1` for older zones)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MainlineAdditionInfo {
    #[serde(alias = "ZoneId", default)]
    pub zone_id: String,
    #[serde(alias = "ChapterId", default)]
    pub chapter_id: String,
    #[serde(alias = "ZoneOpenTime", default)]
    pub zone_open_time: i64,
}
