#![allow(clippy::unwrap_used, clippy::expect_used)]
use std::collections::HashMap;

use myrtle_engine_planner::{
    progress::*,
    types::{OperatorPlan, PlanInput},
};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::operator::Operator;
use myrtle_types::roster::RosterEntry;
use uuid::Uuid;

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
    OperatorPlan {
        operator_id: "char_test".to_owned(),
        target_elite: elite,
        target_level: level,
        target_skill_level: skill_level,
        target_skills: skills,
        target_modules: modules,
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
            Some(roster(2, 60, s2m3, json!([]))),
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
    use myrtle_gamedata_raw::tables::{
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

#[test]
fn locked_module_is_stage_zero() {
    use serde_json::json;
    let gamedata = GameModel::default();
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
    let costs = get_plan_direct_materials(&gamedata, &target, &operator, Some(&locked)).unwrap();
    assert_eq!(costs, HashMap::from([("unlock".to_owned(), 5)]));
    assert!(!stored_plan_met(&target, Some(&locked)));

    let unlocked = roster_entry(2, 1, 1, json!([]), module(false));
    let costs = get_plan_direct_materials(&gamedata, &target, &operator, Some(&unlocked)).unwrap();
    assert!(costs.is_empty());
    assert!(stored_plan_met(&target, Some(&unlocked)));
}

const fn input(
    elite: i16,
    level: i16,
    skill_level: i16,
    skills: serde_json::Value,
    modules: serde_json::Value,
) -> PlanInput {
    PlanInput {
        target_elite: elite,
        target_level: level,
        target_skill_level: skill_level,
        target_skills: skills,
        target_modules: modules,
        display_on_profile: false,
    }
}

#[test]
fn raise_to_roster_lifts_a_plan_below_the_roster() {
    let entry = roster_entry(
        2,
        40,
        7,
        serde_json::json!([{ "index": 0, "mastery": 2 }]),
        serde_json::json!([
            { "id": "uniequip_a", "level": 2, "locked": false },
            { "id": "uniequip_b", "level": 3, "locked": false }
        ]),
    );
    let mut plan = input(
        2,
        1,
        4,
        serde_json::json!([{ "skill_index": 0, "mastery_level": 1 }]),
        serde_json::json!([
            { "module_id": "uniequip_a", "module_stage": 1 },
            { "module_id": "uniequip_b", "module_stage": 0 }
        ]),
    );
    raise_to_roster(&mut plan, Some(&entry)).unwrap();
    assert_eq!((plan.target_elite, plan.target_level), (2, 40));
    assert_eq!(plan.target_skill_level, 7);
    assert_eq!(
        parse_target_skills(&plan.target_skills).unwrap()[0].mastery_level,
        2
    );
    let modules = parse_target_modules(&plan.target_modules).unwrap();
    assert_eq!(modules[0].module_stage, 2);
    assert_eq!(
        modules[1].module_stage, 0,
        "an unplanned module stays unplanned"
    );
}

#[test]
fn raise_to_roster_keeps_a_plan_above_the_roster_and_any_plan_without_one() {
    let entry = roster_entry(1, 70, 7, serde_json::json!([]), serde_json::json!([]));
    let mut plan = input(2, 1, 7, serde_json::json!([]), serde_json::json!([]));
    raise_to_roster(&mut plan, Some(&entry)).unwrap();
    assert_eq!((plan.target_elite, plan.target_level), (2, 1));

    let mut unowned = input(0, 1, 1, serde_json::json!([]), serde_json::json!([]));
    raise_to_roster(&mut unowned, None).unwrap();
    assert_eq!((unowned.target_elite, unowned.target_level), (0, 1));
}
