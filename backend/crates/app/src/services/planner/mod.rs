pub mod max_level;

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use myrtle_db::repos::items::ItemRow;
use myrtle_db::repos::planner::{self as repo, PlanGroupRow, PlanMembershipRow, PlanRow};
use myrtle_db::repos::{items, progress, roster, users};
use myrtle_engine_base::types::UserBuilding;
use myrtle_engine_planner::{
    self as engine, EXP_ITEM, LMD_ITEM, MAX_FLATTEN_TIER, MIN_FLATTEN_TIER, OperatorPlan,
    PlanError, PlanInput, PlanRequirementItem, PlannerCtx, PresetTarget,
};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::operator::Operator;
use myrtle_types::roster::RosterEntry;
use myrtle_types::{Server, UserId};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::policy::Subject;
use crate::privacy::{self, layout::ProfileLayout, layout::ProfileTabId};
use crate::snapshots::Snapshot;
use crate::{AppContext, AppError};

const MAX_BULK_DELETE: usize = 1000;

const PLAN_OPERATOR_EMPTIED_ARRAYS: [&str; 6] = [
    "talents",
    "potentialRanks",
    "favorKeyFrames",
    "baseSkills",
    "audio",
    "drones",
];
const PLAN_OPERATOR_NULLED: [&str; 2] = ["handbook", "profile"];

#[derive(Debug, Clone, Serialize)]
pub struct OperatorPlanResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub operator_id: String,
    pub target_elite: i16,
    pub target_level: i16,
    pub target_skill_level: i16,
    pub target_skills: Value,
    pub target_modules: Value,
    pub display_on_profile: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub groups: Vec<String>,
    pub operator: Value,
    pub met: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanGroup {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub pinned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PlanGroupRow> for PlanGroup {
    fn from(row: PlanGroupRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            name: row.name,
            pinned: row.pinned,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerResponse {
    pub plans: Vec<OperatorPlanResponse>,
    pub aggregated_requirements: Vec<PlanRequirementItem>,
    pub groups: Vec<PlanGroup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_requirements: Option<BTreeMap<String, Vec<PlanRequirementItem>>>,
    pub last_synced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletePlansResponse {
    pub deleted: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanPreset {
    pub id: Uuid,
    pub name: String,
    pub target: PresetTarget,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpsertPlanRequest {
    pub target_elite: i16,
    pub target_level: i16,
    pub target_skill_level: i16,
    pub target_skills: Value,
    pub target_modules: Value,
    pub display_on_profile: bool,
    pub groups: Option<Vec<String>>,
}

fn plan_error(e: PlanError) -> AppError {
    match e {
        PlanError::Invalid(m) => AppError::BadRequest(m),
        PlanError::Encode(e) => AppError::internal(e),
    }
}

fn caller(subject: &Subject) -> Result<UserId, AppError> {
    subject.user_id().ok_or(AppError::Unauthorized)
}

fn loaded_snapshots(ctx: &AppContext) -> Vec<(Server, Arc<Snapshot>)> {
    ctx.snapshots
        .loaded()
        .into_iter()
        .filter_map(|s| Some((s, ctx.snapshots.get(s)?)))
        .collect()
}

fn find_operator<'a>(
    loaded: &'a [(Server, Arc<Snapshot>)],
    id: &str,
) -> Option<(&'a Operator, Server)> {
    loaded
        .iter()
        .find_map(|(server, snap)| snap.model.operators.get(id).map(|op| (op, *server)))
}

fn engine_plan(row: &PlanRow) -> OperatorPlan {
    OperatorPlan {
        operator_id: row.operator_id.clone(),
        target_elite: row.target_elite,
        target_level: row.target_level,
        target_skill_level: row.target_skill_level,
        target_skills: row.target_skills.clone(),
        target_modules: row.target_modules.clone(),
    }
}

fn roster_by_operator(roster: Vec<RosterEntry>) -> HashMap<String, RosterEntry> {
    roster
        .into_iter()
        .map(|r| (r.operator_id.clone(), r))
        .collect()
}

fn group_names_by_plan(memberships: Vec<PlanMembershipRow>) -> HashMap<Uuid, Vec<String>> {
    let mut by_plan: HashMap<Uuid, Vec<String>> = HashMap::new();
    for m in memberships {
        by_plan.entry(m.operator_plan_id).or_default().push(m.name);
    }
    by_plan
}

fn inventory_map(model: &GameModel, inventory: Vec<ItemRow>, lmd: i32) -> HashMap<String, i32> {
    let mut map: HashMap<String, i32> = inventory
        .into_iter()
        .map(|i| (i.item_id, i.quantity))
        .collect();
    map.insert(LMD_ITEM.to_owned(), lmd);
    let total_exp = model.materials.exp_items.values().fold(0i32, |sum, item| {
        let qty = map.get(&item.id).copied().unwrap_or(0);
        sum.saturating_add(qty.saturating_mul(item.gain_exp))
    });
    map.insert(EXP_ITEM.to_owned(), total_exp);
    map
}

pub fn slim_plan_operator(map: &mut Map<String, Value>) {
    for key in PLAN_OPERATOR_EMPTIED_ARRAYS {
        if map.contains_key(key) {
            map.insert(key.to_owned(), Value::Array(Vec::new()));
        }
    }
    for key in PLAN_OPERATOR_NULLED {
        if map.contains_key(key) {
            map.insert(key.to_owned(), Value::Null);
        }
    }
    if let Some(Value::Array(skills)) = map.get_mut("skills") {
        for skill in skills {
            if let Some(Value::Array(levels)) = skill.pointer_mut("/static/Levels") {
                levels.truncate(1);
            }
        }
    }
    if let Some(Value::Array(modules)) = map.get_mut("modules") {
        for module in modules.iter_mut().filter_map(Value::as_object_mut) {
            module.insert("data".to_owned(), Value::Null);
            module.insert("uniEquipDesc".to_owned(), Value::String(String::new()));
        }
    }
}

fn plan_response(
    plan: PlanRow,
    groups: Vec<String>,
    operator: &Operator,
    server: Server,
    met: bool,
) -> Result<OperatorPlanResponse, AppError> {
    let mut operator_json = serde_json::to_value(operator).map_err(AppError::internal)?;
    if let Value::Object(map) = &mut operator_json {
        slim_plan_operator(map);
        map.insert(
            "server".to_owned(),
            Value::String(server.as_str().to_owned()),
        );
    }
    Ok(OperatorPlanResponse {
        id: plan.id,
        user_id: plan.user_id,
        operator_id: plan.operator_id,
        target_elite: plan.target_elite,
        target_level: plan.target_level,
        target_skill_level: plan.target_skill_level,
        target_skills: plan.target_skills,
        target_modules: plan.target_modules,
        display_on_profile: plan.display_on_profile,
        created_at: plan.created_at,
        updated_at: plan.updated_at,
        groups,
        operator: operator_json,
        met,
    })
}

pub async fn list_plans(
    ctx: &AppContext,
    subject: &Subject,
    active: Option<&str>,
    max_tier: Option<i16>,
    by_operator: bool,
) -> Result<PlannerResponse, AppError> {
    let user = caller(subject)?;
    if let Some(tier) = max_tier
        && !(MIN_FLATTEN_TIER..=MAX_FLATTEN_TIER).contains(&tier)
    {
        return Err(AppError::BadRequest(format!(
            "max_tier must be between {MIN_FLATTEN_TIER} and {MAX_FLATTEN_TIER}"
        )));
    }
    let active_ids: Vec<String> = active
        .map(|s| {
            s.split(',')
                .map(|id| id.trim().to_owned())
                .filter(|id| !id.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let mut conn = ctx.db.conn().await?;
    let plans = repo::list_plans(&mut conn, user).await?;
    let roster = roster_by_operator(roster::roster(&mut conn, user).await?);
    let inventory = items::inventory(&mut conn, user).await?;
    let profile = users::profile(&mut conn, user).await?;
    let building = progress::building(&mut conn, user).await?;
    let clears = progress::stage_clears(&mut conn, user).await?.clears;
    let groups: Vec<PlanGroup> = repo::list_groups(&mut conn, user)
        .await?
        .into_iter()
        .map(PlanGroup::from)
        .collect();
    let memberships = repo::list_memberships(&mut conn, user).await?;
    drop(conn);

    let default = ctx.snapshots.or_default(ctx.snapshots.default_server())?;
    let loaded = loaded_snapshots(ctx);
    let lmd = profile.as_ref().and_then(|p| p.lmd).unwrap_or(0);
    let last_synced_at = profile.map(|p| p.updated_at);

    ctx.cpu
        .run("planner_requirements", move || {
            let model = &default.model;
            let inventory_map = inventory_map(model, inventory, lmd);
            let user_building = building.as_ref().map_or_else(
                || UserBuilding {
                    rooms: Vec::new(),
                    drone_capacity: None,
                },
                UserBuilding::from_json,
            );
            let mut group_map = group_names_by_plan(memberships);
            let mut responses = Vec::with_capacity(plans.len());
            let mut plans_with_ops = Vec::with_capacity(plans.len());
            for plan in plans {
                let Some((operator, server)) = find_operator(&loaded, &plan.operator_id) else {
                    continue;
                };
                let entry = roster.get(&plan.operator_id);
                let stored = engine_plan(&plan);
                let met = engine::stored_plan_met(&stored, entry);
                let plan_groups = group_map.remove(&plan.id).unwrap_or_default();
                responses.push(plan_response(plan, plan_groups, operator, server, met)?);
                plans_with_ops.push((stored, operator.clone(), entry));
            }
            let planner = PlannerCtx {
                gamedata: model,
                inventory_map: &inventory_map,
                user_building: &user_building,
                clears: &clears,
            };
            let aggregated_requirements =
                engine::calculate_requirements(&planner, &plans_with_ops, &active_ids, max_tier)
                    .map_err(plan_error)?;
            let operator_requirements = by_operator
                .then(|| {
                    engine::requirements_by_operator(
                        &planner,
                        &plans_with_ops,
                        &active_ids,
                        max_tier,
                    )
                })
                .transpose()
                .map_err(plan_error)?;
            Ok(PlannerResponse {
                plans: responses,
                aggregated_requirements,
                operator_requirements,
                groups,
                last_synced_at,
            })
        })
        .await?
}

async fn plans_owner(ctx: &AppContext, subject: &Subject, uid: &str) -> Result<UserId, AppError> {
    match privacy::resolve_user_id(ctx, subject, Some(uid), &[ProfileTabId::Plans]).await {
        Err(AppError::NotFound(None)) => {
            let id = Uuid::parse_str(uid).map_err(|_| AppError::NotFound(None))?;
            let mut conn = ctx.db.conn().await?;
            let profile = users::profile(&mut conn, UserId(id))
                .await?
                .ok_or(AppError::NotFound(None))?;
            drop(conn);
            let (profile, _) = privacy::ensure_profile_readable(ctx, subject, profile)?;
            let layout = profile
                .profile_layout
                .as_ref()
                .map(ProfileLayout::normalize);
            privacy::ensure_tab_visible(
                ctx,
                subject,
                &profile,
                layout.as_ref(),
                ProfileTabId::Plans,
            )?;
            Ok(UserId(profile.id))
        }
        other => other,
    }
}

pub async fn list_public_plans(
    ctx: &AppContext,
    subject: &Subject,
    uid: &str,
) -> Result<Vec<OperatorPlanResponse>, AppError> {
    let owner = plans_owner(ctx, subject, uid).await?;
    let mut conn = ctx.db.conn().await?;
    let plans = repo::list_plans(&mut conn, owner).await?;
    let mut group_map = group_names_by_plan(repo::list_memberships(&mut conn, owner).await?);
    let roster = roster_by_operator(roster::roster(&mut conn, owner).await?);
    drop(conn);

    let loaded = loaded_snapshots(ctx);
    let mut responses = Vec::new();
    for plan in plans.into_iter().filter(|p| p.display_on_profile) {
        let Some((operator, server)) = find_operator(&loaded, &plan.operator_id) else {
            continue;
        };
        let met = engine::stored_plan_met(&engine_plan(&plan), roster.get(&plan.operator_id));
        let plan_groups = group_map.remove(&plan.id).unwrap_or_default();
        responses.push(plan_response(plan, plan_groups, operator, server, met)?);
    }
    Ok(responses)
}

pub async fn upsert_plan(
    ctx: &AppContext,
    subject: &Subject,
    operator_id: &str,
    request: UpsertPlanRequest,
) -> Result<OperatorPlanResponse, AppError> {
    let user = caller(subject)?;
    engine::ensure_plannable_id(operator_id).map_err(plan_error)?;
    let loaded = loaded_snapshots(ctx);
    let (operator, server) = find_operator(&loaded, operator_id).ok_or(AppError::NotFound(None))?;

    let mut input = PlanInput {
        target_elite: request.target_elite,
        target_level: request.target_level,
        target_skill_level: request.target_skill_level,
        target_skills: request.target_skills,
        target_modules: request.target_modules,
        display_on_profile: request.display_on_profile,
    };
    let mut conn = ctx.db.conn().await?;
    let entry = roster::operator(&mut conn, user, operator_id).await?;
    engine::raise_to_roster(&mut input, entry.as_ref()).map_err(plan_error)?;
    engine::validate_plan(operator, &input).map_err(plan_error)?;

    let stored = repo::PlanInput {
        target_elite: input.target_elite,
        target_level: input.target_level,
        target_skill_level: input.target_skill_level,
        target_skills: &input.target_skills,
        target_modules: &input.target_modules,
        display_on_profile: input.display_on_profile,
    };
    let plan = repo::upsert_plan(
        &mut conn,
        user,
        operator_id,
        stored,
        request.groups.as_deref(),
    )
    .await?;
    let met = engine::stored_plan_met(&engine_plan(&plan), entry.as_ref());
    let groups = repo::group_names(&mut conn, plan.id).await?;
    drop(conn);
    plan_response(plan, groups, operator, server, met)
}

pub async fn delete_plan(
    ctx: &AppContext,
    subject: &Subject,
    operator_id: &str,
) -> Result<(), AppError> {
    let user = caller(subject)?;
    let mut conn = ctx.db.conn().await?;
    Ok(repo::delete_plan(&mut conn, user, operator_id).await?)
}

pub async fn delete_plans(
    ctx: &AppContext,
    subject: &Subject,
    operator_ids: &[String],
) -> Result<DeletePlansResponse, AppError> {
    let user = caller(subject)?;
    if operator_ids.len() > MAX_BULK_DELETE {
        return Err(AppError::BadRequest(format!(
            "At most {MAX_BULK_DELETE} plans can be deleted at once"
        )));
    }
    if operator_ids.is_empty() {
        return Ok(DeletePlansResponse { deleted: 0 });
    }
    let mut conn = ctx.db.conn().await?;
    let deleted = repo::delete_plans(&mut conn, user, operator_ids).await?;
    Ok(DeletePlansResponse {
        deleted: i64::try_from(deleted).unwrap_or(i64::MAX),
    })
}

pub async fn create_group(
    ctx: &AppContext,
    subject: &Subject,
    name: &str,
) -> Result<PlanGroup, AppError> {
    let user = caller(subject)?;
    let mut conn = ctx.db.conn().await?;
    Ok(repo::create_group(&mut conn, user, name).await?.into())
}

pub async fn update_group(
    ctx: &AppContext,
    subject: &Subject,
    old_name: &str,
    name: Option<&str>,
    pinned: Option<bool>,
) -> Result<PlanGroup, AppError> {
    let user = caller(subject)?;
    if name.is_none() && pinned.is_none() {
        return Err(AppError::bad_request(
            "Nothing to update: give `name`, `pinned`, or both",
        ));
    }
    let mut conn = ctx.db.conn().await?;
    Ok(repo::update_group(&mut conn, user, old_name, name, pinned)
        .await?
        .into())
}

pub async fn delete_group(ctx: &AppContext, subject: &Subject, name: &str) -> Result<(), AppError> {
    let user = caller(subject)?;
    let mut conn = ctx.db.conn().await?;
    Ok(repo::delete_group(&mut conn, user, name).await?)
}

pub async fn list_presets(
    ctx: &AppContext,
    subject: &Subject,
) -> Result<Vec<PlanPreset>, AppError> {
    let user = caller(subject)?;
    let mut conn = ctx.db.conn().await?;
    repo::list_presets(&mut conn, user)
        .await?
        .into_iter()
        .map(preset_from_row)
        .collect()
}

fn preset_from_row(row: repo::PlanPresetRow) -> Result<PlanPreset, AppError> {
    Ok(PlanPreset {
        id: row.id,
        name: row.name,
        target: serde_json::from_value(row.target).map_err(AppError::internal)?,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

pub async fn upsert_preset(
    ctx: &AppContext,
    subject: &Subject,
    name: &str,
    target: &PresetTarget,
) -> Result<PlanPreset, AppError> {
    let user = caller(subject)?;
    let name = name.trim();
    engine::validate_preset(name, target).map_err(plan_error)?;
    let target = serde_json::to_value(target).map_err(AppError::internal)?;
    let mut conn = ctx.db.conn().await?;
    preset_from_row(repo::upsert_preset(&mut conn, user, name, &target).await?)
}

pub async fn delete_preset(
    ctx: &AppContext,
    subject: &Subject,
    name: &str,
) -> Result<(), AppError> {
    let user = caller(subject)?;
    let mut conn = ctx.db.conn().await?;
    Ok(repo::delete_preset(&mut conn, user, name).await?)
}
