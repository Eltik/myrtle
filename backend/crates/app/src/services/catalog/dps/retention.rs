use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

use myrtle_gamedata::model::GameModel;
use serde::{Deserialize, Serialize};

use super::rank::{RankPreset, RankRequest, RankResponse, rank_rows_in};
use super::{Data, calculate_in};

pub const REFERENCE: &str = "skillDps at preset p1Max, no buffs, no zones, single target, legacy cycle, default conditionals";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RosterPair {
    pub def: f64,
    pub res: f64,
    pub stages: u64,
    pub spawns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RetentionExclusion {
    pub reason: String,
    pub enemies: usize,
    pub appearances: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionRoster {
    pub enemies: usize,
    pub appearances: usize,
    pub distinct_pairs: usize,
    pub total_spawns: u64,
    pub nonzero_level: usize,
    pub overridden: usize,
    pub no_level_file: usize,
    pub level_fallback: usize,
    pub excluded: Vec<RetentionExclusion>,
}

#[must_use]
pub fn enemy_roster(model: &GameModel) -> (RetentionRoster, Vec<RosterPair>) {
    let mut ids: Vec<&String> = model.enemy_stage_index.keys().collect();
    ids.sort();
    let mut pairs: BTreeMap<(u64, u64), RosterPair> = BTreeMap::new();
    let mut excluded: BTreeMap<&'static str, (usize, usize)> = BTreeMap::new();
    let mut roster = RetentionRoster {
        enemies: 0,
        appearances: 0,
        distinct_pairs: 0,
        total_spawns: 0,
        nonzero_level: 0,
        overridden: 0,
        no_level_file: 0,
        level_fallback: 0,
        excluded: Vec::new(),
    };
    for id in ids {
        let refs = &model.enemy_stage_index[id];
        let levels = match model.enemies.enemy_data.get(id) {
            None => {
                let e = excluded.entry("no handbook entry").or_default();
                e.0 += 1;
                e.1 += refs.len();
                continue;
            }
            Some(enemy) => match enemy.stats.as_ref().filter(|s| !s.levels.is_empty()) {
                None => {
                    let e = excluded.entry("no database stats").or_default();
                    e.0 += 1;
                    e.1 += refs.len();
                    continue;
                }
                Some(stats) => &stats.levels,
            },
        };
        roster.enemies += 1;
        for r in refs {
            let battle = r.battle.unwrap_or_default();
            if r.battle.is_none() {
                roster.no_level_file += 1;
            }
            if battle.level != 0 {
                roster.nonzero_level += 1;
            }
            let at = levels
                .iter()
                .find(|l| l.level == battle.level)
                .unwrap_or_else(|| {
                    roster.level_fallback += 1;
                    &levels[0]
                });
            if battle.def.is_some() || battle.res.is_some() {
                roster.overridden += 1;
            }
            let def = battle.def.unwrap_or_else(|| f64::from(at.attributes.def));
            let res = battle.res.unwrap_or(at.attributes.magic_resistance);
            let (def, res) = (def + 0.0, res + 0.0);
            let p = pairs
                .entry((def.to_bits(), res.to_bits()))
                .or_insert(RosterPair {
                    def,
                    res,
                    stages: 0,
                    spawns: 0,
                });
            p.stages += 1;
            p.spawns += u64::from(r.count);
            roster.appearances += 1;
            roster.total_spawns += u64::from(r.count);
        }
    }
    let mut pairs: Vec<RosterPair> = pairs.into_values().collect();
    pairs.sort_by(|a, b| a.def.total_cmp(&b.def).then(a.res.total_cmp(&b.res)));
    roster.distinct_pairs = pairs.len();
    roster.excluded = excluded
        .into_iter()
        .map(|(reason, (enemies, appearances))| RetentionExclusion {
            reason: reason.to_owned(),
            enemies,
            appearances,
        })
        .collect();
    (roster, pairs)
}

#[must_use]
pub fn reference_context(def: f64, res: f64) -> RankRequest {
    RankRequest {
        defense: Some(def),
        res: Some(res),
        preset: Some(RankPreset::P1Max),
        ..RankRequest::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetentionRow {
    pub operator_id: String,
    pub skill_index: i32,
    pub module_index: i32,
    pub base_dps: f64,
    pub retention: Option<f64>,
    pub retention_by_spawns: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct RetentionTable {
    pub roster: RetentionRoster,
    pub rows: Vec<RetentionRow>,
    pub compute_ms: f64,
    by_key: HashMap<(String, i32, i32), Option<f64>>,
}

impl RetentionTable {
    #[must_use]
    pub fn get(&self, operator_id: &str, skill: i32, module: i32) -> Option<f64> {
        self.by_key
            .get(&(operator_id.to_owned(), skill, module))
            .copied()
            .flatten()
    }

    #[must_use]
    pub const fn scorable(&self) -> bool {
        self.roster.appearances > self.roster.no_level_file
    }
}

#[must_use]
pub fn compute_retention(data: &Data) -> RetentionTable {
    let started = Instant::now();
    let (roster, pairs) = enemy_roster(&data.snapshot.model);
    let base = rank_rows_in(data, &reference_context(0.0, 0.0));
    let w_stages: f64 = pairs.iter().map(|p| p.stages as f64).sum();
    let w_spawns: f64 = pairs.iter().map(|p| p.spawns as f64).sum();
    let contexts: Vec<RankRequest> = pairs
        .iter()
        .map(|p| reference_context(p.def, p.res))
        .collect();

    let scorable = roster.appearances > roster.no_level_file;
    let mut rows = Vec::with_capacity(base.rows.len());
    for row in &base.rows {
        let base_dps = row.skill_dps;
        let mut out = RetentionRow {
            operator_id: row.operator_id.clone(),
            skill_index: row.skill_index,
            module_index: row.module_index,
            base_dps,
            retention: None,
            retention_by_spawns: None,
        };
        if base_dps != 0.0 && scorable {
            let mut sum_stages = 0.0;
            let mut sum_spawns = 0.0;
            let mut ok = true;
            for (p, ctx) in pairs.iter().zip(&contexts) {
                let req = ctx.row_request(&row.operator_id, row.skill_index, row.module_index);
                let Ok(r) = calculate_in(data, req) else {
                    ok = false;
                    break;
                };
                sum_stages = (p.stages as f64).mul_add(r.skill_dps, sum_stages);
                sum_spawns = (p.spawns as f64).mul_add(r.skill_dps, sum_spawns);
            }
            if ok {
                out.retention = Some(sum_stages / (w_stages * base_dps));
                out.retention_by_spawns =
                    (w_spawns > 0.0).then(|| sum_spawns / (w_spawns * base_dps));
            }
        }
        rows.push(out);
    }
    let by_key = rows
        .iter()
        .map(|r| {
            (
                (r.operator_id.clone(), r.skill_index, r.module_index),
                r.retention,
            )
        })
        .collect();
    RetentionTable {
        roster,
        rows,
        compute_ms: started.elapsed().as_secs_f64() * 1000.0,
        by_key,
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RetentionStatus {
    #[default]
    Pending,
    Ready,
    Failed,
    Unavailable,
}

impl RetentionStatus {
    pub(super) const fn of(table: Option<&RetentionTable>) -> Self {
        match table {
            None => Self::Pending,
            Some(t) if t.scorable() => Self::Ready,
            Some(_) => Self::Unavailable,
        }
    }
}

pub fn attach(data: &Data, response: &mut RankResponse) {
    let table = data.retention_table();
    response.retention_status = RetentionStatus::of(table.as_deref());
    if let Some(table) = table {
        for row in &mut response.rows {
            row.retention = table.get(&row.operator_id, row.skill_index, row.module_index);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionResponse {
    pub status: RetentionStatus,
    pub data: String,
    pub reference: String,
    pub roster: Option<RetentionRoster>,
    pub compute_ms: Option<f64>,
    pub rows: Vec<RetentionRow>,
}

impl RetentionResponse {
    pub(super) fn new(
        data: &Data,
        status: RetentionStatus,
        table: Option<&RetentionTable>,
    ) -> Self {
        Self {
            status,
            data: data.identity().to_owned(),
            reference: REFERENCE.to_owned(),
            roster: table.map(|t| t.roster.clone()),
            compute_ms: table.map(|t| t.compute_ms),
            rows: table.map(|t| t.rows.clone()).unwrap_or_default(),
        }
    }
}
