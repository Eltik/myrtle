//! Assignment-scope pool settlement (stage 2a of the pool pass): once an
//! assignment is KNOWN - the player's live base, where every seat is real -
//! generators whose points depend on WHERE their owner sits can settle, and
//! conversion chains relax to a fixed point. The settled totals ride the
//! `facility_counts` synthetic channel (`POOL_<resource>`) into the room
//! scorer, so consumers price them with zero signature changes.
//!
//! Search paths (candidate enumeration, the rotation planner) have no fixed
//! assignment, read no synthetics, and price these consumers at zero - the
//! conservative side of never-guess. Seat incentives for generators are the
//! next stage.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;

use crate::core::gamedata::types::building::{Buff, BuildingDataFile};

use super::buff_registry::BuffResolutionStrategy;
use super::clause::{ClauseKind, PoolBasis, ResourceOp, clauses_from_strategy};
use super::ledger::{MAX_POOL_ROUNDS, POOL_EPS};
use super::types::{OperatorBaseProfile, UserBuilding};

/// `facility_counts` key prefix for settled pool points ("`POOL_bd_dungeon`").
pub(crate) const POOL_PREFIX: &str = "POOL_";

/// Pseudo-pool: the count of Robot-tagged operators seated in Power Plants
/// (the game's "Operation Platform" term, `$cc.tag.op`) - Alanna's basis.
pub const ROBOTS_IN_POWER: &str = "tag_op_in_power";

/// The steady-state fraction of a work block a "when own Morale is above/below
/// N" condition holds: a full bar drains linearly at the baseline rate across
/// a 24h block, so it sits above N for (MAX-N)/MAX of the block and below N
/// for N/MAX. A documented model derived from the same gamedata rates the
/// simulator uses - not a guess, and not fixed to any particular threshold.
fn morale_condition_weight(above: bool, threshold: f64) -> f64 {
    use super::sustain_sim::MORALE_MAX;
    let frac = if above {
        (MORALE_MAX - threshold) / MORALE_MAX
    } else {
        threshold / MORALE_MAX
    };
    frac.clamp(0.0, 1.0)
}

// ── Generator side-channel ───────────────────────────────────────────────────
// Some buffs (Dusk/Ling/Chongyue's Control-Center skills) carry BOTH an effect
// the parser owns as the buff's strategy (a morale aura) AND a pool grant. The
// grant half is extracted here straight from the description - a side-channel
// like `morale_drains`, owned entirely by the settlement, and applied wherever
// the buff's own room type says its owner must sit.

/// Morale-conditional flat grants: "when self/own Morale is above/below N,
/// <Resource> +M" (both of Ling's branches match via `captures_iter`).
static RE_MORALE_COND_GRANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[Mm]orale is (above|below)\s*<@cc\.kw>([\d.]+)</>,\s*<\$cc\.([A-Za-z0-9_]+)>[^+]{0,40}?<@cc\.vup>\+([\d.]+)</>").unwrap()
});

/// An UNCONDITIONAL flat grant: "..., <Resource> +N" with no counter or
/// condition governing it. The regex alone over-captures (the same textual
/// shape ends conditional and per-operator clauses), so every match must pass
/// [`flat_grant_unconditional`] - audited 2026-08-13: the pair accepts exactly
/// the unconditional Control-Center flats (Passion +20/+10/+10, Felvine +8)
/// and rejects every morale-conditional, per-operator, per-dorm-occupant and
/// recruit-slot form (each owned by its own regex or strategy).
static RE_FLAT_GRANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<\$cc\.(bd_[A-Za-z0-9_]+)><@cc\.rem>[^<]+</></>\s*<@cc\.vup>\+([\d.]+)</>")
        .unwrap()
});

/// True when the settlement's grant side-channel captures this text: a buff
/// whose ONLY other clause is Unresolved is then fully priced without a
/// strategy (Dolris' "Idol's Aura" is purely a dorm-occupancy Passion grant),
/// so its Unresolved marker would be label pessimism.
pub(crate) fn has_side_channel_grant(desc: &str) -> bool {
    RE_MORALE_COND_GRANT.is_match(desc)
        || RE_FACTION_GRANT.is_match(desc)
        || RE_TAG_GRANT.is_match(desc)
        || RE_DORM_OCC_GRANT.is_match(desc)
        || RE_FLAT_GRANT
            .captures_iter(desc)
            .any(|c| flat_grant_unconditional(desc, c.get(0).map_or(0, |m| m.start())))
}

/// True when the buff's PARSED strategy already emits a Generate for
/// `resource` - the side-channel must not re-capture a grant the clause layer
/// owns (the Sui generators parse whole; re-reading their text double-counts).
fn strategy_generates(
    registry: &HashMap<String, BuffResolutionStrategy>,
    buff_id: &str,
    buff: &Buff,
    resource: &str,
) -> bool {
    registry.get(buff_id).is_some_and(|s| {
        clauses_from_strategy(buff_id, buff, s).iter().any(|c| {
            matches!(
                &c.kind,
                ClauseKind::ResourceConvert(ResourceOp::Generate { resource: r, .. })
                    if r == resource
            )
        })
    })
}

/// True when the grant at `start` is a plain flat (not governed by a counter
/// or condition): no counting/conditional keyword in the preceding window.
fn flat_grant_unconditional(desc: &str, start: usize) -> bool {
    // 160 chars reaches past the longest counter phrase ("for each <Sui>
    // Operator assigned to buildings other than Dormitories and Activity
    // Rooms,") while the true flats' windows hold only the CC intro.
    let mut from = start.saturating_sub(160);
    while from > 0 && !desc.is_char_boundary(from) {
        from -= 1;
    }
    // Case-insensitive: Dusk's rider reads "when self morale is above 12,
    // Perception Information +10" - a morale-conditional grant the flat
    // channel must not count a second time.
    let window = desc[from..start].to_lowercase();
    ![
        "for each",
        "for every",
        "operator in",
        "operators in",
        "morale is",
        "slot",
    ]
    .iter()
    .any(|kw| window.contains(kw))
}

/// A simple deployed-tag counter: "for each <tag.X> ... Operator, <Resource>
/// +P" (the Felvine generator; no cap, unlike the Sui faction counter).
/// The deployment-independent pool grants a buff's TEXT carries beside the
/// effect the parser owns: unconditional flat grants (Dusk's "Perception
/// Information +10" rider, guarded so a parsed generator is never counted
/// twice), morale-conditional grants at their steady-state weight, and
/// dorm-occupancy counters settled against `dorm_occupants`. Faction and
/// deployed-tag counters need the deployment and stay with their callers.
/// One extractor for the live settlement, the dorm economies and the
/// Control-Center grant bundles, so every path reads the same points.
/// `current_morale` is the owner's REAL bar when the caller knows it (the live
/// settlement, from the sync's last write): a morale-conditional grant
/// then reads as the game shows it - all or nothing by the condition -
/// instead of its steady-state time-share.
fn text_grants(
    buff_id: &str,
    buff: &Buff,
    registry: &HashMap<String, BuffResolutionStrategy>,
    dorm_occupants: f64,
    current_morale: Option<f64>,
) -> Vec<(String, f64)> {
    let mut grants: Vec<(String, f64)> = Vec::new();
    for c in RE_MORALE_COND_GRANT.captures_iter(&buff.description) {
        let above = &c[1] == "above";
        let threshold: f64 = c[2].parse().unwrap_or(0.0);
        let amount: f64 = c[4].parse().unwrap_or(0.0);
        let weight = match current_morale {
            Some(m) => f64::from(u8::from(if above { m > threshold } else { m < threshold })),
            None => morale_condition_weight(above, threshold),
        };
        grants.push((c[3].to_string(), amount * weight));
    }
    for c in RE_FLAT_GRANT.captures_iter(&buff.description) {
        if flat_grant_unconditional(&buff.description, c.get(0).map_or(0, |m| m.start()))
            && !strategy_generates(registry, buff_id, buff, &c[1])
        {
            grants.push((c[1].to_string(), c[2].parse().unwrap_or(0.0)));
        }
    }
    if let Some(c) = RE_DORM_OCC_GRANT.captures(&buff.description)
        && !strategy_generates(registry, buff_id, buff, &c[1])
    {
        let per: f64 = c[2].parse().unwrap_or(0.0);
        grants.push((c[1].to_string(), per * dorm_occupants));
    }
    grants
}

static RE_TAG_GRANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"for each <\$cc\.tag\.([a-z0-9_]+)>.{0,80}?Operator,\s*<\$cc\.(bd_[A-Za-z0-9_]+)>[^+]{0,40}?<@cc\.vup>\+([\d.]+)</>",
    )
    .unwrap()
});

/// A dorm-occupancy counter: "for every Operator in the Dormitories,
/// <Resource> +P" (Sakiko's Passion generator).
static RE_DORM_OCC_GRANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"for <@cc\.vup>every</> Operator in the Dormitories,\s*<\$cc\.(bd_[A-Za-z0-9_]+)>[^+]{0,40}?<@cc\.vup>\+([\d.]+)</>",
    )
    .unwrap()
});

/// True when one of `op`'s buffs carries a MORALE-CONDITIONAL pool grant
/// (Ling's "when own Morale is above/below N, <Resource> +M") - such a
/// generator only sustains its grant with a morale-swap manager holding it at
/// the right side of the bar, so a plan pinning one should also reserve the
/// manager.
pub fn has_morale_conditional_grant(
    op: &OperatorBaseProfile,
    building_data: &BuildingDataFile,
) -> bool {
    op.available_buffs.iter().any(|b| {
        building_data
            .buffs
            .get(b)
            .is_some_and(|buff| RE_MORALE_COND_GRANT.is_match(&buff.description))
    })
}

/// A deployed-faction counter: "for each <g.tag> Operator assigned to
/// buildings other than Dormitories ..., <Resource>+P (max CAP)".
static RE_FACTION_GRANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"for each <\$cc\.g\.([a-z0-9_]+)>.{0,60}?assigned to buildings other than.{0,80}?<\$cc\.([A-Za-z0-9_]+)>[^+]{0,40}?<@cc\.vup>\+([\d.]+)</>\s*\(max ([\d.]+)\)").unwrap()
});

/// Settle every assignment-fed pool against the player's LIVE seats: walk each
/// room's stationed operators, run the generators whose basis needs a seat
/// (own-room-level), then relax conversion chains batched until stable.
/// Layout-derived generators (`PoolBasis::FunctionalLevels`) are deliberately
/// skipped - the ledger settles those room-locally wherever the generator's
/// clause is live, and settling them here too would double-count.
/// `live_morale` is each seated operator's bar as the game last wrote it
/// (empty when the sync carries none): Dusk's "when self morale is above 12,
/// Perception Information +10" counts the full 10 while she IS above 12, as
/// the game showed it, and nothing once she has dropped below.
pub(crate) fn settle_current_pools(
    building: &UserBuilding,
    operators: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    live_morale: &HashMap<String, f64>,
) -> HashMap<String, f64> {
    let by_id: HashMap<&str, &OperatorBaseProfile> =
        operators.iter().map(|o| (o.char_id.as_str(), o)).collect();

    // Operators resting in the dormitories, the basis for dorm-count
    // generators (Rosmontis' Perception Information, Mr. Nothing's Worldly
    // Plight): only seats the sync actually shows, never an assumption.
    #[allow(clippy::cast_precision_loss)]
    let dorm_occupants = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "DORMITORY")
        .map(|r| r.current_operators.len())
        .sum::<usize>() as f64;

    // Deployed operators (seated anywhere but a dormitory), for faction
    // counters and the robot pseudo-pool.
    let deployed: Vec<&OperatorBaseProfile> = building
        .rooms
        .iter()
        .filter(|r| r.room_type != "DORMITORY")
        .flat_map(|r| r.current_operators.iter())
        .filter_map(|id| by_id.get(id.as_str()).copied())
        .collect();
    #[allow(clippy::cast_precision_loss)]
    let deployed_with_tag = |tag: &str| -> f64 {
        deployed
            .iter()
            .filter(|op| op.match_tags.iter().any(|t| t == tag))
            .count() as f64
    };

    let mut points: HashMap<String, f64> = HashMap::new();
    // Conversion clauses live wherever their owner is actually seated.
    let mut converts: Vec<(String, String, f64)> = Vec::new();

    // The robot pseudo-pool: Robot-tagged operators seated in Power Plants
    // (the game's "Operation Platform" count Alanna's skill reads).
    #[allow(clippy::cast_precision_loss)]
    let robots_in_power = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "POWER")
        .flat_map(|r| r.current_operators.iter())
        .filter_map(|id| by_id.get(id.as_str()).copied())
        .filter(|op| op.match_tags.iter().any(|t| t == "robot"))
        .count() as f64;
    if robots_in_power > 0.0 {
        points.insert(ROBOTS_IN_POWER.to_string(), robots_in_power);
    }

    for room in &building.rooms {
        for id in &room.current_operators {
            let Some(op) = by_id.get(id.as_str()) else {
                continue;
            };
            for buff_id in &op.available_buffs {
                let (Some(buff), Some(strategy)) =
                    (building_data.buffs.get(buff_id), registry.get(buff_id))
                else {
                    continue;
                };
                for clause in clauses_from_strategy(buff_id, buff, strategy) {
                    if clause.owner_room_type != room.room_type {
                        continue;
                    }
                    match &clause.kind {
                        ClauseKind::ResourceConvert(ResourceOp::Generate { resource, basis }) => {
                            let generated = match basis {
                                PoolBasis::OwnRoomLevel => {
                                    clause.value * f64::from(room.level.max(0))
                                }
                                PoolBasis::DormOccupants => clause.value * dorm_occupants,
                                // Layout-derived pools settle room-locally in
                                // the ledger; the other bases wait for their
                                // side-channel parsers.
                                _ => continue,
                            };
                            let total = points.entry(resource.clone()).or_insert(0.0);
                            *total = clause
                                .cap
                                .map_or(*total + generated, |cap| (*total + generated).min(cap));
                        }
                        ClauseKind::ResourceConvert(ResourceOp::Convert { from, to, ratio }) => {
                            converts.push((from.clone(), to.clone(), *ratio));
                        }
                        _ => {}
                    }
                }
            }

            // The generator side-channel: a seated operator's buffs for THIS
            // room may carry a pool grant alongside whatever effect the parser
            // owns. Extract the grant half from the text directly.
            for buff_id in &op.available_buffs {
                let Some(buff) = building_data.buffs.get(buff_id) else {
                    continue;
                };
                if buff.room_type != room.room_type {
                    continue;
                }
                let current_morale = live_morale.get(id.as_str()).copied();
                for (resource, amount) in
                    text_grants(buff_id, buff, registry, dorm_occupants, current_morale)
                {
                    *points.entry(resource).or_insert(0.0) += amount;
                }
                if let Some(c) = RE_FACTION_GRANT.captures(&buff.description) {
                    let per: f64 = c[3].parse().unwrap_or(0.0);
                    let unit_cap: f64 = c[4].parse().unwrap_or(f64::INFINITY);
                    let units = deployed_with_tag(&c[1]).min(unit_cap);
                    *points.entry(c[2].to_string()).or_insert(0.0) += per * units;
                }
                if let Some(c) = RE_TAG_GRANT.captures(&buff.description)
                    && !strategy_generates(registry, buff_id, buff, &c[2])
                {
                    let per: f64 = c[3].parse().unwrap_or(0.0);
                    *points.entry(c[2].to_string()).or_insert(0.0) +=
                        per * deployed_with_tag(&c[1]);
                }
            }
        }
    }

    // Batched fixed-point relaxation of conversion chains: each round reads
    // the pool state as it stood at the round's start and applies all moves
    // together, so the result never depends on clause order. `ratio` is
    // from-points-per-to-point, floored like every stepped game counter.
    // Converters COPY (base expert, 2026-09-08: every reader of a pool sees
    // the whole of it - Jieyun's Witchcraft Crystals do not take Worldly
    // Plight away from Shu or Mr. Nothing), each converter once, so a chain
    // still settles over the rounds.
    let mut done: HashSet<usize> = HashSet::new();
    for _ in 0..MAX_POOL_ROUNDS {
        let snapshot = points.clone();
        let mut moved = 0.0f64;
        for (ci, (from, to, ratio)) in converts.iter().enumerate() {
            if done.contains(&ci) || *ratio <= 0.0 {
                continue;
            }
            let available = snapshot.get(from).copied().unwrap_or(0.0);
            let converted = (available / ratio).floor();
            if converted > 0.0 {
                done.insert(ci);
                *points.entry(to.clone()).or_insert(0.0) += converted;
                moved += converted;
            }
        }
        if moved < POOL_EPS {
            break;
        }
    }

    points.retain(|_, v| *v > 0.0);
    points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion_chain_relaxes_to_fixed_point() {
        // 17 A, converted 5:1 into B, then 3:1 into C: 17A -> 3B (2A left),
        // 3B -> 1C (0B left). Two rounds, floored at each hop.
        let mut points: HashMap<String, f64> = HashMap::from([("A".into(), 17.0)]);
        let converts = vec![
            ("A".to_string(), "B".to_string(), 5.0),
            ("B".to_string(), "C".to_string(), 3.0),
        ];
        for _ in 0..MAX_POOL_ROUNDS {
            let snapshot = points.clone();
            let mut moved = 0.0f64;
            for (from, to, ratio) in &converts {
                let available = snapshot.get(from).copied().unwrap_or(0.0);
                let converted = (available / ratio).floor();
                if converted > 0.0 {
                    *points.entry(from.clone()).or_insert(0.0) -= converted * ratio;
                    *points.entry(to.clone()).or_insert(0.0) += converted;
                    moved += converted;
                }
            }
            if moved < POOL_EPS {
                break;
            }
        }
        assert!((points["A"] - 2.0).abs() < 1e-9, "A remainder: {points:?}");
        assert!((points["B"] - 0.0).abs() < 1e-9, "B drained: {points:?}");
        assert!((points["C"] - 1.0).abs() < 1e-9, "C settled: {points:?}");
    }
}

// ── Seat incentives: the optimizer half ──────────────────────────────────────
// The search scores candidate teams with no assignment context, so pool
// consumers read zero there. This planner runs BEFORE the optimal search and
// solves the economies the roster can field, exactly like the perception
// module's proven pattern: consumer buffs get PoolPayoff overrides (so the
// search VALUES them) and generators that must sit somewhere specific get
// pinned seats.
//
// The honesty rule: a consumer is credited ONLY when every generator (and
// converter) feeding its pool is either the consumer themself - seated by the
// same recommendation that seats the consumer - or explicitly PINNED by this
// plan. Value flowing from a third operator the search may never seat is
// phantom value, and stays at zero until joint-seating economics land.

/// A solved economy plan for the optimal search.
#[derive(Debug, Default)]
pub struct EconomyPlan {
    /// `buff_id -> solved productivity %` - consumer buffs to override with
    /// [`BuffResolutionStrategy::PoolPayoff`].
    pub overrides: Vec<(String, f64)>,
    /// `(char_id, room_type)` generator seats the plan reserves (Senshi into
    /// the best dormitory).
    pub pins: Vec<(String, String)>,
    /// `(buff_id, target_room, solved total %)` pool-scaled Control-Center
    /// globals (Sakiko's trading global), to fold as
    /// [`BuffResolutionStrategy::GlobalEffect`].
    pub globals: Vec<(String, String, f64)>,
}

/// Projected dorm occupancy at steady state: dorms hold whoever isn't
/// working, capped by their slots. A deep roster keeps them full; a
/// shallow one can't. Grounded in the layout and roster, not assumed.
fn projected_dorm_occupancy(
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
) -> f64 {
    use super::assignment::total_dorm_capacity;
    use super::util::max_stationed_at_level;
    let working_seats: usize = building
        .rooms
        .iter()
        .filter(|r| r.room_type != "DORMITORY")
        .map(|r| max_stationed_at_level(building_data, &r.room_type, r.level).max(0) as usize)
        .sum();
    #[allow(clippy::cast_sign_loss)]
    let dorm_capacity = total_dorm_capacity(building, building_data).max(0) as usize;
    #[allow(clippy::cast_precision_loss)]
    let occ = dorm_capacity.min(profiles.len().saturating_sub(working_seats)) as f64;
    occ
}

/// One dorm-fed or room-level pool economy of the roster: its generators
/// (with the room their owner must occupy), converters and consumers, and
/// the projected dorm occupancy they settle against.
struct DormEconomy {
    gens: Vec<Gen>,
    /// (converter owner, from, to, ratio)
    converts: Vec<(String, String, String, f64)>,
    /// (owner, buff id, resource, step, pct)
    consumers: Vec<(String, String, String, f64, f64)>,
}

struct Gen {
    owner: String,
    /// The room type the generating skill needs its owner in.
    owner_room: String,
    resource: String,
    points: f64,
    /// A pin this generator needs to produce (own-room-level seats).
    pin: Option<String>,
    /// A parsed generator clause (the dorm economies proper) rather than a
    /// text side-channel grant riding along: only native origins anchor a
    /// shared-pool bundle, side-channel origins join one as co-feeders.
    native: bool,
}

fn collect_dorm_economy(
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> DormEconomy {
    let projected_occupancy = projected_dorm_occupancy(profiles, building, building_data);
    let best_room_level = |room_type: &str| -> f64 {
        f64::from(
            building
                .rooms
                .iter()
                .filter(|r| r.room_type == room_type)
                .map(|r| r.level)
                .max()
                .unwrap_or(0),
        )
    };
    let mut econ = DormEconomy {
        gens: Vec::new(),
        converts: Vec::new(),
        consumers: Vec::new(),
    };
    for op in profiles {
        for buff_id in &op.available_buffs {
            let Some(buff) = building_data.buffs.get(buff_id) else {
                continue;
            };
            // Text side-channel grants (Dusk's Control-Center "Perception
            // Information +10" rider) feed the SAME pool the dorm generators
            // fill - the base expert confirmed 2026-09-08 that Dusk, Iris,
            // Czerny and Whisperain all stack into Rosmontis' count from
            // their own resources. They are origins the shared-pool bundles
            // may pin (into the room the grant's buff requires), never a
            // pin the native plan forces.
            for (resource, points) in
                text_grants(buff_id, buff, registry, projected_occupancy, None)
            {
                econ.gens.push(Gen {
                    owner: op.char_id.clone(),
                    owner_room: buff.room_type.clone(),
                    resource,
                    points,
                    pin: None,
                    native: false,
                });
            }
            let Some(strategy) = registry.get(buff_id) else {
                continue;
            };
            for clause in clauses_from_strategy(buff_id, buff, strategy) {
                match &clause.kind {
                    ClauseKind::ResourceConvert(ResourceOp::Generate { resource, basis }) => {
                        let (points, pin) = match basis {
                            PoolBasis::OwnRoomLevel => {
                                let lvl = best_room_level(&clause.owner_room_type);
                                (clause.value * lvl, Some(clause.owner_room_type.clone()))
                            }
                            PoolBasis::DormOccupants => (clause.value * projected_occupancy, None),
                            // Layout pools settle inside the scorer already;
                            // other bases have no search story yet.
                            _ => continue,
                        };
                        econ.gens.push(Gen {
                            owner: op.char_id.clone(),
                            owner_room: clause.owner_room_type.clone(),
                            resource: resource.clone(),
                            points: clause.cap.map_or(points, |cap| points.min(cap)),
                            pin,
                            native: true,
                        });
                    }
                    ClauseKind::ResourceConvert(ResourceOp::Convert { from, to, ratio }) => {
                        econ.converts
                            .push((op.char_id.clone(), from.clone(), to.clone(), *ratio));
                    }
                    ClauseKind::ScalingPoolPoints { resource, step } => {
                        econ.consumers.push((
                            op.char_id.clone(),
                            buff_id.clone(),
                            resource.clone(),
                            *step,
                            clause.value,
                        ));
                    }
                    _ => {}
                }
            }
        }
    }
    econ
}

/// Settle an economy's pools per ORIGIN operator: `resource -> origin ->
/// points`. Converters COPY points onward (the game credits every converter
/// the whole pool - the live settlement shows Rosmontis and Ebenholz each
/// reading the full Perception Information), and a converted point keeps
/// the origin that generated it, so a consumer can be priced on exactly the
/// origins the plan can vouch for.
fn settle_by_origin(econ: &DormEconomy) -> HashMap<String, HashMap<String, f64>> {
    let mut pools: HashMap<String, HashMap<String, f64>> = HashMap::new();
    for g in &econ.gens {
        *pools
            .entry(g.resource.clone())
            .or_default()
            .entry(g.owner.clone())
            .or_insert(0.0) += g.points;
    }
    let mut done: HashSet<(usize, String)> = HashSet::new();
    for _ in 0..MAX_POOL_ROUNDS {
        let snapshot = pools.clone();
        let mut moved = 0.0f64;
        for (ci, (_, from, to, ratio)) in econ.converts.iter().enumerate() {
            if *ratio <= 0.0 {
                continue;
            }
            let Some(origins) = snapshot.get(from) else {
                continue;
            };
            for (origin, available) in origins {
                if !done.insert((ci, origin.clone())) {
                    continue;
                }
                let converted = (available / ratio).floor();
                if converted > 0.0 {
                    *pools
                        .entry(to.clone())
                        .or_default()
                        .entry(origin.clone())
                        .or_insert(0.0) += converted;
                    moved += converted;
                }
            }
        }
        if moved < POOL_EPS {
            break;
        }
    }
    pools
}

/// Solve the roster's clean pool economies for the optimal view.
pub fn plan_optimal_economies(
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> EconomyPlan {
    let mut plan = EconomyPlan::default();
    let econ = collect_dorm_economy(profiles, building, building_data, registry);
    let mut pinned: Vec<String> = Vec::new();
    for g in &econ.gens {
        if let Some(room) = &g.pin {
            plan.pins.push((g.owner.clone(), room.clone()));
            pinned.push(g.owner.clone());
        }
    }
    let pools = settle_by_origin(&econ);

    // Price each consumer under the honesty rule, per ORIGIN: only the points
    // the consumer themself or a pinned generator produced count. A co-feeder
    // the search might not seat (Ebenholz beside Rosmontis) adds nothing here
    // - the shared-pool bundle below offers that seating to the oracle.
    for (owner, buff_id, resource, step, pct) in &econ.consumers {
        let Some(origins) = pools.get(resource) else {
            continue;
        };
        let points: f64 = origins
            .iter()
            .filter(|(origin, _)| *origin == owner || pinned.contains(origin))
            .map(|(_, p)| p)
            .sum();
        if points <= 0.0 || *step <= 0.0 {
            continue;
        }
        let solved = (points / step).floor() * pct;
        if solved > 0.0 {
            plan.overrides.push((buff_id.clone(), solved));
        }
    }
    plan
}

/// Joint-seating bundles for SHARED dorm-fed pools: a consumer whose pool is
/// also fed by other operators (Rosmontis' Chain of Thought draws on
/// Ebenholz's Musicianship) gets one bundle that pins those co-feeders into
/// the rooms their generators need and prices every consumer of the pool at
/// the full total. The oracle keeps it only if the seats pay for themselves.
fn shared_pool_bundles(
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> Vec<EconomyPlan> {
    let econ = collect_dorm_economy(profiles, building, building_data, registry);
    let pools = settle_by_origin(&econ);
    let mut bundles = Vec::new();
    let mut seen: HashSet<Vec<String>> = HashSet::new();
    let native_owners: HashSet<&str> = econ
        .gens
        .iter()
        .filter(|g| g.native)
        .map(|g| g.owner.as_str())
        .collect();
    for (owner, _, resource, _, _) in &econ.consumers {
        let Some(origins) = pools.get(resource) else {
            continue;
        };
        // A pool fed only by side-channel grants (the Sui Control-Center
        // economy) is the grant-carrier bundles' business, not a dorm pool.
        if !origins.keys().any(|o| native_owners.contains(o.as_str())) {
            continue;
        }
        let mut others: Vec<String> = origins
            .keys()
            .filter(|origin| *origin != owner)
            .cloned()
            .collect();
        others.sort();
        if others.is_empty() {
            continue;
        }
        // Every co-feeder SUBSET is its own bundle: a seat the oracle rejects
        // (Ebenholz's Trading-Post pin) must not sink the co-feeders that pay
        // for themselves (Dusk's Control-Center seat).
        for subset in cofeeder_subsets(&others) {
            if !seen.insert(subset.clone()) {
                continue;
            }
            let pins: Vec<(String, String)> = subset
                .iter()
                .filter_map(|id| {
                    econ.gens
                        .iter()
                        .find(|g| &g.owner == id)
                        .map(|g| (id.clone(), g.owner_room.clone()))
                })
                .collect();
            // Every consumer fed by this pool set is priced on its own origin
            // plus the pinned ones - the points this bundle can vouch for.
            let overrides: Vec<(String, f64)> = econ
                .consumers
                .iter()
                .filter_map(|(c_owner, buff_id, c_res, step, pct)| {
                    let pts: f64 = pools
                        .get(c_res)?
                        .iter()
                        .filter(|(origin, _)| *origin == c_owner || subset.contains(origin))
                        .map(|(_, p)| p)
                        .sum();
                    (*step > 0.0 && pts > 0.0)
                        .then(|| (buff_id.clone(), (pts / step).floor() * pct))
                        .filter(|(_, v)| *v > 0.0)
                })
                .collect();
            if !overrides.is_empty() {
                bundles.push(EconomyPlan {
                    globals: Vec::new(),
                    overrides,
                    pins,
                });
            }
        }
    }
    bundles
}

/// Facility-count modifiers as seat bundles: Eunectes' "+2 Power Plants"
/// needs her in the Control Center and Lancet-2 in a Power Plant, Greyy's
/// "+1" needs her plant seat. Offered only when the roster fields an
/// automation scaler that reads the count (Weedy, Eunectes, Pudding); the
/// oracle keeps the seats if the boosted count pays for them.
fn facility_count_bundles(
    profiles: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> Vec<EconomyPlan> {
    use super::buff_registry::FacilityGate;
    let owned: HashSet<&str> = profiles.iter().map(|p| p.char_id.as_str()).collect();
    let has_scaler = profiles.iter().any(|p| {
        p.available_buffs.iter().any(|b| {
            matches!(
                registry.get(b),
                Some(BuffResolutionStrategy::FacilityCountScaling { .. })
            )
        })
    });
    if !has_scaler {
        return Vec::new();
    }
    let mut bundles = Vec::new();
    for op in profiles {
        for buff_id in &op.available_buffs {
            let Some(BuffResolutionStrategy::FacilityCountModifier {
                owner_room, gate, ..
            }) = registry.get(buff_id)
            else {
                continue;
            };
            let mut pins = vec![(op.char_id.clone(), owner_room.clone())];
            match gate {
                FacilityGate::NamedCharInRoom { char_id, room } => {
                    if !owned.contains(char_id.as_str()) {
                        continue;
                    }
                    pins.push((char_id.clone(), room.clone()));
                }
                FacilityGate::None | FacilityGate::NoRobotsInOtherRooms => {}
            }
            bundles.push(EconomyPlan {
                overrides: Vec::new(),
                pins,
                globals: Vec::new(),
            });
        }
    }
    bundles
}

/// The co-feeder sets a shared pool offers the oracle: every non-empty
/// subset while there are at most three co-feeders, else the full set and
/// each singleton (bounded, and the two shapes that matter: everyone, or
/// one seat that pays for itself).
fn cofeeder_subsets(others: &[String]) -> Vec<Vec<String>> {
    const FULL_ENUMERATION_MAX: usize = 3;
    let mut subsets: Vec<Vec<String>> = Vec::new();
    if others.len() <= FULL_ENUMERATION_MAX {
        for mask in 1u32..(1u32 << others.len()) {
            subsets.push(
                others
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| mask & (1 << i) != 0)
                    .map(|(_, id)| id.clone())
                    .collect(),
            );
        }
    } else {
        subsets.push(others.to_vec());
        subsets.extend(others.iter().map(|id| vec![id.clone()]));
    }
    // Largest first: the full seating is the bundle the oracle should try
    // before its parts.
    subsets.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
    subsets
}

// ── Joint-seating bundles (stage 3b) ─────────────────────────────────────────
// Economies whose generators and consumers are DIFFERENT operators can't pass
// the self-or-pinned rule alone - committing them is a seat-economics judgment
// (three Control-Center seats for the Sui trio cost whatever globals those
// seats would otherwise carry). Rather than modeling displacement by hand, a
// bundle packages the pins and solved consumer overrides together, and the
// CALLER judges it by running the optimal search with and without the bundle
// and keeping whichever total yield wins. The optimizer is the oracle; the
// bundle only has to be priced honestly.

/// Candidate joint-seating bundles for the roster, ready for with/without
/// evaluation. Today: the Sui Control-Center economy (Chongyue's deployed-Sui
/// counter plus Dusk/Ling's conditional grants, powering Shu's factory skill
/// and the Worldly Plight -> Witchcraft Crystal chain), and the robot
/// power-plant economy (Alanna's Operation Platforms).
pub fn candidate_bundles(
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> Vec<EconomyPlan> {
    let mut bundles = shared_pool_bundles(profiles, building, building_data, registry);
    bundles.extend(facility_count_bundles(profiles, registry));

    // Robot displacement (Alanna's Operation Platforms): a consumer whose buff
    // scales with Robot-tagged operators seated in Power Plants. Pin the
    // roster's robots into the plants and price the consumer at its solved
    // payoff; the caller's oracle keeps whichever assignment is worth more -
    // the robots' feed, or the drone specialists those pins displace (their
    // recovery is priced in LMD by the yield model). Robots with the highest
    // own POWER value go first so the bundle surrenders as little recovery as
    // possible.
    let power_seats: usize = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "POWER")
        .map(|r| {
            super::util::max_stationed_at_level(building_data, "POWER", r.level).max(0) as usize
        })
        .sum();
    if power_seats > 0 {
        let mut robots: Vec<&OperatorBaseProfile> = profiles
            .iter()
            .filter(|op| op.match_tags.iter().any(|t| t == "robot"))
            .collect();
        if !robots.is_empty() {
            let facility_counts: HashMap<String, usize> = HashMap::new();
            robots.sort_by(|a, b| {
                let va = super::ledger::op_power_rank_value(
                    a,
                    building_data,
                    registry,
                    &facility_counts,
                );
                let vb = super::ledger::op_power_rank_value(
                    b,
                    building_data,
                    registry,
                    &facility_counts,
                );
                vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal)
            });
            let pinned: Vec<&OperatorBaseProfile> = robots.into_iter().take(power_seats).collect();
            for op in profiles {
                let overrides: Vec<(String, f64)> = op
                    .available_buffs
                    .iter()
                    .filter_map(|buff_id| match registry.get(buff_id) {
                        Some(BuffResolutionStrategy::PoolPointsScaling { resource, per, pct })
                            if resource == ROBOTS_IN_POWER && *per > 0.0 =>
                        {
                            let steps = (pinned.len() as f64 / per).floor();
                            (steps > 0.0).then(|| (buff_id.clone(), pct * steps))
                        }
                        _ => None,
                    })
                    .collect();
                if overrides.is_empty() {
                    continue;
                }
                bundles.push(EconomyPlan {
                    globals: Vec::new(),
                    overrides,
                    pins: pinned
                        .iter()
                        .map(|r| (r.char_id.clone(), "POWER".to_string()))
                        .collect(),
                });
            }
        }
    }

    // The Sui trio: every owned operator whose CONTROL buffs carry a
    // side-channel pool grant (the same regexes the live settlement uses).
    struct GrantCarrier {
        owner: String,
        /// The room type the grant buffs require their owner to occupy - the
        /// pin target (Control Center for the Sui skills, by their own text).
        pin_room: String,
        /// resource, steady-state-weighted amount; dorm-occupancy counters
        /// (Dolris' "Passion +1 per dorm Operator") are settled against the
        /// PROJECTED occupancy, the same figure the dorm economies plan with.
        flat: Vec<(String, f64)>,
        faction: Option<(String, String, f64, f64)>, // tag, resource, per, unit_cap
    }
    let projected_occupancy = projected_dorm_occupancy(profiles, building, building_data);
    let mut cc_gens: Vec<GrantCarrier> = Vec::new();
    for op in profiles {
        let mut carrier: Option<GrantCarrier> = None;
        for buff_id in &op.available_buffs {
            let Some(buff) = building_data.buffs.get(buff_id) else {
                continue;
            };
            let flat: Vec<(String, f64)> =
                text_grants(buff_id, buff, registry, projected_occupancy, None);
            let mut faction = None;
            if let Some(c) = RE_FACTION_GRANT.captures(&buff.description) {
                faction = Some((
                    c[1].to_string(),
                    c[2].to_string(),
                    c[3].parse().unwrap_or(0.0),
                    c[4].parse().unwrap_or(f64::INFINITY),
                ));
            }
            // The simple deployed-tag counter (Felvine per Soubo Adventurer)
            // rides the faction channel: same shape, uncapped.
            if faction.is_none()
                && let Some(c) = RE_TAG_GRANT.captures(&buff.description)
                && !strategy_generates(registry, buff_id, buff, &c[2])
            {
                faction = Some((
                    c[1].to_string(),
                    c[2].to_string(),
                    c[3].parse().unwrap_or(0.0),
                    f64::INFINITY,
                ));
            }
            if flat.is_empty() && faction.is_none() {
                continue;
            }
            let entry = carrier.get_or_insert_with(|| GrantCarrier {
                owner: op.char_id.clone(),
                pin_room: buff.room_type.clone(),
                flat: Vec::new(),
                faction: None,
            });
            entry.flat.extend(flat);
            if faction.is_some() {
                entry.faction = faction;
            }
        }
        if let Some(c) = carrier {
            cc_gens.push(c);
        }
    }
    if cc_gens.is_empty() {
        return bundles;
    }

    // One bundle per resource ECONOMY, not one mega-bundle: carriers granting
    // disjoint resources (the Sui, Mujica and Felvine economies) must compete
    // at the oracle separately - a joint bundle over-pins the Control Center
    // (more pins than seats) and auto-loses, starving every economy at once.
    // Groups merge when carriers share a resource; a consumer's own converter
    // only ever bridges resources its own group already grants.
    let mut groups: Vec<(HashSet<String>, Vec<usize>)> = Vec::new();
    for (i, g) in cc_gens.iter().enumerate() {
        let mut res: HashSet<String> = g.flat.iter().map(|(r, _)| r.clone()).collect();
        if let Some((_, r, _, _)) = &g.faction {
            res.insert(r.clone());
        }
        let (mut merged_res, mut merged_idx) = (res, vec![i]);
        groups.retain_mut(|(gres, gidx)| {
            if gres.is_disjoint(&merged_res) {
                true
            } else {
                merged_res.extend(gres.drain());
                merged_idx.append(gidx);
                false
            }
        });
        groups.push((merged_res, merged_idx));
    }

    for (group_resources, carrier_idx) in groups {
        let group: Vec<&GrantCarrier> = carrier_idx.iter().map(|&i| &cc_gens[i]).collect();
        // Settle the group's pools with its grant-carriers pinned into the CC.
        // The faction counter sees only the pinned members themselves (they sit
        // in the CC, a non-dormitory building) - conservative: any further
        // deployed kin the search seats is upside the bundle doesn't claim.
        let mut pinned: Vec<String> = group.iter().map(|g| g.owner.clone()).collect();
        let mut pin_seats: Vec<(String, String)> = group
            .iter()
            .map(|g| (g.owner.clone(), g.pin_room.clone()))
            .collect();
        // A PURE global consumer (Sakiko: no grants of her own, but her
        // factory global drinks the group's pool) needs a Control-Center seat
        // too - pin her with the carriers so her global may be credited.
        for op in profiles {
            if pinned.contains(&op.char_id) {
                continue;
            }
            let consumes_group = op.available_buffs.iter().any(|b| {
                matches!(
                    registry.get(b),
                    Some(BuffResolutionStrategy::GlobalPoolScaling { resource, .. })
                        if group_resources.contains(resource)
                )
            });
            if consumes_group {
                pinned.push(op.char_id.clone());
                pin_seats.push((op.char_id.clone(), "CONTROL".to_string()));
            }
        }
        let mut points: HashMap<String, f64> = HashMap::new();
        for g in &group {
            for (resource, amount) in &g.flat {
                *points.entry(resource.clone()).or_insert(0.0) += amount;
            }
            if let Some((tag, resource, per, unit_cap)) = &g.faction {
                #[allow(clippy::cast_precision_loss)]
                let kin = pinned
                    .iter()
                    .filter(|id| {
                        profiles
                            .iter()
                            .find(|p| &p.char_id == *id)
                            .is_some_and(|p| p.match_tags.iter().any(|t| t == tag))
                    })
                    .count() as f64;
                *points.entry(resource.clone()).or_insert(0.0) += per * kin.min(*unit_cap);
            }
        }

        // Price consumers and self-owned converter chains against the settled
        // pools, honoring the self-or-pinned rule (sources here are all pinned).
        let mut overrides: Vec<(String, f64)> = Vec::new();
        for op in profiles {
            // The operator's own converters extend the pools they can privately reach.
            let mut reach = points.clone();
            let mut own_converts: Vec<(String, String, f64)> = Vec::new();
            let mut own_consumers: Vec<(String, String, f64, f64)> = Vec::new();
            for buff_id in &op.available_buffs {
                let (Some(buff), Some(strategy)) =
                    (building_data.buffs.get(buff_id), registry.get(buff_id))
                else {
                    continue;
                };
                for clause in clauses_from_strategy(buff_id, buff, strategy) {
                    match &clause.kind {
                        ClauseKind::ResourceConvert(ResourceOp::Convert { from, to, ratio }) => {
                            own_converts.push((from.clone(), to.clone(), *ratio));
                        }
                        ClauseKind::ScalingPoolPoints { resource, step } => {
                            own_consumers.push((
                                buff_id.clone(),
                                resource.clone(),
                                *step,
                                clause.value,
                            ));
                        }
                        _ => {}
                    }
                }
            }
            for _ in 0..MAX_POOL_ROUNDS {
                let snapshot = reach.clone();
                let mut moved = 0.0f64;
                for (from, to, ratio) in &own_converts {
                    let available = snapshot.get(from).copied().unwrap_or(0.0);
                    if *ratio <= 0.0 {
                        continue;
                    }
                    let converted = (available / ratio).floor();
                    if converted > 0.0 {
                        *reach.entry(from.clone()).or_insert(0.0) -= converted * ratio;
                        *reach.entry(to.clone()).or_insert(0.0) += converted;
                        moved += converted;
                    }
                }
                if moved < POOL_EPS {
                    break;
                }
            }
            for (buff_id, resource, step, pct) in own_consumers {
                let p = reach.get(&resource).copied().unwrap_or(0.0);
                if p > 0.0 && step > 0.0 {
                    let solved = (p / step).floor() * pct;
                    if solved > 0.0 {
                        overrides.push((buff_id, solved));
                    }
                }
            }
        }

        // Pool-scaled Control-Center globals (Sakiko's trading global, the
        // Mortis factory global): credited only when the owner is one of this
        // group's pinned generators - they drink the pool they help fill, and
        // the pin guarantees the Control-Center seat the buff requires.
        let mut globals: Vec<(String, String, f64)> = Vec::new();
        for op in profiles.iter().filter(|p| pinned.contains(&p.char_id)) {
            for buff_id in &op.available_buffs {
                if let Some(BuffResolutionStrategy::GlobalPoolScaling {
                    target_room,
                    base_pct,
                    pct,
                    per,
                    resource,
                }) = registry.get(buff_id)
                    && *per > 0.0
                {
                    let p = points.get(resource).copied().unwrap_or(0.0);
                    let solved = base_pct + (p / per).floor() * pct;
                    if solved > 0.0 {
                        globals.push((buff_id.clone(), target_room.clone(), solved));
                    }
                }
            }
        }

        if !overrides.is_empty() || !globals.is_empty() {
            bundles.push(EconomyPlan {
                overrides,
                pins: pin_seats,
                globals,
            });
        }
    }
    bundles
}
