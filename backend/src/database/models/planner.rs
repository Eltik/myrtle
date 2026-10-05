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
    /// Stage clears the item's workshop recipe waits on that the player has
    /// not made, in recipe order. The same gates `craft_reason` names as
    /// `Stage <code>`, structured so a client can link each stage. Empty when
    /// the item has no recipe or its stage gates are all met.
    pub unmet_stages: Vec<PlanUnmetStage>,
}

/// One stage clear a recipe is still waiting on.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanUnmetStage {
    pub stage_id: String,
    /// Display code (`S9-3`), or the stage id when the stage table lacks it.
    pub code: String,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PlanGroup {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    /// Pinned groups list first. See `list_groups` for the order.
    pub pinned: bool,
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
    /// The plan has nothing left to do: the operator is owned and every
    /// target (promotion and level, skill level, each mastery, each module
    /// stage) is reached or passed. Read from the same roster state the
    /// material diff uses, so a met plan contributes no materials.
    pub met: bool,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerResponse {
    pub plans: Vec<OperatorPlanResponse>,
    pub aggregated_requirements: Vec<PlanRequirementItem>,
    pub groups: Vec<PlanGroup>,
    /// When the caller's account was last synced to our database. Every
    /// current-state reading behind the plan (roster, inventory, base, stage
    /// clears) is a snapshot as of this moment. Null for an account with no
    /// profile row.
    pub last_synced_at: Option<DateTime<Utc>>,
}

/// Result of a bulk plan delete.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize)]
pub struct DeletePlansResponse {
    /// Plans removed. Ids the caller had no plan for are not counted.
    #[ts(type = "number")]
    pub deleted: i64,
}

/// A preset's target, independent of rarity: a bulk-add applies it to each
/// selected operator and clamps it to what that operator can reach.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresetTarget {
    /// Promotion, 0 to 2.
    pub elite: i16,
    /// Level at that promotion. Null means the operator's cap at `elite`.
    pub level: Option<i16>,
    /// Skill level, 1 to 7.
    pub skill_level: i16,
    /// Mastery per skill index, each 0 to 3.
    pub masteries: [i16; 3],
    /// Stage for every module, 0 to 3.
    pub module_stage: i16,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize)]
pub struct PlanPreset {
    pub id: Uuid,
    pub name: String,
    pub target: PresetTarget,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
