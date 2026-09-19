use serde::{Deserialize, Serialize};
use sqlx::types::{
    Uuid,
    chrono::{DateTime, Utc},
};
use ts_rs::TS;

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetSkillPlan {
    pub skill_index: i16,
    pub mastery_level: i16,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetModulePlan {
    pub module_id: String,
    pub module_stage: i16,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
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

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanRecipeCost {
    pub count: i32,
    pub item: PlanRequirementItem,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanRecipe {
    pub count: i32,
    pub costs: Vec<PlanRecipeCost>,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
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
    /// The crafting recipe, when this item has one.
    ///
    /// This closes a genuine cycle in the data: a recipe's costs are themselves
    /// `PlanRequirementItem`s, which may in turn be craftable. ts-rs emits
    /// mutually referencing types and handles it, but utoipa inlines nested
    /// schemas while collecting them and would recurse until the stack runs
    /// out, so the cycle is cut here and the schema emits a `$ref` instead.
    /// Removing this attribute makes the whole `OpenAPI` document unbuildable,
    /// which takes the server down at startup rather than failing a test.
    #[schema(no_recursion)]
    pub recipe: Option<PlanRecipe>,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PlanGroup {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize)]
pub struct OperatorPlanResponse {
    #[serde(flatten)]
    pub plan: OperatorPlan,
    pub groups: Vec<String>,
    pub operator: serde_json::Value,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerResponse {
    pub plans: Vec<OperatorPlanResponse>,
    pub aggregated_requirements: Vec<PlanRequirementItem>,
    pub groups: Vec<PlanGroup>,
}
