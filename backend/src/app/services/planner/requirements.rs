//! The plans' combined materials priced against the player's inventory and
//! crafting, one row per item.

use std::collections::{BTreeMap, HashMap};

use super::{EXP_ITEM, LMD_ITEM, progress::get_plan_direct_materials};
use crate::{
    app::error::ApiError,
    core::{
        gamedata::{
            assets::AssetIndex,
            enrich::resolve_item_icon,
            types::{
                GameData,
                building::{BuildingDataFile, FormulaCost, FormulaRoomReq},
                material::{ItemRarity, Materials},
                operator::Operator,
            },
        },
        grade::{base::types::UserBuilding, stages::types::StageClear},
    },
    database::models::{
        planner::{OperatorPlan, PlanRecipe, PlanRecipeCost, PlanRequirementItem, PlanUnmetStage},
        roster::RosterEntry,
    },
};

/// Lowest tier a `max_tier` filter may flatten to.
pub const MIN_FLATTEN_TIER: i16 = 1;
/// Highest tier a `max_tier` filter may flatten to.
pub const MAX_FLATTEN_TIER: i16 = 5;

/// Chip cross-class conversion recipes in the workshop (`F_ASC`, e.g. 3 Medic
/// Chips -> 2 Defender Chips). Converting chips is a waste and never the intended
/// way to acquire them, so the planner treats chips as obtain-only. Dualchip
/// factory recipes carry the same `F_ASC` type but live in `manufact_formulas`
/// and stay craftable: combining chip packs with a catalyst *is* the intended
/// way to obtain dualchips.
const CHIP_CONVERSION_FORMULA_TYPE: &str = "F_ASC";
const CHIP_CRAFT_REASON: &str =
    "Chip conversion isn't recommended: obtain chips from stages, the store, or events";
const NO_RECIPE_REASON: &str = "No workshop or factory formula";
const RECIPE_CYCLE_REASON: &str = "Recipe cycle detected";

/// Upper bound on recipe expansions in `flatten_above_tier`. Game recipes run
/// strictly down in tier, so a real plan stops after a few dozen; the bound
/// only keeps a cyclic recipe table from spinning.
const MAX_FLATTEN_STEPS: usize = 10_000;

/// Read-only inputs shared by the whole requirement computation.
pub(super) struct PlannerCtx<'a> {
    pub(super) gamedata: &'a GameData,
    pub(super) asset_index: &'a AssetIndex,
    pub(super) inventory_map: &'a HashMap<String, i32>,
    pub(super) user_building: &'a UserBuilding,
    pub(super) clears: &'a HashMap<String, StageClear>,
}

/// Display group and rank within it: EXP, LMD, skill summaries, chips,
/// module materials, then everything else, higher rarity first where rarity
/// orders a group.
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

/// An item's display name, type, and tier.
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

/// An item's tier (`TIER_1`..`TIER_6` as 1..6) from the item table. LMD and
/// EXP are tier 0, as in `item_display_meta` (the table files them at
/// `TIER_4` and `TIER_5`), and so is anything the table lacks.
fn item_tier(item_id: &str, materials: &Materials) -> i16 {
    if item_id == LMD_ITEM || item_id == EXP_ITEM {
        return 0;
    }
    materials
        .items
        .get(item_id)
        .map_or(0, |item| rarity_tier(&item.rarity))
}

/// Takes up to `count` units of `item_id` out of `pool` and returns how many
/// it took.
fn claim_from_pool(pool: &mut HashMap<String, i32>, item_id: &str, count: i32) -> i32 {
    let entry = pool.entry(item_id.to_owned()).or_insert(0);
    let take = (*entry).min(count).max(0);
    *entry -= take;
    take
}

/// How many crafts of `output_count` units cover `shortfall`.
const fn crafts_for(shortfall: i32, output_count: i32) -> i32 {
    (shortfall + output_count - 1) / output_count
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

/// What crafting adds to one requirement row.
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
    let inventory_count = *ctx.inventory_map.get(item_id).unwrap_or(&0);

    let satisfied_from_inv =
        reserved_from_pool.unwrap_or_else(|| claim_from_pool(pool, item_id, required_count));
    let shortfall = (required_count - satisfied_from_inv).max(0);

    let craft = match find_recipe(&ctx.gamedata.building, item_id) {
        RecipeLookup::Found(view) => craft_shortfall(ctx, item_id, &view, shortfall, pool, visited),
        RecipeLookup::ChipConversion => CraftOutcome::uncraftable(CHIP_CRAFT_REASON),
        RecipeLookup::None => CraftOutcome::uncraftable(NO_RECIPE_REASON),
    };

    let (icon_id, image) = resolve_item_icon(item_id, &ctx.gamedata.materials, ctx.asset_index);
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

/// Expands `view` to cover `shortfall`, building each ingredient's row in
/// turn. `visited` holds the items being crafted above this one, so a recipe
/// that needs one of them is a cycle and is not expanded.
fn craft_shortfall(
    ctx: &PlannerCtx,
    item_id: &str,
    view: &RecipeView,
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
            // Each ingredient covers its requirement from claimed inventory
            // plus its own recursive crafts; the achievable craft count is the
            // tightest ingredient.
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

/// Whether the plan for `operator_id` counts toward the requirements: it is
/// in `active_ids`, or `active_ids` is empty and every plan counts.
fn is_counted(active_ids: &[String], operator_id: &String) -> bool {
    active_ids.is_empty() || active_ids.contains(operator_id)
}

/// The requirement rows for every plan whose operator is in `active_ids`, or
/// for every plan when `active_ids` is empty.
pub(super) fn calculate_requirements(
    ctx: &PlannerCtx,
    plans_with_ops: &[(OperatorPlan, Operator, Option<&RosterEntry>)],
    active_ids: &[String],
    max_tier: Option<i16>,
) -> Result<Vec<PlanRequirementItem>, ApiError> {
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

/// Each counted plan's own requirement rows, by operator id: what
/// [`calculate_requirements`] gives with that one operator active.
pub(super) fn requirements_by_operator(
    ctx: &PlannerCtx,
    plans_with_ops: &[(OperatorPlan, Operator, Option<&RosterEntry>)],
    active_ids: &[String],
    max_tier: Option<i16>,
) -> Result<BTreeMap<String, Vec<PlanRequirementItem>>, ApiError> {
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
        assert!(!chip.craft_blocked, "no usable recipe is not a gated one");
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
        assert!(a.craft_blocked);
        let b = find(&reqs, "b");
        assert!(!b.craft_blocked);
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
        assert!(a.craft_blocked);
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
}
