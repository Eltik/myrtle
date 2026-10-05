//! Assignment-scope pool settlement (stage 2a). With real seats (the live base),
//! seat-dependent generators settle and conversion chains relax to a fixed
//! point. Totals reach the room scorer as `POOL_<resource>` synthetics in
//! `facility_counts`. Search paths have no fixed assignment and price these
//! consumers at zero; see the seat-incentive planner below.

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

/// Pseudo-pool: Robot-tagged operators in Power Plants ("Operation Platform",
/// `$cc.tag.op`). Alanna's basis.
pub const ROBOTS_IN_POWER: &str = "tag_op_in_power";

/// Fraction of a block a "when own Morale is above/below N" condition holds. A
/// full bar drains linearly, so above N for (MAX-N)/MAX, below for N/MAX.
/// Documented model on the simulator's gamedata rates.
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
// Some CC skills (Dusk/Ling/Chongyue) carry a parsed strategy (a morale aura)
// AND a pool grant. The grant half is read from the description here, a
// side-channel like `morale_drains`.

/// Morale-conditional flat grants: "when self/own Morale is above/below N,
/// <Resource> +M" (both of Ling's branches match via `captures_iter`).
static RE_MORALE_COND_GRANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[Mm]orale is (above|below)\s*<@cc\.kw>([\d.]+)</>,\s*<\$cc\.([A-Za-z0-9_]+)>[^+]{0,40}?<@cc\.vup>\+([\d.]+)</>").unwrap()
});

/// UNCONDITIONAL "..., <Resource> +N". Over-captures alone, so every match must
/// pass [`flat_grant_unconditional`]. Audited 2026-08-13: the pair accepts exactly
/// the unconditional CC flats (Passion +20/+10/+10, Felvine +8) and rejects the
/// morale-conditional, per-operator, per-dorm-occupant and recruit-slot forms.
static RE_FLAT_GRANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<\$cc\.(bd_[A-Za-z0-9_]+)><@cc\.rem>[^<]+</></>\s*<@cc\.vup>\+([\d.]+)</>")
        .unwrap()
});

/// The side-channel captures this text, so an otherwise-Unresolved buff is
/// still priced (Dolris' "Idol's Aura", a dorm-occupancy Passion grant).
pub(crate) fn has_side_channel_grant(desc: &str) -> bool {
    super::buff_registry::RE_SLOT_GRANT.is_match(desc)
        || RE_OWN_LEVEL_GRANT.is_match(desc)
        || RE_MORALE_COND_GRANT.is_match(desc)
        || RE_FACTION_GRANT.is_match(desc)
        || RE_TAG_GRANT.is_match(desc)
        || RE_DORM_OCC_GRANT.is_match(desc)
        || RE_FLAT_GRANT
            .captures_iter(desc)
            .any(|c| flat_grant_unconditional(desc, c.get(0).map_or(0, |m| m.start())))
}

/// The parsed strategy already generates `resource`; re-reading the text would
/// double-count (the Sui generators parse whole).
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

/// No counting/conditional keyword in the window before `start`.
fn flat_grant_unconditional(desc: &str, start: usize) -> bool {
    // 160 covers the longest counter phrase ("for each <Sui> Operator assigned
    // to buildings other than Dormitories and Activity Rooms,"); true flats'
    // windows hold only the CC intro.
    let mut from = start.saturating_sub(160);
    while from > 0 && !desc.is_char_boundary(from) {
        from -= 1;
    }
    // Lowercase: Dusk's "when self morale is above 12, Perception Information
    // +10" is morale-conditional and must not count twice.
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

/// Per-own-room-LEVEL riders (Iris: "for every level of the current Dormitory,
/// 1 level of Dreamland"; Czerny: "each Dormitory level gives 1 Measure").
static RE_OWN_LEVEL_GRANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?:for every level of the current [A-Za-z ]+?,\s*<@cc\.vup>([\d.]+) levels?</>\s*<\$cc\.(bd_[A-Za-z0-9_]+)>|each [A-Za-z ]+? level gives <@cc\.vup>([\d.]+)</>\s*<\$cc\.(bd_[A-Za-z0-9_]+)>)",
    )
    .unwrap()
});

/// Extra recruit slots from the Office level (in-game check from the feedback
/// sheet: Whisperain's +10/slot gives 0 at HR1, 10 at HR2, 20 at HR3).
fn recruit_slots_of(building: &UserBuilding) -> f64 {
    building
        .rooms
        .iter()
        .filter(|r| r.room_type == "HIRE")
        .map(|r| f64::from((r.level - 1).max(0)))
        .fold(0.0, f64::max)
}

/// Deployment-independent grants in a buff's TEXT: unconditional flats (Dusk's
/// "Perception Information +10", never double-counting a parsed generator),
/// morale-conditional grants at steady-state weight, dorm-occupancy counters.
/// Faction/deployed-tag counters stay with callers. Shared by every path so they
/// all read the same points. With a REAL `current_morale` (live sync), a
/// conditional grant is all-or-nothing, as the game shows it.
fn text_grants(
    buff_id: &str,
    buff: &Buff,
    registry: &HashMap<String, BuffResolutionStrategy>,
    dorm_occupants: f64,
    current_morale: Option<f64>,
    own_room_level: f64,
    recruit_slots: f64,
) -> Vec<(String, f64)> {
    let mut grants: Vec<(String, f64)> = Vec::new();
    // Whisperain's Memory Fragments.
    for c in super::buff_registry::RE_SLOT_GRANT.captures_iter(&buff.description) {
        let per: f64 = c[2].parse().unwrap_or(0.0);
        grants.push((c[1].to_string(), per * recruit_slots));
    }
    for c in RE_OWN_LEVEL_GRANT.captures_iter(&buff.description) {
        let (per, resource) = match (c.get(1), c.get(2), c.get(3), c.get(4)) {
            (Some(p), Some(r), _, _) | (_, _, Some(p), Some(r)) => (p.as_str(), r.as_str()),
            _ => continue,
        };
        grants.push((
            resource.to_string(),
            per.parse::<f64>().unwrap_or(0.0) * own_room_level,
        ));
    }
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

/// "for each <tag.X> ... Operator, <Resource> +P" (Felvine; uncapped, unlike Sui).
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

/// MORALE-CONDITIONAL grant (Ling's "when own Morale is above/below N, <Resource>
/// +M"). Sustained only with a morale-swap manager, so pinning one should
/// reserve the manager too.
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

/// Settle assignment-fed pools against LIVE seats, then relax conversion chains.
/// `FunctionalLevels` generators are skipped: the ledger settles them room-locally
/// and this would double-count. `live_morale` is each bar as the game last wrote
/// it (may be empty): Dusk's "when self morale is above 12, Perception Information
/// +10" counts 10 while she IS above 12, nothing below.
pub(crate) fn settle_current_pools(
    building: &UserBuilding,
    operators: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    live_morale: &HashMap<String, f64>,
) -> HashMap<String, f64> {
    let by_id: HashMap<&str, &OperatorBaseProfile> =
        operators.iter().map(|o| (o.char_id.as_str(), o)).collect();

    // Dorm-count generators (Rosmontis, Mr. Nothing): synced seats only.
    #[allow(clippy::cast_precision_loss)]
    let dorm_occupants = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "DORMITORY")
        .map(|r| r.current_operators.len())
        .sum::<usize>() as f64;

    // Non-dorm seats, for faction counters and the robot pseudo-pool.
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

    let recruit_slots = recruit_slots_of(building);
    let mut points: HashMap<String, f64> = HashMap::new();
    let mut converts: Vec<(String, String, f64)> = Vec::new();

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
            // Depleted: holds the seat, generates nothing.
            if live_morale
                .get(id.as_str())
                .is_some_and(|m| *m < super::assignment::INERT_MORALE)
            {
                continue;
            }
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
                                #[allow(clippy::cast_precision_loss)]
                                PoolBasis::OwnRoomOccupants => {
                                    clause.value * room.current_operators.len() as f64
                                }
                                PoolBasis::DormOccupants => clause.value * dorm_occupants,
                                // Layout pools settle in the ledger.
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

            // Generator side-channel for THIS room's buffs.
            for buff_id in &op.available_buffs {
                let Some(buff) = building_data.buffs.get(buff_id) else {
                    continue;
                };
                if buff.room_type != room.room_type {
                    continue;
                }
                let current_morale = live_morale.get(id.as_str()).copied();
                for (resource, amount) in text_grants(
                    buff_id,
                    buff,
                    registry,
                    dorm_occupants,
                    current_morale,
                    f64::from(room.level.max(0)),
                    recruit_slots,
                ) {
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

    // Batched rounds from the round-start state, so clause order never matters.
    // `ratio` = from-points per to-point, floored. Converters COPY (base expert,
    // 2026-09-08: Jieyun's Witchcraft Crystals don't take Worldly Plight from Shu
    // or Mr. Nothing). Output is a LEVEL recomputed each round, not an increment,
    // so Dreamland -> Perception -> Chain of Thought settles in two rounds and
    // Rosmontis reads Iris' points (conversion-order report, 2026-09-13).
    let mut contrib: Vec<f64> = vec![0.0; converts.len()];
    for _ in 0..MAX_POOL_ROUNDS {
        let snapshot = points.clone();
        let mut moved = 0.0f64;
        for (ci, (from, to, ratio)) in converts.iter().enumerate() {
            if *ratio <= 0.0 {
                continue;
            }
            let available = snapshot.get(from).copied().unwrap_or(0.0);
            let level = (available / ratio).floor();
            let delta = level - contrib[ci];
            if delta.abs() > POOL_EPS {
                *points.entry(to.clone()).or_insert(0.0) += delta;
                contrib[ci] = level;
                moved += delta.abs();
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
// The search has no assignment, so consumers read zero there. This runs BEFORE
// the search and solves the roster's economies like the perception module:
// consumers get PoolPayoff overrides, seat-bound generators get pins.
//
// Honesty rule: credit a consumer ONLY when every generator and converter
// feeding it is the consumer themself or PINNED here. Value from a third operator
// the search may never seat is phantom; zero until joint-seating economics land.

#[derive(Debug, Default, PartialEq)]
pub struct EconomyPlan {
    /// `buff_id -> solved productivity %` - consumer buffs to override with
    /// [`BuffResolutionStrategy::PoolPayoff`].
    pub overrides: Vec<(String, f64)>,
    /// `(char_id, room_type)` generator seats the plan reserves (Senshi into
    /// the best dormitory).
    pub pins: Vec<(String, String)>,
    /// `(buff_id, target_room, solved %)` pool-scaled CC globals (Sakiko's
    /// trading global), folded as [`BuffResolutionStrategy::GlobalEffect`].
    pub globals: Vec<(String, String, f64)>,
    /// Who the seats are for (Pozëmka behind her Durins). The oracle skips the
    /// trial while she isn't seated: an unread count isn't worth a run.
    pub beneficiary: Option<String>,
}

/// Steady-state dorm occupancy: whoever isn't working, capped by slots.
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

/// One dorm-fed or room-level pool economy, settled against projected occupancy.
struct DormEconomy {
    gens: Vec<Gen>,
    /// (converter owner, from, to, ratio)
    converts: Vec<(String, String, String, f64)>,
    /// (owner, buff id, resource, step, pct)
    consumers: Vec<(String, String, String, f64, f64)>,
}

struct Gen {
    owner: String,
    owner_room: String,
    resource: String,
    points: f64,
    /// Own-room-level seats need a pin.
    pin: Option<String>,
    /// Parsed generator, not a side-channel grant. Only native origins anchor a
    /// shared-pool bundle; side-channel ones join as co-feeders.
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
    let recruit_slots = recruit_slots_of(building);
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
            // Side-channel grants (Dusk's "Perception Information +10") feed the
            // SAME pool: base expert 2026-09-08, Dusk, Iris, Czerny and Whisperain
            // all stack into Rosmontis' count. Shared-pool bundles may pin them;
            // the native plan never forces it.
            for (resource, points) in text_grants(
                buff_id,
                buff,
                registry,
                projected_occupancy,
                None,
                best_room_level(&buff.room_type),
                recruit_slots,
            ) {
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
                            // Full own room (a pinned Virtuosa fills her dorm).
                            PoolBasis::OwnRoomOccupants => {
                                let seats = building
                                    .rooms
                                    .iter()
                                    .filter(|r| r.room_type == clause.owner_room_type)
                                    .map(|r| {
                                        super::util::max_stationed_at_level(
                                            building_data,
                                            &r.room_type,
                                            r.level,
                                        )
                                        .max(0)
                                    })
                                    .max()
                                    .unwrap_or(0);
                                (
                                    clause.value * f64::from(seats),
                                    Some(clause.owner_room_type.clone()),
                                )
                            }
                            PoolBasis::DormOccupants => (clause.value * projected_occupancy, None),
                            // Layout pools settle in the scorer; others have
                            // no search story yet.
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

/// `resource -> origin -> points`. Converters COPY (live settlement: Rosmontis
/// and Ebenholz each read the full Perception Information); converted points
/// keep their origin, so consumers are priced on origins the plan vouches for.
fn settle_by_origin(econ: &DormEconomy) -> HashMap<String, HashMap<String, f64>> {
    let mut pools: HashMap<String, HashMap<String, f64>> = HashMap::new();
    for g in &econ.gens {
        *pools
            .entry(g.resource.clone())
            .or_default()
            .entry(g.owner.clone())
            .or_insert(0.0) += g.points;
    }
    // Levels, not increments, per (converter, origin): chains settle in
    // any order (see `settle_current_pools`).
    let mut contrib: HashMap<(usize, String), f64> = HashMap::new();
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
                let level = (available / ratio).floor();
                let key = (ci, origin.clone());
                let delta = level - contrib.get(&key).copied().unwrap_or(0.0);
                if delta.abs() > POOL_EPS {
                    *pools
                        .entry(to.clone())
                        .or_default()
                        .entry(origin.clone())
                        .or_insert(0.0) += delta;
                    contrib.insert(key, level);
                    moved += delta.abs();
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

    // Honesty rule per ORIGIN. A co-feeder the search might not seat (Ebenholz
    // beside Rosmontis) adds nothing; the shared-pool bundle offers that seat.
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

/// SHARED dorm-fed pools (Rosmontis' Chain of Thought draws on Ebenholz's
/// Musicianship): pin the co-feeders and price every consumer at the full total.
/// The oracle keeps it only if the seats pay.
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
        // Side-channel-only pools (the Sui CC economy) belong to the
        // grant-carrier bundles.
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
        // One bundle per co-feeder SUBSET: a rejected seat (Ebenholz's Trading
        // Post pin) must not sink one that pays (Dusk's CC seat).
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
            // Own origin plus the pinned ones.
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
                    beneficiary: None,
                });
            }
        }
    }
    bundles
}

/// CC operators gated on a production crew (Viviana: "all Knight Operators
/// assigned to Factories +7%"). The greedy CC selector rarely takes them: the
/// weight is a discounted guess and the crew only forms once she sits. When the
/// roster fields her faction twice with a target-room skill, pin her and let the
/// oracle judge (00980819 ran Fartooth/Ashlock/Wild Mane with Viviana; the plan
/// never assembled them, 2026-09-22).
fn cc_conditional_bundles(
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> Vec<EconomyPlan> {
    if !building.rooms.iter().any(|r| r.room_type == "CONTROL") {
        return Vec::new();
    }
    let mut bundles = Vec::new();
    // (operator, the operators their grant reaches) per qualifying seat.
    let mut seats: Vec<(String, HashSet<String>)> = Vec::new();
    for op in profiles {
        for bonus in super::assignment::cc_bonuses(op, registry, building_data) {
            let Some(cond) = bonus.conditional else {
                continue;
            };
            if !cond.per_operator
                || !building
                    .rooms
                    .iter()
                    .any(|r| r.room_type == cond.target_room)
            {
                continue;
            }
            let reached: HashSet<String> = profiles
                .iter()
                .filter(|p| p.char_id != op.char_id)
                .filter(|p| super::assignment::cc_token_matches(p, &cond.faction_token))
                .filter(|p| {
                    p.available_buffs.iter().any(|b| {
                        building_data
                            .buffs
                            .get(b)
                            .is_some_and(|buff| buff.room_type == cond.target_room)
                    })
                })
                .map(|p| p.char_id.clone())
                .collect();
            if reached.len() < 2 {
                continue;
            }
            bundles.push(EconomyPlan {
                overrides: Vec::new(),
                pins: vec![(op.char_id.clone(), "CONTROL".to_string())],
                globals: Vec::new(),
                beneficiary: None,
            });
            seats.push((op.char_id.clone(), reached));
            break;
        }
    }
    // Overlapping grants pay TOGETHER (Viviana's +7 to Knights and Flametail's
    // +10 Battle Records to Kazimierz hit the same trio: 128 with both, 96 with
    // one), so overlapping pairs are tried as one bundle too.
    for i in 0..seats.len() {
        for j in (i + 1)..seats.len() {
            if seats[i].1.intersection(&seats[j].1).count() >= 2 {
                bundles.push(EconomyPlan {
                    overrides: Vec::new(),
                    pins: vec![
                        (seats[i].0.clone(), "CONTROL".to_string()),
                        (seats[j].0.clone(), "CONTROL".to_string()),
                    ],
                    globals: Vec::new(),
                    beneficiary: None,
                });
            }
        }
    }
    bundles
}

/// Facility-count modifiers as seat bundles: Eunectes' "+2 Power Plants" needs
/// her in the CC and Lancet-2 in a Power Plant; Greyy's "+1" needs her plant
/// seat. Offered only when an automation scaler reads the count (Weedy,
/// Eunectes, Pudding).
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
                    // A robot token can be PARKED at zero morale so a
                    // robot-exclusion count fires beside it
                    // (`assignment::parked_tokens`): seat those holders too.
                    let token_is_robot = profiles
                        .iter()
                        .find(|p| &p.char_id == char_id)
                        .is_some_and(|p| p.match_tags.iter().any(|t| t == "robot"));
                    if token_is_robot {
                        for holder in profiles {
                            let excludes_here = holder.available_buffs.iter().any(|b| {
                                matches!(
                                    registry.get(b),
                                    Some(BuffResolutionStrategy::FacilityCountModifier {
                                        owner_room: o,
                                        gate: FacilityGate::NoRobotsInOtherRooms,
                                        ..
                                    }) if o == room
                                )
                            });
                            if excludes_here && !pins.iter().any(|(id, _)| id == &holder.char_id) {
                                pins.push((holder.char_id.clone(), room.clone()));
                            }
                        }
                    }
                }
                FacilityGate::None | FacilityGate::NoRobotsInOtherRooms => {}
            }
            bundles.push(EconomyPlan {
                overrides: Vec::new(),
                pins,
                globals: Vec::new(),
                beneficiary: None,
            });
        }
    }
    bundles
}

/// Base-wide counts (Pozëmka's Durins, Nasti's Rhine Lab): operators parked in
/// dorms just to be counted. Pin spare matches (fewest other-room skills first,
/// up to the cap); the oracle keeps them if the count pays.
fn base_count_bundles(
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> Vec<EconomyPlan> {
    use super::assignment::other_room_skill_count;
    if !building.rooms.iter().any(|r| r.room_type == "DORMITORY") {
        return Vec::new();
    }
    let mut bundles = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for op in profiles {
        for buff_id in &op.available_buffs {
            let Some(BuffResolutionStrategy::BaseWideMatchCountScaling {
                token, cap_count, ..
            }) = registry.get(buff_id)
            else {
                continue;
            };
            if !seen.insert(token.clone()) {
                continue;
            }
            let holder_counts = op.match_tags.iter().any(|t| t == token);
            let budget = cap_count
                .unwrap_or(usize::MAX)
                .saturating_sub(usize::from(holder_counts));
            let mut kin: Vec<&OperatorBaseProfile> = profiles
                .iter()
                .filter(|p| p.char_id != op.char_id && p.match_tags.iter().any(|t| t == token))
                .collect();
            kin.sort_by_key(|p| {
                (
                    other_room_skill_count(p, "DORMITORY", building_data),
                    p.char_id.clone(),
                )
            });
            let pins: Vec<(String, String)> = kin
                .into_iter()
                .take(budget)
                .map(|p| (p.char_id.clone(), "DORMITORY".to_string()))
                .collect();
            if !pins.is_empty() {
                bundles.push(EconomyPlan {
                    overrides: Vec::new(),
                    pins,
                    globals: Vec::new(),
                    beneficiary: Some(op.char_id.clone()),
                });
            }
        }
    }
    bundles
}

/// Every non-empty subset up to three co-feeders, else the full set plus each
/// singleton (bounded; the shapes that matter are everyone or one paying seat).
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
    // Largest first: try the full seating before its parts.
    subsets.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
    subsets
}

// ── Joint-seating bundles (stage 3b) ─────────────────────────────────────────
// When generators and consumers are DIFFERENT operators, seating them is a
// seat-economics call (three CC seats for the Sui trio cost the globals those
// seats would carry). No hand-modeled displacement: a bundle packs pins and
// overrides, and the CALLER runs the search with and without it. The optimizer
// is the oracle; the bundle just has to be priced honestly.

/// Bundles for with/without trials. Today: the Sui CC economy (Chongyue's
/// deployed-Sui counter + Dusk/Ling's conditional grants, feeding Shu's factory
/// skill and Worldly Plight -> Witchcraft Crystal) and the robot power-plant
/// economy (Alanna's Operation Platforms).
pub fn candidate_bundles(
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> Vec<EconomyPlan> {
    let mut bundles = shared_pool_bundles(profiles, building, building_data, registry);
    bundles.extend(facility_count_bundles(profiles, registry));
    bundles.extend(cc_conditional_bundles(
        profiles,
        building,
        building_data,
        registry,
    ));
    bundles.extend(base_count_bundles(
        profiles,
        building,
        registry,
        building_data,
    ));

    // Alanna: pin robots into the plants and price her payoff; the oracle
    // weighs that against the drone specialists displaced (priced in LMD).
    // Highest own POWER value first, to give up the least recovery.
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
                    beneficiary: None,
                });
            }
        }
    }

    // Owned operators whose CONTROL buffs carry a side-channel grant (the Sui
    // trio and others).
    struct GrantCarrier {
        owner: String,
        /// Pin target, from the buff's own room (CC for the Sui skills).
        pin_room: String,
        /// (resource, steady-state amount). Dorm counters (Dolris' "Passion +1 per
        /// dorm Operator") use PROJECTED occupancy, as the dorm economies do.
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
            let flat: Vec<(String, f64)> = text_grants(
                buff_id,
                buff,
                registry,
                projected_occupancy,
                None,
                0.0,
                recruit_slots_of(building),
            );
            let mut faction = None;
            if let Some(c) = RE_FACTION_GRANT.captures(&buff.description) {
                faction = Some((
                    c[1].to_string(),
                    c[2].to_string(),
                    c[3].parse().unwrap_or(0.0),
                    c[4].parse().unwrap_or(f64::INFINITY),
                ));
            }
            // Felvine per Soubo Adventurer: faction shape, uncapped.
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

    // One bundle per resource ECONOMY (Sui, Mujica, Felvine): a mega-bundle
    // over-pins the CC and auto-loses, starving all of them. Groups merge on a
    // shared resource; a consumer's converter only bridges its own group's.
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
        // The faction counter sees only the pinned carriers; more deployed kin is
        // upside the bundle doesn't claim.
        let mut pinned: Vec<String> = group.iter().map(|g| g.owner.clone()).collect();
        let mut pin_seats: Vec<(String, String)> = group
            .iter()
            .map(|g| (g.owner.clone(), g.pin_room.clone()))
            .collect();
        // A PURE global consumer (Sakiko: no grants, but her factory global
        // drinks the pool) needs a CC pin too.
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

        // Sources here are all pinned, so the honesty rule holds.
        let mut overrides: Vec<(String, f64)> = Vec::new();
        for op in profiles {
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

        // Pool-scaled CC globals (Sakiko's trading, the Mortis factory one):
        // only when the owner is a pinned generator, which guarantees the seat.
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
                beneficiary: None,
            });
        }
    }
    // Dedupe: identical plans (an office pin from two counters, Ling/Dusk alone
    // and inside a wider bundle) cost a full optimizer run each.
    let mut unique: Vec<EconomyPlan> = Vec::with_capacity(bundles.len());
    for bundle in bundles {
        if !unique.contains(&bundle) {
            unique.push(bundle);
        }
    }
    unique
}

/// Registry and pins for an OPTIMAL search: native economies solved
/// (`plan_optimal_economies`), generator seats pinned, Fiammetta reserved for a
/// morale-conditional generator (Ling). Shared by the improvements pipeline and
/// the planner; the planner once ran the bare registry and never seated
/// Rosmontis' feeders.
pub struct SearchEconomy {
    pub registry: HashMap<String, BuffResolutionStrategy>,
    pub pins: Vec<(String, String)>,
    pub manager: Option<String>,
}

pub fn search_economy(
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> SearchEconomy {
    let mut out = registry.clone();
    let mut pins: Vec<(String, String)> = Vec::new();
    let manager_pin = super::dorms::morale_manager_pin(profiles, building, building_data);
    let manager = manager_pin.as_ref().map(|(id, _)| id.clone());
    if let Some(pin) = manager_pin {
        pins.push(pin);
    }
    let native = plan_optimal_economies(profiles, building, building_data, registry);
    for (buff_id, pct) in &native.overrides {
        // Never downgrade a consumer another plan already priced higher.
        if out.get(buff_id).is_none_or(|s| {
            !matches!(
                s,
                BuffResolutionStrategy::PoolPayoff { .. }
                    | BuffResolutionStrategy::GlobalEffect { .. }
            )
        }) {
            out.insert(
                buff_id.clone(),
                BuffResolutionStrategy::PoolPayoff { pct: *pct },
            );
        }
    }
    pins.extend(native.pins.iter().cloned());
    SearchEconomy {
        registry: out,
        pins,
        manager,
    }
}

/// Registry, pins and plan after every bundle that improved realized yield.
pub struct AcceptedEconomy {
    pub registry: HashMap<String, BuffResolutionStrategy>,
    pub pins: Vec<(String, String)>,
    pub optimal: super::types::BaseAssignment,
}

/// Optimal search with bundle trials (Sui CC economy, facility and base counts,
/// resting Durins): keep a bundle only if realized yield improves, so
/// displacement costs show up in the yield. Rosmontis' feeders (Dusk, Ling)
/// reach the CC only through these trials.
///
/// Bundles come from `base_registry` (so an already-solved consumer still gets
/// one); every trial layers on `search_registry`.
pub fn optimal_with_bundles(
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    base_registry: &HashMap<String, BuffResolutionStrategy>,
    search_registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    pins: &[(String, String)],
) -> AcceptedEconomy {
    use super::assignment::{assignment_value, compute_optimal_assignment_with_pins};
    let mut optimal_registry = search_registry.clone();
    let mut optimal_pins: Vec<(String, String)> = pins.to_vec();
    // CC pins from PLAIN (seats-only) bundles; a later plain bundle may
    // displace them.
    let mut plain_cc: HashSet<String> = HashSet::new();
    let mut optimal = compute_optimal_assignment_with_pins(
        profiles,
        building,
        building_data,
        &optimal_registry,
        morale_drains,
        &optimal_pins,
    );
    for bundle in candidate_bundles(profiles, building, building_data, base_registry) {
        if let Some(who) = &bundle.beneficiary
            && !optimal
                .rooms
                .iter()
                .any(|r| r.operators.iter().any(|o| o == who))
        {
            continue;
        }
        let mut trial_registry = optimal_registry.clone();
        for (buff_id, pct) in &bundle.overrides {
            // Never downgrade a consumer another plan priced higher.
            let existing = match trial_registry.get(buff_id) {
                Some(BuffResolutionStrategy::PoolPayoff { pct: p }) => *p,
                _ => f64::NEG_INFINITY,
            };
            if *pct > existing {
                trial_registry.insert(
                    buff_id.clone(),
                    BuffResolutionStrategy::PoolPayoff { pct: *pct },
                );
            }
        }
        // Same never-downgrade rule for pool-scaled CC globals.
        for (buff_id, target_room, pct) in &bundle.globals {
            let existing = match trial_registry.get(buff_id) {
                Some(BuffResolutionStrategy::GlobalEffect { bonus_pct, .. }) => *bonus_pct,
                _ => f64::NEG_INFINITY,
            };
            if *pct > existing {
                trial_registry.insert(
                    buff_id.clone(),
                    BuffResolutionStrategy::GlobalEffect {
                        target_room: target_room.clone(),
                        bonus_pct: *pct,
                    },
                );
            }
        }
        let mut trial_pins = optimal_pins.clone();
        trial_pins.extend(bundle.pins.iter().cloned());
        let plain_bundle = bundle.overrides.is_empty() && bundle.globals.is_empty();
        // Pins are seats: a bundle needing more than the base has can't run
        // (the Mujica five + Dusk + Ling made a seven-seat CC). A PLAIN bundle
        // may displace earlier plain CC pins and keep the plan if the base gains.
        // Before this the first five CC pins were final, and Viviana + Flametail
        // (128 on the Knight trio) was never tried (00980819).
        let mut displaced: Vec<String> = Vec::new();
        if !pins_fit(building, building_data, &trial_pins) {
            if !plain_bundle {
                continue;
            }
            // Newest plain seats go first: an early seat (Eunectes' plant count)
            // beats a late filler (a displaced robot).
            let candidates: Vec<String> = optimal_pins
                .iter()
                .filter(|(id, rt)| {
                    rt == "CONTROL"
                        && plain_cc.contains(id)
                        && !bundle.pins.iter().any(|(b, _)| b == id)
                })
                .map(|(id, _)| id.clone())
                .collect();
            let mut fits = false;
            for id in candidates.iter().rev() {
                displaced.push(id.clone());
                trial_pins = optimal_pins
                    .iter()
                    .filter(|(pid, _)| !displaced.contains(pid))
                    .cloned()
                    .collect();
                trial_pins.extend(bundle.pins.iter().cloned());
                if pins_fit(building, building_data, &trial_pins) {
                    fits = true;
                    break;
                }
            }
            if !fits {
                continue;
            }
        }
        let trial = compute_optimal_assignment_with_pins(
            profiles,
            building,
            building_data,
            &trial_registry,
            morale_drains,
            &trial_pins,
        );
        if assignment_value(&trial.rooms) > assignment_value(&optimal.rooms) + 1e-9 {
            optimal = trial;
            optimal_registry = trial_registry;
            optimal_pins = trial_pins;
            for id in &displaced {
                plain_cc.remove(id);
            }
            if plain_bundle {
                plain_cc.extend(
                    bundle
                        .pins
                        .iter()
                        .filter(|(_, rt)| rt == "CONTROL")
                        .map(|(id, _)| id.clone()),
                );
            }
        }
    }
    AcceptedEconomy {
        registry: optimal_registry,
        pins: optimal_pins,
        optimal,
    }
}

/// Per room type, no more pins than total seats.
fn pins_fit(
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    pins: &[(String, String)],
) -> bool {
    let mut wanted: HashMap<&str, usize> = HashMap::new();
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for (id, room_type) in pins {
        if seen.insert(id.as_str()) {
            *wanted.entry(room_type.as_str()).or_insert(0) += 1;
        }
    }
    wanted.iter().all(|(room_type, &n)| {
        let seats: usize = building
            .rooms
            .iter()
            .filter(|r| r.room_type == *room_type)
            .map(|r| {
                super::util::max_stationed_at_level(building_data, room_type, r.level).max(0)
                    as usize
            })
            .sum();
        n <= seats
    })
}
