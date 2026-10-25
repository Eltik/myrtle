//! Bellone (`char_4037_demetr`), Guard / Fighter, base interval 0.78 s, blocks 1.
//! Hand-written: `ArknightsDpsCompare` has no class for her. Written from the EN game
//! data (`character_table`, `skill_table`, `battle_equip_table`), checked 2026-10-25.
//!
//! `skill_parameters` are the raw blackboard order:
//! - S1 `skchr_demetr_1` (offensive SP, instant): `[0]` `atk_scale` 2.5. "The next
//!   attack deals 250% ATK as Physical damage twice".
//! - S2 `skchr_demetr_2` (22 s): `[0]` `attack@atk_scale` 2.0, `[1]` `attack@max_target`
//!   3, `[2]` `attack_speed` 80.
//! - S3 `skchr_demetr_3` (30 s): `[0]` `attack@demetr_s3[bonus].attack_speed` 50, `[1]`
//!   `...atk` 1.7, `[2]` `...prob_atk_scale` 1.85, `[3]` `...prob` 0.5.
//!
//! `talent1_parameters` (The Way of the Famiglia, E1): `[0]` `attack@def` -0.07 (DEF
//! per stack at E2), `[1]` `attack@limited_stack_cnt` 5, `[2]`
//! `attack@s2_limited_stack_cnt` 8, `[7]` `max_add_on_scale` 0.28 (damage bonus at
//! target HP <= `max_hp_ratio` 0.2). The module overrides it from level 2 (0.42, and
//! DEF -0.08 at level 3) through the engine's talent resolution.
//! Talent 2 (Street Smarts) is Dodge only: not modelled.
//!
//! Module 1 (`uniequip_002_demetr`, FGT-Y): trait `attack_speed` +10 while Bellone is
//! above `hp_ratio` 0.5 HP, read from the module's trait blackboard; gated by the
//! `above50Hp` module conditional.
//!
//! ASSUMPTIONS:
//! - Talent 1 DEF reduction is always at max stacks (5, or 8 during S2): at 0.78 s per
//!   attack she reaches 5 stacks in about 4 s and the 10 s duration refreshes on hit.
//!   Stacks multiply the target's DEF as `defense * (1 - stacks * 0.07)`, the way the
//!   generated formulas apply a self-inflicted percentage shred. PRTS
//!   (https://prts.wiki/w/贝洛内) confirms it: a final multiplier of the text value x
//!   current stacks, applied before the hit's damage, each hit refreshing the 10 s.
//! - Talent 1 damage bonus: `vsLowHp` ON is the maximum bonus (target below 20% HP),
//!   OFF is a full-HP target (no bonus). The linear ramp in between is not modelled.
//! - S1 averages one doubled skill hit per `sp_cost + 1` attacks, as Flint and Hellagur do.
//! - S3: a proc replaces the hit (the proc hit deals 185% ATK instead of 100%). No
//!   source settles replacement against an extra hit; the blackboard key
//!   (`prob_atk_scale`) and arknights.wiki.gg ("a chance to deal increased damage")
//!   read as a replaced multiplier. The
//!   dash, retargeting and the 1 s idle rule are not modelled.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};
use super::util::module_trait_value;

// The `Option` is the registry signature every DPS formula shares.
#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let defense = enemy.defense;
    let atk_interval = f64::from(unit.attack_interval);
    let talent = |i: usize| unit.talent1_parameters.get(i).copied().unwrap_or(0.0);
    let param = |i: usize| unit.skill_parameters.get(i).copied().unwrap_or(0.0);

    // Talent 1 unlocks at E1; below that its slots hold placeholder defaults.
    let (def_per_stack, max_stacks, low_hp_bonus) = if unit.elite > 0 {
        let stacks = if skill == 2 { talent(2) } else { talent(1) };
        (-talent(0), stacks, talent(7))
    } else {
        (0.0, 0.0, 0.0)
    };
    let newdef = defense * (1.0 - def_per_stack * max_stacks).max(0.0);
    let dmg_scale = if unit.talent_damage {
        1.0 + low_hp_bonus
    } else {
        1.0
    };

    let mut aspd = if unit.module_index == 1 && unit.module_damage {
        module_trait_value(unit, "attack_speed").unwrap_or(0.0)
    } else {
        0.0
    };

    let hit =
        |final_atk: f64, scale: f64| (final_atk * scale - newdef).max(final_atk * scale * 0.05);

    let dps = match skill {
        1 => {
            let final_atk = unit.atk * (1.0 + unit.buff_atk) + unit.buff_atk_flat;
            let sp_cost = f64::from(unit.skill_cost);
            let skillhit = hit(final_atk, param(0)) * 2.0;
            let avghit = (sp_cost * hit(final_atk, 1.0) + skillhit) / (sp_cost + 1.0);
            avghit / atk_interval * (unit.attack_speed + aspd) / 100.0
        }
        2 => {
            aspd += param(2);
            let final_atk = unit.atk * (1.0 + unit.buff_atk) + unit.buff_atk_flat;
            hit(final_atk, param(0)) / atk_interval * (unit.attack_speed + aspd) / 100.0
                * f64::from(unit.targets).min(param(1))
        }
        3 => {
            aspd += param(0);
            let final_atk = unit.atk * (1.0 + param(1) + unit.buff_atk) + unit.buff_atk_flat;
            let prob = param(3);
            let avghit = (1.0 - prob) * hit(final_atk, 1.0) + prob * hit(final_atk, param(2));
            avghit / atk_interval * (unit.attack_speed + aspd) / 100.0
        }
        _ => {
            let final_atk = unit.atk * (1.0 + unit.buff_atk) + unit.buff_atk_flat;
            hit(final_atk, 1.0) / atk_interval * (unit.attack_speed + aspd) / 100.0
        }
    };

    Some(dps * dmg_scale)
}
