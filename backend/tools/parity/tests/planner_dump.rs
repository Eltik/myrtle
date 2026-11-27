#![allow(
    clippy::unwrap_used,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::manual_is_multiple_of,
    clippy::missing_const_for_fn,
    clippy::too_many_lines,
    clippy::type_complexity
)]

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use myrtle_engine_base::types::{UserBuilding, UserRoom};
use myrtle_engine_planner::{
    OperatorPlan, PlanError, PlanInput, PlanRequirementItem, PlannerCtx, PresetTarget,
    calculate_leveling_costs, calculate_requirements, ensure_plannable_id, raise_to_roster,
    requirements_by_operator, stored_plan_met, validate_plan, validate_preset,
};
use myrtle_gamedata::compile::{CompileOptions, compile};
use myrtle_gamedata::model::GameModel as Data;
use myrtle_gamedata_raw::tables::operator::Operator;
use myrtle_gamedata_raw::{DirSource, load as load_raw};
use myrtle_types::{Server, roster::RosterEntry, stage_clear::StageClear};
use serde_json::json;
use uuid::Uuid;

struct Loaded {
    data: Data,
}

fn load(root: &Path, dir: &str) -> Option<Loaded> {
    let root = root.join(dir);
    if !root.join("gamedata/excel").is_dir() {
        return None;
    }
    let server = match dir {
        "en" => Server::EN,
        "cn" => Server::CN,
        "jp" => Server::JP,
        _ => Server::KR,
    };
    let raw = load_raw(&DirSource { root }, server).unwrap();
    let data = compile(raw, &CompileOptions::new(server)).unwrap().model;
    Some(Loaded { data })
}

fn data_of(loaded: &Loaded) -> &Data {
    &loaded.data
}

fn ctx<'a>(
    loaded: &'a Loaded,
    inventory: &'a HashMap<String, i32>,
    building: &'a UserBuilding,
    clears: &'a HashMap<String, StageClear>,
) -> PlannerCtx<'a> {
    PlannerCtx {
        gamedata: &loaded.data,
        inventory_map: inventory,
        user_building: building,
        clears,
    }
}

/// The stored plan the engine reads: the operator and the six target columns.
fn plan(operator_id: &str, p: &PlanInput) -> OperatorPlan {
    OperatorPlan {
        operator_id: operator_id.to_owned(),
        target_elite: p.target_elite,
        target_level: p.target_level,
        target_skill_level: p.target_skill_level,
        target_skills: p.target_skills.clone(),
        target_modules: p.target_modules.clone(),
    }
}

fn plan_id(p: &OperatorPlan) -> String {
    p.operator_id.clone()
}

/// What main would answer the request with: the 400's message, or that it was
/// an internal error.
fn message(e: &PlanError) -> String {
    match e {
        PlanError::Invalid(text) => text.clone(),
        PlanError::Encode(_) => "internal".to_owned(),
    }
}

const ROSTERS: usize = 5;

fn fnv(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn input(
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

fn top_elite(op: &Operator) -> i16 {
    (op.phases.len() as i16) - 1
}

fn cap(op: &Operator, elite: i16) -> i16 {
    op.phases[elite as usize].max_level as i16
}

fn top_skill_level(op: &Operator) -> i16 {
    (op.all_skill_level_up.len() + 1) as i16
}

/// Everything the operator has, at its highest.
fn max_plan(op: &Operator) -> PlanInput {
    let elite = top_elite(op);
    let skill_level = top_skill_level(op);
    let skills: Vec<serde_json::Value> = if elite == 2 && skill_level == 7 {
        op.skills
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.level_up_cost_cond.is_empty())
            .map(|(i, s)| json!({"skill_index": i, "mastery_level": s.level_up_cost_cond.len()}))
            .collect()
    } else {
        Vec::new()
    };
    let modules: Vec<serde_json::Value> = op
        .modules
        .iter()
        .filter(|m| m.module.item_cost.is_some())
        .map(|m| json!({"module_id": m.module.uni_equip_id, "module_stage": 3}))
        .collect();
    input(
        elite,
        cap(op, elite),
        skill_level,
        json!(skills),
        json!(modules),
    )
}

fn mid_plan(op: &Operator) -> PlanInput {
    let elite = top_elite(op).min(1);
    let skill_level = top_skill_level(op).min(if elite >= 1 { 7 } else { 4 });
    input(elite, cap(op, elite), skill_level, json!([]), json!([]))
}

fn low_plan() -> PlanInput {
    input(0, 1, 1, json!([]), json!([]))
}

fn partial_plan(op: &Operator) -> Option<PlanInput> {
    if top_elite(op) < 2 || top_skill_level(op) < 7 {
        return None;
    }
    let skills: Vec<serde_json::Value> = op
        .skills
        .iter()
        .take(1)
        .filter(|s| !s.level_up_cost_cond.is_empty())
        .map(|s| json!({"skill_index": 0, "mastery_level": s.level_up_cost_cond.len().min(2)}))
        .collect();
    let modules: Vec<serde_json::Value> = op
        .modules
        .iter()
        .filter(|m| m.module.item_cost.is_some())
        .take(1)
        .map(|m| json!({"module_id": m.module.uni_equip_id, "module_stage": 2}))
        .collect();
    Some(input(
        2,
        cap(op, 2).min(60),
        7,
        json!(skills),
        json!(modules),
    ))
}

fn entry(
    id: &str,
    elite: i16,
    level: i16,
    skill_level: i16,
    masteries: serde_json::Value,
    modules: serde_json::Value,
) -> RosterEntry {
    RosterEntry {
        user_id: Uuid::nil(),
        operator_id: id.to_owned(),
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

/// Five places an account can be with one operator: not owned, fresh, part
/// way with every module locked, far along, and finished.
fn roster(id: &str, op: &Operator, which: usize) -> Option<RosterEntry> {
    let top = top_elite(op);
    let module_rows = |level: i16, locked: bool, first_open: bool| -> serde_json::Value {
        json!(
            op.modules
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    if first_open && i == 0 {
                        json!({"id": m.module.uni_equip_id, "level": 2, "locked": false})
                    } else {
                        json!({"id": m.module.uni_equip_id, "level": level, "locked": locked})
                    }
                })
                .collect::<Vec<_>>()
        )
    };
    match which {
        0 => None,
        1 => Some(entry(id, 0, 1, 1, json!([]), json!([]))),
        2 => {
            let elite = top.min(1);
            Some(entry(
                id,
                elite,
                cap(op, elite).min(30),
                top_skill_level(op).min(4),
                json!([]),
                module_rows(1, true, false),
            ))
        }
        3 => Some(entry(
            id,
            top,
            (cap(op, top) - 10).max(1),
            top_skill_level(op),
            json!([{"index": 0, "mastery": 1}, {"index": 1, "mastery": 3}]),
            module_rows(1, true, true),
        )),
        _ => Some(entry(
            id,
            top,
            cap(op, top),
            top_skill_level(op),
            json!(
                op.skills
                    .iter()
                    .enumerate()
                    .map(|(i, s)| json!({"index": i, "mastery": s.level_up_cost_cond.len()}))
                    .collect::<Vec<_>>()
            ),
            module_rows(3, false, false),
        )),
    }
}

fn verdict<T, E>(result: &Result<T, E>, say: impl Fn(&E) -> String) -> String {
    match result {
        Ok(_) => "ok".to_owned(),
        Err(e) => say(e),
    }
}

fn input_json(p: &PlanInput) -> String {
    json!([
        p.target_elite,
        p.target_level,
        p.target_skill_level,
        p.target_skills,
        p.target_modules,
        p.display_on_profile
    ])
    .to_string()
}

/// Rows as one line: how many, a digest of their JSON, and the three sums a
/// difference would show up in first.
fn digest(rows: &[PlanRequirementItem]) -> String {
    let text = serde_json::to_string(rows).unwrap();
    let required: i64 = rows.iter().map(|r| i64::from(r.required_count)).sum();
    let craftable: i64 = rows.iter().map(|r| i64::from(r.craftable_count)).sum();
    let missing: i64 = rows.iter().map(|r| i64::from(r.missing_count)).sum();
    format!(
        "{} {:016x} {} {required} {craftable} {missing}",
        rows.len(),
        fnv(&text),
        text.len()
    )
}

fn inventory(data: &Data, which: &str) -> HashMap<String, i32> {
    let mut map = HashMap::new();
    match which {
        "some" => {
            for id in data.materials.items.keys() {
                let h = fnv(id);
                if h % 3 != 0 {
                    map.insert(id.clone(), (h % 40) as i32);
                }
            }
            map.insert("4001".to_owned(), 500_000);
            map.insert("5001".to_owned(), 100_000);
        }
        "rich" => {
            for id in data.materials.items.keys() {
                map.insert(id.clone(), 9_999);
            }
        }
        _ => {}
    }
    map
}

fn room(slot: usize, kind: &str, level: i32) -> UserRoom {
    UserRoom {
        slot_id: format!("slot_{slot}"),
        room_type: kind.to_owned(),
        level,
        ..Default::default()
    }
}

fn building(which: &str) -> UserBuilding {
    let rooms = match which {
        "basic" => vec![
            room(0, "CONTROL", 1),
            room(1, "WORKSHOP", 1),
            room(2, "MANUFACTURE", 1),
            room(3, "TRADING", 1),
            room(4, "POWER", 1),
        ],
        "full" => vec![
            room(0, "CONTROL", 5),
            room(1, "WORKSHOP", 3),
            room(2, "MANUFACTURE", 3),
            room(3, "MANUFACTURE", 3),
            room(4, "MANUFACTURE", 3),
            room(5, "MANUFACTURE", 3),
            room(6, "TRADING", 3),
            room(7, "TRADING", 3),
            room(8, "POWER", 3),
            room(9, "POWER", 3),
            room(10, "POWER", 3),
            room(11, "TRAINING", 3),
            room(12, "HIRE", 3),
            room(13, "MEETING", 3),
            room(14, "DORMITORY", 5),
        ],
        _ => Vec::new(),
    };
    UserBuilding {
        rooms,
        drone_capacity: None,
    }
}

fn clears(data: &Data, which: &str) -> HashMap<String, StageClear> {
    let clear = |state: i16, complete_times: i32| StageClear {
        state,
        state_max: state,
        inferred: false,
        complete_times,
        practice_times: 0,
    };
    let mut map = HashMap::new();
    match which {
        "all" => {
            for id in data.stages.keys() {
                map.insert(id.clone(), clear(3, 1));
            }
        }
        "some" => {
            for id in data.stages.keys() {
                match fnv(id) % 4 {
                    0 => {
                        map.insert(id.clone(), clear(2, 1));
                    }
                    // Not cleared by state, but run at least once.
                    1 => {
                        map.insert(id.clone(), clear(1, 2));
                    }
                    // On record and never cleared.
                    2 => {
                        map.insert(id.clone(), clear(1, 0));
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    map
}

fn sorted_costs(map: &HashMap<String, i32>) -> String {
    let mut rows: Vec<(&String, &i32)> = map.iter().collect();
    rows.sort();
    format!("{rows:?}")
}

fn validation_section(tag: &str, ops: &[(&String, &Operator)], text: &mut String) {
    for (id, op) in ops {
        let top = max_plan(op);
        let mut cases: Vec<(&str, PlanInput)> = vec![
            ("max", top.clone()),
            ("mid", mid_plan(op)),
            ("low", low_plan()),
        ];
        if let Some(p) = partial_plan(op) {
            cases.push(("partial", p));
        }
        let with = |change: &dyn Fn(&mut PlanInput)| {
            let mut p = top.clone();
            change(&mut p);
            p
        };
        cases.push(("elite+1", with(&|p| p.target_elite += 1)));
        cases.push(("elite-1", with(&|p| p.target_elite = -1)));
        cases.push(("level 0", with(&|p| p.target_level = 0)));
        cases.push(("level cap+1", with(&|p| p.target_level += 1)));
        cases.push(("skill 0", with(&|p| p.target_skill_level = 0)));
        cases.push(("skill top+1", with(&|p| p.target_skill_level += 1)));
        cases.push((
            "skill index 9",
            with(&|p| p.target_skills = json!([{"skill_index": 9, "mastery_level": 1}])),
        ));
        cases.push((
            "mastery 4",
            with(&|p| p.target_skills = json!([{"skill_index": 0, "mastery_level": 4}])),
        ));
        cases.push((
            "mastery -1",
            with(&|p| p.target_skills = json!([{"skill_index": 0, "mastery_level": -1}])),
        ));
        cases.push(("skills malformed", with(&|p| p.target_skills = json!("x"))));
        cases.push((
            "modules malformed",
            with(&|p| p.target_modules = json!({"a": 1})),
        ));
        cases.push((
            "module unknown",
            with(&|p| p.target_modules = json!([{"module_id": "nope", "module_stage": 1}])),
        ));
        if let Some(first) = op.modules.first() {
            let module_id = first.module.uni_equip_id.clone();
            let m4 = module_id.clone();
            cases.push((
                "module stage 4",
                with(&move |p| p.target_modules = json!([{"module_id": m4, "module_stage": 4}])),
            ));
            let mut early = mid_plan(op);
            early.target_modules = json!([{"module_id": module_id, "module_stage": 1}]);
            cases.push(("module at mid", early));
            let mut first_level = top.clone();
            first_level.target_level = 1;
            cases.push(("modules at level 1", first_level));
        }
        let mut early = mid_plan(op);
        early.target_skills = json!([{"skill_index": 0, "mastery_level": 1}]);
        cases.push(("mastery at mid", early));
        let mut low_skill = top.clone();
        low_skill.target_skill_level = 6.min(top.target_skill_level);
        cases.push(("mastery below skill 7", low_skill));
        let mut low_elite = top.clone();
        low_elite.target_elite = 0;
        low_elite.target_level = 1;
        cases.push(("max at elite 0", low_elite));

        for (name, p) in &cases {
            writeln!(
                text,
                "{tag} validate {id} {name}\t{}",
                verdict(&validate_plan(op, p), message)
            )
            .unwrap();
        }
        writeln!(
            text,
            "{tag} plannable {id}\t{}",
            verdict(&ensure_plannable_id(id), message)
        )
        .unwrap();
    }
}

fn progress_section(tag: &str, data: &Data, ops: &[(&String, &Operator)], text: &mut String) {
    for (id, op) in ops {
        let mut plans: Vec<(&str, PlanInput)> = vec![
            ("max", max_plan(op)),
            ("mid", mid_plan(op)),
            ("low", low_plan()),
        ];
        if let Some(p) = partial_plan(op) {
            plans.push(("partial", p));
        }
        for (name, p) in &plans {
            let stored = plan(id, p);
            let mut met = String::new();
            for which in 0..ROSTERS {
                let r = roster(id, op, which);
                met.push(if stored_plan_met(&stored, r.as_ref()) {
                    '1'
                } else {
                    '0'
                });
                let mut raised = p.clone();
                let outcome = raise_to_roster(&mut raised, r.as_ref());
                writeln!(
                    text,
                    "{tag} raise {id} {name} r{which}\t{}\t{}\t{}",
                    verdict(&outcome, message),
                    input_json(&raised),
                    verdict(&validate_plan(op, &raised), message)
                )
                .unwrap();
            }
            writeln!(text, "{tag} met {id} {name}\t{met}").unwrap();
        }
        let top = top_elite(op);
        let mut level_rows = Vec::new();
        for (from_elite, from_level, to_elite, to_level) in [
            (0, 1, top, cap(op, top)),
            (0, 1, 0, cap(op, 0)),
            (0, cap(op, 0), top, 1),
            (top, 5, top, 5),
            (top, cap(op, top), 0, 1),
            (0, 7, top.min(1), 20),
            (0, 1, top, cap(op, top) + 40),
        ] {
            let mut costs = HashMap::new();
            calculate_leveling_costs(
                op, data, from_elite, from_level, to_elite, to_level, &mut costs,
            );
            level_rows.push(sorted_costs(&costs));
        }
        writeln!(text, "{tag} leveling {id}\t{}", level_rows.join(" | ")).unwrap();
    }
}

fn valid_plans<'a>(
    ops: &[(&'a String, &'a Operator)],
    make: impl Fn(&Operator) -> Option<PlanInput>,
) -> Vec<(&'a String, &'a Operator, PlanInput)> {
    ops.iter()
        .filter_map(|(id, op)| {
            let p = make(op)?;
            validate_plan(op, &p).ok()?;
            ensure_plannable_id(id).ok()?;
            Some((*id, *op, p))
        })
        .collect()
}

fn direct_section(tag: &str, loaded: &Loaded, ops: &[(&String, &Operator)], text: &mut String) {
    let empty = HashMap::new();
    let no_rooms = building("none");
    let no_clears = HashMap::new();
    let c = ctx(loaded, &empty, &no_rooms, &no_clears);
    let kinds: [(&str, &dyn Fn(&Operator) -> Option<PlanInput>); 3] = [
        ("max", &|op| Some(max_plan(op))),
        ("mid", &|op| Some(mid_plan(op))),
        ("partial", &partial_plan),
    ];
    for (name, make) in kinds {
        for (id, op, p) in valid_plans(ops, make) {
            let mut cells = Vec::new();
            for which in 0..ROSTERS {
                let r = roster(id, op, which);
                let rows = [(plan(id, &p), op.clone(), r.as_ref())];
                let out = calculate_requirements(&c, &rows, &[], None);
                cells.push(match &out {
                    Ok(rows) => digest(rows),
                    Err(e) => message(e),
                });
            }
            writeln!(text, "{tag} direct {id} {name}\t{}", cells.join(" | ")).unwrap();
        }
    }
}

fn aggregate_section(tag: &str, loaded: &Loaded, ops: &[(&String, &Operator)], text: &mut String) {
    let data = data_of(loaded);
    let plans = valid_plans(ops, |op| Some(max_plan(op)));
    let rosters: Vec<Option<RosterEntry>> = plans
        .iter()
        .enumerate()
        .map(|(i, (id, op, _))| roster(id, op, i % ROSTERS))
        .collect();
    let rows: Vec<(OperatorPlan, Operator, Option<&RosterEntry>)> = plans
        .iter()
        .zip(&rosters)
        .map(|((id, op, p), r)| (plan(id, p), (*op).clone(), r.as_ref()))
        .collect();
    writeln!(text, "{tag} plans\t{}", rows.len()).unwrap();
    let every_seventh: Vec<String> = plans
        .iter()
        .step_by(7)
        .map(|(id, _, _)| (*id).clone())
        .collect();

    for inv_name in ["empty", "some", "rich"] {
        let inv = inventory(data, inv_name);
        for bld_name in ["none", "basic", "full"] {
            let bld = building(bld_name);
            for clr_name in ["none", "some", "all"] {
                let clr = clears(data, clr_name);
                let c = ctx(loaded, &inv, &bld, &clr);
                for tier in [None, Some(1), Some(2), Some(3), Some(4), Some(5)] {
                    let all = calculate_requirements(&c, &rows, &[], tier).unwrap();
                    let part = calculate_requirements(&c, &rows, &every_seventh, tier).unwrap();
                    writeln!(
                        text,
                        "{tag} agg {inv_name} {bld_name} {clr_name} {tier:?}\t{}\t{}",
                        digest(&all),
                        digest(&part)
                    )
                    .unwrap();
                    // The whole answer for four corners, so a digest that
                    // differs has rows to read beside it.
                    let corner = matches!(
                        (inv_name, bld_name, clr_name, tier),
                        ("empty", "none", "none", None)
                            | ("some", "basic", "some", None)
                            | ("some", "full", "all", Some(3))
                            | ("rich", "full", "all", Some(1))
                    );
                    if corner {
                        for row in &part {
                            writeln!(
                                text,
                                "{tag} row {inv_name} {bld_name} {clr_name} {tier:?} {}\t{}",
                                row.id,
                                serde_json::to_string(row).unwrap()
                            )
                            .unwrap();
                        }
                    }
                }
            }
        }
    }

    let inv = inventory(data, "some");
    let bld = building("basic");
    let clr = clears(data, "some");
    let c = ctx(loaded, &inv, &bld, &clr);
    let few = &rows[..rows.len().min(25)];
    let few_ids: Vec<String> = few.iter().step_by(3).map(|(p, _, _)| plan_id(p)).collect();
    for (name, active, tier) in [
        ("all", Vec::new(), None),
        ("all tier 3", Vec::new(), Some(3)),
        ("thirds", few_ids.clone(), None),
        ("thirds tier 2", few_ids, Some(2)),
    ] {
        let by_id = requirements_by_operator(&c, few, &active, tier).unwrap();
        for (id, rows) in &by_id {
            writeln!(text, "{tag} byop {name} {id}\t{}", digest(rows)).unwrap();
        }
    }
}

fn preset_section(tag: &str, text: &mut String) {
    let long = "x".repeat(101);
    let hundred = "é".repeat(100);
    let names = ["", "Mine", long.as_str(), hundred.as_str()];
    let mut n = 0;
    for name in names {
        for elite in [-1, 0, 1, 2, 3] {
            for level in [
                None,
                Some(0),
                Some(1),
                Some(50),
                Some(51),
                Some(80),
                Some(90),
                Some(91),
            ] {
                for skill_level in [0, 1, 4, 5, 7, 8] {
                    for masteries in [[0, 0, 0], [3, 0, 0], [0, 1, 4], [-1, 0, 0]] {
                        for module_stage in [-1, 0, 3, 4] {
                            let target = PresetTarget {
                                elite,
                                level,
                                skill_level,
                                masteries,
                                module_stage,
                                display_on_profile: n % 2 == 0,
                            };
                            writeln!(
                                text,
                                "{tag} preset {n}\t{}",
                                verdict(&validate_preset(name, &target), message)
                            )
                            .unwrap();
                            n += 1;
                        }
                    }
                }
            }
        }
    }
}

fn dump(tag: &str, loaded: &Loaded) -> String {
    let data = data_of(loaded);
    let mut ops: Vec<(&String, &Operator)> = data
        .operators
        .iter()
        .filter(|(id, op)| id.starts_with("char_") && !op.phases.is_empty())
        .collect();
    ops.sort_by(|a, b| a.0.cmp(b.0));
    let mut text = String::new();
    writeln!(text, "{tag} operators\t{}", ops.len()).unwrap();
    validation_section(tag, &ops, &mut text);
    progress_section(tag, data, &ops, &mut text);
    direct_section(tag, loaded, &ops, &mut text);
    aggregate_section(tag, loaded, &ops, &mut text);
    preset_section(tag, &mut text);
    text
}

#[test]
fn planner_dump() {
    let Ok(out) = std::env::var("PLANNER_DUMP") else {
        return;
    };
    let root = PathBuf::from(std::env::var("PLANNER_ROOT").unwrap());
    let out = PathBuf::from(out);
    std::fs::create_dir_all(&out).unwrap();
    for dir in ["en", "cn", "jp", "kr"] {
        let Some(loaded) = load(&root, dir) else {
            continue;
        };
        std::fs::write(out.join(format!("{dir}.tsv")), dump(dir, &loaded)).unwrap();
    }
}
