//! Per-skill contribution ledger behind the deep-dive UI

use std::collections::HashMap;

use myrtle_gamedata_raw::tables::building::BuildingDataFile;
use serde::Serialize;

use crate::assignment::{CcBonusAccumulator, CcCondition, cc_bonuses, compute_team_efficiency};
use crate::registry::BuffResolutionStrategy;
use crate::types::{OperatorBaseProfile, compute_match_tags};
use crate::util::buff_family;

/// How a zero-marginal line should be read
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LineDisposition {
    Contributes,
    Inactive,
    Covered,
    PerRoom,
    #[serde(rename = "morale")]
    MoraleOnly,
    #[serde(rename = "capacity")]
    CapacityOnly,
    NonProduction,
    Unmodeled,
}

#[derive(Clone, Debug)]
pub struct LedgerLine {
    pub operator_id: String,
    pub buff_id: String,
    pub speed_pct: f64,
    pub value_pct: f64,
    pub from_control_center: bool,
    pub disposition: LineDisposition,
    pub note: Option<String>,
}

const EPS: f64 = 1e-6;

fn ablated(
    op: &OperatorBaseProfile,
    buff_id: &str,
    building_data: &BuildingDataFile,
) -> OperatorBaseProfile {
    ablated_where(op, |b| b != buff_id, building_data)
}

fn ablated_where(
    op: &OperatorBaseProfile,
    keep: impl Fn(&str) -> bool,
    building_data: &BuildingDataFile,
) -> OperatorBaseProfile {
    let mut p = op.clone();
    p.available_buffs.retain(|b| keep(b));
    p.match_tags = compute_match_tags(&p.faction_tags, &p.available_buffs, building_data);
    p
}

struct PriorityExclusion {
    owner: String,
    buff_id: String,
    families: Vec<String>,
}

fn priority_exclusions(
    ctx: &LedgerCtx<'_>,
    ops: &[String],
    room_type: &str,
) -> Vec<PriorityExclusion> {
    ops.iter()
        .filter_map(|id| ctx.op_index.get(id.as_str()).map(|op| (id, op)))
        .flat_map(|(id, op)| {
            op.available_buffs.iter().filter_map(move |b| {
                let live = ctx
                    .building_data
                    .buffs
                    .get(b)
                    .is_some_and(|x| x.room_type == room_type);
                match ctx.registry.get(b) {
                    Some(BuffResolutionStrategy::CapacityTierScaling { excludes, .. })
                        if live && !excludes.is_empty() =>
                    {
                        Some(PriorityExclusion {
                            owner: id.clone(),
                            buff_id: b.clone(),
                            families: excludes.clone(),
                        })
                    }
                    _ => None,
                }
            })
        })
        .collect()
}

fn buff_name<'a>(ctx: &'a LedgerCtx<'_>, buff_id: &'a str) -> &'a str {
    ctx.building_data
        .buffs
        .get(buff_id)
        .map_or(buff_id, |b| b.buff_name.as_str())
}

fn zero_disposition(strategy: Option<&BuffResolutionStrategy>, crew: &[String]) -> LineDisposition {
    match strategy {
        Some(BuffResolutionStrategy::MoraleModifier { .. }) => LineDisposition::MoraleOnly,
        Some(BuffResolutionStrategy::CapacityOnly { .. }) => LineDisposition::CapacityOnly,
        Some(BuffResolutionStrategy::EfficiencyWithOrderLimit { order_limit, .. })
            if *order_limit != 0 =>
        {
            LineDisposition::CapacityOnly
        }
        Some(BuffResolutionStrategy::ConditionalOnTeammate {
            required_char_id,
            order_limit,
            ..
        }) => {
            let gate_met = required_char_id
                .as_ref()
                .is_none_or(|rc| crew.iter().any(|o| o == rc));
            if gate_met && *order_limit != 0 {
                LineDisposition::CapacityOnly
            } else {
                LineDisposition::Inactive
            }
        }
        Some(BuffResolutionStrategy::ControlNonProduction { .. }) => LineDisposition::NonProduction,
        Some(BuffResolutionStrategy::MoraleDrainAuraImmunity) => LineDisposition::MoraleOnly,
        Some(BuffResolutionStrategy::Complex { .. }) | None => LineDisposition::Unmodeled,
        Some(_) => LineDisposition::Inactive,
    }
}

pub struct LedgerCtx<'a> {
    pub op_index: &'a HashMap<&'a str, &'a OperatorBaseProfile>,
    pub registry: &'a HashMap<String, BuffResolutionStrategy>,
    pub building_data: &'a BuildingDataFile,
    pub facility_counts: &'a HashMap<String, usize>,
    pub total_dorm_levels: i32,
    pub morale_drains: &'a HashMap<String, f64>,
}

impl LedgerCtx<'_> {
    fn score(
        &self,
        ops: &[String],
        room_type: &str,
        formula: Option<&str>,
        conditions: &[CcCondition],
        replace: Option<(&str, &OperatorBaseProfile)>,
    ) -> (f64, f64) {
        self.score_with(ops, room_type, formula, conditions, replace.as_slice())
    }

    fn score_with(
        &self,
        ops: &[String],
        room_type: &str,
        formula: Option<&str>,
        conditions: &[CcCondition],
        replace: &[(&str, &OperatorBaseProfile)],
    ) -> (f64, f64) {
        let mut index: HashMap<&str, &OperatorBaseProfile> = self.op_index.clone();
        for (id, p) in replace {
            index.insert(id, p);
        }
        compute_team_efficiency(
            ops,
            room_type,
            formula,
            None,
            &index,
            self.registry,
            self.building_data,
            self.facility_counts,
            self.total_dorm_levels,
            self.morale_drains,
            conditions,
        )
    }

    fn cc_grants(
        &self,
        cc_ops: &[String],
        replace: Option<(&str, &OperatorBaseProfile)>,
    ) -> (HashMap<String, f64>, Vec<CcCondition>) {
        let mut acc = CcBonusAccumulator::default();
        for id in cc_ops {
            let profile = match replace {
                Some((rid, p)) if rid == id => Some(p),
                _ => self.op_index.get(id.as_str()).copied(),
            };
            if let Some(op) = profile {
                acc.add_from(op, &cc_bonuses(op, self.registry, self.building_data));
            }
        }
        acc.finish()
    }
}

/// Globals + conditions a fixed CC crew grants
pub fn grants_of(
    ctx: &LedgerCtx<'_>,
    cc_ops: &[String],
) -> (HashMap<String, f64>, Vec<CcCondition>) {
    ctx.cc_grants(cc_ops, None)
}

/// Breakdown for one production room
#[allow(clippy::too_many_arguments)]
pub fn production_room_ledger(
    ctx: &LedgerCtx<'_>,
    ops: &[String],
    room_type: &str,
    formula: Option<&str>,
    cc_ops: &[String],
    global_bonuses: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
) -> Vec<LedgerLine> {
    let mut out = Vec::new();
    let (full_speed, full_value) = ctx.score(ops, room_type, formula, cc_conditions, None);
    let full_global = global_bonuses.get(room_type).copied().unwrap_or(0.0);

    let exclusions = priority_exclusions(ctx, ops, room_type);
    for id in ops {
        let Some(op) = ctx.op_index.get(id.as_str()) else {
            continue;
        };
        for buff_id in &op.available_buffs {
            let Some(buff) = ctx.building_data.buffs.get(buff_id) else {
                continue;
            };
            if buff.room_type != room_type {
                continue;
            }
            let probe = ablated(op, buff_id, ctx.building_data);
            let kept_out: Vec<(String, OperatorBaseProfile)> = exclusions
                .iter()
                .filter(|e| e.buff_id == *buff_id)
                .flat_map(|e| {
                    ops.iter()
                        .filter(|o| *o != id)
                        .filter_map(|o| ctx.op_index.get(o.as_str()).map(|p| (o, *p)))
                        .map(|(o, other)| {
                            let keep = |b: &str| !e.families.iter().any(|f| f == buff_family(b));
                            (o.clone(), ablated_where(other, keep, ctx.building_data))
                        })
                })
                .collect();
            let mut replacements: Vec<(&str, &OperatorBaseProfile)> = vec![(id.as_str(), &probe)];
            replacements.extend(kept_out.iter().map(|(o, p)| (o.as_str(), p)));
            let (speed, value) =
                ctx.score_with(ops, room_type, formula, cc_conditions, &replacements);
            let d_speed = full_speed - speed;
            let d_value = full_value - value;
            let excluded_by = exclusions
                .iter()
                .find(|e| e.owner != *id && e.families.iter().any(|f| f == buff_family(buff_id)));
            let disposition = if excluded_by.is_some() {
                LineDisposition::Covered
            } else if d_speed.abs() > EPS || d_value.abs() > EPS {
                LineDisposition::Contributes
            } else {
                zero_disposition(ctx.registry.get(buff_id), ops)
            };
            let note = match ctx.registry.get(buff_id) {
                _ if excluded_by.is_some() => excluded_by.map(|e| {
                    format!(
                        "Does not stack with a roommate's {}, which takes priority in this room.",
                        buff_name(ctx, &e.buff_id)
                    )
                }),
                Some(BuffResolutionStrategy::MatchCountScaling {
                    per_match_pct,
                    count_skills: true,
                    capacity: false,
                    ..
                }) if d_speed > EPS && *per_match_pct > 0.0 => {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let n = (d_speed / per_match_pct).round() as usize;
                    Some(format!(
                        "+{per_match_pct:.0}% per counted skill, {n} in this room (this operator's own included); the counted skills' figures already carry it"
                    ))
                }
                _ => None,
            };
            out.push(LedgerLine {
                operator_id: (*id).clone(),
                buff_id: buff_id.clone(),
                speed_pct: d_speed,
                value_pct: d_value,
                from_control_center: false,
                disposition,
                note,
            });
        }
    }

    let (lines, winners) = cc_bonus_lines(ctx, cc_ops);
    for (i, l) in lines.iter().enumerate() {
        if l.bonus.conditional.is_none() {
            if l.bonus.room != room_type {
                continue;
            }
            let claims = !l.gate_unmet
                && (l.bonus.stacks
                    || winners.get(&(l.bonus.room.clone(), l.bonus.family.clone())) == Some(&i));
            let (value, disposition) = if claims {
                (l.bonus.bonus, LineDisposition::Contributes)
            } else if l.gate_unmet {
                (0.0, LineDisposition::Inactive)
            } else {
                (0.0, LineDisposition::Covered)
            };
            out.push(LedgerLine {
                operator_id: l.operator_id.clone(),
                buff_id: l.buff_id.clone(),
                speed_pct: value,
                value_pct: 0.0,
                from_control_center: true,
                disposition,
                note: l.gate_unmet.then(|| {
                    "Needs another operator of its group in the Control Center.".to_string()
                }),
            });
            continue;
        }
        if l.bonus
            .conditional
            .as_ref()
            .is_some_and(|c| c.target_room != room_type)
        {
            continue;
        }
        let Some(op) = ctx.op_index.get(l.operator_id.as_str()) else {
            continue;
        };
        let probe = ablated(op, &l.buff_id, ctx.building_data);
        let (globals, conditions) = ctx.cc_grants(cc_ops, Some((l.operator_id.as_str(), &probe)));
        let (speed, value) = ctx.score(ops, room_type, formula, &conditions, None);
        let global = globals.get(room_type).copied().unwrap_or(0.0);
        let d_speed = (full_speed + full_global) - (speed + global);
        let d_value = full_value - value;
        let disposition = if d_speed.abs() > EPS || d_value.abs() > EPS {
            LineDisposition::Contributes
        } else {
            LineDisposition::Inactive
        };
        out.push(LedgerLine {
            operator_id: l.operator_id.clone(),
            buff_id: l.buff_id.clone(),
            speed_pct: d_speed,
            value_pct: d_value,
            from_control_center: true,
            disposition,
            note: None,
        });
    }
    relabel_feeding_generators(ctx, &mut out);
    out
}

fn relabel_feeding_generators(ctx: &LedgerCtx<'_>, lines: &mut [LedgerLine]) {
    use crate::scoring::clause::{ClauseKind, ResourceOp, clauses_from_strategy};
    let mut consumed: HashMap<String, String> = HashMap::new();
    for l in lines.iter() {
        if l.speed_pct <= EPS {
            continue;
        }
        let (Some(buff), Some(strategy)) = (
            ctx.building_data.buffs.get(&l.buff_id),
            ctx.registry.get(&l.buff_id),
        ) else {
            continue;
        };
        for c in clauses_from_strategy(&l.buff_id, buff, strategy) {
            let resource = match &c.kind {
                ClauseKind::ResourceConvert(ResourceOp::Consume { resource })
                | ClauseKind::ScalingPoolPoints { resource, .. } => resource.clone(),
                _ => continue,
            };
            consumed
                .entry(resource)
                .or_insert_with(|| buff.buff_name.clone());
        }
    }
    if consumed.is_empty() {
        return;
    }
    for l in lines.iter_mut() {
        if l.disposition != LineDisposition::Inactive {
            continue;
        }
        let (Some(buff), Some(strategy)) = (
            ctx.building_data.buffs.get(&l.buff_id),
            ctx.registry.get(&l.buff_id),
        ) else {
            continue;
        };
        let fed: Option<(&String, &String)> = clauses_from_strategy(&l.buff_id, buff, strategy)
            .iter()
            .find_map(|c| match &c.kind {
                ClauseKind::ResourceConvert(
                    ResourceOp::Generate { resource, .. }
                    | ResourceOp::Convert { to: resource, .. },
                ) => consumed.get_key_value(resource),
                _ => None,
            });
        if let Some((resource, reader)) = fed {
            l.disposition = LineDisposition::Contributes;
            l.note = Some(format!(
                "Feeds the {resource} pool; its value shows on the {reader} line."
            ));
        }
    }
}

struct CcBonusLine {
    operator_id: String,
    buff_id: String,
    bonus: crate::assignment::CcBonus,
    gate_unmet: bool,
}

/// Every CC member's bonus-bearing CONTROL buff, plus each non-stacking family's winner
fn cc_bonus_lines(
    ctx: &LedgerCtx<'_>,
    cc_ops: &[String],
) -> (Vec<CcBonusLine>, HashMap<(String, String), usize>) {
    let crew: Vec<(String, Vec<String>)> = cc_ops
        .iter()
        .filter_map(|id| ctx.op_index.get(id.as_str()))
        .map(|op| (op.char_id.clone(), op.match_tags.clone()))
        .collect();
    let mut lines = Vec::new();
    for id in cc_ops {
        let Some(op) = ctx.op_index.get(id.as_str()) else {
            continue;
        };
        for buff_id in &op.available_buffs {
            let Some(buff) = ctx.building_data.buffs.get(buff_id) else {
                continue;
            };
            if let Some(mut bonus) =
                crate::assignment::cc_bonus_for(buff_id, buff, ctx.registry.get(buff_id))
            {
                let gate_unmet = match bonus.crew_gate.take() {
                    None => false,
                    Some(gate) if gate.met(id, &crew) => {
                        bonus.bonus = gate.bonus_pct;
                        false
                    }
                    Some(_) => true,
                };
                lines.push(CcBonusLine {
                    operator_id: (*id).clone(),
                    buff_id: buff_id.clone(),
                    bonus,
                    gate_unmet,
                });
            }
        }
    }
    let mut winners: HashMap<(String, String), usize> = HashMap::new();
    for (i, l) in lines.iter().enumerate() {
        if l.bonus.stacks || l.bonus.conditional.is_some() || l.gate_unmet {
            continue;
        }
        let key = (l.bonus.room.clone(), l.bonus.family.clone());
        match winners.get(&key) {
            Some(&j) if lines[j].bonus.bonus >= l.bonus.bonus => {}
            _ => {
                winners.insert(key, i);
            }
        }
    }
    (lines, winners)
}

fn conditional_reach_note(cond: &crate::assignment::CcCondition, reach: usize) -> String {
    let room = match cond.target_room.as_str() {
        "MANUFACTURE" => "Factories",
        "TRADING" => "Trading Posts",
        "POWER" => "Power Plants",
        other => other,
    };
    let who = if cond.faction_token.starts_with("char_") {
        "the named operator".to_string()
    } else {
        let mut chars = cond.faction_token.chars();
        chars.next().map_or_else(String::new, |c| {
            c.to_uppercase().collect::<String>() + chars.as_str()
        })
    };
    let payload = if cond.order_limit != 0.0 && cond.bonus_pct == 0.0 {
        format!("+{} order limit", cond.order_limit)
    } else {
        format!("+{}%", cond.bonus_pct)
    };
    if cond.per_operator {
        match reach {
            0 => format!("{payload} to each {who} operator in {room}; no team fields one."),
            1 => format!("{payload} to each {who} operator in {room}; 1 fielded."),
            n => format!("{payload} to each {who} operator in {room}; {n} fielded."),
        }
    } else {
        let need = cond.required_count;
        match reach {
            0 => format!("{payload} to {room} with {need} {who} operators; none qualifies."),
            n => format!("{payload} to {room} with {need} {who} operators; {n} qualify."),
        }
    }
}

pub fn control_room_ledger(
    ctx: &LedgerCtx<'_>,
    cc_ops: &[String],
    team_rooms: &[crate::types::RoomAssignment],
) -> Vec<LedgerLine> {
    let mut out = Vec::new();
    for id in cc_ops {
        let Some(op) = ctx.op_index.get(id.as_str()) else {
            continue;
        };
        for buff_id in &op.available_buffs {
            let Some(buff) = ctx.building_data.buffs.get(buff_id) else {
                continue;
            };
            if buff.room_type != "CONTROL"
                || crate::assignment::cc_bonus_for(buff_id, buff, ctx.registry.get(buff_id))
                    .is_some()
            {
                continue;
            }
            let note = match ctx.registry.get(buff_id) {
                Some(BuffResolutionStrategy::MoraleModifier {
                    recovery_per_hour,
                    base_wide,
                    is_self_only,
                    ..
                }) => Some(if *recovery_per_hour > 0.0 {
                    let scope = if *base_wide {
                        "every operator working outside a dormitory"
                    } else {
                        "the Control Center crew"
                    };
                    format!("+{recovery_per_hour} morale per hour to {scope}.")
                } else if *is_self_only {
                    "Affects only this operator's own morale; nothing for the base.".to_string()
                } else {
                    "Conditional morale effect the plan does not price (never guessed); \
                     the seat is not held for it."
                        .to_string()
                }),
                _ => None,
            };
            out.push(LedgerLine {
                operator_id: (*id).clone(),
                buff_id: buff_id.clone(),
                speed_pct: 0.0,
                value_pct: 0.0,
                from_control_center: false,
                disposition: zero_disposition(ctx.registry.get(buff_id), cc_ops),
                note,
            });
        }
    }
    let (lines, winners) = cc_bonus_lines(ctx, cc_ops);
    for (i, l) in lines.iter().enumerate() {
        let claims = l.bonus.stacks
            || winners.get(&(l.bonus.room.clone(), l.bonus.family.clone())) == Some(&i);
        let (value, disposition, note) = if let Some(cond) = &l.bonus.conditional {
            let reach = crate::assignment::cc_condition_reach(cond, team_rooms, ctx.op_index);
            (
                0.0,
                if reach > 0 {
                    LineDisposition::PerRoom
                } else {
                    LineDisposition::Inactive
                },
                Some(conditional_reach_note(cond, reach)),
            )
        } else if claims {
            (l.bonus.bonus, LineDisposition::Contributes, None)
        } else {
            (0.0, LineDisposition::Covered, None)
        };
        out.push(LedgerLine {
            operator_id: l.operator_id.clone(),
            buff_id: l.buff_id.clone(),
            speed_pct: value,
            value_pct: 0.0,
            from_control_center: false,
            disposition,
            note,
        });
    }
    out
}
