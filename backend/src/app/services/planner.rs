use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    app::{error::ApiError, services::operators::resolve_operator, state::AppState},
    core::{
        gamedata::{
            assets::AssetIndex,
            enrich::resolve_item_icon,
            types::{
                GameData,
                building::{BuildingDataFile, FormulaCost, FormulaRoomReq},
                material::{ItemRarity, Materials},
                operator::{Operator, OperatorPhase},
            },
        },
        grade::{base::types::UserBuilding, stages::types::StageClear},
    },
    database::{
        models::{
            planner::{
                OperatorPlan, OperatorPlanResponse, PlanGroup, PlanRecipe, PlanRecipeCost,
                PlanRequirementItem, PlannerResponse, TargetModulePlan, TargetSkillPlan,
            },
            roster::RosterEntry,
        },
        queries::{
            building as building_queries, items as items_queries, planner as queries,
            roster as roster_queries, stages as stages_queries, users as users_queries,
        },
    },
};

#[derive(Deserialize)]
struct RosterMastery {
    index: i16,
    mastery: i16,
}

#[derive(Deserialize)]
struct RosterModule {
    id: String,
    level: i16,
}

const fn phase_to_int(phase: &OperatorPhase) -> i16 {
    match phase {
        OperatorPhase::Elite0 => 0,
        OperatorPhase::Elite1 => 1,
        OperatorPhase::Elite2 => 2,
    }
}

fn module_phase_to_int(phase: &str) -> i16 {
    match phase {
        "PHASE_1" | "1" => 1,
        "PHASE_2" | "2" => 2,
        _ => 0,
    }
}

fn requirement_sort_key(id: &str, name: &str, rarity: i16) -> (i8, i8) {
    let name = name.to_lowercase();
    if id == "5001" {
        return (0, 0);
    }
    if id == "4001" {
        return (1, 0);
    }
    if name.contains("skill summary") {
        return (2, -(rarity as i8));
    }
    if name.contains("dualchip") {
        return (3, 0);
    }
    if name.contains("chip pack") {
        return (3, 1);
    }
    if name.contains("chip") {
        return (3, 2);
    }
    if name.contains("data supplement instrument") {
        return (4, 0);
    }
    if name.contains("data supplement stick") {
        return (4, 1);
    }
    if name.contains("module data block") {
        return (4, 2);
    }
    (5, -(rarity as i8))
}

/// Chip cross-class conversion recipes in the workshop (`F_ASC`, e.g. 3 Medic
/// Chips -> 2 Defender Chips). Converting chips is a waste and never the intended
/// way to acquire them, so the planner treats chips as obtain-only. Dualchip
/// factory recipes carry the same `F_ASC` type but live in `manufact_formulas`
/// and stay craftable — combining chip packs with a catalyst *is* the intended
/// way to obtain dualchips.
const CHIP_CONVERSION_FORMULA_TYPE: &str = "F_ASC";
const CHIP_CRAFT_REASON: &str =
    "Chip conversion isn't recommended — obtain chips from stages, the store, or events";

/// Read-only inputs shared by the whole requirement computation.
struct PlannerCtx<'a> {
    gamedata: &'a GameData,
    asset_index: &'a AssetIndex,
    inventory_map: &'a HashMap<String, i32>,
    user_building: &'a UserBuilding,
    clears: &'a HashMap<String, StageClear>,
}

fn item_display_meta(item_id: &str, materials: &Materials) -> (String, String, i16) {
    if item_id == "4001" {
        return ("LMD".to_owned(), "GOLD".to_owned(), 0);
    }
    if item_id == "5001" {
        return ("EXP".to_owned(), "EXP_PLAYER".to_owned(), 0);
    }
    if let Some(item) = materials.items.get(item_id) {
        let rarity = match item.rarity {
            ItemRarity::Tier1 => 1,
            ItemRarity::Tier2 => 2,
            ItemRarity::Tier3 => 3,
            ItemRarity::Tier4 => 4,
            ItemRarity::Tier5 => 5,
            ItemRarity::Tier6 => 6,
        };
        return (
            item.name.clone(),
            format!("{:?}", item.item_type).to_uppercase(),
            rarity,
        );
    }
    (item_id.to_owned(), "MATERIAL".to_owned(), 0)
}

#[allow(clippy::too_many_arguments)]
fn resolve_requirement_item(
    ctx: &PlannerCtx,
    item_id: &str,
    required_count: i32,
    inventory_count: i32,
    craftable_count: i32,
    missing_count: i32,
    can_craft: bool,
    craft_reason: String,
    recipe: Option<PlanRecipe>,
) -> PlanRequirementItem {
    let (icon_id, image) = resolve_item_icon(item_id, &ctx.gamedata.materials, ctx.asset_index);
    let (name, item_type, rarity) = item_display_meta(item_id, &ctx.gamedata.materials);
    let (sort_group, sort_subrank) = requirement_sort_key(item_id, &name, rarity);

    PlanRequirementItem {
        id: item_id.to_owned(),
        required_count,
        inventory_count,
        missing_count,
        craftable_count,
        name,
        icon_id,
        image,
        item_type,
        rarity,
        sort_group,
        sort_subrank,
        can_craft,
        craft_reason,
        recipe,
    }
}

fn calculate_leveling_costs(
    operator: &Operator,
    gamedata: &GameData,
    current_elite: i16,
    current_level: i16,
    target_elite: i16,
    target_level: i16,
    materials: &mut HashMap<String, i32>,
) {
    let consts = &gamedata.consts;
    let mut exp_needed = 0;
    let mut lmd_needed = 0;

    let compute_range = |elite: usize, from_level: i16, to_level: i16| -> (i32, i32) {
        let mut exp = 0;
        let mut lmd = 0;
        if let Some(exp_map) = consts.character_exp_map.get(elite) {
            for level in (from_level as usize)..(to_level as usize) {
                if let Some(&val) = exp_map.values.get(level - 1) {
                    exp += val;
                }
            }
        }
        if let Some(lmd_map) = consts.character_upgrade_cost_map.get(elite) {
            for level in (from_level as usize)..(to_level as usize) {
                if let Some(&val) = lmd_map.values.get(level - 1) {
                    lmd += val;
                }
            }
        }
        (exp, lmd)
    };

    if target_elite > current_elite {
        let max_level = operator.phases[current_elite as usize].max_level as i16;
        let (exp, lmd) = compute_range(current_elite as usize, current_level, max_level);
        exp_needed += exp;
        lmd_needed += lmd;

        for elite in (current_elite + 1)..target_elite {
            let max_level = operator.phases[elite as usize].max_level as i16;
            let (exp, lmd) = compute_range(elite as usize, 1, max_level);
            exp_needed += exp;
            lmd_needed += lmd;
        }

        let (exp, lmd) = compute_range(target_elite as usize, 1, target_level);
        exp_needed += exp;
        lmd_needed += lmd;
    } else if target_elite == current_elite && target_level > current_level {
        let (exp, lmd) = compute_range(target_elite as usize, current_level, target_level);
        exp_needed += exp;
        lmd_needed += lmd;
    }

    if lmd_needed > 0 {
        *materials.entry("4001".to_owned()).or_insert(0) += lmd_needed;
    }
    if exp_needed > 0 {
        *materials.entry("5001".to_owned()).or_insert(0) += exp_needed;
    }
}

/// Claims up to `count` units of `item_id` from the shared pool, returning how
/// many were actually taken.
fn claim_from_pool(pool: &mut HashMap<String, i32>, item_id: &str, count: i32) -> i32 {
    let entry = pool.entry(item_id.to_owned()).or_insert(0);
    let take = (*entry).min(count).max(0);
    *entry -= take;
    take
}

/// A craft recipe normalized across workshop and factory formulas.
struct RecipeView<'a> {
    output_count: i32,
    costs: &'a [FormulaCost],
    require_rooms: &'a [FormulaRoomReq],
    /// Stage clears gating the recipe (workshop formulas only).
    require_stage_ids: Vec<&'a str>,
}

enum RecipeLookup<'a> {
    Found(RecipeView<'a>),
    ChipConversion,
    None,
}

fn find_recipe<'a>(building: &'a BuildingDataFile, item_id: &str) -> RecipeLookup<'a> {
    if let Some(formula) = building
        .workshop_formulas
        .values()
        .find(|f| f.item_id == item_id)
    {
        if formula.formula_type == CHIP_CONVERSION_FORMULA_TYPE {
            return RecipeLookup::ChipConversion;
        }
        return RecipeLookup::Found(RecipeView {
            output_count: formula.count,
            costs: &formula.costs,
            require_rooms: &formula.require_rooms,
            require_stage_ids: formula
                .require_stages
                .iter()
                .map(|s| s.stage_id.as_str())
                .collect(),
        });
    }
    if let Some(formula) = building
        .manufact_formulas
        .values()
        .find(|f| f.item_id == item_id)
    {
        return RecipeLookup::Found(RecipeView {
            output_count: formula.count,
            costs: &formula.costs,
            require_rooms: &formula.require_rooms,
            require_stage_ids: Vec::new(),
        });
    }
    RecipeLookup::None
}

fn unmet_recipe_requirements(ctx: &PlannerCtx, view: &RecipeView) -> Vec<String> {
    let mut unmet_reqs = Vec::new();
    for room_req in view.require_rooms {
        let matching_count = ctx
            .user_building
            .rooms
            .iter()
            .filter(|r| r.room_type == room_req.room_id && r.level >= room_req.room_level)
            .count();
        if matching_count < room_req.room_count as usize {
            unmet_reqs.push(format!("{} lv.{}", room_req.room_id, room_req.room_level));
        }
    }
    for stage_id in &view.require_stage_ids {
        let is_cleared = ctx
            .clears
            .get(*stage_id)
            .is_some_and(|c| c.state >= 2 || c.complete_times > 0);
        if !is_cleared {
            let stage_code = ctx
                .gamedata
                .stages
                .get(*stage_id)
                .map_or(*stage_id, |s| s.code.as_str());
            unmet_reqs.push(format!("Stage {stage_code}"));
        }
    }
    unmet_reqs
}

/// Builds the requirement node for one item, allocating units out of `pool` — the
/// shared ledger of not-yet-claimed inventory — so the same unit is never counted
/// toward two requirements. Inventory is claimed first; only the remaining
/// shortfall is (recursively) crafted, with ingredient claims drawn from the same
/// ledger. `craftable_count` is therefore plan-scoped: the units that will
/// actually be crafted for this requirement, not total workshop capacity.
fn build_requirement_tree(
    ctx: &PlannerCtx,
    item_id: &str,
    required_count: i32,
    reserved_from_pool: Option<i32>,
    pool: &mut HashMap<String, i32>,
    visited: &[&str],
) -> PlanRequirementItem {
    let inv_count = *ctx.inventory_map.get(item_id).unwrap_or(&0);

    let satisfied_from_inv =
        reserved_from_pool.unwrap_or_else(|| claim_from_pool(pool, item_id, required_count));
    let shortfall = (required_count - satisfied_from_inv).max(0);

    let lookup = find_recipe(&ctx.gamedata.building, item_id);

    let mut craftable_count = 0;
    let mut can_craft = false;
    let mut craft_reason = match &lookup {
        RecipeLookup::ChipConversion => CHIP_CRAFT_REASON.to_owned(),
        _ => "No workshop or factory formula".to_owned(),
    };
    let mut recipe = None;

    if let RecipeLookup::Found(view) = lookup {
        if view.costs.iter().any(|c| visited.contains(&c.id.as_str())) {
            return resolve_requirement_item(
                ctx,
                item_id,
                required_count,
                inv_count,
                0,
                shortfall,
                false,
                "Recipe cycle detected".to_owned(),
                None,
            );
        }

        let unmet_reqs = unmet_recipe_requirements(ctx, &view);
        let gates_met = unmet_reqs.is_empty();

        let crafts_needed = if shortfall > 0 {
            (shortfall + view.output_count - 1) / view.output_count
        } else {
            0
        };

        // A gated recipe is still shown, but nothing will actually be crafted, so
        // expand it against a scratch copy — it must not consume pool units that
        // other requirements can still use.
        let mut scratch;
        let child_pool: &mut HashMap<String, i32> = if gates_met {
            pool
        } else {
            scratch = pool.clone();
            &mut scratch
        };

        let mut next_visited = visited.to_vec();
        next_visited.push(item_id);

        let mut costs = Vec::with_capacity(view.costs.len());
        for cost in view.costs {
            let cost_item = build_requirement_tree(
                ctx,
                &cost.id,
                crafts_needed * cost.count,
                None,
                child_pool,
                &next_visited,
            );
            costs.push(PlanRecipeCost {
                count: cost.count,
                item: cost_item,
            });
        }

        if gates_met {
            can_craft = true;
            craft_reason = String::new();
            if crafts_needed > 0 {
                // Each ingredient covers its requirement from claimed inventory
                // plus its own recursive crafts; the achievable craft count is the
                // tightest ingredient.
                let achievable =
                    costs
                        .iter()
                        .filter(|c| c.count > 0)
                        .fold(crafts_needed, |acc, c| {
                            let satisfied = c.item.required_count - c.item.missing_count;
                            acc.min(satisfied / c.count)
                        });
                craftable_count = achievable * view.output_count;
            }
        } else {
            craft_reason = format!("Requirements not met: {}", unmet_reqs.join(", "));
        }

        recipe = Some(PlanRecipe {
            count: view.output_count,
            costs,
        });
    }

    let missing_count = (shortfall - craftable_count).max(0);

    resolve_requirement_item(
        ctx,
        item_id,
        required_count,
        inv_count,
        craftable_count,
        missing_count,
        can_craft,
        craft_reason,
        recipe,
    )
}

fn get_plan_direct_materials(
    gamedata: &GameData,
    plan: &OperatorPlan,
    operator: &Operator,
    roster_entry: Option<&RosterEntry>,
) -> Result<HashMap<String, i32>, ApiError> {
    let current_elite = roster_entry.map_or(0, |r| r.elite);
    let current_level = roster_entry.map_or(1, |r| r.level);
    let current_skill_level = roster_entry.map_or(1, |r| r.skill_level);

    let current_masteries: Vec<RosterMastery> = roster_entry
        .map(|r| serde_json::from_value(r.masteries.clone()).unwrap_or_default())
        .unwrap_or_default();
    let current_modules: Vec<RosterModule> = roster_entry
        .map(|r| serde_json::from_value(r.modules.clone()).unwrap_or_default())
        .unwrap_or_default();

    let skill_plans: Vec<TargetSkillPlan> = serde_json::from_value(plan.target_skills.clone())
        .map_err(|_| ApiError::BadRequest("Invalid target_skills format".into()))?;

    let module_plans: Vec<TargetModulePlan> =
        serde_json::from_value(plan.target_modules.clone())
            .map_err(|_| ApiError::BadRequest("Invalid target_modules format".into()))?;

    let mut materials = HashMap::new();

    calculate_leveling_costs(
        operator,
        gamedata,
        current_elite,
        current_level,
        plan.target_elite,
        plan.target_level,
        &mut materials,
    );

    if plan.target_elite > current_elite {
        for elite in (current_elite + 1)..=plan.target_elite {
            if let Some(ref evolve_costs) = operator.phases[elite as usize].evolve_cost {
                for cost in evolve_costs {
                    *materials.entry(cost.id.clone()).or_insert(0) += cost.count;
                }
            }
        }
    }

    if plan.target_skill_level > current_skill_level {
        for i in (current_skill_level - 1)..(plan.target_skill_level - 1) {
            if let Some(lvl_up) = operator.all_skill_level_up.get(i as usize) {
                for cost in &lvl_up.lvl_up_cost {
                    *materials.entry(cost.id.clone()).or_insert(0) += cost.count;
                }
            }
        }
    }

    for target_skill in &skill_plans {
        let idx = target_skill.skill_index;
        let current_mast = current_masteries
            .iter()
            .find(|m| m.index == idx)
            .map_or(0, |m| m.mastery);

        if target_skill.mastery_level > current_mast
            && let Some(skill_entry) = operator.skills.get(idx as usize)
        {
            for i in (current_mast as usize)..(target_skill.mastery_level as usize) {
                if let Some(cond) = skill_entry.level_up_cost_cond.get(i) {
                    for cost in &cond.level_up_cost {
                        *materials.entry(cost.id.clone()).or_insert(0) += cost.count;
                    }
                }
            }
        }
    }

    for target_module in &module_plans {
        let current_level = current_modules
            .iter()
            .find(|m| m.id == target_module.module_id)
            .map_or(0, |m| m.level);
        if target_module.module_stage > current_level
            && let Some(op_mod) = operator
                .modules
                .iter()
                .find(|m| m.module.uni_equip_id == target_module.module_id)
            && let Some(ref item_cost_map) = op_mod.module.item_cost
        {
            for stage in (current_level + 1)..=target_module.module_stage {
                if let Some(costs) = item_cost_map.get(&stage.to_string()) {
                    for cost in costs {
                        *materials.entry(cost.id.clone()).or_insert(0) += cost.count;
                    }
                }
            }
        }
    }

    Ok(materials)
}

/// Builds requirement trees for the aggregated plan materials against a single
/// shared inventory ledger. Each requirement's own inventory share is reserved
/// before any crafting is evaluated, so a craft can't consume units another
/// requirement line needs directly.
fn build_all_requirements(
    ctx: &PlannerCtx,
    combined_materials: HashMap<String, i32>,
) -> Vec<PlanRequirementItem> {
    // Deterministic allocation order matching the display sort, so contended
    // inventory is claimed by the rows the user sees first.
    let mut ordered: Vec<(String, i32)> = combined_materials.into_iter().collect();
    ordered.sort_by_cached_key(|(item_id, _)| {
        let (name, _, rarity) = item_display_meta(item_id, &ctx.gamedata.materials);
        let (sort_group, sort_subrank) = requirement_sort_key(item_id, &name, rarity);
        (sort_group, sort_subrank, name.to_lowercase())
    });

    let mut pool = ctx.inventory_map.clone();
    let mut reserved: HashMap<String, i32> = HashMap::with_capacity(ordered.len());
    for (item_id, count) in &ordered {
        reserved.insert(item_id.clone(), claim_from_pool(&mut pool, item_id, *count));
    }

    ordered
        .iter()
        .map(|(item_id, count)| {
            build_requirement_tree(
                ctx,
                item_id,
                *count,
                Some(reserved[item_id]),
                &mut pool,
                &[],
            )
        })
        .collect()
}

fn calculate_requirements(
    ctx: &PlannerCtx,
    plans_with_ops: &[(OperatorPlan, Operator, Option<&RosterEntry>)],
    active_ids: &[String],
) -> Result<Vec<PlanRequirementItem>, ApiError> {
    let mut combined_materials = HashMap::new();
    for (plan, operator, roster_entry) in plans_with_ops {
        let is_active = active_ids.is_empty() || active_ids.contains(&plan.operator_id);
        if !is_active {
            continue;
        }
        let plan_materials =
            get_plan_direct_materials(ctx.gamedata, plan, operator, *roster_entry)?;
        for (item_id, count) in plan_materials {
            *combined_materials.entry(item_id).or_insert(0) += count;
        }
    }

    Ok(build_all_requirements(ctx, combined_materials))
}

pub async fn list_plans(
    state: &AppState,
    user_id: Uuid,
    active_ids: Vec<String>,
) -> Result<PlannerResponse, ApiError> {
    let plans = queries::list_plans(&state.db, user_id).await?;
    let roster = roster_queries::get_roster(&state.db, user_id).await?;

    let roster_map: HashMap<String, RosterEntry> = roster
        .into_iter()
        .map(|r| (r.operator_id.clone(), r))
        .collect();

    let db_inv = items_queries::get_inventory(&state.db, user_id).await?;
    let profile = users_queries::find_by_id(&state.db, user_id).await?;
    let current_lmd = profile.as_ref().and_then(|p| p.lmd).unwrap_or(0);

    let gamedata = state.default_game_data();

    let mut inventory_map: HashMap<String, i32> = db_inv
        .into_iter()
        .map(|i| (i.item_id, i.quantity))
        .collect();
    inventory_map.insert("4001".to_owned(), current_lmd);

    let total_exp: i32 = gamedata
        .materials
        .exp_items
        .values()
        .map(|exp_item| {
            let qty = inventory_map.get(&exp_item.id).copied().unwrap_or(0);
            qty * exp_item.gain_exp
        })
        .sum();
    inventory_map.insert("5001".to_owned(), total_exp);

    let building_json = building_queries::get_building(&state.db, user_id).await?;
    let user_building = building_json.map_or_else(
        || UserBuilding { rooms: Vec::new() },
        |json| UserBuilding::from_json(&json),
    );

    let clears_data = stages_queries::get_user_stage_clears(&state.db, user_id).await?;
    let clears = clears_data.clears;
    let mut plans_with_ops = Vec::new();
    let mut responses = Vec::with_capacity(plans.len());

    let groups = queries::list_groups(&state.db, user_id).await?;
    let mapping = queries::get_all_plan_groups(&state.db, user_id).await?;
    let mut group_map: HashMap<Uuid, Vec<String>> = HashMap::new();
    for (plan_id, group_name) in mapping {
        group_map.entry(plan_id).or_default().push(group_name);
    }

    for plan in plans {
        if let Some((operator, server)) =
            resolve_operator(state, state.default_server, &plan.operator_id)
        {
            let roster_entry = roster_map.get(&plan.operator_id);
            plans_with_ops.push((plan.clone(), operator.clone(), roster_entry));
            let plan_groups = group_map.remove(&plan.id).unwrap_or_default();
            let mut op_val =
                serde_json::to_value(&operator).map_err(|e| ApiError::Internal(e.into()))?;
            if let serde_json::Value::Object(map) = &mut op_val {
                map.insert(
                    "server".to_string(),
                    serde_json::Value::String(server.as_str().to_string()),
                );
            }
            responses.push(OperatorPlanResponse {
                plan,
                groups: plan_groups,
                operator: op_val,
            });
        }
    }

    let asset_index = state.default_asset_index();
    let ctx = PlannerCtx {
        gamedata: gamedata.as_ref(),
        asset_index: asset_index.as_ref(),
        inventory_map: &inventory_map,
        user_building: &user_building,
        clears: &clears,
    };
    let aggregated_requirements = calculate_requirements(&ctx, &plans_with_ops, &active_ids)?;

    Ok(PlannerResponse {
        plans: responses,
        aggregated_requirements,
        groups,
    })
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

#[allow(clippy::too_many_arguments)]
pub async fn upsert_plan(
    state: &AppState,
    user_id: Uuid,
    operator_id: &str,
    target_elite: i16,
    target_level: i16,
    target_skill_level: i16,
    target_skills: serde_json::Value,
    target_modules: serde_json::Value,
    display_on_profile: bool,
    groups: Option<Vec<String>>,
) -> Result<OperatorPlanResponse, ApiError> {
    if operator_id == "char_4195_radian" {
        return Err(ApiError::BadRequest(
            "Raidian's progression is locked to IS6, and cannot be planned".into(),
        ));
    }

    let (operator, server) =
        resolve_operator(state, state.default_server, operator_id).ok_or(ApiError::NotFound)?;

    if operator.is_not_obtainable {
        return Err(ApiError::BadRequest(format!(
            "Operator {} is not obtainable, and their upgrades cannot be planned",
            operator.name
        )));
    }

    let max_elite = (operator.phases.len() as i16) - 1;
    if target_elite < 0 || target_elite > max_elite {
        return Err(ApiError::BadRequest(format!(
            "Invalid target elite promotion for operator {} (max: {})",
            operator.name, max_elite
        )));
    }

    let phase = &operator.phases[target_elite as usize];
    if target_level < 1 || target_level > (phase.max_level as i16) {
        return Err(ApiError::BadRequest(format!(
            "Invalid target level for operator {} (max: {})",
            operator.name, phase.max_level
        )));
    }

    let max_skill_level = (operator.all_skill_level_up.len() + 1) as i16;

    if target_skill_level < 1 || target_skill_level > max_skill_level {
        return Err(ApiError::BadRequest(format!(
            "Invalid target skill level for operator {} (max: {})",
            operator.name, max_skill_level
        )));
    }

    if target_skill_level > 1 {
        for idx in 2..=target_skill_level {
            if let Some(lvl_up) = operator.all_skill_level_up.get((idx - 2) as usize) {
                let required_phase = phase_to_int(&lvl_up.unlock_cond.phase);
                if required_phase > target_elite {
                    return Err(ApiError::BadRequest(format!(
                        "Target skill level {idx} requires Elite {required_phase} or higher"
                    )));
                }
                let required_level = lvl_up.unlock_cond.level as i16;
                if required_phase == target_elite && required_level > target_level {
                    return Err(ApiError::BadRequest(format!(
                        "Target skill level {idx} requires Level {required_level} at Elite {required_phase}"
                    )));
                }
            }
        }
    }

    let skill_plans: Vec<TargetSkillPlan> = serde_json::from_value(target_skills.clone())
        .map_err(|_| ApiError::BadRequest("Invalid target_skills format".into()))?;

    for plan in &skill_plans {
        if plan.skill_index < 0 || plan.skill_index >= (operator.skills.len() as i16) {
            return Err(ApiError::BadRequest(format!(
                "Invalid skill index {} for operator {}",
                plan.skill_index, operator.name,
            )));
        }

        let skill_entry = &operator.skills[plan.skill_index as usize];
        let max_mastery = skill_entry.level_up_cost_cond.len() as i16;
        if plan.mastery_level < 0 || plan.mastery_level > max_mastery {
            return Err(ApiError::BadRequest(format!(
                "Skill mastery must be between 0 and {} for operator {} skill index {}",
                max_mastery, operator.name, plan.skill_index
            )));
        }

        if plan.mastery_level > 0 {
            for m in 1..=plan.mastery_level {
                if let Some(cond) = skill_entry.level_up_cost_cond.get((m - 1) as usize) {
                    let required_phase = phase_to_int(&cond.unlock_cond.phase);
                    let required_level = cond.unlock_cond.level as i16;

                    if required_phase > target_elite
                        || (required_phase == target_elite && required_level > target_level)
                    {
                        return Err(ApiError::BadRequest(format!(
                            "Mastery {} for operator {} skill index {} requires Elite {} level {}",
                            plan.mastery_level,
                            operator.name,
                            plan.skill_index,
                            required_phase,
                            required_level
                        )));
                    }
                }
            }
        }
    }

    let module_plans: Vec<TargetModulePlan> = serde_json::from_value(target_modules.clone())
        .map_err(|_| ApiError::BadRequest("Invalid target_modules format".into()))?;

    for plan in &module_plans {
        let op_mod = operator
            .modules
            .iter()
            .find(|m| m.module.uni_equip_id == plan.module_id)
            .ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "Invalid module ID {} for operator {}",
                    plan.module_id, operator.name
                ))
            })?;

        if plan.module_stage < 0 || plan.module_stage > 3 {
            return Err(ApiError::BadRequest(
                "Module stage must be between 0 and 3".into(),
            ));
        }

        if plan.module_stage > 0 {
            let required_phase = module_phase_to_int(&op_mod.module.unlock_evolve_phase);
            let required_level = op_mod.module.unlock_level as i16;
            if required_phase > target_elite
                || (required_phase == target_elite && required_level > target_level)
            {
                return Err(ApiError::BadRequest(format!(
                    "Module unlock/upgrades require Elite {required_phase} level {required_level}."
                )));
            }
        }
    }

    let mut tx = state.db.begin().await?;

    let plan = sqlx::query_as::<_, OperatorPlan>(
        r"
        INSERT INTO operator_plans (
            user_id,
            operator_id,
            target_elite,
            target_level,
            target_skill_level,
            target_skills,
            target_modules,
            display_on_profile
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (user_id, operator_id) DO UPDATE SET
            target_elite = EXCLUDED.target_elite,
            target_level = EXCLUDED.target_level,
            target_skill_level = EXCLUDED.target_skill_level,
            target_skills = EXCLUDED.target_skills,
            target_modules = EXCLUDED.target_modules,
            display_on_profile = EXCLUDED.display_on_profile,
            updated_at = NOW()
        RETURNING *
        ",
    )
    .bind(user_id)
    .bind(operator_id)
    .bind(target_elite)
    .bind(target_level)
    .bind(target_skill_level)
    .bind(target_skills)
    .bind(target_modules)
    .bind(display_on_profile)
    .fetch_one(&mut *tx)
    .await?;

    if let Some(group_names) = &groups {
        sqlx::query("DELETE FROM plan_group_members WHERE operator_plan_id = $1")
            .bind(plan.id)
            .execute(&mut *tx)
            .await?;

        for group_name in group_names {
            let group_id: Uuid = sqlx::query_scalar(
                r"
                INSERT INTO plan_groups (user_id, name)
                VALUES ($1, $2)
                ON CONFLICT (user_id, name) DO UPDATE SET name = EXCLUDED.name
                RETURNING id
                ",
            )
            .bind(user_id)
            .bind(group_name)
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query(
                r"
                INSERT INTO plan_group_members (plan_group_id, operator_plan_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                ",
            )
            .bind(group_id)
            .bind(plan.id)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    let plan_groups = if let Some(g) = groups {
        g
    } else {
        sqlx::query_scalar(
            r"
            SELECT pg.name
            FROM plan_groups pg
            JOIN plan_group_members pgm ON pgm.plan_group_id = pg.id
            WHERE pgm.operator_plan_id = $1
            ORDER BY pg.name ASC
            ",
        )
        .bind(plan.id)
        .fetch_all(&state.db)
        .await?
    };

    let mut op_val = serde_json::to_value(&operator).map_err(|e| ApiError::Internal(e.into()))?;
    if let serde_json::Value::Object(map) = &mut op_val {
        map.insert(
            "server".to_string(),
            serde_json::Value::String(server.as_str().to_string()),
        );
    }

    Ok(OperatorPlanResponse {
        plan,
        groups: plan_groups,
        operator: op_val,
    })
}

pub async fn upsert_group(
    state: &AppState,
    user_id: Uuid,
    old_name: Option<&str>,
    name: &str,
) -> Result<PlanGroup, ApiError> {
    queries::upsert_group(&state.db, user_id, old_name, name)
        .await
        .map_err(Into::into)
}

pub async fn delete_group(state: &AppState, user_id: Uuid, name: &str) -> Result<(), ApiError> {
    queries::delete_group(&state.db, user_id, name)
        .await
        .map_err(Into::into)
}

pub async fn list_public_plans(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<OperatorPlanResponse>, ApiError> {
    let plans = queries::list_plans(&state.db, user_id).await?;
    let mapping = queries::get_all_plan_groups(&state.db, user_id).await?;
    let mut group_map: HashMap<Uuid, Vec<String>> = HashMap::new();
    for (plan_id, group_name) in mapping {
        group_map.entry(plan_id).or_default().push(group_name);
    }
    let mut responses = Vec::new();
    for plan in plans {
        if plan.display_on_profile
            && let Some((operator, server)) =
                resolve_operator(state, state.default_server, &plan.operator_id)
        {
            let plan_groups = group_map.remove(&plan.id).unwrap_or_default();
            let mut op_val =
                serde_json::to_value(&operator).map_err(|e| ApiError::Internal(e.into()))?;
            if let serde_json::Value::Object(map) = &mut op_val {
                map.insert(
                    "server".to_string(),
                    serde_json::Value::String(server.as_str().to_string()),
                );
            }
            responses.push(OperatorPlanResponse {
                plan,
                groups: plan_groups,
                operator: op_val,
            });
        }
    }
    Ok(responses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gamedata::types::building::{ManufactFormula, WorkshopFormula};

    fn formula_costs(inputs: &[(&str, i32)]) -> Vec<FormulaCost> {
        inputs
            .iter()
            .map(|(id, count)| FormulaCost {
                id: (*id).to_owned(),
                item_type: "MATERIAL".to_owned(),
                count: *count,
            })
            .collect()
    }

    fn workshop_formula(
        item_id: &str,
        output_count: i32,
        formula_type: &str,
        inputs: &[(&str, i32)],
    ) -> WorkshopFormula {
        WorkshopFormula {
            formula_id: format!("wf_{item_id}"),
            formula_type: formula_type.to_owned(),
            item_id: item_id.to_owned(),
            count: output_count,
            rarity: 0,
            gold_cost: 0,
            ap_cost: 0,
            costs: formula_costs(inputs),
            require_rooms: Vec::new(),
            require_stages: Vec::new(),
        }
    }

    fn manufact_formula(
        item_id: &str,
        output_count: i32,
        formula_type: &str,
        inputs: &[(&str, i32)],
    ) -> ManufactFormula {
        ManufactFormula {
            formula_id: format!("mf_{item_id}"),
            formula_type: formula_type.to_owned(),
            buff_type: String::new(),
            item_id: item_id.to_owned(),
            cost_point: 0,
            count: output_count,
            weight: 0,
            costs: formula_costs(inputs),
            require_rooms: Vec::new(),
            require_stages: Vec::new(),
        }
    }

    struct Fixture {
        gamedata: GameData,
        asset_index: AssetIndex,
        inventory: HashMap<String, i32>,
        building: UserBuilding,
        clears: HashMap<String, StageClear>,
    }

    impl Fixture {
        fn new(inventory: &[(&str, i32)]) -> Self {
            Self {
                gamedata: GameData::default(),
                asset_index: AssetIndex::default(),
                inventory: inventory
                    .iter()
                    .map(|(id, count)| ((*id).to_owned(), *count))
                    .collect(),
                building: UserBuilding { rooms: Vec::new() },
                clears: HashMap::new(),
            }
        }

        fn add_workshop(&mut self, formula: WorkshopFormula) {
            self.gamedata
                .building
                .workshop_formulas
                .insert(formula.formula_id.clone(), formula);
        }

        fn add_manufact(&mut self, formula: ManufactFormula) {
            self.gamedata
                .building
                .manufact_formulas
                .insert(formula.formula_id.clone(), formula);
        }

        fn requirements(&self, required: &[(&str, i32)]) -> Vec<PlanRequirementItem> {
            let ctx = PlannerCtx {
                gamedata: &self.gamedata,
                asset_index: &self.asset_index,
                inventory_map: &self.inventory,
                user_building: &self.building,
                clears: &self.clears,
            };
            let combined = required
                .iter()
                .map(|(id, count)| ((*id).to_owned(), *count))
                .collect();
            build_all_requirements(&ctx, combined)
        }
    }

    fn find<'a>(reqs: &'a [PlanRequirementItem], id: &str) -> &'a PlanRequirementItem {
        reqs.iter().find(|r| r.id == id).unwrap()
    }

    /// Regression for the Rephasic Enantiomer report: required 10, have 7, every
    /// ingredient either owned outright or craftable one tier down. The old code
    /// reserved ingredient inventory against the global requirement aggregate and
    /// reported Craftable 0 / Missing 3.
    #[test]
    fn parent_craftable_when_ingredients_owned_or_craftable() {
        let mut fx = Fixture::new(&[("t5", 7), ("t4", 5), ("t3", 30), ("t2a", 39), ("t2b", 4)]);
        fx.add_workshop(workshop_formula(
            "t5",
            1,
            "F_EVOLVE",
            &[("t4", 2), ("t2a", 1), ("t2b", 1)],
        ));
        fx.add_workshop(workshop_formula("t4", 1, "F_EVOLVE", &[("t3", 3)]));

        let reqs = fx.requirements(&[("t5", 10)]);
        let t5 = find(&reqs, "t5");
        assert!(t5.can_craft);
        assert_eq!(t5.craftable_count, 3);
        assert_eq!(t5.missing_count, 0);

        let recipe = t5.recipe.as_ref().unwrap();
        let t4 = &recipe
            .costs
            .iter()
            .find(|c| c.item.id == "t4")
            .unwrap()
            .item;
        assert_eq!(t4.required_count, 6);
        assert_eq!(t4.craftable_count, 1); // 5 owned + 1 crafted from t3
        assert_eq!(t4.missing_count, 0);
    }

    /// An ingredient's inventory reserved by its own top-level requirement can't
    /// be double-spent on crafting a parent.
    #[test]
    fn direct_requirements_reserve_inventory_before_crafting() {
        let mut fx = Fixture::new(&[("ing", 1)]);
        fx.add_workshop(workshop_formula("a", 1, "F_EVOLVE", &[("ing", 1)]));

        let reqs = fx.requirements(&[("a", 1), ("ing", 1)]);
        let ing = find(&reqs, "ing");
        assert_eq!(ing.missing_count, 0);
        let a = find(&reqs, "a");
        assert_eq!(a.craftable_count, 0);
        assert_eq!(a.missing_count, 1);
    }

    /// Two craftable requirements sharing an ingredient pool can't both claim the
    /// same unit.
    #[test]
    fn shared_ingredients_not_double_counted_across_requirements() {
        let mut fx = Fixture::new(&[("ing", 1)]);
        fx.add_workshop(workshop_formula("a", 1, "F_EVOLVE", &[("ing", 1)]));
        fx.add_workshop(workshop_formula("b", 1, "F_EVOLVE", &[("ing", 1)]));

        let reqs = fx.requirements(&[("a", 1), ("b", 1)]);
        let total_craftable: i32 = reqs.iter().map(|r| r.craftable_count).sum();
        let total_missing: i32 = reqs.iter().map(|r| r.missing_count).sum();
        assert_eq!(total_craftable, 1);
        assert_eq!(total_missing, 1);
    }

    /// Chip cross-class conversion (workshop `F_ASC`) must never be proposed as a
    /// craft path.
    #[test]
    fn chip_conversion_is_not_a_craft_path() {
        let mut fx = Fixture::new(&[("chip_b", 10)]);
        fx.add_workshop(workshop_formula("chip_a", 2, "F_ASC", &[("chip_b", 3)]));

        let reqs = fx.requirements(&[("chip_a", 2)]);
        let chip = find(&reqs, "chip_a");
        assert!(!chip.can_craft);
        assert_eq!(chip.craftable_count, 0);
        assert_eq!(chip.missing_count, 2);
        assert!(chip.recipe.is_none());
        assert!(chip.craft_reason.contains("Chip conversion"));
    }

    /// Dualchip factory recipes (manufact `F_ASC`) are the intended acquisition
    /// path and stay craftable.
    #[test]
    fn dualchip_factory_recipe_stays_craftable() {
        let mut fx = Fixture::new(&[("chip_pack", 2), ("catalyst", 1)]);
        fx.add_manufact(manufact_formula(
            "dualchip",
            1,
            "F_ASC",
            &[("chip_pack", 2), ("catalyst", 1)],
        ));

        let reqs = fx.requirements(&[("dualchip", 1)]);
        let dual = find(&reqs, "dualchip");
        assert!(dual.can_craft);
        assert_eq!(dual.craftable_count, 1);
        assert_eq!(dual.missing_count, 0);
    }

    /// A recipe locked behind an unbuilt room is still shown, but must not consume
    /// pool units that other requirements can still use.
    #[test]
    fn gated_recipe_does_not_consume_shared_pool() {
        let mut fx = Fixture::new(&[("ing", 1)]);
        let mut gated = workshop_formula("a", 1, "F_EVOLVE", &[("ing", 1)]);
        gated.require_rooms.push(FormulaRoomReq {
            room_id: "WORKSHOP".to_owned(),
            room_count: 1,
            room_level: 1,
        });
        fx.add_workshop(gated);
        fx.add_workshop(workshop_formula("b", 1, "F_EVOLVE", &[("ing", 1)]));

        let reqs = fx.requirements(&[("a", 1), ("b", 1)]);
        let a = find(&reqs, "a");
        assert!(!a.can_craft);
        assert_eq!(a.craftable_count, 0);
        assert!(a.craft_reason.starts_with("Requirements not met"));
        let b = find(&reqs, "b");
        assert_eq!(b.craftable_count, 1);
        assert_eq!(b.missing_count, 0);
    }
}
