use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::serde_helpers::deserialize_fb_map;

// ============================================================================
// Nested Structs
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillBlackboard {
    #[serde(alias = "Key")]
    pub key: String,
    #[serde(alias = "Value")]
    pub value: f64,
    #[serde(alias = "ValueStr")]
    pub value_str: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSpData {
    #[serde(alias = "SpType")]
    pub sp_type: String,
    #[serde(alias = "LevelUpCost")]
    pub level_up_cost: Option<()>, // Always null in source
    #[serde(alias = "MaxChargeTime")]
    pub max_charge_time: i32,
    #[serde(alias = "SpCost")]
    pub sp_cost: i32,
    #[serde(alias = "InitSp")]
    pub init_sp: i32,
    #[serde(alias = "Increment")]
    pub increment: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillLevel {
    #[serde(alias = "Name")]
    pub name: String,
    #[serde(alias = "RangeId")]
    pub range_id: Option<String>,
    #[serde(alias = "Description")]
    pub description: String,
    #[serde(alias = "SkillType")]
    pub skill_type: String,
    #[serde(alias = "DurationType")]
    pub duration_type: Option<String>,
    #[serde(alias = "SpData")]
    pub sp_data: SkillSpData,
    #[serde(alias = "PrefabId")]
    pub prefab_id: String,
    #[serde(alias = "Duration")]
    pub duration: f64,
    #[serde(alias = "Blackboard")]
    pub blackboard: Vec<SkillBlackboard>,
}

// ============================================================================
// RawSkill
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawSkill {
    #[serde(alias = "SkillId")]
    pub skill_id: String,
    #[serde(alias = "IconId")]
    pub icon_id: Option<String>,
    #[serde(alias = "Hidden")]
    pub hidden: bool,
    #[serde(alias = "Levels")]
    pub levels: Vec<SkillLevel>,
}

// ============================================================================
// Skill
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: Option<String>,
    pub skill_id: String,
    pub icon_id: Option<String>,
    pub image: Option<String>,
    pub hidden: bool,
    pub levels: Vec<SkillLevel>,
}

// ============================================================================
// Table File Wrapper (for loading from FlatBuffer JSON)
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SkillTableFile {
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub skills: HashMap<String, RawSkill>,
}
