//! A rotation's worth, read the way the grade reads a base: each shift
//! stationed and scored through the live path, averaged over the shifts.
//!
//! The Optimize button's proposal is the PEAK single staffing; a player who
//! runs the recommended rotation stations one shift of it at a time, so the
//! peak always reads above their base and "optimized vs your base" never
//! closed (user report 2026-10-15: shifts set exactly as suggested, still
//! "+400 EXP, 6 rooms restaffed"). The honest comparison is rotation against
//! rotation: ours, and the player's saved presets, both valued here.

use std::collections::HashMap;

use crate::core::gamedata::types::building::BuildingDataFile;

use super::assignment::compute_current_assignment;
use super::buff_registry::BuffResolutionStrategy;
use super::shift_rotation::ShiftRotation;
use super::types::{BaseAssignment, OperatorBaseProfile, UserBuilding};
use super::util::is_production_room;
use super::yield_model::BaseFlows;

/// Daily figures of one staffing, as the assignment DTO reports them.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StaffingValue {
    pub production_efficiency: f64,
    pub lmd_per_day: f64,
    pub exp_per_day: f64,
    pub total_value: f64,
}

impl StaffingValue {
    pub fn of(asn: &BaseAssignment) -> Self {
        let mut flows = BaseFlows::default();
        for r in &asn.rooms {
            flows.add_room(
                &r.room_type,
                r.formula_type.as_deref(),
                r.level,
                r.total_efficiency,
                r.order_gold,
                r.order_value,
                r.operators.len(),
                r.order_limit,
            );
        }
        Self {
            production_efficiency: asn.total_production_efficiency,
            lmd_per_day: flows.realized_lmd(),
            exp_per_day: flows.exp,
            total_value: flows.total_value(),
        }
    }

    fn mean(values: &[Self]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }
        #[allow(clippy::cast_precision_loss)]
        let n = values.len() as f64;
        Some(Self {
            production_efficiency: values.iter().map(|v| v.production_efficiency).sum::<f64>() / n,
            lmd_per_day: values.iter().map(|v| v.lmd_per_day).sum::<f64>() / n,
            exp_per_day: values.iter().map(|v| v.exp_per_day).sum::<f64>() / n,
            total_value: values.iter().map(|v| v.total_value).sum::<f64>() / n,
        })
    }
}

/// The recommended rotation valued as the player will run it: every shift's
/// active cells stationed on the building and scored through the live path,
/// averaged over the shifts.
pub fn rotation_value(
    rotation: &ShiftRotation,
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> Option<StaffingValue> {
    let values: Vec<StaffingValue> = rotation
        .shifts
        .iter()
        .map(|shift| {
            let mut stationed = building.clone();
            for r in &mut stationed.rooms {
                r.current_operators = shift
                    .rooms
                    .iter()
                    .find(|c| c.slot_id == r.slot_id && c.active)
                    .map(|c| c.recommended.clone())
                    .unwrap_or_default();
            }
            StaffingValue::of(&compute_current_assignment(
                operators,
                &stationed,
                building_data,
                registry,
                morale_drains,
                None,
            ))
        })
        .collect();
    StaffingValue::mean(&values)
}

/// The preset indices the grade counts: every production room with presets
/// has a non-empty crew at that index.
pub fn usable_preset_shifts(building: &UserBuilding) -> Vec<usize> {
    let with_presets: Vec<&super::types::UserRoom> = building
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type) && !r.preset_shifts.is_empty())
        .collect();
    let most = with_presets
        .iter()
        .map(|r| r.preset_shifts.len())
        .max()
        .unwrap_or(0);
    (0..most)
        .filter(|&i| {
            with_presets
                .iter()
                .all(|r| r.preset_shifts.get(i).is_some_and(|c| !c.is_empty()))
        })
        .collect()
}

/// The player's saved presets valued the same way: each usable preset shift
/// scored through the live path, averaged. `None` without usable presets.
pub fn presets_value(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> Option<StaffingValue> {
    let values: Vec<StaffingValue> = usable_preset_shifts(building)
        .into_iter()
        .map(|i| {
            StaffingValue::of(&compute_current_assignment(
                operators,
                building,
                building_data,
                registry,
                morale_drains,
                Some(i),
            ))
        })
        .collect();
    StaffingValue::mean(&values)
}
