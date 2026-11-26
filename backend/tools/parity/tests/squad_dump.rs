#![allow(clippy::unwrap_used)]

use std::fmt::Write as _;
use std::path::PathBuf;

use myrtle_engine_squad::types::{SquadAnalysis, SquadAnalyzeRequest, SquadMemberBuild};
use myrtle_engine_squad::{SquadData, analyze};
use myrtle_gamedata::compile::{CompileOptions, compile};
use myrtle_gamedata::model::GameModel as Data;
use myrtle_gamedata_raw::tables::module::ModuleType;
use myrtle_gamedata_raw::{DirSource, load};
use myrtle_types::Server;

fn model(root: PathBuf, server: Server) -> Data {
    let raw = load(&DirSource { root }, server).unwrap();
    compile(raw, &CompileOptions::new(server)).unwrap().model
}

fn run(data: &Data, english: &Data, request: &SquadAnalyzeRequest) -> SquadAnalysis {
    analyze(&SquadData::from(data), Some(&english.squad.text), request)
}

#[allow(clippy::unnecessary_wraps)]
fn member(
    id: &str,
    phase: i32,
    level: i32,
    potential: i32,
    skill_index: i32,
    mastery: i32,
    module: Option<&str>,
) -> Option<SquadMemberBuild> {
    Some(SquadMemberBuild {
        char_id: id.to_string(),
        phase,
        level,
        potential,
        skill_index,
        skill_level: 7,
        mastery,
        module_id: module.map(str::to_string),
        module_level: if module.is_some() { 3 } else { 1 },
    })
}

const fn request(members: Vec<Option<SquadMemberBuild>>) -> SquadAnalyzeRequest {
    SquadAnalyzeRequest {
        members,
        support: None,
        start_dp: 10,
        deploy_limit: None,
        dub: None,
    }
}

/// Every operator and skill at a full and an Elite 0 build, plus a five-seat
/// squad on the first skill, one JSON analysis per line.
fn dump(data: &Data, english: &Data) -> String {
    let mut ids: Vec<&String> = data
        .operators
        .keys()
        .filter(|k| k.starts_with("char_"))
        .collect();
    ids.sort();
    let mut text = String::new();
    for id in ids {
        let op = &data.operators[id];
        let phase = i32::try_from(op.phases.len().saturating_sub(1)).unwrap_or(0);
        let level = op.phases.last().map_or(1, |p| p.max_level);
        let module = op
            .modules
            .iter()
            .find(|m| m.module.module_type == ModuleType::Advanced)
            .map(|m| m.module.uni_equip_id.as_str());
        for si in 0..op.skills.len().max(1) {
            let si = i32::try_from(si).unwrap_or(0);
            let mut cases = vec![
                ("full", vec![member(id, phase, level, 5, si, 3, module)]),
                ("low", vec![member(id, 0, 1, 0, si, 3, module)]),
            ];
            if si == 0 {
                cases.push((
                    "squad",
                    vec![
                        member(id, phase, level, 5, 0, 3, module),
                        member("char_151_myrtle", 2, 70, 0, 0, 0, None),
                        member("char_103_angel", 2, 80, 0, 2, 3, None),
                        member("char_002_amiya", 2, 80, 0, 1, 3, None),
                        member("char_202_demkni", 2, 90, 0, 1, 3, None),
                    ],
                ));
            }
            for (tag, members) in cases {
                let a = run(data, english, &request(members));
                writeln!(
                    text,
                    "{id} s{si} {tag}\t{}",
                    serde_json::to_string(&a).unwrap()
                )
                .unwrap();
            }
            if si == 0 {
                let over = Some(SquadMemberBuild {
                    char_id: id.clone(),
                    phase: 9,
                    level: 999,
                    potential: 9,
                    skill_index: 9,
                    skill_level: 99,
                    mastery: 9,
                    module_id: Some("uniequip_none".to_string()),
                    module_level: 9,
                });
                let edge = SquadAnalyzeRequest {
                    members: vec![
                        member("char_151_myrtle", 2, 70, 0, 0, 0, None),
                        None,
                        member("char_none", 2, 1, 0, 0, 0, None),
                        member(id, phase, level, 0, 0, 0, None),
                    ],
                    support: over,
                    start_dp: 22,
                    deploy_limit: Some(3),
                    dub: Some("JP".to_string()),
                };
                let a = run(data, english, &edge);
                writeln!(
                    text,
                    "{id} s{si} edge\t{}",
                    serde_json::to_string(&a).unwrap()
                )
                .unwrap();
            }
        }
    }
    text
}

#[test]
fn squad_dump() {
    let Ok(out) = std::env::var("SQUAD_DUMP") else {
        println!("SQUAD_DUMP unset, skipping");
        return;
    };
    let out = PathBuf::from(out);
    std::fs::create_dir_all(&out).unwrap();
    let en = model(
        PathBuf::from(std::env::var("GAMEDATA_ROOT").unwrap()),
        Server::EN,
    );
    std::fs::write(out.join("en.tsv"), dump(&en, &en)).unwrap();
    if let Ok(root) = std::env::var("SQUAD_CN_ROOT") {
        let cn = model(PathBuf::from(root), Server::CN);
        std::fs::write(out.join("cn.tsv"), dump(&cn, &en)).unwrap();
    }
}
