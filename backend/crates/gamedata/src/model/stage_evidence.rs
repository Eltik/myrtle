use std::collections::HashMap;

use myrtle_gamedata_raw::tables::{
    activity::ActivityMission, medal::MedalDefinition, stage::Stage,
};
use serde::Serialize;
use serde_json::{Value, json};

pub const STATE_UNPLAYED: i16 = 0;
pub const STATE_PASS: i16 = 2;
pub const STATE_COMPLETE: i16 = 3;

const MAX_ROUNDS: usize = 8;

#[derive(Debug, Clone, Serialize)]
struct MissionRule {
    stage_id: String,
    state: i16,
    refutes: bool,
}

#[derive(Debug, Clone, Serialize)]
struct MedalRule {
    stages: Vec<String>,
    state: i16,
    counted: bool,
}

#[derive(Debug, Clone, Serialize)]
struct Prerequisite {
    stage_id: String,
    state: i16,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct StageEvidenceIndex {
    missions: HashMap<String, MissionRule>,
    medals: HashMap<String, MedalRule>,
    unlocks: HashMap<String, Vec<Prerequisite>>,
}

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
    #[must_use]
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
    #[must_use]
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
