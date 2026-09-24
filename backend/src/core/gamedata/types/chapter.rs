//! `chapter_table` types: the main story's chapter names (`Hour of An
//! Awakening`, ...) and the zone range each spans. Chapter 3's range is
//! `act2mainss_zone1..act3mainss_zone1`, not `main_15..main_16`, so the
//! zone -> chapter link is read off `zone_table.MainlineAdditionInfo`
//! (`ChapterId` per zone) first and the numeric range is the fallback.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

use super::serde_helpers::deserialize_fb_map_or_default;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
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

/// `zone_table.MainlineAdditionInfo[zone]`: the chapter a main zone belongs
/// to, and when the zone opened. `ZoneOpenTime` is a real unix time for
/// `main_10` through `main_14` on EN and `-1` for the twelve older zones,
/// which is the only release date the main story carries: every mainline
/// `story_review_table` row writes `StartTime: -1`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MainlineAdditionInfo {
    #[serde(alias = "ZoneId", default)]
    pub zone_id: String,
    #[serde(alias = "ChapterId", default)]
    pub chapter_id: String,
    #[serde(alias = "ZoneOpenTime", default)]
    pub zone_open_time: i64,
}
