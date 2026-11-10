#![allow(clippy::unwrap_used, clippy::cast_possible_truncation)]

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::path::PathBuf;

use myrtle_engine_grade::inputs::{BaseScore, GradeInputs, StageClears};
use myrtle_engine_grade::medals::{UserMedalRow, grade_medals};
use myrtle_engine_grade::operators::{ScoreModel, grade_operators_in};
use myrtle_engine_grade::roguelike::grade_roguelike;
use myrtle_engine_grade::stages::grade_stages_detail;
use myrtle_engine_grade::{GradeData, calculate_user_grade};
use myrtle_gamedata::compile::{CompileOptions, compile};
use myrtle_gamedata_raw::tables::sandbox_universe::SandboxUniverse as RawSandboxUniverse;
use myrtle_gamedata_raw::{DirSource, load};
use myrtle_types::Server;
use myrtle_types::roster::RosterEntry;
use myrtle_types::stage_clear::StageClear;
use serde_json::Value;

fn line(out: &mut String, i: usize, k: &str, v: f64) {
    writeln!(out, "{i}\t{k}\t{:016x}\t{v:?}", v.to_bits()).unwrap();
}

fn sandbox_v2_only(gd: &GradeData, p: Option<&Value>) -> f64 {
    myrtle_engine_grade::sandbox::grade_progress(p, &gd.sandbox_universe, false).total
}

#[test]
fn grade_dump() {
    let Ok(players) = std::env::var("GRADE_DIFF") else {
        println!("GRADE_DIFF unset, skipping");
        return;
    };
    let out_path = std::env::var("GRADE_OUT").unwrap();
    let root = PathBuf::from(std::env::var("GAMEDATA_ROOT").unwrap());
    let server = match std::env::var("GAMEDATA_SERVER").as_deref() {
        Ok("cn") => Server::CN,
        _ => Server::EN,
    };
    let raw = load(&DirSource { root }, server).unwrap();
    let sandbox_universe = RawSandboxUniverse::build(&raw.sandbox.perm);
    let m = compile(raw, &CompileOptions::new(server)).unwrap().model;
    let gd = GradeData {
        operators: m.operators.clone(),
        favor: m.favor.clone(),
        roguelike: m.roguelike.clone(),
        medals: m.medals.clone(),
        stage_universe: m.stage_universe,
        sandbox_universe,
    };
    let input: Value = serde_json::from_slice(&std::fs::read(players).unwrap()).unwrap();
    let now = input["now"].as_i64().unwrap();
    let model = ScoreModel::from_vars(|_| None);
    let mut out = String::new();
    for (i, p) in input["players"].as_array().unwrap().iter().enumerate() {
        let roster: Vec<RosterEntry> = serde_json::from_value(p["roster"].clone()).unwrap();
        let supports: Vec<String> = serde_json::from_value(p["supports"].clone()).unwrap();
        let support_ids: HashSet<&str> = supports.iter().map(String::as_str).collect();
        let owned: HashSet<&str> = roster.iter().map(|e| e.operator_id.as_str()).collect();
        let medals: Vec<UserMedalRow> = serde_json::from_value(p["medals"].clone()).unwrap();
        let rogue: Vec<(String, Value)> = serde_json::from_value(p["roguelike"].clone()).unwrap();
        let clears: HashMap<String, StageClear> = p["stage_clears"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, c)| {
                (
                    k.clone(),
                    StageClear {
                        state: c["state"].as_i64().unwrap() as i16,
                        state_max: c["state_max"].as_i64().unwrap() as i16,
                        inferred: c["inferred"].as_bool().unwrap(),
                        complete_times: c["complete_times"].as_i64().unwrap() as i32,
                        practice_times: c["practice_times"].as_i64().unwrap() as i32,
                    },
                )
            })
            .collect();
        let known: HashSet<String> = serde_json::from_value(p["known_stages"].clone()).unwrap();
        let stage_clears = StageClears {
            clears,
            last_synced_ts: p["last_synced_ts"].as_i64(),
        };
        let sandbox = if p["sandbox"].is_null() {
            None
        } else {
            Some(p["sandbox"].clone())
        };
        let base = p["base"].as_f64().unwrap();

        let op = grade_operators_in(&roster, &gd, &support_ids, model);
        let medal = grade_medals(&medals, &gd.medals, &owned, now);
        let rog = grade_roguelike(&rogue, &gd.roguelike);
        let st = grade_stages_detail(&gd, &stage_clears, &known, now);
        let sb_v2 = sandbox_v2_only(&gd, sandbox.as_ref());
        let g = calculate_user_grade(
            &gd,
            &GradeInputs {
                roster: roster.clone(),
                supports: supports.clone(),
                base: BaseScore {
                    score: base,
                    utilization: 0.0,
                    infrastructure: 0.0,
                },
                roguelike: rogue.clone(),
                medals: medals.clone(),
                stage_clears: stage_clears.clone(),
                known_stages: known.clone(),
                sandbox: sandbox.clone(),
                now_unix: now,
                score_model: model,
            },
        );
        assert_eq!(g.operator_grade.to_bits(), op.to_bits());
        line(&mut out, i, "operators", op);
        line(&mut out, i, "medals", medal);
        line(&mut out, i, "roguelike", rog);
        line(&mut out, i, "stages_permanent", st.permanent_score);
        line(&mut out, i, "stages_event", st.event_score);
        line(&mut out, i, "stages", st.total);
        line(&mut out, i, "sandbox", g.sandbox_grade);
        line(&mut out, i, "sandbox_v2_only", sb_v2);
        line(&mut out, i, "total", g.total_score);
    }
    std::fs::write(out_path, out).unwrap();
}
