use std::collections::HashMap;

use myrtle_gamedata::index::squad::{StatScale, TalentText, TextFacts};

use crate::{
    Ctx,
    build::{Board, Seat, bb},
    targets::fits,
    types::{
        Cut, CutKind, DpSource, Economy, EconomyBonus, EconomyNote, EconomyNoteKind, EconomySeat,
        NoteStat, SourceKind, SourceRef, SquadAnalyzeRequest, SynergyCond,
    },
};

pub struct EconomyOut {
    pub economy: Economy,
    pub cost_by_seat: HashMap<i32, i32>,
}

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

const fn is_self(t: &TalentText) -> bool {
    t.opens_with_own_stat || t.target.is_none()
}

pub fn talent_gains_dp(f: &TextFacts, board: &Board) -> bool {
    bb(board, "cost").is_some_and(|c| c > 0.0)
        && f.talent.as_ref().is_some_and(|t| t.speaks_of_dp_gain)
}

fn talent_start_bonus(f: &TextFacts, board: &Board) -> bool {
    talent_gains_dp(f, board) && f.talent.as_ref().is_some_and(|t| t.in_squad)
}

pub fn skill_dp(board: &Board, facts: Option<&TextFacts>) -> Option<f64> {
    if let Some(f) = facts {
        let total: f64 = f
            .dp_gain
            .iter()
            .filter_map(|k| board.iter().find(|(bk, _)| bk.eq_ignore_ascii_case(k)))
            .map(|(_, v)| *v)
            .filter(|v| *v > 0.0)
            .sum();
        return (total > 0.0).then_some(total);
    }
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
        stat: None,
        ratio: None,
    }
}

fn cost_note(kind: EconomyNoteKind, s: &Seat<'_>, name: &str, amount: f64) -> EconomyNote {
    EconomyNote {
        stat: Some(NoteStat::Cost),
        ..note(kind, s, name, Some(amount))
    }
}

fn respawn_note(
    kind: EconomyNoteKind,
    s: &Seat<'_>,
    name: &str,
    v: f64,
    text: Option<&TalentText>,
) -> EconomyNote {
    EconomyNote {
        stat: Some(NoteStat::Respawn),
        ratio: respawn_change(v, text).1,
        ..note(kind, s, name, Some(v))
    }
}

fn apply(base: f64, cuts: &[Cut]) -> f64 {
    let sum: f64 = cuts.iter().map(|c| c.amount).sum();
    let mult: f64 = cuts
        .iter()
        .filter_map(|c| c.ratio)
        .map(|r| 1.0 + r)
        .product();
    ((base + sum) * mult).max(0.0)
}

fn cost_after(base_cost: i32, cuts: &[Cut]) -> i32 {
    apply(f64::from(base_cost), cuts).trunc() as i32
}

#[allow(clippy::too_many_lines)]
pub fn compute(ctx: &Ctx<'_>, seats: &[Seat<'_>], req: &SquadAnalyzeRequest) -> EconomyOut {
    let mut rows: Vec<EconomySeat> = Vec::new();
    let mut notes: Vec<EconomyNote> = Vec::new();
    let mut counted: Vec<EconomyNote> = Vec::new();
    let mut start_bonus: Vec<EconomyBonus> = Vec::new();
    let mut sources: Vec<DpSource> = Vec::new();
    let mut dp_lower_limit = 0.0;

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
            let Some(tt) = ctx.facts(&t.key).and_then(|f| f.talent.as_ref()) else {
                continue;
            };
            if !is_self(tt) {
                continue;
            }
            if let Some(c) = bb(&t.board, "cost").filter(|c| *c < 0.0)
                && tt.says_dp_cost {
                    cuts.push(cut(CutKind::Talent, Some(t.name.clone()), None, (c, None)));
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

        for hb in &s.hidden_boards {
            let Some(v) = bb(hb, "respawn_time").filter(|v| *v != 0.0) else {
                continue;
            };
            let owner = s.talents.iter().find_map(|t| {
                let tt = ctx.facts(&t.key)?.talent.as_ref()?;
                Some((t, tt.own_redeploy_scale(v)?))
            });
            if let Some((t, scale)) = owner {
                let change = match scale {
                    StatScale::Flat => (v, None),
                    StatScale::Ratio if v < 0.0 => (0.0, Some(v)),
                    StatScale::Ratio => (0.0, Some(v - 1.0)),
                };
                respawn_cuts.push(cut(CutKind::Talent, Some(t.name.clone()), None, change));
            }
        }

        let mut first_respawn_cuts = Vec::new();
        if ctx.trait_facts(s).is_some_and(|f| f.first_redeploy)
            && let Some(v) = bb(&s.trait_board, "respawn_time").filter(|v| *v != 0.0)
        {
            first_respawn_cuts.push(cut(CutKind::Trait, None, None, respawn_change(v, None)));
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
            first_respawn: None,
            first_respawn_cuts,
        });
    }

    let own_cost: HashMap<i32, i32> = rows
        .iter()
        .map(|r| (r.seat, cost_after(r.base_cost, &r.cuts)))
        .collect();

    let mut sp_bonus: HashMap<i32, f64> = HashMap::new();
    let mut sp_givers: Vec<(&Seat<'_>, String, f64, Vec<i32>)> = Vec::new();

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
            let value_cut = bb(&t.board, "value").filter(|v| *v < 0.0 && tt.says_dp_cost);
            if let Some(v) = value_cut {
                let kind = if tt.next_operator {
                    EconomyNoteKind::NextOperator
                } else if cond == SynergyCond::Field || cond == SynergyCond::Skill {
                    EconomyNoteKind::OnceDeployed
                } else {
                    EconomyNoteKind::Conditional
                };
                notes.push(cost_note(kind, g, &t.name, v));
            }
            if let Some(v) = bb(&t.board, "runtime_cost").filter(|v| *v < 0.0) {
                notes.push(cost_note(EconomyNoteKind::FirstDeploy, g, &t.name, v));
            }
            let floor = bb(&t.board, "cost").filter(|c| *c < 0.0 && tt.lowers_dp_floor);
            if let Some(c) = floor {
                if cond == SynergyCond::Squad {
                    dp_lower_limit += c;
                    counted.push(note(EconomyNoteKind::DpFloor, g, &t.name, Some(c)));
                } else {
                    notes.push(note(EconomyNoteKind::OnceDeployed, g, &t.name, Some(c)));
                }
            }
            if is_self(tt) {
                continue;
            }
            let Some(target) = tt.target.as_ref().filter(|t| !t.unseen) else {
                continue;
            };
            let receivers: Vec<i32> = seats
                .iter()
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

            if let Some(c) = bb(&t.board, "cost").filter(|c| *c < 0.0 && floor.is_none()) {
                if cond == SynergyCond::Squad {
                    let given = cut(
                        CutKind::Squad,
                        Some(t.name.clone()),
                        Some(g.seat),
                        (c, None),
                    );
                    give_cut(&mut rows, &receivers, |r| &mut r.cuts, &given);
                    if !receivers.is_empty() {
                        counted.push(cost_note(EconomyNoteKind::CostCut, g, &t.name, c));
                    }
                } else {
                    notes.push(cost_note(EconomyNoteKind::OnceDeployed, g, &t.name, c));
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
                        counted.push(respawn_note(
                            EconomyNoteKind::RedeployCut,
                            g,
                            &t.name,
                            v,
                            Some(tt),
                        ));
                    }
                } else {
                    let kind = if cond == SynergyCond::Field {
                        EconomyNoteKind::OnceDeployed
                    } else {
                        EconomyNoteKind::Conditional
                    };
                    notes.push(respawn_note(kind, g, &t.name, v, Some(tt)));
                }
            }
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

    for s in seats {
        let trait_refund = ctx.trait_facts(s).is_some_and(|f| f.refund);
        let trait_cost = bb(&s.trait_board, "cost");
        let trait_interval = bb(&s.trait_board, "interval");
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
            let interval = if interval > 0.0 { interval } else { 1.0 };
            notes.push(note(EconomyNoteKind::Drain, s, "", Some(c / interval)));
        }
        if let Some(sk) = &s.skill
            && let Some(f) = ctx.facts(&sk.key)
        {
            for (key, minus) in &f.dp_cost_cut {
                let Some((_, v)) = sk.board.iter().find(|(k, _)| k.eq_ignore_ascii_case(key))
                else {
                    continue;
                };
                let amount = if *minus { -v } else { *v };
                if amount < 0.0 {
                    notes.push(cost_note(
                        EconomyNoteKind::SkillActive,
                        s,
                        &sk.level.name,
                        amount,
                    ));
                }
            }
        }
        if let Some(sk) = &s.skill
            && let Some(amount) = skill_dp(&sk.board, ctx.facts(&sk.key))
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
        if !r.first_respawn_cuts.is_empty() {
            let all: Vec<Cut> = r
                .respawn_cuts
                .iter()
                .chain(&r.first_respawn_cuts)
                .cloned()
                .collect();
            r.first_respawn = Some(apply(r.base_respawn, &all));
        }
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
    let dp_lower_limit = dp_lower_limit.trunc() as i32;
    let openers = i32::try_from(
        rows.iter()
            .filter(|r| r.cost <= start_dp - dp_lower_limit)
            .count(),
    )
    .unwrap_or(0);

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
            dp_lower_limit,
            openers,
            sources,
            counted,
            notes,
        },
        cost_by_seat,
    }
}
