//! The ledger scorer

use std::collections::{HashMap, HashSet};

use myrtle_gamedata_raw::tables::building::BuildingDataFile;

use crate::assignment::CcCondition;
use crate::registry::{BuffResolutionStrategy, OrderEffect};
use crate::scoring::clause::{
    Clause, ClauseKind, CondScope, Metric, Subject, SuppressExempt, TENURE_HOURS,
    clauses_from_strategy, ramp_average,
};

fn tenure_hours(facility_counts: &HashMap<String, usize>) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    facility_counts
        .get(TENURE_HOURS)
        .map_or(24.0, |h| *h as f64)
}
use crate::types::OperatorBaseProfile;
use crate::util::buff_family;

const MIN_TRADING_ORDER_LIMIT: i32 = 1;
const FALLBACK_TRADING_ORDER_LIMIT: i32 = 6;

const MAX_PEER_ROUNDS: usize = 12;
const PEER_EPS: f64 = 1e-6;
pub const MAX_POOL_ROUNDS: usize = 8;
pub const POOL_EPS: f64 = 1e-6;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    Direct,
    RoomCountScaled,
    PeerScaled,
    PoolDrain,
    /// Granted by an EXTERNAL source
    Granted,
}

struct Entry {
    entity: usize,
    metric: Metric,
    amount: f64,
    source: Source,
}

/// A queued peer-scaling clause
struct PeerItem {
    entity: usize,
    metric_out: Metric,
    basis_metric: Metric,
    include_self: bool,
    step: f64,
    stage: u8,
    quantized: bool,
    own_skills_only: bool,
    value: f64,
    cap: Option<f64>,
    emitted: f64,
}

struct SuppressItem {
    entity: usize,
    metrics: Vec<Metric>,
}

struct OrderItem {
    entity: usize,
    metric: Metric,
    effect: OrderEffect,
}

pub struct RoomEval<'a> {
    pub member_ids: &'a [String],
    pub room_type: &'a str,
    pub formula_type: Option<&'a str>,
    pub op_index: &'a HashMap<&'a str, &'a OperatorBaseProfile>,
    pub registry: &'a HashMap<String, BuffResolutionStrategy>,
    pub building_data: &'a BuildingDataFile,
    pub facility_counts: &'a HashMap<String, usize>,
    pub total_dorm_levels: i32,
    pub cc_conditions: &'a [CcCondition],
    /// `None` = the base-wide `TRADING_MIN_LEVEL` synthetic
    pub room_level: Option<i32>,
    pub deployed_work_area: Option<&'a HashSet<String>>,
}

const ASSUMED_CAPACITY_POINTS: f64 = 20.0;

pub struct RoomTotals {
    pub speed_pct: f64,
    /// Order VALUE, percent over a bare post
    pub order_value_pct: f64,
    /// Order gold THROUGHPUT, percent over a bare post
    pub order_gold_pct: f64,
    pub capacity_delta: f64,
    pub order_limit: Option<i32>,
}

/// Pre-skill order limit
pub(crate) fn trading_base_limit(
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    room_level: Option<i32>,
) -> i32 {
    let phases = &building_data.trading_data.phases;
    trading_level(facility_counts, room_level)
        .and_then(|lv| phases.get(lv.saturating_sub(1)))
        .or_else(|| phases.last())
        .map_or(FALLBACK_TRADING_ORDER_LIMIT, |p| p.order_limit)
}

fn trading_level(
    facility_counts: &HashMap<String, usize>,
    room_level: Option<i32>,
) -> Option<usize> {
    room_level
        .and_then(|lv| usize::try_from(lv).ok())
        .or_else(|| {
            facility_counts
                .get(crate::assignment::TRADING_MIN_LEVEL)
                .copied()
        })
}

pub fn score_room(ev: &RoomEval<'_>) -> RoomTotals {
    let members: Vec<&OperatorBaseProfile> = ev
        .member_ids
        .iter()
        .filter_map(|id| ev.op_index.get(id.as_str()).copied())
        .collect();

    let mut member_clauses: Vec<Vec<(Clause, bool)>> = members
        .iter()
        .map(|op| {
            op.available_buffs
                .iter()
                .filter_map(|b| {
                    let buff = ev.building_data.buffs.get(b)?;
                    let strategy = ev.registry.get(b)?;
                    Some((b, buff, strategy))
                })
                .flat_map(|(b, buff, strategy)| {
                    let facility = matches!(
                        strategy,
                        BuffResolutionStrategy::FacilityCountScaling { .. }
                            | BuffResolutionStrategy::RoomPerOperatorGrant { .. }
                    );
                    clauses_from_strategy(b, buff, strategy)
                        .into_iter()
                        .map(move |c| (c, facility))
                })
                .filter(|(c, _)| c.owner_room_type == ev.room_type)
                .filter(|(c, _)| config_factor(c, ev.formula_type) > 0.0)
                .collect()
        })
        .collect();

    let exclusions: Vec<(usize, Vec<String>)> = member_clauses
        .iter()
        .enumerate()
        .flat_map(|(i, cs)| {
            cs.iter().filter_map(move |(c, _)| match &c.kind {
                ClauseKind::ExcludesBuffs { prefixes } => Some((i, prefixes.clone())),
                _ => None,
            })
        })
        .collect();
    for (owner, prefixes) in &exclusions {
        for (j, cs) in member_clauses.iter_mut().enumerate() {
            if j == *owner {
                continue;
            }
            cs.retain(|(c, _)| !prefixes.iter().any(|p| p == buff_family(&c.buff_id)));
        }
    }

    let automation_wipe = member_clauses.iter().flatten().any(|(c, _)| {
        matches!(
            &c.kind,
            ClauseKind::SuppressesOthers {
                exempt: SuppressExempt::RoomCountScaledSources,
                ..
            }
        )
    });

    // P0: tag augmentation
    let grants: Vec<(&Vec<String>, &String)> = member_clauses
        .iter()
        .flatten()
        .filter_map(|(c, _)| match &c.kind {
            ClauseKind::GrantsTag { from, to } => Some((from, to)),
            _ => None,
        })
        .collect();
    let skill_tags: Vec<Vec<String>> = members
        .iter()
        .map(|op| {
            let mut t: Vec<String> = op
                .available_buffs
                .iter()
                .filter_map(|b| ev.building_data.buffs.get(b))
                .filter_map(|buff| buff.buff_name.split([' ', '-']).next())
                .map(str::to_lowercase)
                .filter(|w| !w.is_empty())
                .collect();
            for (from, to) in &grants {
                if t.iter().any(|tag| from.contains(tag)) && !t.iter().any(|tag| tag == *to) {
                    t.push((*to).clone());
                }
            }
            t
        })
        .collect();
    let tags: Vec<Vec<String>> = members
        .iter()
        .map(|op| {
            let mut t = op.match_tags.clone();
            for (from, to) in &grants {
                if t.iter().any(|tag| from.contains(tag)) && !t.iter().any(|tag| tag == *to) {
                    t.push((*to).clone());
                }
            }
            t
        })
        .collect();

    let own_capacity: Vec<f64> = members
        .iter()
        .enumerate()
        .map(|(i, _)| {
            member_clauses[i]
                .iter()
                .filter(|(c, _)| c.metric == Metric::CapacityLimit)
                .map(|(c, _)| match &c.kind {
                    ClauseKind::SelfValue => c.value,
                    ClauseKind::RequiresChar {
                        chars,
                        scope: CondScope::Room,
                    } if chars
                        .iter()
                        .any(|req| members.iter().any(|m| &m.char_id == req)) =>
                    {
                        c.value
                    }
                    ClauseKind::ScalingCount {
                        subject,
                        include_self,
                    } => {
                        let n = count_matches(
                            subject,
                            i,
                            &members,
                            &tags,
                            &skill_tags,
                            &[],
                            *include_self,
                        );
                        let raw = n * c.value;
                        c.cap.map_or(raw, |cap| raw.min(cap))
                    }
                    _ => 0.0,
                })
                .sum()
        })
        .collect();

    // P2: layout-derived pool settlement
    let room_pools: HashMap<&str, f64> = {
        let functional_levels = ev
            .facility_counts
            .get(crate::assignment::FUNCTIONAL_LEVEL_SUM)
            .copied()
            .unwrap_or(0) as f64;
        let mut pools: HashMap<&str, f64> = HashMap::new();
        for (c, _) in member_clauses.iter().flatten() {
            if let ClauseKind::ResourceConvert(crate::scoring::clause::ResourceOp::Generate {
                resource,
                basis: crate::scoring::clause::PoolBasis::FunctionalLevels,
            }) = &c.kind
            {
                let points = c.cap.map_or(c.value * functional_levels, |cap| {
                    (c.value * functional_levels).min(cap)
                });
                *pools.entry(resource.as_str()).or_insert(0.0) += points;
            }
        }
        pools
    };

    // P1: immediate emission + deferred queues
    let mut entries: Vec<Entry> = Vec::new();
    let mut peers: Vec<PeerItem> = Vec::new();
    let mut suppressors: Vec<SuppressItem> = Vec::new();
    let mut order_items: Vec<OrderItem> = Vec::new();
    let mut limit_items: Vec<(usize, Metric, f64)> = Vec::new();

    for (i, _op) in members.iter().enumerate() {
        for (clause, from_facility) in &member_clauses[i] {
            let factor = config_factor(clause, ev.formula_type);
            let direct_source = if *from_facility {
                Source::RoomCountScaled
            } else {
                Source::Direct
            };
            match &clause.kind {
                ClauseKind::OrderMix(effect) => {
                    if factor > 0.0 {
                        order_items.push(OrderItem {
                            entity: i,
                            metric: clause.metric.clone(),
                            effect: effect.clone(),
                        });
                    }
                }
                ClauseKind::SelfValue => entries.push(Entry {
                    entity: i,
                    metric: clause.metric.clone(),
                    amount: clause.value * factor,
                    source: direct_source,
                }),
                ClauseKind::ScalingCount {
                    subject,
                    include_self,
                } => {
                    let count = count_matches(
                        subject,
                        i,
                        &members,
                        &tags,
                        &skill_tags,
                        &own_capacity,
                        *include_self,
                    );
                    let scaled = clause.value * count;
                    let amount = clause.cap.map_or(scaled, |cap| scaled.min(cap));
                    entries.push(Entry {
                        entity: i,
                        metric: clause.metric.clone(),
                        amount: amount * factor,
                        source: direct_source,
                    });
                }
                ClauseKind::ScalingLevelSum { .. } => {
                    let scaled = clause.value * f64::from(ev.total_dorm_levels);
                    let amount = clause.cap.map_or(scaled, |cap| scaled.min(cap));
                    entries.push(Entry {
                        entity: i,
                        metric: clause.metric.clone(),
                        amount: amount * factor,
                        source: Source::RoomCountScaled,
                    });
                }
                ClauseKind::ScalingRoomCount { room } => {
                    let count = ev.facility_counts.get(room).copied().unwrap_or(0);
                    let scaled = clause.value * count as f64;
                    let amount = clause.cap.map_or(scaled, |cap| scaled.min(cap));
                    entries.push(Entry {
                        entity: i,
                        metric: clause.metric.clone(),
                        amount: amount * factor,
                        source: Source::RoomCountScaled,
                    });
                }
                ClauseKind::TimeRamp {
                    first_hour,
                    per_hour,
                } => {
                    let hours = tenure_hours(ev.facility_counts);
                    entries.push(Entry {
                        entity: i,
                        metric: clause.metric.clone(),
                        amount: ramp_average(*first_hour, *per_hour, clause.value, hours) * factor,
                        source: Source::Direct,
                    });
                }
                ClauseKind::ScalingPoolPoints { resource, step } => {
                    let points = room_pools
                        .get(resource.as_str())
                        .copied()
                        .or_else(|| {
                            ev.facility_counts
                                .get(&format!("{}{resource}", crate::scoring::pools::POOL_PREFIX))
                                .map(|p| *p as f64)
                        })
                        .unwrap_or(0.0);
                    let amount = (points / step).floor() * clause.value;
                    entries.push(Entry {
                        entity: i,
                        metric: clause.metric.clone(),
                        amount: amount * factor,
                        source: Source::PoolDrain,
                    });
                }
                ClauseKind::RequiresChar { chars, scope } => {
                    let present = match scope {
                        CondScope::Room => chars.iter().any(|req| {
                            members
                                .iter()
                                .enumerate()
                                .any(|(j, m)| j != i && &m.char_id == req)
                        }),
                        CondScope::BaseWorkArea => ev
                            .deployed_work_area
                            .is_some_and(|d| chars.iter().any(|req| d.contains(req))),
                        CondScope::BaseAnywhere | CondScope::RoomTypeElsewhere => false,
                    };
                    if present {
                        entries.push(Entry {
                            entity: i,
                            metric: clause.metric.clone(),
                            amount: clause.value * factor,
                            source: Source::Direct,
                        });
                    }
                }
                ClauseKind::RequiresTag { tag, scope } => {
                    let present = match scope {
                        CondScope::Room => members
                            .iter()
                            .enumerate()
                            .any(|(j, _)| j != i && tags[j].iter().any(|t| t == tag)),
                        CondScope::BaseWorkArea | CondScope::BaseAnywhere => false,
                        CondScope::RoomTypeElsewhere => false,
                    };
                    if present {
                        entries.push(Entry {
                            entity: i,
                            metric: clause.metric.clone(),
                            amount: clause.value * factor,
                            source: Source::Direct,
                        });
                    }
                }
                ClauseKind::RequiresAbsentTag { subject } => {
                    let others = match subject {
                        Subject::AnyOtherOccupant => members.len() > 1,
                        _ => false,
                    };
                    if !others {
                        entries.push(Entry {
                            entity: i,
                            metric: clause.metric.clone(),
                            amount: clause.value * factor,
                            source: Source::Direct,
                        });
                    }
                }
                ClauseKind::RequiresCountTag { tag, count } => {
                    let n = members
                        .iter()
                        .enumerate()
                        .filter(|(j, _)| *j != i && tags[*j].iter().any(|t| t == tag))
                        .count();
                    if n >= *count {
                        entries.push(Entry {
                            entity: i,
                            metric: clause.metric.clone(),
                            amount: clause.value * factor,
                            source: Source::Direct,
                        });
                    }
                }
                ClauseKind::ScalingPeerMetric {
                    metric,
                    include_self,
                    step,
                    stage,
                    quantized,
                    own_skills_only,
                } => peers.push(PeerItem {
                    entity: i,
                    metric_out: clause.metric.clone(),
                    basis_metric: metric.clone(),
                    include_self: *include_self,
                    step: *step,
                    stage: *stage,
                    quantized: *quantized,
                    own_skills_only: *own_skills_only,
                    value: clause.value * factor,
                    cap: clause.cap,
                    emitted: 0.0,
                }),
                ClauseKind::ScalingRoomOrderLimit => {
                    limit_items.push((i, clause.metric.clone(), clause.value * factor));
                }
                ClauseKind::ExcludesBuffs { .. } => {}
                ClauseKind::SuppressesOthers { metrics, .. } => {
                    suppressors.push(SuppressItem {
                        entity: i,
                        metrics: metrics.clone(),
                    });
                }
                ClauseKind::ResourceConvert(crate::scoring::clause::ResourceOp::Consume {
                    ..
                }) => {
                    entries.push(Entry {
                        entity: i,
                        metric: clause.metric.clone(),
                        amount: clause.value * factor,
                        source: Source::PoolDrain,
                    });
                }
                ClauseKind::ResourceConvert(_) => {}
                ClauseKind::RoomTypeGlobal { .. }
                | ClauseKind::GrantsTag { .. }
                | ClauseKind::Unresolved => {}
            }
        }
    }

    // Per-operator Control-Center grants
    {
        let speed_metric = Metric::speed_for_room(ev.room_type);
        for cond in ev.cc_conditions {
            if !cond.per_operator || cond.target_room != ev.room_type {
                continue;
            }
            let amount = cond.bonus_for(ev.formula_type);
            for (i, m) in members.iter().enumerate() {
                if !crate::assignment::cc_token_matches(m, &cond.faction_token) {
                    continue;
                }
                entries.push(Entry {
                    entity: i,
                    metric: speed_metric.clone(),
                    amount,
                    source: Source::Granted,
                });
                if cond.order_limit != 0.0 {
                    entries.push(Entry {
                        entity: i,
                        metric: Metric::CapacityLimit,
                        amount: cond.order_limit,
                        source: Source::Granted,
                    });
                }
            }
        }
    }

    // P3: peer scaling by deltas, stage by stage
    let last_stage = peers.iter().map(|p| p.stage).max().unwrap_or(0);
    for stage in 0..=last_stage {
        for _ in 0..MAX_PEER_ROUNDS {
            let mut max_delta = 0.0f64;
            let mut emissions: Vec<Entry> = Vec::new();
            for item in peers.iter_mut().filter(|p| p.stage == stage) {
                let basis_total: f64 = entries
                    .iter()
                    .filter(|e| e.metric == item.basis_metric)
                    .filter(|e| item.include_self || e.entity != item.entity)
                    .filter(|e| {
                        !item.own_skills_only
                            || !matches!(e.source, Source::RoomCountScaled | Source::Granted)
                    })
                    .map(|e| e.amount)
                    .sum();
                let scaled = if item.quantized {
                    (basis_total.max(0.0) / item.step).floor() * item.value
                } else {
                    basis_total / item.step * item.value
                };
                let target = item.cap.map_or(scaled, |cap| scaled.min(cap));
                let delta = target - item.emitted;
                if delta.abs() > 0.0 {
                    emissions.push(Entry {
                        entity: item.entity,
                        metric: item.metric_out.clone(),
                        amount: delta,
                        source: Source::PeerScaled,
                    });
                    item.emitted = target;
                }
                max_delta = max_delta.max(delta.abs());
            }
            entries.extend(emissions);
            if max_delta < PEER_EPS {
                break;
            }
        }
    }

    // Order limit
    let cc_room_capacity: f64 = ev
        .cc_conditions
        .iter()
        .filter(|c| !c.per_operator)
        .map(|c| c.capacity_contribution(ev.room_type, &members))
        .sum();
    let capacity_delta: f64 = entries
        .iter()
        .filter(|e| e.metric == Metric::CapacityLimit)
        .map(|e| e.amount)
        .sum::<f64>()
        + cc_room_capacity;
    let order_limit: Option<i32> = (ev.room_type == "TRADING").then(|| {
        #[allow(clippy::cast_possible_truncation)]
        let delta = capacity_delta.round() as i32;
        (trading_base_limit(ev.building_data, ev.facility_counts, ev.room_level) + delta)
            .max(MIN_TRADING_ORDER_LIMIT)
    });
    if let Some(limit) = order_limit {
        for (entity, metric, value) in &limit_items {
            entries.push(Entry {
                entity: *entity,
                metric: metric.clone(),
                amount: value * f64::from(limit),
                source: Source::Direct,
            });
        }
    }

    // P4: suppression, strictly last
    let speed_metric = Metric::speed_for_room(ev.room_type);

    if automation_wipe {
        for e in &mut entries {
            if e.metric == speed_metric && e.source != Source::RoomCountScaled {
                e.amount = 0.0;
            }
        }
    }
    for s in &suppressors {
        for e in &mut entries {
            if e.entity == s.entity && e.source != Source::Granted {
                continue;
            }
            if automation_wipe && e.metric == speed_metric {
                continue;
            }
            if !s.metrics.contains(&e.metric) {
                continue;
            }
            e.amount = 0.0;
        }
    }

    order_items.retain(|item| {
        !suppressors
            .iter()
            .any(|s| s.entity != item.entity && s.metrics.contains(&item.metric))
    });

    // P5: finalization
    let mut speed: f64 = entries
        .iter()
        .filter(|e| e.metric == speed_metric)
        .map(|e| e.amount)
        .sum();

    let (order_value, order_gold) = if order_items.is_empty() {
        (0.0, 0.0)
    } else {
        let level = trading_level(ev.facility_counts, ev.room_level)
            .map_or(i32::MAX, |lv| i32::try_from(lv).unwrap_or(i32::MAX));
        let rarity = crate::scoring::order_mix::rarity_for_level(ev.building_data, level);
        let effects: Vec<OrderEffect> = order_items.iter().map(|o| o.effect.clone()).collect();
        (
            crate::scoring::order_mix::value_pct(&effects, rarity),
            crate::scoring::order_mix::gold_pct(&effects, rarity),
        )
    };

    speed += ev
        .cc_conditions
        .iter()
        .filter(|c| !c.per_operator)
        .map(|c| c.contribution(ev.room_type, &members, ev.formula_type))
        .sum::<f64>();

    RoomTotals {
        speed_pct: speed,
        order_value_pct: order_value,
        order_gold_pct: order_gold,
        capacity_delta,
        order_limit,
    }
}

fn applicable_clauses<'a>(
    op: &'a OperatorBaseProfile,
    room_type: &'a str,
    formula_type: Option<&'a str>,
    registry: &'a HashMap<String, BuffResolutionStrategy>,
    building_data: &'a BuildingDataFile,
) -> impl Iterator<Item = (Clause, f64)> + 'a {
    op.available_buffs
        .iter()
        .filter_map(move |b| {
            let buff = building_data.buffs.get(b)?;
            let strategy = registry.get(b)?;
            Some(clauses_from_strategy(b, buff, strategy))
        })
        .flatten()
        .filter(move |c| c.owner_room_type == room_type)
        .filter_map(move |c| {
            let factor = config_factor(&c, formula_type);
            (factor > 0.0).then_some((c, factor))
        })
}

/// Optimistic solo upper bound for candidate ranking
#[allow(clippy::too_many_arguments, clippy::cast_precision_loss)]
pub fn op_optimistic_bound(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    max_slots: usize,
    companions: &[OrderEffect],
) -> f64 {
    let teammates_assumed = max_slots.saturating_sub(1) as f64;
    let speed_metric = Metric::speed_for_room(room_type);
    let counts_toward_bound =
        |m: &Metric| *m == speed_metric || matches!(m, Metric::OrderValue { .. });
    let order_rarity = crate::scoring::order_mix::rarity_for_level(
        building_data,
        facility_counts
            .get(crate::assignment::TRADING_MIN_LEVEL)
            .map_or(i32::MAX, |lv| i32::try_from(*lv).unwrap_or(i32::MAX)),
    );
    let functional_levels = facility_counts
        .get(crate::assignment::FUNCTIONAL_LEVEL_SUM)
        .copied()
        .unwrap_or(0) as f64;
    let full_dorms = {
        let dorms = facility_counts.get("DORMITORY").copied().unwrap_or(0) as f64;
        let top_level = building_data
            .rooms
            .get("DORMITORY")
            .map_or(1, |def| i32::try_from(def.phases.len()).unwrap_or(1));
        dorms
            * f64::from(
                crate::util::max_stationed_at_level(building_data, "DORMITORY", top_level).max(0),
            )
    };
    let mut own_pools: HashMap<String, f64> =
        applicable_clauses(op, room_type, formula_type, registry, building_data)
            .filter_map(|(c, _)| match &c.kind {
                ClauseKind::ResourceConvert(crate::scoring::clause::ResourceOp::Generate {
                    resource,
                    basis: crate::scoring::clause::PoolBasis::FunctionalLevels,
                }) => {
                    let points = c.cap.map_or(c.value * functional_levels, |cap| {
                        (c.value * functional_levels).min(cap)
                    });
                    Some((resource.clone(), points))
                }
                ClauseKind::ResourceConvert(crate::scoring::clause::ResourceOp::Generate {
                    resource,
                    basis: crate::scoring::clause::PoolBasis::DormOccupants,
                }) => {
                    let points = c
                        .cap
                        .map_or(c.value * full_dorms, |cap| (c.value * full_dorms).min(cap));
                    Some((resource.clone(), points))
                }
                _ => None,
            })
            .collect();
    for (c, _) in applicable_clauses(op, room_type, formula_type, registry, building_data) {
        if let ClauseKind::ResourceConvert(crate::scoring::clause::ResourceOp::Convert {
            from,
            to,
            ratio,
        }) = &c.kind
            && *ratio > 0.0
            && let Some(points) = own_pools.get(from).copied()
        {
            *own_pools.entry(to.clone()).or_insert(0.0) += points / ratio;
        }
    }
    let mut total = 0.0;
    for (c, factor) in applicable_clauses(op, room_type, formula_type, registry, building_data) {
        if !counts_toward_bound(&c.metric) {
            continue;
        }
        let v = c.value * factor;
        total += match &c.kind {
            ClauseKind::SelfValue => v,
            ClauseKind::OrderMix(effect) => {
                crate::scoring::order_mix::optimistic_value_pct_among(
                    effect,
                    order_rarity,
                    companions,
                ) * factor
            }
            ClauseKind::ResourceConvert(crate::scoring::clause::ResourceOp::Consume { .. }) => v,
            ClauseKind::ScalingPoolPoints { resource, step } => {
                (own_pools.get(resource).copied().unwrap_or(0.0) / step).floor() * v
            }
            ClauseKind::RequiresChar { .. } | ClauseKind::RequiresTag { .. } => v,
            ClauseKind::ScalingCount {
                subject,
                include_self,
            } => {
                let assumed = match subject {
                    Subject::CapacityPoints { .. } => {
                        (teammates_assumed + f64::from(u8::from(*include_self)))
                            * ASSUMED_CAPACITY_POINTS
                    }
                    Subject::PeerMetricAbove { .. }
                    | Subject::AnyOtherOccupant
                    | Subject::Tag(_)
                    | Subject::SkillTag(_)
                    | Subject::SkillIdPrefix(_)
                    | Subject::Chars(_) => teammates_assumed + f64::from(u8::from(*include_self)),
                };
                let scaled = v * assumed;
                c.cap.map_or(scaled, |cap| scaled.min(cap))
            }
            ClauseKind::ScalingRoomCount { room } => {
                let scaled = v * facility_counts.get(room).copied().unwrap_or(0) as f64;
                c.cap.map_or(scaled, |cap| scaled.min(cap))
            }
            ClauseKind::TimeRamp {
                first_hour,
                per_hour,
            } => ramp_average(*first_hour, *per_hour, v, tenure_hours(facility_counts)),
            ClauseKind::ScalingLevelSum { .. } => {
                let scaled = v * f64::from(total_dorm_levels);
                c.cap.map_or(scaled, |cap| scaled.min(cap))
            }
            ClauseKind::ScalingPeerMetric { metric, step, .. } => {
                if *metric == Metric::CapacityLimit {
                    if room_type == "MANUFACTURE" {
                        const ASSUMED_CAP: f64 = 20.0;
                        let scaled = (ASSUMED_CAP / step).floor() * v;
                        c.cap.map_or(scaled, |cap| scaled.min(cap))
                    } else {
                        0.0
                    }
                } else {
                    c.cap.unwrap_or(0.0)
                }
            }
            ClauseKind::ScalingRoomOrderLimit => {
                v * f64::from(trading_base_limit(building_data, facility_counts, None))
            }
            _ => 0.0,
        };
    }
    total
}

/// The facility-count part of an operator's value
pub fn op_facility_only_value(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
) -> f64 {
    op.available_buffs
        .iter()
        .filter(|b| {
            matches!(
                registry.get(*b),
                Some(BuffResolutionStrategy::FacilityCountScaling { .. })
            )
        })
        .filter_map(|b| {
            let buff = building_data.buffs.get(b)?;
            let strategy = registry.get(b)?;
            Some(clauses_from_strategy(b, buff, strategy))
        })
        .flatten()
        .filter(|c| c.owner_room_type == room_type)
        .map(|c| {
            let factor = config_factor(&c, formula_type);
            #[allow(clippy::cast_precision_loss)]
            let raw = match &c.kind {
                ClauseKind::SelfValue => c.value,
                ClauseKind::ScalingRoomCount { room } => {
                    let scaled = c.value * facility_counts.get(room).copied().unwrap_or(0) as f64;
                    c.cap.map_or(scaled, |cap| scaled.min(cap))
                }
                ClauseKind::ScalingLevelSum { .. } => {
                    let scaled = c.value * f64::from(total_dorm_levels);
                    c.cap.map_or(scaled, |cap| scaled.min(cap))
                }
                _ => 0.0,
            };
            raw * factor
        })
        .sum()
}

pub fn op_capacity_limit(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    present: &HashSet<String>,
) -> f64 {
    applicable_clauses(op, room_type, formula_type, registry, building_data)
        .filter(|(c, _)| c.metric == Metric::CapacityLimit)
        .map(|(c, _)| match &c.kind {
            ClauseKind::SelfValue => c.value,
            ClauseKind::RequiresChar {
                chars,
                scope: CondScope::Room,
            } if chars.iter().any(|req| present.contains(req)) => c.value,
            ClauseKind::ScalingCount {
                include_self: true, ..
            } => c.value,
            _ => 0.0,
        })
        .sum()
}

/// Order VALUE that survives a Shamare-type nullifier
pub fn op_surviving_order_value(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    companions: &[OrderEffect],
) -> f64 {
    let top = crate::scoring::order_mix::rarity_for_level(building_data, i32::MAX);
    applicable_clauses(op, room_type, formula_type, registry, building_data)
        .filter(|(c, _)| c.metric == (Metric::OrderValue { pure_gold: false }))
        .filter_map(|(c, factor)| match &c.kind {
            ClauseKind::OrderMix(effect) => Some(
                crate::scoring::order_mix::optimistic_value_pct_among(effect, top, companions)
                    * factor,
            ),
            _ => None,
        })
        .sum()
}

pub fn roster_order_effects(
    operators: &[&OperatorBaseProfile],
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> Vec<OrderEffect> {
    let _ = formula_type;
    let mut out: Vec<OrderEffect> = Vec::new();
    if room_type != "TRADING" {
        return out;
    }
    for op in operators {
        for buff_id in &op.available_buffs {
            if let Some(BuffResolutionStrategy::OrderValue { effect, .. }) = registry.get(buff_id)
                && building_data
                    .buffs
                    .get(buff_id)
                    .is_some_and(|b| b.room_type == room_type)
                && !out.contains(effect)
            {
                out.push(effect.clone());
            }
        }
    }
    out
}

/// Ranking value of the strongest POWER buff
pub fn op_power_rank_value(
    op: &OperatorBaseProfile,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    facility_counts: &HashMap<String, usize>,
) -> f64 {
    const ENABLER_RANK_WEIGHT: f64 = 30.0;
    op.available_buffs
        .iter()
        .filter_map(|b| {
            let buff = building_data.buffs.get(b)?;
            (buff.room_type == "POWER").then_some(())?;
            let strategy = registry.get(b)?;
            Some(clauses_from_strategy(b, buff, strategy))
        })
        .map(|set| {
            set.iter()
                .map(|c| match &c.kind {
                    ClauseKind::SelfValue if c.metric == Metric::FacilityCount("POWER".into()) => {
                        c.value * ENABLER_RANK_WEIGHT
                    }
                    ClauseKind::SelfValue
                        if c.metric == Metric::speed_for_room(&c.owner_room_type) =>
                    {
                        c.value
                    }
                    #[allow(clippy::cast_precision_loss)]
                    ClauseKind::ScalingRoomCount { room } => {
                        let scaled =
                            c.value * facility_counts.get(room).copied().unwrap_or(0) as f64;
                        c.cap.map_or(scaled, |cap| scaled.min(cap))
                    }
                    _ => 0.0,
                })
                .sum::<f64>()
        })
        .fold(0.0, f64::max)
}

pub const UNCONFIGURED_OUTPUT_FACTOR: f64 = 0.5;

pub(crate) fn config_factor(clause: &Clause, formula_type: Option<&str>) -> f64 {
    if clause.output_targets.is_empty() {
        return 1.0;
    }
    match formula_type {
        Some(f) => {
            if clause.output_targets.iter().any(|t| t == f) {
                1.0
            } else {
                0.0
            }
        }
        None if crate::assignment::is_generic_targets(&clause.output_targets) => 1.0,
        None => UNCONFIGURED_OUTPUT_FACTOR,
    }
}

fn count_matches(
    subject: &Subject,
    owner: usize,
    members: &[&OperatorBaseProfile],
    tags: &[Vec<String>],
    skill_tags: &[Vec<String>],
    own_capacity: &[f64],
    include_self: bool,
) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let n = match subject {
        Subject::Tag(token) => (0..members.len())
            .filter(|&j| j != owner || include_self)
            .filter(|&j| tags[j].iter().any(|t| t == token))
            .count(),
        Subject::SkillTag(token) => (0..members.len())
            .filter(|&j| j != owner || include_self)
            .filter(|&j| skill_tags[j].iter().any(|t| t == token))
            .count(),
        Subject::SkillIdPrefix(prefix) => (0..members.len())
            .filter(|&j| j != owner)
            .filter(|&j| {
                members[j]
                    .available_buffs
                    .iter()
                    .any(|b| b.starts_with(prefix.as_str()))
            })
            .count(),
        Subject::AnyOtherOccupant => {
            let others = members.len().saturating_sub(1);
            if include_self { others + 1 } else { others }
        }
        Subject::Chars(chars) => (0..members.len())
            .filter(|&j| j != owner || include_self)
            .filter(|&j| chars.iter().any(|c| c == &members[j].char_id))
            .count(),
        Subject::PeerMetricAbove { metric, threshold } => {
            if *metric == Metric::CapacityLimit {
                (0..members.len())
                    .filter(|&j| j != owner || include_self)
                    .filter(|&j| own_capacity.get(j).copied().unwrap_or(0.0) > *threshold)
                    .count()
            } else {
                0
            }
        }
        Subject::CapacityPoints { threshold, above } => {
            return (0..members.len())
                .filter(|&j| j != owner || include_self)
                .map(|j| own_capacity.get(j).copied().unwrap_or(0.0))
                .filter(|cap| (*cap > *threshold) == *above)
                .map(|cap| cap.max(0.0))
                .sum();
        }
    };
    n as f64
}
