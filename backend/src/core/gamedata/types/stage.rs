//! `stage_table` types.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

use super::serde_helpers::{FbKeyValue, deserialize_fb_map};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
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
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
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
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
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
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[ts(rename = "StageUnlockCondition")]
pub struct UnlockCondition {
    #[serde(alias = "StageId")]
    pub stage_id: String,

    #[serde(alias = "CompleteState")]
    pub complete_state: String,
}

/// A single item the stage can drop, with its drop category and occurrence band.
///
/// `drop_type` (ONCE / NORMAL / SPECIAL / ADDITIONAL / COMPLETE / `CONDITION_DROP`),
/// `occ_percent` (ALWAYS / ALMOST / USUAL / OFTEN / SOMETIMES / RARELY) and
/// `item_type` (MATERIAL / CHAR / `CARD_EXP` / DIAMOND / …) are kept as raw strings
/// for forward-compatibility with new game-data values.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct DisplayDetailReward {
    #[serde(alias = "DropType")]
    pub drop_type: String,

    #[serde(alias = "Id")]
    pub id: String,

    #[serde(alias = "OccPercent")]
    pub occ_percent: String,

    #[serde(alias = "Type_", alias = "Type")]
    pub item_type: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StageDropInfo {
    #[serde(alias = "DisplayDetailRewards", default)]
    pub display_detail_rewards: Vec<DisplayDetailReward>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
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
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StageData {
    pub stages: HashMap<String, Stage>,
}

// ============================================================================
// Storylines (the Archives' own shelves)
// ============================================================================

/// `StorylineLocation.MainlineSplitData`: the arc header the game draws
/// between two runs of mainline chapters (`HOUR OF AN AWAKENING`).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MainlineSplitData {
    #[serde(default)]
    pub sub_name: Option<String>,
    #[serde(default)]
    pub icon_id: Option<String>,
}

/// One stop on a storyline's rail. `STORY_SET` and `BEFORE` point at a
/// [`StorylineStorySet`]; `MAINLINE_SPLIT` carries an arc name and nothing
/// else.
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

/// `stage_table.Storylines`: the game's own themed shelves over the Archives,
/// `mainLine` plus 13 `ssLine_*` on EN.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Storyline {
    pub storyline_id: String,
    #[serde(default)]
    pub storyline_name: String,
    #[serde(default)]
    pub storyline_type: String,
    /// The small glyph the client draws on the shelf header
    /// (`storyline_abbr_Rl`), in `spritepack/mixstory_abbr_sprites_h2_0`. The
    /// `mainLine` shelf is the one row that ships without one.
    #[serde(default)]
    pub storyline_icon_id: Option<String>,
    /// The shelf's full logotype (`storyline_Ms`), in
    /// `spritepack/mixstory_logo_sprites_0`. All 14 EN rows carry one, so it
    /// is what `mainLine` falls back to.
    #[serde(default)]
    pub storyline_logo_id: Option<String>,
    #[serde(default)]
    pub sort_id: i32,
    #[serde(default, deserialize_with = "deserialize_fb_list")]
    pub locations: Vec<StorylineLocation>,
}

/// `StorylineStorySet.MainlineData`: a mainline set names the ZONE it covers,
/// which is also the `story_review_table` group id.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MainlineSetData {
    #[serde(default)]
    pub zone_id: Option<String>,
    /// The chapter's own small glyph (`deco_evil_time_part1`), in
    /// `spritepack/mixstory_deco_sprites_h2_0`. MAINLINE sets only: an `SS` or
    /// `COLLECT` set carries no deco.
    #[serde(default)]
    pub deco_image_id: Option<String>,
}

/// `stage_table.StorylineStorySets`: the join between a storyline's locations
/// and the Archives. A `MAINLINE` set joins on `MainlineData.ZoneId`, an `SS`
/// or `COLLECT` set on `RelevantActivityId`; both are `story_review_table`
/// group ids.
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
    /// The set's KEY VISUAL (`kv_evil_time_part1`), in
    /// `spritepack/mixstory_kv_sprites_{0,1,2}`: the card art the Story
    /// Collection draws for a chapter or an event. All 81 EN sets carry one.
    #[serde(default)]
    pub kv_image_id: Option<String>,
    /// The set's title logotype (`title_evil_time_part1`), in
    /// `spritepack/mixstory_title_sprites_0`. All 81 EN sets carry one; it is
    /// read here so a consumer can find it without a second table pass.
    #[serde(default)]
    pub title_image_id: Option<String>,
}

impl StorylineStorySet {
    /// The `story_review_table` group this set points at, by the rule above.
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

/// `FlatBuffer` writes an ORDERED list as `[{key, value}]` too. Storyline
/// locations are read as a list, because their order carries the arc splits
/// that a map would drop.
fn deserialize_fb_list<'de, D, V>(deserializer: D) -> Result<Vec<V>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
{
    let items: Vec<FbKeyValue<String, V>> = Vec::deserialize(deserializer)?;
    Ok(items.into_iter().map(|kv| kv.value).collect())
}

// ============================================================================
// Table File Wrapper (for loading from FlatBuffer JSON)
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StageTableFile {
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub stages: HashMap<String, Stage>,

    /// The Archives' themed shelves, in the table's own order.
    #[serde(default, deserialize_with = "deserialize_fb_list")]
    pub storylines: Vec<Storyline>,

    /// Keyed by `StorySetId`, which is what a location's
    /// `RelevantStorySetId` names.
    #[serde(default, deserialize_with = "deserialize_fb_map_story_sets")]
    pub storyline_story_sets: HashMap<String, StorylineStorySet>,
    // Other fields like TileInfo, MapThemes, etc.
    // are not needed for the randomizer
}

/// `StorylineStorySets` keyed by `StorySetId` rather than by the outer `key`,
/// which is the same string but is not guaranteed to be.
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
