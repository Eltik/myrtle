//! Dorms modeled per room, not as one averaged recovery pool. The pool erased the
//! two decisions the game rewards: neediest resters into the HIGHEST-level dorm (a
//! 2/5/2 base under-levels dorms to afford the fifth factory, so its one good dorm
//! matters), and dorm-skill holders as permanent staff: whole-dorm auras ("+0.15/hr
//! to all Operators in that Dormitory") and single-target healers ("+0.55/hr to
//! another Operator whose Morale is not full").

use std::collections::{HashMap, HashSet};

use crate::core::gamedata::types::building::BuildingDataFile;

use super::buff_registry::BuffResolutionStrategy;
use super::types::{OperatorBaseProfile, UserBuilding};
use super::util::max_stationed_at_level;

#[derive(Debug, Clone)]
pub struct Dorm {
    pub slot_id: String,
    pub level: i32,
    /// Seats (`MaxStationedNum` at this level).
    pub capacity: usize,
    /// Morale/hour a rester recovers here (`DormData.Phases[level].ManpowerRecover / 100`),
    /// before dorm-skill auras.
    pub recovery_per_hour: f64,
}

/// Dorms BEST FIRST (recovery rate, then level, then slot id). The neediest
/// rester gets the best dorm.
pub fn dorm_list(building: &UserBuilding, building_data: &BuildingDataFile) -> Vec<Dorm> {
    let phases = &building_data.dorm_data.phases;
    let mut dorms: Vec<Dorm> = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "DORMITORY")
        .map(|r| {
            let idx = (r.level.max(1) as usize - 1).min(phases.len().saturating_sub(1));
            let mut rate = phases
                .get(idx)
                .map_or(0.0, |p| f64::from(p.manpower_recover) / 100.0);
            // Ambience: `comfort / ComfortManpowerRecoverFactor` manpower/sec
            // (comfort/2500 morale/hr at factor 25). Drafted rooms have no comfort.
            let factor = building_data.comfort_manpower_recover_factor;
            if factor > 0.0 && r.comfort > 0 {
                rate += f64::from(r.comfort) / (factor * 100.0);
            }
            #[allow(clippy::cast_sign_loss)]
            let capacity =
                max_stationed_at_level(building_data, "DORMITORY", r.level).max(0) as usize;
            Dorm {
                slot_id: r.slot_id.clone(),
                level: r.level,
                capacity,
                recovery_per_hour: rate,
            }
        })
        .collect();
    dorms.sort_by(|a, b| {
        b.recovery_per_hour
            .partial_cmp(&a.recovery_per_hour)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.level.cmp(&a.level))
            .then(a.slot_id.cmp(&b.slot_id))
    });
    dorms
}

/// WHOLE-DORM aura ("+X/hr to all Operators in that Dormitory"), 0 if none.
/// Non-stacking ("only the strongest effect of this type"): one holder per dorm.
pub fn dorm_aura_value(
    op: &OperatorBaseProfile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> f64 {
    dorm_skill_value(op, registry, building_data, false)
}

/// SINGLE-TARGET dorm heal ("+X/hr to another Operator in that Dormitory whose
/// Morale is not full"), 0 if none. Also non-stacking within its type.
pub fn dorm_single_value(
    op: &OperatorBaseProfile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> f64 {
    dorm_skill_value(op, registry, building_data, true)
}

fn dorm_skill_value(
    op: &OperatorBaseProfile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    want_single: bool,
) -> f64 {
    op.available_buffs
        .iter()
        .filter_map(|b| {
            let buff = building_data.buffs.get(b)?;
            (buff.room_type == "DORMITORY").then_some(())?;
            match registry.get(b) {
                Some(BuffResolutionStrategy::MoraleModifier {
                    recovery_per_hour,
                    is_self_only: false,
                    single_target,
                    ..
                }) if *single_target == want_single && *recovery_per_hour > 0.0 => {
                    Some(*recovery_per_hour)
                }
                _ => None,
            }
        })
        .fold(0.0, f64::max)
}

/// A morale-swap manager's own recovery rate.
///
/// Fiammetta's "Self-Discipline": "self Morale recovered +2 per hour, and cannot
/// gain Morale recovery from any other source", so this is her WHOLE rate whatever
/// the dorm level, auras or ambience (the worst dorm costs her nothing). "Communal
/// Suffering" swaps only at FULL morale: `MORALE_MAX / rate` hours to recharge,
/// 12h at +2/hr, one login.
pub fn manager_swap_rate(
    manager: &OperatorBaseProfile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> f64 {
    manager
        .available_buffs
        .iter()
        .filter_map(|b| {
            let buff = building_data.buffs.get(b)?;
            (buff.room_type == "DORMITORY").then_some(())?;
            match registry.get(b) {
                Some(BuffResolutionStrategy::MoraleModifier {
                    recovery_per_hour,
                    is_self_only: true,
                    ..
                }) if *recovery_per_hour > 0.0 => Some(*recovery_per_hour),
                _ => None,
            }
        })
        .fold(0.0, f64::max)
}

/// Can the manager hold `drain` at full morale 24/7? She hands over a full
/// 24-point bar per `MORALE_MAX / swap_rate` hours, so `drain <= swap_rate`.
/// Fiammetta (+2/hr) sustains up to 2.0/hr; a 3.0/hr Enforcer-class drainer outruns her.
pub fn manager_can_sustain(swap_rate: f64, drain_per_hour: f64) -> bool {
    swap_rate > 0.0 && drain_per_hour <= swap_rate + 1e-9
}

/// Can the building HOST a morale-swap manager? She must STAY in a dorm ("When
/// this Operator is assigned to a Dormitory, ... swaps Morale with the previous
/// Operator assigned to that Dormitory") with a free seat beside her for the
/// drained operator. No two-seat dorm, no 24/7 sustain.
pub fn building_hosts_manager(building: &UserBuilding, building_data: &BuildingDataFile) -> bool {
    dorm_list(building, building_data)
        .iter()
        .any(|d| d.capacity >= 2)
}

/// `(manager, "DORMITORY")` pin when the roster owns a morale-swap manager
/// (Fiammetta) and a dorm can host her. She works from a dorm seat, either holding
/// a morale-conditional generator (Ling) on the right side of its bar or
/// sustaining a producer 24/7; her kit has no other use, so the pin is free.
/// The "you'd want one but don't own one" flag is the caller's job.
pub fn morale_manager_pin(
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    registry_building_data: &BuildingDataFile,
) -> Option<(String, String)> {
    if !building_hosts_manager(building, registry_building_data) {
        return None;
    }
    super::assignment::morale_swap_enabler(profiles, registry_building_data)
        .map(|id| (id, "DORMITORY".to_string()))
}

/// Permanent 24/7 dorm staff, picked from operators the plan left unseated.
///
/// Same policy players run:
/// - one AURA holder per dorm, strongest into the best dorm (a second aura in
///   the same dorm is worthless, non-stacking);
/// - then single-target healers, strongest into the best dorm with room;
/// - at most `headroom` staff (capacity minus peak resting demand): each staff
///   seat evicts a rester, and faster recovery is worthless if it does that.
pub fn plan_dorm_staffing(
    dorms: &[Dorm],
    leftovers: &[&OperatorBaseProfile],
    headroom: usize,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> Vec<(String, Vec<String>)> {
    let mut staffed: Vec<(String, Vec<String>)> = dorms
        .iter()
        .map(|d| (d.slot_id.clone(), Vec::new()))
        .collect();
    if dorms.is_empty() || headroom == 0 {
        return staffed;
    }
    let mut budget = headroom;
    let mut used: HashSet<&str> = HashSet::new();

    let mut aura_ranked: Vec<(&&OperatorBaseProfile, f64)> = leftovers
        .iter()
        .map(|op| (op, dorm_aura_value(op, registry, building_data)))
        .filter(|(_, v)| *v > 0.0)
        .collect();
    aura_ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut aura_iter = aura_ranked.into_iter();
    for (i, dorm) in dorms.iter().enumerate() {
        if budget == 0 {
            break;
        }
        let Some((op, _)) = aura_iter.next() else {
            break;
        };
        if staffed[i].1.len() >= dorm.capacity {
            continue;
        }
        used.insert(op.char_id.as_str());
        staffed[i].1.push(op.char_id.clone());
        budget -= 1;
    }

    let mut single_ranked: Vec<(&&OperatorBaseProfile, f64)> = leftovers
        .iter()
        .filter(|op| !used.contains(op.char_id.as_str()))
        .map(|op| (op, dorm_single_value(op, registry, building_data)))
        .filter(|(_, v)| *v > 0.0)
        .collect();
    single_ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut single_iter = single_ranked.into_iter();
    'outer: for (i, dorm) in dorms.iter().enumerate() {
        // One healer per dorm: also "only the strongest".
        if staffed[i].1.len() >= dorm.capacity {
            continue;
        }
        loop {
            if budget == 0 {
                break 'outer;
            }
            let Some((op, _)) = single_iter.next() else {
                break 'outer;
            };
            if used.insert(op.char_id.as_str()) {
                staffed[i].1.push(op.char_id.clone());
                budget -= 1;
                break;
            }
        }
    }

    staffed
}
