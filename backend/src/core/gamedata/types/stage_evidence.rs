//! What a player's account still proves about event stages after the client
//! drops their battle records.
//!
//! `dungeon.stages` only keeps a limited event's battle stages while the event
//! is open (its story stages stay). Three things survive the close and each
//! bounds a stage's clear state:
//!
//! * event missions (`mission.missions.*`): the stage-clear templates name one
//!   stage and the state they ask for; a claimed mission proves the stage
//!   reached that state, an unclaimed one that stayed at zero progress proves
//!   it did not;
//! * medals with a `PassStage*` template: an obtained medal proves every listed
//!   stage reached the state; `PassStageSome` also carries how many of the
//!   listed stages did, which pins the rest once the others are known;
//! * `status.flags["activities/{act}/level_{stage}_end"]`, the post-battle
//!   story unlock, which only a clear sets;
//!
//! and the stage table's unlock chain ties them together: a cleared stage
//! proves its prerequisites, a stage that cannot be unlocked cannot be cleared.
//! The result is a `[lo, hi]` band on the game's clear state per stage
//! (0 unplayed, 2 cleared, 3 three-starred); first-clear Originite Prime is
//! claimed at 3.

use std::collections::HashMap;

use serde_json::{Value, json};

use super::{activity::ActivityMission, medal::MedalDefinition, stage::Stage};

pub const STATE_UNPLAYED: i16 = 0;
pub const STATE_PASS: i16 = 2;
pub const STATE_COMPLETE: i16 = 3;

const MAX_ROUNDS: usize = 8;

#[derive(Debug, Clone)]
struct MissionRule {
    stage_id: String,
    state: i16,
    refutes: bool,
}

#[derive(Debug, Clone)]
struct MedalRule {
    stages: Vec<String>,
    state: i16,
    counted: bool,
}

#[derive(Debug, Clone)]
struct Prerequisite {
    stage_id: String,
    state: i16,
}

#[derive(Debug, Clone, Default)]
pub struct StageEvidenceIndex {
    missions: HashMap<String, MissionRule>,
    medals: HashMap<String, MedalRule>,
    unlocks: HashMap<String, Vec<Prerequisite>>,
}

/// The band a stage's clear state is proven to lie in, with the record ids
/// that set each end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bounds {
    pub lo: i16,
    pub hi: i16,
    pub evidence: Vec<String>,
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            lo: STATE_UNPLAYED,
            hi: STATE_COMPLETE,
            evidence: Vec::new(),
        }
    }
}

impl Bounds {
    pub const fn informative(&self) -> bool {
        self.lo >= STATE_PASS || self.hi < STATE_COMPLETE
    }

    fn raise(&mut self, state: i16, source: &str) -> bool {
        if state <= self.lo {
            return false;
        }
        self.lo = state;
        self.push(source);
        true
    }

    fn lower(&mut self, state: i16, source: &str) -> bool {
        if state >= self.hi {
            return false;
        }
        self.hi = state;
        self.push(source);
        true
    }

    fn push(&mut self, source: &str) {
        if !self.evidence.iter().any(|s| s == source) {
            self.evidence.push(source.to_string());
        }
    }
}

fn parse_state(raw: &str) -> Option<i16> {
    match raw {
        "2" => Some(STATE_PASS),
        "3" => Some(STATE_COMPLETE),
        _ => None,
    }
}

fn unlock_state(raw: &str) -> Option<i16> {
    match raw {
        "PASS" => Some(STATE_PASS),
        "COMPLETE" => Some(STATE_COMPLETE),
        _ => None,
    }
}

fn single_stage<'a>(raw: &'a str, stages: &HashMap<String, Stage>) -> Option<&'a str> {
    (!raw.contains(['^', ';']) && stages.contains_key(raw)).then_some(raw)
}

fn stage_list(raw: &str, stages: &HashMap<String, Stage>) -> Option<Vec<String>> {
    let ids: Vec<String> = raw
        .split(';')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    (!ids.is_empty() && ids.iter().all(|id| stages.contains_key(id))).then_some(ids)
}

impl StageEvidenceIndex {
    pub fn build(
        missions: &[ActivityMission],
        medals: &HashMap<String, MedalDefinition>,
        stages: &HashMap<String, Stage>,
    ) -> Self {
        let mut index = Self::default();
        for m in missions {
            let p = &m.param;
            let rule = match (m.template.as_str(), p.len()) {
                ("CompleteAnyStage", 3) => single_stage(&p[1], stages)
                    .zip(parse_state(&p[2]))
                    .map(|(s, st)| (s, st, true)),
                ("CompleteStageAct", 4) => single_stage(&p[1], stages)
                    .zip(parse_state(&p[3]))
                    .map(|(s, st)| (s, st, true)),
                ("CompleteStageAct", 3) => {
                    single_stage(&p[1], stages).map(|s| (s, STATE_PASS, false))
                }
                ("CompleteStageCondition", n) if n >= 3 => single_stage(&p[2], stages)
                    .zip(parse_state(&p[1]))
                    .map(|(s, st)| (s, st, false)),
                _ => None,
            };
            if let Some((stage_id, state, refutes)) = rule {
                index.missions.insert(
                    m.id.clone(),
                    MissionRule {
                        stage_id: stage_id.to_string(),
                        state,
                        refutes,
                    },
                );
            }
        }
        for (id, medal) in medals {
            if !medal.template.starts_with("PassStage") || medal.unlock_param.len() < 2 {
                continue;
            }
            let Some(state) = parse_state(&medal.unlock_param[0]) else {
                continue;
            };
            let Some(list) = stage_list(&medal.unlock_param[1], stages) else {
                continue;
            };
            let counted = medal.template == "PassStageSome"
                && medal
                    .unlock_param
                    .get(2)
                    .and_then(|n| n.parse::<usize>().ok())
                    == Some(list.len());
            index.medals.insert(
                id.clone(),
                MedalRule {
                    stages: list,
                    state,
                    counted,
                },
            );
        }
        for (id, stage) in stages {
            let prereqs: Vec<Prerequisite> = stage
                .unlock_condition
                .iter()
                .filter(|c| stages.contains_key(&c.stage_id))
                .filter_map(|c| {
                    unlock_state(&c.complete_state).map(|state| Prerequisite {
                        stage_id: c.stage_id.clone(),
                        state,
                    })
                })
                .collect();
            if !prereqs.is_empty() {
                index.unlocks.insert(id.clone(), prereqs);
            }
        }
        index
    }

    /// Bounds for every stage the account's surviving records say something
    /// about. `user` is the `user` object of the sync payload.
    pub fn infer(&self, user: &Value) -> HashMap<String, Bounds> {
        let mut bounds: HashMap<String, Bounds> = HashMap::new();
        let mut counted: Vec<(&MedalRule, usize, String)> = Vec::new();

        if let Some(flags) = user.pointer("/status/flags").and_then(Value::as_object) {
            for (key, value) in flags {
                if value.as_i64() != Some(1) {
                    continue;
                }
                let Some(stage_id) = key
                    .strip_prefix("activities/")
                    .and_then(|rest| rest.split_once("/level_"))
                    .and_then(|(_, level)| level.strip_suffix("_end"))
                else {
                    continue;
                };
                bounds
                    .entry(stage_id.to_string())
                    .or_default()
                    .raise(STATE_PASS, "flag");
            }
        }

        if let Some(groups) = user.pointer("/mission/missions").and_then(Value::as_object) {
            for records in groups.values().filter_map(Value::as_object) {
                for (id, rec) in records {
                    let Some(rule) = self.missions.get(id) else {
                        continue;
                    };
                    let source = format!("mission:{id}");
                    let entry = bounds.entry(rule.stage_id.clone()).or_default();
                    match mission_outcome(rec) {
                        MissionOutcome::Done => {
                            entry.raise(rule.state, &source);
                        }
                        MissionOutcome::Untouched if rule.refutes => {
                            entry.lower(rule.state - 1, &source);
                        }
                        _ => {}
                    }
                }
            }
        }

        if let Some(medals) = user.pointer("/medal/medals").and_then(Value::as_object) {
            for (id, rec) in medals {
                let Some(rule) = self.medals.get(id) else {
                    continue;
                };
                let source = format!("medal:{id}");
                let obtained = rec.get("fts").and_then(Value::as_i64).unwrap_or(-1) > 0;
                if obtained {
                    for stage in &rule.stages {
                        bounds
                            .entry(stage.clone())
                            .or_default()
                            .raise(rule.state, &source);
                    }
                    continue;
                }
                if rule.counted {
                    let got = rec.pointer("/val/0/0").and_then(Value::as_u64).unwrap_or(0) as usize;
                    counted.push((rule, got, source));
                }
            }
        }

        for _ in 0..MAX_ROUNDS {
            let mut changed = false;
            for (rule, got, source) in &counted {
                changed |= apply_count(&mut bounds, rule, *got, source);
            }
            changed |= self.apply_unlocks(&mut bounds);
            if !changed {
                break;
            }
        }

        bounds.retain(|_, b| b.informative());
        bounds
    }

    fn apply_unlocks(&self, bounds: &mut HashMap<String, Bounds>) -> bool {
        let mut changed = false;
        for (stage, prereqs) in &self.unlocks {
            let cleared = bounds.get(stage).is_some_and(|b| b.lo >= STATE_PASS);
            let mut locked_by: Option<&str> = None;
            for p in prereqs {
                if cleared {
                    changed |= bounds
                        .entry(p.stage_id.clone())
                        .or_default()
                        .raise(p.state, &format!("unlock:{stage}"));
                }
                if bounds.get(&p.stage_id).is_some_and(|b| b.hi < p.state) {
                    locked_by = Some(&p.stage_id);
                }
            }
            if let Some(gate) = locked_by {
                changed |= bounds
                    .entry(stage.clone())
                    .or_default()
                    .lower(STATE_UNPLAYED, &format!("unlock:{gate}"));
            }
        }
        changed
    }
}

fn apply_count(
    bounds: &mut HashMap<String, Bounds>,
    rule: &MedalRule,
    got: usize,
    source: &str,
) -> bool {
    let reached: Vec<&String> = rule
        .stages
        .iter()
        .filter(|s| bounds.get(*s).is_some_and(|b| b.lo >= rule.state))
        .collect();
    let possible: Vec<&String> = rule
        .stages
        .iter()
        .filter(|s| bounds.get(*s).is_none_or(|b| b.hi >= rule.state))
        .collect();
    let mut changed = false;
    if got == 0 || reached.len() == got {
        for stage in &rule.stages {
            if reached.contains(&stage) {
                continue;
            }
            changed |= bounds
                .entry(stage.clone())
                .or_default()
                .lower(rule.state - 1, source);
        }
    } else if possible.len() == got {
        for stage in possible {
            changed |= bounds
                .entry(stage.clone())
                .or_default()
                .raise(rule.state, source);
        }
    }
    changed
}

enum MissionOutcome {
    Done,
    Untouched,
    Unknown,
}

fn mission_outcome(rec: &Value) -> MissionOutcome {
    let state = rec.get("state").and_then(Value::as_i64).unwrap_or(0);
    let value = rec.pointer("/progress/0/value").and_then(Value::as_i64);
    let target = rec.pointer("/progress/0/target").and_then(Value::as_i64);
    let met = matches!((value, target), (Some(v), Some(t)) if t > 0 && v >= t);
    if state == 3 || met {
        MissionOutcome::Done
    } else if state == 2 && value == Some(0) {
        MissionOutcome::Untouched
    } else {
        MissionOutcome::Unknown
    }
}

/// Add one `{state, stateMax, inferred, evidence}` record per informative
/// stage that has no battle record, returning how many were added.
pub fn fold_into_records(stages: &mut Value, bounds: &HashMap<String, Bounds>) -> usize {
    if !stages.is_object() {
        *stages = json!({});
    }
    let Some(obj) = stages.as_object_mut() else {
        return 0;
    };
    let mut ids: Vec<&String> = bounds.keys().collect();
    ids.sort();
    let mut folded = 0;
    for id in ids {
        if obj.contains_key(id) {
            continue;
        }
        let b = &bounds[id];
        obj.insert(
            id.clone(),
            json!({
                "stageId": id,
                "state": b.lo,
                "stateMax": b.hi,
                "completeTimes": 0,
                "practiceTimes": 0,
                "inferred": true,
                "evidence": b.evidence,
            }),
        );
        folded += 1;
    }
    folded
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gamedata::types::stage::UnlockCondition;

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
}
