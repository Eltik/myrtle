#![allow(clippy::unwrap_used)]

use std::collections::HashMap;

use myrtle_gamedata::model::stage_evidence::*;
use myrtle_gamedata_raw::tables::{
    activity::ActivityMission,
    medal::MedalDefinition,
    stage::{Stage, UnlockCondition},
};
use serde_json::{Value, json};

fn stage(id: &str, prereq: Option<(&str, &str)>) -> (String, Stage) {
    let mut s = Stage {
        stage_id: id.to_string(),
        ..Default::default()
    };
    if let Some((p, state)) = prereq {
        s.unlock_condition.push(UnlockCondition {
            stage_id: p.to_string(),
            complete_state: state.to_string(),
        });
    }
    (id.to_string(), s)
}

fn mission(id: &str, template: &str, param: &[&str]) -> ActivityMission {
    ActivityMission {
        id: id.to_string(),
        template: template.to_string(),
        param: param.iter().map(|s| (*s).to_string()).collect(),
        rewards: Vec::new(),
    }
}

fn medal(id: &str, template: &str, param: &[&str]) -> (String, MedalDefinition) {
    let def: MedalDefinition = serde_json::from_value(json!({
        "MedalId": id, "MedalName": "", "MedalType": "activityMedal", "SlotId": 0,
        "Rarity": "T2", "Template": template, "UnlockParam": param,
    }))
    .unwrap();
    (id.to_string(), def)
}

fn index() -> StageEvidenceIndex {
    let stages: HashMap<String, Stage> = [
        stage("act_tr01", None),
        stage("act_01", Some(("act_tr01", "PASS"))),
        stage("act_02", Some(("act_01", "PASS"))),
        stage("act_ex01", None),
        stage("act_ex01#f#", Some(("act_ex01", "COMPLETE"))),
        stage("act_ex02", None),
        stage("act_ex02#f#", Some(("act_ex02", "COMPLETE"))),
        stage("act_sp01", Some(("act_02", "PASS"))),
    ]
    .into_iter()
    .collect();
    let missions = vec![
        mission("m1", "CompleteAnyStage", &["0", "act_01", "2"]),
        mission("m2", "CompleteStageAct", &["1", "act_02", "1", "3"]),
        mission("m3", "CompleteAnyStage", &["0", "act_ex01", "2"]),
        mission("m4", "CompleteAnyStage", &["0", "act_ex02", "2"]),
    ];
    let medals: HashMap<String, MedalDefinition> = [
        medal(
            "cm",
            "PassStageSome",
            &["3", "act_ex01#f#;act_ex02#f#", "2"],
        ),
        medal("num", "PassStageSome", &["3", "act_01;act_02", "2"]),
    ]
    .into_iter()
    .collect();
    StageEvidenceIndex::build(&missions, &medals, &stages)
}

fn done() -> Value {
    json!({"state": 3, "progress": [{"target": 1, "value": 1}]})
}

fn untouched() -> Value {
    json!({"state": 2, "progress": [{"target": 1, "value": 0}]})
}

#[test]
fn missions_flags_medals_and_unlock_chain_bound_each_stage() {
    let user = json!({
        "status": {"flags": {"activities/act/level_act_01_end": 1}},
        "mission": {"missions": {"ACTIVITY": {
            "m1": done(), "m2": done(), "m3": done(), "m4": untouched(),
        }}},
        "medal": {"medals": {
            "cm": {"fts": -1, "val": [[1, 2]]},
            "num": {"fts": 1_700_000_000, "val": [[2, 2]]},
        }},
    });
    let b = index().infer(&user);
    assert_eq!(b["act_01"].lo, STATE_COMPLETE);
    assert_eq!(b["act_02"].lo, STATE_COMPLETE);
    assert_eq!(
        b["act_tr01"].lo, STATE_PASS,
        "cleared act_01 proves its prerequisite"
    );
    assert_eq!(
        b["act_ex01"].lo, STATE_COMPLETE,
        "its cleared challenge mode needs the three-star base"
    );
    assert_eq!(
        b["act_ex02"].hi,
        STATE_PASS - 1,
        "untouched clear mission refutes"
    );
    assert_eq!(
        b["act_ex02#f#"].hi, STATE_UNPLAYED,
        "cannot unlock behind an uncleared stage"
    );
    assert_eq!(
        b["act_ex01#f#"].lo, STATE_COMPLETE,
        "the one counted CM clear is the only one possible"
    );
    assert!(
        !b.contains_key("act_sp01"),
        "nothing proves the optional stage either way"
    );
}

#[test]
fn zero_count_refutes_every_listed_stage() {
    let user = json!({
        "mission": {"missions": {"ACTIVITY": {"m3": done(), "m4": done()}}},
        "medal": {"medals": {"cm": {"fts": -1, "val": [[0, 2]]}}},
    });
    let b = index().infer(&user);
    assert_eq!(b["act_ex01#f#"].hi, STATE_PASS);
    assert_eq!(b["act_ex02#f#"].hi, STATE_PASS);
}

#[test]
fn fold_adds_only_missing_records() {
    let mut stages = json!({"act_01": {"stageId": "act_01", "state": 3}});
    let mut bounds = HashMap::new();
    bounds.insert(
        "act_01".to_string(),
        Bounds {
            lo: 2,
            hi: 3,
            evidence: vec!["flag".into()],
        },
    );
    bounds.insert(
        "act_02".to_string(),
        Bounds {
            lo: 0,
            hi: 1,
            evidence: vec!["mission:m2".into()],
        },
    );
    assert_eq!(fold_into_records(&mut stages, &bounds), 1);
    assert_eq!(stages["act_01"]["state"], 3);
    assert_eq!(stages["act_02"]["stateMax"], 1);
    assert_eq!(stages["act_02"]["inferred"], true);
}
