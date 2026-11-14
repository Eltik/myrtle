use std::collections::{HashMap, HashSet};

use super::build_op_index;
use super::facilities::max_stationed_for_room;
use super::morale::{cc_global_morale_recovery, cc_room_morale_recovery, reserve_cc_morale_seats};
use super::teams::{applicable_strategies, op_surviving_order_value};
use crate::{
    registry::BuffResolutionStrategy,
    rotation::team_select::PlannedGroup,
    scoring::clause::clauses_from_strategy,
    types::{OperatorBaseProfile, RoomAssignment, UserBuilding, UserRoom},
    util::buff_family,
};
use myrtle_gamedata_raw::tables::building::{Buff, BuildingDataFile};

/// A Control Center assignment plus the production bonuses granted
pub struct ControlCenter {
    pub(crate) operators: Vec<String>,
    slot_id: Option<String>,
    level: i32,
    /// Total global bonus % across all targeted production room types
    total_global_pct: f64,
}

impl ControlCenter {
    pub(crate) fn into_room(self) -> Option<RoomAssignment> {
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

/// Append unused operators to `slots` until it reaches `max_slots`.
///
/// Fillers add no value, picks lowest *opportunity cost*: fewest base skills for
/// OTHER rooms, keeping operators with useful Dormitory/Training/etc. skills (e.g. Schwarz)
/// for the rooms they help.
pub fn fill_remaining_slots(
    slots: &mut Vec<String>,
    max_slots: i32,
    room_type: &str,
    operators: &[OperatorBaseProfile],
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    assigned: &mut HashSet<String>,
) -> Vec<String> {
    let mut added = Vec::new();
    while (slots.len() as i32) < max_slots {
        let coverage = if room_type == "CONTROL" {
            cc_seated_coverage(slots, operators, building_data, registry)
        } else {
            HashMap::new()
        };
        let pick = operators
            .iter()
            .filter(|op| !assigned.contains(&op.char_id))
            .min_by(|a, b| {
                let ka = other_room_skill_count(a, room_type, building_data);
                let kb = other_room_skill_count(b, room_type, building_data);
                if room_type != "CONTROL" {
                    return ka.cmp(&kb);
                }
                let ma = cc_global_morale_recovery(a, registry, building_data);
                let mb = cc_global_morale_recovery(b, registry, building_data);
                let ra = cc_room_morale_recovery(a, registry, building_data);
                let rb = cc_room_morale_recovery(b, registry, building_data);
                let (fa, ca) = cc_spare_seat_split(a, building_data, registry, &coverage);
                let (fb, cb) = cc_spare_seat_split(b, building_data, registry, &coverage);
                let va = fa + ca + ma + ra;
                let vb = fb + cb + mb + rb;
                let desc = |x: f64, y: f64| y.partial_cmp(&x).unwrap_or(std::cmp::Ordering::Equal);
                (va <= 0.0)
                    .cmp(&(vb <= 0.0))
                    .then_with(|| ka.cmp(&kb))
                    .then_with(|| desc(ma, mb))
                    .then_with(|| desc(ra, rb))
                    .then_with(|| desc(fa, fb))
                    .then_with(|| desc(ca, cb))
            });
        let Some(op) = pick else { break };
        assigned.insert(op.char_id.clone());
        slots.push(op.char_id.clone());
        added.push(op.char_id.clone());
    }
    added
}

#[must_use]
pub fn cc_non_production_effects(
    crew: &[String],
    operators: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
    team_rooms: &[RoomAssignment],
) -> Vec<(String, f64)> {
    let mut best: HashMap<String, f64> = HashMap::new();
    let mut record = |room: &str, value: f64| {
        let e = best.entry(room.to_string()).or_insert(0.0);
        if value > *e {
            *e = value;
        }
    };
    for id in crew {
        let Some(op) = operators.iter().find(|o| &o.char_id == id) else {
            continue;
        };
        for b in &op.available_buffs {
            match registry.get(b) {
                Some(BuffResolutionStrategy::ControlNonProduction {
                    target_room,
                    value,
                    same_room_gate,
                }) => {
                    let live = match same_room_gate {
                        Some(chars) => chars.iter().any(|c| crew.contains(c)),
                        None => true,
                    };
                    if live {
                        record(target_room, *value);
                    }
                }
                Some(BuffResolutionStrategy::NamedCharRoomGrants { grants }) => {
                    for g in grants.iter().filter(|g| g.nonprod_pct != 0.0) {
                        let live = team_rooms.iter().any(|r| {
                            r.room_type == g.target_room && r.operators.contains(&g.char_id)
                        });
                        if live {
                            record(&g.target_room, g.nonprod_pct);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut out: Vec<(String, f64)> = best.into_iter().collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[must_use]
pub fn cc_seated_coverage(
    seated: &[String],
    operators: &[OperatorBaseProfile],
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> HashMap<crate::scoring::clause::NonProdKind, f64> {
    use crate::scoring::clause::{ClauseKind, Metric};
    let mut best = HashMap::new();
    for op in seated
        .iter()
        .filter_map(|id| operators.iter().find(|o| &o.char_id == id))
    {
        for b in &op.available_buffs {
            let Some(buff) = building_data.buffs.get(b) else {
                continue;
            };
            if buff.room_type != "CONTROL" || cc_buff_stacks(buff) {
                continue;
            }
            let Some(strategy) = registry.get(b) else {
                continue;
            };
            for c in clauses_from_strategy(b, buff, strategy) {
                if let (ClauseKind::SelfValue, Metric::NonProduction(kind)) = (&c.kind, &c.metric) {
                    let slot = best.entry(*kind).or_insert(0.0_f64);
                    *slot = slot.max(c.value);
                }
            }
        }
    }
    best
}

#[must_use]
pub fn cc_spare_seat_value(
    op: &OperatorBaseProfile,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    coverage: &HashMap<crate::scoring::clause::NonProdKind, f64>,
) -> f64 {
    let (facility, clue) = cc_spare_seat_split(op, building_data, registry, coverage);
    facility + clue
}

#[must_use]
pub fn cc_spare_seat_split(
    op: &OperatorBaseProfile,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    coverage: &HashMap<crate::scoring::clause::NonProdKind, f64>,
) -> (f64, f64) {
    use crate::scoring::clause::{ClauseKind, Metric, NonProdKind};
    op.available_buffs
        .iter()
        .filter_map(|b| {
            let buff = building_data.buffs.get(b)?;
            (buff.room_type == "CONTROL").then_some(())?;
            let strategy = registry.get(b)?;
            let stacks = cc_buff_stacks(buff);
            Some(
                clauses_from_strategy(b, buff, strategy)
                    .iter()
                    .filter(|c| matches!(c.kind, ClauseKind::SelfValue))
                    .fold((0.0_f64, 0.0_f64), |(facility, clue), c| {
                        let Metric::NonProduction(kind) = &c.metric else {
                            return (facility, clue);
                        };
                        let value = if stacks {
                            c.value
                        } else {
                            (c.value - coverage.get(kind).copied().unwrap_or(0.0)).max(0.0)
                        };
                        match kind {
                            NonProdKind::HrContact => (facility + value * 2.0, clue),
                            NonProdKind::Training => (facility + value, clue),
                            NonProdKind::ClueSearch => (facility, clue + value),
                            _ => (facility, clue),
                        }
                    }),
            )
        })
        .fold(
            (0.0, 0.0),
            |(bf, bc), (f, c)| {
                if f + c > bf + bc { (f, c) } else { (bf, bc) }
            },
        )
}

/// Opportunity cost of parking `op` in a room as a FILLER
pub fn padding_cost(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    team_nullifies: bool,
    team_automation: bool,
) -> usize {
    let mut cost = other_room_skill_count(op, room_type, building_data);
    let has_room_skill =
        applicable_strategies(op, room_type, formula_type, registry, building_data).count() > 0;
    if has_room_skill {
        let survives = if team_nullifies {
            op_surviving_order_value(op, room_type, formula_type, registry, building_data) > 0.0
        } else if team_automation {
            applicable_strategies(op, room_type, formula_type, registry, building_data)
                .any(|s| matches!(s, BuffResolutionStrategy::FacilityCountScaling { .. }))
        } else {
            true
        };
        if !survives {
            cost += 100;
        }
    }
    cost
}

/// How many *other* room types this operator has base skills for
pub fn other_room_skill_count(
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

#[derive(Clone)]
pub struct CcBonus {
    pub(crate) room: String,
    pub(crate) family: String,
    pub(crate) bonus: f64,
    /// True when the buff has no non-stacking clause
    pub(crate) stacks: bool,
    pub(crate) conditional: Option<CcCondition>,
    pub(crate) crew_gate: Option<CrewGate>,
}

#[derive(Clone)]
pub struct CrewGate {
    pub(crate) tag: String,
    pub(crate) min_others: usize,
    pub(crate) bonus_pct: f64,
}

impl CrewGate {
    /// Met when `min_others` crew members other than `owner` carry tag
    pub(crate) fn met(&self, owner: &str, crew: &[(String, Vec<String>)]) -> bool {
        crew.iter()
            .filter(|(id, tags)| id != owner && tags.contains(&self.tag))
            .count()
            >= self.min_others
    }
}

#[derive(Clone)]
pub struct CcCondition {
    pub(crate) target_room: String,
    pub(crate) faction_token: String,
    pub(crate) required_count: usize,
    pub(crate) per_operator: bool,
    pub(crate) bonus_pct: f64,
    pub(crate) order_limit: f64,
    pub(crate) formula_bonuses: Vec<(String, f64)>,
}

pub fn cc_token_matches(op: &OperatorBaseProfile, token: &str) -> bool {
    op.char_id == token || op.match_tags.iter().any(|t| t == token)
}

impl CcCondition {
    pub(crate) fn fingerprint(&self) -> String {
        let formulas: Vec<String> = self
            .formula_bonuses
            .iter()
            .map(|(f, pct)| format!("{f}={}", pct.to_bits()))
            .collect();
        format!(
            "{}|{}|{}|{}|{}|{}|{}",
            self.target_room,
            self.faction_token,
            self.required_count,
            self.per_operator,
            self.bonus_pct.to_bits(),
            self.order_limit.to_bits(),
            formulas.join(",")
        )
    }

    pub(crate) fn bonus_for(&self, formula: Option<&str>) -> f64 {
        if self.formula_bonuses.is_empty() {
            return self.bonus_pct;
        }
        formula
            .and_then(|f| self.formula_bonuses.iter().find(|(k, _)| k == f))
            .map_or(0.0, |(_, v)| *v)
    }

    pub(crate) fn contribution(
        &self,
        room_type: &str,
        team: &[&OperatorBaseProfile],
        formula: Option<&str>,
    ) -> f64 {
        if self.target_room != room_type {
            return 0.0;
        }
        let bonus = self.bonus_for(formula);
        let count = team
            .iter()
            .filter(|op| cc_token_matches(op, &self.faction_token))
            .count();
        if self.per_operator {
            bonus * count as f64
        } else if count >= self.required_count {
            bonus
        } else {
            0.0
        }
    }

    pub(crate) fn capacity_contribution(
        &self,
        room_type: &str,
        team: &[&OperatorBaseProfile],
    ) -> f64 {
        if self.order_limit == 0.0 || self.target_room != room_type {
            return 0.0;
        }
        let count = team
            .iter()
            .filter(|op| cc_token_matches(op, &self.faction_token))
            .count();
        if self.per_operator {
            self.order_limit * count as f64
        } else if count >= self.required_count {
            self.order_limit
        } else {
            0.0
        }
    }
}

/// Folds Control Center bonuses together under the non-stacking rule
#[derive(Default)]
pub struct CcBonusAccumulator {
    /// Clause-bearing buffs
    best: HashMap<(String, String), f64>,
    /// Clause-less buffs
    stacked: HashMap<String, f64>,
    conditions: HashMap<(String, String), CcCondition>,
    cond_families: HashSet<(String, String)>,
    /// `(char_id, match tags)`
    crew: Vec<(String, Vec<String>)>,
    gated: Vec<(String, CcBonus)>,
}

impl CcBonusAccumulator {
    /// The marginal flat gain from adding `bonuses`
    pub(crate) fn marginal(&self, bonuses: &[CcBonus]) -> f64 {
        bonuses
            .iter()
            .filter_map(|b| self.resolve_gate(b, ""))
            .map(|b| {
                if b.stacks && b.conditional.is_none() {
                    b.bonus
                } else {
                    let cur = self
                        .best
                        .get(&(b.room.clone(), b.family.clone()))
                        .copied()
                        .unwrap_or(0.0);
                    (b.bonus - cur).max(0.0)
                }
            })
            .sum()
    }

    fn resolve_gate(&self, b: &CcBonus, owner: &str) -> Option<CcBonus> {
        match &b.crew_gate {
            None => Some(b.clone()),
            Some(gate) if gate.met(owner, &self.crew) => Some(CcBonus {
                bonus: gate.bonus_pct,
                crew_gate: None,
                ..b.clone()
            }),
            Some(_) => None,
        }
    }

    pub(crate) fn add_from(&mut self, op: &OperatorBaseProfile, bonuses: &[CcBonus]) {
        self.crew.push((op.char_id.clone(), op.match_tags.clone()));
        for b in bonuses {
            if b.crew_gate.is_some() {
                self.gated.push((op.char_id.clone(), b.clone()));
            } else {
                self.add(std::slice::from_ref(b));
            }
        }
    }

    pub(crate) fn add(&mut self, bonuses: &[CcBonus]) {
        for b in bonuses {
            if b.crew_gate.is_some() {
                continue;
            }
            if b.stacks && b.conditional.is_none() {
                *self.stacked.entry(b.room.clone()).or_insert(0.0) += b.bonus;
                continue;
            }
            let key = (b.room.clone(), b.family.clone());
            let slot = self.best.entry(key.clone()).or_insert(0.0);
            *slot = slot.max(b.bonus);
            if let Some(cond) = &b.conditional {
                self.cond_families.insert(key.clone());
                self.conditions
                    .entry(key)
                    .and_modify(|c| {
                        if (cond.bonus_pct, cond.order_limit) > (c.bonus_pct, c.order_limit) {
                            *c = cond.clone();
                        }
                    })
                    .or_insert_with(|| cond.clone());
            }
        }
    }

    /// `(flat per-room totals, faction-gated conditions)`
    pub(crate) fn finish(mut self) -> (HashMap<String, f64>, Vec<CcCondition>) {
        let gated = std::mem::take(&mut self.gated);
        for (owner, b) in &gated {
            if let Some(resolved) = self.resolve_gate(b, owner) {
                self.add(std::slice::from_ref(&resolved));
            }
        }
        let mut global = self.stacked;
        for ((room, family), bonus) in &self.best {
            if !self.cond_families.contains(&(room.clone(), family.clone())) {
                *global.entry(room.clone()).or_insert(0.0) += bonus;
            }
        }
        (global, self.conditions.into_values().collect())
    }
}

pub fn cc_bonus_for(
    buff_id: &str,
    buff: &Buff,
    strategy: Option<&BuffResolutionStrategy>,
) -> Option<CcBonus> {
    if buff.room_type != "CONTROL" {
        return None;
    }
    match strategy {
        Some(BuffResolutionStrategy::GlobalEffect {
            target_room,
            bonus_pct,
        }) => {
            let stacks = cc_buff_stacks(buff);
            Some(CcBonus {
                room: target_room.clone(),
                family: format!("{target_room}#global"),
                bonus: *bonus_pct,
                stacks,
                conditional: None,
                crew_gate: None,
            })
        }
        Some(BuffResolutionStrategy::TagBased {
            target_room,
            bonus_pct,
            ..
        }) => Some(CcBonus {
            room: target_room.clone(),
            family: buff_family(buff_id).to_string(),
            bonus: bonus_pct * 0.5,
            stacks: true,
            conditional: None,
            crew_gate: None,
        }),
        Some(BuffResolutionStrategy::ConditionalGlobalEffect {
            target_room,
            faction_token,
            required_count,
            per_operator,
            bonus_pct,
            order_limit,
            formula_bonuses,
        }) => Some(CcBonus {
            room: target_room.clone(),
            family: buff_family(buff_id).to_string(),
            bonus: bonus_pct * 0.5,
            stacks: false,
            conditional: Some(CcCondition {
                target_room: target_room.clone(),
                faction_token: faction_token.clone(),
                required_count: *required_count,
                per_operator: *per_operator,
                bonus_pct: *bonus_pct,
                order_limit: f64::from(*order_limit),
                formula_bonuses: formula_bonuses.clone(),
            }),
            crew_gate: None,
        }),
        Some(BuffResolutionStrategy::GlobalEffectWithCrewTag {
            target_room,
            bonus_pct,
            tag,
            min_others,
        }) => Some(CcBonus {
            room: target_room.clone(),
            family: format!("{target_room}#global"),
            bonus: bonus_pct * 0.5,
            stacks: cc_buff_stacks(buff),
            conditional: None,
            crew_gate: Some(CrewGate {
                tag: tag.clone(),
                min_others: *min_others,
                bonus_pct: *bonus_pct,
            }),
        }),
        Some(BuffResolutionStrategy::NamedCharRoomGrants { grants }) => grants
            .iter()
            .find(|g| g.order_limit != 0.0)
            .map(|g| CcBonus {
                room: g.target_room.clone(),
                family: buff_family(buff_id).to_string(),
                bonus: 0.0,
                stacks: false,
                conditional: Some(CcCondition {
                    target_room: g.target_room.clone(),
                    faction_token: g.char_id.clone(),
                    required_count: 1,
                    per_operator: false,
                    bonus_pct: 0.0,
                    order_limit: g.order_limit,
                    formula_bonuses: Vec::new(),
                }),
                crew_gate: None,
            }),
        _ => None,
    }
}

pub fn cc_bonuses(
    op: &OperatorBaseProfile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> Vec<CcBonus> {
    op.available_buffs
        .iter()
        .filter_map(|buff_id| {
            let buff = building_data.buffs.get(buff_id)?;
            cc_bonus_for(buff_id, buff, registry.get(buff_id))
        })
        .collect()
}

pub fn cc_marginal_over(
    crew: &[String],
    candidate_id: &str,
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> f64 {
    let mut acc = CcBonusAccumulator::default();
    for id in crew {
        if id == candidate_id {
            continue;
        }
        if let Some(op) = op_index.get(id.as_str()) {
            acc.add_from(op, &cc_bonuses(op, registry, building_data));
        }
    }
    op_index.get(candidate_id).map_or(0.0, |op| {
        acc.marginal(&cc_bonuses(op, registry, building_data))
    })
}

pub(super) fn cc_condition_feasible(
    cond: &CcCondition,
    operators: &[OperatorBaseProfile],
    building_data: &BuildingDataFile,
) -> bool {
    let eligible = operators
        .iter()
        .filter(|op| cc_token_matches(op, &cond.faction_token))
        .filter(|op| {
            op.char_id == cond.faction_token
                || op.available_buffs.iter().any(|b| {
                    building_data
                        .buffs
                        .get(b)
                        .is_some_and(|buff| buff.room_type == cond.target_room)
                })
        })
        .count();
    eligible >= cond.required_count
}

pub fn cc_condition_fires(
    cond: &CcCondition,
    rooms: &[RoomAssignment],
    op_index: &HashMap<&str, &OperatorBaseProfile>,
) -> bool {
    rooms
        .iter()
        .filter(|r| r.room_type == cond.target_room)
        .any(|r| {
            let count = r
                .operators
                .iter()
                .filter_map(|id| op_index.get(id.as_str()))
                .filter(|op| cc_token_matches(op, &cond.faction_token))
                .count();
            if cond.per_operator {
                count >= 1
            } else {
                count >= cond.required_count
            }
        })
}

pub fn cc_condition_reach(
    cond: &CcCondition,
    rooms: &[RoomAssignment],
    op_index: &HashMap<&str, &OperatorBaseProfile>,
) -> usize {
    rooms
        .iter()
        .filter(|r| r.room_type == cond.target_room)
        .map(|r| {
            let count = r
                .operators
                .iter()
                .filter_map(|id| op_index.get(id.as_str()))
                .filter(|op| cc_token_matches(op, &cond.faction_token))
                .count();
            if cond.per_operator {
                count
            } else {
                usize::from(count >= cond.required_count)
            }
        })
        .sum()
}

pub fn cc_op_is_dead(
    op: &OperatorBaseProfile,
    rooms: &[RoomAssignment],
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> bool {
    let bonuses = cc_bonuses(op, registry, building_data);
    if bonuses.is_empty() {
        return false; // no CC value
    }
    !bonuses.iter().any(|b| match &b.conditional {
        None => true, // an unconditional bonus always lands
        Some(c) => cc_condition_fires(c, rooms, op_index),
    })
}

pub(super) fn cc_buff_stacks(buff: &Buff) -> bool {
    let d = buff.description.to_lowercase();
    !(d.contains("only the most effective")
        || d.contains("strongest effect of this type")
        || d.contains("strongest effect of the same type")
        || d.contains("only the strongest"))
}

/// Assign operators to the Control Center to maximize global production bonuses.
/// Returns the CC assignment and the global bonuses map (`room_type` -> total bonus %).
pub fn assign_control_center(
    operators: &[OperatorBaseProfile],
    control_room: Option<&UserRoom>,
    max_slots: i32,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    exclude: &HashSet<String>,
    forced: &[String],
) -> (ControlCenter, HashMap<String, f64>, Vec<CcCondition>) {
    let candidates: Vec<(&OperatorBaseProfile, Vec<CcBonus>)> = operators
        .iter()
        .filter(|op| !exclude.contains(&op.char_id) && !forced.contains(&op.char_id))
        .map(|op| {
            let bonuses = cc_bonuses(op, registry, building_data)
                .into_iter()
                .filter(|b| {
                    b.conditional
                        .as_ref()
                        .is_none_or(|c| cc_condition_feasible(c, operators, building_data))
                })
                .collect::<Vec<_>>();
            (op, bonuses)
        })
        .filter(|(_, b)| !b.is_empty())
        .collect();

    let mut seed = CcBonusAccumulator::default();
    for id in forced {
        if let Some(op) = operators.iter().find(|o| &o.char_id == id) {
            let bonuses: Vec<CcBonus> = cc_bonuses(op, registry, building_data)
                .into_iter()
                .filter(|b| {
                    b.conditional
                        .as_ref()
                        .is_none_or(|c| cc_condition_feasible(c, operators, building_data))
                })
                .collect();
            seed.add_from(op, &bonuses);
        }
    }

    let (cc_assigned, acc) = greedy_cc_fill(seed, &candidates, max_slots);

    let (global_bonuses, conditions) = acc.finish();
    let cc = ControlCenter {
        operators: cc_assigned,
        slot_id: control_room.map(|r| r.slot_id.clone()),
        level: control_room.map_or(1, |r| r.level),
        total_global_pct: global_bonuses.values().sum(),
    };
    let mut conditions = conditions;
    conditions.extend(seat_grant_conditions(registry));
    (cc, global_bonuses, conditions)
}

pub fn seat_grant_conditions(
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> Vec<CcCondition> {
    let mut out: Vec<CcCondition> = registry
        .values()
        .filter_map(|s| match s {
            BuffResolutionStrategy::NamedTargetRoomBoost {
                target_char_id,
                target_room,
                bonus_pct,
                active: true,
                ..
            } => Some(CcCondition {
                target_room: target_room.clone(),
                faction_token: target_char_id.clone(),
                required_count: 1,
                per_operator: true,
                bonus_pct: *bonus_pct,
                order_limit: 0.0,
                formula_bonuses: Vec::new(),
            }),
            _ => None,
        })
        .collect();
    out.sort_by(|a, b| a.faction_token.cmp(&b.faction_token));
    out
}

pub struct RotationCcPlan {
    pub(crate) slot_id: Option<String>,
    pub(crate) control_slots: i32,
    pub(crate) squad1: Vec<String>,
    pub(crate) conditions: Vec<CcCondition>,
    pub(crate) global_bonuses: HashMap<String, f64>,
}

impl RotationCcPlan {
    pub(crate) fn squad2(
        &self,
        operators: &[OperatorBaseProfile],
        building_data: &BuildingDataFile,
        registry: &HashMap<String, BuffResolutionStrategy>,
        exclude: &HashSet<String>,
        team_rooms: &[RoomAssignment],
    ) -> Vec<String> {
        let op_index = build_op_index(operators);
        let mut exclude = exclude.clone();
        exclude.extend(self.squad1.iter().cloned());
        loop {
            let (mut cc, _, _) = assign_control_center(
                operators,
                None,
                self.control_slots,
                registry,
                building_data,
                &exclude,
                &[],
            );
            let newly_dead: Vec<String> = cc
                .operators
                .iter()
                .filter(|id| {
                    op_index.get(id.as_str()).is_some_and(|op| {
                        cc_op_is_dead(op, team_rooms, &op_index, registry, building_data)
                    })
                })
                .cloned()
                .collect();
            if !newly_dead.is_empty() {
                exclude.extend(newly_dead);
                continue;
            }
            let mut assigned = exclude;
            assigned.extend(cc.operators.iter().cloned());
            reserve_cc_morale_seats(
                &mut cc,
                self.control_slots,
                operators,
                &mut assigned,
                registry,
                building_data,
            );
            return cc.operators;
        }
    }
}

pub fn rotation_cc_plan<F>(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    forced: &[String],
    mut plan_teams: F,
) -> (RotationCcPlan, Vec<PlannedGroup>)
where
    F: FnMut(&HashSet<String>, &HashMap<String, f64>, &[CcCondition]) -> Vec<PlannedGroup>,
{
    let op_index = build_op_index(operators);
    let control_room = building.rooms.iter().find(|r| r.room_type == "CONTROL");
    let control_slots = if control_room.is_some() {
        max_stationed_for_room(building, building_data, "CONTROL")
    } else {
        0
    };
    let mut exclude_cc: HashSet<String> = HashSet::new();
    loop {
        let (mut cc_room, global_bonuses, cc_conditions) = assign_control_center(
            operators,
            control_room,
            control_slots,
            registry,
            building_data,
            &exclude_cc,
            forced,
        );
        let mut assigned: HashSet<String> = cc_room.operators.iter().cloned().collect();
        reserve_cc_morale_seats(
            &mut cc_room,
            control_slots,
            operators,
            &mut assigned,
            registry,
            building_data,
        );
        let groups = plan_teams(&assigned, &global_bonuses, &cc_conditions);
        let team_rooms: Vec<RoomAssignment> = groups
            .iter()
            .flat_map(|g| {
                g.teams
                    .iter()
                    .filter(|t| !t.ops.is_empty())
                    .map(|t| RoomAssignment {
                        room_type: g.room_type.clone(),
                        formula_type: g.formula_type.clone(),
                        operators: t.ops.clone(),
                        ..Default::default()
                    })
            })
            .collect();
        let newly_dead: Vec<String> = cc_room
            .operators
            .iter()
            .filter(|id| !exclude_cc.contains(*id))
            .filter(|id| {
                op_index.get(id.as_str()).is_some_and(|op| {
                    cc_op_is_dead(op, &team_rooms, &op_index, registry, building_data)
                })
            })
            .cloned()
            .collect();
        if newly_dead.is_empty() {
            return (
                RotationCcPlan {
                    slot_id: control_room.map(|r| r.slot_id.clone()),
                    control_slots,
                    squad1: cc_room.operators,
                    conditions: cc_conditions,
                    global_bonuses,
                },
                groups,
            );
        }
        exclude_cc.extend(newly_dead);
    }
}

pub(super) fn greedy_cc_fill(
    acc: CcBonusAccumulator,
    candidates: &[(&OperatorBaseProfile, Vec<CcBonus>)],
    max_slots: i32,
) -> (Vec<String>, CcBonusAccumulator) {
    let mut acc = acc;
    let mut chosen: HashSet<String> = HashSet::new();
    let mut cc_assigned: Vec<String> = Vec::new();
    for _ in 0..max_slots.max(0) {
        let mut best_op: Option<&OperatorBaseProfile> = None;
        let mut best_bonuses: &[CcBonus] = &[];
        let mut best_gain = 0.0;
        for (op, bonuses) in candidates {
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
    (cc_assigned, acc)
}
