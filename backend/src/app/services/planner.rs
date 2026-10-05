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
                DeletePlansResponse, OperatorPlan, OperatorPlanResponse, PlanGroup, PlanPreset,
                PlanRecipe, PlanRecipeCost, PlanRequirementItem, PlanUnmetStage, PlannerResponse,
                PresetTarget, TargetModulePlan, TargetSkillPlan,
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
    /// The save keeps a module the player has not unlocked at `level` 1 with
    /// this flag set (every locked row in the database carries level 1 or
    /// more), so `level` alone reads a locked module as stage 1.
    #[serde(default)]
    locked: bool,
}

/// The roster state a plan is measured from. The material diff
/// (`get_plan_direct_materials`) and the completion check (`plan_met`) both
/// read it, and every cost the diff adds sits behind the negation of one of
/// the `reaches_*` checks `plan_met` requires. A met plan therefore costs
/// nothing by construction; the two cannot disagree.
struct CurrentState {
    owned: bool,
    elite: i16,
    level: i16,
    skill_level: i16,
    masteries: Vec<RosterMastery>,
    modules: Vec<RosterModule>,
}

impl CurrentState {
    /// An operator the player does not own reads as Elite 0 level 1, skill
    /// level 1, nothing mastered or unlocked.
    fn from_roster(entry: Option<&RosterEntry>) -> Self {
        Self {
            owned: entry.is_some(),
            elite: entry.map_or(0, |r| r.elite),
            level: entry.map_or(1, |r| r.level),
            skill_level: entry.map_or(1, |r| r.skill_level),
            masteries: entry
                .map(|r| serde_json::from_value(r.masteries.clone()).unwrap_or_default())
                .unwrap_or_default(),
            modules: entry
                .map(|r| serde_json::from_value(r.modules.clone()).unwrap_or_default())
                .unwrap_or_default(),
        }
    }

    fn mastery(&self, skill_index: i16) -> i16 {
        self.masteries
            .iter()
            .find(|m| m.index == skill_index)
            .map_or(0, |m| m.mastery)
    }

    /// A locked module is stage 0, whatever level the save carries for it.
    fn module_stage(&self, module_id: &str) -> i16 {
        self.modules
            .iter()
            .find(|m| m.id == module_id)
            .map_or(0, |m| if m.locked { 0 } else { m.level })
    }

    const fn reaches_level(&self, target_elite: i16, target_level: i16) -> bool {
        self.elite > target_elite || (self.elite == target_elite && self.level >= target_level)
    }

    const fn reaches_skill_level(&self, target: i16) -> bool {
        self.skill_level >= target
    }

    fn reaches_mastery(&self, target: &TargetSkillPlan) -> bool {
        self.mastery(target.skill_index) >= target.mastery_level
    }

    fn reaches_module(&self, target: &TargetModulePlan) -> bool {
        self.module_stage(&target.module_id) >= target.module_stage
    }
}

/// A plan's mastery and module targets, parsed from their jsonb columns.
struct PlanTargets {
    skills: Vec<TargetSkillPlan>,
    modules: Vec<TargetModulePlan>,
}

impl PlanTargets {
    fn parse(plan: &OperatorPlan) -> Result<Self, ApiError> {
        Ok(Self {
            skills: serde_json::from_value(plan.target_skills.clone())
                .map_err(|_| ApiError::BadRequest("Invalid target_skills format".into()))?,
            modules: serde_json::from_value(plan.target_modules.clone())
                .map_err(|_| ApiError::BadRequest("Invalid target_modules format".into()))?,
        })
    }
}

/// Whether the plan has nothing left to do: the operator is owned and every
/// target is reached or passed. A target mastery or module stage of 0 is
/// always reached.
fn plan_met(plan: &OperatorPlan, targets: &PlanTargets, current: &CurrentState) -> bool {
    current.owned
        && current.reaches_level(plan.target_elite, plan.target_level)
        && current.reaches_skill_level(plan.target_skill_level)
        && targets.skills.iter().all(|t| current.reaches_mastery(t))
        && targets.modules.iter().all(|t| current.reaches_module(t))
}

/// `plan_met` for a stored plan, reading its own roster entry. A plan whose
/// target columns do not parse is not met.
fn stored_plan_met(plan: &OperatorPlan, roster_entry: Option<&RosterEntry>) -> bool {
    PlanTargets::parse(plan)
        .is_ok_and(|targets| plan_met(plan, &targets, &CurrentState::from_roster(roster_entry)))
}

const fn phase_to_int(phase: &OperatorPhase) -> i16 {
    match phase {
        OperatorPhase::Elite0 => 0,
        OperatorPhase::Elite1 => 1,
        OperatorPhase::Elite2 => 2,
    }
}

pub(crate) fn module_phase_to_int(phase: &str) -> i16 {
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
/// and stay craftable: combining chip packs with a catalyst *is* the intended
/// way to obtain dualchips.
const CHIP_CONVERSION_FORMULA_TYPE: &str = "F_ASC";
const CHIP_CRAFT_REASON: &str =
    "Chip conversion isn't recommended: obtain chips from stages, the store, or events";

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

/// An item's tier (`TIER_1`..`TIER_6` as 1..6) from the item table. LMD and
/// EXP are tier 0, as in `item_display_meta` (the table files them at
/// `TIER_4` and `TIER_5`), and so is anything the table lacks.
fn item_tier(item_id: &str, materials: &Materials) -> i16 {
    if item_id == "4001" || item_id == "5001" {
        return 0;
    }
    materials
        .items
        .get(item_id)
        .map_or(0, |item| rarity_tier(&item.rarity))
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
        unmet_stages: Vec::new(),
    }
}

pub(crate) fn calculate_leveling_costs(
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

fn claim_from_pool(pool: &mut HashMap<String, i32>, item_id: &str, count: i32) -> i32 {
    let entry = pool.entry(item_id.to_owned()).or_insert(0);
    let take = (*entry).min(count).max(0);
    *entry -= take;
    take
}

/// A craft recipe normalized across workshop and factory formulas.
struct RecipeView<'a> {
    output_count: i32,
    /// LMD per craft (workshop formulas only; the factory charges none).
    gold_cost: i64,
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

/// The recipe's unmet gates as display strings (`WORKSHOP lv.3`, `Stage S9-3`),
/// and its unmet stage gates again as structured entries.
fn unmet_recipe_requirements(
    ctx: &PlannerCtx,
    view: &RecipeView,
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
            .is_some_and(|c| c.state >= 2 || c.complete_times > 0);
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

/// Builds the requirement node for one item, allocating units out of `pool`, the
/// shared ledger of not-yet-claimed inventory, so the same unit is never counted
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
    let mut unmet_stages = Vec::new();

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

        let (unmet_reqs, stages) = unmet_recipe_requirements(ctx, &view);
        unmet_stages = stages;
        let gates_met = unmet_reqs.is_empty();

        let crafts_needed = if shortfall > 0 {
            (shortfall + view.output_count - 1) / view.output_count
        } else {
            0
        };

        // A gated recipe is still shown, but nothing will actually be crafted, so
        // expand it against a scratch copy: it must not consume pool units that
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

    let mut item = resolve_requirement_item(
        ctx,
        item_id,
        required_count,
        inv_count,
        craftable_count,
        missing_count,
        can_craft,
        craft_reason,
        recipe,
    );
    item.unmet_stages = unmet_stages;
    item
}

/// Materials from the current state to the plan's targets. Every cost is
/// added behind the negation of a `CurrentState::reaches_*` check, the same
/// checks `plan_met` requires, so a met plan yields an empty map.
fn get_plan_direct_materials(
    gamedata: &GameData,
    plan: &OperatorPlan,
    operator: &Operator,
    roster_entry: Option<&RosterEntry>,
) -> Result<HashMap<String, i32>, ApiError> {
    let current = CurrentState::from_roster(roster_entry);
    let targets = PlanTargets::parse(plan)?;
    let current_elite = current.elite;
    let current_skill_level = current.skill_level;

    let mut materials = HashMap::new();

    if !current.reaches_level(plan.target_elite, plan.target_level) {
        calculate_leveling_costs(
            operator,
            gamedata,
            current_elite,
            current.level,
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
    }

    if !current.reaches_skill_level(plan.target_skill_level) {
        for i in (current_skill_level - 1)..(plan.target_skill_level - 1) {
            if let Some(lvl_up) = operator.all_skill_level_up.get(i as usize) {
                for cost in &lvl_up.lvl_up_cost {
                    *materials.entry(cost.id.clone()).or_insert(0) += cost.count;
                }
            }
        }
    }

    for target_skill in &targets.skills {
        let idx = target_skill.skill_index;
        let current_mast = current.mastery(idx);

        if !current.reaches_mastery(target_skill)
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

    for target_module in &targets.modules {
        let current_level = current.module_stage(&target_module.module_id);
        if !current.reaches_module(target_module)
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

/// Upper bound on recipe expansions in `flatten_above_tier`. Game recipes run
/// strictly down in tier, so a real plan stops after a few dozen; the bound
/// only keeps a cyclic recipe table from spinning.
const MAX_FLATTEN_STEPS: usize = 10_000;

/// Rewrites aggregated plan materials so nothing above `max_tier` is left
/// that a recipe can replace.
///
/// Each above-tier item, highest tier first, first claims owned copies from
/// its own ledger (a copy of the inventory that only above-tier items draw
/// on); the remaining shortfall becomes `ceil(shortfall / output)` crafts,
/// whose ingredients and workshop LMD are added to the matching entries. The
/// item itself leaves the map, owned copies included, since nothing above the
/// tier is shown. An ingredient that is itself above the tier is expanded in
/// turn, and an item that gains need after its expansion is expanded again
/// against what is left in the ledger, so no owned copy is counted twice.
///
/// Recipes are expanded whatever their room and stage gates: the filter
/// answers "what do I farm", and a gate is a step on the way, not a reason to
/// farm the higher tier. Items with no recipe (or chips, whose conversion is
/// never proposed) stay as they are, above the tier or not.
fn flatten_above_tier(
    ctx: &PlannerCtx,
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
            // Highest tier first, ties by id, so the result never depends on
            // the map's iteration order.
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
        let crafts = (shortfall + view.output_count - 1) / view.output_count;
        for cost in view.costs {
            *need.entry(cost.id.clone()).or_insert(0) += crafts * cost.count;
        }
        if view.gold_cost > 0 {
            let lmd = i32::try_from(i64::from(crafts) * view.gold_cost).unwrap_or(i32::MAX);
            let entry = need.entry("4001".to_owned()).or_insert(0);
            *entry = entry.saturating_add(lmd);
        }
    }

    need
}

/// The requirement rows for aggregated plan materials, optionally flattened
/// to `max_tier` first. With no tier the rows are exactly what
/// `build_all_requirements` makes of the materials.
fn aggregate_requirements(
    ctx: &PlannerCtx,
    combined_materials: HashMap<String, i32>,
    max_tier: Option<i16>,
) -> Vec<PlanRequirementItem> {
    let combined_materials = match max_tier {
        Some(tier) => flatten_above_tier(ctx, combined_materials, tier),
        None => combined_materials,
    };
    build_all_requirements(ctx, combined_materials)
}

fn calculate_requirements(
    ctx: &PlannerCtx,
    plans_with_ops: &[(OperatorPlan, Operator, Option<&RosterEntry>)],
    active_ids: &[String],
    max_tier: Option<i16>,
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

    Ok(aggregate_requirements(ctx, combined_materials, max_tier))
}

pub async fn list_plans(
    state: &AppState,
    user_id: Uuid,
    active_ids: Vec<String>,
    max_tier: Option<i16>,
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
            let met = stored_plan_met(&plan, roster_entry);
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
                met,
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
    let aggregated_requirements =
        calculate_requirements(&ctx, &plans_with_ops, &active_ids, max_tier)?;

    Ok(PlannerResponse {
        plans: responses,
        aggregated_requirements,
        groups,
        last_synced_at: profile.as_ref().map(|p| p.updated_at),
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

/// Most operator ids one bulk delete takes. Above the whole operator roster,
/// so a "delete every plan" request always fits.
const MAX_BULK_DELETE: usize = 1000;

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

        // A fixed lock order keeps concurrent upserts sharing groups from deadlocking.
        let mut group_names: Vec<&String> = group_names.iter().collect();
        group_names.sort();
        group_names.dedup();
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

    let roster_entry = roster_queries::get_operator(&state.db, user_id, operator_id).await?;
    let met = stored_plan_met(&plan, roster_entry.as_ref());

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
        met,
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
    let roster_map: HashMap<String, RosterEntry> = roster_queries::get_roster(&state.db, user_id)
        .await?
        .into_iter()
        .map(|r| (r.operator_id.clone(), r))
        .collect();
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
            let met = stored_plan_met(&plan, roster_map.get(&plan.operator_id));
            responses.push(OperatorPlanResponse {
                plan,
                groups: plan_groups,
                operator: op_val,
                met,
            });
        }
    }
    Ok(responses)
}

/// Checks a preset's ranges. The bounds are the ones every operator shares:
/// level caps of 50 / 80 / 90 at Elite 0 / 1 / 2, skill levels past 4 from
/// Elite 1, masteries at Elite 2 and skill level 7, modules at Elite 2. A
/// bulk-add clamps each operator further to its own rarity.
fn validate_preset_target(target: &PresetTarget) -> Result<(), ApiError> {
    const LEVEL_CAPS: [i16; 3] = [50, 80, 90];
    if !(0..=2).contains(&target.elite) {
        return Err(ApiError::BadRequest("elite must be between 0 and 2".into()));
    }
    let cap = LEVEL_CAPS[target.elite as usize];
    if let Some(level) = target.level
        && !(1..=cap).contains(&level)
    {
        return Err(ApiError::BadRequest(format!(
            "level must be between 1 and {cap} at Elite {}, or null for the cap",
            target.elite
        )));
    }
    if !(1..=7).contains(&target.skill_level) {
        return Err(ApiError::BadRequest(
            "skill_level must be between 1 and 7".into(),
        ));
    }
    if target.skill_level > 4 && target.elite < 1 {
        return Err(ApiError::BadRequest(
            "skill_level above 4 requires Elite 1".into(),
        ));
    }
    if target.masteries.iter().any(|m| !(0..=3).contains(m)) {
        return Err(ApiError::BadRequest(
            "each mastery must be between 0 and 3".into(),
        ));
    }
    if target.masteries.iter().any(|&m| m > 0) && (target.elite < 2 || target.skill_level < 7) {
        return Err(ApiError::BadRequest(
            "masteries require Elite 2 and skill_level 7".into(),
        ));
    }
    if !(0..=3).contains(&target.module_stage) {
        return Err(ApiError::BadRequest(
            "module_stage must be between 0 and 3".into(),
        ));
    }
    if target.module_stage > 0 && target.elite < 2 {
        return Err(ApiError::BadRequest("modules require Elite 2".into()));
    }
    Ok(())
}

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
    if name.is_empty() || name.chars().count() > 100 {
        return Err(ApiError::BadRequest(
            "Preset name must be 1 to 100 characters".into(),
        ));
    }
    validate_preset_target(target)?;
    let target_json = serde_json::to_value(target).map_err(|e| ApiError::Internal(e.into()))?;
    let row = queries::upsert_preset(&state.db, user_id, name, target_json).await?;
    preset_from_row(row)
}

pub async fn delete_preset(state: &AppState, user_id: Uuid, name: &str) -> Result<(), ApiError> {
    queries::delete_preset(&state.db, user_id, name).await?;
    Ok(())
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

        fn add_item(&mut self, item_id: &str, rarity: ItemRarity) {
            self.gamedata.materials.items.insert(
                item_id.to_owned(),
                crate::core::gamedata::types::material::Item {
                    item_id: item_id.to_owned(),
                    name: item_id.to_owned(),
                    rarity,
                    ..Default::default()
                },
            );
        }

        fn ctx(&self) -> PlannerCtx<'_> {
            PlannerCtx {
                gamedata: &self.gamedata,
                asset_index: &self.asset_index,
                inventory_map: &self.inventory,
                user_building: &self.building,
                clears: &self.clears,
            }
        }

        fn requirements(&self, required: &[(&str, i32)]) -> Vec<PlanRequirementItem> {
            build_all_requirements(&self.ctx(), materials(required))
        }

        fn requirements_at(
            &self,
            required: &[(&str, i32)],
            max_tier: Option<i16>,
        ) -> Vec<PlanRequirementItem> {
            aggregate_requirements(&self.ctx(), materials(required), max_tier)
        }
    }

    fn materials(required: &[(&str, i32)]) -> HashMap<String, i32> {
        required
            .iter()
            .map(|(id, count)| ((*id).to_owned(), *count))
            .collect()
    }

    fn find<'a>(reqs: &'a [PlanRequirementItem], id: &str) -> &'a PlanRequirementItem {
        reqs.iter().find(|r| r.id == id).unwrap()
    }

    /// Regression (Rephasic Enantiomer): required 10, have 7, every ingredient
    /// owned outright or craftable one tier down. Reserving ingredient inventory
    /// against the global requirement aggregate reported Craftable 0 / Missing 3.
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

    /// A stage gate the player has not cleared is named in `craft_reason` and
    /// listed, with its id, in `unmet_stages`.
    #[test]
    fn unmet_stage_gate_is_structured() {
        let mut fx = Fixture::new(&[]);
        let mut gated = workshop_formula("a", 1, "F_EVOLVE", &[("ing", 1)]);
        gated.require_stages.push(
            crate::core::gamedata::types::building::WorkshopFormulaUnlockStage {
                stage_id: "main_09-03".to_owned(),
                rank: 2,
            },
        );
        fx.add_workshop(gated);
        fx.gamedata.stages.insert(
            "main_09-03".to_owned(),
            crate::core::gamedata::types::stage::Stage {
                stage_id: "main_09-03".to_owned(),
                code: "9-3".to_owned(),
                ..Default::default()
            },
        );

        let reqs = fx.requirements(&[("a", 1)]);
        let a = find(&reqs, "a");
        assert!(!a.can_craft);
        assert_eq!(a.craft_reason, "Requirements not met: Stage 9-3");
        assert_eq!(a.unmet_stages.len(), 1);
        assert_eq!(a.unmet_stages[0].stage_id, "main_09-03");
        assert_eq!(a.unmet_stages[0].code, "9-3");

        fx.clears.insert(
            "main_09-03".to_owned(),
            StageClear {
                state: 3,
                state_max: 3,
                inferred: false,
                complete_times: 1,
                practice_times: 0,
            },
        );
        let reqs = fx.requirements(&[("a", 1)]);
        assert!(find(&reqs, "a").unmet_stages.is_empty());
    }

    /// The T5 / T4 / T3 chain of the Rephasic regression, with LMD on each
    /// workshop craft: 10 T5 needed, 7 owned, 5 T4 owned.
    fn tiered_fixture() -> Fixture {
        let mut fx = Fixture::new(&[("t5", 7), ("t4", 5), ("t3", 30), ("t2a", 39), ("t2b", 4)]);
        fx.add_item("t5", ItemRarity::Tier5);
        fx.add_item("t4", ItemRarity::Tier4);
        fx.add_item("t3", ItemRarity::Tier3);
        fx.add_item("t2a", ItemRarity::Tier2);
        fx.add_item("t2b", ItemRarity::Tier2);
        let mut t5 = workshop_formula("t5", 1, "F_EVOLVE", &[("t4", 2), ("t2a", 1), ("t2b", 1)]);
        t5.gold_cost = 300;
        fx.add_workshop(t5);
        let mut t4 = workshop_formula("t4", 1, "F_EVOLVE", &[("t3", 3)]);
        t4.gold_cost = 200;
        fx.add_workshop(t4);
        fx
    }

    fn required_counts(reqs: &[PlanRequirementItem]) -> Vec<(&str, i32)> {
        let mut counts: Vec<(&str, i32)> = reqs
            .iter()
            .map(|r| (r.id.as_str(), r.required_count))
            .collect();
        counts.sort_unstable();
        counts
    }

    /// No tier leaves the requirement rows byte for byte what the unflattened
    /// builder makes of the same materials.
    #[test]
    fn no_max_tier_is_byte_identical() {
        let fx = tiered_fixture();
        let required = [("t5", 10), ("t4", 2), ("4001", 5000)];
        let plain = serde_json::to_string(&fx.requirements(&required)).unwrap();
        let unfiltered = serde_json::to_string(&fx.requirements_at(&required, None)).unwrap();
        assert_eq!(plain, unfiltered);
    }

    /// T5 x10 with 7 owned flattened to T3: the 3 short T5 take 6 T4, 3 + 3
    /// T2 and 900 LMD; the 6 T4 claim the 5 owned and the 1 short takes 3 T3
    /// and 200 LMD. No T4 or T5 row is left.
    #[test]
    fn max_tier_three_flattens_through_t4() {
        let fx = tiered_fixture();
        let reqs = fx.requirements_at(&[("t5", 10)], Some(3));
        assert_eq!(
            required_counts(&reqs),
            vec![("4001", 1100), ("t2a", 3), ("t2b", 3), ("t3", 3)]
        );
        assert_eq!(find(&reqs, "t3").missing_count, 0);
        assert_eq!(find(&reqs, "t2b").missing_count, 0);
    }

    /// Flattened to T4, the T4 row keeps the full 6 and its owned 5 count
    /// once: 1 is crafted from T3 inside the row, exactly as unflattened.
    #[test]
    fn max_tier_four_keeps_t4_and_its_inventory() {
        let fx = tiered_fixture();
        let reqs = fx.requirements_at(&[("t5", 10)], Some(4));
        assert_eq!(
            required_counts(&reqs),
            vec![("4001", 900), ("t2a", 3), ("t2b", 3), ("t4", 6)]
        );
        let t4 = find(&reqs, "t4");
        assert_eq!(t4.inventory_count, 5);
        assert_eq!(t4.craftable_count, 1);
        assert_eq!(t4.missing_count, 0);
    }

    /// A direct T4 need and the T4 a T5 expands into merge into one row
    /// before the owned T4 are claimed: 2 + 6 = 8 needed, 5 owned, 3 short,
    /// 9 T3 and 900 + 600 LMD.
    #[test]
    fn flattened_need_merges_with_direct_need() {
        let fx = tiered_fixture();
        let reqs = fx.requirements_at(&[("t5", 10), ("t4", 2)], Some(3));
        assert_eq!(
            required_counts(&reqs),
            vec![("4001", 1500), ("t2a", 3), ("t2b", 3), ("t3", 9)]
        );
    }

    /// An above-tier item fully covered by inventory leaves nothing behind,
    /// and an above-tier item with no recipe stays.
    #[test]
    fn flatten_drops_owned_and_keeps_uncraftable() {
        let mut fx = tiered_fixture();
        fx.add_item("relic", ItemRarity::Tier5);
        let reqs = fx.requirements_at(&[("t5", 7), ("relic", 2)], Some(3));
        assert_eq!(required_counts(&reqs), vec![("relic", 2)]);
    }

    fn roster_entry(
        elite: i16,
        level: i16,
        skill_level: i16,
        masteries: serde_json::Value,
        modules: serde_json::Value,
    ) -> RosterEntry {
        RosterEntry {
            user_id: Uuid::nil(),
            operator_id: "char_test".to_owned(),
            elite,
            level,
            exp: 0,
            potential: 0,
            skill_level,
            favor_point: 0,
            skin_id: None,
            default_skill: None,
            voice_lan: None,
            current_equip: None,
            current_tmpl: None,
            obtained_at: None,
            masteries,
            modules,
        }
    }

    fn plan(
        elite: i16,
        level: i16,
        skill_level: i16,
        skills: serde_json::Value,
        modules: serde_json::Value,
    ) -> OperatorPlan {
        let now = sqlx::types::chrono::Utc::now();
        OperatorPlan {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            operator_id: "char_test".to_owned(),
            target_elite: elite,
            target_level: level,
            target_skill_level: skill_level,
            target_skills: skills,
            target_modules: modules,
            display_on_profile: false,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn plan_met_cases() {
        use serde_json::json;
        let e2_s2m3 = || {
            plan(
                2,
                60,
                7,
                json!([{"skill_index": 1, "mastery_level": 3}]),
                json!([{"module_id": "mod_x", "module_stage": 2}]),
            )
        };
        let roster =
            |elite, level, masteries, modules| roster_entry(elite, level, 7, masteries, modules);
        let mod_x =
            |level: i16, locked: bool| json!([{"id": "mod_x", "level": level, "locked": locked}]);
        let s2m3 = json!([{"index": 1, "mastery": 3}]);

        let cases: Vec<(&str, OperatorPlan, Option<RosterEntry>, bool)> = vec![
            ("unowned", e2_s2m3(), None, false),
            (
                "unowned, E0 L1 plan",
                plan(0, 1, 1, json!([]), json!([])),
                None,
                false,
            ),
            (
                "exact",
                e2_s2m3(),
                Some(roster(2, 60, s2m3.clone(), mod_x(2, false))),
                true,
            ),
            (
                "exceeded: higher level, stage, mastery elsewhere",
                e2_s2m3(),
                Some(roster(
                    2,
                    90,
                    json!([{"index": 0, "mastery": 3}, {"index": 1, "mastery": 3}]),
                    mod_x(3, false),
                )),
                true,
            ),
            (
                "higher elite at a lower level",
                plan(1, 70, 7, json!([]), json!([])),
                Some(roster(2, 1, json!([]), json!([]))),
                true,
            ),
            (
                "level short",
                e2_s2m3(),
                Some(roster(2, 59, s2m3.clone(), mod_x(2, false))),
                false,
            ),
            (
                "skill level short",
                e2_s2m3(),
                Some(roster_entry(2, 60, 6, s2m3.clone(), mod_x(2, false))),
                false,
            ),
            (
                "mastery on the wrong skill index",
                e2_s2m3(),
                Some(roster(
                    2,
                    60,
                    json!([{"index": 0, "mastery": 3}]),
                    mod_x(2, false),
                )),
                false,
            ),
            (
                "mastery short",
                e2_s2m3(),
                Some(roster(
                    2,
                    60,
                    json!([{"index": 1, "mastery": 2}]),
                    mod_x(2, false),
                )),
                false,
            ),
            (
                "module locked at a high level",
                e2_s2m3(),
                Some(roster(2, 60, s2m3.clone(), mod_x(3, true))),
                false,
            ),
            (
                "module missing",
                e2_s2m3(),
                Some(roster(2, 60, s2m3.clone(), json!([]))),
                false,
            ),
            (
                "zero targets are always reached",
                plan(
                    2,
                    60,
                    7,
                    json!([{"skill_index": 2, "mastery_level": 0}]),
                    json!([{"module_id": "mod_x", "module_stage": 0}]),
                ),
                Some(roster(2, 60, json!([]), mod_x(1, true))),
                true,
            ),
        ];

        for (name, plan, entry, expected) in cases {
            assert_eq!(stored_plan_met(&plan, entry.as_ref()), expected, "{name}");
        }
    }

    fn operator_with_module(module_id: &str, stage_costs: &[(&str, &str, i32)]) -> Operator {
        use crate::core::gamedata::types::{
            module::{Module, ModuleItemCost},
            operator::OperatorModule,
        };
        let mut item_cost: HashMap<String, Vec<ModuleItemCost>> = HashMap::new();
        for (stage, id, count) in stage_costs {
            item_cost
                .entry((*stage).to_owned())
                .or_default()
                .push(ModuleItemCost {
                    id: (*id).to_owned(),
                    count: *count,
                    ..Default::default()
                });
        }
        Operator {
            modules: vec![OperatorModule {
                module: Module {
                    uni_equip_id: module_id.to_owned(),
                    item_cost: Some(item_cost),
                    ..Default::default()
                },
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    /// A locked module costs its stage 1 even though the save stores it at
    /// level 1; unlocked at level 1, stage 1 costs nothing.
    #[test]
    fn locked_module_is_stage_zero() {
        use serde_json::json;
        let gamedata = GameData::default();
        let operator = operator_with_module("mod_x", &[("1", "unlock", 5), ("2", "upgrade", 3)]);
        let target = plan(
            2,
            1,
            1,
            json!([]),
            json!([{"module_id": "mod_x", "module_stage": 1}]),
        );
        let module = |locked: bool| json!([{"id": "mod_x", "level": 1, "locked": locked}]);

        let locked = roster_entry(2, 1, 1, json!([]), module(true));
        let costs =
            get_plan_direct_materials(&gamedata, &target, &operator, Some(&locked)).unwrap();
        assert_eq!(costs, materials(&[("unlock", 5)]));
        assert!(!stored_plan_met(&target, Some(&locked)));

        let unlocked = roster_entry(2, 1, 1, json!([]), module(false));
        let costs =
            get_plan_direct_materials(&gamedata, &target, &operator, Some(&unlocked)).unwrap();
        assert!(costs.is_empty());
        assert!(stored_plan_met(&target, Some(&unlocked)));
    }
}
