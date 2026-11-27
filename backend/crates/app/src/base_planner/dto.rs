use std::collections::{HashMap, HashSet};

use myrtle_engine_base::assignment::{
    build_op_index, cc_non_production_effects, effective_facility_counts, resolve_base_wide,
    resolve_layout_branches, resolve_room_presence, team_value, working_crew,
};
use myrtle_engine_base::registry::{
    BuffResolutionStrategy, build_name_to_char, targeted_morale_effects,
};
use myrtle_engine_base::rotation::shift::{Shift, ShiftRoom, ShiftRotation};
use myrtle_engine_base::rotation::sustain_sim::{
    SustainabilityReport, Verdict, simulate_rotation_from,
};
use myrtle_engine_base::scoring::skill_ledger::{
    LedgerCtx, LedgerLine, LineDisposition, control_room_ledger, grants_of, production_room_ledger,
};
use myrtle_engine_base::scoring::yield_model::{BaseFlows, room_yield};
use myrtle_engine_base::types::{
    BaseAssignment, OperatorBaseProfile, RoomAssignment, UserBuilding,
};
use myrtle_engine_base::{BaseData, is_production_room};
use serde::Serialize;

const EQUIVALENT_LENIENCY: f64 = 0.05;

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BaseAssignmentDto {
    pub rooms: Vec<RoomAssignmentDto>,
    pub total_production_efficiency: f64,
    pub yield_lmd_per_day: f64,
    pub yield_exp_per_day: f64,
    pub yield_total_value: f64,
    pub drones_per_day: f64,
    pub drone_lmd_per_day: f64,
    pub drone_exp_per_day: f64,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RoomAssignmentDto {
    pub slot_id: String,
    pub room_type: String,
    pub level: i32,
    pub formula_type: Option<String>,
    pub total_efficiency: f64,
    pub order_value: f64,
    pub locked: bool,
    pub operators: Vec<AssignedOperator>,
    pub yield_lmd_per_day: f64,
    pub yield_gold_per_day: f64,
    pub yield_exp_per_day: f64,
    pub non_production: Vec<NonProdEffectDto>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ledger: Vec<SkillLineDto>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<i32>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct NonProdEffectDto {
    pub room_type: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct SkillLineDto {
    pub operator_id: String,
    pub operator_name: String,
    pub buff_id: String,
    pub buff_name: String,
    pub speed_pct: f64,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub value_pct: f64,
    #[serde(default)]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub from_control_center: bool,
    pub disposition: LineDisposition,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(v: &f64) -> bool {
    v.abs() < 1e-9
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct AssignedOperator {
    pub operator_id: String,
    pub name: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub bench: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub parked: bool,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct MovedOperator {
    pub operator: AssignedOperator,
    pub to_room_type: String,
    pub to_team_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ShiftRotationDto {
    pub shifts: Vec<ShiftDto>,
    pub sustained: Vec<AssignedOperator>,
    pub parked: Vec<AssignedOperator>,
    pub sustainability: SustainabilityDto,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct SustainabilityDto {
    pub verdict: String,
    pub horizon_hours: f64,
    pub depleted: Vec<DepletedOperatorDto>,
    pub dorm_overflow: usize,
    pub timeline: Vec<MoraleTimelineDto>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub facilities: Vec<FacilityOutputDto>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct FacilityOutputDto {
    pub slot_id: String,
    pub room_type: String,
    pub formula_type: Option<String>,
    pub lmd: f64,
    pub gold: f64,
    pub exp: f64,
    pub idle_hours: f64,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct MoraleTimelineDto {
    pub operator: AssignedOperator,
    pub room_type: String,
    pub slot_id: String,
    pub samples: Vec<f64>,
    pub end: f64,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DepletedOperatorDto {
    pub operator: AssignedOperator,
    pub at_hours: f64,
    pub room_type: String,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ShiftDto {
    pub index: usize,
    pub rooms: Vec<ShiftRoomDto>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ShiftRoomDto {
    pub slot_id: String,
    pub room_type: String,
    pub formula_type: Option<String>,
    pub active: bool,
    pub recommended: Vec<AssignedOperator>,
    pub current: Vec<AssignedOperator>,
    pub swap_in: Vec<AssignedOperator>,
    pub swap_out: Vec<AssignedOperator>,
    pub moved_out: Vec<MovedOperator>,
    pub matches: bool,
    pub equivalent: bool,
    pub gap_pct: Option<f64>,
    pub efficiency: Option<f64>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ledger: Vec<SkillLineDto>,
    pub team_id: Option<String>,
    pub team_label: Option<String>,
}

#[must_use]
pub fn assigned_operator(id: &str, data: &BaseData) -> AssignedOperator {
    AssignedOperator {
        operator_id: id.to_owned(),
        name: operator_name(id, data),
        bench: false,
        parked: false,
    }
}

#[must_use]
pub fn operator_name(id: &str, data: &BaseData) -> String {
    data.operators()
        .get(id)
        .map_or_else(|| id.to_owned(), |o| o.name.clone())
}

fn operators(ids: &[String], data: &BaseData) -> Vec<AssignedOperator> {
    ids.iter().map(|id| assigned_operator(id, data)).collect()
}

fn skill_line_dto(l: &LedgerLine, data: &BaseData) -> SkillLineDto {
    SkillLineDto {
        operator_id: l.operator_id.clone(),
        operator_name: operator_name(&l.operator_id, data),
        buff_id: l.buff_id.clone(),
        buff_name: data
            .building()
            .buffs
            .get(&l.buff_id)
            .map_or_else(String::new, |b| b.buff_name.clone()),
        speed_pct: l.speed_pct,
        value_pct: l.value_pct,
        from_control_center: l.from_control_center,
        disposition: l.disposition,
        note: l.note.clone(),
    }
}

pub fn base_assignment_to_dto(
    asn: &BaseAssignment,
    data: &BaseData,
    profiles: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> BaseAssignmentDto {
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
    let bench: HashSet<&str> = asn.bench.iter().map(String::as_str).collect();
    let drones = flows.drones();
    BaseAssignmentDto {
        rooms: asn
            .rooms
            .iter()
            .map(|r| room_assignment_to_dto(r, data, profiles, registry, &bench, &asn.rooms))
            .collect(),
        total_production_efficiency: asn.total_production_efficiency,
        yield_lmd_per_day: flows.lmd_per_day(),
        yield_exp_per_day: flows.exp_per_day(),
        yield_total_value: flows.value_with_drones(),
        drones_per_day: drones.per_day,
        drone_lmd_per_day: drones.lmd,
        drone_exp_per_day: drones.exp,
    }
}

fn room_assignment_to_dto(
    room: &RoomAssignment,
    data: &BaseData,
    profiles: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
    bench: &HashSet<&str>,
    sibling_rooms: &[RoomAssignment],
) -> RoomAssignmentDto {
    let y = room_yield(
        &room.room_type,
        room.formula_type.as_deref(),
        room.level,
        room.total_efficiency,
        room.order_value,
        room.operators.len(),
        room.order_limit,
    );
    let non_production = if room.room_type == "CONTROL" {
        cc_non_production_effects(&room.operators, profiles, registry, sibling_rooms)
            .into_iter()
            .map(|(room_type, value)| NonProdEffectDto { room_type, value })
            .collect()
    } else {
        Vec::new()
    };
    RoomAssignmentDto {
        slot_id: room.slot_id.clone(),
        room_type: room.room_type.clone(),
        level: room.level,
        formula_type: room.formula_type.clone(),
        total_efficiency: room.total_efficiency,
        order_value: room.order_value,
        locked: room.locked,
        operators: room
            .operators
            .iter()
            .map(|id| AssignedOperator {
                bench: bench.contains(id.as_str()),
                ..assigned_operator(id, data)
            })
            .collect(),
        yield_lmd_per_day: y.lmd_per_day,
        yield_gold_per_day: y.gold_per_day,
        yield_exp_per_day: y.exp_per_day,
        non_production,
        ledger: room
            .ledger
            .iter()
            .map(|l| skill_line_dto(l, data))
            .collect(),
        capacity: room.fill.as_ref().map(|f| f.capacity),
        fill_hours: room.fill.as_ref().map(|f| f.fill_hours),
    }
}

fn sustainability_dto(
    sim: &SustainabilityReport,
    data: &BaseData,
    room_type_of: impl Fn(&str) -> String,
    with_timeline: bool,
) -> SustainabilityDto {
    let mut timeline: Vec<MoraleTimelineDto> = if with_timeline {
        sim.timeline
            .iter()
            .map(|t| MoraleTimelineDto {
                operator: assigned_operator(&t.char_id, data),
                room_type: room_type_of(&t.home_slot_id),
                slot_id: t.home_slot_id.clone(),
                end: t.samples.last().copied().unwrap_or(0.0),
                samples: t.samples.clone(),
            })
            .collect()
    } else {
        Vec::new()
    };
    timeline.sort_by(|a, b| {
        a.end
            .total_cmp(&b.end)
            .then_with(|| a.operator.name.cmp(&b.operator.name))
    });
    SustainabilityDto {
        verdict: match sim.verdict {
            Verdict::HoldsUp => "holds_up",
            Verdict::Depletes => "depletes",
        }
        .to_owned(),
        horizon_hours: sim.horizon_hours,
        depleted: sim
            .depleted
            .iter()
            .map(|d| DepletedOperatorDto {
                operator: assigned_operator(&d.char_id, data),
                at_hours: d.at_hours,
                room_type: room_type_of(&d.slot_id),
            })
            .collect(),
        dorm_overflow: sim.dorm_overflow,
        timeline,
        facilities: sim
            .facilities
            .iter()
            .map(|f| FacilityOutputDto {
                slot_id: f.slot_id.clone(),
                room_type: f.room_type.clone(),
                formula_type: f.formula_type.clone(),
                lmd: f.lmd,
                gold: f.gold,
                exp: f.exp,
                idle_hours: f.idle_hours,
            })
            .collect(),
    }
}

fn simulate(
    rotation: &ShiftRotation,
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    data: &BaseData,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    initial_morale: Option<&HashMap<String, f64>>,
) -> SustainabilityReport {
    let targeted = targeted_morale_effects(
        &data.building().buffs,
        &build_name_to_char(data.operators()),
    );
    simulate_rotation_from(
        rotation,
        profiles,
        building,
        data.building(),
        registry,
        morale_drains,
        &targeted,
        initial_morale,
    )
}

#[must_use]
pub fn static_sustainability(
    building: &UserBuilding,
    assignment: &BaseAssignment,
    profiles: &[OperatorBaseProfile],
    data: &BaseData,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    initial_morale: Option<&HashMap<String, f64>>,
) -> Option<SustainabilityDto> {
    let efficiency_of: HashMap<&str, f64> = assignment
        .rooms
        .iter()
        .map(|r| (r.slot_id.as_str(), r.total_efficiency))
        .collect();
    let cells: Vec<ShiftRoom> = building
        .rooms
        .iter()
        .filter(|r| !r.current_operators.is_empty())
        .map(|r| ShiftRoom {
            slot_id: r.slot_id.clone(),
            room_type: r.room_type.clone(),
            formula_type: r.current_formula.clone(),
            recommended: r.current_operators.clone(),
            current: Vec::new(),
            active: true,
            efficiency: efficiency_of.get(r.slot_id.as_str()).copied(),
            team_id: None,
            team_label: None,
        })
        .collect();
    if cells.is_empty() {
        return None;
    }
    let rotation = ShiftRotation {
        shifts: (1..=3)
            .map(|index| Shift {
                index,
                rooms: cells.clone(),
            })
            .collect(),
        sustained: Vec::new(),
        bench: Vec::new(),
        parked: Vec::new(),
    };
    let sim = simulate(
        &rotation,
        profiles,
        building,
        data,
        registry,
        morale_drains,
        initial_morale,
    );
    let room_type_of = |slot_id: &str| {
        building
            .rooms
            .iter()
            .find(|r| r.slot_id == slot_id)
            .map_or_else(|| slot_id.to_owned(), |r| r.room_type.clone())
    };
    Some(sustainability_dto(&sim, data, room_type_of, false))
}

fn match_current_teams(rotation: &ShiftRotation) -> HashMap<(usize, String), Vec<String>> {
    struct Cell<'a> {
        key: (usize, String),
        set: HashSet<&'a str>,
        ops: &'a [String],
    }
    type Group = (usize, String, Option<String>);
    let mut recommended: HashMap<Group, Vec<Cell<'_>>> = HashMap::new();
    let mut current: HashMap<Group, Vec<Cell<'_>>> = HashMap::new();
    for shift in &rotation.shifts {
        for room in &shift.rooms {
            let key = (shift.index, room.slot_id.clone());
            let group = (
                shift.index,
                room.room_type.clone(),
                room.formula_type.clone(),
            );
            recommended.entry(group.clone()).or_default().push(Cell {
                key: key.clone(),
                set: room.recommended.iter().map(String::as_str).collect(),
                ops: &room.recommended,
            });
            if !room.current.is_empty() {
                current.entry(group).or_default().push(Cell {
                    key,
                    set: room.current.iter().map(String::as_str).collect(),
                    ops: &room.current,
                });
            }
        }
    }

    let mut out = HashMap::new();
    for (group, rec_cells) in &recommended {
        let cur_cells = current.get(group).map_or(&[][..], Vec::as_slice);
        let mut pairs: Vec<(usize, usize, usize)> = Vec::new();
        for (i, r) in rec_cells.iter().enumerate() {
            for (j, c) in cur_cells.iter().enumerate() {
                let inter = r.set.intersection(&c.set).count();
                pairs.push((r.set.len() + c.set.len() - 2 * inter, i, j));
            }
        }
        pairs.sort_by_key(|p| p.0);
        let mut rec_used = vec![false; rec_cells.len()];
        let mut cur_used = vec![false; cur_cells.len()];
        for (_, i, j) in pairs {
            if rec_used[i] || cur_used[j] {
                continue;
            }
            rec_used[i] = true;
            cur_used[j] = true;
            out.insert(rec_cells[i].key.clone(), cur_cells[j].ops.to_vec());
        }
        for (cell, used) in rec_cells.iter().zip(rec_used) {
            if !used {
                out.insert(cell.key.clone(), Vec::new());
            }
        }
    }
    out
}

fn seats<'a>(rooms: impl Iterator<Item = &'a ShiftRoom>) -> HashMap<String, String> {
    rooms
        .filter(|r| r.active)
        .flat_map(|r| {
            r.recommended
                .iter()
                .map(move |op| (op.clone(), r.room_type.clone()))
        })
        .collect()
}

pub fn shift_rotation_to_dto(
    rotation: &ShiftRotation,
    data: &BaseData,
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> ShiftRotationDto {
    let building_data = data.building();
    let bench_ids: HashSet<&str> = rotation.bench.iter().map(String::as_str).collect();
    let parked_ids: HashSet<&str> = rotation.parked.iter().map(String::as_str).collect();
    let parked_set: HashSet<String> = rotation.parked.iter().cloned().collect();
    let matched = match_current_teams(rotation);
    let mut rec_shifts: HashMap<&str, HashSet<usize>> = HashMap::new();
    for shift in &rotation.shifts {
        for room in shift.rooms.iter().filter(|r| r.active) {
            for id in &room.recommended {
                rec_shifts
                    .entry(id.as_str())
                    .or_default()
                    .insert(shift.index);
            }
        }
    }
    let op_index = build_op_index(profiles);
    let rotation_seats = seats(rotation.shifts.iter().flat_map(|s| s.rooms.iter()));
    let layout_registry = resolve_layout_branches(
        registry,
        building,
        building_data,
        &effective_facility_counts(building, profiles, registry, building_data, &rotation_seats),
    );

    let mut shift_dtos = Vec::with_capacity(rotation.shifts.len());
    for shift in &rotation.shifts {
        let shift_seats = seats(shift.rooms.iter());
        let working: HashSet<String> = shift_seats
            .iter()
            .filter(|(_, rt)| *rt != "DORMITORY")
            .map(|(op, _)| op.clone())
            .collect();
        let shift_registry = resolve_room_presence(
            &resolve_base_wide(
                &layout_registry,
                &working,
                &shift_seats.keys().cloned().collect(),
            ),
            &shift_seats,
            profiles,
        );
        let facility_counts = effective_facility_counts(
            building,
            profiles,
            &shift_registry,
            building_data,
            &shift_seats,
        );
        let ledger_ctx = LedgerCtx {
            op_index: &op_index,
            registry: &shift_registry,
            building_data,
            facility_counts: &facility_counts,
            total_dorm_levels: building.total_dorm_levels(),
            morale_drains,
        };
        let shift_cc: Vec<String> = shift
            .rooms
            .iter()
            .find(|r| r.room_type == "CONTROL" && r.active)
            .map(|r| r.recommended.clone())
            .unwrap_or_default();
        let (shift_globals, shift_conditions) = grants_of(&ledger_ctx, &shift_cc);
        let rec_owner: HashMap<&str, &str> = shift
            .rooms
            .iter()
            .filter(|r| r.active)
            .flat_map(|r| {
                r.recommended
                    .iter()
                    .map(move |id| (id.as_str(), r.slot_id.as_str()))
            })
            .collect();
        let elsewhere = |id: &str, slot: &str| rec_owner.get(id).is_some_and(|o| *o != slot);
        let mut kept: HashSet<String> = HashSet::new();

        let mut room_dtos = Vec::with_capacity(shift.rooms.len());
        for room in &shift.rooms {
            let current = matched
                .get(&(shift.index, room.slot_id.clone()))
                .cloned()
                .unwrap_or_default();
            let rec: HashSet<&str> = room.recommended.iter().map(String::as_str).collect();
            let cur: HashSet<&str> = current.iter().map(String::as_str).collect();
            let mut swap_in: Vec<String> = room
                .recommended
                .iter()
                .filter(|id| !cur.contains(id.as_str()))
                .cloned()
                .collect();
            let mut swap_out: Vec<String> = current
                .iter()
                .filter(|id| !rec.contains(id.as_str()) && !elsewhere(id, &room.slot_id))
                .cloned()
                .collect();
            let mut moved_out: Vec<MovedOperator> = current
                .iter()
                .filter(|id| !rec.contains(id.as_str()) && elsewhere(id, &room.slot_id))
                .filter_map(|id| {
                    let owner = rec_owner.get(id.as_str())?;
                    let dest = shift.rooms.iter().find(|r| r.slot_id == *owner)?;
                    Some(MovedOperator {
                        operator: assigned_operator(id, data),
                        to_room_type: dest.room_type.clone(),
                        to_team_label: dest.team_label.clone(),
                    })
                })
                .collect();
            let mut matches = !current.is_empty()
                && swap_in.is_empty()
                && swap_out.is_empty()
                && moved_out.is_empty();

            let mut equivalent = false;
            let mut gap_pct = None;
            if !current.is_empty()
                && matches!(room.room_type.as_str(), "MANUFACTURE" | "TRADING" | "POWER")
            {
                let value_of = |team: &[String]| {
                    team_value(
                        team,
                        &room.room_type,
                        room.formula_type.as_deref(),
                        profiles,
                        building,
                        building_data,
                        registry,
                        morale_drains,
                    )
                };
                let rec_v = value_of(&room.recommended);
                let cur_v = value_of(&current);
                if rec_v > 0.0 {
                    gap_pct = Some((cur_v - rec_v) / rec_v * 100.0);
                }
                let conflicts = current.iter().any(|id| {
                    elsewhere(id, &room.slot_id)
                        || kept.contains(id)
                        || rec_shifts
                            .get(id.as_str())
                            .is_some_and(|s| !s.contains(&shift.index))
                });
                if !matches && cur_v >= rec_v * (1.0 - EQUIVALENT_LENIENCY) && !conflicts {
                    equivalent = true;
                    matches = true;
                    swap_in.clear();
                    swap_out.clear();
                    moved_out.clear();
                }
            }
            if room.active {
                kept.extend(
                    if matches { &current } else { &room.recommended }
                        .iter()
                        .cloned(),
                );
            }

            let ledger = if !room.active {
                Vec::new()
            } else if room.room_type == "CONTROL" {
                let team_rooms: Vec<RoomAssignment> = shift
                    .rooms
                    .iter()
                    .filter(|r| r.active && is_production_room(&r.room_type))
                    .map(|r| RoomAssignment {
                        room_type: r.room_type.clone(),
                        formula_type: r.formula_type.clone(),
                        operators: r.recommended.clone(),
                        ..Default::default()
                    })
                    .collect();
                control_room_ledger(&ledger_ctx, &room.recommended, &team_rooms)
            } else if is_production_room(&room.room_type) {
                production_room_ledger(
                    &ledger_ctx,
                    &working_crew(&room.recommended, &parked_set),
                    &room.room_type,
                    room.formula_type.as_deref(),
                    &shift_cc,
                    &shift_globals,
                    &shift_conditions,
                )
            } else {
                Vec::new()
            };
            room_dtos.push(ShiftRoomDto {
                slot_id: room.slot_id.clone(),
                room_type: room.room_type.clone(),
                formula_type: room.formula_type.clone(),
                active: room.active,
                ledger: ledger.iter().map(|l| skill_line_dto(l, data)).collect(),
                recommended: room
                    .recommended
                    .iter()
                    .map(|id| AssignedOperator {
                        bench: bench_ids.contains(id.as_str()),
                        parked: parked_ids.contains(id.as_str()),
                        ..assigned_operator(id, data)
                    })
                    .collect(),
                current: operators(&current, data),
                matches,
                equivalent,
                gap_pct,
                efficiency: room.efficiency,
                team_id: room.team_id.clone(),
                team_label: room.team_label.clone(),
                swap_in: operators(&swap_in, data),
                swap_out: operators(&swap_out, data),
                moved_out,
            });
        }
        shift_dtos.push(ShiftDto {
            index: shift.index,
            rooms: room_dtos,
        });
    }
    let sim = simulate(
        rotation,
        profiles,
        building,
        data,
        registry,
        morale_drains,
        None,
    );
    let room_type_of = |slot_id: &str| {
        rotation
            .shifts
            .iter()
            .flat_map(|s| s.rooms.iter())
            .find(|r| r.slot_id == slot_id)
            .map_or_else(|| slot_id.to_owned(), |r| r.room_type.clone())
    };
    ShiftRotationDto {
        shifts: shift_dtos,
        sustained: operators(&rotation.sustained, data),
        parked: operators(&rotation.parked, data),
        sustainability: sustainability_dto(&sim, data, room_type_of, true),
    }
}
