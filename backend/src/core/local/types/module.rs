use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::material::ItemType;

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
    TalentDataOnly,
    Talent,
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
    pub id: String,
    pub count: i32,
    #[serde(rename = "type")]
    pub item_type: ItemType,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubProfession {
    pub sub_profession_id: String,
    pub sub_profession_name: String,
    pub sub_profession_category: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipTrackItem {
    pub char_id: String,
    pub equip_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipTrack {
    pub timestamp: i64,
    pub track_list: Vec<EquipTrackItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mission {
    pub template: String,
    pub desc: String,
    pub param_list: Vec<String>,
    pub uni_equip_mission_id: String,
    pub uni_equip_mission_sort: i32,
    pub uni_equip_id: String,
    pub jump_stage_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleBlackboard {
    pub key: String,
    pub value: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleUnlockCondition {
    pub phase: i32,
    pub level: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddModuleCandidates {
    pub display_range_id: bool,
    pub upgrade_description: String,
    pub talent_index: i32,
    pub unlock_condition: ModuleUnlockCondition,
    pub required_potential_rank: i32,
    pub prefab_key: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub range_id: Option<String>,
    pub blackboard: Vec<ModuleBlackboard>,
    pub token_key: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleCandidates {
    pub additional_description: String,
    pub unlock_condition: ModuleUnlockCondition,
    pub required_potential_rank: i32,
    pub blackboard: Vec<ModuleBlackboard>,
    pub override_description: Option<String>,
    pub prefab_key: Option<String>,
    pub range_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddOrOverrideTalentDataBundle {
    pub candidates: Option<Vec<AddModuleCandidates>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverrideTraitDataBundle {
    pub candidates: Option<Vec<ModuleCandidates>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModulePart {
    pub res_key: String,
    pub target: ModuleTarget,
    pub is_token: bool,
    pub add_or_override_talent_data_bundle: AddOrOverrideTalentDataBundle,
    pub override_trait_data_bundle: OverrideTraitDataBundle,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModulePhase {
    pub equip_level: i32,
    pub parts: Vec<ModulePart>,
    pub attribute_blackboard: Vec<ModuleBlackboard>,
    pub token_attribute_blackboard: HashMap<String, Vec<ModuleBlackboard>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleData {
    pub phases: Vec<ModulePhase>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawModule {
    pub uni_equip_id: String,
    pub uni_equip_name: String,
    pub uni_equip_icon: String,
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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawModules {
    pub equip_dict: HashMap<String, RawModule>,
    pub mission_list: HashMap<String, Mission>,
    pub sub_prof_dict: HashMap<String, SubProfession>,
    pub char_equip: HashMap<String, Vec<String>>,
    pub equip_track_dict: HashMap<String, EquipTrack>,
}

/// BattleEquip is a map of module IDs to their battle data
pub type BattleEquip = HashMap<String, ModuleData>;
