//! Rotation-sustainability simulator: steps the recommended 3-shift rotation
//! through a week of 12h blocks with GAME-TRUE morale rates and reports
//! whether any operator runs dry mid-shift. The static planner already scores
//! output; this pass VALIDATES the rhythm - a recommendation that quietly
//! drains its operators is flagged, not shipped as if it held up.
//!
//! Rates come from gamedata, not the planner's relative uptime calibration:
//! a full morale bar is 24 points (`MaxManpower` 8,640,000 ap = 360,000 ap per
//! point, drained over 24h baseline -> 1.0 point/hour), and a dormitory
//! recovers `DormData.Phases[level].ManpowerRecover / 100` points/hour
//! (1.6/hr at L1 -> 2.0/hr at L5). Furniture comfort (up to ~+0.35/hr) is NOT
//! modeled - it isn't synced - which errs conservative: a rotation that holds
//! up here also holds up in game.

use std::collections::HashMap;

use crate::core::gamedata::types::building::BuildingDataFile;

use super::buff_registry::{BuffResolutionStrategy, TargetedMoraleEffect};
use super::clause::{ClauseKind, Metric, clauses_from_strategy};
use super::shift_rotation::ShiftRotation;
use super::types::{OperatorBaseProfile, UserBuilding};

/// The game's full morale bar.
pub(crate) const MORALE_MAX: f64 = 24.0;
/// Baseline drain while working: one point per hour (8,640,000 ap over 24h).
const GAME_BASE_MORALE_DRAIN: f64 = 1.0;
/// Skills can slow drain but never fully stop it (matches the planner's floor).
const MIN_MORALE_DRAIN: f64 = 0.05;
/// One shift of the login rhythm.
const SHIFT_HOURS: f64 = 12.0;
/// A week of the rotation - enough cycles for slow leaks to surface.
pub const SIM_HORIZON_HOURS: f64 = 168.0;

/// Did the rotation survive the horizon?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Nobody ran dry: the rhythm is sustainable as recommended.
    HoldsUp,
    /// At least one operator's morale hit zero mid-shift.
    Depletes,
}

/// An operator whose morale reached zero while working.
#[derive(Debug, Clone)]
pub struct DepletedOperator {
    pub char_id: String,
    /// Hours into the simulation when the bar emptied.
    pub at_hours: f64,
    /// The room they were working when it happened.
    pub slot_id: String,
}

#[derive(Debug, Clone)]
pub struct SustainabilityReport {
    pub verdict: Verdict,
    pub horizon_hours: f64,
    /// First depletion per operator, ordered by when it happened.
    pub depleted: Vec<DepletedOperator>,
    /// Peak number of resting operators the dorms could NOT hold at once
    /// (they recover nothing that block). Zero for a healthy base.
    pub dorm_overflow: usize,
    /// Every simulated operator's morale over the horizon - the data behind a
    /// "morale over time" chart.
    pub timeline: Vec<OperatorMoraleTimeline>,
    /// Per-facility production totals and lost hours over the horizon.
    pub facilities: Vec<FacilityOutput>,
}

/// One production room's simulated totals over the horizon. Each 12h block
/// contributes `rate x mean crew alive-fraction`: an operator's buffs stop
/// the moment their bar empties, so the room's output is scaled by how much
/// of the block its crew actually had morale. `idle_hours` counts the lost
/// time - dark shifts (the cell rests unstaffed) plus the post-depletion
/// remainder of working blocks.
#[derive(Debug, Clone)]
pub struct FacilityOutput {
    pub slot_id: String,
    pub room_type: String,
    pub formula_type: Option<String>,
    /// Simulated totals over the horizon, in each room's own resource.
    pub lmd: f64,
    pub gold: f64,
    pub exp: f64,
    pub idle_hours: f64,
}

/// One operator's simulated morale, sampled at every 12h block boundary
/// (`samples[0]` is t=0 = a full bar; one more sample per block).
#[derive(Debug, Clone)]
pub struct OperatorMoraleTimeline {
    pub char_id: String,
    /// The room they work most often - their "home" for display. A permanent
    /// dorm resident's home is their dormitory; a sustained 24/7 operator's is
    /// the room the rotation pins them to.
    pub home_slot_id: String,
    pub samples: Vec<f64>,
}

/// Effective working drain at GAME rates, in morale points per hour: the
/// 1.0/hr baseline plus the operator's per-buff deltas.
pub fn game_morale_drain(op: &OperatorBaseProfile, morale_drains: &HashMap<String, f64>) -> f64 {
    let modifier: f64 = op
        .available_buffs
        .iter()
        .filter_map(|b| morale_drains.get(b))
        .sum();
    (GAME_BASE_MORALE_DRAIN + modifier).max(MIN_MORALE_DRAIN)
}

/// True when a full 24h block (two consecutive shifts) fits inside the
/// operator's morale bar: 24 points over 24 hours means drain must not exceed
/// the 1.0/hr baseline. This is BAR-feasibility only and deliberately
/// dorm-independent - a recovery shortfall is the simulator's report, not a
/// seating rule. The planner uses it to keep heavy-drainers out of 24h-block
/// seats (production teams, Squad 1) while leaving them eligible for the
/// single-shift Squad 2 positions they can genuinely work.
pub fn sustains_24h_block(op: &OperatorBaseProfile, morale_drains: &HashMap<String, f64>) -> bool {
    game_morale_drain(op, morale_drains) <= GAME_BASE_MORALE_DRAIN + 1e-9
}

/// Per-operator schedule across the 3-shift cycle: which shifts they work and
/// where. Derived from the rotation's recommended cells.
struct OpSchedule {
    /// `slot_id` worked per shift index, `None` = resting that shift.
    works: [Option<String>; 3],
    /// Game-true drain per working hour.
    drain: f64,
}

/// Simulate the rotation's login rhythm over [`SIM_HORIZON_HOURS`].
pub fn simulate_rotation(
    rotation: &ShiftRotation,
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    targeted: &HashMap<String, TargetedMoraleEffect>,
) -> SustainabilityReport {
    let profile_by_id: HashMap<&str, &OperatorBaseProfile> =
        profiles.iter().map(|p| (p.char_id.as_str(), p)).collect();

    // Targeted morale effects between CO-SEATED operators: the Ave Mujica
    // riders raise Sakiko's drain while their owners share her room, Mortis'
    // amnesty cancels her own rider, and Nian's faction version cancels every
    // co-seated Sui operator's self-drain riders. Deltas accrue per
    // (operator, shift); the owner's buff must belong to the room it fires in.
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
    // An operator's aura clauses of one metric, summed (their room-wide drain
    // aura / their dorm-recovery aura), derived from the same clause registry
    // the scorer reads.
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

    // Per (slot, shift): the summed room drain aura its members carry, and per
    // shift the best dorm-recovery aura among that shift's workers (the
    // non-stacking clause: only the strongest applies).
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

    // Targeted per-operator effects, resolved per (operator, shift) once the
    // room auras are known: the Ave Mujica riders and amnesties, Waaifu's
    // room-aura immunity, and Cement's formula-conditional drain. Every
    // effect's buff must belong to the room type it fires in.
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

    // Control-Center recovery auras, per shift: the `control_mp_cost` family
    // ("+0.05/hr to all Operators in the Control Center") offsets the CC's own
    // workers' drain, while the "other buildings" auras (Chongyue's) offset
    // every NON-CC worker instead. Only CONTROL-room buffs count - a dorm
    // recovery skill on a CC-seated operator does nothing outside its room.
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

    // Working drain per operator: game baseline plus their buffs' per-hour
    // deltas (the same per-buff numbers the planner's uptime model reads).
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

    // Dormitory RESIDENTS per (dorm slot, shift): the dorm-cell members who
    // never work a shift - permanent staff (aura holders, single-target
    // healers, a parked morale-swap manager). A dorm seat is rest, not work:
    // residents never drain, they hold seats and project their dorm skills
    // onto whoever rests beside them. Workers the rotation SHOWS resting in a
    // dorm cell are not residents - the simulator re-derives their rest from
    // the schedule and assigns them to dorms by live morale below.
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

    // Fiammetta-held 24/7 operators are morale-swapped every login; they never
    // drain and never occupy a dorm slot.
    for id in &rotation.sustained {
        schedules.remove(id);
    }

    // The base's dorms, best first - the neediest rester always gets the
    // highest-recovery dorm, exactly the assignment a player makes. Per (dorm,
    // shift): seats already held by residents, the strongest whole-dorm aura
    // among them, and the strongest single-target heal (both non-stacking
    // within their type, so the max IS the whole effect).
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
        .map(|id| (id.clone(), MORALE_MAX))
        .collect();
    let mut depleted: Vec<DepletedOperator> = Vec::new();
    let mut dorm_overflow = 0usize;
    // Morale sampled per operator at every block boundary (t=0 is full).
    let mut samples: HashMap<String, Vec<f64>> = schedules
        .keys()
        .map(|id| (id.clone(), vec![MORALE_MAX]))
        .collect();

    // Room levels, for converting a cell's efficiency into a resource rate.
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

        // How much of this block each worker had morale for (1.0 = the whole
        // block) - the fraction of the block their buffs were live.
        let mut alive_frac: HashMap<&str, f64> = HashMap::new();

        // Workers drain. Running dry STRICTLY inside a block is a depletion;
        // landing on exactly zero at the block boundary is the intended rhythm
        // (the bar empties right as the login swap rests the team).
        const EPS: f64 = 1e-9;
        for (id, sched) in &schedules {
            let Some(slot) = &sched.works[shift] else {
                continue;
            };
            // The room's drain aura (a teammate's "-0.1/hr to everyone here")
            // shifts this block's effective drain, and the Control Center's
            // recovery auras offset it - the CC's own aura for its workers,
            // the "other buildings" auras for everyone else. Floor applies.
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

        // Per-facility production this block: rate x mean crew alive-fraction.
        // A dark cell (the room rests unstaffed this shift) is fully idle; a
        // working crew that runs dry mid-block idles for the remainder.
        // (A rotation can carry fewer than 3 shifts in synthetic fixtures -
        // a missing shift simply contributes nothing.)
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
                // 24/7-sustained and resident operators aren't in `schedules`;
                // their bars are held full by definition.
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
            );
            let day_frac = SHIFT_HOURS / 24.0 * crew_frac;
            acc.lmd += y.lmd_per_day * day_frac;
            acc.gold += y.gold_per_day * day_frac;
            acc.exp += y.exp_per_day * day_frac;
            acc.idle_hours += SHIFT_HOURS * (1.0 - crew_frac);
        }

        // Resters recover lowest-morale first, filling the BEST dorm's free
        // seats before the next: each recovers at that dorm's own level rate,
        // plus its residents' whole-dorm aura, plus the working Control-Center
        // aura ("all Operators in Dormitories recover +0.05/hr"). The dorm's
        // single-target healer tops up its neediest rester. Anyone beyond the
        // last free seat recovers nothing that block.
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
                // The queue is needy-first, so the dorm's first intake is its
                // neediest occupant - the single-target heal lands there.
                let boost = if taken == 0 { single } else { 0.0 };
                let m = morale.get_mut(id.as_str()).expect("rester has morale");
                *m = (*m + (rate + boost) * SHIFT_HOURS).min(MORALE_MAX);
            }
        }
        unseated += queue.count();
        dorm_overflow = dorm_overflow.max(unseated);

        // Sample everyone at the block boundary.
        for (id, track) in &mut samples {
            track.push(morale.get(id.as_str()).copied().unwrap_or(MORALE_MAX));
        }
    }

    // The chart data: every scheduled operator's sampled bar, homed to the
    // slot they work most; permanent dorm residents and 24/7-sustained
    // operators ride along as flat full bars (residents never drain, the
    // manager's swap keeps a sustained operator topped up by definition).
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
