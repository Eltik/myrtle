use std::collections::{HashMap, HashSet};

use super::auxiliary::{
    BACKUP_COVERAGE, BASE_MORALE_DRAIN, MAX_DORM_LEVEL, MIN_RECOVERY_FACTOR, MORALE_RECOVERY,
};
use super::control::ControlCenter;
use super::facilities::effective_facility_counts;
use super::live::seats_of;
use super::teams::{
    applicable_strategies, compute_team_efficiency, factory_cap_factor, room_search_score,
    team_capacity_bonus,
};
use super::{base_flows, build_op_index, optimal_inner};
use crate::{
    registry::BuffResolutionStrategy,
    rotation::dorms::{building_hosts_manager, manager_can_sustain, manager_swap_rate},
    rotation::sustain_sim::game_morale_drain,
    types::{
        BaseAssignment, OperatorBaseProfile, RoomAssignment, RoomRotation, RotationAssignment,
        RotationMember, RotationSet, RotationSetRoom, UserBuilding,
    },
    util::{is_production_room, max_stationed_at_level},
};
use myrtle_gamedata_raw::tables::building::BuildingDataFile;

/// Effective per-hour morale recovery for this base
pub fn morale_recovery(building: &UserBuilding) -> f64 {
    let levels: Vec<i32> = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "DORMITORY")
        .map(|r| r.level)
        .collect();
    if levels.is_empty() {
        return MORALE_RECOVERY * MIN_RECOVERY_FACTOR;
    }
    let avg = f64::from(levels.iter().sum::<i32>()) / levels.len() as f64;
    MORALE_RECOVERY * (avg / MAX_DORM_LEVEL).clamp(MIN_RECOVERY_FACTOR, 1.0)
}

/// Total operators the base can rest at once
pub fn total_dorm_capacity(building: &UserBuilding, building_data: &BuildingDataFile) -> i32 {
    building
        .rooms
        .iter()
        .filter(|r| r.room_type == "DORMITORY")
        .map(|r| max_stationed_at_level(building_data, "DORMITORY", r.level))
        .sum()
}

pub(super) fn dorm_capacity_factor(rest_demand: f64, capacity: i32) -> f64 {
    let cap = f64::from(capacity.max(0));
    if rest_demand <= cap || rest_demand <= 0.0 {
        1.0
    } else {
        cap / rest_demand
    }
}

pub(super) fn cc_global_morale_recovery(
    op: &OperatorBaseProfile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> f64 {
    op.available_buffs
        .iter()
        .filter_map(|b| {
            let buff = building_data.buffs.get(b)?;
            (buff.room_type == "CONTROL").then_some(())?;
            match registry.get(b) {
                Some(BuffResolutionStrategy::MoraleModifier {
                    recovery_per_hour,
                    base_wide: true,
                    ..
                }) => Some(*recovery_per_hour),
                _ => None,
            }
        })
        .sum()
}

pub(super) fn cc_room_morale_recovery(
    op: &OperatorBaseProfile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> f64 {
    op.available_buffs
        .iter()
        .filter_map(|b| {
            let buff = building_data.buffs.get(b)?;
            (buff.room_type == "CONTROL").then_some(())?;
            match registry.get(b) {
                Some(BuffResolutionStrategy::MoraleModifier {
                    recovery_per_hour,
                    base_wide: false,
                    is_self_only: false,
                    ..
                }) if *recovery_per_hour > 0.0 => Some(*recovery_per_hour),
                _ => None,
            }
        })
        .sum()
}

pub fn reserve_cc_morale_seats(
    cc_room: &mut ControlCenter,
    control_slots: i32,
    operators: &[OperatorBaseProfile],
    assigned: &mut HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) {
    let mut morale_ops: Vec<(&OperatorBaseProfile, f64)> = operators
        .iter()
        .filter(|op| !assigned.contains(&op.char_id))
        .filter_map(|op| {
            let boost = cc_global_morale_recovery(op, registry, building_data);
            (boost > 0.0).then_some((op, boost))
        })
        .collect();
    morale_ops.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    for (op, _) in morale_ops {
        if cc_room.operators.len() as i32 >= control_slots {
            break;
        }
        assigned.insert(op.char_id.clone());
        cc_room.operators.push(op.char_id.clone());
    }
}

pub(super) fn cc_morale_boost(
    main: &BaseAssignment,
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> f64 {
    main.rooms
        .iter()
        .filter(|r| r.room_type == "CONTROL")
        .flat_map(|r| r.operators.iter())
        .filter_map(|id| op_index.get(id.as_str()))
        .map(|op| cc_global_morale_recovery(op, registry, building_data))
        .sum()
}

pub(super) fn op_morale_drain(
    op: &OperatorBaseProfile,
    morale_drains: &HashMap<String, f64>,
) -> f64 {
    let modifier: f64 = op
        .available_buffs
        .iter()
        .filter_map(|b| morale_drains.get(b))
        .sum();
    (BASE_MORALE_DRAIN + modifier).max(0.05)
}

/// Fraction of time an operator is working at steady state
pub fn op_uptime(
    op: &OperatorBaseProfile,
    morale_drains: &HashMap<String, f64>,
    recovery: f64,
) -> f64 {
    let d = op_morale_drain(op, morale_drains);
    recovery / (d + recovery)
}

pub(super) const ROTATION_BASE_HOURS: f64 = 24.0;

pub(super) fn op_lasts_hours(
    op: &OperatorBaseProfile,
    morale_drains: &HashMap<String, f64>,
    room_aura: f64,
) -> f64 {
    let drain = (op_morale_drain(op, morale_drains) + room_aura).max(0.05);
    (ROTATION_BASE_HOURS * BASE_MORALE_DRAIN / drain).round()
}

pub fn crew_drain_aura(
    crew: &[String],
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> f64 {
    use crate::scoring::clause::{ClauseKind, Metric, clauses_from_strategy};
    crew.iter()
        .filter_map(|id| op_index.get(id.as_str()))
        .flat_map(|op| op.available_buffs.iter())
        .filter_map(|b| {
            let buff = building_data.buffs.get(b)?;
            let strategy = registry.get(b)?;
            Some(
                clauses_from_strategy(b, buff, strategy)
                    .into_iter()
                    .filter(|c| c.metric == Metric::MoraleDrainAura)
                    .filter(|c| matches!(c.kind, ClauseKind::SelfValue))
                    .map(|c| c.value)
                    .sum::<f64>(),
            )
        })
        .sum()
}

pub fn has_drain_aura_immunity(
    op: &OperatorBaseProfile,
    room_type: &str,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> bool {
    op.available_buffs.iter().any(|b| {
        building_data
            .buffs
            .get(b)
            .is_some_and(|buff| buff.room_type == room_type)
            && matches!(
                registry.get(b),
                Some(BuffResolutionStrategy::MoraleDrainAuraImmunity)
            )
    })
}

/// The roster's morale-swap manager, if any
pub fn morale_swap_enabler(
    operators: &[OperatorBaseProfile],
    building_data: &BuildingDataFile,
) -> Option<String> {
    operators
        .iter()
        .find(|op| {
            op.available_buffs.iter().any(|b| {
                building_data.buffs.get(b).is_some_and(|buff| {
                    buff.description.to_lowercase().contains("swap")
                        && buff.description.to_lowercase().contains("morale with")
                })
            })
        })
        .map(|op| op.char_id.clone())
}

pub fn num_morale_swap_managers(
    operators: &[OperatorBaseProfile],
    building_data: &BuildingDataFile,
) -> usize {
    operators
        .iter()
        .filter(|op| {
            op.available_buffs.iter().any(|b| {
                building_data.buffs.get(b).is_some_and(|buff| {
                    let d = buff.description.to_lowercase();
                    d.contains("swap") && d.contains("morale with")
                })
            })
        })
        .count()
}

/// Production operators the roster's morale-swap managers keep at full morale 24/7
pub fn morale_sustained_beneficiaries(
    main: &BaseAssignment,
    operators: &[OperatorBaseProfile],
    morale_drains: &HashMap<String, f64>,
    recovery: f64,
    building: &UserBuilding,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> HashSet<String> {
    let managers = num_morale_swap_managers(operators, building_data);
    if managers == 0 || !building_hosts_manager(building, building_data) {
        return HashSet::new();
    }
    let swap_rate = morale_swap_enabler(operators, building_data)
        .and_then(|id| operators.iter().find(|o| o.char_id == id))
        .map_or(0.0, |m| manager_swap_rate(m, registry, building_data));
    let op_index = build_op_index(operators);
    let mut ranked: Vec<(String, f64)> = Vec::new();
    for r in main
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
    {
        let n = r.operators.len().max(1) as f64;
        let room_mag = r.total_efficiency + r.order_value;
        for id in &r.operators {
            if let Some(op) = op_index.get(id.as_str()) {
                if !manager_can_sustain(swap_rate, game_morale_drain(op, morale_drains)) {
                    continue;
                }
                let up = op_uptime(op, morale_drains, recovery);
                let gain = room_mag * (1.0 - up) * (1.0 - BACKUP_COVERAGE) / n;
                if gain > 1e-9 {
                    ranked.push((id.clone(), gain));
                }
            }
        }
    }
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked
        .into_iter()
        .take(managers)
        .map(|(id, _)| id)
        .collect()
}

pub(super) fn room_sustain_factor(
    room_ops: &[String],
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    morale_drains: &HashMap<String, f64>,
    recovery: f64,
    morale_sustained: &HashSet<String>,
) -> f64 {
    let uptimes: Vec<f64> = room_ops
        .iter()
        .filter_map(|id| op_index.get(id.as_str()).map(|op| (id, op)))
        .map(|(id, op)| {
            if morale_sustained.contains(id) {
                1.0
            } else {
                op_uptime(op, morale_drains, recovery)
            }
        })
        .collect();
    if uptimes.is_empty() {
        return 1.0;
    }
    let avg_uptime = uptimes.iter().sum::<f64>() / uptimes.len() as f64;
    1.0 - (1.0 - avg_uptime) * (1.0 - BACKUP_COVERAGE)
}

/// Sustained 24/7 efficiency of a staffing under a staggered rotation
pub fn sustained_efficiency_of(
    main: &BaseAssignment,
    operators: &[OperatorBaseProfile],
    morale_drains: &HashMap<String, f64>,
    recovery: f64,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    morale_sustained: &HashSet<String>,
) -> f64 {
    let op_index = build_op_index(operators);
    main.rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
        .map(|r| {
            let rotation = room_sustain_factor(
                &r.operators,
                &op_index,
                morale_drains,
                recovery,
                morale_sustained,
            );
            let cap = team_capacity_bonus(
                &r.operators,
                &r.room_type,
                r.formula_type.as_deref(),
                &op_index,
                registry,
                building_data,
            );
            let cap_factor = factory_cap_factor(
                building_data,
                r.formula_type.as_deref(),
                r.level,
                r.total_efficiency,
                cap,
            );
            r.total_efficiency * rotation * cap_factor
        })
        .sum()
}

/// Sustained LMD-equivalent daily value of an assignment
pub fn sustained_assignment_value(
    main: &BaseAssignment,
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> f64 {
    let op_index = build_op_index(operators);
    let cc_recovery = cc_morale_boost(main, &op_index, registry, building_data);
    let recovery = morale_recovery(building) + cc_recovery;
    let morale_sustained = morale_sustained_beneficiaries(
        main,
        operators,
        morale_drains,
        recovery,
        building,
        registry,
        building_data,
    );
    let throttled: Vec<RoomAssignment> = main
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
        .filter(|r| !r.operators.is_empty())
        .map(|r| {
            let rotation = room_sustain_factor(
                &r.operators,
                &op_index,
                morale_drains,
                recovery,
                &morale_sustained,
            );
            let cap = team_capacity_bonus(
                &r.operators,
                &r.room_type,
                r.formula_type.as_deref(),
                &op_index,
                registry,
                building_data,
            );
            let cap_factor = factory_cap_factor(
                building_data,
                r.formula_type.as_deref(),
                r.level,
                r.total_efficiency,
                cap,
            );
            RoomAssignment {
                total_efficiency: r.total_efficiency * rotation * cap_factor,
                ..r.clone()
            }
        })
        .collect();
    let mut flows = base_flows(&throttled);
    flows.gold_factories += idle_gold_factories(&main.rooms);
    flows.total_value()
}

pub(super) fn idle_gold_factories(rooms: &[RoomAssignment]) -> usize {
    rooms
        .iter()
        .filter(|r| r.operators.is_empty() && is_gold_factory(r))
        .count()
}

pub(super) fn is_gold_factory(room: &RoomAssignment) -> bool {
    room.room_type == "MANUFACTURE" && room.formula_type.as_deref() == Some("F_GOLD")
}

pub fn compute_sustained_assignment(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> RotationAssignment {
    let main = optimal_inner(
        operators,
        building,
        building_data,
        registry,
        morale_drains,
        &[],
        true,
    );
    let facility_counts = effective_facility_counts(
        building,
        operators,
        registry,
        building_data,
        &seats_of(&main),
    );
    let total_dorm_levels = building.total_dorm_levels();
    let op_index = build_op_index(operators);
    let cc_recovery = cc_morale_boost(&main, &op_index, registry, building_data);
    let recovery = morale_recovery(building) + cc_recovery;

    let morale_sustained = morale_sustained_beneficiaries(
        &main,
        operators,
        morale_drains,
        recovery,
        building,
        registry,
        building_data,
    );

    let mut sustained_efficiency = sustained_efficiency_of(
        &main,
        operators,
        morale_drains,
        recovery,
        registry,
        building_data,
        &morale_sustained,
    );

    let rest_demand: f64 = main
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
        .flat_map(|r| r.operators.iter())
        .filter(|id| !morale_sustained.contains(*id))
        .filter_map(|id| op_index.get(id.as_str()).copied())
        .map(|op| 1.0 - op_uptime(op, morale_drains, recovery))
        .sum();
    sustained_efficiency *=
        dorm_capacity_factor(rest_demand, total_dorm_capacity(building, building_data));

    let mains: HashSet<&str> = main
        .rooms
        .iter()
        .flat_map(|r| r.operators.iter().map(String::as_str))
        .collect();

    let best_filler = |room_type: &str,
                       formula: Option<&str>,
                       room_mains: &[String]|
     -> Option<&OperatorBaseProfile> {
        let score = |ops: &[String]| -> f64 {
            let (s, v) = compute_team_efficiency(
                ops,
                room_type,
                formula,
                None,
                &op_index,
                registry,
                building_data,
                &facility_counts,
                total_dorm_levels,
                morale_drains,
                &[],
            );
            room_search_score(room_type, s, v)
        };
        let base = score(room_mains);
        operators
            .iter()
            .filter(|op| !mains.contains(op.char_id.as_str()))
            .filter(|op| {
                applicable_strategies(op, room_type, formula, registry, building_data)
                    .next()
                    .is_some()
            })
            .filter_map(|op| {
                let mut with = room_mains.to_vec();
                with.push(op.char_id.clone());
                let marginal = score(&with) - base;
                (marginal > 1e-6).then_some((op, marginal))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(op, _)| op)
    };

    let mut rooms = Vec::new();
    let mut bench: Vec<String> = Vec::new();
    for room in main
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
        .filter(|r| !r.operators.is_empty())
    {
        let formula = room.formula_type.as_deref();

        let room_aura = crew_drain_aura(&room.operators, &op_index, building_data, registry);
        let mut members: Vec<RotationMember> = room
            .operators
            .iter()
            .filter_map(|id| op_index.get(id.as_str()).copied())
            .map(|op| {
                let aura = if has_drain_aura_immunity(op, &room.room_type, registry, building_data)
                {
                    -cc_recovery
                } else {
                    room_aura - cc_recovery
                };
                RotationMember {
                    operator: op.char_id.clone(),
                    lasts_hours: if morale_sustained.contains(&op.char_id) {
                        f64::INFINITY
                    } else {
                        op_lasts_hours(op, morale_drains, aura)
                    },
                }
            })
            .collect();
        members.sort_by(|a, b| {
            a.lasts_hours
                .partial_cmp(&b.lasts_hours)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let backup = if room.locked {
            None
        } else {
            best_filler(&room.room_type, formula, &room.operators).map(|op| op.char_id.clone())
        };
        if let Some(id) = &backup
            && !bench.contains(id)
        {
            bench.push(id.clone());
        }

        rooms.push(RoomRotation {
            slot_id: room.slot_id.clone(),
            room_type: room.room_type.clone(),
            members,
            backup,
        });
    }

    const MAX_ROTATION_SETS: usize = 3;
    const ROTATION_CYCLE_HOURS: f64 = 36.0;
    let needs_rest = |m: &RotationMember| m.lasts_hours <= ROTATION_CYCLE_HOURS;
    let num_sets = rooms
        .iter()
        .map(|r| {
            if r.backup.is_some() {
                r.members.iter().filter(|m| needs_rest(m)).count()
            } else {
                0
            }
        })
        .max()
        .unwrap_or(0)
        .min(MAX_ROTATION_SETS);
    let sets: Vec<RotationSet> = (0..num_sets)
        .map(|k| RotationSet {
            rooms: rooms
                .iter()
                .map(|r| {
                    let mut working: Vec<String> =
                        r.members.iter().map(|m| m.operator.clone()).collect();
                    let resting = match (r.members.get(k), &r.backup) {
                        (Some(m), Some(b)) if needs_rest(m) => {
                            working[k] = b.clone();
                            Some(m.operator.clone())
                        }
                        _ => None,
                    };
                    RotationSetRoom {
                        slot_id: r.slot_id.clone(),
                        room_type: r.room_type.clone(),
                        working,
                        resting,
                    }
                })
                .collect(),
        })
        .collect();

    RotationAssignment {
        main,
        rooms,
        shared_bench: bench,
        sets,
        sustained_efficiency,
    }
}

/// Depleted as the game shows it
pub const INERT_MORALE: f64 = 1.0 / 60.0;
