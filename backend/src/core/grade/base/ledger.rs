//! The ledger scorer: resolves a room's assigned operators' CLAUSES into
//! per-(entity, metric) contributions, then settles the non-linear shapes in a
//! strict order - pools, then peer scaling by deltas, then suppression LAST -
//! exactly as the algorithm reference prescribes.
//!
//! Recording every entity's contribution per metric (not just a running total)
//! is what makes suppression and peer scaling possible at all: both look
//! backward at what was already computed, not forward at raw game data.

use std::collections::{HashMap, HashSet};

use crate::core::gamedata::types::building::BuildingDataFile;

use super::assignment::CcCondition;
use super::buff_registry::BuffResolutionStrategy;
use super::clause::{
    Clause, ClauseKind, CombineRule, CondScope, Metric, Subject, SuppressExempt,
    clauses_from_strategy,
};
use super::types::OperatorBaseProfile;

/// The game floor on a trading post's order limit - a team that slashes the
/// limit below it (Degenbrecher) still banks at least one order.
const MIN_TRADING_ORDER_LIMIT: i32 = 1;
/// Fallback base order limit when neither the `TRADING_MIN_LEVEL` synthetic
/// nor gamedata trading phases are available (hand-built test contexts).
const FALLBACK_TRADING_ORDER_LIMIT: i32 = 6;

/// Fixed-point bounds. Every real case converges in one or two rounds; the
/// limits are headroom against pathological inputs, not tuned settings.
const MAX_PEER_ROUNDS: usize = 12;
const PEER_EPS: f64 = 1e-6;
/// Pool-settlement bounds, shared with the assignment-scope pass in `pools.rs`.
pub const MAX_POOL_ROUNDS: usize = 8;
pub const POOL_EPS: f64 = 1e-6;

/// Where a ledger entry came from - drives suppression exemptions and the
/// legacy mirror basis.
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
    /// Granted onto this operator by an EXTERNAL source (a per-operator
    /// Control-Center conditional, e.g. Umiri). Not the operator's own skill:
    /// a teammate nullifier (Shamare) kills it even on the nullifier's own
    /// row, because "contributions from other operators" means other SOURCES.
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
    value: f64,
    cap: Option<f64>,
    emitted: f64,
}

struct SuppressItem {
    entity: usize,
    metrics: Vec<Metric>,
}

/// The room being scored plus the shared base context.
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
    /// Operators actively working any non-dormitory room, for
    /// `RequiresChar { scope: BaseWorkArea }`. `None` = unknown (candidate
    /// enumeration) - such clauses then contribute 0.
    pub deployed_work_area: Option<&'a HashSet<String>>,
}

pub struct RoomTotals {
    pub speed_pct: f64,
    pub order_value_pct: f64,
}

/// Score one room's team: the clause walk (P0-P1), pool settlement (P2), peer
/// relaxation (P3), suppression (P4), and metric finalization (P5).
pub fn score_room(ev: &RoomEval) -> RoomTotals {
    let members: Vec<&OperatorBaseProfile> = ev
        .member_ids
        .iter()
        .filter_map(|id| ev.op_index.get(id.as_str()).copied())
        .collect();

    // Materialize each member's applicable clauses once, each tagged with
    // whether its buff is a facility-count strategy (that provenance decides
    // what survives an automation wipe, at BUFF granularity like the old
    // engine). Clauses are derived on the fly from the (possibly
    // base-wide-resolved) strategy registry; once the registry dies (CP5) this
    // reads a prebuilt clause store instead.
    let member_clauses: Vec<Vec<(Clause, bool)>> = members
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

    // The automation wipe fires only when a member's automation clause is LIVE
    // in this room (its buff applies here) - clause-gated like every other
    // shape. (The legacy engine triggered on merely CARRYING an automation
    // buff, so Weedy evaluated in a Trading Post wiped the team; that quirk
    // was reproduced through the shadow migration and fixed after.)
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

    // Per-member own capacity-limit total (Self + resolved room-scoped gates):
    // the basis for capacity-tier subjects, order-limit peer scaling, and the
    // trading throughput factor. Matches the legacy `compute_order_limit`.
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
                    _ => 0.0,
                })
                .sum()
        })
        .collect();

    // ── P2 (room-local slice): layout-derived pool settlement ────────────────
    // A generator whose points come purely from the LAYOUT (Minimalist's
    // Engineering Robots: per functional-facility level, capped) settles right
    // here: its points exist wherever the generator's clause is live. Pools
    // fed by the ASSIGNMENT (dorm occupants, resting operators) are not
    // settleable per room and wait for the assignment-scope pass - their
    // consumers read zero points, never a guess.
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

    for (i, _op) in members.iter().enumerate() {
        for (clause, from_facility) in &member_clauses[i] {
            let factor = config_factor(clause, ev.formula_type);
            // A facility-count buff's flat base rides its scaled part through
            // an automation wipe: the old engine kept or dropped whole BUFFS.
            let direct_source = if *from_facility {
                Source::RoomCountScaled
            } else {
                Source::Direct
            };
            match &clause.kind {
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
                    let count =
                        count_matches(subject, i, &members, &tags, &own_capacity, *include_self);
                    let scaled = clause.value * count;
                    let amount = clause.cap.map_or(scaled, |cap| scaled.min(cap));
                    entries.push(Entry {
                        entity: i,
                        metric: clause.metric.clone(),
                        amount: amount * factor,
                        source: Source::Direct,
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
                // Stepped pool consumer: +value per `step` settled points,
                // floored. Points come from the room-local settlement above,
                // or from the assignment-scope settlement's `POOL_<resource>`
                // synthetic; an unfed pool reads zero - never a guess.
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
                        // Resolved by registry rewrite (resolve_room_presence)
                        // before scoring; context-free passes credit 0.
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
                ClauseKind::RequiresTag { tag, scope } => {
                    let present = match scope {
                        CondScope::Room => members
                            .iter()
                            .enumerate()
                            .any(|(j, _)| j != i && tags[j].iter().any(|t| t == tag)),
                        CondScope::BaseWorkArea => false,
                        // Resolved by registry rewrite (resolve_room_presence)
                        // before scoring; context-free passes credit 0.
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
                } => peers.push(PeerItem {
                    entity: i,
                    metric_out: clause.metric.clone(),
                    basis_metric: metric.clone(),
                    include_self: *include_self,
                    step: *step,
                    value: clause.value * factor,
                    cap: clause.cap,
                    emitted: 0.0,
                }),
                // The exempt marker on automation clauses is honored via the
                // wipe's source check, not per-suppressor state.
                ClauseKind::SuppressesOthers { metrics, .. } => {
                    suppressors.push(SuppressItem {
                        entity: i,
                        metrics: metrics.clone(),
                    });
                }
                // A pre-SOLVED pool drain (perception seam) lands directly on
                // the drain channel; unsettled Generate/Convert clauses wait
                // for the native pool pass and contribute nothing here.
                ClauseKind::ResourceConvert(super::clause::ResourceOp::Consume { .. }) => {
                    entries.push(Entry {
                        entity: i,
                        metric: clause.metric.clone(),
                        amount: clause.value * factor,
                        source: Source::PoolDrain,
                    });
                }
                ClauseKind::ResourceConvert(_) => {}
                // CC globals fan out at assignment scope, not into the owner's
                // own room total. GrantsTag was consumed in P0. Unresolved and
                // FacilityCount contribute nothing here.
                ClauseKind::RoomTypeGlobal { .. }
                | ClauseKind::GrantsTag { .. }
                | ClauseKind::Unresolved => {}
            }
        }
    }

    // ── P3: peer scaling by deltas ───────────────────────────────────────────
    // (P2, pool settlement, is assignment-scoped; nothing to do per room until
    // the perception seam feeds pools - see `settle_pools`.)
    for _ in 0..MAX_PEER_ROUNDS {
        let mut max_delta = 0.0f64;
        let mut emissions: Vec<Entry> = Vec::new();
        for item in &mut peers {
            // The reference-pure basis: the FULL metric total the roommates are
            // contributing right now - facility-scaled, count-scaled and
            // pool-drained entries included. Two mirrors reading each other is
            // the genuinely-circular case the delta relaxation exists for.
            let basis_total: f64 = entries
                .iter()
                .filter(|e| e.metric == item.basis_metric)
                .filter(|e| item.include_self || e.entity != item.entity)
                .map(|e| e.amount)
                .sum();
            // A capacity basis ("per 5 CAP") counts only positive headroom and
            // quantizes by the step; a percentage mirror is continuous.
            let scaled = if item.basis_metric == Metric::CapacityLimit {
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

    // ── Per-operator Control-Center grants ──────────────────────────────────
    // Umiri-style conditionals ("all Siracusa Operators assigned to Trading
    // Posts gain +5%") buff the OPERATORS, so they enter the ledger as
    // per-entity contributions and die under a team suppressor (Shamare
    // cancels everything not sourced from herself) exactly like any other
    // teammate contribution. Threshold conditionals ("...with 3 Kjerag
    // Operators") buff the POST and stay CC-sourced (added in P5, immune to
    // suppression, like the unconditional globals).
    {
        let speed_metric = Metric::speed_for_room(ev.room_type);
        for cond in ev.cc_conditions {
            if !cond.per_operator || cond.target_room != ev.room_type {
                continue;
            }
            for (i, m) in members.iter().enumerate() {
                if m.match_tags.iter().any(|t| t == &cond.faction_token) {
                    entries.push(Entry {
                        entity: i,
                        metric: speed_metric.clone(),
                        amount: cond.bonus_pct,
                        source: Source::Granted,
                    });
                }
            }
        }
    }

    // ── P4: suppression, strictly last ───────────────────────────────────────
    let speed_metric = Metric::speed_for_room(ev.room_type);

    // Automation wipe first: the CURRENT room's speed dies for EVERYONE -
    // automation owners' non-facility buffs included - except contributions
    // granted by facility count ("excluding productivity granted based on
    // facility count"), which is why automation operators stack with each
    // other and with facility-count scalers.
    if automation_wipe {
        for e in &mut entries {
            if e.metric == speed_metric && e.source != Source::RoomCountScaled {
                e.amount = 0.0;
            }
        }
    }
    for s in &suppressors {
        for e in &mut entries {
            // The suppressor's OWN skill survives - but an external grant
            // landing on the suppressor's row (Umiri's per-operator CC bonus)
            // is another operator's contribution and dies with the rest.
            if e.entity == s.entity && e.source != Source::Granted {
                continue;
            }
            // Under an automation wipe, speed is fully governed by the wipe
            // (facility-granted parts survive a nullifier too); the nullifier
            // still applies to non-speed metrics (Pure-Gold order value).
            if automation_wipe && e.metric == speed_metric {
                continue;
            }
            if !s.metrics.contains(&e.metric) {
                continue;
            }
            e.amount = 0.0;
        }
    }

    // ── P5: finalization ─────────────────────────────────────────────────────
    let mut speed: f64 = entries
        .iter()
        .filter(|e| e.metric == speed_metric)
        .map(|e| e.amount)
        .sum();

    // Order VALUE combines max-per-entity: value operators target the same
    // orders by disjoint rules, so only the strongest applies.
    let mut order_value = 0.0f64;
    if matches!(
        Metric::OrderValue { pure_gold: true }.combine(),
        CombineRule::MaxPerEntity
    ) {
        for i in 0..members.len() {
            let entity_value: f64 = entries
                .iter()
                .filter(|e| e.entity == i && matches!(e.metric, Metric::OrderValue { .. }))
                .map(|e| e.amount)
                .sum();
            order_value = order_value.max(entity_value);
        }
    }

    // Threshold-gated Control-Center bonuses buff the POST itself, so they
    // are added here, past suppression - CC-sourced like the unconditional
    // globals. (Per-operator conditionals already entered the ledger above.)
    speed += ev
        .cc_conditions
        .iter()
        .filter(|c| !c.per_operator)
        .map(|c| c.contribution(ev.room_type, &members))
        .sum::<f64>();

    // Trading throughput is bounded by the order buffer: a slashed limit
    // throttles the post no matter how fast it acquires orders; surplus limit
    // is not rewarded. The BASE limit scales with the post's level (6/8/10 for
    // L1-L3, gamedata `TradingData`), resolved through the `TRADING_MIN_LEVEL`
    // synthetic so search and display always agree.
    if ev.room_type == "TRADING" {
        let phases = &ev.building_data.trading_data.phases;
        let base_limit = ev
            .facility_counts
            .get(super::assignment::TRADING_MIN_LEVEL)
            .and_then(|lv| phases.get(lv.saturating_sub(1)))
            .or_else(|| phases.last())
            .map_or(FALLBACK_TRADING_ORDER_LIMIT, |p| p.order_limit);
        // Named-operator CC grants ("that Trading Post's order limit +2"
        // while Hoederer is seated here) add to the same pool as the crew's
        // own capacity skills. CC-sourced, so no suppressor touches them.
        let cc_capacity: f64 = ev
            .cc_conditions
            .iter()
            .map(|c| c.capacity_contribution(ev.room_type, &members))
            .sum();
        let net_limit: f64 = own_capacity.iter().sum::<f64>() + cc_capacity;
        #[allow(clippy::cast_possible_truncation)]
        let effective = (base_limit + net_limit.round() as i32).max(MIN_TRADING_ORDER_LIMIT);
        let capacity = (f64::from(effective) / f64::from(base_limit)).min(1.0);
        speed = ((1.0 + speed / 100.0) * capacity - 1.0) * 100.0;
    }

    RoomTotals {
        speed_pct: speed,
        order_value_pct: order_value,
    }
}

/// One operator's applicable clauses in a room: derived from the registry,
/// gated on the owner's room type and the configuration discount, each paired
/// with its config factor. The shared iteration under every per-operator
/// clause query below.
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

/// An optimistic upper bound on an operator's solo value in a room, used only
/// for ranking candidates: every gate assumed satisfied, every teammate scaler
/// assumed fully matched, facility scaling at real counts. Never a final score
/// (the ledger is), so it must only stay >= the true value.
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
) -> f64 {
    let teammates_assumed = max_slots.saturating_sub(1) as f64;
    let speed_metric = Metric::speed_for_room(room_type);
    let counts_toward_bound =
        |m: &Metric| *m == speed_metric || matches!(m, Metric::OrderValue { .. });
    // The op's own layout-derived pools (generator + consumer travel together,
    // e.g. Minimalist), so their consumer value is exact, not optimistic.
    let functional_levels = facility_counts
        .get(super::assignment::FUNCTIONAL_LEVEL_SUM)
        .copied()
        .unwrap_or(0) as f64;
    let own_pools: HashMap<String, f64> =
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
                _ => None,
            })
            .collect();
    let mut total = 0.0;
    for (c, factor) in applicable_clauses(op, room_type, formula_type, registry, building_data) {
        if !counts_toward_bound(&c.metric) {
            continue;
        }
        let v = c.value * factor;
        total += match &c.kind {
            ClauseKind::SelfValue => v,
            // A solved pool payoff counts at face value - the consumer must
            // rank high enough to be SEATED for the pool to pay out at all.
            ClauseKind::ResourceConvert(super::clause::ResourceOp::Consume { .. }) => v,
            // A stepped pool consumer fed by the op's own generator: exact.
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
                    // Peer-capacity tiers: assume everyone clears the threshold.
                    Subject::PeerMetricAbove { .. }
                    | Subject::AnyOtherOccupant
                    | Subject::Tag(_)
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
                    // Capacity conversion: TRADING posts almost never field
                    // capacity-adders, so keep their bound at zero; FACTORIES
                    // routinely stack them (Vermeil's whole comp), so credit an
                    // optimistic capacity-rich room there - otherwise she'd rank
                    // at zero and be cut before the scorer tries her with the
                    // teammates that make her strong.
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
            _ => 0.0,
        };
    }
    total
}

/// Only the facility-count-granted part of an operator's value - what survives
/// in an automation room (base value of a facility buff rides along, matching
/// the wipe's buff-granular survival rule).
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

/// An operator's own capacity-limit total in a room: flat contributions plus
/// named-teammate conditional capacity resolved against `present` (which
/// INCLUDES the operator, mirroring the game's room-scoped check).
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
            _ => 0.0,
        })
        .sum()
}

/// An operator's order VALUE that survives a Shamare-type nullifier: flat and
/// Precious-Metal value counts, Pure-Gold value dies with the Pure-Gold orders
/// the nullifier shifts the post away from. What separates a genuine Shamare
/// partner from a warm body.
pub fn op_surviving_order_value(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> f64 {
    applicable_clauses(op, room_type, formula_type, registry, building_data)
        .filter(|(c, _)| c.metric == (Metric::OrderValue { pure_gold: false }))
        .map(|(c, factor)| c.value * factor)
        .sum()
}

/// Ranking value of an operator's strongest POWER-room buff: solo clause value
/// at real facility counts, with a facility-count ENABLER (Greyy the
/// Lightningbearer's "+1 Power Plant") ranked far above ordinary drone output -
/// it must be STATIONED in a power plant for the count bonus to fire.
pub fn op_power_rank_value(
    op: &OperatorBaseProfile,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    facility_counts: &HashMap<String, usize>,
) -> f64 {
    /// Rank weight per unit of facility count granted (an enabler outranks any
    /// realistic drone-recovery percentage).
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

/// Discount for a formula-specific clause in an UNCONFIGURED room. The value
/// is real - were the room configured to the matching product, it would apply
/// in full - but the live snapshot doesn't prove it, so it is discounted:
/// never zeroed (that punishes unconfigured rooms as if the skill were inert)
/// and never full (that scores speculative output as realized). Tunable.
pub const UNCONFIGURED_OUTPUT_FACTOR: f64 = 0.5;

/// The three-tier configuration discount:
/// - generic clause, or configured room whose formula the clause targets → 1.0
/// - configured room whose formula the clause does NOT target → 0.0 - a
///   mechanical fact, never discounted
/// - unconfigured room → [`UNCONFIGURED_OUTPUT_FACTOR`] - real but unproven,
///   never zeroed. A clause covering both Gold and EXP serves any factory
///   equally, so it counts as generic.
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

/// How many entities match a scaling subject.
fn count_matches(
    subject: &Subject,
    owner: usize,
    members: &[&OperatorBaseProfile],
    tags: &[Vec<String>],
    own_capacity: &[f64],
    include_self: bool,
) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let n = match subject {
        Subject::Tag(token) => (0..members.len())
            .filter(|&j| j != owner || include_self)
            .filter(|&j| tags[j].iter().any(|t| t == token))
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
                    .filter(|&j| own_capacity[j] > *threshold)
                    .count()
            } else {
                0
            }
        }
    };
    n as f64
}

#[cfg(test)]
mod tests {
    // Engine-level behaviors are exercised through the shadow-equivalence
    // integration test and the clause goldens; the pure-function tests here
    // cover the pieces with no legacy counterpart.
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
        // Unconfigured room: formula-specific value is discounted, never
        // zeroed; a clause serving every family is generic - always full.
        assert!((config_factor(&mk(&["F_EXP"]), None) - UNCONFIGURED_OUTPUT_FACTOR).abs() < 1e-9);
        assert!((config_factor(&mk(&["F_GOLD", "F_EXP", "F_DIAMOND"]), None) - 1.0).abs() < 1e-9);
        assert!(
            (config_factor(&mk(&["F_GOLD", "F_EXP", "F_DIAMOND"]), Some("F_DIAMOND")) - 1.0).abs()
                < 1e-9
        );
    }
}
