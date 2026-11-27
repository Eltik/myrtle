#![allow(clippy::unwrap_used)]

use std::fmt::Write as _;
use std::path::PathBuf;

use myrtle_engine_base::assignment::{
    align_rooms_to_current, compute_current_assignment, compute_live_assignment,
    compute_sustained_assignment, sustained_assignment_value,
};
use myrtle_engine_base::registry::{
    build_name_to_char, resolve_account_facts, targeted_morale_effects,
};
use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
use myrtle_engine_base::rotation::sustain_sim::{simulate_rotation, synced_live_morale};
use myrtle_engine_base::rotation::value::{presets_value, rotation_value};
use myrtle_engine_base::scoring::pools::{optimal_with_bundles, search_economy};
use myrtle_engine_base::types::{BaseAssignment, UserBuilding};
use myrtle_engine_base::{BaseContext, BaseData, grade_base, is_production_room};
use myrtle_gamedata::compile::{CompileOptions, compile};
use myrtle_gamedata_raw::{DirSource, load};
use myrtle_types::Server;
use myrtle_types::roster::RosterEntry;
use serde_json::Value;

fn load_base_data() -> BaseData {
    let root = PathBuf::from(std::env::var("GAMEDATA_ROOT").unwrap());
    let raw = load(&DirSource { root }, Server::EN).unwrap();
    let m = compile(raw, &CompileOptions::new(Server::EN))
        .unwrap()
        .model;
    BaseData::new(m.operators, m.building, m.consts)
}

fn num(out: &mut String, i: usize, k: &str, v: f64) {
    writeln!(out, "{i}\t{k}\t{:016x}\t{v:?}", v.to_bits()).unwrap();
}

fn text(out: &mut String, i: usize, k: &str, v: &str) {
    writeln!(out, "{i}\t{k}\t{v}").unwrap();
}

fn rooms(out: &mut String, i: usize, k: &str, a: &BaseAssignment) {
    for r in &a.rooms {
        let key = format!("{k}.{}", r.slot_id);
        text(
            out,
            i,
            &key,
            &format!(
                "{} {} {:?} [{}]",
                r.room_type,
                r.level,
                r.formula_type,
                r.operators.join(",")
            ),
        );
        num(out, i, &format!("{key}.eff"), r.total_efficiency);
        num(out, i, &format!("{key}.value"), r.order_value);
        num(out, i, &format!("{key}.gold"), r.order_gold);
        text(
            out,
            i,
            &format!("{key}.limit"),
            &format!("{:?}", r.order_limit),
        );
    }
    text(out, i, &format!("{k}.bench"), &a.bench.join(","));
}

#[test]
fn base_dump() {
    let Ok(players) = std::env::var("BASE_DIFF") else {
        println!("BASE_DIFF unset, skipping");
        return;
    };
    let out_path = std::env::var("BASE_OUT").unwrap();
    let rotations: usize = std::env::var("BASE_ROTATIONS")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(0);
    let gd = load_base_data();
    let input: Value = serde_json::from_slice(&std::fs::read(players).unwrap()).unwrap();
    let mut out = String::new();
    for (i, p) in input["players"].as_array().unwrap().iter().enumerate() {
        let roster: Vec<RosterEntry> = serde_json::from_value(p["roster"].clone()).unwrap();
        let json = &p["building"];
        println!("player {i}: {} operators", roster.len());

        let g = grade_base(&roster, Some(json), &gd);
        num(&mut out, i, "grade", g.score);
        num(&mut out, i, "utilization", g.utilization);
        num(&mut out, i, "infrastructure", g.infrastructure);

        let building = UserBuilding::from_json(json);
        if !building
            .rooms
            .iter()
            .any(|r| is_production_room(&r.room_type))
        {
            continue;
        }
        let ctx = BaseContext::build(&roster, &gd, false);
        text(&mut out, i, "profiles", &ctx.profiles.len().to_string());
        let value = |a: &BaseAssignment| {
            sustained_assignment_value(
                a,
                &ctx.profiles,
                &building,
                gd.building(),
                &ctx.registry,
                &ctx.morale_drains,
            )
        };

        let live = synced_live_morale(json);
        let stationed = compute_live_assignment(
            &ctx.profiles,
            &building,
            gd.building(),
            &ctx.registry,
            &ctx.morale_drains,
            None,
            &live,
        );
        rooms(&mut out, i, "live", &stationed);
        num(&mut out, i, "live_value", value(&stationed));

        let economy = search_economy(&ctx.profiles, &building, gd.building(), &ctx.registry);
        let accepted = optimal_with_bundles(
            &ctx.profiles,
            &building,
            gd.building(),
            &ctx.registry,
            &economy.registry,
            &ctx.morale_drains,
            &economy.pins,
        );
        let mut plan = accepted.optimal.clone();
        align_rooms_to_current(&building, &mut plan);
        rooms(&mut out, i, "plan", &plan);
        num(&mut out, i, "plan_value", value(&plan));
        text(&mut out, i, "pins", &format!("{:?}", accepted.pins));

        let current = compute_current_assignment(
            &ctx.profiles,
            &building,
            gd.building(),
            &ctx.registry,
            &ctx.morale_drains,
            None,
        );
        rooms(&mut out, i, "current", &current);

        let sustained = compute_sustained_assignment(
            &ctx.profiles,
            &building,
            gd.building(),
            &ctx.registry,
            &ctx.morale_drains,
        );
        rooms(&mut out, i, "sustained", &sustained.main);
        num(&mut out, i, "sustained.eff", sustained.sustained_efficiency);
        text(
            &mut out,
            i,
            "sustained.bench",
            &sustained.shared_bench.join(","),
        );
        text(
            &mut out,
            i,
            "sustained.sets",
            &sustained.sets.len().to_string(),
        );

        let facts = resolve_account_facts(&ctx.registry, &gd.building().buffs, 3);
        let with_facts = compute_live_assignment(
            &ctx.profiles,
            &building,
            gd.building(),
            &facts,
            &ctx.morale_drains,
            None,
            &live,
        );
        rooms(&mut out, i, "facts", &with_facts);

        if let Some(p) = presets_value(
            &ctx.profiles,
            &building,
            gd.building(),
            &ctx.registry,
            &ctx.morale_drains,
        ) {
            num(&mut out, i, "presets.total", p.value.total_value);
            num(&mut out, i, "presets.lmd", p.value.lmd_per_day);
        }

        if i < rotations {
            let rot = recommend_shift_rotation(
                &ctx.profiles,
                &building,
                gd.building(),
                &accepted.registry,
                &ctx.morale_drains,
                &accepted.pins,
            );
            for s in &rot.shifts {
                for r in &s.rooms {
                    text(
                        &mut out,
                        i,
                        &format!("rot.{}.{}", s.index, r.slot_id),
                        &format!(
                            "{} {:?} {} [{}]",
                            r.room_type,
                            r.formula_type,
                            r.active,
                            r.recommended.join(",")
                        ),
                    );
                }
            }
            text(&mut out, i, "rot.sustained", &rot.sustained.join(","));
            text(&mut out, i, "rot.bench", &rot.bench.join(","));
            text(&mut out, i, "rot.parked", &rot.parked.join(","));
            if let Some(v) = rotation_value(
                &rot,
                &ctx.profiles,
                &building,
                gd.building(),
                &ctx.registry,
                &ctx.morale_drains,
            ) {
                num(&mut out, i, "rot.value.eff", v.production_efficiency);
                num(&mut out, i, "rot.value.lmd", v.lmd_per_day);
                num(&mut out, i, "rot.value.exp", v.exp_per_day);
                num(&mut out, i, "rot.value.total", v.total_value);
            }
            let targeted =
                targeted_morale_effects(&gd.building().buffs, &build_name_to_char(gd.operators()));
            let sim = simulate_rotation(
                &rot,
                &ctx.profiles,
                &building,
                gd.building(),
                &accepted.registry,
                &ctx.morale_drains,
                &targeted,
            );
            text(&mut out, i, "sim.verdict", &format!("{:?}", sim.verdict));
            text(&mut out, i, "sim.depleted", &sim.depleted.len().to_string());
            text(&mut out, i, "sim.overflow", &sim.dorm_overflow.to_string());
        }
    }
    std::fs::write(out_path, out).unwrap();
}
