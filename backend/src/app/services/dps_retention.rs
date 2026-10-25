//! Retention: how much of a ranked row's damage survives the real enemy
//! roster (phase 4 of docs/dps-refactor.md). Our own definition.
//!
//! ```text
//! retention(row) = sum_p w_p * dps(row, DEF_p, RES_p) / (sum_p w_p * dps(row, 0, 0))
//! ```
//!
//! over the distinct (DEF, RES) pairs `p` of every enemy appearance in a
//! stage, `w_p` the number of (enemy, stage) appearances at that pair, and
//! `dps` the row's `skill_dps` at the REFERENCE context: preset `p1Max`, no
//! buffs, no zones, one target, legacy cycle, default conditionals. It is a
//! property of the (operator, skill, module) row, never of a request's buffs.
//!
//! An appearance takes the DEF / RES of the database level the stage fields
//! (`EnemyDbRefs[].Level`, 1,235 of 18,177 EN refs use level 1 or 2) with the
//! stage's `OverwrittenData` DEF / RES on top when it sets them. Stage runes
//! (stage-wide attribute modifiers) are NOT applied.
//!
//! Computed once per game-data load, in the background, under a CPU permit;
//! a rank request only ever reads the finished table and never waits for it.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, Weak};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::app::cpu;
use crate::app::services::dps::{
    RankPreset, RankRequest, RankResponse, calculate_in, rank_rows_in,
};
use crate::core::gamedata::types::GameData;

/// What the reference context is, for the response.
pub const REFERENCE: &str = "skillDps at preset p1Max, no buffs, no zones, single target, legacy cycle, default conditionals";

/// One distinct (DEF, RES) pair of the roster and its two weights.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RosterPair {
    pub def: f64,
    pub res: f64,
    /// (enemy, stage) appearances at this pair: the score's weight.
    pub stages: u64,
    /// Total spawns over those appearances (`count` per stage summed): the
    /// alternative weight. 0 for a pair only summoned or declared.
    pub spawns: u64,
}

/// Enemies the roster leaves out, by reason.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RetentionExclusion {
    pub reason: String,
    pub enemies: usize,
    /// Stage appearances those enemies had.
    pub appearances: usize,
}

/// The enemy set the score runs over.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RetentionRoster {
    /// Distinct enemies with at least one stage appearance and stats.
    pub enemies: usize,
    /// (enemy, stage) appearances: the total weight.
    pub appearances: usize,
    /// Distinct (DEF, RES) pairs the appearances collapse into.
    pub distinct_pairs: usize,
    /// Sum of the alternative weight (spawn counts).
    #[ts(type = "number")]
    pub total_spawns: u64,
    /// Appearances at a database level other than 0.
    pub nonzero_level: usize,
    /// Appearances whose stage overrides DEF or RES.
    pub overridden: usize,
    /// Appearances with no level file (boss declarations), taken at level 0.
    pub no_level_file: usize,
    /// Appearances whose stage names a level the database lacks, taken at
    /// level 0.
    pub level_fallback: usize,
    pub excluded: Vec<RetentionExclusion>,
}

/// The roster as pairs, plus its summary. Pairs are in (DEF, RES) order.
pub fn enemy_roster(gd: &GameData) -> (RetentionRoster, Vec<RosterPair>) {
    let mut ids: Vec<&String> = gd.enemy_stage_index.keys().collect();
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
        let refs = &gd.enemy_stage_index[id];
        let levels = match gd.enemies.enemy_data.get(id) {
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
            // DEF and RES are never negative in game data; `+ 0.0` folds -0.0
            // into 0.0 so the two share a key.
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

/// The reference context at one enemy.
pub fn reference_context(def: f64, res: f64) -> RankRequest {
    RankRequest {
        defense: Some(def),
        res: Some(res),
        preset: Some(RankPreset::P1Max),
        ..Default::default()
    }
}

/// One row's score.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RetentionRow {
    pub operator_id: String,
    pub skill_index: i32,
    pub module_index: i32,
    /// `skillDps` at DEF 0 RES 0, the denominator's per-enemy value.
    pub base_dps: f64,
    /// Weighted by stage appearances. `null` when `baseDps` is 0 or a pair
    /// failed to compute.
    pub retention: Option<f64>,
    /// The same, weighted by spawn counts instead (the alternative weight).
    pub retention_by_spawns: Option<f64>,
}

/// The finished table for one game-data load.
#[derive(Debug, Clone)]
pub struct RetentionTable {
    pub roster: RetentionRoster,
    pub rows: Vec<RetentionRow>,
    /// Wall time of the computation, milliseconds.
    pub compute_ms: f64,
    by_key: HashMap<(String, i32, i32), Option<f64>>,
}

impl RetentionTable {
    pub fn get(&self, operator_id: &str, skill: i32, module: i32) -> Option<f64> {
        self.by_key
            .get(&(operator_id.to_owned(), skill, module))
            .copied()
            .flatten()
    }
}

/// Every row's retention, synchronously (minutes in a debug build: run it on
/// the blocking pool). Rows are the reference-context ranking's rows; each
/// pair evaluation is `calculate_in`, the single-call path.
pub fn compute_retention(gd: &Arc<GameData>) -> RetentionTable {
    let started = Instant::now();
    let (roster, pairs) = enemy_roster(gd);
    let base = rank_rows_in(gd, &reference_context(0.0, 0.0));
    let w_stages: f64 = pairs.iter().map(|p| p.stages as f64).sum();
    let w_spawns: f64 = pairs.iter().map(|p| p.spawns as f64).sum();
    let contexts: Vec<RankRequest> = pairs
        .iter()
        .map(|p| reference_context(p.def, p.res))
        .collect();

    // Boss declarations alone are not a roster: with no level file, no score.
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
                let Ok(r) = calculate_in(gd, req) else {
                    ok = false;
                    break;
                };
                sum_stages += p.stages as f64 * r.skill_dps;
                sum_spawns += p.spawns as f64 * r.skill_dps;
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

/// Where a load's table is.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, ToSchema, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RetentionStatus {
    /// Being computed, or about to be: `retention` is `null` on every row.
    #[default]
    Pending,
    Ready,
    /// The computation panicked; it is not retried for this load.
    Failed,
    /// The load has no level files (every appearance is a boss declaration
    /// without one), so there is no roster to score against; every row is
    /// `null`.
    Unavailable,
}

#[derive(Clone)]
enum Slot {
    Pending,
    Ready(Arc<RetentionTable>),
    Failed,
    Unavailable(Arc<RetentionTable>),
}

static SLOTS: Mutex<Vec<(Weak<GameData>, Slot)>> = Mutex::new(Vec::new());

fn slots() -> std::sync::MutexGuard<'static, Vec<(Weak<GameData>, Slot)>> {
    SLOTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The table for this load if it is finished; otherwise starts it in the
/// background (once per load) and returns at once. Never waits.
pub fn table_or_start(gd: &Arc<GameData>) -> (RetentionStatus, Option<Arc<RetentionTable>>) {
    let owner = Arc::downgrade(gd);
    {
        let mut s = slots();
        s.retain(|(o, _)| o.strong_count() > 0);
        if let Some((_, slot)) = s.iter().find(|(o, _)| o.ptr_eq(&owner)) {
            return match slot {
                Slot::Pending => (RetentionStatus::Pending, None),
                Slot::Ready(t) => (RetentionStatus::Ready, Some(Arc::clone(t))),
                Slot::Failed => (RetentionStatus::Failed, None),
                Slot::Unavailable(t) => (RetentionStatus::Unavailable, Some(Arc::clone(t))),
            };
        }
        s.push((owner, Slot::Pending));
    }
    let gd = Arc::clone(gd);
    tokio::spawn(async move {
        let weak = Arc::downgrade(&gd);
        let outcome = cpu::run("dps_retention", move || compute_retention(&gd)).await;
        let mut s = slots();
        let Some(i) = s.iter().position(|(o, _)| o.ptr_eq(&weak)) else {
            return;
        };
        match outcome {
            Ok(table) => {
                tracing::info!(
                    rows = table.rows.len(),
                    pairs = table.roster.distinct_pairs,
                    ms = table.compute_ms,
                    "dps retention computed"
                );
                s[i].1 = if table.roster.appearances > table.roster.no_level_file {
                    Slot::Ready(Arc::new(table))
                } else {
                    tracing::warn!("dps retention: no level files in this load, no roster");
                    Slot::Unavailable(Arc::new(table))
                };
            }
            // No permit within the wait: forget the slot so the next
            // request starts it again.
            Err(crate::app::error::ApiError::ServiceUnavailable) => {
                s.remove(i);
            }
            Err(e) => {
                tracing::error!(error = %e, "dps retention failed");
                s[i].1 = Slot::Failed;
            }
        }
    });
    (RetentionStatus::Pending, None)
}

/// Fills `retention` on a rank response's rows from the finished table, and
/// says whether it is finished. A row the table does not hold stays `null`.
pub fn attach(gd: &Arc<GameData>, response: &mut RankResponse) {
    let (status, table) = table_or_start(gd);
    response.retention_status = status;
    if let Some(table) = table {
        for row in &mut response.rows {
            row.retention = table.get(&row.operator_id, row.skill_index, row.module_index);
        }
    }
}

/// `GET /dps/retention`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RetentionResponse {
    pub status: RetentionStatus,
    /// The game-data identity (a hash of the operator table; the enemy
    /// tables are not in it, the in-process table is keyed by the load).
    pub data: String,
    /// The context every `dps` value is computed at.
    pub reference: String,
    /// `null` until `status` is `ready`.
    pub roster: Option<RetentionRoster>,
    /// Wall time of the computation, milliseconds; `null` until ready.
    pub compute_ms: Option<f64>,
    /// Every ranked (operator, skill, module) of the reference context, in
    /// operator, skill, module order. Empty until ready.
    pub rows: Vec<RetentionRow>,
}

pub fn response(gd: &Arc<GameData>, data: &str) -> RetentionResponse {
    let (status, table) = table_or_start(gd);
    RetentionResponse {
        status,
        data: data.to_owned(),
        reference: REFERENCE.to_owned(),
        roster: table.as_ref().map(|t| t.roster.clone()),
        compute_ms: table.as_ref().map(|t| t.compute_ms),
        rows: table.map(|t| t.rows.clone()).unwrap_or_default(),
    }
}
