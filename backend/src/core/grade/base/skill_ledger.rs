//! Per-skill contribution ledger — the "how this room's number is calculated"
//! breakdown behind the deep-dive UI.
//!
//! Every line is a MARGINAL measured by ablation: remove exactly one buff from
//! one operator, re-score the room with everything else unchanged, and report
//! the delta. That definition survives every mechanism the engine models —
//! pair riders (Lemuen loses her +25 if Exusiai leaves), non-stacking families
//! (a second +7% global shows 0), faction gates, recipe-type scaling — because
//! it asks the scorer itself rather than re-deriving the rules.
//!
//! Lines that measure 0 are classified rather than hidden: a priced strategy
//! whose gate isn't met here reads "inactive", a morale/drain skill reads
//! "morale" (it moves the sustain sim, not efficiency), a capacity skill reads
//! "capacity", and a buff the engine deliberately doesn't price (never-guess)
//! reads "unmodeled" — three honest states where a ✓/✗ ledger has two.

use std::collections::HashMap;

use crate::core::gamedata::types::building::BuildingDataFile;
use serde::Serialize;
use ts_rs::TS;

use super::assignment::{CcBonusAccumulator, CcCondition, cc_bonuses, compute_team_efficiency};
use super::buff_registry::BuffResolutionStrategy;
use super::types::{OperatorBaseProfile, compute_match_tags};

/// How a zero-marginal line should be read.
///
/// Serialized straight into `SkillLineDto`, so the generated TS binding is a
/// union rather than a bare `string` — the frontend switches on these values.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
#[derive(TS)]
#[ts(export)]
pub enum LineDisposition {
    /// Non-zero marginal: this line is part of the room's number.
    Contributes,
    /// Priced strategy, but its gate isn't met by this crew/base.
    Inactive,
    /// Real and unconditional, but a stronger skill of the same non-stacking
    /// type is already active in this crew - the game's "(only the most
    /// effective one will take effect)" clause. Zero marginal, not a fault.
    Covered,
    /// A conditional Control-Center skill whose gate IS met by at least one
    /// production team: its credit lives inside those rooms' numbers (see
    /// their breakdowns), never on the CC row itself.
    PerRoom,
    /// Moves morale/drain (the sustain sim), not room efficiency.
    #[serde(rename = "morale")]
    MoraleOnly,
    /// Moves the order/stock capacity, not the speed number shown.
    #[serde(rename = "capacity")]
    CapacityOnly,
    /// Non-production Control-Center value (clue/training/HR) — reported in
    /// the room's `non_production` block, not its efficiency.
    NonProduction,
    /// The engine deliberately prices this at zero (never-guess).
    Unmodeled,
}

/// One skill line of a room's breakdown.
#[derive(Clone, Debug)]
pub struct LedgerLine {
    pub operator_id: String,
    pub buff_id: String,
    /// Marginal order/production SPEED % in this exact crew.
    pub speed_pct: f64,
    /// Marginal order VALUE %.
    pub value_pct: f64,
    /// Set for Control-Center lines shown on a production room: the line's
    /// owner sits in the CC, not this room.
    pub from_control_center: bool,
    pub disposition: LineDisposition,
}

const EPS: f64 = 1e-6;

/// A profile identical to `op` with one buff removed. Match tags are
/// recomputed — some derive from the buff set, and a stale tag would keep a
/// faction gate satisfied that the ablation should break.
fn ablated(
    op: &OperatorBaseProfile,
    buff_id: &str,
    building_data: &BuildingDataFile,
) -> OperatorBaseProfile {
    let mut p = op.clone();
    p.available_buffs.retain(|b| b != buff_id);
    p.match_tags = compute_match_tags(&p.faction_tags, &p.available_buffs, building_data);
    p
}

fn zero_disposition(strategy: Option<&BuffResolutionStrategy>, crew: &[String]) -> LineDisposition {
    match strategy {
        Some(BuffResolutionStrategy::MoraleModifier { .. }) => LineDisposition::MoraleOnly,
        Some(BuffResolutionStrategy::CapacityOnly { .. }) => LineDisposition::CapacityOnly,
        // A capacity component with 0 efficiency marginal is still ACTIVE
        // capacity when its teammate gate is met (Lappland's "+4 order limit
        // with Texas" reads "capacity", not "inactive", while Texas shares
        // the post).
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
        // Drain-aura immunity (Waai Fu's Team Spirit) is real and priced, but
        // entirely a morale effect - zero efficiency marginal by design.
        Some(BuffResolutionStrategy::MoraleDrainAuraImmunity) => LineDisposition::MoraleOnly,
        Some(BuffResolutionStrategy::Complex { .. }) | None => LineDisposition::Unmodeled,
        Some(_) => LineDisposition::Inactive,
    }
}

/// The scoring context a room ledger re-runs its ablations against. All fields
/// are exactly what `compute_team_efficiency` was called with for the real
/// number, so a marginal of 0 genuinely means "removing this changes nothing".
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
        // Borrow everything from the real index, swapping in at most one
        // ablated profile.
        let mut index: HashMap<&str, &OperatorBaseProfile> = self.op_index.clone();
        if let Some((id, p)) = replace {
            index.insert(id, p);
        }
        compute_team_efficiency(
            ops,
            room_type,
            formula,
            &index,
            self.registry,
            self.building_data,
            self.facility_counts,
            self.total_dorm_levels,
            self.morale_drains,
            conditions,
        )
    }

    /// Global bonuses + conditions a fixed CC crew grants, with at most one
    /// member's profile replaced by an ablated copy.
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
                acc.add(&cc_bonuses(op, self.registry, self.building_data));
            }
        }
        acc.finish()
    }
}

/// The global bonuses + conditions a fixed Control-Center crew grants -
/// exposed so rotation cells can build per-shift ledgers against the crew
/// that actually works their shift.
pub(crate) fn grants_of(
    ctx: &LedgerCtx,
    cc_ops: &[String],
) -> (HashMap<String, f64>, Vec<CcCondition>) {
    ctx.cc_grants(cc_ops, None)
}

/// The breakdown for one production room (TRADING / MANUFACTURE / POWER):
/// each crew member's same-room buffs, plus every Control-Center line that
/// targets this room type. `global_bonuses`/`cc_conditions` must be the grants
/// of `cc_ops` exactly as the room was really scored with.
#[allow(clippy::too_many_arguments)]
pub(crate) fn production_room_ledger(
    ctx: &LedgerCtx,
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

    // The room's own crew, one line per same-room buff.
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
            let (speed, value) =
                ctx.score(ops, room_type, formula, cc_conditions, Some((id, &probe)));
            let d_speed = full_speed - speed;
            let d_value = full_value - value;
            let disposition = if d_speed.abs() > EPS || d_value.abs() > EPS {
                LineDisposition::Contributes
            } else {
                zero_disposition(ctx.registry.get(buff_id), ops)
            };
            out.push(LedgerLine {
                operator_id: (*id).clone(),
                buff_id: buff_id.clone(),
                speed_pct: d_speed,
                value_pct: d_value,
                from_control_center: false,
                disposition,
            });
        }
    }

    // Control-Center lines that target this room type. Unconditional globals
    // use family attribution (the winner claims the family's value, duplicate
    // copies read "covered" - ablation is tie-blind and would let the value
    // go unclaimed); conditional globals keep the ablation path, since their
    // credit genuinely depends on THIS room's crew meeting the gate.
    let (lines, winners) = cc_bonus_lines(ctx, cc_ops);
    for (i, l) in lines.iter().enumerate() {
        if l.bonus.conditional.is_none() {
            if l.bonus.room != room_type {
                continue;
            }
            let claims = l.bonus.stacks
                || winners.get(&(l.bonus.room.clone(), l.bonus.family.clone())) == Some(&i);
            let (value, disposition) = if claims {
                (l.bonus.bonus, LineDisposition::Contributes)
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
            });
            continue;
        }
        // Conditional global: does the gate fire in THIS crew? Ablate and
        // re-score under the reduced grants.
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
        });
    }
    out
}

/// The Control Center row's own breakdown: each member's CONTROL buff, valued
/// as the marginal on the SUM of global bonuses the crew grants (the number the
/// CC row displays). Non-global CC skills classify by strategy.
/// One Control-Center bonus line, pre-attribution.
struct CcBonusLine {
    operator_id: String,
    buff_id: String,
    bonus: super::assignment::CcBonus,
}

/// Gather every CC member's bonus-bearing CONTROL buff, plus the winner of
/// each non-stacking family: the strongest member, first-in-crew-order on
/// ties - the game's own "(only the most effective one will take effect)"
/// rule. Ablation marginals are tie-blind (with two +7% copies, removing
/// either changes nothing, so NOBODY claims the +7 that is really there);
/// explicit attribution keeps the lines summing to the room's number.
fn cc_bonus_lines(
    ctx: &LedgerCtx,
    cc_ops: &[String],
) -> (Vec<CcBonusLine>, HashMap<(String, String), usize>) {
    let mut lines = Vec::new();
    for id in cc_ops {
        let Some(op) = ctx.op_index.get(id.as_str()) else {
            continue;
        };
        for buff_id in &op.available_buffs {
            let Some(buff) = ctx.building_data.buffs.get(buff_id) else {
                continue;
            };
            if let Some(bonus) =
                super::assignment::cc_bonus_for(buff_id, buff, ctx.registry.get(buff_id))
            {
                lines.push(CcBonusLine {
                    operator_id: (*id).clone(),
                    buff_id: buff_id.clone(),
                    bonus,
                });
            }
        }
    }
    let mut winners: HashMap<(String, String), usize> = HashMap::new();
    for (i, l) in lines.iter().enumerate() {
        if l.bonus.stacks || l.bonus.conditional.is_some() {
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

pub(crate) fn control_room_ledger(
    ctx: &LedgerCtx,
    cc_ops: &[String],
    team_rooms: &[super::types::RoomAssignment],
) -> Vec<LedgerLine> {
    let mut out = Vec::new();
    // Non-bonus CONTROL buffs (morale, clue, unmodeled...) classify by
    // strategy as before.
    for id in cc_ops {
        let Some(op) = ctx.op_index.get(id.as_str()) else {
            continue;
        };
        for buff_id in &op.available_buffs {
            let Some(buff) = ctx.building_data.buffs.get(buff_id) else {
                continue;
            };
            if buff.room_type != "CONTROL"
                || super::assignment::cc_bonus_for(buff_id, buff, ctx.registry.get(buff_id))
                    .is_some()
            {
                continue;
            }
            out.push(LedgerLine {
                operator_id: (*id).clone(),
                buff_id: buff_id.clone(),
                speed_pct: 0.0,
                value_pct: 0.0,
                from_control_center: false,
                disposition: zero_disposition(ctx.registry.get(buff_id), cc_ops),
            });
        }
    }
    // Bonus lines: stacking entries contribute outright, each non-stacking
    // family is claimed by its winner, the rest read "covered". Conditional
    // globals are credited on their target rooms, not here.
    let (lines, winners) = cc_bonus_lines(ctx, cc_ops);
    for (i, l) in lines.iter().enumerate() {
        let claims = l.bonus.stacks
            || winners.get(&(l.bonus.room.clone(), l.bonus.family.clone())) == Some(&i);
        let (value, disposition) = if let Some(cond) = &l.bonus.conditional {
            // Umiri-style: the credit lands inside the rooms whose crews meet
            // the gate. On the CC row, say WHERE it went - "inactive" is only
            // honest when no team satisfies it.
            let fires = super::assignment::cc_condition_fires(cond, team_rooms, ctx.op_index);
            (
                0.0,
                if fires {
                    LineDisposition::PerRoom
                } else {
                    LineDisposition::Inactive
                },
            )
        } else if claims {
            (l.bonus.bonus, LineDisposition::Contributes)
        } else {
            (0.0, LineDisposition::Covered)
        };
        out.push(LedgerLine {
            operator_id: l.operator_id.clone(),
            buff_id: l.buff_id.clone(),
            speed_pct: value,
            value_pct: 0.0,
            from_control_center: false,
            disposition,
        });
    }
    out
}
