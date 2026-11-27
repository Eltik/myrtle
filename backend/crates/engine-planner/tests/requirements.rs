#![allow(clippy::unwrap_used, clippy::expect_used)]
use std::collections::HashMap;

use myrtle_engine_base::types::UserBuilding;
use myrtle_engine_planner::{requirements::*, types::PlanRequirementItem};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::{
    building::{FormulaCost, FormulaRoomReq, ManufactFormula, WorkshopFormula},
    material::ItemRarity,
};
use myrtle_types::stage_clear::StageClear;

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
    gamedata: GameModel,
    inventory: HashMap<String, i32>,
    building: UserBuilding,
    clears: HashMap<String, StageClear>,
}

impl Fixture {
    fn new(inventory: &[(&str, i32)]) -> Self {
        Self {
            gamedata: GameModel::default(),
            inventory: inventory
                .iter()
                .map(|(id, count)| ((*id).to_owned(), *count))
                .collect(),
            building: UserBuilding {
                drone_capacity: None,
                rooms: Vec::new(),
            },
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
            myrtle_gamedata_raw::tables::material::Item {
                item_id: item_id.to_owned(),
                name: item_id.to_owned(),
                rarity,
                ..Default::default()
            },
        );
    }

    const fn ctx(&self) -> PlannerCtx<'_> {
        PlannerCtx {
            gamedata: &self.gamedata,
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

#[test]
fn unmet_stage_gate_is_structured() {
    let mut fx = Fixture::new(&[]);
    let mut gated = workshop_formula("a", 1, "F_EVOLVE", &[("ing", 1)]);
    gated.require_stages.push(
        myrtle_gamedata_raw::tables::building::WorkshopFormulaUnlockStage {
            stage_id: "main_09-03".to_owned(),
            rank: 2,
        },
    );
    fx.add_workshop(gated);
    fx.gamedata.stages.insert(
        "main_09-03".to_owned(),
        myrtle_gamedata_raw::tables::stage::Stage {
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

#[test]
fn no_max_tier_is_byte_identical() {
    let fx = tiered_fixture();
    let required = [("t5", 10), ("t4", 2), ("4001", 5000)];
    let plain = serde_json::to_string(&fx.requirements(&required)).unwrap();
    let unfiltered = serde_json::to_string(&fx.requirements_at(&required, None)).unwrap();
    assert_eq!(plain, unfiltered);
}

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

#[test]
fn flattened_need_merges_with_direct_need() {
    let fx = tiered_fixture();
    let reqs = fx.requirements_at(&[("t5", 10), ("t4", 2)], Some(3));
    assert_eq!(
        required_counts(&reqs),
        vec![("4001", 1500), ("t2a", 3), ("t2b", 3), ("t3", 9)]
    );
}

#[test]
fn flatten_drops_owned_and_keeps_uncraftable() {
    let mut fx = tiered_fixture();
    fx.add_item("relic", ItemRarity::Tier5);
    let reqs = fx.requirements_at(&[("t5", 7), ("relic", 2)], Some(3));
    assert_eq!(required_counts(&reqs), vec![("relic", 2)]);
}
