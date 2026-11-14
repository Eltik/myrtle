//! Recommended 3-shift base rotation, to compare against the player's presets

use std::collections::{HashMap, HashSet};

use myrtle_gamedata_raw::tables::building::BuildingDataFile;

use crate::types::UserRoom;
use crate::{
    assignment::{
        assign_auxiliary_rooms, base_wide_relevant, build_op_index, compute_team_efficiency,
        compute_team_totals, effective_facility_counts, fill_remaining_slots, morale_recovery,
        num_morale_swap_managers, op_uptime, resolve_base_wide, resolve_room_presence,
        room_presence_relevant, room_search_score, rotation_cc_plan,
    },
    registry::BuffResolutionStrategy,
    rotation::sustain_sim::SHIFT_HOURS,
    rotation::team_select::{PlannedGroup, Tiling, plan_production_groups, tiled_objective},
    scoring::clause::TENURE_HOURS,
    types::{OperatorBaseProfile, UserBuilding},
    util::{is_production_room, max_stationed_at_level},
};

pub const SHIFT_COUNT: usize = 3;

/// `SHIFT_COUNT` shifts of recommended crews beside the player's presets
pub struct ShiftRotation {
    pub shifts: Vec<Shift>,
    /// Held 24/7 by a morale-swap manager
    pub sustained: Vec<String>,
    /// Operators on spare CC seats
    pub bench: Vec<String>,
    /// Zero-morale tokens
    pub parked: Vec<String>,
}

pub struct Shift {
    /// 1-indexed shift number
    pub index: usize,
    pub rooms: Vec<ShiftRoom>,
}

#[derive(Clone)]
pub struct ShiftRoom {
    pub slot_id: String,
    pub room_type: String,
    pub formula_type: Option<String>,
    pub recommended: Vec<String>,
    /// The player's preset for this room and shift
    pub current: Vec<String>,
    /// False when deliberately unstaffed
    pub active: bool,
    pub efficiency: Option<f64>,
    pub team_id: Option<String>,
    pub team_label: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum SquadPattern {
    /// Squad 1 works shifts 1 & 3, Squad 2 shift 2
    Alternating,
    /// Squad 1 works shifts 1+2, Squad 2 shift 3
    Block,
}

impl SquadPattern {
    /// Which squad (0/1) staffs shift `k`
    fn squad_at(self, k: usize) -> usize {
        match self {
            Self::Alternating => k % 2,
            Self::Block => usize::from(k >= 2),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn seat_pinned_operator(
    groups: &mut [PlannedGroup],
    slot: &str,
    pinned: &str,
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    morale_drains: &HashMap<String, f64>,
    cc_conditions: &[crate::assignment::CcCondition],
) {
    for g in groups.iter_mut() {
        let Some(ri) = g.rooms.iter().position(|(s, _)| s == slot) else {
            continue;
        };
        for shift in 0..SHIFT_COUNT {
            let Some(team) = g.teams.get(g.cells[ri][shift]) else {
                continue;
            };
            let mut ops = team.ops.clone();
            if !ops.iter().any(|o| o == pinned) {
                ops.push(pinned.to_string());
            }
            let totals = compute_team_totals(
                &ops,
                &g.room_type,
                g.formula_type.as_deref(),
                Some(g.rooms[ri].1),
                op_index,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
                morale_drains,
                cc_conditions,
            );
            let score = room_search_score(&g.room_type, totals.speed_pct, totals.order_value_pct);
            g.teams.push(crate::assignment::CandidateTeam {
                ops,
                speed: totals.speed_pct,
                value: totals.order_value_pct,
                gold: totals.order_gold_pct,
                order_limit: totals.order_limit,
                score,
            });
            g.cells[ri][shift] = g.teams.len() - 1;
        }
    }
}

fn preset_sustained_operators(
    building: &UserBuilding,
    operators: &[OperatorBaseProfile],
    morale_drains: &HashMap<String, f64>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> HashSet<String> {
    let managers = num_morale_swap_managers(operators, building_data);
    if managers == 0 || !crate::rotation::dorms::building_hosts_manager(building, building_data) {
        return HashSet::new();
    }
    let swap_rate = crate::assignment::morale_swap_enabler(operators, building_data)
        .and_then(|id| operators.iter().find(|o| o.char_id == id))
        .map_or(0.0, |m| {
            crate::rotation::dorms::manager_swap_rate(m, registry, building_data)
        });
    let mut candidates: HashSet<String> = HashSet::new();
    for room in building
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
    {
        let presets: Vec<&Vec<String>> = room
            .preset_shifts
            .iter()
            .filter(|p| !p.is_empty())
            .collect();
        if presets.len() < 2 {
            continue;
        }
        for op in presets[0] {
            if presets.iter().all(|p| p.contains(op)) {
                let feasible = operators.iter().find(|o| &o.char_id == op).is_none_or(|o| {
                    crate::rotation::dorms::manager_can_sustain(
                        swap_rate,
                        crate::rotation::sustain_sim::game_morale_drain(o, morale_drains),
                    )
                });
                if feasible {
                    candidates.insert(op.clone());
                }
            }
        }
    }
    let recovery = morale_recovery(building);
    let op_index: HashMap<&str, &OperatorBaseProfile> =
        operators.iter().map(|o| (o.char_id.as_str(), o)).collect();
    let mut ranked: Vec<String> = candidates.into_iter().collect();
    ranked.sort_by(|a, b| {
        let ua = op_index
            .get(a.as_str())
            .map_or(1.0, |o| op_uptime(o, morale_drains, recovery));
        let ub = op_index
            .get(b.as_str())
            .map_or(1.0, |o| op_uptime(o, morale_drains, recovery));
        ua.partial_cmp(&ub)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.cmp(b))
    });
    ranked.truncate(managers);
    ranked.into_iter().collect()
}

/// Minimum-cost perfect assignment of rows to columns
fn assignment(cost: &[Vec<i64>]) -> Vec<usize> {
    let n = cost.len();
    if n == 0 {
        return Vec::new();
    }
    let mut u = vec![0i64; n + 1];
    let mut v = vec![0i64; n + 1];
    let mut p = vec![0usize; n + 1];
    let mut way = vec![0usize; n + 1];
    for i in 1..=n {
        p[0] = i;
        let mut j0 = 0usize;
        let mut minv = vec![i64::MAX; n + 1];
        let mut used = vec![false; n + 1];
        loop {
            used[j0] = true;
            let i0 = p[j0];
            let mut delta = i64::MAX;
            let mut j1 = 0usize;
            for j in 1..=n {
                if used[j] {
                    continue;
                }
                let cur = cost[i0 - 1][j - 1]
                    .saturating_sub(u[i0])
                    .saturating_sub(v[j]);
                if cur < minv[j] {
                    minv[j] = cur;
                    way[j] = j0;
                }
                if minv[j] < delta {
                    delta = minv[j];
                    j1 = j;
                }
            }
            for j in 0..=n {
                if used[j] {
                    u[p[j]] = u[p[j]].saturating_add(delta);
                    v[j] = v[j].saturating_sub(delta);
                } else {
                    minv[j] = minv[j].saturating_sub(delta);
                }
            }
            j0 = j1;
            if p[j0] == 0 {
                break;
            }
        }
        loop {
            let j1 = way[j0];
            p[j0] = p[j1];
            j0 = j1;
            if j0 == 0 {
                break;
            }
        }
    }
    let mut out = vec![0usize; n];
    for j in 1..=n {
        if p[j] != 0 {
            out[p[j] - 1] = j - 1;
        }
    }
    out
}

fn permutations(n: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let mut current: Vec<usize> = Vec::with_capacity(n);
    let mut used = vec![false; n];
    fn rec(n: usize, current: &mut Vec<usize>, used: &mut [bool], out: &mut Vec<Vec<usize>>) {
        if current.len() == n {
            out.push(current.clone());
            return;
        }
        for i in 0..n {
            if !used[i] {
                used[i] = true;
                current.push(i);
                rec(n, current, used, out);
                current.pop();
                used[i] = false;
            }
        }
    }
    rec(n, &mut current, &mut used, &mut out);
    out
}

fn small_subsets(pool: &[String], size: usize) -> Vec<Vec<String>> {
    if size == 0 {
        return vec![Vec::new()];
    }
    if pool.len() < size {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut idx: Vec<usize> = (0..size).collect();
    loop {
        out.push(idx.iter().map(|&i| pool[i].clone()).collect());
        let mut i = size;
        loop {
            if i == 0 {
                return out;
            }
            i -= 1;
            if idx[i] != i + pool.len() - size {
                break;
            }
        }
        idx[i] += 1;
        for j in (i + 1)..size {
            idx[j] = idx[j - 1] + 1;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn merge_kept(
    kept: &[String],
    team: &[String],
    capacity: usize,
    room_type: &str,
    formula_type: Option<&str>,
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    morale_drains: &HashMap<String, f64>,
) -> Vec<String> {
    let mut crew: Vec<String> = kept.to_vec();
    crew.dedup();
    let members: Vec<String> = team
        .iter()
        .filter(|op| !crew.contains(op))
        .cloned()
        .collect();
    let free = capacity.saturating_sub(crew.len());
    if crew.is_empty() || members.len() <= free {
        for op in members.into_iter().take(free) {
            crew.push(op);
        }
        return crew;
    }
    let mut best: Option<(f64, Vec<String>)> = None;
    for subset in small_subsets(&members, free) {
        let mut trial = crew.clone();
        trial.extend(subset);
        let (speed, value) = compute_team_efficiency(
            &trial,
            room_type,
            formula_type,
            None,
            op_index,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
            morale_drains,
            &[],
        );
        let score = room_search_score(room_type, speed, value);
        if best.as_ref().is_none_or(|(b, _)| score > *b) {
            best = Some((score, trial));
        }
    }
    best.map_or(crew, |(_, trial)| trial)
}

/// Build the recommended rotation
#[must_use]
pub fn recommend_shift_rotation(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    pins: &[(String, String)],
) -> ShiftRotation {
    let pinned_seats: HashMap<String, String> = pins.iter().cloned().collect();
    if !base_wide_relevant(operators, registry) && !room_presence_relevant(operators, registry) {
        return rotation_core(
            operators,
            building,
            building_data,
            registry,
            morale_drains,
            pins,
            &pinned_seats,
        );
    }
    let layout_registry = crate::assignment::resolve_layout_branches(
        registry,
        building,
        building_data,
        &effective_facility_counts(building, operators, registry, building_data, &pinned_seats),
    );
    let registry = &layout_registry;
    let pass1_registry = resolve_room_presence(
        &resolve_base_wide(registry, &HashSet::new(), &HashSet::new()),
        &HashMap::new(),
        operators,
    );
    let pass1 = rotation_core(
        operators,
        building,
        building_data,
        &pass1_registry,
        morale_drains,
        pins,
        &pinned_seats,
    );
    let deployed: HashSet<String> = pass1
        .shifts
        .iter()
        .flat_map(|s| s.rooms.iter().filter(|r| r.active))
        .flat_map(|r| r.recommended.iter().cloned())
        .collect();
    let deployed_rooms: HashMap<String, String> = pass1
        .shifts
        .iter()
        .flat_map(|s| s.rooms.iter().filter(|r| r.active))
        .flat_map(|r| {
            r.recommended
                .iter()
                .map(|op| (op.clone(), r.room_type.clone()))
        })
        .collect();
    let pass2_registry = resolve_room_presence(
        &resolve_base_wide(
            registry,
            &deployed,
            &deployed
                .iter()
                .cloned()
                .chain(crate::assignment::dorm_residents(building))
                .collect(),
        ),
        &deployed_rooms,
        operators,
    );
    let mut pass2_seats = deployed_rooms;
    pass2_seats.extend(pins.iter().cloned());
    let counts_unchanged =
        effective_facility_counts(building, operators, registry, building_data, &pass2_seats)
            == effective_facility_counts(
                building,
                operators,
                registry,
                building_data,
                &pinned_seats,
            );
    if pass2_registry == pass1_registry && counts_unchanged {
        return pass1;
    }
    rotation_core(
        operators,
        building,
        building_data,
        &pass2_registry,
        morale_drains,
        pins,
        &pass2_seats,
    )
}

fn rotation_core(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    pins: &[(String, String)],
    seats: &HashMap<String, String>,
) -> ShiftRotation {
    rotation_core_tiled(
        operators,
        building,
        building_data,
        registry,
        morale_drains,
        pins,
        seats,
        Tiling::Shifts,
    )
}

#[allow(clippy::too_many_arguments)]
fn rotation_core_tiled(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    pins: &[(String, String)],
    seats: &HashMap<String, String>,
    tiling: Tiling,
) -> ShiftRotation {
    let production_seats: usize = building
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
        .map(|r| max_stationed_at_level(building_data, &r.room_type, r.level).max(0) as usize)
        .sum();
    let beds: usize = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "DORMITORY")
        .map(|r| max_stationed_at_level(building_data, "DORMITORY", r.level).max(0) as usize)
        .sum();
    if tiling == Tiling::Shifts && production_seats > beds {
        return rotation_core_tiled(
            operators,
            building,
            building_data,
            registry,
            morale_drains,
            pins,
            seats,
            Tiling::Blocks,
        );
    }
    let mut facility_counts =
        effective_facility_counts(building, operators, registry, building_data, seats);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let tenure_hours = match tiling {
        Tiling::Shifts => SHIFT_HOURS as usize,
        Tiling::Blocks => (2.0 * SHIFT_HOURS) as usize,
    };
    facility_counts.insert(TENURE_HOURS.to_string(), tenure_hours);
    let parked = crate::assignment::parked_tokens(seats, operators, registry);
    let total_dorm_levels = building.total_dorm_levels();
    let production_rooms: Vec<&UserRoom> = building
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
        .collect();

    let roster_ids: HashSet<&str> = operators.iter().map(|o| o.char_id.as_str()).collect();
    let mut pinned_ids: HashSet<String> = pins
        .iter()
        .filter(|(id, _)| roster_ids.contains(id.as_str()))
        .map(|(id, _)| id.clone())
        .collect();
    let cc_pinned: Vec<String> = pins
        .iter()
        .filter(|(id, rt)| rt == "CONTROL" && pinned_ids.contains(id))
        .map(|(id, _)| id.clone())
        .collect();

    let manager_resident: Option<String> =
        if crate::rotation::dorms::building_hosts_manager(building, building_data) {
            crate::assignment::morale_swap_enabler(operators, building_data)
        } else {
            None
        };
    if let Some(m) = &manager_resident {
        pinned_ids.insert(m.clone());
    }
    pinned_ids.extend(parked.iter().cloned());

    let memo = crate::rotation::team_select::EnumerationMemo::default();
    let (cc_plan, mut groups) = rotation_cc_plan(
        operators,
        building,
        building_data,
        registry,
        &cc_pinned,
        |assigned, global_bonuses, cc_conditions| {
            let mut assigned = assigned.clone();
            assigned.extend(pinned_ids.iter().cloned());
            plan_production_groups(
                &production_rooms,
                operators,
                &assigned,
                registry,
                building_data,
                &facility_counts,
                total_dorm_levels,
                global_bonuses,
                cc_conditions,
                morale_drains,
                &HashMap::new(),
                &HashMap::new(),
                &memo,
                tiling,
            )
        },
    );

    let mut sustained =
        preset_sustained_operators(building, operators, morale_drains, registry, building_data);
    let managers = num_morale_swap_managers(operators, building_data);
    let op_index = build_op_index(operators);
    let mut oracle_slot: HashMap<String, String> = HashMap::new();
    if sustained.len() < managers
        && crate::rotation::dorms::building_hosts_manager(building, building_data)
    {
        let recovery = morale_recovery(building);
        let swap_rate = crate::assignment::morale_swap_enabler(operators, building_data)
            .and_then(|id| operators.iter().find(|o| o.char_id == id))
            .map_or(0.0, |m| {
                crate::rotation::dorms::manager_swap_rate(m, registry, building_data)
            });
        let mut exclude_base: HashSet<String> = cc_plan.squad1.iter().cloned().collect();
        exclude_base.extend(pinned_ids.iter().cloned());
        let mut cands: Vec<(String, f64)> = Vec::new();
        let trading_slots: Vec<String> = groups
            .iter()
            .filter(|g| g.room_type == "TRADING")
            .flat_map(|g| g.rooms.iter().map(|(s, _)| s.clone()))
            .collect();
        for g in groups.iter().filter(|g| g.room_type == "TRADING") {
            let group_rooms: Vec<String> = g.rooms.iter().map(|(s, _)| s.clone()).collect();
            for team in g.teams.iter().filter(|t| !t.ops.is_empty()) {
                for id in &team.ops {
                    if sustained.contains(id) || cands.iter().any(|(c, _)| c == id) {
                        continue;
                    }
                    let feasible = op_index.get(id.as_str()).is_none_or(|o| {
                        crate::rotation::dorms::manager_can_sustain(
                            swap_rate,
                            crate::rotation::sustain_sim::game_morale_drain(o, morale_drains),
                        )
                    });
                    if !feasible {
                        continue;
                    }
                    let slots: Vec<String> = match preset_room_of(building, id)
                        .filter(|slot| trading_slots.contains(slot))
                    {
                        Some(slot) => vec![slot],
                        None => trading_slots.clone(),
                    };
                    let _ = &group_rooms;
                    let uptime = op_index
                        .get(id.as_str())
                        .map_or(1.0, |op| op_uptime(op, morale_drains, recovery));
                    let extra_uptime = 1.0 - uptime * (2.0 / 3.0);
                    let mut best: Option<(String, f64)> = None;
                    for slot in slots {
                        let mut exclude = exclude_base.clone();
                        exclude.insert(id.clone());
                        let reserved: HashMap<String, usize> = HashMap::from([(slot.clone(), 1)]);
                        let mut pinned_plan = plan_production_groups(
                            &production_rooms,
                            operators,
                            &exclude,
                            registry,
                            building_data,
                            &facility_counts,
                            total_dorm_levels,
                            &cc_plan.global_bonuses,
                            &cc_plan.conditions,
                            morale_drains,
                            &reserved,
                            &HashMap::new(),
                            &memo,
                            tiling,
                        );
                        let unseated = tiled_objective(&pinned_plan, &cc_plan.global_bonuses);
                        seat_pinned_operator(
                            &mut pinned_plan,
                            &slot,
                            id,
                            &op_index,
                            registry,
                            building_data,
                            &facility_counts,
                            total_dorm_levels,
                            morale_drains,
                            &cc_plan.conditions,
                        );
                        let seated = tiled_objective(&pinned_plan, &cc_plan.global_bonuses);
                        let gain = (seated - unseated).max(0.0) * extra_uptime;
                        if best.as_ref().is_none_or(|(_, g)| gain > *g + 1e-9) {
                            best = Some((slot, gain));
                        }
                    }
                    if let Some((slot, gain)) = best {
                        oracle_slot.insert(id.clone(), slot);
                        cands.push((id.clone(), gain));
                    }
                }
            }
        }
        cands.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        for (id, gain) in cands {
            if sustained.len() >= managers {
                break;
            }
            if gain > 1e-9 && !sustained.contains(&id) {
                sustained.insert(id);
            }
        }
    }
    let mut kept_by_room: HashMap<String, Vec<String>> = HashMap::new();
    for g in &groups {
        let group_rooms: Vec<String> = g.rooms.iter().map(|(s, _)| s.clone()).collect();
        for team in &g.teams {
            for op in team.ops.iter().filter(|op| sustained.contains(*op)) {
                let same_kind = |slot: &String| {
                    group_rooms.contains(slot)
                        || (g.room_type == "TRADING"
                            && groups
                                .iter()
                                .filter(|h| h.room_type == "TRADING")
                                .any(|h| h.rooms.iter().any(|(s, _)| s == slot)))
                };
                let room = preset_room_of(building, op)
                    .filter(same_kind)
                    .or_else(|| oracle_slot.get(op).cloned())
                    .or_else(|| group_rooms.first().cloned());
                if let Some(slot) = room {
                    kept_by_room.entry(slot).or_default().push(op.clone());
                }
            }
        }
    }
    let mut sustained_label: Vec<String> = kept_by_room.values().flatten().cloned().collect();
    sustained_label.sort();
    sustained_label.dedup();
    let heavy: Vec<String> = operators
        .iter()
        .filter(|_| tiling == Tiling::Blocks)
        .filter(|op| !crate::rotation::sustain_sim::sustains_24h_block(op, morale_drains))
        .filter(|op| !sustained_label.contains(&op.char_id) && !pinned_ids.contains(&op.char_id))
        .map(|op| op.char_id.clone())
        .collect();
    if !sustained_label.is_empty() || !heavy.is_empty() {
        let mut exclude: HashSet<String> = cc_plan.squad1.iter().cloned().collect();
        exclude.extend(sustained_label.iter().cloned());
        exclude.extend(pinned_ids.iter().cloned());
        exclude.extend(heavy);
        groups = plan_production_groups(
            &production_rooms,
            operators,
            &exclude,
            registry,
            building_data,
            &facility_counts,
            total_dorm_levels,
            &cc_plan.global_bonuses,
            &cc_plan.conditions,
            morale_drains,
            &HashMap::new(),
            &kept_by_room,
            &memo,
            tiling,
        );
    }

    let mut used: HashSet<String> = cc_plan.squad1.iter().cloned().collect();
    for g in &groups {
        for t in &g.teams {
            used.extend(t.ops.iter().cloned());
        }
    }
    used.extend(sustained_label.iter().cloned());
    used.extend(pinned_ids.iter().cloned());

    let aux_squads = |assigned: &mut HashSet<String>,
                      only_24h_sustainable: bool|
     -> HashMap<String, (String, Vec<String>)> {
        let mut rooms = Vec::new();
        assign_auxiliary_rooms(
            &mut rooms,
            building,
            building_data,
            operators,
            registry,
            morale_drains,
            assigned,
            only_24h_sustainable,
        );
        rooms
            .into_iter()
            .map(|r| (r.slot_id.clone(), (r.room_type, r.operators)))
            .collect()
    };
    let mut assigned = used.clone();
    let mut aux1 = aux_squads(&mut assigned, true);
    let mut aux2 = aux_squads(&mut assigned, false);

    for (id, room_type) in pins.iter().filter(|(id, _)| pinned_ids.contains(id)) {
        if !matches!(room_type.as_str(), "HIRE" | "MEETING") {
            continue;
        }
        let Some(room) = building.rooms.iter().find(|r| &r.room_type == room_type) else {
            continue;
        };
        #[allow(clippy::cast_sign_loss)]
        let capacity = max_stationed_at_level(building_data, room_type, room.level).max(1) as usize;
        for squads in [&mut aux1, &mut aux2] {
            let (_, ops) = squads
                .entry(room.slot_id.clone())
                .or_insert_with(|| (room_type.clone(), Vec::new()));
            if ops.iter().any(|o| o == id) {
                continue;
            }
            if ops.len() >= capacity {
                ops.truncate(capacity - 1);
            }
            ops.insert(0, id.clone());
        }
    }

    let team_rooms: Vec<crate::types::RoomAssignment> = groups
        .iter()
        .flat_map(|g| {
            g.teams
                .iter()
                .filter(|t| !t.ops.is_empty())
                .map(|t| crate::types::RoomAssignment {
                    room_type: g.room_type.clone(),
                    formula_type: g.formula_type.clone(),
                    operators: t.ops.clone(),
                    ..Default::default()
                })
        })
        .collect();
    let cc_squad2 = cc_plan.squad2(operators, building_data, registry, &assigned, &team_rooms);
    assigned.extend(cc_squad2.iter().cloned());

    let power_plan = build_power_plan(
        operators,
        building,
        building_data,
        registry,
        &facility_counts,
        &assigned,
        morale_drains,
        &parked,
    );
    for plant in &power_plan {
        assigned.extend(plant.main.iter().cloned());
        assigned.extend(plant.backup.iter().cloned());
    }

    let mut cc1 = cc_plan.squad1.clone();
    let mut cc2 = cc_squad2;
    #[allow(clippy::cast_sign_loss)]
    let cc_cap = cc_plan.control_slots.max(0) as usize;
    for id in cc_pinned.iter().rev() {
        if !cc1.contains(id) {
            cc1.insert(0, id.clone());
        }
    }
    if cc_cap > 0 {
        cc1.truncate(cc_cap);
    }
    cc2.retain(|id| !cc_pinned.contains(id));
    let mut bench: HashSet<String> = HashSet::new();
    if !cc1.is_empty() {
        bench.extend(fill_remaining_slots(
            &mut cc1,
            cc_plan.control_slots,
            "CONTROL",
            operators,
            building_data,
            registry,
            &mut assigned,
        ));
    }
    if !cc2.is_empty() {
        bench.extend(fill_remaining_slots(
            &mut cc2,
            cc_plan.control_slots,
            "CONTROL",
            operators,
            building_data,
            registry,
            &mut assigned,
        ));
    }

    let mut cc_pattern = SquadPattern::Alternating;
    let mut linked_groups: HashSet<usize> = HashSet::new();
    for (gi, g) in groups.iter_mut().enumerate() {
        let mut linked: Option<usize> = None;
        let mut best_weight = 0.0;
        for (ti, team) in g.teams.iter().enumerate() {
            if team.ops.is_empty() {
                continue;
            }
            let scored = |conditions: &[crate::assignment::CcCondition]| {
                compute_team_efficiency(
                    &team.ops,
                    &g.room_type,
                    g.formula_type.as_deref(),
                    None,
                    &op_index,
                    registry,
                    building_data,
                    &facility_counts,
                    total_dorm_levels,
                    morale_drains,
                    conditions,
                )
                .0
            };
            let weight = scored(&cc_plan.conditions) - scored(&[]);
            if weight > best_weight + 1e-9 {
                best_weight = weight;
                linked = Some(ti);
            }
        }
        if let Some(ti) = linked
            && best_weight > 0.0
        {
            cc_pattern = SquadPattern::Block;
            linked_groups.insert(gi);
            if ti != 0 {
                g.teams.swap(0, ti);
            }
        }
    }

    if cc_pattern == SquadPattern::Block {
        let is_pin = |id: &str| cc_pinned.iter().any(|p| p == id);
        let sustains = |id: &str| {
            op_index.get(id).is_none_or(|op| {
                crate::rotation::sustain_sim::sustains_24h_block(op, morale_drains)
            })
        };
        let mut heavy: Vec<String> = Vec::new();
        cc1.retain(|id| {
            if is_pin(id) || sustains(id) {
                true
            } else {
                heavy.push(id.clone());
                false
            }
        });
        for id in heavy {
            let best = cc2
                .iter()
                .enumerate()
                .filter(|(_, c)| !is_pin(c) && sustains(c) && !cc1.contains(*c))
                .map(|(j, c)| {
                    let gain = crate::assignment::cc_marginal_over(
                        &cc1,
                        c,
                        &op_index,
                        registry,
                        building_data,
                    );
                    let loss = crate::assignment::cc_marginal_over(
                        &cc2,
                        c,
                        &op_index,
                        registry,
                        building_data,
                    );
                    (j, gain - loss)
                })
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(j, _)| j);
            if let Some(j) = best {
                cc1.push(cc2[j].clone());
                cc2[j] = id;
            } else if !cc2.contains(&id) && (cc2.len() as i32) < cc_plan.control_slots {
                cc2.push(id);
            }
        }
    }

    let presets: HashMap<&str, Vec<&Vec<String>>> = building
        .rooms
        .iter()
        .map(|r| {
            let non_empty: Vec<&Vec<String>> =
                r.preset_shifts.iter().filter(|p| !p.is_empty()).collect();
            (r.slot_id.as_str(), non_empty)
        })
        .collect();
    let preset_for = |slot: &str, k: usize| -> Vec<String> {
        presets
            .get(slot)
            .filter(|p| !p.is_empty())
            .map(|p| p[k % p.len()].clone())
            .unwrap_or_default()
    };

    let sym_diff = |a: &[String], b: &[String]| -> usize {
        if b.is_empty() {
            return 0; // no preset saved
        }
        let sa: HashSet<&str> = a.iter().map(String::as_str).collect();
        let sb: HashSet<&str> = b.iter().map(String::as_str).collect();
        sa.symmetric_difference(&sb).count()
    };
    for (gi, g) in groups.iter_mut().enumerate() {
        let n = g.rooms.len();
        if n == 0 || g.teams.is_empty() {
            continue;
        }
        let linked = linked_groups.contains(&gi);
        let mut ordinal_cells = vec![0usize; g.teams.len()];
        for row in &g.cells {
            for &t in row {
                ordinal_cells[t] += 1;
            }
        }
        if ordinal_cells.iter().all(|&c| c == 1) && g.teams.len() == n * SHIFT_COUNT {
            let cells: Vec<(usize, usize)> = (0..n)
                .flat_map(|ri| (0..SHIFT_COUNT).map(move |k| (ri, k)))
                .collect();
            let cell_of = |ri: usize, k: usize| g.cells[ri][k];
            const CHURN_SCALE: i64 = 1_000;
            const FORBIDDEN: i64 = i64::MAX / 4;
            let mut cost = vec![vec![0i64; g.teams.len()]; cells.len()];
            for (ci, &(ri, k)) in cells.iter().enumerate() {
                let (slot, level) = &g.rooms[ri];
                let kept = kept_by_room.get(slot).map_or(&[][..], Vec::as_slice);
                let capacity =
                    max_stationed_at_level(building_data, &g.room_type, *level).max(0) as usize;
                for (t, team) in g.teams.iter().enumerate() {
                    if linked && ((ci == cell_of(0, 0)) != (t == 0)) {
                        cost[ci][t] = FORBIDDEN;
                        continue;
                    }
                    let crew = merge_kept(
                        kept,
                        &team.ops,
                        capacity,
                        &g.room_type,
                        g.formula_type.as_deref(),
                        &op_index,
                        registry,
                        building_data,
                        &facility_counts,
                        total_dorm_levels,
                        morale_drains,
                    );
                    let (speed, value) = compute_team_efficiency(
                        &crew,
                        &g.room_type,
                        g.formula_type.as_deref(),
                        Some(*level),
                        &op_index,
                        registry,
                        building_data,
                        &facility_counts,
                        total_dorm_levels,
                        morale_drains,
                        &cc_plan.conditions,
                    );
                    let output =
                        (room_search_score(&g.room_type, speed, value) * 100.0).round() as i64;
                    let churn = sym_diff(&crew, &preset_for(slot, k)) as i64;
                    cost[ci][t] = -output * CHURN_SCALE + churn;
                }
            }
            let assigned = assignment(&cost);
            let mut team_perm = vec![0usize; g.teams.len()];
            for (ci, &(ri, k)) in cells.iter().enumerate() {
                team_perm[cell_of(ri, k)] = assigned[ci];
            }
            g.teams = team_perm.iter().map(|&t| g.teams[t].clone()).collect();
            continue;
        }
        if n > 3 {
            continue;
        }
        let mut best: Option<(i64, usize, Vec<usize>, Vec<usize>)> = None;
        for room_order in permutations(n) {
            for team_perm in permutations(g.teams.len()) {
                if linked && team_perm[0] != 0 {
                    continue;
                }
                if team_perm
                    .iter()
                    .enumerate()
                    .any(|(o, &t)| ordinal_cells[o] != ordinal_cells[t])
                {
                    continue;
                }
                let mut cost = 0usize;
                let mut output = 0.0f64;
                for (ri, &old_ri) in room_order.iter().enumerate() {
                    let (slot, level) = &g.rooms[old_ri];
                    let kept = kept_by_room.get(slot).map_or(&[][..], Vec::as_slice);
                    let capacity =
                        max_stationed_at_level(building_data, &g.room_type, *level).max(0) as usize;
                    for k in 0..SHIFT_COUNT {
                        let team = &g.teams[team_perm[g.cells[ri][k]]];
                        let crew = merge_kept(
                            kept,
                            &team.ops,
                            capacity,
                            &g.room_type,
                            g.formula_type.as_deref(),
                            &op_index,
                            registry,
                            building_data,
                            &facility_counts,
                            total_dorm_levels,
                            morale_drains,
                        );
                        let (speed, value) = compute_team_efficiency(
                            &crew,
                            &g.room_type,
                            g.formula_type.as_deref(),
                            Some(*level),
                            &op_index,
                            registry,
                            building_data,
                            &facility_counts,
                            total_dorm_levels,
                            morale_drains,
                            &cc_plan.conditions,
                        );
                        output += room_search_score(&g.room_type, speed, value);
                        cost += sym_diff(&crew, &preset_for(slot, k));
                    }
                }
                let output_key = (output * 100.0).round() as i64;
                let better = match &best {
                    None => true,
                    Some((bo, bc, _, _)) => output_key > *bo || (output_key == *bo && cost < *bc),
                };
                if better {
                    best = Some((output_key, cost, room_order.clone(), team_perm));
                }
            }
        }
        if let Some((_, _, room_order, team_perm)) = best {
            g.rooms = room_order.iter().map(|&ri| g.rooms[ri].clone()).collect();
            g.teams = team_perm.iter().map(|&t| g.teams[t].clone()).collect();
        }
    }
    let flip_costs = |slot: &str, s1: &[String], s2: &[String]| -> (usize, usize) {
        let (mut keep, mut flip) = (0usize, 0usize);
        for k in 0..SHIFT_COUNT {
            let preset = preset_for(slot, k);
            let (a, b) = if SquadPattern::Alternating.squad_at(k) == 0 {
                (s1, s2)
            } else {
                (s2, s1)
            };
            keep += sym_diff(a, &preset);
            flip += sym_diff(b, &preset);
        }
        (keep, flip)
    };
    let flip_better = |slot: &str, s1: &[String], s2: &[String]| -> bool {
        if s2.is_empty() {
            return false;
        }
        let (keep, flip) = flip_costs(slot, s1, s2);
        flip < keep
    };
    let mut power_plan = power_plan;
    if power_plan.iter().all(|p| !p.backup.is_empty()) {
        let squad_total = |pick: fn(&PowerPlant) -> &Vec<String>| -> f64 {
            power_plan
                .iter()
                .flat_map(|p| pick(p).iter())
                .filter_map(|id| op_index.get(id.as_str()).copied())
                .map(|op| power_value(op, building_data, registry, &facility_counts))
                .sum()
        };
        let (main_total, backup_total) = (squad_total(|p| &p.main), squad_total(|p| &p.backup));
        let (keep, flip) = power_plan.iter().fold((0, 0), |(k, f), p| {
            let (pk, pf) = flip_costs(&p.slot_id, &p.main, &p.backup);
            (k + pk, f + pf)
        });
        let bar_ok_ids = |ids: &[String]| -> bool {
            ids.iter().all(|id| {
                op_index.get(id.as_str()).is_none_or(|op| {
                    crate::rotation::sustain_sim::sustains_24h_block(op, morale_drains)
                })
            })
        };
        if flip < keep
            && backup_total + 1e-9 >= main_total
            && power_plan.iter().all(|p| bar_ok_ids(&p.backup))
        {
            for plant in &mut power_plan {
                std::mem::swap(&mut plant.main, &mut plant.backup);
            }
        }
    }
    let bar_ok_ids = |ids: &[String]| -> bool {
        ids.iter().all(|id| {
            op_index.get(id.as_str()).is_none_or(|op| {
                crate::rotation::sustain_sim::sustains_24h_block(op, morale_drains)
            })
        })
    };
    for (slot, (_, squad1)) in &mut aux1 {
        if let Some((_, squad2)) = aux2.get_mut(slot)
            && flip_better(slot, squad1, squad2)
            && bar_ok_ids(squad2)
        {
            std::mem::swap(squad1, squad2);
        }
    }
    if cc_pattern == SquadPattern::Alternating
        && let Some(cc_slot) = &cc_plan.slot_id
        && flip_better(cc_slot, &cc1, &cc2)
        && bar_ok_ids(&cc2)
    {
        std::mem::swap(&mut cc1, &mut cc2);
    }

    if let Some(k2) = (0..SHIFT_COUNT).find(|&k| cc_pattern.squad_at(k) == 1)
        && !cc2.is_empty()
    {
        let room_type_of: HashMap<&str, &str> = building
            .rooms
            .iter()
            .map(|r| (r.slot_id.as_str(), r.room_type.as_str()))
            .collect();
        let mut shift_rooms: Vec<crate::types::RoomAssignment> = groups
            .iter()
            .flat_map(|g| {
                g.rooms.iter().enumerate().filter_map(|(ri, (slot, _))| {
                    let team = &g.teams[g.cells[ri][k2]];
                    let mut ops = kept_by_room.get(slot).cloned().unwrap_or_default();
                    ops.extend(team.ops.iter().cloned());
                    (!ops.is_empty()).then(|| crate::types::RoomAssignment {
                        room_type: g.room_type.clone(),
                        formula_type: g.formula_type.clone(),
                        operators: ops,
                        ..Default::default()
                    })
                })
            })
            .collect();
        for (slot, ops) in &kept_by_room {
            if !groups
                .iter()
                .any(|g| g.rooms.iter().any(|(s, _)| s == slot))
                && let Some(rt) = room_type_of.get(slot.as_str())
            {
                shift_rooms.push(crate::types::RoomAssignment {
                    room_type: (*rt).to_string(),
                    formula_type: None,
                    operators: ops.clone(),
                    ..Default::default()
                });
            }
        }
        cc2.retain(|id| {
            op_index.get(id.as_str()).is_none_or(|op| {
                !crate::assignment::cc_op_is_dead(
                    op,
                    &shift_rooms,
                    &op_index,
                    registry,
                    building_data,
                )
            })
        });
        if !cc2.is_empty() {
            bench.extend(fill_remaining_slots(
                &mut cc2,
                cc_plan.control_slots,
                "CONTROL",
                operators,
                building_data,
                registry,
                &mut assigned,
            ));
        }
    }

    let team_letter = |ordinal: usize| -> String {
        char::from(b'A' + u8::try_from(ordinal % 26).unwrap_or(0)).to_string()
    };
    let crew_efficiency = |crew: &[String], room_type: &str, formula: Option<&str>| -> f64 {
        let (speed, _) = compute_team_efficiency(
            crew,
            room_type,
            formula,
            None,
            &op_index,
            registry,
            building_data,
            &facility_counts,
            total_dorm_levels,
            morale_drains,
            &cc_plan.conditions,
        );
        speed
            + cc_plan
                .global_bonuses
                .get(room_type)
                .copied()
                .unwrap_or(0.0)
    };

    let mut shifts = Vec::with_capacity(SHIFT_COUNT);
    for k in 0..SHIFT_COUNT {
        let mut rooms = Vec::new();

        for g in &groups {
            for (ri, (slot, level)) in g.rooms.iter().enumerate() {
                let ordinal = g.cells[ri][k];
                let team = &g.teams[ordinal];
                let capacity =
                    max_stationed_at_level(building_data, &g.room_type, *level).max(0) as usize;
                let kept = kept_by_room.get(slot).cloned().unwrap_or_default();
                let crew = merge_kept(
                    &kept,
                    &team.ops,
                    capacity,
                    &g.room_type,
                    g.formula_type.as_deref(),
                    &op_index,
                    registry,
                    building_data,
                    &facility_counts,
                    total_dorm_levels,
                    morale_drains,
                );
                let group_tag = g
                    .formula_type
                    .as_deref()
                    .map_or_else(|| g.room_type.clone(), |f| format!("{}:{f}", g.room_type));
                let efficiency = crew_efficiency(&crew, &g.room_type, g.formula_type.as_deref());
                rooms.push(ShiftRoom {
                    slot_id: slot.clone(),
                    room_type: g.room_type.clone(),
                    formula_type: g.formula_type.clone(),
                    active: !crew.is_empty(),
                    recommended: crew,
                    current: preset_for(slot, k),
                    efficiency: Some(efficiency),
                    team_id: Some(format!("{group_tag}:{ordinal}")),
                    team_label: Some(format!("Team {}", team_letter(ordinal))),
                });
            }
        }

        for plant in &power_plan {
            let squad = SquadPattern::Alternating.squad_at(k);
            let crew = if squad == 0 || plant.backup.is_empty() {
                &plant.main
            } else {
                &plant.backup
            };
            let active = (squad == 0 || !plant.backup.is_empty()) && !crew.is_empty();
            let efficiency = crew_efficiency(
                &crate::assignment::working_crew(crew, &parked),
                "POWER",
                None,
            );
            rooms.push(ShiftRoom {
                slot_id: plant.slot_id.clone(),
                room_type: "POWER".to_string(),
                formula_type: None,
                recommended: crew.clone(),
                current: preset_for(&plant.slot_id, k),
                active,
                efficiency: Some(efficiency),
                team_id: Some(format!("POWER:{}:{squad}", plant.slot_id)),
                team_label: Some(format!("Squad {}", squad + 1)),
            });
        }

        for (slot, (room_type, squad1)) in &aux1 {
            let squad = SquadPattern::Alternating.squad_at(k);
            let squad2 = aux2.get(slot).map(|(_, ops)| ops);
            let (crew, active) = match (squad, squad2) {
                (1, Some(b)) if !b.is_empty() => (b.clone(), true),
                (1, _) => (squad1.clone(), false),
                _ => (squad1.clone(), true),
            };
            rooms.push(ShiftRoom {
                slot_id: slot.clone(),
                room_type: room_type.clone(),
                formula_type: None,
                recommended: crew,
                current: preset_for(slot, k),
                active,
                efficiency: None,
                team_id: Some(format!("{room_type}:{slot}:{squad}")),
                team_label: Some(format!("Squad {}", squad + 1)),
            });
        }

        if let Some(cc_slot) = &cc_plan.slot_id {
            let squad = cc_pattern.squad_at(k);
            let (crew, active) = match (squad, cc2.is_empty()) {
                (1, false) => (cc2.clone(), true),
                (1, true) => (cc1.clone(), false),
                _ => (cc1.clone(), true),
            };
            rooms.push(ShiftRoom {
                slot_id: cc_slot.clone(),
                room_type: "CONTROL".to_string(),
                formula_type: None,
                recommended: crew,
                current: preset_for(cc_slot, k),
                active,
                efficiency: None,
                team_id: Some(format!("CONTROL:{squad}")),
                team_label: Some(format!("Squad {}", squad + 1)),
            });
        }

        let shift_seats: HashMap<String, String> = rooms
            .iter()
            .flat_map(|r| {
                r.recommended
                    .iter()
                    .map(move |op| (op.clone(), r.room_type.clone()))
            })
            .collect();
        let mut shift_counts =
            effective_facility_counts(building, operators, registry, building_data, &shift_seats);
        shift_counts.insert(TENURE_HOURS.to_string(), tenure_hours);
        if shift_counts != facility_counts {
            for r in &mut rooms {
                if r.efficiency.is_none()
                    || !(is_production_room(&r.room_type) || r.room_type == "POWER")
                {
                    continue;
                }
                let (speed, _) = compute_team_efficiency(
                    &crate::assignment::working_crew(&r.recommended, &parked),
                    &r.room_type,
                    r.formula_type.as_deref(),
                    None,
                    &op_index,
                    registry,
                    building_data,
                    &shift_counts,
                    total_dorm_levels,
                    morale_drains,
                    &cc_plan.conditions,
                );
                r.efficiency = Some(
                    speed
                        + cc_plan
                            .global_bonuses
                            .get(&r.room_type)
                            .copied()
                            .unwrap_or(0.0),
                );
            }
        }

        shifts.push(Shift {
            index: k + 1,
            rooms,
        });
    }

    let dorms = crate::rotation::dorms::dorm_list(building, building_data);
    if !dorms.is_empty() {
        let mut seated: HashSet<String> = HashSet::new();
        let mut working: Vec<HashSet<String>> = vec![HashSet::new(); SHIFT_COUNT];
        for (k, shift) in shifts.iter().enumerate() {
            for room in shift.rooms.iter().filter(|r| r.active) {
                for id in &room.recommended {
                    seated.insert(id.clone());
                    working[k].insert(id.clone());
                }
            }
        }
        let sustained_set: HashSet<&str> = sustained_label.iter().map(String::as_str).collect();
        let resters_by_shift: Vec<Vec<String>> = (0..SHIFT_COUNT)
            .map(|k| {
                let mut r: Vec<String> = seated
                    .iter()
                    .filter(|id| {
                        !working[k].contains(id.as_str()) && !sustained_set.contains(id.as_str())
                    })
                    .cloned()
                    .collect();
                r.sort_by(|a, b| {
                    let da = op_index.get(a.as_str()).map_or(0.0, |op| {
                        crate::rotation::sustain_sim::game_morale_drain(op, morale_drains)
                    });
                    let db = op_index.get(b.as_str()).map_or(0.0, |op| {
                        crate::rotation::sustain_sim::game_morale_drain(op, morale_drains)
                    });
                    db.partial_cmp(&da)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.cmp(b))
                });
                r
            })
            .collect();
        let mut dorm_pinned: Vec<String> = pins
            .iter()
            .filter(|(id, rt)| rt == "DORMITORY" && !seated.contains(id))
            .map(|(id, _)| id.clone())
            .collect();
        if let Some(m) = &manager_resident
            && !dorm_pinned.contains(m)
            && !seated.contains(m)
        {
            dorm_pinned.push(m.clone());
        }
        let peak_rest = resters_by_shift.iter().map(Vec::len).max().unwrap_or(0);
        let capacity: usize = dorms.iter().map(|d| d.capacity).sum();
        let headroom = capacity
            .saturating_sub(peak_rest)
            .saturating_sub(dorm_pinned.len() * 2);
        let leftovers: Vec<&OperatorBaseProfile> = operators
            .iter()
            .filter(|op| !seated.contains(&op.char_id))
            .filter(|op| !dorm_pinned.iter().any(|id| id == &op.char_id))
            .filter(|op| !sustained_set.contains(op.char_id.as_str()))
            .collect();
        let mut staffing = crate::rotation::dorms::plan_dorm_staffing(
            &dorms,
            &leftovers,
            headroom,
            registry,
            building_data,
        );
        for id in &dorm_pinned {
            if let Some((_, crew)) = staffing.iter_mut().rev().find(|(slot, crew)| {
                let cap = dorms
                    .iter()
                    .find(|d| &d.slot_id == slot)
                    .map_or(0, |d| d.capacity);
                crew.len() < cap
            }) {
                crew.push(id.clone());
            }
        }
        for (k, shift) in shifts.iter_mut().enumerate() {
            let mut queue = resters_by_shift[k].iter();
            for (di, dorm) in dorms.iter().enumerate() {
                let mut crew = staffing[di].1.clone();
                let free = dorm.capacity.saturating_sub(crew.len());
                crew.extend(queue.by_ref().take(free).cloned());
                shift.rooms.push(ShiftRoom {
                    slot_id: dorm.slot_id.clone(),
                    room_type: "DORMITORY".to_string(),
                    formula_type: None,
                    active: !crew.is_empty(),
                    recommended: crew,
                    current: preset_for(&dorm.slot_id, k),
                    efficiency: None,
                    team_id: Some(format!("DORMITORY:{}", dorm.slot_id)),
                    team_label: Some("Resting".to_string()),
                });
            }
        }
    }

    if tiling == Tiling::Shifts {
        let seats_of = |slot: &str| -> usize {
            building
                .rooms
                .iter()
                .find(|r| r.slot_id == slot)
                .map_or(0, |r| {
                    max_stationed_at_level(building_data, &r.room_type, r.level).max(0) as usize
                })
        };
        let short = shifts.iter().any(|s| {
            s.rooms.iter().any(|r| {
                is_production_room(&r.room_type)
                    && r.active
                    && r.recommended.len() < seats_of(&r.slot_id)
            })
        });
        if short {
            return rotation_core_tiled(
                operators,
                building,
                building_data,
                registry,
                morale_drains,
                pins,
                seats,
                Tiling::Blocks,
            );
        }
    }
    ShiftRotation {
        shifts,
        sustained: sustained_label,
        bench: bench.into_iter().collect(),
        parked: parked.into_iter().collect(),
    }
}

fn preset_room_of(building: &UserBuilding, op: &str) -> Option<String> {
    building
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
        .find(|r| {
            let presets: Vec<&Vec<String>> =
                r.preset_shifts.iter().filter(|p| !p.is_empty()).collect();
            !presets.is_empty() && presets.iter().all(|p| p.iter().any(|id| id == op))
        })
        .map(|r| r.slot_id.clone())
}

fn has_power_skill(op: &OperatorBaseProfile, building_data: &BuildingDataFile) -> bool {
    op.available_buffs.iter().any(|b| {
        building_data
            .buffs
            .get(b)
            .is_some_and(|buff| buff.room_type == "POWER")
    })
}

fn power_value(
    op: &OperatorBaseProfile,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    facility_counts: &HashMap<String, usize>,
) -> f64 {
    crate::scoring::ledger::op_power_rank_value(op, building_data, registry, facility_counts)
}

struct PowerPlant {
    slot_id: String,
    /// Squad 1's shifts
    main: Vec<String>,
    /// Squad 2's shift; empty if no spare power operator
    backup: Vec<String>,
}

#[allow(clippy::too_many_arguments)]
fn build_power_plan(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    facility_counts: &HashMap<String, usize>,
    used: &HashSet<String>,
    morale_drains: &HashMap<String, f64>,
    parked: &HashSet<String>,
) -> Vec<PowerPlant> {
    let power_rooms: Vec<&UserRoom> = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "POWER")
        .collect();
    if power_rooms.is_empty() {
        return Vec::new();
    }

    let mut ranked: Vec<(String, f64)> = operators
        .iter()
        .filter(|op| !used.contains(op.char_id.as_str()))
        .filter(|op| has_power_skill(op, building_data))
        .map(|op| {
            (
                op.char_id.clone(),
                power_value(op, building_data, registry, facility_counts),
            )
        })
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let slots: Vec<usize> = power_rooms
        .iter()
        .map(|r| max_stationed_at_level(building_data, "POWER", r.level).max(1) as usize)
        .collect();
    let mut reserved: Vec<Vec<String>> = vec![Vec::new(); power_rooms.len()];
    let mut tokens: Vec<&String> = parked.iter().collect();
    tokens.sort();
    for token in tokens {
        if let Some(i) = (0..power_rooms.len()).find(|&i| reserved[i].len() < slots[i]) {
            reserved[i].push(token.clone());
        }
    }
    let per_squad: usize =
        slots.iter().sum::<usize>() - reserved.iter().map(Vec::len).sum::<usize>();

    let bar_ok = |id: &str| -> bool {
        operators
            .iter()
            .find(|op| op.char_id == id)
            .is_none_or(|op| crate::rotation::sustain_sim::sustains_24h_block(op, morale_drains))
    };
    let mut squads: [(Vec<String>, f64); 2] = [(Vec::new(), 0.0), (Vec::new(), 0.0)];
    for (id, value) in ranked.into_iter().take(per_squad * 2) {
        let pick = (0..2)
            .filter(|&i| squads[i].0.len() < per_squad)
            .filter(|&i| i == 1 || bar_ok(&id))
            .min_by(|&a, &b| {
                squads[a]
                    .1
                    .partial_cmp(&squads[b].1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        if let Some(i) = pick {
            squads[i].0.push(id);
            squads[i].1 += value;
        }
    }
    if squads[1].1 > squads[0].1 && squads[1].0.iter().all(|id| bar_ok(id)) {
        squads.swap(0, 1);
    }
    let [(squad1, _), (squad2, _)] = squads;
    let mut squad1 = squad1.into_iter();
    let mut squad2 = squad2.into_iter();

    power_rooms
        .iter()
        .zip(&slots)
        .zip(reserved)
        .map(|((r, n), held)| {
            let free = n - held.len();
            PowerPlant {
                slot_id: r.slot_id.clone(),
                main: held
                    .iter()
                    .cloned()
                    .chain((0..free).filter_map(|_| squad1.next()))
                    .collect(),
                backup: {
                    let picks: Vec<String> = (0..free).filter_map(|_| squad2.next()).collect();
                    if picks.is_empty() && held.is_empty() {
                        Vec::new()
                    } else {
                        held.iter().cloned().chain(picks).collect()
                    }
                },
            }
        })
        .collect()
}
