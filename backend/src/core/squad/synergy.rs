//! The Synergy panel: squad seats' talents (module-upgraded where the build
//! has the module) that reach someone other than their holder.
//!
//! Who a talent reaches and when is the load-time English reading
//! (`index::TalentText`, vocabulary in `targets`), applied by id to the
//! reader's server. A support is never a giver (challenge rules count the
//! squad), but it is a receiver. A talent with no English reading comes back
//! as a Check item with reason `Unread`, never silently dropped. Of a chosen
//! skill only the ally faction its text names is read
//! (`TextFacts::faction`, Nearl the Radiant Knight S2 "If the previously
//! deployed Operator is from Kazimierz"), as a Skill item.

use std::collections::HashMap;

use super::Ctx;
use super::build::Seat;
use super::index::TalentText;
use super::targets::{Target, fits};
use super::types::{
    Synergy, SynergyCond, SynergyCounts, SynergyGroup, SynergyItem, SynergyNeed, SynergyReason,
    SourceRef, SynergyTarget, SynergyTargetKind,
};

/// The target in the reader's server's words where its tables name it.
fn label(ctx: &Ctx<'_>, t: &Target) -> String {
    let data = ctx.data;
    let v = t.value.as_deref().unwrap_or("");
    let named = match t.kind {
        SynergyTargetKind::Class => data.profession_names.get(v).cloned(),
        SynergyTargetKind::Faction => data
            .factions
            .get(v)
            .map(|f| f.power_name.clone())
            .filter(|n| !n.is_empty()),
        SynergyTargetKind::Tag => v
            .parse::<i32>()
            .ok()
            .and_then(|id| data.squad.tags.name(id))
            .map(str::to_string),
        SynergyTargetKind::Position => super::targets::POSITION_TAG_IDS
            .iter()
            .find(|(code, _)| *code == v)
            .and_then(|(_, id)| data.squad.tags.name(*id))
            .map(str::to_string),
        _ => None,
    };
    named.unwrap_or_else(|| t.label.clone())
}

/// One effect of a giver: a talent, or a chosen skill's faction reading.
struct Effect<'t> {
    name: String,
    source: SourceRef,
    text: String,
    cond: SynergyCond,
    /// The talent reading, for its count and "only the highest" rules.
    talent: Option<&'t TalentText>,
}

/// The item for `effect` of giver `g` aimed at `target`: who it reaches here
/// and which section it lands in.
fn reach(
    ctx: &Ctx<'_>,
    seats: &[Seat<'_>],
    g: &Seat<'_>,
    cost: &impl Fn(&Seat<'_>) -> i32,
    target: &Target,
    effect: Effect<'_>,
) -> SynergyItem {
    let cond = effect.cond;
    // A receiver the game picks ("a random ally") is no seat the page can
    // name.
    let to: Vec<i32> = seats
        .iter()
        .filter(|_| !target.unseen)
        .filter(|r| r.seat != g.seat)
        .filter(|r| fits(target, r, cost(r), &r.tag_ids))
        .map(|r| r.seat)
        .collect();
    let everyone = target.kind == SynergyTargetKind::Everyone;
    let to_all = everyone && !to.is_empty();
    let need = effect.talent.and_then(|tt| tt.need).map(|(count, counts_giver)| {
        let giver = counts_giver && fits(target, g, cost(g), &g.tag_ids);
        SynergyNeed {
            count,
            have: i32::try_from(to.len()).unwrap_or(0) + i32::from(giver),
        }
    });

    let (group, reason) = if to.is_empty() {
        let reason = match target.kind {
            _ if target.unseen => SynergyReason::Unseen,
            SynergyTargetKind::Unknown => SynergyReason::Unknown,
            SynergyTargetKind::Next => SynergyReason::Unseen,
            _ => SynergyReason::Nobody,
        };
        (SynergyGroup::Check, Some(reason))
    } else if need.as_ref().is_some_and(|n| n.have < n.count) {
        (SynergyGroup::Check, Some(SynergyReason::NeedsCount))
    } else {
        let group = match cond {
            SynergyCond::Near => SynergyGroup::Place,
            SynergyCond::Squad => SynergyGroup::Squad,
            SynergyCond::Field | SynergyCond::Skill => SynergyGroup::Field,
        };
        (group, None)
    };

    SynergyItem {
        seat: g.seat,
        char_id: g.char_id.clone(),
        support: g.support,
        name: effect.name,
        source: effect.source,
        cond,
        group,
        text: effect.text,
        target: SynergyTarget {
            kind: target.kind,
            value: target.value.clone(),
            label: label(ctx, target),
        },
        to: if to_all { Vec::new() } else { to },
        to_all,
        reason,
        need,
        highest_only: effect.talent.is_some_and(|tt| tt.highest_only),
    }
}

pub fn compute(ctx: &Ctx<'_>, seats: &[Seat<'_>], cost_by_seat: &HashMap<i32, i32>) -> Synergy {
    let mut items = Vec::new();
    let cost = |s: &Seat<'_>| cost_by_seat.get(&s.seat).copied().unwrap_or(0);
    for g in seats.iter().filter(|s| !s.support) {
        for t in &g.talents {
            let source = t.source();
            let Some(tt) = ctx.facts(&t.key).map(|f| f.talent.as_ref()) else {
                items.push(SynergyItem {
                    seat: g.seat,
                    char_id: g.char_id.clone(),
                    support: g.support,
                    name: t.name.clone(),
                    source,
                    cond: SynergyCond::Field,
                    group: SynergyGroup::Check,
                    text: t.description.clone(),
                    target: SynergyTarget {
                        kind: SynergyTargetKind::Unknown,
                        value: None,
                        label: String::new(),
                    },
                    to: Vec::new(),
                    to_all: false,
                    reason: Some(SynergyReason::Unread),
                    need: None,
                    highest_only: false,
                });
                continue;
            };
            let Some(tt) = tt else { continue };
            let Some(target) = tt.target.as_ref() else {
                continue;
            };
            let effect = Effect {
                name: t.name.clone(),
                source,
                text: t.description.clone(),
                cond: tt.cond,
                talent: Some(tt),
            };
            items.push(reach(ctx, seats, g, &cost, target, effect));
        }
        if let Some(sk) = &g.skill
            && let Some(target) = ctx.facts(&sk.key).and_then(|f| f.faction.as_ref())
        {
            let effect = Effect {
                name: sk.level.name.clone(),
                source: sk.source(),
                text: sk.level.description.clone(),
                cond: SynergyCond::Skill,
                talent: None,
            };
            items.push(reach(ctx, seats, g, &cost, target, effect));
        }
    }

    let mut counts = SynergyCounts::default();
    for i in &items {
        match i.group {
            SynergyGroup::Check => counts.check += 1,
            SynergyGroup::Squad => counts.squad += 1,
            SynergyGroup::Field => counts.field += 1,
            SynergyGroup::Place => counts.place += 1,
        }
    }
    Synergy { items, counts }
}
