//! The operator planner: a player's upgrade targets per operator, and what
//! they still cost against the synced account.
//!
//! - [`progress`]: where an operator stands against its plan, and the
//!   materials between the two.
//! - [`requirements`]: those materials, summed over the plans, priced against
//!   inventory and crafting.
//! - [`validation`]: what a plan or a preset may ask for.

mod progress;
mod requirements;
mod validation;

use std::collections::HashMap;
use uuid::Uuid;

pub(crate) use progress::calculate_leveling_costs;
pub use requirements::{MAX_FLATTEN_TIER, MIN_FLATTEN_TIER};
pub(crate) use validation::module_phase_to_int;

use self::{
    progress::{raise_to_roster, stored_plan_met},
    requirements::{PlannerCtx, calculate_requirements, requirements_by_operator},
};
use crate::{
    app::{error::ApiError, services::operators::resolve_operator, state::AppState},
    core::{
        gamedata::types::operator::Operator, grade::base::types::UserBuilding,
        hypergryph::constants::Server,
    },
    database::{
        models::{
            planner::{
                DeletePlansResponse, OperatorPlan, OperatorPlanResponse, PlanGroup, PlanInput,
                PlanPreset, PlannerResponse, PresetTarget,
            },
            roster::RosterEntry,
        },
        queries::{
            building as building_queries, items as items_queries, planner as queries,
            roster as roster_queries, stages as stages_queries, users as users_queries,
        },
    },
};

/// LMD's item id, and its key in the planner's material map.
pub(crate) const LMD_ITEM: &str = "4001";
/// Operator EXP's key in the planner's material map. The id is the game's
/// player EXP item, borrowed for the EXP the battle records grant.
pub(crate) const EXP_ITEM: &str = "5001";

/// Most operator ids one bulk delete takes. Above the whole operator roster,
/// so a "delete every plan" request always fits.
const MAX_BULK_DELETE: usize = 1000;

async fn roster_by_operator(
    state: &AppState,
    user_id: Uuid,
) -> Result<HashMap<String, RosterEntry>, ApiError> {
    Ok(roster_queries::get_roster(&state.db, user_id)
        .await?
        .into_iter()
        .map(|r| (r.operator_id.clone(), r))
        .collect())
}

/// Each of the caller's plans' group names, by plan id.
async fn group_names_by_plan(
    state: &AppState,
    user_id: Uuid,
) -> Result<HashMap<Uuid, Vec<String>>, ApiError> {
    let mut by_plan: HashMap<Uuid, Vec<String>> = HashMap::new();
    for (plan_id, group_name) in queries::get_all_plan_groups(&state.db, user_id).await? {
        by_plan.entry(plan_id).or_default().push(group_name);
    }
    Ok(by_plan)
}

/// Operator fields no plan view reads, emptied from every plan's operator.
/// The plan card, requirements panel, completed-plans dialog and profile plan
/// views read names, rarity, class, phases, skill names and icons, mastery
/// costs and module names and icons; the editor fetches the full operator on
/// its own. A full operator is 37,526 B (Jessica) to 97,930 B (Exusiai the
/// New Covenant), so 193 plans shipped about 10 MB per list, twice (backend to
/// the frontend server, then to the browser).
const PLAN_OPERATOR_EMPTIED_ARRAYS: [&str; 6] = [
    "talents",
    "potentialRanks",
    "favorKeyFrames",
    "baseSkills",
    "audio",
    "drones",
];
const PLAN_OPERATOR_NULLED: [&str; 2] = ["handbook", "profile"];

/// Strips `operator_json` to what a plan view reads: the fields above, every
/// skill level past the first (the views read only the skill name), and each
/// module's stat `data` and description.
fn slim_plan_operator(map: &mut serde_json::Map<String, serde_json::Value>) {
    use serde_json::Value;
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

/// A plan as the API returns it, with the operator's game data, slimmed to
/// what a plan view reads, tagged with the server it was resolved from.
fn plan_response(
    plan: OperatorPlan,
    groups: Vec<String>,
    operator: &Operator,
    server: Server,
    met: bool,
) -> Result<OperatorPlanResponse, ApiError> {
    let mut operator_json =
        serde_json::to_value(operator).map_err(|e| ApiError::Internal(e.into()))?;
    if let serde_json::Value::Object(map) = &mut operator_json {
        slim_plan_operator(map);
        map.insert(
            "server".to_string(),
            serde_json::Value::String(server.as_str().to_string()),
        );
    }
    Ok(OperatorPlanResponse {
        plan,
        groups,
        operator: operator_json,
        met,
    })
}

pub async fn list_plans(
    state: &AppState,
    user_id: Uuid,
    active_ids: Vec<String>,
    max_tier: Option<i16>,
    by_operator: bool,
) -> Result<PlannerResponse, ApiError> {
    let plans = queries::list_plans(&state.db, user_id).await?;
    let roster_map = roster_by_operator(state, user_id).await?;

    let db_inv = items_queries::get_inventory(&state.db, user_id).await?;
    let profile = users_queries::find_by_id(&state.db, user_id).await?;
    let current_lmd = profile.as_ref().and_then(|p| p.lmd).unwrap_or(0);

    let gamedata = state.default_game_data();

    // The profile row, not the item table, holds LMD; EXP is what the owned
    // battle records grant.
    let mut inventory_map: HashMap<String, i32> = db_inv
        .into_iter()
        .map(|i| (i.item_id, i.quantity))
        .collect();
    inventory_map.insert(LMD_ITEM.to_owned(), current_lmd);
    let total_exp: i32 = gamedata
        .materials
        .exp_items
        .values()
        .map(|exp_item| {
            let qty = inventory_map.get(&exp_item.id).copied().unwrap_or(0);
            qty * exp_item.gain_exp
        })
        .sum();
    inventory_map.insert(EXP_ITEM.to_owned(), total_exp);

    let user_building = building_queries::get_building(&state.db, user_id)
        .await?
        .map_or_else(
            || UserBuilding { rooms: Vec::new() },
            |json| UserBuilding::from_json(&json),
        );
    let clears = stages_queries::get_user_stage_clears(&state.db, user_id)
        .await?
        .clears;

    let groups = queries::list_groups(&state.db, user_id).await?;
    let mut group_map = group_names_by_plan(state, user_id).await?;

    let mut plans_with_ops = Vec::new();
    let mut responses = Vec::with_capacity(plans.len());
    for plan in plans {
        let Some((operator, server)) =
            resolve_operator(state, state.default_server, &plan.operator_id)
        else {
            continue;
        };
        let roster_entry = roster_map.get(&plan.operator_id);
        let met = stored_plan_met(&plan, roster_entry);
        let plan_groups = group_map.remove(&plan.id).unwrap_or_default();
        responses.push(plan_response(
            plan.clone(),
            plan_groups,
            &operator,
            server,
            met,
        )?);
        plans_with_ops.push((plan, operator, roster_entry));
    }

    let asset_index = state.default_asset_index();
    let ctx = PlannerCtx {
        gamedata: gamedata.as_ref(),
        asset_index: asset_index.as_ref(),
        inventory_map: &inventory_map,
        user_building: &user_building,
        clears: &clears,
    };
    let aggregated_requirements =
        calculate_requirements(&ctx, &plans_with_ops, &active_ids, max_tier)?;
    let operator_requirements = by_operator
        .then(|| requirements_by_operator(&ctx, &plans_with_ops, &active_ids, max_tier))
        .transpose()?;

    Ok(PlannerResponse {
        plans: responses,
        aggregated_requirements,
        operator_requirements,
        groups,
        last_synced_at: profile.as_ref().map(|p| p.updated_at),
    })
}

/// The player's plans they chose to show on their profile.
pub async fn list_public_plans(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<OperatorPlanResponse>, ApiError> {
    let plans = queries::list_plans(&state.db, user_id).await?;
    let mut group_map = group_names_by_plan(state, user_id).await?;
    let roster_map = roster_by_operator(state, user_id).await?;

    let mut responses = Vec::new();
    for plan in plans {
        if !plan.display_on_profile {
            continue;
        }
        let Some((operator, server)) =
            resolve_operator(state, state.default_server, &plan.operator_id)
        else {
            continue;
        };
        let plan_groups = group_map.remove(&plan.id).unwrap_or_default();
        let met = stored_plan_met(&plan, roster_map.get(&plan.operator_id));
        responses.push(plan_response(plan, plan_groups, &operator, server, met)?);
    }
    Ok(responses)
}

/// Creates or replaces the caller's plan for one operator. `groups` replaces
/// the plan's group memberships; `None` keeps them.
pub async fn upsert_plan(
    state: &AppState,
    user_id: Uuid,
    operator_id: &str,
    mut input: PlanInput,
    groups: Option<Vec<String>>,
) -> Result<OperatorPlanResponse, ApiError> {
    validation::ensure_plannable_id(operator_id)?;
    let (operator, server) =
        resolve_operator(state, state.default_server, operator_id).ok_or(ApiError::NotFound)?;

    // Plans only go up: a target below the roster is raised to it before the
    // range checks, so the stored plan is what validation passed.
    let roster_entry = roster_queries::get_operator(&state.db, user_id, operator_id).await?;
    raise_to_roster(&mut input, roster_entry.as_ref())?;
    validation::validate_plan(&operator, &input)?;

    let plan =
        queries::upsert_plan(&state.db, user_id, operator_id, &input, groups.as_deref()).await?;

    let met = stored_plan_met(&plan, roster_entry.as_ref());

    // Read back rather than echo the request: the store sorts and dedups the
    // names, and the list endpoints order them the same way.
    let plan_groups = queries::get_plan_group_names(&state.db, plan.id).await?;

    plan_response(plan, plan_groups, &operator, server, met)
}

pub async fn delete_plan(
    state: &AppState,
    user_id: Uuid,
    operator_id: &str,
) -> Result<(), ApiError> {
    queries::delete_plan(&state.db, user_id, operator_id)
        .await
        .map_err(Into::into)
}

pub async fn delete_plans(
    state: &AppState,
    user_id: Uuid,
    operator_ids: &[String],
) -> Result<DeletePlansResponse, ApiError> {
    if operator_ids.len() > MAX_BULK_DELETE {
        return Err(ApiError::BadRequest(format!(
            "At most {MAX_BULK_DELETE} plans can be deleted at once"
        )));
    }
    if operator_ids.is_empty() {
        return Ok(DeletePlansResponse { deleted: 0 });
    }
    let deleted = queries::delete_plans(&state.db, user_id, operator_ids).await?;
    Ok(DeletePlansResponse {
        deleted: i64::try_from(deleted).unwrap_or(i64::MAX),
    })
}

pub async fn create_group(
    state: &AppState,
    user_id: Uuid,
    name: &str,
) -> Result<PlanGroup, ApiError> {
    queries::create_group(&state.db, user_id, name)
        .await
        .map_err(Into::into)
}

/// Renames and/or pins a group. At least one of `name` and `pinned` must be
/// given.
pub async fn update_group(
    state: &AppState,
    user_id: Uuid,
    old_name: &str,
    name: Option<&str>,
    pinned: Option<bool>,
) -> Result<PlanGroup, ApiError> {
    if name.is_none() && pinned.is_none() {
        return Err(ApiError::BadRequest(
            "Nothing to update: give `name`, `pinned`, or both".into(),
        ));
    }
    queries::update_group(&state.db, user_id, old_name, name, pinned)
        .await
        .map_err(Into::into)
}

pub async fn delete_group(state: &AppState, user_id: Uuid, name: &str) -> Result<(), ApiError> {
    queries::delete_group(&state.db, user_id, name)
        .await
        .map_err(Into::into)
}

/// A stored preset row as the API returns it. The row was validated on the
/// way in, so a target that does not parse is a server fault.
fn preset_from_row(row: queries::PlanPresetRow) -> Result<PlanPreset, ApiError> {
    let target = serde_json::from_value(row.target).map_err(|e| ApiError::Internal(e.into()))?;
    Ok(PlanPreset {
        id: row.id,
        name: row.name,
        target,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

pub async fn list_presets(state: &AppState, user_id: Uuid) -> Result<Vec<PlanPreset>, ApiError> {
    queries::list_presets(&state.db, user_id)
        .await?
        .into_iter()
        .map(preset_from_row)
        .collect()
}

/// Creates the preset, or replaces the target of the caller's preset by that
/// name.
pub async fn upsert_preset(
    state: &AppState,
    user_id: Uuid,
    name: &str,
    target: &PresetTarget,
) -> Result<PlanPreset, ApiError> {
    let name = name.trim();
    validation::validate_preset(name, target)?;
    let target_json = serde_json::to_value(target).map_err(|e| ApiError::Internal(e.into()))?;
    let row = queries::upsert_preset(&state.db, user_id, name, target_json).await?;
    preset_from_row(row)
}

pub async fn delete_preset(state: &AppState, user_id: Uuid, name: &str) -> Result<(), ApiError> {
    queries::delete_preset(&state.db, user_id, name).await?;
    Ok(())
}
