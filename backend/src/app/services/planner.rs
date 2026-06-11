use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    app::{error::ApiError, services::operators::resolve_operator, state::AppState},
    core::{
        gamedata::{
            enrich::resolve_item_icon,
            types::{
                material::ItemRarity,
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

fn phase_to_int(phase: &OperatorPhase) -> i16 {
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

#[allow(clippy::too_many_arguments)]
fn resolve_requirement_item(
    item_id: &str,
    required_count: i32,
    inventory_count: i32,
    craftable_count: i32,
    can_craft: bool,
    craft_reason: String,
    recipe: Option<PlanRecipe>,
    state: &AppState,
) -> PlanRequirementItem {
    let gamedata = state.default_game_data();
    let materials = &gamedata.materials;
    let asset_index = state.default_asset_index();
    let (icon_id, image) = resolve_item_icon(item_id, materials, &asset_index);

    let mut name = item_id.to_owned();
    let mut item_type = "MATERIAL".to_owned();
    let mut rarity: i16 = 0;

    if item_id == "4001" {
        name = "LMD".to_owned();
        item_type = "GOLD".to_owned();
    } else if item_id == "5001" {
        name = "EXP".to_owned();
        item_type = "EXP_PLAYER".to_owned();
    } else if let Some(item) = materials.items.get(item_id) {
        name = item.name.clone();
        item_type = format!("{:?}", item.item_type).to_uppercase();
        rarity = match item.rarity {
            ItemRarity::Tier1 => 1,
            ItemRarity::Tier2 => 2,
            ItemRarity::Tier3 => 3,
            ItemRarity::Tier4 => 4,
            ItemRarity::Tier5 => 5,
            ItemRarity::Tier6 => 6,
        };
    }

    let missing_count = (required_count - inventory_count - craftable_count).max(0);
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
    state: &AppState,
    current_elite: i16,
    current_level: i16,
    target_elite: i16,
    target_level: i16,
    materials: &mut HashMap<String, i32>,
) {
    let gamedata = state.default_game_data();
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

#[allow(clippy::too_many_arguments)]
fn build_requirement_tree(
    item_id: &str,
    required_count: i32,
    materials: &HashMap<String, i32>,
    inventory_map: &HashMap<String, i32>,
    user_building: &UserBuilding,
    clears: &HashMap<String, StageClear>,
    state: &AppState,
    visited: &[&str],
) -> PlanRequirementItem {
    let inv_count = *inventory_map.get(item_id).unwrap_or(&0);
    let deficit = (required_count - inv_count).max(0);

    let gamedata = state.default_game_data();

    let mut has_cycle = false;
    if let Some(formula) = gamedata
        .building
        .workshop_formulas
        .values()
        .find(|f| f.item_id == item_id)
    {
        has_cycle = formula
            .costs
            .iter()
            .any(|c| visited.contains(&c.id.as_str()));
    } else if let Some(formula) = gamedata
        .building
        .manufact_formulas
        .values()
        .find(|f| f.item_id == item_id)
    {
        has_cycle = formula
            .costs
            .iter()
            .any(|c| visited.contains(&c.id.as_str()));
    }

    if has_cycle {
        return resolve_requirement_item(
            item_id,
            required_count,
            inv_count,
            0,
            false,
            "Recipe cycle detected".to_owned(),
            None,
            state,
        );
    }

    let mut recipe = None;
    let mut craftable_count = 0;
    let mut can_craft = false;
    let mut craft_reason = "No workshop or factory formula".to_owned();

    if let Some(formula) = gamedata
        .building
        .workshop_formulas
        .values()
        .find(|f| f.item_id == item_id)
    {
        let mut unmet_reqs = Vec::new();
        for room_req in &formula.require_rooms {
            let matching_count = user_building
                .rooms
                .iter()
                .filter(|r| r.room_type == room_req.room_id && r.level >= room_req.room_level)
                .count();
            if matching_count < room_req.room_count as usize {
                unmet_reqs.push(format!("{} lv.{}", room_req.room_id, room_req.room_level));
            }
        }
        for stage_req in &formula.require_stages {
            let is_cleared = clears
                .get(&stage_req.stage_id)
                .is_some_and(|c| c.state >= 2 || c.complete_times > 0);
            if !is_cleared {
                let stage_code = gamedata
                    .stages
                    .get(&stage_req.stage_id)
                    .map(|s| s.code.as_str())
                    .unwrap_or(stage_req.stage_id.as_str());
                unmet_reqs.push(format!("Stage {}", stage_code));
            }
        }

        let mut costs = Vec::new();
        let num_crafts_needed = if deficit > 0 {
            (deficit + formula.count - 1) / formula.count
        } else {
            0
        };

        for cost in &formula.costs {
            let cost_required = num_crafts_needed * cost.count;
            let mut next_visited = visited.to_vec();
            next_visited.push(item_id);
            let cost_item = build_requirement_tree(
                &cost.id,
                cost_required,
                materials,
                inventory_map,
                user_building,
                clears,
                state,
                &next_visited,
            );
            costs.push(PlanRecipeCost {
                count: cost.count,
                item: cost_item,
            });
        }

        if unmet_reqs.is_empty() {
            can_craft = true;
            craft_reason = "".to_owned();

            let mut max_crafts = i32::MAX;
            for cost in &formula.costs {
                let ingredient_inv = *inventory_map.get(&cost.id).unwrap_or(&0);
                let ingredient_needed = materials.get(&cost.id).copied().unwrap_or(0);
                let ingredient_avail_inv = (ingredient_inv - ingredient_needed).max(0);

                let child_cost_item = costs.iter().find(|c| c.item.id == cost.id).unwrap();
                let ingredient_avail_craft = child_cost_item.item.craftable_count;

                let ingredient_avail = ingredient_avail_inv + ingredient_avail_craft;
                let craft_limit = ingredient_avail / cost.count;
                max_crafts = max_crafts.min(craft_limit);
            }

            if max_crafts != i32::MAX && max_crafts > 0 {
                craftable_count = max_crafts * formula.count;
            }
        } else {
            can_craft = false;
            craft_reason = format!("Requirements not met: {}", unmet_reqs.join(", "));
        }

        recipe = Some(PlanRecipe {
            count: formula.count,
            costs,
        });
    } else if let Some(formula) = gamedata
        .building
        .manufact_formulas
        .values()
        .find(|f| f.item_id == item_id)
    {
        let mut unmet_reqs = Vec::new();
        for room_req in &formula.require_rooms {
            let matching_count = user_building
                .rooms
                .iter()
                .filter(|r| r.room_type == room_req.room_id && r.level >= room_req.room_level)
                .count();
            if matching_count < room_req.room_count as usize {
                unmet_reqs.push(format!("{} lv.{}", room_req.room_id, room_req.room_level));
            }
        }

        let mut costs = Vec::new();
        let num_crafts_needed = if deficit > 0 {
            (deficit + formula.count - 1) / formula.count
        } else {
            0
        };

        for cost in &formula.costs {
            let cost_required = num_crafts_needed * cost.count;
            let mut next_visited = visited.to_vec();
            next_visited.push(item_id);
            let cost_item = build_requirement_tree(
                &cost.id,
                cost_required,
                materials,
                inventory_map,
                user_building,
                clears,
                state,
                &next_visited,
            );
            costs.push(PlanRecipeCost {
                count: cost.count,
                item: cost_item,
            });
        }

        if unmet_reqs.is_empty() {
            can_craft = true;
            craft_reason = "".to_owned();

            let mut max_crafts = i32::MAX;
            for cost in &formula.costs {
                let ingredient_inv = *inventory_map.get(&cost.id).unwrap_or(&0);
                let ingredient_needed = materials.get(&cost.id).copied().unwrap_or(0);
                let ingredient_avail_inv = (ingredient_inv - ingredient_needed).max(0);

                let child_cost_item = costs.iter().find(|c| c.item.id == cost.id).unwrap();
                let ingredient_avail_craft = child_cost_item.item.craftable_count;

                let ingredient_avail = ingredient_avail_inv + ingredient_avail_craft;
                let craft_limit = ingredient_avail / cost.count;
                max_crafts = max_crafts.min(craft_limit);
            }

            if max_crafts != i32::MAX && max_crafts > 0 {
                craftable_count = max_crafts * formula.count;
            }
        } else {
            can_craft = false;
            craft_reason = format!("Requirements not met: {}", unmet_reqs.join(", "));
        }

        recipe = Some(PlanRecipe {
            count: formula.count,
            costs,
        });
    }

    resolve_requirement_item(
        item_id,
        required_count,
        inv_count,
        craftable_count,
        can_craft,
        craft_reason,
        recipe,
        state,
    )
}

fn get_plan_direct_materials(
    state: &AppState,
    plan: &OperatorPlan,
    operator: &Operator,
    roster_entry: Option<&RosterEntry>,
) -> Result<HashMap<String, i32>, ApiError> {
    let current_elite = roster_entry.map(|r| r.elite).unwrap_or(0);
    let current_level = roster_entry.map(|r| r.level).unwrap_or(1);
    let current_skill_level = roster_entry.map(|r| r.skill_level).unwrap_or(1);

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
        state,
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
            .map(|m| m.mastery)
            .unwrap_or(0);

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
            .map(|m| m.level)
            .unwrap_or(0);
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

fn calculate_requirements(
    state: &AppState,
    plans_with_ops: &[(OperatorPlan, Operator, Option<&RosterEntry>)],
    active_ids: &[String],
    inventory_map: &HashMap<String, i32>,
    user_building: &UserBuilding,
    clears: &HashMap<String, StageClear>,
) -> Result<Vec<PlanRequirementItem>, ApiError> {
    let mut combined_materials = HashMap::new();
    for (plan, operator, roster_entry) in plans_with_ops {
        let is_active = active_ids.is_empty() || active_ids.contains(&plan.operator_id);
        if !is_active {
            continue;
        }
        let plan_materials = get_plan_direct_materials(state, plan, operator, *roster_entry)?;
        for (item_id, count) in plan_materials {
            *combined_materials.entry(item_id).or_insert(0) += count;
        }
    }

    let mut requirements: Vec<PlanRequirementItem> = Vec::with_capacity(combined_materials.len());

    for (item_id, &count) in &combined_materials {
        let req_item = build_requirement_tree(
            item_id,
            count,
            &combined_materials,
            inventory_map,
            user_building,
            clears,
            state,
            &[],
        );
        requirements.push(req_item);
    }

    requirements.sort_by_key(|r| (r.sort_group, r.sort_subrank, r.name.to_lowercase()));

    Ok(requirements)
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
    let user_building = building_json
        .map(|json| UserBuilding::from_json(&json))
        .unwrap_or_else(|| UserBuilding { rooms: Vec::new() });

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
            let mut op_val = serde_json::to_value(&operator).unwrap();
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

    let aggregated_requirements = calculate_requirements(
        state,
        &plans_with_ops,
        &active_ids,
        &inventory_map,
        &user_building,
        &clears,
    )?;

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
                        "Target skill level {} requires Elite {} or higher",
                        idx, required_phase
                    )));
                }
                let required_level = lvl_up.unlock_cond.level as i16;
                if required_phase == target_elite && required_level > target_level {
                    return Err(ApiError::BadRequest(format!(
                        "Target skill level {} requires Level {} at Elite {}",
                        idx, required_level, required_phase
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
                    "Module unlock/upgrades require Elite {} level {}.",
                    required_phase, required_level
                )));
            }
        }
    }

    let mut tx = state.db.begin().await?;

    let plan = sqlx::query_as::<_, OperatorPlan>(
        r#"
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
        "#,
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
                r#"
                INSERT INTO plan_groups (user_id, name)
                VALUES ($1, $2)
                ON CONFLICT (user_id, name) DO UPDATE SET name = EXCLUDED.name
                RETURNING id
                "#,
            )
            .bind(user_id)
            .bind(group_name)
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query(
                r#"
                INSERT INTO plan_group_members (plan_group_id, operator_plan_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
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
            r#"
            SELECT pg.name
            FROM plan_groups pg
            JOIN plan_group_members pgm ON pgm.plan_group_id = pg.id
            WHERE pgm.operator_plan_id = $1
            ORDER BY pg.name ASC
            "#,
        )
        .bind(plan.id)
        .fetch_all(&state.db)
        .await?
    };

    let mut op_val = serde_json::to_value(&operator).unwrap();
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
            let mut op_val = serde_json::to_value(&operator).unwrap();
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
