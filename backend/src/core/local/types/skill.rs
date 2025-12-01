use serde::{Deserialize, Serialize};

// ============================================================================
// Nested Structs
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillBlackboard {
    pub key: String,
    pub value: f64,
    pub value_str: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSpData {
    pub sp_type: String,
    pub level_up_cost: Option<()>, // Always null in source
    pub max_charge_time: i32,
    pub sp_cost: i32,
    pub init_sp: i32,
    pub increment: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillLevel {
    pub name: String,
    pub range_id: Option<String>,
    pub description: String,
    pub skill_type: String,
    pub sp_data: SkillSpData,
    pub prefab_id: String,
    pub duration: f64,
    pub blackboard: Vec<SkillBlackboard>,
}

// ============================================================================
// RawSkill
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawSkill {
    pub skill_id: String,
    pub icon_id: Option<String>,
    pub hidden: bool,
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
