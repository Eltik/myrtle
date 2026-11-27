use std::collections::{HashMap, HashSet};

use super::control::CcCondition;
use super::facilities::effective_facility_counts;
use super::live::stationed_seats;
use super::{CANDIDATE_LIMIT, build_op_index};
use crate::{
    registry::BuffResolutionStrategy,
    rotation::sustain_sim::sustains_hours,
    scoring::clause::clauses_from_strategy,
    types::{OperatorBaseProfile, UserBuilding, UserRoom},
};
use myrtle_gamedata_raw::tables::building::{Buff, BuildingDataFile};

/// True for a fixed synergy squad whose operators depend on each other
pub fn team_is_locked(
    ops: &[String],
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> bool {
    if ops.len() < 2 {
        return false;
    }
    let present: HashSet<&str> = ops.iter().map(String::as_str).collect();

    let has_nullifier = ops.iter().any(|id| {
        building_data.chars.get(id).is_some_and(|c| {
            c.buff_char.iter().flat_map(|s| &s.buff_data).any(|e| {
                matches!(
                    registry.get(&e.buff_id),
                    Some(BuffResolutionStrategy::NullifyTeammatesSelfScaling { .. })
                )
            })
        })
    });
    if has_nullifier {
        return true;
    }

    for id in ops {
        for strategy in profile_strategies(id, room_type, formula_type, registry, building_data) {
            if let BuffResolutionStrategy::ConditionalOnTeammate {
                required_char_id: Some(req),
                ..
            } = strategy
                && present.contains(req.as_str())
            {
                return true;
            }
        }
    }
    false
}

pub(super) fn profile_strategies<'a>(
    char_id: &str,
    room_type: &'a str,
    formula_type: Option<&'a str>,
    registry: &'a HashMap<String, BuffResolutionStrategy>,
    building_data: &'a BuildingDataFile,
) -> Vec<&'a BuffResolutionStrategy> {
    let Some(bc) = building_data.chars.get(char_id) else {
        return Vec::new();
    };
    bc.buff_char
        .iter()
        .filter_map(|s| s.buff_data.last())
        .filter_map(|e| {
            applicable_strategy(&e.buff_id, room_type, formula_type, registry, building_data)
        })
        .collect()
}

pub(super) fn buff_applies(buff: &Buff, room_type: &str, formula_type: Option<&str>) -> bool {
    if buff.room_type != room_type {
        return false;
    }
    if let Some(formula) = formula_type
        && !buff.targets.is_empty()
        && !buff.targets.iter().any(|t| t == formula)
    {
        return false;
    }
    true
}

pub fn is_generic_targets(targets: &[String]) -> bool {
    targets.iter().any(|t| t == "F_GOLD") && targets.iter().any(|t| t == "F_EXP")
}

pub(super) fn op_is_formula_specialist(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    building_data: &BuildingDataFile,
) -> bool {
    let Some(formula) = formula_type else {
        return false;
    };
    op.available_buffs.iter().any(|b| {
        building_data.buffs.get(b).is_some_and(|buff| {
            buff_applies(buff, room_type, formula_type)
                && buff.targets.iter().any(|t| t == formula)
                && !is_generic_targets(&buff.targets)
        })
    })
}

pub(super) fn applicable_strategy<'a>(
    buff_id: &str,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &'a HashMap<String, BuffResolutionStrategy>,
    building_data: &'a BuildingDataFile,
) -> Option<&'a BuffResolutionStrategy> {
    let buff = building_data.buffs.get(buff_id)?;
    buff_applies(buff, room_type, formula_type)
        .then(|| registry.get(buff_id))
        .flatten()
}

pub(super) fn applicable_strategies<'a>(
    op: &'a OperatorBaseProfile,
    room_type: &'a str,
    formula_type: Option<&'a str>,
    registry: &'a HashMap<String, BuffResolutionStrategy>,
    building_data: &'a BuildingDataFile,
) -> impl Iterator<Item = &'a BuffResolutionStrategy> {
    op.available_buffs.iter().filter_map(move |b| {
        applicable_strategy(b, room_type, formula_type, registry, building_data)
    })
}

pub(super) fn op_surviving_order_value(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> f64 {
    crate::scoring::ledger::op_surviving_order_value(
        op,
        room_type,
        formula_type,
        registry,
        building_data,
        &[],
    )
}

pub(super) fn op_surviving_order_value_among(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    companions: &[crate::registry::OrderEffect],
) -> f64 {
    crate::scoring::ledger::op_surviving_order_value(
        op,
        room_type,
        formula_type,
        registry,
        building_data,
        companions,
    )
}

/// One scored team combination for a room type/formula
/// rotation selector packs into disjoint team sets.
#[derive(Clone, Debug)]
pub struct CandidateTeam {
    pub(crate) ops: Vec<String>,
    pub(crate) speed: f64,
    pub(crate) value: f64,
    /// The gold-throughput part of `value`
    pub(crate) gold: f64,
    /// Trading post order limit with this team
    pub(crate) order_limit: Option<i32>,
    pub(crate) score: f64,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn best_team_for_room(
    room: &UserRoom,
    formula_type: Option<&str>,
    operators: &[OperatorBaseProfile],
    already_assigned: &HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    max_slots: i32,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
    cap_aware: bool,
) -> (Vec<String>, f64, f64, f64, Option<i32>) {
    enumerate_candidate_teams(
        &room.room_type,
        room.level,
        formula_type,
        operators,
        already_assigned,
        registry,
        building_data,
        facility_counts,
        total_dorm_levels,
        max_slots,
        cc_conditions,
        morale_drains,
        cap_aware,
        CANDIDATE_LIMIT,
        false,
        0,
    )
    .into_iter()
    .find(|c| c.score > 0.0)
    .map_or((Vec::new(), 0.0, 0.0, 0.0, None), |c| {
        (c.ops, c.speed, c.value, c.gold, c.order_limit)
    })
}

/// The operators a room's team search may draw from
#[allow(clippy::too_many_arguments)]
pub fn candidate_pool<'a>(
    room_type: &str,
    formula_type: Option<&str>,
    operators: &'a [OperatorBaseProfile],
    already_assigned: &HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    morale_drains: &HashMap<String, f64>,
    include_automation: bool,
    sustain_hours: u32,
) -> Vec<&'a OperatorBaseProfile> {
    operators
        .iter()
        .filter(|op| !already_assigned.contains(&op.char_id))
        .filter(|op| {
            op.available_buffs.iter().any(|b| {
                building_data
                    .buffs
                    .get(b)
                    .is_some_and(|buff| buff_applies(buff, room_type, formula_type))
            })
        })
        .filter(|op| include_automation || !has_automation_buff(op, registry))
        .filter(|op| {
            sustain_hours == 0 || sustains_hours(op, morale_drains, f64::from(sustain_hours))
        })
        .collect()
}

/// Every scored team combination for a room type/formula, best first
#[allow(clippy::too_many_arguments)]
pub fn enumerate_candidate_teams(
    room_type: &str,
    room_level: i32,
    formula_type: Option<&str>,
    operators: &[OperatorBaseProfile],
    already_assigned: &HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    max_slots: i32,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
    cap_aware: bool,
    pool_limit: usize,
    include_automation: bool,
    sustain_hours: u32,
) -> Vec<CandidateTeam> {
    if max_slots <= 0 {
        return Vec::new();
    }
    let max_slots = max_slots as usize;
    let op_index = build_op_index(operators);

    let candidates = candidate_pool(
        room_type,
        formula_type,
        operators,
        already_assigned,
        registry,
        building_data,
        morale_drains,
        include_automation,
        sustain_hours,
    );

    if candidates.is_empty() {
        return Vec::new();
    }

    let mut enabler_boost: HashMap<String, f64> = HashMap::new();
    for op in &candidates {
        for b in &op.available_buffs {
            if let Some(BuffResolutionStrategy::ConditionalOnTeammate {
                required_char_id: Some(req),
                efficiency,
                ..
            }) = registry.get(b)
            {
                let slot = enabler_boost.entry(req.clone()).or_insert(0.0);
                *slot = slot.max(*efficiency);
            }
        }
    }

    let capacity_rate = {
        use crate::scoring::clause::{ClauseKind, Metric};
        let speed_metric = Metric::speed_for_room(room_type);
        candidates
            .iter()
            .flat_map(|op| op.available_buffs.iter())
            .filter_map(|b| {
                let buff = building_data.buffs.get(b)?;
                (buff.room_type == room_type).then_some(())?;
                let strategy = registry.get(b)?;
                Some(clauses_from_strategy(b, buff, strategy))
            })
            .flatten()
            .filter_map(|c| match c.kind {
                ClauseKind::ScalingPeerMetric {
                    metric: Metric::CapacityLimit,
                    step,
                    ..
                } if c.metric == speed_metric && step > 0.0 => Some(c.value / step),
                _ => None,
            })
            .fold(0.0_f64, f64::max)
    };
    let capacity_boost = |op: &OperatorBaseProfile| -> f64 {
        if capacity_rate <= 0.0 {
            return 0.0;
        }
        let alone: HashSet<String> = HashSet::from([op.char_id.clone()]);
        capacity_rate
            * f64::from(compute_order_limit(
                op,
                room_type,
                formula_type,
                registry,
                building_data,
                &alone,
            ))
    };

    let has_nullifier = candidates
        .iter()
        .any(|op| op_is_nullifier(op, room_type, formula_type, registry, building_data));

    let companions = crate::scoring::ledger::roster_order_effects(
        &candidates,
        room_type,
        formula_type,
        registry,
        building_data,
    );
    let pools: Vec<(Vec<&OperatorBaseProfile>, bool)> = if has_nullifier {
        let restricted: Vec<&OperatorBaseProfile> = candidates
            .iter()
            .copied()
            .filter(|op| {
                op_is_nullifier(op, room_type, formula_type, registry, building_data)
                    || op_surviving_order_value(
                        op,
                        room_type,
                        formula_type,
                        registry,
                        building_data,
                    ) > 0.0
            })
            .collect();
        let without: Vec<&OperatorBaseProfile> = candidates
            .iter()
            .copied()
            .filter(|op| !op_is_nullifier(op, room_type, formula_type, registry, building_data))
            .collect();
        vec![(restricted, true), (without, false)]
    } else {
        vec![(candidates, false)]
    };

    let mut teams: Vec<CandidateTeam> = Vec::new();
    let mut seen: HashSet<Vec<String>> = HashSet::new();
    for (pool_candidates, nullifier_pool) in pools {
        let mut ranked: Vec<(&OperatorBaseProfile, f64, bool)> = pool_candidates
            .iter()
            .map(|op| {
                let value_boost = if nullifier_pool {
                    op_surviving_order_value_among(
                        op,
                        room_type,
                        formula_type,
                        registry,
                        building_data,
                        &companions,
                    ) * 5.0
                } else {
                    0.0
                };
                let bound = optimistic_bound(
                    op,
                    room_type,
                    formula_type,
                    registry,
                    building_data,
                    facility_counts,
                    total_dorm_levels,
                    max_slots,
                    &companions,
                ) + enabler_boost.get(&op.char_id).copied().unwrap_or(0.0)
                    + capacity_boost(op)
                    + value_boost
                    + cc_conditions
                        .iter()
                        .filter(|c| c.per_operator)
                        .map(|c| c.contribution(room_type, &[*op], formula_type).max(0.0))
                        .sum::<f64>();
                let specialist =
                    op_is_formula_specialist(op, room_type, formula_type, building_data);
                (*op, bound, specialist)
            })
            .collect();
        ranked.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.2.cmp(&a.2))
        });
        ranked.truncate(pool_limit);

        let pool: Vec<String> = ranked.iter().map(|(op, _, _)| op.char_id.clone()).collect();

        for combo in combinations(&pool, max_slots) {
            let mut key = combo.clone();
            key.sort();
            if !seen.insert(key) {
                continue;
            }
            let totals = compute_team_totals(
                &combo,
                room_type,
                formula_type,
                Some(room_level),
                &op_index,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
                morale_drains,
                cc_conditions,
            );
            let (mut speed, value, gold) = (
                totals.speed_pct,
                totals.order_value_pct,
                totals.order_gold_pct,
            );
            if nullifier_pool {
                let empty_seats = max_slots.saturating_sub(combo.len()) as f64;
                if empty_seats > 0.0 {
                    let per_teammate = combo
                        .iter()
                        .filter_map(|id| op_index.get(id.as_str()).copied())
                        .flat_map(|op| {
                            applicable_strategies(
                                op,
                                room_type,
                                formula_type,
                                registry,
                                building_data,
                            )
                        })
                        .filter_map(|s| match s {
                            BuffResolutionStrategy::NullifyTeammatesSelfScaling {
                                per_teammate_pct,
                            } => Some(*per_teammate_pct),
                            _ => None,
                        })
                        .fold(0.0, f64::max);
                    speed += per_teammate * empty_seats;
                }
            }
            let mut score = room_search_score(room_type, speed, value);
            score *= crate::scoring::yield_model::trading_cap_factor(
                room_type,
                room_level,
                speed,
                combo.len(),
                totals.order_limit,
            );
            if cap_aware {
                let cap = team_capacity_bonus(
                    &combo,
                    room_type,
                    formula_type,
                    &op_index,
                    registry,
                    building_data,
                );
                score *= factory_cap_factor(building_data, formula_type, room_level, speed, cap);
            }
            teams.push(CandidateTeam {
                ops: combo,
                speed,
                value,
                gold,
                order_limit: totals.order_limit,
                score,
            });
        }
    }
    teams.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    teams
}

#[allow(clippy::too_many_arguments)]
pub(super) fn best_automation_team(
    room: &UserRoom,
    formula_type: Option<&str>,
    operators: &[OperatorBaseProfile],
    already_assigned: &HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    max_slots: i32,
) -> (Vec<String>, f64) {
    let facility_value = |op: &OperatorBaseProfile| {
        score_operator_facility_only(
            op,
            &room.room_type,
            formula_type,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
        )
    };
    let mut scored: Vec<(&OperatorBaseProfile, f64, bool)> = operators
        .iter()
        .filter(|op| !already_assigned.contains(&op.char_id))
        .filter(|op| has_facility_scaled_buff(op, registry))
        .map(|op| (op, facility_value(op), has_automation_buff(op, registry)))
        .filter(|(_, v, _)| *v > 0.0)
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let Some(lead) = scored.iter().position(|(_, _, nullifies)| *nullifies) else {
        return (Vec::new(), f64::NEG_INFINITY);
    };
    let leader = scored.remove(lead);
    let mut ops = vec![leader.0.char_id.clone()];
    let mut total = leader.1;
    for (op, val, _) in scored.into_iter().take(max_slots.max(1) as usize - 1) {
        ops.push(op.char_id.clone());
        total += val;
    }
    (ops, total)
}

pub fn has_facility_scaled_buff(
    op: &OperatorBaseProfile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> bool {
    op.available_buffs.iter().any(|b| {
        matches!(
            registry.get(b),
            Some(BuffResolutionStrategy::FacilityCountScaling { .. })
        )
    })
}

pub fn has_automation_buff(
    op: &OperatorBaseProfile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> bool {
    op.available_buffs.iter().any(|b| {
        matches!(
            registry.get(b),
            Some(BuffResolutionStrategy::FacilityCountScaling {
                nullifies_others: true,
                ..
            })
        )
    })
}

pub fn op_is_nullifier(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> bool {
    applicable_strategies(op, room_type, formula_type, registry, building_data).any(|s| {
        matches!(
            s,
            BuffResolutionStrategy::NullifyTeammatesSelfScaling { .. }
        )
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn optimistic_bound(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    max_slots: usize,
    companions: &[crate::registry::OrderEffect],
) -> f64 {
    crate::scoring::ledger::op_optimistic_bound(
        op,
        room_type,
        formula_type,
        registry,
        building_data,
        facility_counts,
        total_dorm_levels,
        max_slots,
        companions,
    )
}

pub(super) fn combinations(pool: &[String], max_size: usize) -> Vec<Vec<String>> {
    let mut result = Vec::new();
    let max_size = max_size.min(pool.len());
    let mut current = Vec::new();
    for size in 1..=max_size {
        combos_of_size(pool, size, 0, &mut current, &mut result);
    }
    result
}

pub(super) fn combos_of_size(
    pool: &[String],
    size: usize,
    start: usize,
    current: &mut Vec<String>,
    out: &mut Vec<Vec<String>>,
) {
    if current.len() == size {
        out.push(current.clone());
        return;
    }
    let need = size - current.len();
    if pool.len() - start < need {
        return;
    }
    for i in start..pool.len() {
        current.push(pool[i].clone());
        combos_of_size(pool, size, i + 1, current, out);
        current.pop();
    }
}

pub(super) fn score_operator_facility_only(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
) -> f64 {
    crate::scoring::ledger::op_facility_only_value(
        op,
        room_type,
        formula_type,
        registry,
        building_data,
        facility_counts,
        total_dorm_levels,
    )
}

#[allow(clippy::cast_possible_truncation)]
pub(super) fn compute_order_limit(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    present: &HashSet<String>,
) -> i32 {
    crate::scoring::ledger::op_capacity_limit(
        op,
        room_type,
        formula_type,
        registry,
        building_data,
        present,
    )
    .round() as i32
}

/// Returns `(speed_pct, value_pct)`
#[allow(clippy::too_many_arguments)]
pub fn compute_team_efficiency(
    member_ids: &[String],
    room_type: &str,
    formula_type: Option<&str>,
    room_level: Option<i32>,
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    morale_drains: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
) -> (f64, f64) {
    let totals = compute_team_totals(
        member_ids,
        room_type,
        formula_type,
        room_level,
        op_index,
        registry,
        building_data,
        facility_counts,
        total_dorm_levels,
        morale_drains,
        cc_conditions,
    );
    (totals.speed_pct, totals.order_value_pct)
}

#[allow(clippy::too_many_arguments)]
pub fn compute_team_totals(
    member_ids: &[String],
    room_type: &str,
    formula_type: Option<&str>,
    room_level: Option<i32>,
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    morale_drains: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
) -> crate::scoring::ledger::RoomTotals {
    let _ = morale_drains;
    crate::scoring::ledger::score_room(&crate::scoring::ledger::RoomEval {
        member_ids,
        room_type,
        formula_type,
        op_index,
        registry,
        building_data,
        facility_counts,
        total_dorm_levels,
        cc_conditions,
        room_level,
        deployed_work_area: None,
    })
}

/// Score an arbitrary team with the room search's speed/value objective
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn team_value(
    team: &[String],
    room_type: &str,
    formula_type: Option<&str>,
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> f64 {
    let op_index = build_op_index(operators);
    let facility_counts = effective_facility_counts(
        building,
        operators,
        registry,
        building_data,
        &stationed_seats(building, None),
    );
    let (speed, value) = compute_team_efficiency(
        team,
        room_type,
        formula_type,
        None,
        &op_index,
        registry,
        building_data,
        &facility_counts,
        building.total_dorm_levels(),
        morale_drains,
        &[],
    );
    room_search_score(room_type, speed, value)
}

/// The optimizer's per-room objective
pub fn room_search_score(room_type: &str, speed_pct: f64, value_pct: f64) -> f64 {
    let speed_mult = 1.0 + speed_pct / 100.0;
    if room_type == "TRADING" {
        speed_mult * (1.0 + value_pct / 100.0)
    } else {
        speed_mult
    }
}

// Cap-aware sustained model: AFK product-buffer stall
/// Assumed days between product collections for the sustained metric
pub const SUSTAINED_AFK_DAYS: f64 = 1.5;
pub(super) const SECONDS_PER_DAY: f64 = 86_400.0;

pub(super) fn gold_products_per_day(building_data: &BuildingDataFile) -> f64 {
    building_data
        .manufact_formulas
        .get("4")
        .map_or(20.0, |f| SECONDS_PER_DAY / f.cost_point as f64)
}

pub fn gold_base_capacity(building_data: &BuildingDataFile, level: i32) -> f64 {
    let phases = &building_data.manufact_data.phases;
    let idx = (level.max(1) as usize - 1).min(phases.len().saturating_sub(1));
    phases
        .get(idx)
        .map_or(54.0, |p| f64::from(p.output_capacity))
}

/// Fraction of a gold factory's peak output that survives the AFK buffer stall
pub fn factory_cap_factor(
    building_data: &BuildingDataFile,
    formula: Option<&str>,
    level: i32,
    efficiency: f64,
    capacity_bonus: i32,
) -> f64 {
    if formula != Some("F_GOLD") {
        return 1.0;
    }
    let buffer = gold_base_capacity(building_data, level) + f64::from(capacity_bonus.max(0));
    let peak_over_window =
        gold_products_per_day(building_data) * (1.0 + efficiency / 100.0) * SUSTAINED_AFK_DAYS;
    if peak_over_window <= 0.0 {
        return 1.0;
    }
    (buffer / peak_over_window).min(1.0)
}

pub(super) fn team_capacity_bonus(
    team: &[String],
    room_type: &str,
    formula_type: Option<&str>,
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> i32 {
    let present: HashSet<String> = team.iter().cloned().collect();
    team.iter()
        .filter_map(|id| op_index.get(id.as_str()).copied())
        .map(|op| {
            compute_order_limit(
                op,
                room_type,
                formula_type,
                registry,
                building_data,
                &present,
            )
        })
        .sum()
}

/// The gold-factory counts a planner tries for `factory_rooms`
pub fn gold_split_candidates(
    factory_rooms: &[&&UserRoom],
    fed: usize,
    frozen_gold: usize,
) -> Vec<usize> {
    let num_factories = factory_rooms.len();
    let max_gold = (fed + 1).min(num_factories);
    let mut splits: Vec<usize> = (fed..=max_gold).collect();
    if fed > 0 && frozen_gold == 0 && player_runs_no_gold(factory_rooms) {
        splits.insert(0, 0);
    }
    splits
}

pub(super) fn player_runs_no_gold(factory_rooms: &[&&UserRoom]) -> bool {
    let mut recorded = factory_rooms
        .iter()
        .filter_map(|r| r.current_formula.as_deref())
        .peekable();
    recorded.peek().is_some() && recorded.all(|f| f != "F_GOLD")
}
