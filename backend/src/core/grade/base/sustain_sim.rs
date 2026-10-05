//! Rotation-sustainability check: steps the 3-shift rotation through a week of
//! 12h blocks at game-true morale rates and reports anyone running dry
//! mid-shift, so a plan that quietly drains its operators gets flagged.
//!
//! Rates come from gamedata, not the planner's uptime calibration: a full bar is
//! 24 points (`MaxManpower` 8,640,000 ap = 360,000 ap/point, 1.0 point/h
//! baseline), and a dorm recovers `DormData.Phases[level].ManpowerRecover / 100`
//! points/h (1.6 at L1 -> 2.0 at L5). Furniture comfort (up to ~+0.35/h) isn't
//! synced and isn't modeled, which errs conservative.

use std::collections::HashMap;

use crate::core::gamedata::types::building::BuildingDataFile;

use super::buff_registry::{BuffResolutionStrategy, TargetedMoraleEffect};
use super::clause::{ClauseKind, Metric, clauses_from_strategy};
use super::shift_rotation::ShiftRotation;
use super::types::{OperatorBaseProfile, UserBuilding};

pub(crate) const MORALE_MAX: f64 = 24.0;
/// Baseline drain while working: one point per hour (8,640,000 ap over 24h).
const GAME_BASE_MORALE_DRAIN: f64 = 1.0;
/// Skills slow drain but never stop it (the planner's floor).
const MIN_MORALE_DRAIN: f64 = 0.05;
const SHIFT_HOURS: f64 = 12.0;
/// A week: enough cycles for slow leaks to surface.
pub const SIM_HORIZON_HOURS: f64 = 168.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    HoldsUp,
    Depletes,
}

#[derive(Debug, Clone)]
pub struct DepletedOperator {
    pub char_id: String,
    pub at_hours: f64,
    pub slot_id: String,
}

#[derive(Debug, Clone)]
pub struct SustainabilityReport {
    pub verdict: Verdict,
    pub horizon_hours: f64,
    /// First depletion per operator, ordered by when it happened.
    pub depleted: Vec<DepletedOperator>,
    /// Peak resting operators the dorms could NOT seat at once (they recover nothing
    /// that block). Zero for a healthy base.
    pub dorm_overflow: usize,
    /// Every simulated operator's morale over the horizon, for the chart.
    pub timeline: Vec<OperatorMoraleTimeline>,
    pub facilities: Vec<FacilityOutput>,
}

/// One production room's totals over the horizon. Each 12h block contributes
/// `rate x mean crew alive-fraction`: buffs stop when a bar empties.
/// `idle_hours` = dark shifts (cell rests unstaffed) plus the post-depletion
/// remainder of working blocks.
#[derive(Debug, Clone)]
pub struct FacilityOutput {
    pub slot_id: String,
    pub room_type: String,
    pub formula_type: Option<String>,
    /// In each room's own resource.
    pub lmd: f64,
    pub gold: f64,
    pub exp: f64,
    pub idle_hours: f64,
}

/// Morale sampled at every 12h block boundary (`samples[0]` = t=0, full bar).
#[derive(Debug, Clone)]
pub struct OperatorMoraleTimeline {
    pub char_id: String,
    /// Room worked most, for display. A dorm resident's is their dorm; a 24/7
    /// operator's is the room the rotation pins them to.
    pub home_slot_id: String,
    pub samples: Vec<f64>,
}

/// Working drain at game rates (points/h): 1.0 baseline plus per-buff deltas.
pub fn game_morale_drain(op: &OperatorBaseProfile, morale_drains: &HashMap<String, f64>) -> f64 {
    let modifier: f64 = op
        .available_buffs
        .iter()
        .filter_map(|b| morale_drains.get(b))
        .sum();
    (GAME_BASE_MORALE_DRAIN + modifier).max(MIN_MORALE_DRAIN)
}

/// Whether a 24h block (two shifts) fits in the bar: drain <= the 1.0/h
/// baseline. Bar-feasibility only and dorm-independent on purpose; a recovery
/// shortfall is the simulator's report, not a seating rule. Keeps heavy drainers
/// out of 24h seats (production, Squad 1) but eligible for single-shift Squad 2.
pub fn sustains_24h_block(op: &OperatorBaseProfile, morale_drains: &HashMap<String, f64>) -> bool {
    game_morale_drain(op, morale_drains) <= GAME_BASE_MORALE_DRAIN + 1e-9
}

/// Which shifts an operator works and where, from the rotation's cells.
struct OpSchedule {
    /// `None` = resting that shift.
    works: [Option<String>; 3],
    drain: f64,
}

/// Morale as the game last wrote it, unprojected. Live views describe the synced
/// base, so pool counters read these bars: projecting a stale sync weeks forward
/// reports a state the game never showed (Dusk drained below 12 while her
/// counter read the full grant). Empty when the sync has no bars; callers fall
/// back to steady-state models.
pub fn synced_live_morale(building_json: &serde_json::Value) -> HashMap<String, f64> {
    super::types::live_morale_snapshot(building_json)
        .into_iter()
        .map(|(id, s)| (id, s.morale))
        .collect()
}

/// Project snapshots to `now_unix`: working rooms drain at the game rate, dorms
/// recover at their rate (level + ambience), unstationed is frozen (the game
/// only moves morale in rooms). Auras and single-target healers are ignored on
/// purpose: a now-cast, not the block sim, and conservative beats over-promising.
pub fn project_morale(
    snapshots: &HashMap<String, super::types::MoraleSnapshot>,
    now_unix: i64,
    building: &UserBuilding,
    profiles: &[OperatorBaseProfile],
    building_data: &BuildingDataFile,
    morale_drains: &HashMap<String, f64>,
) -> HashMap<String, f64> {
    let dorms: HashMap<String, f64> = super::dorms::dorm_list(building, building_data)
        .into_iter()
        .map(|d| (d.slot_id, d.recovery_per_hour))
        .collect();
    let working: HashMap<&str, &str> = building
        .rooms
        .iter()
        .filter(|r| r.room_type != "DORMITORY")
        .flat_map(|r| {
            r.current_operators
                .iter()
                .map(move |id| (id.as_str(), r.slot_id.as_str()))
        })
        .collect();
    let profile_of: HashMap<&str, &OperatorBaseProfile> =
        profiles.iter().map(|p| (p.char_id.as_str(), p)).collect();
    snapshots
        .iter()
        .map(|(id, snap)| {
            let hours = ((now_unix - snap.at_unix).max(0) as f64) / 3600.0;
            // Known dorm -> recover; known working room -> drain; anything else
            // (unstationed, unmodeled rooms like private rooms) -> frozen. An unknown seat
            // must not fabricate drain or recovery.
            let projected = if let Some(rate) = dorms.get(&snap.room_slot) {
                snap.morale + rate * hours
            } else if working.contains_key(id.as_str()) {
                let drain = profile_of
                    .get(id.as_str())
                    .map_or(MIN_MORALE_DRAIN, |p| game_morale_drain(p, morale_drains));
                snap.morale - drain * hours
            } else {
                snap.morale
            };
            (id.clone(), projected.clamp(0.0, MORALE_MAX))
        })
        .collect()
}

pub fn simulate_rotation(
    rotation: &ShiftRotation,
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    targeted: &HashMap<String, TargetedMoraleEffect>,
) -> SustainabilityReport {
    simulate_rotation_from(
        rotation,
        profiles,
        building,
        building_data,
        registry,
        morale_drains,
        targeted,
        None,
    )
}

/// [`simulate_rotation`] optionally seeded with REAL current bars. The
/// steady-state verdict starts full ("does the rhythm hold?"); the unrotated
/// "from now" sim seeds live bars ("what happens next?").
#[allow(clippy::too_many_arguments)]
pub fn simulate_rotation_from(
    rotation: &ShiftRotation,
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    targeted: &HashMap<String, TargetedMoraleEffect>,
    initial_morale: Option<&HashMap<String, f64>>,
) -> SustainabilityReport {
    let profile_by_id: HashMap<&str, &OperatorBaseProfile> =
        profiles.iter().map(|p| (p.char_id.as_str(), p)).collect();

    // Targeted effects between co-seated operators: Ave Mujica riders raise
    // Sakiko's drain while their owners share her room, Mortis' amnesty cancels her
    // own rider, Nian's faction version cancels co-seated Sui self-drain riders.
    // Per (operator, shift); the owner's buff must belong to the room it fires in.
    let own_drain_increase = |id: &str| -> f64 {
        profile_by_id.get(id).map_or(0.0, |p| {
            p.available_buffs
                .iter()
                .filter_map(|b| building_data.buffs.get(b))
                .map(|buff| {
                    super::buff_registry::parse_morale_drain_increase(&buff.description)
                        .unwrap_or(0.0)
                        + super::buff_registry::parse_morale_loss_increase(&buff.description)
                            .unwrap_or(0.0)
                })
                .sum()
        })
    };
    // Sum of an operator's aura clauses for one metric (room drain / dorm
    // recovery), from the scorer's clause registry.
    let aura_total = |id: &str, metric: &Metric| -> f64 {
        profile_by_id.get(id).map_or(0.0, |p| {
            p.available_buffs
                .iter()
                .filter_map(|b| {
                    let buff = building_data.buffs.get(b)?;
                    let strategy = registry.get(b)?;
                    Some(
                        clauses_from_strategy(b, buff, strategy)
                            .into_iter()
                            .filter(|c| &c.metric == metric)
                            .filter(|c| matches!(c.kind, ClauseKind::SelfValue))
                            .map(|c| c.value)
                            .sum::<f64>(),
                    )
                })
                .sum()
        })
    };

    // Per (slot, shift): summed room drain aura. Per shift: the best dorm-recovery
    // aura among workers (non-stacking, strongest only).
    let mut room_aura: HashMap<(String, usize), f64> = HashMap::new();
    let mut dorm_aura_by_shift = [0.0f64; 3];
    for (k, shift) in rotation.shifts.iter().enumerate().take(3) {
        for room in shift.rooms.iter().filter(|r| r.active) {
            let aura: f64 = room
                .recommended
                .iter()
                .map(|id| aura_total(id, &Metric::MoraleDrainAura))
                .sum();
            if aura != 0.0 {
                room_aura.insert((room.slot_id.clone(), k), aura);
            }
            for id in &room.recommended {
                let d = aura_total(id, &Metric::DormRecoveryAura);
                if d > dorm_aura_by_shift[k] {
                    dorm_aura_by_shift[k] = d;
                }
            }
        }
    }

    // Per (operator, shift), once room auras are known: Ave Mujica riders and
    // amnesties, Waaifu's room-aura immunity, Cement's formula-conditional drain.
    // Each buff must belong to the room type it fires in.
    let mut targeted_delta: HashMap<(String, usize), f64> = HashMap::new();
    for (k, shift) in rotation.shifts.iter().enumerate().take(3) {
        for room in shift.rooms.iter().filter(|r| r.active) {
            for owner_id in &room.recommended {
                let Some(p) = profile_by_id.get(owner_id.as_str()) else {
                    continue;
                };
                for b in &p.available_buffs {
                    let Some(eff) = targeted.get(b) else { continue };
                    let applies_here = building_data
                        .buffs
                        .get(b)
                        .is_some_and(|buff| buff.room_type == room.room_type);
                    if !applies_here {
                        continue;
                    }
                    match eff {
                        TargetedMoraleEffect::Rider {
                            target: Some(t),
                            delta,
                        } if room.recommended.contains(t) => {
                            *targeted_delta.entry((t.clone(), k)).or_insert(0.0) += delta;
                        }
                        TargetedMoraleEffect::NegatesOwnLoss { target: Some(t) }
                            if room.recommended.contains(t) =>
                        {
                            *targeted_delta.entry((t.clone(), k)).or_insert(0.0) -=
                                own_drain_increase(t);
                        }
                        TargetedMoraleEffect::NegatesFactionOwnLoss { faction } => {
                            for member in &room.recommended {
                                let is_kin = profile_by_id
                                    .get(member.as_str())
                                    .is_some_and(|m| m.faction_tags.iter().any(|t| t == faction));
                                if is_kin {
                                    *targeted_delta.entry((member.clone(), k)).or_insert(0.0) -=
                                        own_drain_increase(member);
                                }
                            }
                        }
                        TargetedMoraleEffect::SelfAuraImmunity => {
                            // Cancel the room aura for the owner alone.
                            let aura = room_aura
                                .get(&(room.slot_id.clone(), k))
                                .copied()
                                .unwrap_or(0.0);
                            if aura != 0.0 {
                                *targeted_delta.entry((owner_id.clone(), k)).or_insert(0.0) -= aura;
                            }
                        }
                        TargetedMoraleEffect::SelfFormulaDrain { targets, delta } => {
                            let produces_target = room
                                .formula_type
                                .as_deref()
                                .is_some_and(|f| targets.iter().any(|t| t == f));
                            if produces_target {
                                *targeted_delta.entry((owner_id.clone(), k)).or_insert(0.0) +=
                                    delta;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // CC recovery auras per shift: `control_mp_cost` ("+0.05/hr to all Operators in
    // the Control Center") offsets CC workers, "other buildings" auras (Chongyue's)
    // offset every non-CC worker. CONTROL-room buffs only; a dorm skill on a
    // CC-seated operator does nothing outside its room.
    let cc_recovery_total = |id: &str, base_wide: bool| -> f64 {
        profile_by_id.get(id).map_or(0.0, |p| {
            p.available_buffs
                .iter()
                .filter_map(|b| {
                    let buff = building_data.buffs.get(b)?;
                    (buff.room_type == "CONTROL").then_some(())?;
                    let strategy = registry.get(b)?;
                    Some(
                        clauses_from_strategy(b, buff, strategy)
                            .into_iter()
                            .filter(|c| c.metric == Metric::MoraleRecovery { base_wide })
                            .filter(|c| matches!(c.kind, ClauseKind::SelfValue))
                            .map(|c| c.value)
                            .sum::<f64>(),
                    )
                })
                .sum()
        })
    };
    let mut cc_room_recovery = [0.0f64; 3];
    let mut base_wide_recovery = [0.0f64; 3];
    let mut cc_slot: [Option<String>; 3] = [None, None, None];
    for (k, shift) in rotation.shifts.iter().enumerate().take(3) {
        if let Some(cc) = shift
            .rooms
            .iter()
            .find(|r| r.room_type == "CONTROL" && r.active)
        {
            cc_slot[k] = Some(cc.slot_id.clone());
            cc_room_recovery[k] = cc
                .recommended
                .iter()
                .map(|id| cc_recovery_total(id, false))
                .sum();
            base_wide_recovery[k] = cc
                .recommended
                .iter()
                .map(|id| cc_recovery_total(id, true))
                .sum();
        }
    }

    // Baseline plus per-buff deltas (the numbers the planner's uptime model reads).
    let op_drain = |id: &str| -> f64 {
        profile_by_id.get(id).map_or(GAME_BASE_MORALE_DRAIN, |p| {
            game_morale_drain(p, morale_drains)
        })
    };

    let mut schedules: HashMap<String, OpSchedule> = HashMap::new();
    let mut dorm_cells: HashMap<(String, usize), Vec<String>> = HashMap::new();
    for (k, shift) in rotation.shifts.iter().enumerate().take(3) {
        for room in shift.rooms.iter().filter(|r| r.active) {
            if room.room_type == "DORMITORY" {
                dorm_cells.insert((room.slot_id.clone(), k), room.recommended.clone());
                continue;
            }
            for id in &room.recommended {
                let entry = schedules.entry(id.clone()).or_insert_with(|| OpSchedule {
                    works: [None, None, None],
                    drain: op_drain(id),
                });
                entry.works[k] = Some(room.slot_id.clone());
            }
        }
    }

    // Dorm RESIDENTS per (dorm slot, shift): dorm-cell members who never work
    // (aura holders, single-target healers, a parked morale-swap manager). They
    // never drain; they hold seats and project dorm skills on whoever rests beside
    // them. Workers shown resting in a dorm cell are not residents: their rest is
    // re-derived from the schedule and seated by live morale below.
    let residents: HashMap<(String, usize), Vec<String>> = dorm_cells
        .into_iter()
        .map(|(key, ids)| {
            let pure: Vec<String> = ids
                .into_iter()
                .filter(|id| !schedules.contains_key(id))
                .collect();
            (key, pure)
        })
        .collect();

    // Fiammetta-held 24/7 operators are swapped every login: no drain, no dorm seat.
    for id in &rotation.sustained {
        schedules.remove(id);
    }
    // A parked token sits at zero by design: no drain, no running dry, no bed.
    for id in &rotation.parked {
        schedules.remove(id);
    }

    // Dorms best first: the neediest rester gets the highest-recovery dorm, as a
    // player would. Per (dorm, shift): resident-held seats, the strongest
    // whole-dorm aura and single-target heal among them (each non-stacking, so the
    // max is the whole effect).
    let dorm_list = super::dorms::dorm_list(building, building_data);
    let resident_aura = |slot: &str, k: usize, single: bool| -> f64 {
        residents.get(&(slot.to_string(), k)).map_or(0.0, |ids| {
            ids.iter()
                .filter_map(|id| profile_by_id.get(id.as_str()))
                .map(|p| {
                    if single {
                        super::dorms::dorm_single_value(p, registry, building_data)
                    } else {
                        super::dorms::dorm_aura_value(p, registry, building_data)
                    }
                })
                .fold(0.0, f64::max)
        })
    };

    let mut morale: HashMap<String, f64> = schedules
        .keys()
        .map(|id| {
            let start = initial_morale
                .and_then(|m| m.get(id))
                .copied()
                .unwrap_or(MORALE_MAX);
            (id.clone(), start)
        })
        .collect();
    let mut depleted: Vec<DepletedOperator> = Vec::new();
    let mut dorm_overflow = 0usize;
    // Sampled at every block boundary (t=0 is full).
    let mut samples: HashMap<String, Vec<f64>> = schedules
        .keys()
        .map(|id| {
            (
                id.clone(),
                vec![morale.get(id).copied().unwrap_or(MORALE_MAX)],
            )
        })
        .collect();

    let level_of: HashMap<&str, i32> = building
        .rooms
        .iter()
        .map(|r| (r.slot_id.as_str(), r.level))
        .collect();
    let mut facility_acc: HashMap<String, FacilityOutput> = HashMap::new();

    let blocks = (SIM_HORIZON_HOURS / SHIFT_HOURS) as usize;
    for block in 0..blocks {
        let shift = block % 3;
        let t0 = block as f64 * SHIFT_HOURS;

        // Fraction of this block each worker had morale (buffs live).
        let mut alive_frac: HashMap<&str, f64> = HashMap::new();

        // Running dry STRICTLY inside a block is a depletion; hitting exactly zero at
        // the boundary is the intended rhythm (empties as the login swap rests them).
        const EPS: f64 = 1e-9;
        for (id, sched) in &schedules {
            let Some(slot) = &sched.works[shift] else {
                continue;
            };
            // The room's drain aura (a teammate's "-0.1/hr to everyone here") shifts the
            // drain; CC recovery auras offset it (CC's own for its workers, "other
            // buildings" for the rest). Floor applies.
            let aura = room_aura
                .get(&(slot.clone(), shift))
                .copied()
                .unwrap_or(0.0);
            let recovery_aura = if cc_slot[shift].as_deref() == Some(slot.as_str()) {
                cc_room_recovery[shift]
            } else {
                base_wide_recovery[shift]
            };
            let pair_delta = targeted_delta
                .get(&(id.clone(), shift))
                .copied()
                .unwrap_or(0.0);
            let drain = (sched.drain + aura - recovery_aura + pair_delta).max(MIN_MORALE_DRAIN);
            let m = morale.get_mut(id).expect("scheduled op has morale");
            let before = *m;
            *m = (before - drain * SHIFT_HOURS).max(0.0);
            alive_frac.insert(id.as_str(), (before / (drain * SHIFT_HOURS)).min(1.0));
            if before < drain * SHIFT_HOURS - EPS && !depleted.iter().any(|d| &d.char_id == id) {
                depleted.push(DepletedOperator {
                    char_id: id.clone(),
                    at_hours: t0 + before / drain,
                    slot_id: slot.clone(),
                });
            }
        }

        // A dark cell (room rests unstaffed) is fully idle; a crew that runs dry
        // mid-block idles for the remainder. Synthetic fixtures may have fewer than 3
        // shifts; a missing shift contributes nothing.
        for room in rotation
            .shifts
            .get(shift)
            .map(|s| s.rooms.as_slice())
            .unwrap_or_default()
            .iter()
            .filter(|r| super::util::is_production_room(&r.room_type))
        {
            let acc = facility_acc
                .entry(room.slot_id.clone())
                .or_insert_with(|| FacilityOutput {
                    slot_id: room.slot_id.clone(),
                    room_type: room.room_type.clone(),
                    formula_type: room.formula_type.clone(),
                    lmd: 0.0,
                    gold: 0.0,
                    exp: 0.0,
                    idle_hours: 0.0,
                });
            if !room.active || room.recommended.is_empty() {
                acc.idle_hours += SHIFT_HOURS;
                continue;
            }
            #[allow(clippy::cast_precision_loss)]
            let crew_frac = room
                .recommended
                .iter()
                // 24/7 and resident operators aren't in `schedules`; their bars stay full.
                .map(|id| alive_frac.get(id.as_str()).copied().unwrap_or(1.0))
                .sum::<f64>()
                / room.recommended.len() as f64;
            let level = level_of.get(room.slot_id.as_str()).copied().unwrap_or(1);
            let y = super::yield_model::room_yield(
                &room.room_type,
                room.formula_type.as_deref(),
                level,
                room.efficiency.unwrap_or(0.0),
                0.0,
                room.recommended.len(),
                // Cells carry no order limit: the sustained sim prices the rate, the
                // assignment objective the buffer.
                None,
            );
            let day_frac = SHIFT_HOURS / 24.0 * crew_frac;
            acc.lmd += y.lmd_per_day * day_frac;
            acc.gold += y.gold_per_day * day_frac;
            acc.exp += y.exp_per_day * day_frac;
            acc.idle_hours += SHIFT_HOURS * (1.0 - crew_frac);
        }

        // Lowest morale rests first, filling the best dorm's free seats before the
        // next: dorm level rate + residents' whole-dorm aura + the working CC aura ("all
        // Operators in Dormitories recover +0.05/hr"). The single-target healer tops up
        // the neediest. Past the last free seat, no recovery that block.
        let mut resting: Vec<&String> = schedules
            .iter()
            .filter(|(id, s)| s.works[shift].is_none() && morale[id.as_str()] < MORALE_MAX)
            .map(|(id, _)| id)
            .collect();
        resting.sort_by(|a, b| {
            morale[a.as_str()]
                .partial_cmp(&morale[b.as_str()])
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.cmp(b))
        });
        let mut queue = resting.into_iter();
        let mut unseated = 0usize;
        for dorm in &dorm_list {
            let held = residents
                .get(&(dorm.slot_id.clone(), shift))
                .map_or(0, Vec::len);
            let free = dorm.capacity.saturating_sub(held);
            let rate = dorm.recovery_per_hour
                + resident_aura(&dorm.slot_id, shift, false)
                + dorm_aura_by_shift[shift];
            let single = resident_aura(&dorm.slot_id, shift, true);
            for taken in 0..free {
                let Some(id) = queue.next() else { break };
                // Queue is needy-first, so a dorm's first intake gets the single-target heal.
                let boost = if taken == 0 { single } else { 0.0 };
                let m = morale.get_mut(id.as_str()).expect("rester has morale");
                *m = (*m + (rate + boost) * SHIFT_HOURS).min(MORALE_MAX);
            }
        }
        unseated += queue.count();
        dorm_overflow = dorm_overflow.max(unseated);

        for (id, track) in &mut samples {
            track.push(morale.get(id.as_str()).copied().unwrap_or(MORALE_MAX));
        }
    }

    // Every scheduled operator's samples, homed to the slot they work most. Dorm
    // residents and 24/7 operators ride along as flat full bars.
    let flat = vec![MORALE_MAX; blocks + 1];
    let mut timeline: Vec<OperatorMoraleTimeline> = samples
        .into_iter()
        .map(|(char_id, track)| {
            let home = schedules
                .get(&char_id)
                .and_then(|s| {
                    let mut counts: HashMap<&String, usize> = HashMap::new();
                    for slot in s.works.iter().flatten() {
                        *counts.entry(slot).or_insert(0) += 1;
                    }
                    counts
                        .into_iter()
                        .max_by_key(|(slot, n)| (*n, std::cmp::Reverse(slot.as_str())))
                        .map(|(slot, _)| slot.clone())
                })
                .unwrap_or_default();
            OperatorMoraleTimeline {
                char_id,
                home_slot_id: home,
                samples: track,
            }
        })
        .collect();
    for ((slot, k), ids) in &residents {
        if *k != 0 {
            continue;
        }
        for id in ids {
            timeline.push(OperatorMoraleTimeline {
                char_id: id.clone(),
                home_slot_id: slot.clone(),
                samples: flat.clone(),
            });
        }
    }
    for id in &rotation.sustained {
        let home = rotation
            .shifts
            .first()
            .and_then(|s| {
                s.rooms
                    .iter()
                    .find(|r| r.recommended.iter().any(|o| o == id))
                    .map(|r| r.slot_id.clone())
            })
            .unwrap_or_default();
        timeline.push(OperatorMoraleTimeline {
            char_id: id.clone(),
            home_slot_id: home,
            samples: flat.clone(),
        });
    }

    depleted.sort_by(|a, b| {
        a.at_hours
            .partial_cmp(&b.at_hours)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut facilities: Vec<FacilityOutput> = facility_acc.into_values().collect();
    facilities.sort_by(|a, b| a.slot_id.cmp(&b.slot_id));
    SustainabilityReport {
        verdict: if depleted.is_empty() {
            Verdict::HoldsUp
        } else {
            Verdict::Depletes
        },
        horizon_hours: SIM_HORIZON_HOURS,
        depleted,
        dorm_overflow,
        timeline,
        facilities,
    }
}
