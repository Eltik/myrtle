use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::material::ItemType;
use super::serde_helpers::deserialize_fb_map;

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ModuleType {
    Initial,
    Advanced,
}

impl Default for ModuleType {
    fn default() -> Self {
        Self::Initial
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModuleTarget {
    Trait,
    TraitDataOnly,
    TalentDataOnly,
    Talent,
    Display,
    OverwriteBattleData,
    #[serde(other)]
    Unknown,
}

impl Default for ModuleTarget {
    fn default() -> Self {
        Self::Trait
    }
}

// ============================================================================
// Nested Structs
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleItemCost {
    #[serde(alias = "Id")]
    pub id: String,
    #[serde(alias = "Count")]
    pub count: i32,
    #[serde(rename = "type", alias = "Type_")]
    pub item_type: ItemType,
    #[serde(default)]
    pub icon_id: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubProfession {
    #[serde(alias = "SubProfessionId")]
    pub sub_profession_id: String,
    #[serde(alias = "SubProfessionName")]
    pub sub_profession_name: String,
    #[serde(alias = "SubProfessionCategory", alias = "SubProfessionCatagory")]
    pub sub_profession_category: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipTrackItem {
    #[serde(alias = "CharId")]
    pub char_id: String,
    #[serde(alias = "EquipId")]
    pub equip_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipTrack {
    #[serde(alias = "Timestamp")]
    pub timestamp: i64,
    #[serde(alias = "TrackList")]
    pub track_list: Vec<EquipTrackItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mission {
    #[serde(alias = "Template")]
    pub template: String,
    #[serde(alias = "Desc")]
    pub desc: String,
    #[serde(alias = "ParamList")]
    pub param_list: Vec<String>,
    #[serde(alias = "UniEquipMissionId")]
    pub uni_equip_mission_id: String,
    #[serde(alias = "UniEquipMissionSort")]
    pub uni_equip_mission_sort: i32,
    #[serde(alias = "UniEquipId")]
    pub uni_equip_id: String,
    #[serde(alias = "JumpStageId")]
    pub jump_stage_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleBlackboard {
    #[serde(alias = "Key", alias = "key")]
    pub key: String,
    #[serde(alias = "Value", alias = "value", default)]
    pub value: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleUnlockCondition {
    #[serde(alias = "Phase")]
    pub phase: String,
    #[serde(alias = "Level")]
    pub level: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddModuleCandidates {
    #[serde(alias = "DisplayRangeId", default)]
    pub display_range_id: bool,
    #[serde(alias = "UpgradeDescription", default)]
    pub upgrade_description: String,
    #[serde(alias = "TalentIndex", default)]
    pub talent_index: i32,
    #[serde(alias = "UnlockCondition", default)]
    pub unlock_condition: ModuleUnlockCondition,
    #[serde(alias = "RequiredPotentialRank", default)]
    pub required_potential_rank: i32,
    #[serde(alias = "PrefabKey")]
    pub prefab_key: Option<String>,
    #[serde(alias = "Name", default)]
    pub name: String,
    #[serde(alias = "Description")]
    pub description: Option<String>,
    #[serde(alias = "RangeId")]
    pub range_id: Option<String>,
    #[serde(alias = "Blackboard", default)]
    pub blackboard: Vec<ModuleBlackboard>,
    #[serde(alias = "TokenKey")]
    pub token_key: Option<String>,
    #[serde(alias = "IsHideTalent")]
    pub is_hide_talent: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleCandidates {
    #[serde(alias = "AdditionalDescription", default)]
    pub additional_description: String,
    #[serde(alias = "UnlockCondition", default)]
    pub unlock_condition: ModuleUnlockCondition,
    #[serde(alias = "RequiredPotentialRank", default)]
    pub required_potential_rank: i32,
    #[serde(alias = "Blackboard", default)]
    pub blackboard: Vec<ModuleBlackboard>,
    #[serde(alias = "OverrideDescription", alias = "OverrideDescripton")]
    pub override_description: Option<String>,
    #[serde(alias = "PrefabKey")]
    pub prefab_key: Option<String>,
    #[serde(alias = "RangeId")]
    pub range_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddOrOverrideTalentDataBundle {
    #[serde(alias = "Candidates")]
    pub candidates: Option<Vec<AddModuleCandidates>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverrideTraitDataBundle {
    #[serde(alias = "Candidates")]
    pub candidates: Option<Vec<ModuleCandidates>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModulePart {
    #[serde(alias = "ResKey")]
    pub res_key: Option<String>,
    #[serde(alias = "Target")]
    pub target: ModuleTarget,
    #[serde(alias = "IsToken")]
    pub is_token: bool,
    #[serde(alias = "AddOrOverrideTalentDataBundle")]
    pub add_or_override_talent_data_bundle: AddOrOverrideTalentDataBundle,
    #[serde(alias = "OverrideTraitDataBundle")]
    pub override_trait_data_bundle: OverrideTraitDataBundle,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModulePhase {
    #[serde(alias = "EquipLevel")]
    pub equip_level: i32,
    #[serde(alias = "Parts")]
    pub parts: Vec<ModulePart>,
    #[serde(alias = "AttributeBlackboard")]
    pub attribute_blackboard: Vec<ModuleBlackboard>,
    #[serde(alias = "TokenAttributeBlackboard", default)]
    pub token_attribute_blackboard: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleData {
    #[serde(alias = "Phases")]
    pub phases: Vec<ModulePhase>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawModule {
    #[serde(alias = "UniEquipId")]
    pub uni_equip_id: String,
    #[serde(alias = "UniEquipName")]
    pub uni_equip_name: String,
    #[serde(alias = "UniEquipIcon")]
    pub uni_equip_icon: String,
    #[serde(alias = "UniEquipDesc")]
    pub uni_equip_desc: String,
    #[serde(alias = "TypeIcon")]
    pub type_icon: String,
    #[serde(alias = "TypeName1")]
    pub type_name1: String,
    #[serde(alias = "TypeName2")]
    pub type_name2: Option<String>,
    #[serde(alias = "EquipShiningColor")]
    pub equip_shining_color: String,
    #[serde(alias = "ShowEvolvePhase")]
    pub show_evolve_phase: String,
    #[serde(alias = "UnlockEvolvePhase")]
    pub unlock_evolve_phase: String,
    #[serde(alias = "CharId")]
    pub char_id: String,
    #[serde(alias = "TmplId")]
    pub tmpl_id: Option<String>,
    #[serde(alias = "ShowLevel")]
    pub show_level: i32,
    #[serde(alias = "UnlockLevel")]
    pub unlock_level: i32,
    #[serde(alias = "UnlockFavorPoint", default)]
    pub unlock_favor_point: i32,
    #[serde(alias = "MissionList")]
    pub mission_list: Vec<String>,
    #[serde(alias = "ItemCost", default)]
    pub item_cost: Option<Vec<serde_json::Value>>,
    #[serde(rename = "type", alias = "Type_")]
    pub module_type: ModuleType,
    #[serde(alias = "UniEquipGetTime")]
    pub uni_equip_get_time: i64,
    #[serde(alias = "CharEquipOrder")]
    pub char_equip_order: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    pub id: Option<String>,
    pub uni_equip_id: String,
    pub uni_equip_name: String,
    pub uni_equip_icon: String,
    pub image: Option<String>,
    pub uni_equip_desc: String,
    pub type_icon: String,
    pub type_name1: String,
    pub type_name2: Option<String>,
    pub equip_shining_color: String,
    pub show_evolve_phase: String,
    pub unlock_evolve_phase: String,
    pub char_id: String,
    pub tmpl_id: Option<String>,
    pub show_level: i32,
    pub unlock_level: i32,
    pub unlock_favor_point: i32,
    pub mission_list: Vec<String>,
    pub item_cost: Option<HashMap<String, ModuleItemCost>>,
    #[serde(rename = "type")]
    pub module_type: ModuleType,
    pub uni_equip_get_time: i64,
    pub char_equip_order: i32,
}

// ============================================================================
// Container Types
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Modules {
    pub equip_dict: HashMap<String, Module>,
    pub mission_list: HashMap<String, Mission>,
    pub sub_prof_dict: HashMap<String, SubProfession>,
    pub char_equip: HashMap<String, Vec<String>>,
    pub equip_track_dict: HashMap<String, EquipTrack>,
    #[serde(skip)]
    pub battle_equip: BattleEquip,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawModules {
    pub equip_dict: HashMap<String, RawModule>,
    pub mission_list: HashMap<String, Mission>,
    pub sub_prof_dict: HashMap<String, SubProfession>,
    pub char_equip: HashMap<String, Vec<String>>,
    // equip_track_dict has complex nested structure, stored as raw JSON for now
    pub equip_track_dict: Vec<serde_json::Value>,
}

/// BattleEquip is a map of module IDs to their battle data
pub type BattleEquip = HashMap<String, ModuleData>;

// ============================================================================
// Table File Wrappers (for loading from FlatBuffer JSON)
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UniequipTableFile {
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub equip_dict: HashMap<String, RawModule>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub mission_list: HashMap<String, Mission>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub sub_prof_dict: HashMap<String, SubProfession>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub char_equip: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub equip_track_dict: Vec<serde_json::Value>, // Complex nested structure
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BattleEquipTableFile {
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub equips: HashMap<String, ModuleData>,
}
