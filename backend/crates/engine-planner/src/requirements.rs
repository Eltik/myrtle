use std::collections::{BTreeMap, HashMap};

use myrtle_engine_base::types::UserBuilding;
use myrtle_gamedata::{enrich::resolve_item_icon, model::GameModel};
use myrtle_gamedata_raw::tables::{
    building::{BuildingDataFile, FormulaCost, FormulaRoomReq},
    material::{ItemRarity, Materials},
    operator::Operator,
};
use myrtle_types::{roster::RosterEntry, stage_clear::StageClear};

use crate::{
    EXP_ITEM, LMD_ITEM, PlanError,
    progress::get_plan_direct_materials,
    types::{OperatorPlan, PlanRecipe, PlanRecipeCost, PlanRequirementItem, PlanUnmetStage},
};

pub const MIN_FLATTEN_TIER: i16 = 1;
pub const MAX_FLATTEN_TIER: i16 = 5;

const CHIP_CONVERSION_FORMULA_TYPE: &str = "F_ASC";
const CHIP_CRAFT_REASON: &str =
    "Chip conversion isn't recommended: obtain chips from stages, the store, or events";
const NO_RECIPE_REASON: &str = "No workshop or factory formula";
const RECIPE_CYCLE_REASON: &str = "Recipe cycle detected";

const MAX_FLATTEN_STEPS: usize = 10_000;

#[derive(Clone, Copy)]
pub struct PlannerCtx<'a> {
    pub gamedata: &'a GameModel,
    pub inventory_map: &'a HashMap<String, i32>,
    pub user_building: &'a UserBuilding,
    pub clears: &'a HashMap<String, StageClear>,
}

fn requirement_sort_key(id: &str, name: &str, rarity: i16) -> (i8, i8) {
    let name = name.to_lowercase();
    if id == EXP_ITEM {
        return (0, 0);
    }
    if id == LMD_ITEM {
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

fn item_display_meta(item_id: &str, materials: &Materials) -> (String, String, i16) {
    if item_id == LMD_ITEM {
        return ("LMD".to_owned(), "GOLD".to_owned(), 0);
    }
    if item_id == EXP_ITEM {
        return ("EXP".to_owned(), "EXP_PLAYER".to_owned(), 0);
    }
    if let Some(item) = materials.items.get(item_id) {
        return (
            item.name.clone(),
            format!("{:?}", item.item_type).to_uppercase(),
            rarity_tier(&item.rarity),
        );
    }
    (item_id.to_owned(), "MATERIAL".to_owned(), 0)
}

const fn rarity_tier(rarity: &ItemRarity) -> i16 {
    match rarity {
        ItemRarity::Tier1 => 1,
        ItemRarity::Tier2 => 2,
        ItemRarity::Tier3 => 3,
        ItemRarity::Tier4 => 4,
        ItemRarity::Tier5 => 5,
        ItemRarity::Tier6 => 6,
    }
}

fn item_tier(item_id: &str, materials: &Materials) -> i16 {
    if item_id == LMD_ITEM || item_id == EXP_ITEM {
        return 0;
    }
    materials
        .items
        .get(item_id)
        .map_or(0, |item| rarity_tier(&item.rarity))
}

fn claim_from_pool(pool: &mut HashMap<String, i32>, item_id: &str, count: i32) -> i32 {
    let entry = pool.entry(item_id.to_owned()).or_insert(0);
    let take = (*entry).min(count).max(0);
    *entry -= take;
    take
}

/// How many crafts of `output_count` units cover `shortfall`
const fn crafts_for(shortfall: i32, output_count: i32) -> i32 {
    (shortfall + output_count - 1) / output_count
}

struct RecipeView<'a> {
    output_count: i32,
    /// LMD per craft
    gold_cost: i64,
    costs: &'a [FormulaCost],
    require_rooms: &'a [FormulaRoomReq],
    /// Stage clears gating crafting a recipe
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
            gold_cost: formula.gold_cost,
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
            gold_cost: 0,
            costs: &formula.costs,
            require_rooms: &formula.require_rooms,
            require_stage_ids: Vec::new(),
        });
    }
    RecipeLookup::None
}

fn unmet_recipe_requirements(
    ctx: &PlannerCtx<'_>,
    view: &RecipeView<'_>,
) -> (Vec<String>, Vec<PlanUnmetStage>) {
    let mut unmet_reqs = Vec::new();
    let mut unmet_stages = Vec::new();
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
            .is_some_and(|c| c.is_cleared() || c.complete_times > 0);
        if !is_cleared {
            let stage_code = ctx
                .gamedata
                .stages
                .get(*stage_id)
                .map_or(*stage_id, |s| s.code.as_str());
            unmet_reqs.push(format!("Stage {stage_code}"));
            unmet_stages.push(PlanUnmetStage {
                stage_id: (*stage_id).to_owned(),
                code: stage_code.to_owned(),
            });
        }
    }
    (unmet_reqs, unmet_stages)
}

/// What crafting adds to one requirement row
struct CraftOutcome {
    craftable_count: i32,
    can_craft: bool,
    craft_reason: String,
    craft_blocked: bool,
    recipe: Option<PlanRecipe>,
    unmet_stages: Vec<PlanUnmetStage>,
}

impl CraftOutcome {
    fn uncraftable(reason: &str) -> Self {
        Self {
            craftable_count: 0,
            can_craft: false,
            craft_reason: reason.to_owned(),
            craft_blocked: false,
            recipe: None,
            unmet_stages: Vec::new(),
        }
    }
}

/// Builds the requirement node for one item
fn build_requirement_tree(
    ctx: &PlannerCtx<'_>,
    item_id: &str,
    required_count: i32,
    reserved_from_pool: Option<i32>,
    pool: &mut HashMap<String, i32>,
    visited: &[&str],
) -> PlanRequirementItem {
    let inventory_count = *ctx.inventory_map.get(item_id).unwrap_or(&0);

    let satisfied_from_inv =
        reserved_from_pool.unwrap_or_else(|| claim_from_pool(pool, item_id, required_count));
    let shortfall = (required_count - satisfied_from_inv).max(0);

    let craft = match find_recipe(&ctx.gamedata.building, item_id) {
        RecipeLookup::Found(view) => craft_shortfall(ctx, item_id, &view, shortfall, pool, visited),
        RecipeLookup::ChipConversion => CraftOutcome::uncraftable(CHIP_CRAFT_REASON),
        RecipeLookup::None => CraftOutcome::uncraftable(NO_RECIPE_REASON),
    };

    let (icon_id, image) =
        resolve_item_icon(item_id, &ctx.gamedata.materials, &ctx.gamedata.assets);
    let (name, item_type, rarity) = item_display_meta(item_id, &ctx.gamedata.materials);
    let (sort_group, sort_subrank) = requirement_sort_key(item_id, &name, rarity);

    PlanRequirementItem {
        id: item_id.to_owned(),
        required_count,
        inventory_count,
        missing_count: (shortfall - craft.craftable_count).max(0),
        craftable_count: craft.craftable_count,
        name,
        icon_id,
        image,
        item_type,
        rarity,
        sort_group,
        sort_subrank,
        can_craft: craft.can_craft,
        craft_reason: craft.craft_reason,
        craft_blocked: craft.craft_blocked,
        recipe: craft.recipe,
        unmet_stages: craft.unmet_stages,
    }
}

/// Expands `view` to cover `shortfall`
fn craft_shortfall(
    ctx: &PlannerCtx<'_>,
    item_id: &str,
    view: &RecipeView<'_>,
    shortfall: i32,
    pool: &mut HashMap<String, i32>,
    visited: &[&str],
) -> CraftOutcome {
    if view.costs.iter().any(|c| visited.contains(&c.id.as_str())) {
        return CraftOutcome::uncraftable(RECIPE_CYCLE_REASON);
    }

    let (unmet_reqs, unmet_stages) = unmet_recipe_requirements(ctx, view);
    let gates_met = unmet_reqs.is_empty();

    let crafts_needed = if shortfall > 0 {
        crafts_for(shortfall, view.output_count)
    } else {
        0
    };

    // A gated recipe is still shown, but nothing will actually be crafted, so
    // expand it against a scratch copy
    let mut scratch;
    let child_pool: &mut HashMap<String, i32> = if gates_met {
        pool
    } else {
        scratch = pool.clone();
        &mut scratch
    };

    let mut next_visited = visited.to_vec();
    next_visited.push(item_id);

    let costs: Vec<PlanRecipeCost> = view
        .costs
        .iter()
        .map(|cost| PlanRecipeCost {
            count: cost.count,
            item: build_requirement_tree(
                ctx,
                &cost.id,
                crafts_needed * cost.count,
                None,
                child_pool,
                &next_visited,
            ),
        })
        .collect();

    let mut craftable_count = 0;
    let craft_reason = if gates_met {
        if crafts_needed > 0 {
            let achievable = costs
                .iter()
                .filter(|c| c.count > 0)
                .fold(crafts_needed, |acc, c| {
                    let satisfied = c.item.required_count - c.item.missing_count;
                    acc.min(satisfied / c.count)
                });
            craftable_count = achievable * view.output_count;
        }
        String::new()
    } else {
        format!("Requirements not met: {}", unmet_reqs.join(", "))
    };

    CraftOutcome {
        craftable_count,
        can_craft: gates_met,
        craft_reason,
        craft_blocked: !gates_met,
        recipe: Some(PlanRecipe {
            count: view.output_count,
            costs,
        }),
        unmet_stages,
    }
}

#[must_use]
pub fn build_all_requirements(
    ctx: &PlannerCtx<'_>,
    combined_materials: HashMap<String, i32>,
) -> Vec<PlanRequirementItem> {
    let mut ordered: Vec<(String, i32)> = combined_materials.into_iter().collect();
    ordered.sort_by_cached_key(|(item_id, _)| {
        let (name, _, rarity) = item_display_meta(item_id, &ctx.gamedata.materials);
        let (sort_group, sort_subrank) = requirement_sort_key(item_id, &name, rarity);
        (sort_group, sort_subrank, name.to_lowercase())
    });

    let mut pool = ctx.inventory_map.clone();
    let reserved: HashMap<String, i32> = ordered
        .iter()
        .map(|(item_id, count)| (item_id.clone(), claim_from_pool(&mut pool, item_id, *count)))
        .collect();

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

fn flatten_above_tier(
    ctx: &PlannerCtx<'_>,
    mut need: HashMap<String, i32>,
    max_tier: i16,
) -> HashMap<String, i32> {
    let materials = &ctx.gamedata.materials;
    let building = &ctx.gamedata.building;
    let mut pool = ctx.inventory_map.clone();

    for _ in 0..MAX_FLATTEN_STEPS {
        let next = need
            .iter()
            .filter(|&(id, &count)| {
                count > 0
                    && item_tier(id, materials) > max_tier
                    && matches!(find_recipe(building, id), RecipeLookup::Found(_))
            })
            .map(|(id, _)| (item_tier(id, materials), id.clone()))
            .max_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));
        let Some((_, item_id)) = next else {
            break;
        };

        let count = need.remove(&item_id).unwrap_or(0);
        let shortfall = count - claim_from_pool(&mut pool, &item_id, count);
        if shortfall <= 0 {
            continue;
        }
        let RecipeLookup::Found(view) = find_recipe(building, &item_id) else {
            continue;
        };
        let crafts = crafts_for(shortfall, view.output_count);
        for cost in view.costs {
            *need.entry(cost.id.clone()).or_insert(0) += crafts * cost.count;
        }
        if view.gold_cost > 0 {
            let lmd = i32::try_from(i64::from(crafts) * view.gold_cost).unwrap_or(i32::MAX);
            let entry = need.entry(LMD_ITEM.to_owned()).or_insert(0);
            *entry = entry.saturating_add(lmd);
        }
    }

    need
}

#[must_use]
pub fn aggregate_requirements(
    ctx: &PlannerCtx<'_>,
    combined_materials: HashMap<String, i32>,
    max_tier: Option<i16>,
) -> Vec<PlanRequirementItem> {
    let combined_materials = match max_tier {
        Some(tier) => flatten_above_tier(ctx, combined_materials, tier),
        None => combined_materials,
    };
    build_all_requirements(ctx, combined_materials)
}

/// Whether the plan for `operator_id` counts toward the requirements
fn is_counted(active_ids: &[String], operator_id: &String) -> bool {
    active_ids.is_empty() || active_ids.contains(operator_id)
}

pub fn calculate_requirements(
    ctx: &PlannerCtx<'_>,
    plans_with_ops: &[(OperatorPlan, Operator, Option<&RosterEntry>)],
    active_ids: &[String],
    max_tier: Option<i16>,
) -> Result<Vec<PlanRequirementItem>, PlanError> {
    let mut combined_materials = HashMap::new();
    for (plan, operator, roster_entry) in plans_with_ops {
        if !is_counted(active_ids, &plan.operator_id) {
            continue;
        }
        let plan_materials =
            get_plan_direct_materials(ctx.gamedata, plan, operator, *roster_entry)?;
        for (item_id, count) in plan_materials {
            *combined_materials.entry(item_id).or_insert(0) += count;
        }
    }

    Ok(aggregate_requirements(ctx, combined_materials, max_tier))
}

pub fn requirements_by_operator(
    ctx: &PlannerCtx<'_>,
    plans_with_ops: &[(OperatorPlan, Operator, Option<&RosterEntry>)],
    active_ids: &[String],
    max_tier: Option<i16>,
) -> Result<BTreeMap<String, Vec<PlanRequirementItem>>, PlanError> {
    let mut by_id = BTreeMap::new();
    for (plan, _, _) in plans_with_ops {
        if !is_counted(active_ids, &plan.operator_id) {
            continue;
        }
        let own = std::slice::from_ref(&plan.operator_id);
        by_id.insert(
            plan.operator_id.clone(),
            calculate_requirements(ctx, plans_with_ops, own, max_tier)?,
        );
    }
    Ok(by_id)
}
