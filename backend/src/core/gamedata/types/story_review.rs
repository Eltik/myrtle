//! `story_review_table` types: the Archives library index (story groups and
//! the stories under each). EN carries 451 groups and 1,887 stories; the 364
//! `EntryType: NONE` groups are all operator records (`obt/memory/...`), the
//! same stories `handbook_info_table` lists per operator.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

use super::serde_helpers::deserialize_fb_map_or_default;

/// One stage gate on a story (`RequiredStages[]`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StoryRequiredStage {
    #[serde(alias = "StageId", default)]
    pub stage_id: String,
    #[serde(alias = "MinState", default)]
    pub min_state: String,
    #[serde(alias = "MaxState", default)]
    pub max_state: String,
}

/// One story under a group (`InfoUnlockDatas[]`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StoryUnlockData {
    #[serde(alias = "StoryId", default)]
    pub story_id: String,
    #[serde(alias = "StoryGroup", default)]
    pub story_group: String,
    #[serde(alias = "StoryName", default)]
    pub story_name: String,
    /// The operation code (`GT-1`); absent on records and interludes.
    #[serde(alias = "StoryCode", default)]
    pub story_code: Option<String>,
    #[serde(alias = "StorySort", default)]
    pub story_sort: i32,
    /// `Before Operation`, `After Operation` or `Interlude`.
    #[serde(alias = "AvgTag", default)]
    pub avg_tag: Option<String>,
    /// Script path under `gamedata/story/`, without extension.
    #[serde(alias = "StoryTxt", default)]
    pub story_txt: String,
    /// Summary path under `gamedata/story/`, without extension.
    #[serde(alias = "StoryInfo", default)]
    pub story_info: Option<String>,
    #[serde(alias = "StoryDependence", default)]
    pub story_dependence: Option<String>,
    #[serde(alias = "RequiredStages", default)]
    pub required_stages: Vec<StoryRequiredStage>,
    #[serde(alias = "UnLockType", default)]
    pub un_lock_type: String,
    #[serde(alias = "StoryReviewType", default)]
    pub story_review_type: String,
}

/// One Archives group: a main chapter, an event, or an operator record set.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StoryReviewGroup {
    #[serde(alias = "Id", default)]
    pub id: String,
    #[serde(alias = "Name", default)]
    pub name: String,
    /// `MAINLINE`, `ACTIVITY`, `MINI_ACTIVITY` or `NONE`.
    #[serde(alias = "EntryType", default)]
    pub entry_type: String,
    /// `MAIN_STORY`, `ACTIVITY_STORY`, `MINI_STORY` or `NONE`.
    #[serde(alias = "ActType", default)]
    pub act_type: String,
    /// `storyEntryPic_{id}`, the Archives cover; 70 of 451 groups carry one on EN.
    #[serde(alias = "StoryEntryPicId", default)]
    pub story_entry_pic_id: Option<String>,
    #[serde(alias = "StartTime", default)]
    #[ts(type = "number")]
    pub start_time: i64,
    #[serde(alias = "InfoUnlockDatas", default)]
    pub info_unlock_datas: Vec<StoryUnlockData>,
}

// ============================================================================
// Table File Wrapper (for loading from FlatBuffer JSON)
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
pub struct StoryReviewTableFile {
    /// The unpacker writes this key as `Story_reviews`, not `StoryReviews`.
    #[serde(
        alias = "Story_reviews",
        alias = "StoryReviews",
        alias = "storyreviewtable",
        deserialize_with = "deserialize_fb_map_or_default",
        default
    )]
    pub story_reviews: HashMap<String, StoryReviewGroup>,
}
