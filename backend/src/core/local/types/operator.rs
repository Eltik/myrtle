use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::handbook::{HandbookItem, OperatorProfile};
use super::material::{Item, ItemType};
use super::module::{Module, ModuleData};
use super::serde_helpers::{deserialize_fb_map, deserialize_fb_map_option};
use super::skill::SkillLevel;

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OperatorPosition {
    Ranged,
    Melee,
    All,
    None,
}

impl Default for OperatorPosition {
    fn default() -> Self {
        Self::Melee
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatorPhase {
    #[serde(rename = "PHASE_0")]
    Elite0,
    #[serde(rename = "PHASE_1")]
    Elite1,
    #[serde(rename = "PHASE_2")]
    Elite2,
}

impl Default for OperatorPhase {
    fn default() -> Self {
        Self::Elite0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatorRarity {
    #[serde(rename = "TIER_6")]
    SixStar,
    #[serde(rename = "TIER_5")]
    FiveStar,
    #[serde(rename = "TIER_4")]
    FourStar,
    #[serde(rename = "TIER_3")]
    ThreeStar,
    #[serde(rename = "TIER_2")]
    TwoStar,
    #[serde(rename = "TIER_1")]
    OneStar,
}

impl Default for OperatorRarity {
    fn default() -> Self {
        Self::OneStar
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatorProfession {
    #[serde(rename = "MEDIC")]
    Medic,
    #[serde(rename = "CASTER")]
    Caster,
    #[serde(rename = "WARRIOR")]
    Guard,
    #[serde(rename = "PIONEER")]
    Vanguard,
    #[serde(rename = "SNIPER")]
    Sniper,
    #[serde(rename = "SPECIAL")]
    Specialist,
    #[serde(rename = "SUPPORT")]
    Supporter,
    #[serde(rename = "TANK")]
    Defender,
    #[serde(rename = "TOKEN")]
    Token,
    #[serde(rename = "TRAP")]
    Trap,
}

impl Default for OperatorProfession {
    fn default() -> Self {
        Self::Guard
    }
}

// ============================================================================
// Nested Structs
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UnlockCondition {
    pub phase: OperatorPhase,
    pub level: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Blackboard {
    #[serde(alias = "key")]
    pub key: String,
    #[serde(alias = "value")]
    pub value: f64,
    #[serde(alias = "valueStr")]
    pub value_str: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TraitCandidate {
    pub unlock_condition: UnlockCondition,
    pub required_potential_rank: i32,
    pub blackboard: Vec<Blackboard>,
    pub override_description: Option<String>,
    pub prefab_key: Option<String>,
    pub range_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Trait {
    pub candidates: Vec<TraitCandidate>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttributeData {
    pub max_hp: i32,
    pub atk: i32,
    pub def: i32,
    pub magic_resistance: f64,
    pub cost: i32,
    pub block_cnt: i32,
    pub move_speed: f64,
    pub attack_speed: f64,
    pub base_attack_time: f64,
    pub respawn_time: i32,
    pub hp_recovery_per_sec: f64,
    pub sp_recovery_per_sec: f64,
    pub max_deploy_count: i32,
    pub max_deck_stack_cnt: i32,
    pub taunt_level: i32,
    pub mass_level: i32,
    pub base_force_level: i32,
    pub stun_immune: bool,
    pub silence_immune: bool,
    pub sleep_immune: bool,
    pub frozen_immune: bool,
    pub levitate_immune: bool,
    pub disarmed_combat_immune: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttributeKeyFrame {
    pub level: i32,
    pub data: AttributeData,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EvolveCost {
    pub id: String,
    pub count: i32,
    #[serde(rename = "Type_")]
    pub item_type: ItemType,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Phase {
    pub character_prefab_key: String,
    #[serde(default)]
    pub range_id: Option<String>,
    pub max_level: i32,
    #[serde(default)]
    pub attributes_key_frames: Vec<AttributeKeyFrame>,
    #[serde(default)]
    pub evolve_cost: Option<Vec<EvolveCost>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LevelUpCostItem {
    pub id: String,
    pub count: i32,
    #[serde(rename = "Type_")]
    pub item_type: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LevelUpCostCond {
    pub unlock_cond: UnlockCondition,
    pub lvl_up_time: i32,
    #[serde(default)]
    pub level_up_cost: Vec<LevelUpCostItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SpData {
    pub sp_type: String,
    pub level_up_cost: Vec<()>, // Empty array in source
    pub max_charge_time: i32,
    pub sp_cost: i32,
    pub init_sp: i32,
    pub increment: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SkillStatic {
    pub levels: Vec<SkillLevel>,
    pub skill_id: String,
    pub icon_id: Option<String>,
    pub hidden: bool,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OperatorSkillRef {
    #[serde(default)]
    pub skill_id: Option<String>,
    #[serde(default)]
    pub override_prefab_key: Option<String>,
    #[serde(default)]
    pub override_token_key: Option<String>,
    #[serde(default)]
    pub level_up_cost_cond: Vec<LevelUpCostCond>,
    #[serde(default)]
    pub unlock_cond: Option<UnlockCondition>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TalentCandidate {
    pub unlock_condition: UnlockCondition,
    #[serde(default)]
    pub required_potential_rank: i32,
    #[serde(default)]
    pub prefab_key: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub range_id: Option<String>,
    #[serde(default)]
    pub blackboard: Vec<Blackboard>,
    #[serde(default)]
    pub token_key: Option<String>,
    #[serde(default)]
    pub is_hide_talent: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Talent {
    #[serde(default)]
    pub candidates: Vec<TalentCandidate>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttributeModifier {
    pub attribute_type: String,
    pub formula_item: String,
    #[serde(default)]
    pub value: f64,
    pub load_from_blackboard: bool,
    pub fetch_base_value_from_source_entity: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PotentialBuffAttributes {
    pub abnormal_flags: Option<()>,
    pub abnormal_immunes: Option<()>,
    pub abnormal_antis: Option<()>,
    pub abnormal_combos: Option<()>,
    pub abnormal_combo_immunes: Option<()>,
    pub attribute_modifiers: Option<Vec<AttributeModifier>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PotentialBuff {
    pub attributes: PotentialBuffAttributes,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PotentialRank {
    #[serde(rename = "Type_")]
    pub potential_type: String,
    pub description: String,
    #[serde(default)]
    pub buff: Option<PotentialBuff>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AllSkillLevelUp {
    pub unlock_cond: UnlockCondition,
    #[serde(default)]
    pub lvl_up_cost: Vec<LevelUpCostItem>,
}

/// Wrapper for character_table.json
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CharacterTable {
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub characters: HashMap<String, RawOperator>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawOperator {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub can_use_general_potential_item: bool,
    #[serde(default)]
    pub can_use_activity_potential_item: bool,
    #[serde(default)]
    pub potential_item_id: Option<String>,
    #[serde(default)]
    pub activity_potential_item_id: Option<String>,
    #[serde(default)]
    pub classic_potential_item_id: Option<String>,
    #[serde(default)]
    pub nation_id: Option<String>,
    #[serde(default)]
    pub group_id: Option<String>,
    #[serde(default)]
    pub team_id: Option<String>,
    #[serde(default)]
    pub display_number: Option<String>,
    pub appellation: String,
    #[serde(default)]
    pub position: OperatorPosition,
    #[serde(default)]
    pub tag_list: Option<Vec<String>>,
    #[serde(default)]
    pub item_usage: Option<String>,
    #[serde(default)]
    pub item_desc: Option<String>,
    #[serde(default)]
    pub item_obtain_approach: Option<String>,
    #[serde(default)]
    pub is_not_obtainable: bool,
    #[serde(default)]
    pub is_sp_char: bool,
    #[serde(default)]
    pub max_potential_level: i32,
    #[serde(default)]
    pub rarity: OperatorRarity,
    #[serde(default)]
    pub profession: OperatorProfession,
    #[serde(default)]
    pub sub_profession_id: Option<String>,
    #[serde(default, rename = "Trait_")]
    pub trait_data: Option<Trait>,
    #[serde(default)]
    pub phases: Vec<Phase>,
    #[serde(default)]
    pub skills: Vec<OperatorSkillRef>,
    #[serde(default, deserialize_with = "deserialize_fb_map_option")]
    pub display_token_dict: Option<HashMap<String, bool>>,
    #[serde(default)]
    pub talents: Option<Vec<Talent>>,
    #[serde(default)]
    pub potential_ranks: Vec<PotentialRank>,
    #[serde(default)]
    pub favor_key_frames: Option<Vec<AttributeKeyFrame>>,
    #[serde(default)]
    pub all_skill_lvlup: Vec<AllSkillLevelUp>,
    // Extra fields in unpacked data
    #[serde(default)]
    pub sort_index: Option<i32>,
    #[serde(default)]
    pub sp_target_type: Option<String>,
}

// ============================================================================
// Operator
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorModule {
    #[serde(flatten)]
    pub module: Module,
    pub data: ModuleData,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichedSkill {
    pub skill_id: String,
    pub override_prefab_key: Option<String>,
    pub override_token_key: Option<String>,
    pub level_up_cost_cond: Vec<LevelUpCostCond>,
    #[serde(rename = "static")]
    pub static_data: Option<SkillStatic>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operator {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub can_use_general_potential_item: bool,
    pub can_use_activity_potential_item: bool,
    pub potential_item_id: String,
    pub activity_potential_item_id: Option<String>,
    pub classic_potential_item_id: Option<String>,
    pub nation_id: String,
    pub group_id: Option<String>,
    pub team_id: Option<String>,
    pub display_number: String,
    pub appellation: String,
    pub position: OperatorPosition,
    pub tag_list: Vec<String>,
    pub item_usage: String,
    pub item_desc: String,
    pub item_obtain_approach: String,
    pub is_not_obtainable: bool,
    pub is_sp_char: bool,
    pub max_potential_level: i32,
    pub rarity: OperatorRarity,
    pub profession: OperatorProfession,
    pub sub_profession_id: String,
    #[serde(rename = "trait")]
    pub trait_data: Option<Trait>,
    pub phases: Vec<Phase>,
    pub skills: Vec<EnrichedSkill>,
    pub display_token_dict: Option<HashMap<String, bool>>,
    pub talents: Vec<Talent>,
    pub potential_ranks: Vec<PotentialRank>,
    pub favor_key_frames: Vec<AttributeKeyFrame>,
    pub all_skill_level_up: Vec<AllSkillLevelUp>,
    pub modules: Vec<OperatorModule>,
    pub handbook: HandbookItem,
    pub profile: Option<OperatorProfile>,
    pub artists: Vec<String>,
    pub portrait: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Elite {
    E0,
    E1,
    E2,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EliteCostItem {
    pub quantity: i32,
    pub material: Item,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelCost {
    pub level: i32,
    pub elite_cost: Vec<EliteCostItem>,
    pub elite: Elite,
}

pub type LevelUpCost = Vec<LevelCost>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MasteryLevel {
    M1,
    M2,
    M3,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillCostItem {
    pub quantity: i32,
    pub material: Item,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillCost {
    pub unlock_condition: UnlockCondition,
    pub lvl_up_time: i32,
    pub skill_cost: Vec<SkillCostItem>,
    pub level: MasteryLevel,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillLevelCost {
    pub skill_id: String,
    pub cost: Vec<SkillCost>,
}

pub type SkillLevelUpCost = Vec<SkillLevelCost>;

// ============================================================================
// Drone Type
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Drone {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub can_use_general_potential_item: bool,
    pub can_use_activity_potential_item: bool,
    pub potential_item_id: Option<String>,
    pub activity_potential_item_id: Option<String>,
    pub classic_potential_item_id: Option<String>,
    pub nation_id: Option<String>,
    pub group_id: Option<String>,
    pub team_id: Option<String>,
    pub display_number: Option<String>,
    pub appellation: String,
    pub position: OperatorPosition,
    pub tag_list: Vec<String>,
    pub item_usage: Option<String>,
    pub item_desc: Option<String>,
    pub item_obtain_approach: Option<String>,
    pub is_not_obtainable: bool,
    pub is_sp_char: bool,
    pub max_potential_level: i32,
    pub rarity: OperatorRarity,
    pub profession: String, // Always "TOKEN" for drones
    pub sub_profession_id: String,
    #[serde(rename = "trait")]
    pub trait_data: Option<Trait>,
    pub phases: Vec<Phase>,
    pub skills: Vec<OperatorSkillRef>,
    pub display_token_dict: Option<HashMap<String, bool>>,
    pub talents: Vec<Talent>,
    pub potential_ranks: Vec<PotentialRank>,
    pub favor_key_frames: Option<()>, // Always null for drones
    pub all_skill_lvlup: Vec<AllSkillLevelUp>,
    pub modules: Vec<OperatorModule>,
}
