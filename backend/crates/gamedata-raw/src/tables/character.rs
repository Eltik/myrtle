use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::consts::GameDataConst;
use super::handbook::HandbookTableFile;
use super::handbook_team::HandbookTeamTableFile;
use super::item::SkinChannel;
use super::module::{BattleEquipTableFile, UniequipTableFile};
use super::operator::{CharPatchTable, CharacterTable};
use super::range::Ranges;
use super::skill::SkillTableFile;
use super::trust::Favor;
use crate::serde_helpers::deserialize_fb_map;

/// Operator, skill, module and handbook tables
#[derive(Debug, Default)]
pub struct CharacterTables {
    /// `character_table`, the one required table
    pub characters: CharacterTable,
    /// `char_patch_table`
    pub patch: CharPatchTable,
    /// `skill_table`
    pub skills: SkillTableFile,
    /// `uniequip_table`
    pub uniequip: UniequipTableFile,
    /// `battle_equip_table`
    pub battle_equip: BattleEquipTableFile,
    /// `handbook_info_table`
    pub handbook: HandbookTableFile,
    /// `handbook_team_table`
    pub handbook_team: HandbookTeamTableFile,
    /// `favor_table`
    pub favor: Favor,
    /// `gamedata_const`
    pub consts: GameDataConst,
    /// `skin_table`
    pub skins: SkinTableFile,
    /// `range_table`
    pub ranges: Ranges,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenSkinMapEntry {
    #[serde(alias = "TokenId")]
    pub token_id: String,
    #[serde(alias = "TokenSkinId")]
    pub token_skin_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BattleSkin {
    #[serde(alias = "OverwritePrefab", default)]
    pub overwrite_prefab: bool,
    #[serde(alias = "SkinOrPrefabId", default)]
    pub skin_or_prefab_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplaySkin {
    #[serde(alias = "SkinName")]
    pub skin_name: Option<String>,
    #[serde(alias = "ColorList", default)]
    pub color_list: Vec<String>,
    #[serde(alias = "TitleList", default)]
    pub title_list: Vec<String>,
    #[serde(alias = "ModelName", default)]
    pub model_name: String,
    #[serde(alias = "DrawerList", default)]
    pub drawer_list: Vec<String>,
    #[serde(alias = "DesignerList")]
    pub designer_list: Option<Vec<String>>,
    #[serde(alias = "SkinGroupId", default)]
    pub skin_group_id: String,
    #[serde(alias = "SkinGroupName", default)]
    pub skin_group_name: String,
    #[serde(alias = "SkinGroupSortIndex", default)]
    pub skin_group_sort_index: i32,
    #[serde(alias = "Content", default)]
    pub content: String,
    #[serde(alias = "Dialog")]
    pub dialog: Option<String>,
    #[serde(alias = "Usage")]
    pub usage: Option<String>,
    #[serde(alias = "Description")]
    pub description: Option<String>,
    #[serde(alias = "ObtainApproach")]
    pub obtain_approach: Option<String>,
    #[serde(alias = "SortId")]
    pub sort_id: i32,
    #[serde(alias = "DisplayTagId")]
    pub display_tag_id: Option<String>,
    #[serde(alias = "GetTime")]
    pub get_time: i64,
    #[serde(alias = "OnYear")]
    pub on_year: i32,
    #[serde(alias = "OnPeriod")]
    pub on_period: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandGroup {
    #[serde(alias = "SkinGroupId")]
    pub skin_group_id: String,
    #[serde(alias = "PublishTime")]
    pub publish_time: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandKvImg {
    #[serde(alias = "KvImgId")]
    pub kv_img_id: String,
    #[serde(alias = "LinkedSkinGroupId")]
    pub linked_skin_group_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Brand {
    #[serde(alias = "BrandId")]
    pub brand_id: String,
    #[serde(alias = "GroupList")]
    pub group_list: Vec<BrandGroup>,
    #[serde(alias = "KvImgIdList")]
    pub kv_img_id_list: Vec<BrandKvImg>,
    #[serde(alias = "BrandName")]
    pub brand_name: String,
    #[serde(alias = "BrandCapitalName")]
    pub brand_capital_name: String,
    #[serde(alias = "Description")]
    pub description: String,
    #[serde(alias = "PublishTime")]
    pub publish_time: i64,
    #[serde(alias = "SortId")]
    pub sort_id: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    #[serde(alias = "SkinId", default)]
    pub skin_id: String,
    #[serde(alias = "CharId", default)]
    pub char_id: String,
    #[serde(alias = "TokenSkinMap")]
    pub token_skin_map: Option<Vec<TokenSkinMapEntry>>,
    #[serde(alias = "IllustId", default)]
    pub illust_id: String,
    #[serde(alias = "DynIllustId")]
    pub dyn_illust_id: Option<String>,
    #[serde(alias = "AvatarId", default)]
    pub avatar_id: String,
    #[serde(alias = "PortraitId", default)]
    pub portrait_id: String,
    #[serde(alias = "DynPortraitId")]
    pub dyn_portrait_id: Option<String>,
    #[serde(alias = "DynEntranceId")]
    pub dyn_entrance_id: Option<String>,
    #[serde(alias = "BuildingId")]
    pub building_id: Option<String>,
    #[serde(alias = "BattleSkin", default)]
    pub battle_skin: BattleSkin,
    #[serde(alias = "IsBuySkin", default)]
    pub is_buy_skin: bool,
    #[serde(alias = "TmplId")]
    pub tmpl_id: Option<String>,
    #[serde(alias = "VoiceId")]
    pub voice_id: Option<String>,
    #[serde(alias = "SpDynIllustId")]
    pub sp_dyn_illust_id: Option<String>,
    #[serde(alias = "VoiceType", default)]
    pub voice_type: String,
    #[serde(alias = "DisplaySkin", default)]
    pub display_skin: DisplaySkin,
    /// `displaySkin.displayTagId` as a category
    #[serde(skip_deserializing)]
    pub obtain_channel: SkinChannel,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecialSkinInfo {
    #[serde(alias = "SkinId")]
    pub skin_id: String,
    #[serde(alias = "StartTime")]
    pub start_time: i64,
    #[serde(alias = "EndTime")]
    pub end_time: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SkinTableFile {
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub char_skins: HashMap<String, Skin>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub buildin_evolve_map: HashMap<String, Value>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub buildin_patch_map: HashMap<String, Value>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub brand_list: HashMap<String, Brand>,
    #[serde(default)]
    pub special_skin_info_list: Vec<SpecialSkinInfo>,
}
