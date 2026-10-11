//! The DPS calculator hand-off: per seat, the ATK, flat ATK and ASPD buffs that
//! squadmates' talents give it by being in the squad or on the field.
//!
//! Reads the giving talent's blackboard `atk` and `attack_speed`. Whether `atk`
//! is a ratio or flat ATK is how the talent's text prints it
//! (`TalentText::scale_of`); a value the text does not print is not the ATK
//! stat and is left out. Only a talent with no English reading falls back to
//! the magnitude (|v| < 1 a ratio, larger flat ATK). Placement-bound and skill-bound effects,
//! effects with an unmet count, and effects with none of those keys are
//! listed as left out.

use super::Ctx;
use super::build::{Seat, bb};
use super::index::StatScale;
use super::types::{
    HandoffBuffs, HandoffLeftOut, HandoffLeftOutReason, HandoffSource, SeatHandoff, Synergy,
    SynergyCond, SynergyReason, SynergyTargetKind,
};

pub fn compute(ctx: &Ctx<'_>, seats: &[Seat<'_>], synergy: &Synergy) -> Vec<SeatHandoff> {
    seats
        .iter()
        .map(|s| {
            let mut buffs = HandoffBuffs::default();
            let mut counted = Vec::new();
            let mut left_out = Vec::new();
            for item in &synergy.items {
                // A buff to an ally the game picks ("a random ally") may land
                // on any other seat: listed as Conditional, never counted.
                let random = item.reason == Some(SynergyReason::Unseen)
                    && item.target.kind == SynergyTargetKind::Everyone;
                if item.seat != s.seat && random {
                    left_out.push(HandoffLeftOut {
                        from_seat: item.seat,
                        char_id: item.char_id.clone(),
                        name: item.name.clone(),
                        reason: HandoffLeftOutReason::Conditional,
                    });
                    continue;
                }
                if item.seat == s.seat || !(item.to_all || item.to.contains(&s.seat)) {
                    continue;
                }
                let left = |reason| HandoffLeftOut {
                    from_seat: item.seat,
                    char_id: item.char_id.clone(),
                    name: item.name.clone(),
                    reason,
                };
                if item.need.as_ref().is_some_and(|n| n.have < n.count) {
                    left_out.push(left(HandoffLeftOutReason::Conditional));
                    continue;
                }
                match item.cond {
                    SynergyCond::Near => {
                        left_out.push(left(HandoffLeftOutReason::Placement));
                        continue;
                    }
                    SynergyCond::Skill => {
                        left_out.push(left(HandoffLeftOutReason::Skill));
                        continue;
                    }
                    SynergyCond::Squad | SynergyCond::Field => {}
                }
                let Some(giver) = seats.iter().find(|g| g.seat == item.seat) else {
                    continue;
                };
                let Some(t) = giver.talents.iter().find(|t| t.name == item.name) else {
                    continue;
                };
                let atk = bb(&t.board, "atk").unwrap_or(0.0);
                let text = ctx.facts(&t.key).and_then(|f| f.talent.as_ref());
                let scale = match text {
                    Some(tt) => tt.scale_of(atk, false),
                    // FALLBACK: no English reading of this talent at all.
                    None if atk.abs() < 1.0 => Some(StatScale::Ratio),
                    None => Some(StatScale::Flat),
                };
                let (ratio, flat) = match scale {
                    Some(StatScale::Ratio) => (atk, 0),
                    Some(StatScale::Flat) => (0.0, atk.trunc() as i32),
                    None => (0.0, 0),
                };
                let aspd = bb(&t.board, "attack_speed").unwrap_or(0.0).trunc() as i32;
                if ratio == 0.0 && flat == 0 && aspd == 0 {
                    left_out.push(left(HandoffLeftOutReason::NotAStat));
                    continue;
                }
                buffs.atk += ratio;
                buffs.flat_atk += flat;
                buffs.aspd += aspd;
                counted.push(HandoffSource {
                    from_seat: item.seat,
                    char_id: item.char_id.clone(),
                    name: item.name.clone(),
                    cond: item.cond,
                    atk: ratio,
                    flat_atk: flat,
                    aspd,
                });
            }
            SeatHandoff {
                seat: s.seat,
                char_id: s.char_id.clone(),
                buffs,
                counted,
                left_out,
            }
        })
        .collect()
}
