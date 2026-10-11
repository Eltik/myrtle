//! The Economy panel: each seat's DP cost and redeploy time after every cut,
//! the starting DP, and where DP comes from.
//!
//! Numbers read: the promotion key frame's `cost` and `respawn_time`;
//! `potential_ranks[i].buff.attributes.attribute_modifiers` of type `COST` and
//! `RESPAWN_TIME` for the ranks the build has; talent blackboard `cost`,
//! `respawn_time`, `value`, `runtime_cost` and `sp`; the module rank's
//! `attribute_blackboard` `cost` and `respawn_time`; the trait blackboard `cost`
//! and `interval`; the chosen skill's blackboard `cost`, `value`, `interval`
//! and its `sp_data`. Who a squad talent reaches comes from `targets`.

use std::collections::HashMap;

use super::Ctx;
use super::build::{Board, Seat, bb};
use super::index::{StatScale, TalentText, TextFacts};
use super::targets::fits;
use super::types::{
    Cut, CutKind, DpSource, Economy, EconomyBonus, EconomyNote, EconomyNoteKind, EconomySeat,
    SourceKind, SourceRef, SquadAnalyzeRequest, SynergyCond,
};

/// The economy plus each seat's final cost, which the synergy panel needs for
/// "DP Cost no more than N" targets.
pub struct EconomyOut {
    pub economy: Economy,
    pub cost_by_seat: HashMap<i32, i32>,
}

/// One cut; `change` is `(amount, ratio)` as `respawn_change` returns it.
const fn cut(
    kind: CutKind,
    name: Option<String>,
    from_seat: Option<i32>,
    (amount, ratio): (f64, Option<f64>),
) -> Cut {
    Cut {
        kind,
        name,
        from_seat,
        amount,
        ratio,
    }
}

/// A squad talent's cut, added to each receiving seat's `cuts` or
/// `respawn_cuts` (`list`).
fn give_cut(
    rows: &mut [EconomySeat],
    receivers: &[i32],
    list: fn(&mut EconomySeat) -> &mut Vec<Cut>,
    c: &Cut,
) {
    for r in receivers {
        if let Some(row) = rows.iter_mut().find(|x| x.seat == *r) {
            list(row).push(c.clone());
        }
    }
}

/// A DP source from `s` with every optional field empty.
fn dp_source(s: &Seat<'_>, source: SourceRef, amount: f64) -> DpSource {
    DpSource {
        seat: s.seat,
        char_id: s.char_id.clone(),
        support: s.support,
        source,
        amount,
        per_kill: false,
        refund: false,
        duration: None,
        interval: None,
        sp_cost: None,
        sp_init: None,
        first_ready: None,
        charges_on_attack: false,
        auto: false,
    }
}

/// A talent's cost or redeploy change is the talent holder's own when its text
/// opens with the stat ("DP Cost -1", "Redeployment Time -30 seconds") or it
/// names no other target. Gravel's "DP Cost -1; All units that cost 10 or
/// less DP gain DEF" names a target for the DEF half only.
const fn is_self(t: &TalentText) -> bool {
    t.opens_with_own_stat || t.target.is_none()
}

/// A talent that earns DP rather than cutting a cost: blackboard `cost` > 0
/// and text that speaks of gaining DP, not of a DP Cost bound (Sussurro's
/// `cost` 10 is "DP Cost no more than 10"; Texas's `cost` 2 is DP gained:
/// the same key, told apart only by the wording).
pub fn talent_gains_dp(f: &TextFacts, board: &Board) -> bool {
    bb(board, "cost").is_some_and(|c| c > 0.0)
        && f.talent.as_ref().is_some_and(|t| t.speaks_of_dp_gain)
}

/// A talent that adds DP at the start by being in the squad (Texas "Tactical
/// Delivery": `cost` 2, "when Texas is in the squad").
fn talent_start_bonus(f: &TextFacts, board: &Board) -> bool {
    talent_gains_dp(f, board) && f.talent.as_ref().is_some_and(|t| t.in_squad)
}

/// The DP a skill earns per activation, when it earns any. A key `cost` (or a
/// namespaced `...[cost].cost`) > 0 marks a DP skill; a skill that drips DP
/// (`interval` present) states its total in `value` or `fake_cost`, the tick
/// in `cost` (Myrtle S1: `cost` 1 every `interval` 0.57 s, `value` 14).
pub fn skill_dp(board: &Board) -> Option<f64> {
    let tick = board
        .iter()
        .find(|(k, v)| (k == "cost" || k.ends_with("[cost].cost")) && *v > 0.0)
        .map(|(_, v)| *v)?;
    let has_interval = board
        .iter()
        .any(|(k, _)| k == "interval" || k.ends_with("[cost].interval"));
    if has_interval {
        Some(
            bb(board, "value")
                .or_else(|| bb(board, "fake_cost"))
                .unwrap_or(tick),
        )
    } else {
        Some(tick)
    }
}

/// A redeploy change: seconds, or a ratio, negative as a cut (`SilverAsh`
/// -0.12 = -12%), positive as the multiplier itself (Chiave 0.25 = -75%).
/// Which one is how the talent's text prints it (`TalentText::scale_of`, 22
/// of 22 EN talents read); a module's attribute (no text) or an unread
/// talent falls back to the magnitude, |v| >= 1 seconds. Returns
/// `(amount, ratio)`.
fn respawn_change(v: f64, text: Option<&TalentText>) -> (f64, Option<f64>) {
    let flat = match text.and_then(|tt| tt.scale_of(v, true)) {
        Some(scale) => scale == StatScale::Flat,
        None => v.abs() >= 1.0,
    };
    if flat {
        (v, None)
    } else if v < 0.0 {
        (0.0, Some(v))
    } else {
        (0.0, Some(v - 1.0))
    }
}

fn note(kind: EconomyNoteKind, s: &Seat<'_>, name: &str, amount: Option<f64>) -> EconomyNote {
    EconomyNote {
        kind,
        seat: s.seat,
        char_id: s.char_id.clone(),
        name: name.to_string(),
        amount,
    }
}

/// Cost after every cut: amounts summed onto the base, ratios applied after,
/// rounded towards zero, never below 0.
fn apply(base: f64, cuts: &[Cut]) -> f64 {
    let sum: f64 = cuts.iter().map(|c| c.amount).sum();
    let mult: f64 = cuts
        .iter()
        .filter_map(|c| c.ratio)
        .map(|r| 1.0 + r)
        .product();
    ((base + sum) * mult).max(0.0)
}

/// A seat's DP cost after `cuts`, truncated to whole DP.
fn cost_after(base_cost: i32, cuts: &[Cut]) -> i32 {
    apply(f64::from(base_cost), cuts).trunc() as i32
}

#[allow(clippy::too_many_lines)] // one pass over the seats, kept in order
pub fn compute(ctx: &Ctx<'_>, seats: &[Seat<'_>], req: &SquadAnalyzeRequest) -> EconomyOut {
    let mut rows: Vec<EconomySeat> = Vec::new();
    let mut notes: Vec<EconomyNote> = Vec::new();
    let mut counted: Vec<EconomyNote> = Vec::new();
    let mut start_bonus: Vec<EconomyBonus> = Vec::new();
    let mut sources: Vec<DpSource> = Vec::new();

    // Own cuts first: potential, own talents, module.
    for s in seats {
        let kf = s.key_frame();
        let base_cost = kf.map_or(0, |k| k.cost);
        let base_respawn = f64::from(kf.map_or(0, |k| k.respawn_time));
        let mut cuts = Vec::new();
        let mut respawn_cuts = Vec::new();

        let mut pot_cost = 0.0;
        let mut pot_resp = 0.0;
        let pot = usize::try_from(s.build.potential).unwrap_or(0);
        for rank in s.op.potential_ranks.iter().take(pot) {
            let mods = rank
                .buff
                .as_ref()
                .and_then(|b| b.attributes.attribute_modifiers.as_ref());
            for m in mods.into_iter().flatten() {
                match m.attribute_type.as_str() {
                    "COST" => pot_cost += m.value,
                    "RESPAWN_TIME" => pot_resp += m.value,
                    _ => {}
                }
            }
        }
        if pot_cost != 0.0 {
            cuts.push(cut(CutKind::Potential, None, None, (pot_cost, None)));
        }
        if pot_resp != 0.0 {
            respawn_cuts.push(cut(CutKind::Potential, None, None, (pot_resp, None)));
        }

        for t in &s.talents {
            // A talent with no English reading changes nothing here: whether
            // its `cost` is the holder's or the squad's is in the wording.
            let Some(tt) = ctx.facts(&t.key).and_then(|f| f.talent.as_ref()) else {
                continue;
            };
            if !is_self(tt) {
                continue;
            }
            if let Some(c) = bb(&t.board, "cost").filter(|c| *c < 0.0) {
                // Swire the Elegant Wit's `cost` -5 is DP spent, not a DP Cost
                // cut; the cut talents all say "DP Cost".
                if tt.says_dp_cost {
                    cuts.push(cut(CutKind::Talent, Some(t.name.clone()), None, (c, None)));
                }
            }
            if let Some(v) = bb(&t.board, "respawn_time").filter(|v| *v != 0.0) {
                respawn_cuts.push(cut(
                    CutKind::Talent,
                    Some(t.name.clone()),
                    None,
                    respawn_change(v, Some(tt)),
                ));
            }
        }

        if let Some(mp) = s.module_phase {
            for b in &mp.attribute_blackboard {
                match b.key.as_str() {
                    "cost" if b.value != 0.0 => {
                        cuts.push(cut(CutKind::Module, s.module_name(), None, (b.value, None)));
                    }
                    "respawn_time" if b.value != 0.0 => respawn_cuts.push(cut(
                        CutKind::Module,
                        s.module_name(),
                        None,
                        respawn_change(b.value, None),
                    )),
                    _ => {}
                }
            }
        }

        rows.push(EconomySeat {
            seat: s.seat,
            char_id: s.char_id.clone(),
            support: s.support,
            cost: 0,
            base_cost,
            cuts,
            respawn: 0.0,
            base_respawn,
            respawn_cuts,
        });
    }

    // Own costs, for "DP Cost no more than N" receivers of squad effects.
    let own_cost: HashMap<i32, i32> = rows
        .iter()
        .map(|r| (r.seat, cost_after(r.base_cost, &r.cuts)))
        .collect();

    // SP bonuses from squad talents, keyed by receiving seat.
    let mut sp_bonus: HashMap<i32, f64> = HashMap::new();
    let mut sp_givers: Vec<(&Seat<'_>, String, f64, Vec<i32>)> = Vec::new();

    // Squad effects from other seats' talents. A support never gives.
    for g in seats.iter().filter(|s| !s.support) {
        for t in &g.talents {
            let Some(f) = ctx.facts(&t.key) else { continue };
            let Some(tt) = f.talent.as_ref() else {
                continue;
            };
            let cond = tt.cond;
            if talent_start_bonus(f, &t.board) {
                start_bonus.push(EconomyBonus {
                    seat: g.seat,
                    char_id: g.char_id.clone(),
                    name: t.name.clone(),
                    amount: bb(&t.board, "cost").unwrap_or(0.0).trunc() as i32,
                });
                continue;
            }
            // Effects that change a cost only after a deployment: not in the
            // starting numbers, listed with why.
            let value_cut = bb(&t.board, "value").filter(|v| *v < 0.0 && tt.says_dp_cost);
            if let Some(v) = value_cut {
                let kind = if tt.next_operator {
                    EconomyNoteKind::NextOperator
                } else if cond == SynergyCond::Field || cond == SynergyCond::Skill {
                    EconomyNoteKind::OnceDeployed
                } else {
                    EconomyNoteKind::Conditional
                };
                notes.push(note(kind, g, &t.name, Some(v)));
            }
            if let Some(v) = bb(&t.board, "runtime_cost").filter(|v| *v < 0.0) {
                notes.push(note(EconomyNoteKind::FirstDeploy, g, &t.name, Some(v)));
            }
            if is_self(tt) {
                continue;
            }
            // A game-picked receiver ("a random ally") changes no seat's
            // starting numbers.
            let Some(target) = tt.target.as_ref().filter(|t| !t.unseen) else {
                continue;
            };
            let receivers: Vec<i32> = seats
                .iter()
                .filter(|r| r.seat != g.seat)
                .filter(|r| {
                    fits(
                        target,
                        r,
                        own_cost.get(&r.seat).copied().unwrap_or(0),
                        &r.tag_ids,
                    )
                })
                .map(|r| r.seat)
                .collect();

            if let Some(c) = bb(&t.board, "cost").filter(|c| *c < 0.0) {
                if cond == SynergyCond::Squad {
                    let given = cut(
                        CutKind::Squad,
                        Some(t.name.clone()),
                        Some(g.seat),
                        (c, None),
                    );
                    give_cut(&mut rows, &receivers, |r| &mut r.cuts, &given);
                    if !receivers.is_empty() {
                        counted.push(note(EconomyNoteKind::CostCut, g, &t.name, Some(c)));
                    }
                } else {
                    notes.push(note(EconomyNoteKind::OnceDeployed, g, &t.name, Some(c)));
                }
            }
            if let Some(v) = bb(&t.board, "respawn_time").filter(|v| *v != 0.0) {
                if cond == SynergyCond::Squad {
                    let given = cut(
                        CutKind::Squad,
                        Some(t.name.clone()),
                        Some(g.seat),
                        respawn_change(v, Some(tt)),
                    );
                    give_cut(&mut rows, &receivers, |r| &mut r.respawn_cuts, &given);
                    if !receivers.is_empty() {
                        counted.push(note(EconomyNoteKind::RedeployCut, g, &t.name, Some(v)));
                    }
                } else {
                    let kind = if cond == SynergyCond::Field {
                        EconomyNoteKind::OnceDeployed
                    } else {
                        EconomyNoteKind::Conditional
                    };
                    notes.push(note(kind, g, &t.name, Some(v)));
                }
            }
            // Starting SP for a squad's skills (Bagpipe "Martial Tradition":
            // `sp` to Vanguards in the squad) moves a DP skill's first use.
            if let Some(sp) = bb(&t.board, "sp").filter(|v| *v > 0.0)
                && cond == SynergyCond::Squad
                && tt.starting_sp
            {
                for r in &receivers {
                    *sp_bonus.entry(*r).or_insert(0.0) += sp;
                }
                sp_givers.push((g, t.name.clone(), sp, receivers.clone()));
            }
        }
    }

    // DP sources.
    for s in seats {
        let trait_refund = ctx.trait_facts(s).is_some_and(|f| f.refund);
        let trait_cost = bb(&s.trait_board, "cost");
        let trait_interval = bb(&s.trait_board, "interval");
        // A positive trait `cost` with no `interval` is DP per kill (the
        // Charger shape, 7 of 7 on 2026-10-11); a negative one with an
        // `interval` is a drain (the Merchant shape, 5 of 5).
        if trait_interval.is_none()
            && let Some(c) = trait_cost.filter(|c| *c > 0.0)
        {
            sources.push(DpSource {
                per_kill: true,
                refund: trait_refund,
                ..dp_source(s, SourceRef::bare(SourceKind::Trait), c)
            });
        }
        if let (Some(c), Some(interval)) = (trait_cost.filter(|c| *c < 0.0), trait_interval) {
            // Merchants pay `cost` DP every `interval` seconds while
            // deployed; the note carries DP per second.
            let interval = if interval > 0.0 { interval } else { 1.0 };
            notes.push(note(EconomyNoteKind::Drain, s, "", Some(c / interval)));
        }
        if let Some(sk) = &s.skill
            && let Some(amount) = skill_dp(&sk.board)
        {
            let sp = &sk.level.sp_data;
            let bonus = sp_bonus.get(&s.seat).copied().unwrap_or(0.0);
            let sp_init = f64::from(sp.init_sp) + bonus;
            let time = sp.sp_type == "INCREASE_WITH_TIME";
            let inc = if sp.increment > 0.0 {
                sp.increment
            } else {
                1.0
            };
            let interval = bb(&sk.board, "interval").or_else(|| {
                sk.board
                    .iter()
                    .find(|(k, _)| k.ends_with("[cost].interval"))
                    .map(|(_, v)| *v)
            });
            sources.push(DpSource {
                duration: (sk.level.duration > 0.0).then_some(sk.level.duration),
                interval,
                sp_cost: Some(sp.sp_cost),
                sp_init: Some(sp_init.trunc() as i32),
                first_ready: time.then(|| ((f64::from(sp.sp_cost) - sp_init) / inc).max(0.0)),
                charges_on_attack: sp.sp_type == "INCREASE_WHEN_ATTACK",
                auto: sk.level.skill_type == "AUTO",
                ..dp_source(s, sk.source(), amount)
            });
        }
        for t in &s.talents {
            let Some(f) = ctx.facts(&t.key) else { continue };
            if !talent_gains_dp(f, &t.board) || talent_start_bonus(f, &t.board) {
                continue;
            }
            let Some(tt) = f.talent.as_ref() else {
                continue;
            };
            sources.push(DpSource {
                per_kill: tt.on_defeat,
                interval: bb(&t.board, "interval"),
                ..dp_source(s, t.source(), bb(&t.board, "cost").unwrap_or(0.0))
            });
            if tt.conditional {
                notes.push(note(
                    EconomyNoteKind::Conditional,
                    s,
                    &t.name,
                    bb(&t.board, "cost"),
                ));
            }
        }
    }
    for (g, name, sp, receivers) in sp_givers {
        let moved = sources
            .iter()
            .any(|d| d.source.kind == SourceKind::Skill && receivers.contains(&d.seat));
        if moved {
            counted.push(note(EconomyNoteKind::SpBonus, g, &name, Some(sp)));
        }
    }

    let mut cost_by_seat = HashMap::new();
    for r in &mut rows {
        r.cost = cost_after(r.base_cost, &r.cuts);
        r.respawn = apply(r.base_respawn, &r.respawn_cuts);
        cost_by_seat.insert(r.seat, r.cost);
    }
    rows.sort_by_key(|r| (r.cost, r.seat));

    let start_base = req.start_dp;
    let start_dp = start_base + start_bonus.iter().map(|b| b.amount).sum::<i32>();
    let total = rows.iter().map(|r| r.cost).sum();
    let take = req.deploy_limit.map_or(rows.len(), |n| {
        usize::try_from(n.max(0)).unwrap_or(0).min(rows.len())
    });
    let cheapest = rows.iter().take(take).map(|r| r.cost).sum();
    let openers = i32::try_from(rows.iter().filter(|r| r.cost <= start_dp).count()).unwrap_or(0);

    EconomyOut {
        economy: Economy {
            start_dp,
            start_base,
            start_bonus,
            seats: rows,
            total,
            cheapest,
            cheapest_count: i32::try_from(take).unwrap_or(0),
            deploy_limit: req.deploy_limit,
            openers,
            sources,
            counted,
            notes,
        },
        cost_by_seat,
    }
}
