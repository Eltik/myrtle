#[derive(Debug, Default)]
pub struct StageTables {}

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

use crate::serde_helpers::{FbKeyValue, deserialize_fb_map};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct UnlockCondition {
    #[serde(alias = "StageId")]
    pub stage_id: String,

    #[serde(alias = "CompleteState")]
    pub complete_state: String,
}

/// Single item stage can drop, with drop category and occurrence band
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct DisplayDetailReward {
    /// e.g. `ONCE`, `NORMAL`, `SPECIAL`, `ADDITIONAL`, `COMPLETE`, `CONDITION_DROP`
    #[serde(alias = "DropType")]
    pub drop_type: String,

    #[serde(alias = "Id")]
    pub id: String,

    /// e.g. `ALWAYS`, `ALMOST`, `USUAL`, `OFTEN`, `SOMETIMES`, `RARELY`
    #[serde(alias = "OccPercent")]
    pub occ_percent: String,

    /// e.g. `MATERIAL`, `CHAR`, `CARD_EXP`, `DIAMOND`, etc.
    #[serde(alias = "Type_", alias = "Type")]
    pub item_type: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct StageDropInfo {
    #[serde(alias = "DisplayDetailRewards", default)]
    pub display_detail_rewards: Vec<DisplayDetailReward>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
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

    #[serde(alias = "DiamondOnceDrop", default)]
    pub diamond_once_drop: i32,

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

    #[serde(default)]
    #[serde(alias = "StageDropInfo", skip_serializing_if = "Option::is_none")]
    pub stage_drop_info: Option<StageDropInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct StageData {
    pub stages: HashMap<String, Stage>,
}

/// Arc header between two runs of mainline chapters (e.g. `HOUR OF AN AWAKENING`)
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MainlineSplitData {
    #[serde(default)]
    pub sub_name: Option<String>,
    #[serde(default)]
    pub icon_id: Option<String>,
}

/// One stop on a storyline. `STORY_SET` and `BEFORE` point at a
/// [`StorylineStorySet`], `MAINLINE_SPLIT` carries only an arc name
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StorylineLocation {
    pub location_id: String,
    #[serde(default)]
    pub location_type: String,
    #[serde(default)]
    pub relevant_story_set_id: Option<String>,
    #[serde(default)]
    pub mainline_split_data: Option<MainlineSplitData>,
    #[serde(default)]
    pub sort_id: i32,
}

/// Themed story shelves, e.g. `mainLine`, `ssLine_*`
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Storyline {
    pub storyline_id: String,
    #[serde(default)]
    pub storyline_name: String,
    #[serde(default)]
    pub storyline_type: String,
    /// Shelf header glyph (e.g. `storyline_abbr_Rl`), in `spritepack/mixstory_abbr_sprites_h2_0`
    #[serde(default)]
    pub storyline_icon_id: Option<String>,
    /// Shelf logotype (e.g. `storyline_Ms`), in `spritepack/mixstory_logo_sprites_0`
    #[serde(default)]
    pub storyline_logo_id: Option<String>,
    #[serde(default)]
    pub sort_id: i32,
    #[serde(default, deserialize_with = "deserialize_fb_list")]
    pub locations: Vec<StorylineLocation>,
}

/// Zone a mainline set covers, also the `story_review_table` group id
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MainlineSetData {
    #[serde(default)]
    pub zone_id: Option<String>,
    /// Chapter glyph (e.g. `deco_evil_time_part1`), in `spritepack/mixstory_deco_sprites_h2_0`.
    /// `MAINLINE` sets only
    #[serde(default)]
    pub deco_image_id: Option<String>,
}

/// Join between storyline locations and `story_review_table` groups.
/// `MAINLINE` joins on `MainlineData.ZoneId`, `SS`/`COLLECT` on `RelevantActivityId`
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StorylineStorySet {
    pub story_set_id: String,
    #[serde(default)]
    pub story_set_type: String,
    #[serde(default)]
    pub relevant_activity_id: Option<String>,
    #[serde(default)]
    pub mainline_data: Option<MainlineSetData>,
    /// Key visual (e.g. `kv_evil_time_part1`), in `spritepack/mixstory_kv_sprites_{0,1,2}`
    #[serde(default)]
    pub kv_image_id: Option<String>,
    /// Title logotype (e.g. `title_evil_time_part1`), in `spritepack/mixstory_title_sprites_0`
    #[serde(default)]
    pub title_image_id: Option<String>,
}

impl StorylineStorySet {
    #[must_use]
    pub fn group_id(&self) -> Option<&str> {
        if let Some(zone) = self
            .mainline_data
            .as_ref()
            .and_then(|m| m.zone_id.as_deref())
            .filter(|z| !z.is_empty())
        {
            return Some(zone);
        }
        self.relevant_activity_id
            .as_deref()
            .filter(|a| !a.is_empty())
    }
}

/// Ordered `FlatBuffer` `[{key, value}]` list, values only
fn deserialize_fb_list<'de, D, V>(deserializer: D) -> Result<Vec<V>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
{
    let items: Vec<FbKeyValue<String, V>> = Vec::deserialize(deserializer)?;
    Ok(items.into_iter().map(|kv| kv.value).collect())
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StageTableFile {
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub stages: HashMap<String, Stage>,

    /// In table order
    #[serde(default, deserialize_with = "deserialize_fb_list")]
    pub storylines: Vec<Storyline>,

    /// Keyed by `StorySetId`
    #[serde(default, deserialize_with = "deserialize_fb_map_story_sets")]
    pub storyline_story_sets: HashMap<String, StorylineStorySet>,
}

/// `StorylineStorySets` keyed by `StorySetId` instead of outer `key`
fn deserialize_fb_map_story_sets<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, StorylineStorySet>, D::Error>
where
    D: Deserializer<'de>,
{
    let items: Vec<FbKeyValue<String, StorylineStorySet>> = Vec::deserialize(deserializer)?;
    Ok(items
        .into_iter()
        .map(|kv| (kv.value.story_set_id.clone(), kv.value))
        .collect())
}
