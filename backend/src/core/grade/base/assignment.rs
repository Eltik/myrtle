use std::collections::{HashMap, HashSet};

use crate::core::{
    gamedata::types::building::{Buff, BuildingDataFile},
    grade::base::{
        buff_registry::BuffResolutionStrategy,
        evaluate::evaluate_buff,
        types::{
            BaseAssignment, EvalContext, OperatorBaseProfile, RoomAssignment, ShiftAssignment,
            TeammateInfo, UserBuilding, UserRoom,
        },
    },
};

/// Most candidate operators considered per room before exhaustively searching
/// team combinations. The optimal team is essentially always among the top
/// candidates by optimistic bound (synergy enablers are boosted into this set),
/// so 16 covers realistic optima while keeping C(16,3) combinations cheap.
const CANDIDATE_LIMIT: usize = 16;

/// Build a `char_id` → profile index for O(1) lookups in the hot inner loops.
fn build_op_index(operators: &[OperatorBaseProfile]) -> HashMap<&str, &OperatorBaseProfile> {
    operators.iter().map(|o| (o.char_id.as_str(), o)).collect()
}

pub fn compute_optimal_assignment(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> BaseAssignment {
    let facility_counts = count_facilities(building);
    let total_dorm_levels = building.total_dorm_levels();

    let mut assigned: HashSet<String> = HashSet::new();

    let control_room = building.rooms.iter().find(|r| r.room_type == "CONTROL");
    // Only staff the Control Center if one is actually built - otherwise we'd
    // strip CC operators out of the production pool for a room that doesn't exist.
    let control_slots = if control_room.is_some() {
        max_stationed_for_room(building, building_data, "CONTROL")
    } else {
        0
    };
    let (cc_room, global_bonuses, cc_conditions) = assign_control_center(
        operators,
        control_room,
        control_slots,
        registry,
        building_data,
        &assigned,
    );
    for id in &cc_room.operators {
        assigned.insert(id.clone());
    }

    let production_rooms: Vec<&UserRoom> = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "MANUFACTURE" || r.room_type == "TRADING")
        .collect();

    let room_assignments = assign_production_rooms(
        &production_rooms,
        operators,
        &mut assigned,
        registry,
        building_data,
        &facility_counts,
        total_dorm_levels,
        &global_bonuses,
        &cc_conditions,
        morale_drains,
    );

    let total_production_efficiency = room_assignments.iter().map(|r| r.total_efficiency).sum();

    let mut rooms = room_assignments;
    // Surface the Control Center so its operators (and the global bonuses they
    // provide to every production room) are visible in the recommendation.
    // Fill any remaining CC slots with leftover operators - production has
    // already claimed everyone it benefits from, so these are true spares.
    let mut cc_room = cc_room;
    fill_remaining_slots(
        &mut cc_room.operators,
        control_slots,
        "CONTROL",
        operators,
        building_data,
        &mut assigned,
    );
    if let Some(cc) = cc_room.into_room() {
        rooms.insert(0, cc);
    }

    BaseAssignment {
        rooms,
        total_production_efficiency,
    }
}

pub fn compute_sustained_assignment(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> ShiftAssignment {
    let facility_counts = count_facilities(building);
    let total_dorm_levels = building.total_dorm_levels();

    let production_rooms: Vec<&UserRoom> = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "MANUFACTURE" || r.room_type == "TRADING")
        .collect();

    let control_room = building.rooms.iter().find(|r| r.room_type == "CONTROL");
    let control_slots = if control_room.is_some() {
        max_stationed_for_room(building, building_data, "CONTROL")
    } else {
        0
    };

    // A sustained 24/7 base rotates EVERY occupied seat - including the Control
    // Center - so its operators recover morale too. Each shift takes a disjoint CC
    // team (with its own global bonuses) and disjoint production team from the
    // operators not yet used by the previous shift.
    let mut assigned: HashSet<String> = HashSet::new();
    let assign_shift = |assigned: &mut HashSet<String>| {
        let (cc, global, cond) = assign_control_center(
            operators,
            control_room,
            control_slots,
            registry,
            building_data,
            assigned,
        );
        assigned.extend(cc.operators.iter().cloned());
        let rooms = assign_production_rooms(
            &production_rooms,
            operators,
            assigned,
            registry,
            building_data,
            &facility_counts,
            total_dorm_levels,
            &global,
            &cond,
            morale_drains,
        );
        let total: f64 = rooms.iter().map(|r| r.total_efficiency).sum();
        (cc, rooms, total)
    };

    let (mut cc_a, mut rooms_a, total_a) = assign_shift(&mut assigned);
    let (mut cc_b, mut rooms_b, total_b) = assign_shift(&mut assigned);

    // Top up each shift's CC seats with operators still idle in that shift.
    fill_remaining_slots(
        &mut cc_a.operators,
        control_slots,
        "CONTROL",
        operators,
        building_data,
        &mut assigned,
    );
    fill_remaining_slots(
        &mut cc_b.operators,
        control_slots,
        "CONTROL",
        operators,
        building_data,
        &mut assigned,
    );

    if let Some(cc) = cc_a.into_room() {
        rooms_a.insert(0, cc);
    }
    if let Some(cc) = cc_b.into_room() {
        rooms_b.insert(0, cc);
    }

    ShiftAssignment {
        shift_a: BaseAssignment {
            rooms: rooms_a,
            total_production_efficiency: total_a,
        },
        shift_b: BaseAssignment {
            rooms: rooms_b,
            total_production_efficiency: total_b,
        },
        sustained_efficiency: (total_a + total_b) / 2.0,
    }
}

/// Which operators occupy a room for a given live view: the static stationed
/// crew (`shift = None`) or one of the player's planned preset rotation shifts.
fn room_ops_for_shift(room: &UserRoom, shift: Option<usize>) -> Vec<String> {
    match shift {
        Some(i) => room
            .preset_shifts
            .get(i)
            .cloned()
            .unwrap_or_else(|| room.current_operators.clone()),
        None => room.current_operators.clone(),
    }
}

/// Does the building define preset rotation shifts the player has planned?
pub fn has_preset_shifts(building: &UserBuilding) -> bool {
    building.rooms.iter().any(|r| r.preset_shifts.len() > 1)
}

/// Evaluate the player's CURRENT base as stationed right now (`shift = None`) or
/// for one of their planned preset rotation shifts (`shift = Some(i)`), so it can
/// be compared against the optimizer's output.
pub fn compute_current_assignment(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    shift: Option<usize>,
) -> BaseAssignment {
    let facility_counts = count_facilities(building);
    let total_dorm_levels = building.total_dorm_levels();
    let op_index = build_op_index(operators);

    // Global bonuses from whoever the player has in the Control Center for this
    // shift (same non-stacking rule as the optimizer).
    let control_room = building.rooms.iter().find(|r| r.room_type == "CONTROL");
    let cc_ops = control_room
        .map(|cc| room_ops_for_shift(cc, shift))
        .unwrap_or_default();
    let mut acc = CcBonusAccumulator::default();
    for op in cc_ops
        .iter()
        .filter_map(|id| op_index.get(id.as_str()).copied())
    {
        acc.add(&cc_bonuses(op, registry, building_data));
    }
    let (global_bonuses, cc_conditions) = acc.finish();

    let mut rooms: Vec<RoomAssignment> = Vec::new();

    // Show the current Control Center (with the bonuses it currently provides).
    if let Some(cc) = control_room
        && !cc_ops.is_empty()
    {
        rooms.push(RoomAssignment {
            slot_id: cc.slot_id.clone(),
            room_type: "CONTROL".to_string(),
            level: cc.level,
            formula_type: None,
            operators: cc_ops,
            total_efficiency: global_bonuses.values().sum(),
            ..Default::default()
        });
    }

    let mut total = 0.0;
    for room in building
        .rooms
        .iter()
        .filter(|r| r.room_type == "MANUFACTURE" || r.room_type == "TRADING")
    {
        // Keep only operators that actually have base profiles (drop tokens etc.).
        let ops: Vec<String> = room_ops_for_shift(room, shift)
            .into_iter()
            .filter(|id| op_index.contains_key(id.as_str()))
            .collect();
        let formula = if room.room_type == "MANUFACTURE" {
            room.current_formula
                .clone()
                .or_else(|| Some("F_GOLD".to_string()))
        } else {
            None
        };
        let global = *global_bonuses.get(&room.room_type).unwrap_or(&0.0);
        let (speed, value) = compute_team_efficiency(
            &ops,
            &room.room_type,
            formula.as_deref(),
            &op_index,
            registry,
            building_data,
            &facility_counts,
            total_dorm_levels,
            morale_drains,
            &cc_conditions,
        );
        let eff = speed + global;
        total += eff;
        rooms.push(RoomAssignment {
            slot_id: room.slot_id.clone(),
            room_type: room.room_type.clone(),
            level: room.level,
            formula_type: formula,
            operators: ops,
            total_efficiency: eff,
            order_value: value,
        });
    }

    BaseAssignment {
        rooms,
        total_production_efficiency: total,
    }
}

fn count_facilities(building: &UserBuilding) -> HashMap<String, usize> {
    let mut facility_counts: HashMap<String, usize> = HashMap::new();
    for room in &building.rooms {
        *facility_counts.entry(room.room_type.clone()).or_insert(0) += 1;
    }
    facility_counts
}

/// Get the max operator slots for the first room of a given type in the user's base.
/// Falls back to 1 if not found.
/// Max operators stationable in a room of `room_type` at a given `level`.
fn max_stationed_at_level(building_data: &BuildingDataFile, room_type: &str, level: i32) -> i32 {
    building_data
        .rooms
        .get(room_type)
        .and_then(|def| def.phases.get((level - 1).max(0) as usize))
        .map_or(1, |phase| phase.max_stationed_num)
}

/// Max stationed for the user's highest-level room of `room_type` (e.g. their CC).
fn max_stationed_for_room(
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    room_type: &str,
) -> i32 {
    let level = building
        .rooms
        .iter()
        .filter(|r| r.room_type == room_type)
        .map(|r| r.level)
        .max()
        .unwrap_or(1);
    max_stationed_at_level(building_data, room_type, level)
}

/// A Control Center assignment plus the global production bonuses it grants.
struct ControlCenter {
    operators: Vec<String>,
    /// The user's CONTROL room (slot/level), if they have one built.
    slot_id: Option<String>,
    level: i32,
    /// Total global bonus % across all targeted production room types - shown
    /// as the CC room's "efficiency" so its impact is visible.
    total_global_pct: f64,
}

impl ControlCenter {
    fn into_room(self) -> Option<RoomAssignment> {
        let slot_id = self.slot_id?;
        if self.operators.is_empty() {
            return None;
        }
        Some(RoomAssignment {
            slot_id,
            room_type: "CONTROL".to_string(),
            level: self.level,
            formula_type: None,
            operators: self.operators,
            total_efficiency: self.total_global_pct,
            ..Default::default()
        })
    }
}

/// Append unused operators to `slots` until it reaches `max_slots`, marking each
/// as `assigned`. Fillers occupy seats but add no value, so we pick the operators
/// with the lowest *opportunity cost* - those with the fewest base skills for
/// OTHER rooms - keeping operators with useful Dormitory/Training/etc. skills
/// (e.g. Schwarz) available for the rooms where they actually help.
fn fill_remaining_slots(
    slots: &mut Vec<String>,
    max_slots: i32,
    room_type: &str,
    operators: &[OperatorBaseProfile],
    building_data: &BuildingDataFile,
    assigned: &mut HashSet<String>,
) {
    while (slots.len() as i32) < max_slots {
        let pick = operators
            .iter()
            .filter(|op| !assigned.contains(&op.char_id))
            .min_by_key(|op| other_room_skill_count(op, room_type, building_data));
        let Some(op) = pick else { break };
        assigned.insert(op.char_id.clone());
        slots.push(op.char_id.clone());
    }
}

/// How many *other* room types this operator has base skills for (a proxy for
/// how much value is lost by using them as a filler here). 0 = pure benchwarmer.
fn other_room_skill_count(
    op: &OperatorBaseProfile,
    room_type: &str,
    building_data: &BuildingDataFile,
) -> usize {
    let mut rooms: Vec<&str> = op
        .available_buffs
        .iter()
        .filter_map(|b| building_data.buffs.get(b))
        .map(|buff| buff.room_type.as_str())
        .filter(|rt| *rt != room_type)
        .collect();
    rooms.sort_unstable();
    rooms.dedup();
    rooms.len()
}

/// A single Control Center global bonus: which production room it boosts, the
/// "skill effect family" (buff id without its `[NNN]` rank suffix), and the
/// bonus %. Same-family bonuses from different operators do NOT stack - only the
/// strongest applies ("only the most effective one will take effect when
/// assigned Operators have the same skill effect") - so we key on the family.
struct CcBonus {
    room: String,
    family: String,
    bonus: f64,
    /// `Some` when this bonus only applies to production rooms whose team meets a
    /// faction condition - credited per-room, not flat. `bonus` above is then a
    /// discounted *selection* weight, not the value actually granted.
    conditional: Option<CcCondition>,
}

/// A Control Center bonus that is gated on a production room's team composition.
/// Resolved per-room in `compute_team_efficiency` rather than added flat.
#[derive(Clone)]
struct CcCondition {
    target_room: String,
    faction_token: String,
    required_count: usize,
    per_operator: bool,
    bonus_pct: f64,
}

impl CcCondition {
    /// This condition's contribution to a room of `room_type` staffed by `team`.
    fn contribution(&self, room_type: &str, team: &[&OperatorBaseProfile]) -> f64 {
        if self.target_room != room_type {
            return 0.0;
        }
        let count = team
            .iter()
            .filter(|op| op.match_tags.contains(&self.faction_token))
            .count();
        if self.per_operator {
            self.bonus_pct * count as f64
        } else if count >= self.required_count {
            self.bonus_pct
        } else {
            0.0
        }
    }
}

/// Folds Control Center bonuses together under the non-stacking rule (only the
/// strongest per skill-effect family applies), separating unconditional bonuses
/// (summed flat per room) from faction-gated ones (returned for per-room,
/// team-dependent evaluation). Shared by the optimizer and the live-base reader.
#[derive(Default)]
struct CcBonusAccumulator {
    best: HashMap<(String, String), f64>,
    conditions: HashMap<(String, String), CcCondition>,
    cond_families: HashSet<(String, String)>,
}

impl CcBonusAccumulator {
    /// The marginal flat gain from adding `bonuses` - each one only counts for the
    /// amount it exceeds the strongest already chosen in its family.
    fn marginal(&self, bonuses: &[CcBonus]) -> f64 {
        bonuses
            .iter()
            .map(|b| {
                let cur = self
                    .best
                    .get(&(b.room.clone(), b.family.clone()))
                    .copied()
                    .unwrap_or(0.0);
                (b.bonus - cur).max(0.0)
            })
            .sum()
    }

    /// Fold one operator's bonuses in, keeping the strongest per family.
    fn add(&mut self, bonuses: &[CcBonus]) {
        for b in bonuses {
            let key = (b.room.clone(), b.family.clone());
            let slot = self.best.entry(key.clone()).or_insert(0.0);
            *slot = slot.max(b.bonus);
            if let Some(cond) = &b.conditional {
                self.cond_families.insert(key.clone());
                self.conditions
                    .entry(key)
                    .and_modify(|c| {
                        if cond.bonus_pct > c.bonus_pct {
                            *c = cond.clone();
                        }
                    })
                    .or_insert_with(|| cond.clone());
            }
        }
    }

    /// `(flat per-room totals, faction-gated conditions)`. Conditional families are
    /// kept out of the flat totals - they're applied per room against the team.
    fn finish(self) -> (HashMap<String, f64>, Vec<CcCondition>) {
        let mut global: HashMap<String, f64> = HashMap::new();
        for ((room, family), bonus) in &self.best {
            if !self.cond_families.contains(&(room.clone(), family.clone())) {
                *global.entry(room.clone()).or_insert(0.0) += bonus;
            }
        }
        (global, self.conditions.into_values().collect())
    }
}

/// Collect an operator's Control Center global production bonuses.
fn cc_bonuses(
    op: &OperatorBaseProfile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> Vec<CcBonus> {
    let mut out = Vec::new();
    for buff_id in &op.available_buffs {
        if let Some(buff) = building_data.buffs.get(buff_id)
            && buff.room_type != "CONTROL"
        {
            continue;
        }
        let family = buff_id.split('[').next().unwrap_or(buff_id).to_string();
        match registry.get(buff_id) {
            Some(BuffResolutionStrategy::GlobalEffect {
                target_room,
                bonus_pct,
            }) => out.push(CcBonus {
                room: target_room.clone(),
                family,
                bonus: *bonus_pct,
                conditional: None,
            }),
            Some(BuffResolutionStrategy::TagBased {
                target_room,
                bonus_pct,
                ..
            }) => out.push(CcBonus {
                room: target_room.clone(),
                family,
                // Faction bonuses only reach matching operators - credit half.
                bonus: bonus_pct * 0.5,
                conditional: None,
            }),
            Some(BuffResolutionStrategy::ConditionalGlobalEffect {
                target_room,
                faction_token,
                required_count,
                per_operator,
                bonus_pct,
            }) => out.push(CcBonus {
                room: target_room.clone(),
                family,
                // Selection weight: discounted since the gate may not be met. The
                // real value is granted per-room via `conditional`.
                bonus: bonus_pct * 0.5,
                conditional: Some(CcCondition {
                    target_room: target_room.clone(),
                    faction_token: faction_token.clone(),
                    required_count: *required_count,
                    per_operator: *per_operator,
                    bonus_pct: *bonus_pct,
                }),
            }),
            _ => {}
        }
    }
    out
}

/// Assign operators to the Control Center to maximize global production bonuses.
/// Returns the CC assignment and the global bonuses map (`room_type` -> total bonus %).
///
/// Selection is marginal-greedy and respects the non-stacking rule: filling a
/// slot with a second operator whose only effect duplicates one already present
/// adds nothing, so a complementary operator (different effect family / target
/// room) is preferred instead.
fn assign_control_center(
    operators: &[OperatorBaseProfile],
    control_room: Option<&UserRoom>,
    max_slots: i32,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    exclude: &HashSet<String>,
) -> (ControlCenter, HashMap<String, f64>, Vec<CcCondition>) {
    let candidates: Vec<(&OperatorBaseProfile, Vec<CcBonus>)> = operators
        .iter()
        .filter(|op| !exclude.contains(&op.char_id))
        .map(|op| (op, cc_bonuses(op, registry, building_data)))
        .filter(|(_, b)| !b.is_empty())
        .collect();

    // Marginal-greedy selection: each slot takes the operator adding the most flat
    // gain over what's already chosen (the non-stacking rule lives in the accumulator).
    let mut acc = CcBonusAccumulator::default();
    let mut chosen: HashSet<String> = HashSet::new();
    let mut cc_assigned: Vec<String> = Vec::new();

    for _ in 0..max_slots.max(0) {
        let mut best_op: Option<&OperatorBaseProfile> = None;
        let mut best_bonuses: &[CcBonus] = &[];
        let mut best_gain = 0.0;
        for (op, bonuses) in &candidates {
            if chosen.contains(&op.char_id) {
                continue;
            }
            let gain = acc.marginal(bonuses);
            if gain > best_gain {
                best_gain = gain;
                best_op = Some(op);
                best_bonuses = bonuses;
            }
        }
        let Some(op) = best_op else { break };
        chosen.insert(op.char_id.clone());
        cc_assigned.push(op.char_id.clone());
        acc.add(best_bonuses);
    }

    let (global_bonuses, conditions) = acc.finish();
    let cc = ControlCenter {
        operators: cc_assigned,
        slot_id: control_room.map(|r| r.slot_id.clone()),
        level: control_room.map_or(1, |r| r.level),
        total_global_pct: global_bonuses.values().sum(),
    };
    (cc, global_bonuses, conditions)
}

#[allow(clippy::too_many_arguments)]
fn assign_production_rooms(
    rooms: &[&UserRoom],
    operators: &[OperatorBaseProfile],
    assigned: &mut HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    global_bonuses: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
) -> Vec<RoomAssignment> {
    let factory_rooms: Vec<&&UserRoom> = rooms
        .iter()
        .filter(|r| r.room_type == "MANUFACTURE")
        .collect();
    let trading_rooms: Vec<&&UserRoom> =
        rooms.iter().filter(|r| r.room_type == "TRADING").collect();

    let num_factories = factory_rooms.len();

    // Try all gold/EXP splits and pick the best
    let mut best_assignments: Vec<RoomAssignment> = Vec::new();
    let mut best_objective: f64 = f64::NEG_INFINITY;
    let mut best_assigned_snapshot: HashSet<String> = assigned.clone();

    // Gold factories must be >= trading post count (TPs need gold bars to trade).
    let min_gold = trading_rooms.len().min(num_factories);
    // The yield-coupled optimum is "just enough gold to feed the trading posts"
    // (excess gold is unsold and worth less than EXP), so only the splits at and
    // just above min_gold are ever competitive - no need to scan all the way up.
    let max_gold = (min_gold + 1).min(num_factories);

    for num_gold in min_gold..=max_gold {
        let mut trial_assigned = assigned.clone();
        let mut trial_rooms: Vec<RoomAssignment> = Vec::new();

        // Assign factories: first num_gold get F_GOLD, rest get F_EXP
        for (i, factory) in factory_rooms.iter().enumerate() {
            let formula = if i < num_gold { "F_GOLD" } else { "F_EXP" };
            let room_assignment = assign_single_room(
                factory,
                Some(formula),
                operators,
                &mut trial_assigned,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
                global_bonuses,
                cc_conditions,
                morale_drains,
            );
            trial_rooms.push(room_assignment);
        }

        // Assign trading posts (no formula filter)
        for tp in &trading_rooms {
            let room_assignment = assign_single_room(
                tp,
                None,
                operators,
                &mut trial_assigned,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
                global_bonuses,
                cc_conditions,
                morale_drains,
            );
            trial_rooms.push(room_assignment);
        }

        // Balance operators across same-type/formula rooms (diminishing-returns
        // aware), then score this split by the soft-capped objective so stacking
        // everything into one room is penalized.
        rebalance_rooms(
            &mut trial_rooms,
            operators,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
            global_bonuses,
            cc_conditions,
            morale_drains,
        );

        // Score the split by realized daily YIELD (LMD-equivalent), not by a sum
        // of efficiency %. This makes the gold/EXP split value-based: gold
        // factories only pay off up to the trading posts' selling capacity, so
        // excess gold factories are correctly switched to EXP.
        let trial_objective = assignment_value(&trial_rooms);
        if trial_objective > best_objective {
            best_objective = trial_objective;
            best_assignments = trial_rooms;
            best_assigned_snapshot = trial_assigned;
        }
    }

    // Apply the winning assignment's used operators to the real assigned set
    *assigned = best_assigned_snapshot;

    // Now that every room has its best *beneficial* team (and only now - so a
    // valuable operator is never wasted as a filler in a room where they do
    // nothing), top each room up to a full crew with leftover operators.
    pad_production_rooms(
        &mut best_assignments,
        operators,
        assigned,
        registry,
        building_data,
        facility_counts,
        total_dorm_levels,
        global_bonuses,
        cc_conditions,
        morale_drains,
    );

    best_assignments
}

/// Fill every production room up to its slot count with leftover operators.
/// Runs only after all beneficial assignment is done, so the fillers are
/// operators no room benefits from. A normal room's efficiency is recomputed
/// (a filler with a small relevant buff still helps); an automation room keeps
/// its efficiency since extra operators are nullified.
#[allow(clippy::too_many_arguments)]
fn pad_production_rooms(
    rooms: &mut [RoomAssignment],
    operators: &[OperatorBaseProfile],
    assigned: &mut HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    global_bonuses: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
) {
    let op_index = build_op_index(operators);
    for room in rooms.iter_mut() {
        let max_slots = max_stationed_at_level(building_data, &room.room_type, room.level);
        let global = *global_bonuses.get(&room.room_type).unwrap_or(&0.0);
        let is_auto = room
            .operators
            .iter()
            .filter_map(|id| operators.iter().find(|o| o.char_id == *id))
            .any(|op| has_automation_buff(op, registry));

        if is_auto {
            // Extra operators are nullified - occupy the seats with the lowest
            // opportunity-cost benchwarmers available.
            fill_remaining_slots(
                &mut room.operators,
                max_slots,
                &room.room_type,
                operators,
                building_data,
                assigned,
            );
        } else {
            // Add the leftover that maximizes the room's efficiency (usually +0);
            // among equally-good picks, prefer the lowest opportunity-cost one so
            // operators useful elsewhere aren't burned as fillers.
            while (room.operators.len() as i32) < max_slots {
                let mut best_id: Option<String> = None;
                let mut best_score = f64::NEG_INFINITY;
                let mut best_speed = 0.0;
                let mut best_value = 0.0;
                let mut best_cost = usize::MAX;
                for op in operators {
                    if assigned.contains(&op.char_id) {
                        continue;
                    }
                    let mut trial = room.operators.clone();
                    trial.push(op.char_id.clone());
                    let (speed, value) = compute_team_efficiency(
                        &trial,
                        &room.room_type,
                        room.formula_type.as_deref(),
                        &op_index,
                        registry,
                        building_data,
                        facility_counts,
                        total_dorm_levels,
                        morale_drains,
                        cc_conditions,
                    );
                    let score = room_search_score(&room.room_type, speed, value);
                    let cost = other_room_skill_count(op, &room.room_type, building_data);
                    let better = best_id.is_none()
                        || score > best_score + 1e-9
                        || ((score - best_score).abs() <= 1e-9 && cost < best_cost);
                    if better {
                        best_score = score;
                        best_speed = speed;
                        best_value = value;
                        best_cost = cost;
                        best_id = Some(op.char_id.clone());
                    }
                }
                match best_id {
                    Some(id) => {
                        assigned.insert(id.clone());
                        room.operators.push(id);
                        room.total_efficiency = best_speed + global;
                        room.order_value = best_value;
                    }
                    None => break, // roster exhausted
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn assign_single_room(
    room: &UserRoom,
    formula_type: Option<&str>,
    operators: &[OperatorBaseProfile],
    assigned: &mut HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    global_bonuses: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
) -> RoomAssignment {
    let max_slots = max_stationed_at_level(building_data, &room.room_type, room.level);
    let global = *global_bonuses.get(&room.room_type).unwrap_or(&0.0);

    // Mode 1: best normal team (exhaustive search over top candidate combinations)
    let (normal_ops, normal_speed, normal_value) = best_team_for_room(
        room,
        formula_type,
        operators,
        assigned,
        registry,
        building_data,
        facility_counts,
        total_dorm_levels,
        max_slots,
        cc_conditions,
        morale_drains,
    );

    // Mode 2: automation (factories only) - a single op's facility-count buff
    // nullifies teammates, so this team is scored independently. No order value.
    let (auto_ops, auto_speed) = if room.room_type == "MANUFACTURE" {
        best_automation_team(
            room,
            formula_type,
            operators,
            assigned,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
            max_slots,
        )
    } else {
        (Vec::new(), f64::NEG_INFINITY)
    };

    // Pick the mode by realized value (order value multiplies trading LMD).
    let normal_score = room_search_score(&room.room_type, normal_speed, normal_value);
    let auto_score = room_search_score(&room.room_type, auto_speed, 0.0);
    let (room_ops, speed, value) = if auto_score > normal_score && !auto_ops.is_empty() {
        (auto_ops, auto_speed, 0.0)
    } else {
        (normal_ops, normal_speed, normal_value)
    };

    for id in &room_ops {
        assigned.insert(id.clone());
    }

    RoomAssignment {
        slot_id: room.slot_id.clone(),
        room_type: room.room_type.clone(),
        level: room.level,
        formula_type: formula_type.map(std::string::ToString::to_string),
        operators: room_ops,
        total_efficiency: speed + global,
        order_value: value,
    }
}

/// Does this buff apply to the given room type and (optional) formula?
fn buff_applies(buff: &Buff, room_type: &str, formula_type: Option<&str>) -> bool {
    if buff.room_type != room_type {
        return false;
    }
    if let Some(formula) = formula_type
        && !buff.targets.is_empty()
        && !buff.targets.iter().any(|t| t == formula)
    {
        return false;
    }
    true
}

/// Resolved strategies for an operator's buffs that apply in the given room -
/// the shared "look up the buff, gate on room/formula, resolve to a strategy"
/// skeleton used by the per-operator scoring helpers.
fn applicable_strategies<'a>(
    op: &'a OperatorBaseProfile,
    room_type: &'a str,
    formula_type: Option<&'a str>,
    registry: &'a HashMap<String, BuffResolutionStrategy>,
    building_data: &'a BuildingDataFile,
) -> impl Iterator<Item = &'a BuffResolutionStrategy> {
    op.available_buffs.iter().filter_map(move |buff_id| {
        let buff = building_data.buffs.get(buff_id)?;
        buff_applies(buff, room_type, formula_type)
            .then(|| registry.get(buff_id))
            .flatten()
    })
}

/// Find the best NORMAL-mode team for a room by exhaustively evaluating
/// combinations of the top candidate operators.
///
/// Greedy slot-by-slot filling can't discover synergies that require *both*
/// members present (e.g. Texas's +65% only triggers with Lappland, and Lappland
/// alone scores ~0) - neither operator bootstraps the pair. Searching whole-team
/// combinations evaluates the pair together, so the true optimum is found.
#[allow(clippy::too_many_arguments)]
fn best_team_for_room(
    room: &UserRoom,
    formula_type: Option<&str>,
    operators: &[OperatorBaseProfile],
    already_assigned: &HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    max_slots: i32,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
) -> (Vec<String>, f64, f64) {
    if max_slots <= 0 {
        return (Vec::new(), 0.0, 0.0);
    }
    let max_slots = max_slots as usize;
    let op_index = build_op_index(operators);

    // Candidate pool: operators with at least one applicable buff, excluding
    // automation operators (their nullify-others buff can't mix with a team).
    let candidates: Vec<&OperatorBaseProfile> = operators
        .iter()
        .filter(|op| !already_assigned.contains(&op.char_id))
        .filter(|op| {
            op.available_buffs.iter().any(|b| {
                building_data
                    .buffs
                    .get(b)
                    .is_some_and(|buff| buff_applies(buff, &room.room_type, formula_type))
            })
        })
        .filter(|op| !has_automation_buff(op, registry))
        .collect();

    if candidates.is_empty() {
        return (Vec::new(), 0.0, 0.0);
    }

    // Operators that *enable* another candidate's conditional buff get that
    // unlocked value folded into their ranking score, so synergy enablers (who
    // score ~0 standalone) survive the top-K cut.
    let mut enabler_boost: HashMap<String, f64> = HashMap::new();
    for op in &candidates {
        for b in &op.available_buffs {
            if let Some(BuffResolutionStrategy::ConditionalOnTeammate {
                required_char_id: Some(req),
                efficiency,
                ..
            }) = registry.get(b)
            {
                let slot = enabler_boost.entry(req.clone()).or_insert(0.0);
                *slot = slot.max(*efficiency);
            }
        }
    }

    let mut ranked: Vec<(&OperatorBaseProfile, f64)> = candidates
        .iter()
        .map(|op| {
            let bound = optimistic_bound(
                op,
                &room.room_type,
                formula_type,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
                max_slots,
            ) + enabler_boost.get(&op.char_id).copied().unwrap_or(0.0);
            (*op, bound)
        })
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(CANDIDATE_LIMIT);

    let pool: Vec<String> = ranked.iter().map(|(op, _)| op.char_id.clone()).collect();

    let mut best_team: Vec<String> = Vec::new();
    let mut best_score = 0.0;
    let mut best_speed = 0.0;
    let mut best_value = 0.0;

    for combo in combinations(&pool, max_slots) {
        let (speed, value) = compute_team_efficiency(
            &combo,
            &room.room_type,
            formula_type,
            &op_index,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
            morale_drains,
            cc_conditions,
        );
        let score = room_search_score(&room.room_type, speed, value);
        if score > best_score {
            best_score = score;
            best_speed = speed;
            best_value = value;
            best_team = combo;
        }
    }

    (best_team, best_speed, best_value)
}

/// Best automation team for a factory: greedily take the operators with the
/// highest facility-count-scaling value (these don't synergize with teammates).
#[allow(clippy::too_many_arguments)]
fn best_automation_team(
    room: &UserRoom,
    formula_type: Option<&str>,
    operators: &[OperatorBaseProfile],
    already_assigned: &HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    max_slots: i32,
) -> (Vec<String>, f64) {
    let mut scored: Vec<(&OperatorBaseProfile, f64)> = operators
        .iter()
        .filter(|op| !already_assigned.contains(&op.char_id))
        .filter(|op| has_automation_buff(op, registry))
        .map(|op| {
            let val = score_operator_facility_only(
                op,
                &room.room_type,
                formula_type,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
            );
            (op, val)
        })
        .filter(|(_, v)| *v > 0.0)
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut ops = Vec::new();
    let mut total = 0.0;
    for (op, val) in scored.into_iter().take(max_slots.max(0) as usize) {
        ops.push(op.char_id.clone());
        total += val;
    }
    (ops, total)
}

fn has_automation_buff(
    op: &OperatorBaseProfile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> bool {
    op.available_buffs.iter().any(|b| {
        matches!(
            registry.get(b),
            Some(BuffResolutionStrategy::FacilityCountScaling {
                nullifies_others: true,
                ..
            })
        )
    })
}

/// True if the operator nullifies teammates' output in this room (Shamare-type).
fn op_is_nullifier(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> bool {
    applicable_strategies(op, room_type, formula_type, registry, building_data).any(|s| {
        matches!(
            s,
            BuffResolutionStrategy::NullifyTeammatesSelfScaling { .. }
        )
    })
}

/// An optimistic upper bound on an operator's solo value in a room, used only
/// for ranking candidates (conditionals assumed satisfied, teammate-scaling
/// assumed fully matched). Never used as the final score.
#[allow(clippy::too_many_arguments)]
fn optimistic_bound(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    max_slots: usize,
) -> f64 {
    let mut total = 0.0;
    let teammates_assumed = max_slots.saturating_sub(1) as f64;
    for strategy in applicable_strategies(op, room_type, formula_type, registry, building_data) {
        match strategy {
            BuffResolutionStrategy::DirectEfficiency { value }
            | BuffResolutionStrategy::Complex {
                estimated_pct: value,
            }
            | BuffResolutionStrategy::OrderValue {
                estimated_pct: value,
            }
            | BuffResolutionStrategy::MoraleDecayEfficiency {
                time_averaged_value: value,
            } => {
                total += value;
            }
            BuffResolutionStrategy::EfficiencyWithOrderLimit { efficiency, .. } => {
                total += efficiency;
            }
            // Optimistic: assume every gating teammate / faction is present.
            BuffResolutionStrategy::ConditionalOnTeammate {
                base_efficiency,
                efficiency,
                ..
            }
            | BuffResolutionStrategy::ConditionalOnFaction {
                base_efficiency,
                efficiency,
                ..
            } => {
                total += base_efficiency + efficiency;
            }
            BuffResolutionStrategy::TeammateSkillScaling { per_match_pct, .. } => {
                total += per_match_pct * teammates_assumed;
            }
            BuffResolutionStrategy::MatchCountScaling {
                per_match_pct,
                cap_pct,
                bonus_pct,
                ..
            } => {
                let v = per_match_pct * teammates_assumed;
                total += cap_pct.map_or(v, |c| v.min(c)) + bonus_pct;
            }
            BuffResolutionStrategy::NullifyTeammatesSelfScaling { per_teammate_pct } => {
                total += per_teammate_pct * teammates_assumed;
            }
            BuffResolutionStrategy::TeammateOutputMirroring { cap_pct, .. }
            | BuffResolutionStrategy::OrderLimitScaling { cap_pct, .. } => total += cap_pct,
            strategy @ BuffResolutionStrategy::FacilityCountScaling { .. } => {
                let ctx = EvalContext {
                    facility_counts,
                    total_dorm_levels,
                    room_teammates: Vec::new(),
                };
                total += evaluate_buff(strategy, &ctx);
            }
            _ => {}
        }
    }
    total
}

/// Generate all non-empty combinations of `pool` with size 1..=`max_size`.
fn combinations(pool: &[String], max_size: usize) -> Vec<Vec<String>> {
    let mut result = Vec::new();
    let max_size = max_size.min(pool.len());
    let mut current = Vec::new();
    for size in 1..=max_size {
        combos_of_size(pool, size, 0, &mut current, &mut result);
    }
    result
}

fn combos_of_size(
    pool: &[String],
    size: usize,
    start: usize,
    current: &mut Vec<String>,
    out: &mut Vec<Vec<String>>,
) {
    if current.len() == size {
        out.push(current.clone());
        return;
    }
    // Stop early if not enough remaining elements to reach `size`.
    let need = size - current.len();
    if pool.len() - start < need {
        return;
    }
    for i in start..pool.len() {
        current.push(pool[i].clone());
        combos_of_size(pool, size, i + 1, current, out);
        current.pop();
    }
}

/// Score only `FacilityCountScaling` buffs for an operator (used in automation rooms).
fn score_operator_facility_only(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
) -> f64 {
    let ctx = EvalContext {
        facility_counts,
        total_dorm_levels,
        room_teammates: Vec::new(),
    };

    let mut total = 0.0;
    for strategy in applicable_strategies(op, room_type, formula_type, registry, building_data) {
        if matches!(
            strategy,
            BuffResolutionStrategy::FacilityCountScaling { .. }
        ) {
            total += evaluate_buff(strategy, &ctx);
        }
    }
    total
}

/// Compute only the `DirectEfficiency`-style contribution for an operator,
/// used as a teammate's `direct_efficiency` (feeds `TeammateOutputMirroring`).
/// Conditional efficiency is counted only when its required teammate is present.
fn compute_direct_efficiency(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    present: &HashSet<String>,
) -> f64 {
    let mut total = 0.0;
    for strategy in applicable_strategies(op, room_type, formula_type, registry, building_data) {
        match strategy {
            BuffResolutionStrategy::DirectEfficiency { value } => total += value,
            BuffResolutionStrategy::EfficiencyWithOrderLimit { efficiency, .. } => {
                total += efficiency;
            }
            BuffResolutionStrategy::ConditionalOnTeammate {
                required_char_id,
                base_efficiency,
                efficiency,
                ..
            } => {
                total += base_efficiency;
                if let Some(req) = required_char_id
                    && present.contains(req)
                {
                    total += efficiency;
                }
            }
            // Faction-conditional: only the always-on base counts here - the gated
            // bonus needs teammate faction tags this mirroring source doesn't resolve.
            BuffResolutionStrategy::ConditionalOnFaction {
                base_efficiency, ..
            } => {
                total += base_efficiency;
            }
            _ => {}
        }
    }
    total
}

/// Compute the total order limit contribution for an operator in a room.
/// Conditional order limit counts only when its required teammate is present.
fn compute_order_limit(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    present: &HashSet<String>,
) -> i32 {
    let mut total: i32 = 0;
    for strategy in applicable_strategies(op, room_type, formula_type, registry, building_data) {
        match strategy {
            BuffResolutionStrategy::EfficiencyWithOrderLimit { order_limit, .. } => {
                total += order_limit;
            }
            BuffResolutionStrategy::ConditionalOnTeammate {
                required_char_id: Some(req),
                order_limit,
                ..
            } if present.contains(req) => total += order_limit,
            _ => {}
        }
    }
    total
}

/// Room efficiency resolved against the full team (conditionals, teammate- and
/// faction-scaling, output mirroring, order-limit scaling). Returns
/// `(speed_pct, value_pct)`: order-acquisition SPEED (the productivity % the game
/// shows) and order VALUE (LMD per order, Proviso etc.), kept separate so value
/// doesn't inflate the displayed efficiency.
#[allow(clippy::too_many_arguments)]
fn compute_team_efficiency(
    member_ids: &[String],
    room_type: &str,
    formula_type: Option<&str>,
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    morale_drains: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
) -> (f64, f64) {
    // Resolve members once via the index (O(1) each) instead of scanning.
    let members: Vec<&OperatorBaseProfile> = member_ids
        .iter()
        .filter_map(|id| op_index.get(id.as_str()).copied())
        .collect();
    let present: HashSet<String> = members.iter().map(|o| o.char_id.clone()).collect();

    // Collect any skill-type converters in the room (Highmore-style). They
    // reclassify teammates' tags so e.g. Rhine operators count for a
    // Standardization scaler.
    let converters: Vec<(&[String], &str)> = members
        .iter()
        .flat_map(|op| {
            applicable_strategies(op, room_type, formula_type, registry, building_data).filter_map(
                |s| match s {
                    BuffResolutionStrategy::SkillTypeConversion {
                        from_tokens,
                        to_token,
                    } => Some((from_tokens.as_slice(), to_token.as_str())),
                    _ => None,
                },
            )
        })
        .collect();

    let teammates: Vec<TeammateInfo> = members
        .iter()
        .map(|op| {
            // Precomputed tags; only clone (no recomputation) and augment for converters.
            let mut match_tags = op.match_tags.clone();
            for (from, to) in &converters {
                if match_tags.iter().any(|t| from.contains(t))
                    && !match_tags.iter().any(|t| t == to)
                {
                    match_tags.push((*to).to_string());
                }
            }
            TeammateInfo {
                char_id: op.char_id.clone(),
                match_tags,
                buff_ids: op.available_buffs.clone(),
                direct_efficiency: compute_direct_efficiency(
                    op,
                    room_type,
                    formula_type,
                    registry,
                    building_data,
                    &present,
                ),
                order_limit_contribution: compute_order_limit(
                    op,
                    room_type,
                    formula_type,
                    registry,
                    building_data,
                    &present,
                ),
            }
        })
        .collect();

    // Shamare-type operators zero out every teammate's output. If one is in the
    // room, only the nullifier(s) contribute (their own value still scales with
    // the number of zeroed teammates).
    let has_nullifier = members
        .iter()
        .any(|op| op_is_nullifier(op, room_type, formula_type, registry, building_data));

    let mut speed = 0.0;
    let mut value = 0.0;

    for (i, op) in members.iter().enumerate() {
        // A nullifier (Shamare) zeroes teammates' *speed/efficiency*, but their
        // *order-value* buffs survive - this is the Shamare + Proviso + Tequila
        // synergy. So a nullified teammate still contributes its OrderValue.
        let nullified =
            has_nullifier && !op_is_nullifier(op, room_type, formula_type, registry, building_data);

        let others: Vec<&TeammateInfo> = teammates
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, t)| t)
            .collect();

        let ctx = EvalContext {
            facility_counts,
            total_dorm_levels,
            room_teammates: others,
        };

        for strategy in applicable_strategies(op, room_type, formula_type, registry, building_data)
        {
            // Order VALUE (LMD/order) is tracked separately and survives a
            // nullifier; everything else is order SPEED, which the nullifier zeroes.
            if let BuffResolutionStrategy::OrderValue { estimated_pct } = strategy {
                value += estimated_pct;
            } else if !nullified {
                speed += evaluate_buff(strategy, &ctx);
            }
        }
    }
    // Faction-gated Control Center bonuses: order SPEED, but only for the posts
    // whose team actually satisfies the faction condition.
    speed += cc_conditions
        .iter()
        .map(|c| c.contribution(room_type, &members))
        .sum::<f64>();

    let _ = morale_drains;
    (speed, value)
}

/// The optimizer's per-room objective: a room's LMD-equivalent output, with order
/// VALUE multiplying trading LMD (not adding to speed). Within a room the base
/// rate is constant, so maximizing this multiplier maximizes the room's yield.
fn room_search_score(room_type: &str, speed_pct: f64, value_pct: f64) -> f64 {
    let speed_mult = 1.0 + speed_pct / 100.0;
    if room_type == "TRADING" {
        speed_mult * (1.0 + value_pct / 100.0)
    } else {
        speed_mult
    }
}

/// Total realized daily output of a production assignment, as a single
/// LMD-equivalent value. Each room's efficiency is soft-capped (per-room
/// throughput ceiling) and converted to its resource yield; the gold→trade loop
/// is then coupled so LMD = min(gold made, gold sold) × 500 and EXP adds at 1:1.
fn assignment_value(rooms: &[RoomAssignment]) -> f64 {
    let mut flows = crate::core::grade::base::yield_model::BaseFlows::default();
    for r in rooms {
        let speed = room_value(r.total_efficiency, &r.room_type);
        flows.add_room(
            &r.room_type,
            r.formula_type.as_deref(),
            r.level,
            speed,
            r.order_value,
        );
    }
    flows.total_value()
}

/// Soft-capped value of a room's raw efficiency. Production rooms have a
/// practical throughput ceiling (orders/credits accumulate only up to the order
/// limit between collections), so efficiency stacked far past that point is
/// worth progressively less. Modeling this is what makes spreading operators
/// across multiple rooms beat dumping every strong operator into one.
fn room_value(raw_efficiency: f64, room_type: &str) -> f64 {
    // Beyond this much total efficiency, extra % is discounted.
    let soft_cap = match room_type {
        "TRADING" | "MANUFACTURE" => 200.0,
        _ => return raw_efficiency,
    };
    if raw_efficiency <= soft_cap {
        raw_efficiency
    } else {
        soft_cap + (raw_efficiency - soft_cap) * 0.5
    }
}

/// Balance operators across rooms that share the same type and formula by
/// swapping pairs whenever it raises the soft-capped objective. With multiple
/// trading posts / gold factories this spreads efficiency instead of stacking it
/// all in one room (diminishing returns). Only same-type/formula rooms are
/// considered so buff applicability is identical on both sides of a swap.
#[allow(clippy::too_many_arguments)]
fn rebalance_rooms(
    rooms: &mut [RoomAssignment],
    operators: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    global_bonuses: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
) {
    let op_index = build_op_index(operators);
    // (speed incl. global, value) for a room's operator set.
    let recompute = |room_type: &str, formula: Option<&str>, ops: &[String]| -> (f64, f64) {
        let (speed, value) = compute_team_efficiency(
            ops,
            room_type,
            formula,
            &op_index,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
            morale_drains,
            cc_conditions,
        );
        (
            speed + *global_bonuses.get(room_type).unwrap_or(&0.0),
            value,
        )
    };
    // Balancing objective: soft-capped speed (diminishing returns) × order value.
    let obj = |room_type: &str, speed: f64, value: f64| -> f64 {
        room_search_score(room_type, room_value(speed, room_type), value)
    };

    // Up to a few passes; each pass tries all same-type/formula room pairs.
    for _pass in 0..4 {
        let mut improved = false;
        for a in 0..rooms.len() {
            for b in (a + 1)..rooms.len() {
                if rooms[a].room_type != rooms[b].room_type
                    || rooms[a].formula_type != rooms[b].formula_type
                {
                    continue;
                }
                let base_obj = obj(
                    &rooms[a].room_type,
                    rooms[a].total_efficiency,
                    rooms[a].order_value,
                ) + obj(
                    &rooms[b].room_type,
                    rooms[b].total_efficiency,
                    rooms[b].order_value,
                );

                let mut best_delta = 0.0;
                let mut best_swap: Option<(usize, usize)> = None;

                for ia in 0..rooms[a].operators.len() {
                    for ib in 0..rooms[b].operators.len() {
                        let mut ops_a = rooms[a].operators.clone();
                        let mut ops_b = rooms[b].operators.clone();
                        std::mem::swap(&mut ops_a[ia], &mut ops_b[ib]);

                        let (sa, va) = recompute(
                            &rooms[a].room_type,
                            rooms[a].formula_type.as_deref(),
                            &ops_a,
                        );
                        let (sb, vb) = recompute(
                            &rooms[b].room_type,
                            rooms[b].formula_type.as_deref(),
                            &ops_b,
                        );
                        let new_obj =
                            obj(&rooms[a].room_type, sa, va) + obj(&rooms[b].room_type, sb, vb);
                        let delta = new_obj - base_obj;
                        if delta > best_delta + f64::EPSILON {
                            best_delta = delta;
                            best_swap = Some((ia, ib));
                        }
                    }
                }

                if let Some((ia, ib)) = best_swap {
                    let tmp = rooms[a].operators[ia].clone();
                    rooms[a].operators[ia] = rooms[b].operators[ib].clone();
                    rooms[b].operators[ib] = tmp;
                    let (sa, va) = recompute(
                        &rooms[a].room_type,
                        rooms[a].formula_type.as_deref(),
                        &rooms[a].operators,
                    );
                    let (sb, vb) = recompute(
                        &rooms[b].room_type,
                        rooms[b].formula_type.as_deref(),
                        &rooms[b].operators,
                    );
                    rooms[a].total_efficiency = sa;
                    rooms[a].order_value = va;
                    rooms[b].total_efficiency = sb;
                    rooms[b].order_value = vb;
                    improved = true;
                }
            }
        }
        if !improved {
            break;
        }
    }
}
