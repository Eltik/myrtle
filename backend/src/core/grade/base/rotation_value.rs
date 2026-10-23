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

use super::assignment::compute_current_assignment_over;
use super::buff_registry::BuffResolutionStrategy;
use super::dorms::dorm_list;
use super::pools::optimal_with_bundles;
use super::shift_rotation::ShiftRotation;
use super::sustain_sim::{MORALE_MAX, SHIFT_HOURS, SIM_HORIZON_HOURS, game_morale_drain};
use super::types::{BaseAssignment, OperatorBaseProfile, UserBuilding, UserRoom};
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

    /// Weighted by `weights` (summing to one): a cycle's legs by their hours.
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

/// Hours a production crew stays seated in this rotation: two shifts when some
/// crew works consecutive shifts (the 24h-block tiling), else one. Time ramps
/// restart with every crew change, so they are valued over this.
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

/// The player's saved presets, valued as they run them.
#[derive(Clone, Debug, PartialEq)]
pub struct PresetsValuation {
    /// Daily figures, weighted by how long each preset works.
    pub value: StaffingValue,
    /// Hours each usable preset works per cycle when the cycle is morale-bound
    /// (two presets, see [`preset_cycle`]); empty when the presets are read as
    /// equal shifts.
    pub legs_hours: Vec<f64>,
}

/// The player's saved presets valued the same way as the recommendation: each
/// usable preset scored through the live path. Three presets are the game's
/// Rotate rhythm, one shift each. Two presets are the hand-run cycle
/// ([`preset_cycle`]): the first works until a room runs dry, the second until
/// the first has recovered, and the figures weight each by its hours. `None`
/// without usable presets.
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
            // Time ramps read the preset's tenure.
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

/// How long each of two presets works per cycle when the player swaps by
/// hand instead of by the clock (uid 93881052, 2026-10-23: "all operators
/// remain working until one facility hits zero morale, then rotate the whole
/// base", 34h then 8-19h). The first preset works until its first worker runs
/// dry; the second until every first-preset worker resting in a dorm is back
/// to full, or until one of its own workers runs dry. Resters recover at their
/// dorm's rate ([`dorm_list`]); an operator neither working nor in a dorm
/// holds. Cycles repeat from full bars until the leg lengths settle. `None`
/// unless exactly two presets are usable.
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
    // (worker id, drain/h) and (rester id, recovery/h) of a preset.
    type Side = Vec<(String, f64)>;
    let leg = |i: usize| -> (Side, Side) {
        let crew = |r: &super::types::UserRoom| -> Vec<String> {
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
        // The first set's workers now resting: the second leg ends when the
        // slowest of them is full, unless a second-set worker runs dry first.
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

/// A hand-swapped two-stationing cycle the optimizer proposes beside the
/// Rotate-rhythm rotation: the first stationing works until a room runs dry,
/// the second until the first has recovered (see [`preset_cycle`]). The game's
/// Rotate button cannot run it; the player swaps by hand, as uid 93881052
/// does (2026-10-23), and their 34h/19h cycle out-earned our 3 x 12h plan by 7%.
#[derive(Clone, Debug)]
pub struct CyclePlan {
    /// Per leg, every room's crew (`slot_id`, operators), dorms included: the
    /// dorms hold the other leg's workers so they recover.
    pub legs: Vec<Vec<(String, Vec<String>)>>,
    /// Hours each leg works per cycle.
    pub legs_hours: Vec<f64>,
    /// Daily figures, leg-weighted.
    pub value: StaffingValue,
}

/// Plan the cycle: the first stationing is the peak plan (`first`), the second
/// the best plan of everyone the first does not work, and each leg's dorms take
/// the other leg's workers, highest drain first into the best dorm. Valued as a
/// saved pair of presets would be, so the comparison is like for like. `None`
/// when a leg's workers outnumber the beds (see below) or a leg has nobody. A
/// first version: the peak staffing is not re-chosen for a longer first leg, and
/// a base short of beds gets no cycle rather than a partial swap.
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
    // Dorm seating per leg: the OTHER leg's workers, highest drain first, best
    // dorm first, as many as the beds take.
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
    // Every worker of a leg must have a bed in the other: an operator who
    // works one leg and rests nowhere runs dry within a few cycles and the
    // first leg collapses to nothing (the full roster on a 4-dorm base: 23
    // workers, 20 beds). No bed, no cycle proposal.
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
