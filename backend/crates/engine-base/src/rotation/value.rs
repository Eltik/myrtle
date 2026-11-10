//! A rotation's worth, each shift scored through the live path

use std::collections::HashMap;

use myrtle_gamedata_raw::tables::building::BuildingDataFile;

use crate::assignment::compute_current_assignment_over;
use crate::registry::BuffResolutionStrategy;
use crate::rotation::dorms::dorm_list;
use crate::rotation::shift::ShiftRotation;
use crate::rotation::sustain_sim::{MORALE_MAX, SHIFT_HOURS, SIM_HORIZON_HOURS, game_morale_drain};
use crate::scoring::pools::optimal_with_bundles;
use crate::scoring::yield_model::BaseFlows;
use crate::types::{BaseAssignment, OperatorBaseProfile, UserBuilding, UserRoom};
use crate::util::is_production_room;

/// Daily figures of one staffing
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
            lmd_per_day: flows.lmd_per_day(),
            exp_per_day: flows.exp_per_day(),
            total_value: flows.value_with_drones(),
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

    fn weighted(values: &[Self], weights: &[f64]) -> Self {
        let sum = |pick: fn(&Self) -> f64| -> f64 {
            values.iter().zip(weights).map(|(v, w)| pick(v) * w).sum()
        };
        Self {
            production_efficiency: sum(|v| v.production_efficiency),
            lmd_per_day: sum(|v| v.lmd_per_day),
            exp_per_day: sum(|v| v.exp_per_day),
            total_value: sum(|v| v.total_value),
        }
    }
}

/// The recommended rotation valued as the player will run it
pub fn rotation_value(
    rotation: &ShiftRotation,
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> Option<StaffingValue> {
    let tenure = crew_tenure_hours(rotation);
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
            StaffingValue::of(&compute_current_assignment_over(
                operators,
                &stationed,
                building_data,
                registry,
                morale_drains,
                None,
                tenure,
            ))
        })
        .collect();
    StaffingValue::mean(&values)
}

fn crew_tenure_hours(rotation: &ShiftRotation) -> f64 {
    let n = rotation.shifts.len();
    let stays = n > 1
        && rotation.shifts.iter().enumerate().any(|(k, shift)| {
            let next = &rotation.shifts[(k + 1) % n];
            shift.rooms.iter().any(|c| {
                c.active
                    && is_production_room(&c.room_type)
                    && !c.recommended.is_empty()
                    && next.rooms.iter().any(|d| {
                        d.slot_id == c.slot_id && d.active && d.recommended == c.recommended
                    })
            })
        });
    if stays {
        2.0 * SHIFT_HOURS
    } else {
        SHIFT_HOURS
    }
}

pub fn usable_preset_shifts(building: &UserBuilding) -> Vec<usize> {
    let with_presets: Vec<&crate::types::UserRoom> = building
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

/// The player's saved presets, valued as they run them
#[derive(Clone, Debug, PartialEq)]
pub struct PresetsValuation {
    pub value: StaffingValue,
    /// Hours each usable preset works per cycle; empty for equal shifts
    pub legs_hours: Vec<f64>,
}

pub fn presets_value(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> Option<PresetsValuation> {
    let usable = usable_preset_shifts(building);
    let legs = preset_cycle(&usable, operators, building, building_data, morale_drains);
    let hours: Vec<f64> = legs
        .clone()
        .unwrap_or_else(|| vec![SHIFT_HOURS; usable.len()]);
    let values: Vec<StaffingValue> = usable
        .iter()
        .zip(&hours)
        .map(|(&i, &h)| {
            StaffingValue::of(&compute_current_assignment_over(
                operators,
                building,
                building_data,
                registry,
                morale_drains,
                Some(i),
                h.max(1.0),
            ))
        })
        .collect();
    let total: f64 = hours.iter().sum();
    let value = if total > 0.0 {
        let w: Vec<f64> = hours.iter().map(|h| h / total).collect();
        Some(StaffingValue::weighted(&values, &w))
    } else {
        StaffingValue::mean(&values)
    }?;
    Some(PresetsValuation {
        value,
        legs_hours: legs.unwrap_or_default(),
    })
}

/// How long each of two presets works per cycle when swapped by hand
pub fn preset_cycle(
    usable: &[usize],
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    morale_drains: &HashMap<String, f64>,
) -> Option<Vec<f64>> {
    let [a, b] = usable else {
        return None;
    };
    let profiles: HashMap<&str, &OperatorBaseProfile> =
        operators.iter().map(|o| (o.char_id.as_str(), o)).collect();
    let dorm_rate: HashMap<String, f64> = dorm_list(building, building_data)
        .into_iter()
        .map(|d| (d.slot_id, d.recovery_per_hour))
        .collect();
    type Side = Vec<(String, f64)>;
    let leg = |i: usize| -> (Side, Side) {
        let crew = |r: &crate::types::UserRoom| -> Vec<String> {
            r.preset_shifts
                .get(i)
                .cloned()
                .unwrap_or_else(|| r.current_operators.clone())
        };
        let mut workers: Vec<(String, f64)> = Vec::new();
        for r in building.rooms.iter().filter(|r| r.room_type != "DORMITORY") {
            for id in crew(r) {
                if workers.iter().any(|(w, _)| *w == id) {
                    continue;
                }
                let drain = profiles
                    .get(id.as_str())
                    .map_or(0.0, |o| game_morale_drain(o, morale_drains));
                workers.push((id, drain));
            }
        }
        let mut resters: Vec<(String, f64)> = Vec::new();
        for r in building.rooms.iter().filter(|r| r.room_type == "DORMITORY") {
            let rate = dorm_rate.get(&r.slot_id).copied().unwrap_or(0.0);
            for id in crew(r) {
                if workers.iter().any(|(w, _)| *w == id) || resters.iter().any(|(w, _)| *w == id) {
                    continue;
                }
                resters.push((id, rate));
            }
        }
        (workers, resters)
    };
    let (work_a, rest_a) = leg(*a);
    let (work_b, rest_b) = leg(*b);
    if work_a.is_empty() || work_b.is_empty() {
        return None;
    }
    let mut morale: HashMap<String, f64> = HashMap::new();
    for (id, _) in work_a.iter().chain(&rest_a).chain(&work_b).chain(&rest_b) {
        morale.insert(id.clone(), MORALE_MAX);
    }
    let first_dry = |workers: &[(String, f64)], morale: &HashMap<String, f64>| -> f64 {
        workers
            .iter()
            .filter(|(_, d)| *d > 0.0)
            .map(|(id, d)| morale.get(id).copied().unwrap_or(MORALE_MAX) / d)
            .fold(SIM_HORIZON_HOURS, f64::min)
    };
    let advance = |workers: &[(String, f64)],
                   resters: &[(String, f64)],
                   hours: f64,
                   morale: &mut HashMap<String, f64>| {
        for (id, d) in workers {
            let m = morale.entry(id.clone()).or_insert(MORALE_MAX);
            *m = (*m - d * hours).max(0.0);
        }
        for (id, r) in resters {
            let m = morale.entry(id.clone()).or_insert(MORALE_MAX);
            *m = (*m + r * hours).min(MORALE_MAX);
        }
    };
    let mut legs = (0.0, 0.0);
    for _ in 0..8 {
        let t_a = first_dry(&work_a, &morale);
        advance(&work_a, &rest_a, t_a, &mut morale);
        let recovered = work_a
            .iter()
            .filter_map(|(id, _)| rest_b.iter().find(|(r, _)| r == id))
            .filter(|(_, rate)| *rate > 0.0)
            .map(|(id, rate)| (MORALE_MAX - morale.get(id).copied().unwrap_or(MORALE_MAX)) / rate)
            .fold(0.0, f64::max);
        let t_b = first_dry(&work_b, &morale).min(if recovered > 0.0 {
            recovered
        } else {
            SIM_HORIZON_HOURS
        });
        advance(&work_b, &rest_b, t_b, &mut morale);
        let settled = (t_a - legs.0).abs() < 0.05 && (t_b - legs.1).abs() < 0.05;
        legs = (t_a, t_b);
        if settled {
            break;
        }
    }
    (legs.0 + legs.1 > 0.0).then_some(vec![legs.0, legs.1])
}

/// A hand-swapped two-stationing cycle
#[derive(Clone, Debug)]
pub struct CyclePlan {
    /// Per leg, every room's crew (`slot_id`, operators), dorms included
    pub legs: Vec<Vec<(String, Vec<String>)>>,
    pub legs_hours: Vec<f64>,
    pub value: StaffingValue,
}

#[allow(clippy::too_many_arguments)]
pub fn plan_cycle(
    first: &BaseAssignment,
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    base_registry: &HashMap<String, BuffResolutionStrategy>,
    search_registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    pins: &[(String, String)],
) -> Option<CyclePlan> {
    let workers = |asn: &BaseAssignment| -> Vec<String> {
        let mut ids: Vec<String> = Vec::new();
        for r in asn.rooms.iter().filter(|r| r.room_type != "DORMITORY") {
            for id in &r.operators {
                if !ids.contains(id) {
                    ids.push(id.clone());
                }
            }
        }
        ids
    };
    let first_workers = workers(first);
    if first_workers.is_empty() {
        return None;
    }
    let rest: Vec<OperatorBaseProfile> = operators
        .iter()
        .filter(|o| !first_workers.contains(&o.char_id))
        .cloned()
        .collect();
    let rest_pins: Vec<(String, String)> = pins
        .iter()
        .filter(|(id, _)| rest.iter().any(|o| &o.char_id == id))
        .cloned()
        .collect();
    let second = optimal_with_bundles(
        &rest,
        building,
        building_data,
        base_registry,
        search_registry,
        morale_drains,
        &rest_pins,
    )
    .optimal;
    let second_workers = workers(&second);
    if second_workers.is_empty() {
        return None;
    }
    let crew_of = |asn: &BaseAssignment, slot: &str| -> Vec<String> {
        asn.rooms
            .iter()
            .find(|r| r.slot_id == slot)
            .map(|r| r.operators.clone())
            .unwrap_or_default()
    };
    let by_drain = |ids: &[String]| -> Vec<String> {
        let mut v: Vec<(String, f64)> = ids
            .iter()
            .map(|id| {
                let d = operators
                    .iter()
                    .find(|o| &o.char_id == id)
                    .map_or(0.0, |o| game_morale_drain(o, morale_drains));
                (id.clone(), d)
            })
            .collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        v.into_iter().map(|(id, _)| id).collect()
    };
    let dorms = dorm_list(building, building_data);
    let seat = |resters: &[String]| -> HashMap<String, Vec<String>> {
        let mut out: HashMap<String, Vec<String>> = HashMap::new();
        let mut next = resters.iter();
        for d in &dorms {
            let crew: Vec<String> = next.by_ref().take(d.capacity).cloned().collect();
            out.insert(d.slot_id.clone(), crew);
        }
        out
    };
    let beds: usize = dorms.iter().map(|d| d.capacity).sum();
    if first_workers.len() > beds || second_workers.len() > beds {
        return None;
    }
    let dorms_a = seat(&by_drain(&second_workers));
    let dorms_b = seat(&by_drain(&first_workers));
    let mut staged = building.clone();
    for r in &mut staged.rooms {
        let (a, b) = if r.room_type == "DORMITORY" {
            (
                dorms_a.get(&r.slot_id).cloned().unwrap_or_default(),
                dorms_b.get(&r.slot_id).cloned().unwrap_or_default(),
            )
        } else {
            (crew_of(first, &r.slot_id), crew_of(&second, &r.slot_id))
        };
        r.current_operators.clone_from(&a);
        r.preset_shifts = vec![a, b];
    }
    let valued = presets_value(
        operators,
        &staged,
        building_data,
        base_registry,
        morale_drains,
    )?;
    if valued.legs_hours.len() != 2 {
        return None;
    }
    let leg = |i: usize| -> Vec<(String, Vec<String>)> {
        staged
            .rooms
            .iter()
            .map(|r: &UserRoom| (r.slot_id.clone(), r.preset_shifts[i].clone()))
            .collect()
    };
    Some(CyclePlan {
        legs: vec![leg(0), leg(1)],
        legs_hours: valued.legs_hours,
        value: valued.value,
    })
}
