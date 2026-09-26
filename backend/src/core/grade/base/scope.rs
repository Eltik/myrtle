//! A scoped planner run ("optimize this room only").
//!
//! The search plans the scoped rooms beside the frozen production rooms, but
//! it still re-seats the rooms it has no freeze for (dormitories, the Control
//! Center, power plants), and where a scoped room is not the base's
//! bottleneck - a gold factory already covering the posts' demand - every
//! crew meeting that demand ties in the objective, so it may hand back a crew
//! slower than the draft (110% -> 104%, user report 2026-09-26). This pass
//! settles the proposal the way the player reads it: every room outside the
//! scope is exactly the draft, and a scoped room changes only when its own
//! yield rises.

use std::collections::{HashMap, HashSet};

use crate::core::gamedata::types::building::BuildingDataFile;

use super::assignment::compute_current_assignment;
use super::buff_registry::BuffResolutionStrategy;
use super::types::{BaseAssignment, OperatorBaseProfile, RoomAssignment, UserBuilding};
use super::yield_model::room_yield;

/// The scoped proposal as the player will see it once applied: the drafted
/// building with the search's crews in the scoped rooms, scored through the
/// live path, with any scoped room the search made worse kept as drafted.
/// Reverting a room hands its drafted operators back, so a crew the search
/// built from them elsewhere in the scope loses those seats rather than
/// double-booking anyone.
#[allow(clippy::too_many_arguments)]
pub fn settle_scoped_proposal(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    scope: &HashSet<String>,
    baseline: &BaseAssignment,
    proposal: &BaseAssignment,
) -> BaseAssignment {
    let proposed: HashMap<&str, &RoomAssignment> = proposal
        .rooms
        .iter()
        .map(|r| (r.slot_id.as_str(), r))
        .collect();
    let drafted: HashMap<&str, &RoomAssignment> = baseline
        .rooms
        .iter()
        .map(|r| (r.slot_id.as_str(), r))
        .collect();
    let mut keep_draft: HashSet<String> = HashSet::new();
    loop {
        let planned = |room: &super::types::UserRoom| {
            scope.contains(&room.slot_id) && !keep_draft.contains(&room.slot_id)
        };
        let taken: HashSet<&str> = building
            .rooms
            .iter()
            .filter(|r| !planned(r))
            .flat_map(|r| r.current_operators.iter().map(String::as_str))
            .collect();
        let stationed = UserBuilding {
            rooms: building
                .rooms
                .iter()
                .map(|r| {
                    if !planned(r) {
                        return r.clone();
                    }
                    let mut room = r.clone();
                    let p = proposed.get(r.slot_id.as_str());
                    room.current_operators = p.map_or_else(Vec::new, |p| {
                        p.operators
                            .iter()
                            .filter(|o| !taken.contains(o.as_str()))
                            .cloned()
                            .collect()
                    });
                    if let Some(p) = p.filter(|_| r.room_type == "MANUFACTURE") {
                        room.current_formula.clone_from(&p.formula_type);
                    }
                    room
                })
                .collect(),
        };
        let scored = compute_current_assignment(
            operators,
            &stationed,
            building_data,
            registry,
            morale_drains,
            None,
        );
        let worse: Vec<String> = scored
            .rooms
            .iter()
            .filter(|r| scope.contains(&r.slot_id) && !keep_draft.contains(&r.slot_id))
            .filter(|r| {
                drafted
                    .get(r.slot_id.as_str())
                    .is_some_and(|d| is_worse(r, d))
            })
            .map(|r| r.slot_id.clone())
            .collect();
        if worse.is_empty() {
            return scored;
        }
        keep_draft.extend(worse);
    }
}

/// A room reads worse than another when it earns less per day, or the same
/// at a lower efficiency (rooms the yield model prices at zero - power,
/// the Control Center - compare on efficiency alone).
fn is_worse(room: &RoomAssignment, than: &RoomAssignment) -> bool {
    const EPS: f64 = 1e-9;
    let worth = |r: &RoomAssignment| {
        room_yield(
            &r.room_type,
            r.formula_type.as_deref(),
            r.level,
            r.total_efficiency,
            r.order_value,
            r.operators.len(),
            r.order_limit,
        )
        .lmd_equivalent()
    };
    let (a, b) = (worth(room), worth(than));
    a < b - EPS || ((a - b).abs() <= EPS && room.total_efficiency < than.total_efficiency - EPS)
}
