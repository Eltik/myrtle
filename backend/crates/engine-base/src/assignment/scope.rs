//! Scoped planner run

use std::collections::{HashMap, HashSet};

use myrtle_gamedata_raw::tables::building::BuildingDataFile;

use crate::assignment::compute_current_assignment;
use crate::registry::BuffResolutionStrategy;
use crate::scoring::yield_model::room_yield;
use crate::types::{BaseAssignment, OperatorBaseProfile, RoomAssignment, UserBuilding};

/// The scoped proposal as applied
#[allow(clippy::too_many_arguments)]
#[must_use]
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
        let planned = |room: &crate::types::UserRoom| {
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
            drone_capacity: building.drone_capacity,
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
