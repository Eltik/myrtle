//! Dormitory modeling: which dorm an operator rests in, at what rate, and who
//! staffs the dorms to make everyone else's rest faster.
//!
//! Dorms produce nothing, so every earlier pass treated them as one averaged
//! recovery pool. That erases the two decisions the game actually rewards:
//! resting the neediest operators in the HIGHEST-level dorm (a 2/5/2 base's
//! dorms are deliberately under-leveled to afford the fifth factory, so its
//! one good dorm matters), and seating dorm-skill operators - whole-dorm
//! auras ("+0.15/hr to all Operators in that Dormitory") and single-target
//! healers ("+0.55/hr to another Operator whose Morale is not full") - as
//! permanent dorm staff.

use std::collections::{HashMap, HashSet};

use crate::core::gamedata::types::building::BuildingDataFile;

use super::buff_registry::BuffResolutionStrategy;
use super::types::{OperatorBaseProfile, UserBuilding};
use super::util::max_stationed_at_level;

/// One dormitory of the base, with its game-true recovery rate.
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

/// The base's dormitories, BEST FIRST (highest recovery rate, then level, then
/// slot id for determinism). Fill order everywhere: the neediest rester gets
/// the best dorm.
pub fn dorm_list(building: &UserBuilding, building_data: &BuildingDataFile) -> Vec<Dorm> {
    let phases = &building_data.dorm_data.phases;
    let mut dorms: Vec<Dorm> = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "DORMITORY")
        .map(|r| {
            let idx = (r.level.max(1) as usize - 1).min(phases.len().saturating_sub(1));
            let rate = phases
                .get(idx)
                .map_or(0.0, |p| f64::from(p.manpower_recover) / 100.0);
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

/// An operator's WHOLE-DORM recovery aura ("+X/hr to all Operators in that
/// Dormitory"), 0 when they have none. The game's non-stacking rule ("only the
/// strongest effect of this type") means one aura holder per dorm is the whole
/// benefit - value ranked for exactly that seat.
pub fn dorm_aura_value(
    op: &OperatorBaseProfile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> f64 {
    dorm_skill_value(op, registry, building_data, false)
}

/// An operator's SINGLE-TARGET dorm heal ("+X/hr to another Operator in that
/// Dormitory whose Morale is not full"), 0 when they have none. Also
/// non-stacking within its own type.
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

/// The `(manager, "DORMITORY")` pin a plan should reserve when the roster owns
/// both a morale-conditional generator (Ling's "when own Morale is above/below
/// N" grants) and a morale-swap manager (Fiammetta) - the manager works FROM a
/// dormitory seat, holding the generator's morale where its grant fires.
/// `None` when either half is missing; the flag for "you'd want one but don't
/// own one" is the caller's `has_conditional && pin.is_none()`.
pub fn morale_manager_pin(
    profiles: &[OperatorBaseProfile],
    registry_building_data: &BuildingDataFile,
) -> Option<(String, String)> {
    let has_conditional = profiles
        .iter()
        .any(|op| super::pools::has_morale_conditional_grant(op, registry_building_data));
    if !has_conditional {
        return None;
    }
    super::assignment::morale_swap_enabler(profiles, registry_building_data)
        .map(|id| (id, "DORMITORY".to_string()))
}

/// Permanent dorm staff for a rotation: dorm-skill holders seated 24/7 in
/// specific dorms, chosen from operators the plan left unseated.
///
/// Policy (the same one players run):
/// - one whole-dorm AURA holder per dorm, strongest first into the best dorm -
///   the non-stacking rule makes a second aura in the same dorm worthless;
/// - then single-target healers wherever seats remain, strongest first into
///   the best dorm with room;
/// - never more staff than `headroom` - every staffed seat is one fewer
///   rester the dorms can hold, so callers pass how many seats the resting
///   rhythm can spare (total capacity minus peak resting demand). Boosting
///   recovery is worthless if it evicts the people who need to recover.
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

    // Whole-dorm auras: one per dorm, best aura into the best dorm.
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

    // Single-target healers: fill remaining budgeted seats, best dorm first.
    let mut single_ranked: Vec<(&&OperatorBaseProfile, f64)> = leftovers
        .iter()
        .filter(|op| !used.contains(op.char_id.as_str()))
        .map(|op| (op, dorm_single_value(op, registry, building_data)))
        .filter(|(_, v)| *v > 0.0)
        .collect();
    single_ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut single_iter = single_ranked.into_iter();
    'outer: for (i, dorm) in dorms.iter().enumerate() {
        // One healer per dorm: their skill is also "only the strongest".
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
