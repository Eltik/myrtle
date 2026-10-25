//! SilverAsh the Reignfrost (`char_1045_svash2`), Vanguard / Tactician-Counsellor
//! (`counsellor`), base attack time 1.2 s, Physical, blocks 2. Hand-written:
//! `ArknightsDpsCompare` has no class for him. Written from the EN game data
//! (`character_table`, `skill_table`), checked 2026-10-25. He has no module.
//!
//! Game data behind each number:
//! - Talents: "Open Game" (starting SP, Redeployment Time, the Eye of the Blizzard) and
//!   "Snow Realm Trailblazer" (Kjerag DEF, HP regen, Frozen immunity). No damage.
//! - S1 `skchr_svash2_1` "Pawn Break": DP and Deployment Waiting Zone effects only;
//!   his damage is his normal attacks.
//! - S2 `skchr_svash2_2` "Countergambit" (17 SP over time, no duration, 2 charges),
//!   `skill_parameters`: `[0]` `cost` 11, `[1]` `atk_scale` 3.8, `[2]` `cold` 6, `[3]`
//!   `max_target` 6. 380% ATK Physical to 6 enemies in front.
//! - S3 `skchr_svash2_3` "Castled King" (48 s): `[4]` `bird_atk_scale` 2.0, `[5]`
//!   `damage_scale` 1.3 (`[9]` `damage_scale_display` 0.3). Attacks deal 200% ATK as
//!   Physical damage and inflict 30% Fragile to the targets in a straight line.
//!
//! Conditionals:
//! - skill `withFragile` (ON, S3): his targets carry his own 30% Fragile, which does not
//!   stack with external Fragile (the larger applies, the upstream convention). OFF: the
//!   first hit, before the Fragile lands.
//!
//! ASSUMPTIONS and simplifications:
//! - S2 is manual (`SkillType` MANUAL) and assumed cast every time it is ready: one
//!   380% ATK hit to min(targets, 6) enemies per (SP cost + 1.2 s lockout) / (1 + SP
//!   boost), on top of his normal attacks, as the upstream instant-skill convention
//!   (Gracebearer S2). The second charge only banks a cast; it does not change the
//!   long-run rate.
//! - S3: every enemy in the line is hit, so the damage scales with the target count
//!   (no `max_target` key; arknights.wiki.gg: a 3-tile projectile, 0.8 hitbox).
//!   `weak[limit]` 2 is read as the Fragile's 2 s duration (arknights.wiki.gg and PRTS
//!   https://prts.wiki/w/凛御银灰 give the S3 Fragile 2 s, not stacking), not a target
//!   or stack cap: at 1.2 s per attack the Fragile stays up on every enemy in the line.
//!   That reading is inferred; if it were a 2-target cap, `withFragile` ON overstates
//!   the 3rd and later targets by x1.3.
//! - NOT modelled: the S2 casts by allies deployed left of the Eye of the Blizzard (their
//!   own ATK), and the Eye of the Blizzard token.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};

/// Upstream SP lockout after an instant skill (Gracebearer S2, Vigil S2).
const SP_LOCKOUT: f64 = 1.2;

// The `Option` is the registry signature every DPS formula shares.
#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let defense = enemy.defense;
    let sp = |i: usize, d: f64| unit.skill_parameters.get(i).copied().unwrap_or(d);

    let atk_interval = f64::from(unit.attack_interval);
    let final_atk = unit.atk * (1.0 + unit.buff_atk) + unit.buff_atk_flat;
    let phys = |scale: f64| (final_atk * scale - defense).max(final_atk * scale * 0.05);
    let attack_rate = unit.attack_speed / 100.0 / atk_interval;

    let dps = if skill == 3 {
        let mut fragile = if unit.skill_damage {
            sp(5, 1.0) - 1.0
        } else {
            0.0
        };
        fragile = fragile.max(unit.buff_fragile);
        let hitdmg = phys(sp(4, 1.0)) * (1.0 + fragile) / (1.0 + unit.buff_fragile);
        hitdmg * attack_rate * f64::from(unit.targets)
    } else if skill == 2 {
        let skilldmg = phys(sp(1, 1.0));
        let cycle = (f64::from(unit.skill_cost) + SP_LOCKOUT) / (1.0 + f64::from(unit.sp_boost));
        phys(1.0) * attack_rate + skilldmg * f64::from(unit.targets).min(sp(3, 1.0)) / cycle
    } else {
        phys(1.0) * attack_rate
    };

    Some(dps)
}
