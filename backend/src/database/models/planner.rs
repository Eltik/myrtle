use serde::{Deserialize, Serialize};
use sqlx::types::{
    Uuid,
    chrono::{DateTime, Utc},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetSkillPlan {
    pub skill_index: i16,
    pub mastery_level: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetModulePlan {
    pub module_id: String,
    pub module_stage: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OperatorPlan {
    pub id: Uuid,
    pub user_id: Uuid,
    pub operator_id: String,
    pub target_elite: i16,
    pub target_level: i16,
    pub target_skill_level: i16,
    pub target_skills: serde_json::Value,
    pub target_modules: serde_json::Value,
    pub display_on_profile: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanRequirementItem {
    pub id: String,
    pub name: String,
    pub icon_id: Option<String>,
    pub image: Option<String>,
    pub item_type: String,
    pub rarity: i16,
    pub sort_group: i8,
    pub sort_subrank: i8,
    pub required_count: i32,
    pub inventory_count: i32,
    pub craftable_count: i32,
    pub missing_count: i32,
    pub can_craft: bool,
    pub craft_reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperatorPlanResponse {
    #[serde(flatten)]
    pub plan: OperatorPlan,
    pub requirements: Vec<PlanRequirementItem>,
}
