//! The ledger scorer. Resolves a room's clauses into per-(entity, metric)
//! entries, then settles in the algorithm reference's order: pools, peer scaling
//! by deltas, suppression LAST. Per-entity entries (not a running total) are what
//! let suppression and peer scaling look back at what was already computed.

use std::collections::{HashMap, HashSet};

use crate::core::gamedata::types::building::BuildingDataFile;

use super::assignment::CcCondition;
use super::buff_registry::{BuffResolutionStrategy, OrderEffect};
use super::clause::{
    Clause, ClauseKind, CondScope, Metric, Subject, SuppressExempt, clauses_from_strategy,
};
use super::types::OperatorBaseProfile;
use super::util::buff_family;

/// Game floor: a team that slashes the limit (Degenbrecher) still banks one order.
const MIN_TRADING_ORDER_LIMIT: i32 = 1;
/// No `TRADING_MIN_LEVEL` synthetic and no trading phases (hand-built test contexts).
const FALLBACK_TRADING_ORDER_LIMIT: i32 = 6;

/// Real cases converge in one or two rounds; these are guards, not tuning.
const MAX_PEER_ROUNDS: usize = 12;
const PEER_EPS: f64 = 1e-6;
/// Pool-settlement bounds, shared with the assignment-scope pass in `pools.rs`.
pub const MAX_POOL_ROUNDS: usize = 8;
pub const POOL_EPS: f64 = 1e-6;

/// Drives suppression exemptions and the legacy mirror basis.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    /// Flat / gate-resolved contribution.
    Direct,
    /// Scales on a facility/room count - survives automation suppression
    /// ("excluding productivity granted based on facility count").
    RoomCountScaled,
    /// Emitted by the peer-scaling relaxation.
    PeerScaled,
    /// Drained out of a resource pool.
    PoolDrain,
    /// Granted by an EXTERNAL source (per-operator CC conditional, e.g. Umiri).
    /// A nullifier (Shamare) kills it even on her own row: "contributions from
    /// other operators" means other SOURCES.
    Granted,
}

struct Entry {
    entity: usize,
    metric: Metric,
    amount: f64,
    source: Source,
}

/// A queued peer-scaling clause with its owner and its last-round emission.
struct PeerItem {
    entity: usize,
    metric_out: Metric,
    basis_metric: Metric,
    include_self: bool,
    step: f64,
    /// Evaluation stage: see `ClauseKind::ScalingPeerMetric`.
    stage: u8,
    /// Floor the basis by `step` and ignore a negative total.
    quantized: bool,
    /// Count only the roommates' own skill entries in the basis.
    own_skills_only: bool,
    value: f64,
    cap: Option<f64>,
    emitted: f64,
}

struct SuppressItem {
    entity: usize,
    metrics: Vec<Metric>,
}

/// Order-VALUE shape deferred to P5, priced with the room's other shapes.
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
    /// A trading post's rarity and base limit follow ITS level. `None` = the
    /// base-wide `TRADING_MIN_LEVEL` synthetic (candidate ranking, no concrete room).
    pub room_level: Option<i32>,
    /// Operators working any non-dorm room, for `RequiresChar { scope: BaseWorkArea }`.
    /// `None` = unknown (candidate enumeration); those clauses then give 0.
    pub deployed_work_area: Option<&'a HashSet<String>>,
}

/// Per-seat capacity points assumed when ranking a capacity-point scaler
/// (Bubble) before the team is known.
const ASSUMED_CAPACITY_POINTS: f64 = 20.0;

pub struct RoomTotals {
    pub speed_pct: f64,
    /// Order VALUE: LMD per hour over a bare post's, minus one (percent).
    pub order_value_pct: f64,
    /// Order gold THROUGHPUT, percent over a bare post: the part that draws bars
    /// from stock.
    pub order_gold_pct: f64,
    /// Crew capacity skills, per-operator and post-level CC grants, peer-scaled cuts.
    pub capacity_delta: f64,
    /// Trading post only: level base + `capacity_delta`, floored at 1.
    pub order_limit: Option<i32>,
}

/// Pre-skill order limit (6/8/10 for L1-L3, `TradingData.Phases`), resolved
/// through `TRADING_MIN_LEVEL` so search and display agree.
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

/// Own level when known, else the base-wide minimum. (Pricing every post at the
/// minimum made an L2 + L3 base read both wrong.)
fn trading_level(
    facility_counts: &HashMap<String, usize>,
    room_level: Option<i32>,
) -> Option<usize> {
    room_level
        .and_then(|lv| usize::try_from(lv).ok())
        .or_else(|| {
            facility_counts
                .get(super::assignment::TRADING_MIN_LEVEL)
                .copied()
        })
}

/// Clause walk (P0-P1), pools (P2), peer relaxation (P3), suppression (P4),
/// finalization (P5).
pub fn score_room(ev: &RoomEval) -> RoomTotals {
    let members: Vec<&OperatorBaseProfile> = ev
        .member_ids
        .iter()
        .filter_map(|id| ev.op_index.get(id.as_str()).copied())
        .collect();

    // Each clause tagged with whether its buff is a facility-count strategy:
    // that decides what survives an automation wipe, at BUFF granularity like
    // the old engine. Derived from the registry until it dies (CP5), then from a
    // prebuilt clause store.
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

    // "Does not stack with X and takes priority over it": drop every ROOMMATE's
    // clause of X's family before pricing (Bubble's Bigger is Better! over
    // Vermeil's Recycling: the game credits her tiers alone, 33 not 99).
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

    // Wipe only when the automation clause is LIVE in this room. The legacy
    // engine fired on merely CARRYING the buff (Weedy in a Trading Post wiped the
    // team); kept through the shadow migration, fixed after.
    let automation_wipe = member_clauses.iter().flatten().any(|(c, _)| {
        matches!(
            &c.kind,
            ClauseKind::SuppressesOthers {
                exempt: SuppressExempt::RoomCountScaledSources,
                ..
            }
        )
    });

    // ── P0: tag augmentation (Highmore-style converters) ─────────────────────
    let grants: Vec<(&Vec<String>, &String)> = member_clauses
        .iter()
        .flatten()
        .filter_map(|(c, _)| match &c.kind {
            ClauseKind::GrantsTag { from, to } => Some((from, to)),
            _ => None,
        })
        .collect();
    // Leading word of each skill name, the basis for skill counts. P0
    // converters apply here too.
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

    // Own capacity (Self + room-scoped gates), the basis for capacity-tier
    // subjects (Bubble). The room's final limit comes off the ledger after P3.
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
                    // Astgenne's "+5 Storage Capacity for each Rhine Tech-type
                    // skill in that Factory": counts skills/tags, never capacity
                    // tiers, so no capacity basis needed.
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

    // ── P2 (room-local slice): layout-derived pool settlement ────────────────
    // LAYOUT-fed generators (Minimalist's Engineering Robots: per functional
    // facility level, capped) settle here. ASSIGNMENT-fed pools (dorm occupants,
    // resters) wait for the assignment-scope pass; their consumers read zero.
    let room_pools: HashMap<&str, f64> = {
        let functional_levels = ev
            .facility_counts
            .get(super::assignment::FUNCTIONAL_LEVEL_SUM)
            .copied()
            .unwrap_or(0) as f64;
        let mut pools: HashMap<&str, f64> = HashMap::new();
        for (c, _) in member_clauses.iter().flatten() {
            if let ClauseKind::ResourceConvert(super::clause::ResourceOp::Generate {
                resource,
                basis: super::clause::PoolBasis::FunctionalLevels,
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

    // ── P1: immediate emission + deferred queues ─────────────────────────────
    let mut entries: Vec<Entry> = Vec::new();
    let mut peers: Vec<PeerItem> = Vec::new();
    let mut suppressors: Vec<SuppressItem> = Vec::new();
    let mut order_items: Vec<OrderItem> = Vec::new();
    // Per-order riders on the room's FINAL limit: (entity, metric, per order).
    let mut limit_items: Vec<(usize, Metric, f64)> = Vec::new();

    for (i, _op) in members.iter().enumerate() {
        for (clause, from_facility) in &member_clauses[i] {
            let factor = config_factor(clause, ev.formula_type);
            // A facility-count buff's flat base survives an automation wipe with
            // its scaled part: the old engine kept or dropped whole BUFFS.
            let direct_source = if *from_facility {
                Source::RoomCountScaled
            } else {
                Source::Direct
            };
            match &clause.kind {
                // Priced jointly in P5. A zero config factor drops it.
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
                    // Facility-provenance counts (Snegurochka's "that Factory's
                    // productivity" per occupant) survive the automation wipe.
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
                // +value per `step` settled points, floored. Points from the
                // room-local settlement or the `POOL_<resource>` synthetic; an
                // unfed pool reads zero.
                ClauseKind::ScalingPoolPoints { resource, step } => {
                    let points = room_pools
                        .get(resource.as_str())
                        .copied()
                        .or_else(|| {
                            ev.facility_counts
                                .get(&format!("{}{resource}", super::pools::POOL_PREFIX))
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
                        // Rewritten by resolve_base_wide / resolve_room_presence
                        // before scoring; context-free passes credit 0.
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
                        // Rewritten by resolve_room_presence; context-free passes
                        // credit 0.
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
                // Already applied to the member clause lists above.
                ClauseKind::ExcludesBuffs { .. } => {}
                // The automation exempt marker is honored by the wipe's source
                // check, not here.
                ClauseKind::SuppressesOthers { metrics, .. } => {
                    suppressors.push(SuppressItem {
                        entity: i,
                        metrics: metrics.clone(),
                    });
                }
                // Pre-SOLVED drain (perception seam). Generate/Convert wait for
                // the pool pass.
                ClauseKind::ResourceConvert(super::clause::ResourceOp::Consume { .. }) => {
                    entries.push(Entry {
                        entity: i,
                        metric: clause.metric.clone(),
                        amount: clause.value * factor,
                        source: Source::PoolDrain,
                    });
                }
                ClauseKind::ResourceConvert(_) => {}
                // CC globals fan out at assignment scope; GrantsTag was used in P0.
                ClauseKind::RoomTypeGlobal { .. }
                | ClauseKind::GrantsTag { .. }
                | ClauseKind::Unresolved => {}
            }
        }
    }

    // ── Per-operator Control-Center grants ──────────────────────────────────
    // Umiri-style ("all Siracusa Operators assigned to Trading Posts gain +5%")
    // buff OPERATORS: per-entity entries a suppressor (Shamare) kills like any
    // teammate's. Threshold ones ("...with 3 Kjerag Operators") buff the POST and
    // are added in P5, immune to suppression. Lands BEFORE peer scaling: Gnosis's
    // "-15% efficiency and +6 order limit" per Kjerag trader is what Degenbrecher,
    // Swire and Jaye's cut read.
    {
        let speed_metric = Metric::speed_for_room(ev.room_type);
        for cond in ev.cc_conditions {
            if !cond.per_operator || cond.target_room != ev.room_type {
                continue;
            }
            let amount = cond.bonus_for(ev.formula_type);
            for (i, m) in members.iter().enumerate() {
                if !super::assignment::cc_token_matches(m, &cond.faction_token) {
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

    // ── P3: peer scaling by deltas, stage by stage ───────────────────────────
    // A reader sees everything settled before its stage; readers sharing a
    // stage relax to a fixed point (two mirrors reading each other).
    let last_stage = peers.iter().map(|p| p.stage).max().unwrap_or(0);
    for stage in 0..=last_stage {
        for _ in 0..MAX_PEER_ROUNDS {
            let mut max_delta = 0.0f64;
            let mut emissions: Vec<Entry> = Vec::new();
            for item in peers.iter_mut().filter(|p| p.stage == stage) {
                // Reference-pure basis: the roommates' FULL current total, all
                // sources. An own-skills reader (Waai Fu's "provided by all other
                // Operators ... excluding facility count") drops facility-count
                // parts and external grants.
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
                // Quantized ("per 5 CAP", "-1 per 10%"): positive basis only,
                // floored by step. A percentage mirror is continuous.
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

    // ── Order limit: the room's capacity after every delta ──────────────────
    // Skills, per-operator grants and peer cuts are entries by now; post-level CC
    // grants (Wiš'adel's "that Trading Post's order limit +2" with Hoederer) add
    // on top. No nullifier targets capacity, so this is final. "(minimum 1)"
    // floors the ROOM total, never a single skill.
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
    // Jaye's per-order riders pay on the FINAL limit, base included, then are
    // plain speed a nullifier can kill.
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

    // ── P4: suppression, strictly last ───────────────────────────────────────
    let speed_metric = Metric::speed_for_room(ev.room_type);

    // Automation wipe: the room's speed dies for EVERYONE, owners included,
    // "excluding productivity granted based on facility count". That's why
    // automation ops stack with each other and with facility-count scalers.
    if automation_wipe {
        for e in &mut entries {
            if e.metric == speed_metric && e.source != Source::RoomCountScaled {
                e.amount = 0.0;
            }
        }
    }
    for s in &suppressors {
        for e in &mut entries {
            // The suppressor's OWN skill survives; an external grant on her row
            // (Umiri's CC bonus) is someone else's and dies.
            if e.entity == s.entity && e.source != Source::Granted {
                continue;
            }
            // Under a wipe the wipe owns speed (facility parts survive a
            // nullifier too); non-speed metrics (Pure-Gold value) still die.
            if automation_wipe && e.metric == speed_metric {
                continue;
            }
            if !s.metrics.contains(&e.metric) {
                continue;
            }
            e.amount = 0.0;
        }
    }

    // Deferred order shapes too: Shamare's Precious-Metal shift kills Proviso's
    // Pure-Gold value, spares flat value.
    order_items.retain(|item| {
        !suppressors
            .iter()
            .any(|s| s.entity != item.entity && s.metrics.contains(&item.metric))
    });

    // ── P5: finalization ─────────────────────────────────────────────────────
    let mut speed: f64 = entries
        .iter()
        .filter(|e| e.metric == speed_metric)
        .map(|e| e.amount)
        .sum();

    // Shapes priced together against the post's rarity (see `order_mix`).
    let (order_value, order_gold) = if order_items.is_empty() {
        (0.0, 0.0)
    } else {
        let level = trading_level(ev.facility_counts, ev.room_level)
            .map_or(i32::MAX, |lv| i32::try_from(lv).unwrap_or(i32::MAX));
        let rarity = super::order_mix::rarity_for_level(ev.building_data, level);
        let effects: Vec<OrderEffect> = order_items.iter().map(|o| o.effect.clone()).collect();
        (
            super::order_mix::value_pct(&effects, rarity),
            super::order_mix::gold_pct(&effects, rarity),
        )
    };

    // Threshold CC bonuses buff the POST: added past suppression.
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

/// An operator's clauses live in `room_type`, each with its config factor (>0).
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

/// Optimistic solo upper bound for candidate ranking: gates met, teammate
/// scalers fully matched, facility scaling at real counts. Must stay >= the true
/// value; never a final score.
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
    // TRADING_MIN_LEVEL; top tier when no post is known.
    let order_rarity = super::order_mix::rarity_for_level(
        building_data,
        facility_counts
            .get(super::assignment::TRADING_MIN_LEVEL)
            .map_or(i32::MAX, |lv| i32::try_from(*lv).unwrap_or(i32::MAX)),
    );
    // Own layout-derived pools (generator + consumer on one op, e.g. Minimalist):
    // exact, not optimistic.
    let functional_levels = facility_counts
        .get(super::assignment::FUNCTIONAL_LEVEL_SUM)
        .copied()
        .unwrap_or(0) as f64;
    // Dorm-occupancy generators (Rosmontis' Extrasensory): assume FULL top-level dorms.
    let full_dorms = {
        let dorms = facility_counts.get("DORMITORY").copied().unwrap_or(0) as f64;
        let top_level = building_data
            .rooms
            .get("DORMITORY")
            .map_or(1, |def| i32::try_from(def.phases.len()).unwrap_or(1));
        dorms
            * f64::from(
                super::util::max_stationed_at_level(building_data, "DORMITORY", top_level).max(0),
            )
    };
    let mut own_pools: HashMap<String, f64> =
        applicable_clauses(op, room_type, formula_type, registry, building_data)
            .filter_map(|(c, _)| match &c.kind {
                ClauseKind::ResourceConvert(super::clause::ResourceOp::Generate {
                    resource,
                    basis: super::clause::PoolBasis::FunctionalLevels,
                }) => {
                    let points = c.cap.map_or(c.value * functional_levels, |cap| {
                        (c.value * functional_levels).min(cap)
                    });
                    Some((resource.clone(), points))
                }
                ClauseKind::ResourceConvert(super::clause::ResourceOp::Generate {
                    resource,
                    basis: super::clause::PoolBasis::DormOccupants,
                }) => {
                    let points = c
                        .cap
                        .map_or(c.value * full_dorms, |cap| (c.value * full_dorms).min(cap));
                    Some((resource.clone(), points))
                }
                _ => None,
            })
            .collect();
    // Own converters (Perception Information -> Chain of Thought).
    for (c, _) in applicable_clauses(op, room_type, formula_type, registry, building_data) {
        if let ClauseKind::ResourceConvert(super::clause::ResourceOp::Convert { from, to, ratio }) =
            &c.kind
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
            // Solo, or the marginal beside a mix-shifting partner (Tequila + Tailoring).
            ClauseKind::OrderMix(effect) => {
                super::order_mix::optimistic_value_pct_among(effect, order_rarity, companions)
                    * factor
            }
            // Face value: the consumer must rank high enough to be SEATED for the
            // pool to pay at all.
            ClauseKind::ResourceConvert(super::clause::ResourceOp::Consume { .. }) => v,
            // Fed by the op's own generator: exact.
            ClauseKind::ScalingPoolPoints { resource, step } => {
                (own_pools.get(resource).copied().unwrap_or(0.0) / step).floor() * v
            }
            // Assume the named teammate / faction is present.
            ClauseKind::RequiresChar { .. } | ClauseKind::RequiresTag { .. } => v,
            // Assume a full room of matches; a stated cap clamps the scaled part.
            ClauseKind::ScalingCount {
                subject,
                include_self,
            } => {
                let assumed = match subject {
                    Subject::CapacityPoints { .. } => {
                        (teammates_assumed + f64::from(u8::from(*include_self)))
                            * ASSUMED_CAPACITY_POINTS
                    }
                    // Peer-capacity tiers: assume everyone clears the threshold.
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
            // Facility scaling is real, not assumed: evaluate against actual counts.
            ClauseKind::ScalingRoomCount { room } => {
                let scaled = v * facility_counts.get(room).copied().unwrap_or(0) as f64;
                c.cap.map_or(scaled, |cap| scaled.min(cap))
            }
            ClauseKind::ScalingLevelSum { .. } => {
                let scaled = v * f64::from(total_dorm_levels);
                c.cap.map_or(scaled, |cap| scaled.min(cap))
            }
            ClauseKind::ScalingPeerMetric { metric, step, .. } => {
                if *metric == Metric::CapacityLimit {
                    // Trading posts almost never field capacity-adders: bound 0.
                    // Factories stack them (Vermeil's comp), so assume a rich
                    // room or she ranks at zero and is cut before the scorer
                    // tries her with the teammates that make her strong.
                    if room_type == "MANUFACTURE" {
                        const ASSUMED_CAP: f64 = 20.0;
                        let scaled = (ASSUMED_CAP / step).floor() * v;
                        c.cap.map_or(scaled, |cap| scaled.min(cap))
                    } else {
                        0.0
                    }
                } else {
                    // A mirror's best case is its cap.
                    c.cap.unwrap_or(0.0)
                }
            }
            // Jaye: priced at the level's base limit, before peers add and his
            // own cut subtracts.
            ClauseKind::ScalingRoomOrderLimit => {
                v * f64::from(trading_base_limit(building_data, facility_counts, None))
            }
            _ => 0.0,
        };
    }
    total
}

/// The facility-count part of an operator's value: what survives an automation
/// room (a facility buff's base value rides along, as in the wipe).
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

/// Own capacity: flat plus named-teammate conditionals against `present`, which
/// INCLUDES the operator (the game's room-scoped check does too).
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
            // No roster here: the holder's own match is the floor (Astgenne's
            // own Rhine Tech skill); the ledger prices the rest.
            ClauseKind::ScalingCount {
                include_self: true, ..
            } => c.value,
            _ => 0.0,
        })
        .sum()
}

/// Order VALUE that survives a Shamare-type nullifier: flat and Precious-Metal
/// count, Pure-Gold dies. Separates a real Shamare partner from a warm body.
pub fn op_surviving_order_value(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    companions: &[OrderEffect],
) -> f64 {
    // No room level: top rarity, where value operators normally sit.
    let top = super::order_mix::rarity_for_level(building_data, i32::MAX);
    applicable_clauses(op, room_type, formula_type, registry, building_data)
        .filter(|(c, _)| c.metric == (Metric::OrderValue { pure_gold: false }))
        .filter_map(|(c, factor)| match &c.kind {
            ClauseKind::OrderMix(effect) => {
                Some(super::order_mix::optimistic_value_pct_among(effect, top, companions) * factor)
            }
            _ => None,
        })
        .sum()
}

/// Deduped order shapes `operators` field: the companions a ranking bound
/// prices a marginal against.
pub fn roster_order_effects(
    operators: &[&OperatorBaseProfile],
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> Vec<OrderEffect> {
    // Registry lookup, not a clause build: runs inside every team search.
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

/// Ranking value of the strongest POWER buff. A facility-count ENABLER (Greyy the
/// Lightningbearer's "+1 Power Plant") ranks far above drone output: it only
/// fires while STATIONED in a power plant.
pub fn op_power_rank_value(
    op: &OperatorBaseProfile,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    facility_counts: &HashMap<String, usize>,
) -> f64 {
    /// Outranks any realistic drone-recovery %.
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

/// Formula-specific clause in an UNCONFIGURED room: real but unproven. Zero
/// would treat the skill as inert, full would score speculation as realized.
/// Tunable.
pub const UNCONFIGURED_OUTPUT_FACTOR: f64 = 0.5;

/// - generic clause, or configured room with the targeted formula -> 1.0
/// - configured room with another formula -> 0.0 (mechanical fact)
/// - unconfigured room -> [`UNCONFIGURED_OUTPUT_FACTOR`]; a clause covering
///   Gold and EXP serves any factory, so it counts as generic.
fn config_factor(clause: &Clause, formula_type: Option<&str>) -> f64 {
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
        None if super::assignment::is_generic_targets(&clause.output_targets) => 1.0,
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
        // One per member: a kit never holds two skills of one type.
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
        // Positive only: a slashed limit earns nothing.
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

#[cfg(test)]
mod tests {
    // Engine behavior lives in the shadow-equivalence test and clause goldens.
    use super::*;

    #[test]
    fn config_factor_zeroes_mechanical_mismatches_only() {
        let mk = |targets: &[&str]| Clause {
            buff_id: "b".into(),
            owner_room_type: "MANUFACTURE".into(),
            metric: Metric::ManufactureSpeed,
            kind: ClauseKind::SelfValue,
            value: 10.0,
            cap: None,
            output_targets: targets.iter().map(|s| (*s).to_string()).collect(),
            non_stacking_family: None,
        };
        assert!((config_factor(&mk(&[]), Some("F_GOLD")) - 1.0).abs() < 1e-9);
        assert!((config_factor(&mk(&["F_GOLD"]), Some("F_GOLD")) - 1.0).abs() < 1e-9);
        assert!(config_factor(&mk(&["F_EXP"]), Some("F_GOLD")).abs() < 1e-9);
        // Unconfigured: discounted, never zeroed; every-family clause = generic.
        assert!((config_factor(&mk(&["F_EXP"]), None) - UNCONFIGURED_OUTPUT_FACTOR).abs() < 1e-9);
        assert!((config_factor(&mk(&["F_GOLD", "F_EXP", "F_DIAMOND"]), None) - 1.0).abs() < 1e-9);
        assert!(
            (config_factor(&mk(&["F_GOLD", "F_EXP", "F_DIAMOND"]), Some("F_DIAMOND")) - 1.0).abs()
                < 1e-9
        );
    }
}
