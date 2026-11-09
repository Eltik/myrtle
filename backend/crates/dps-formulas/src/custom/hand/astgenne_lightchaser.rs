//! Astgenne the Lightchaser (`char_1047_halo2`), Supporter / Decel Binder, Arts,
//! base attack time 1.9 s. Hand-written: `ArknightsDpsCompare` has no class for her.
//!
//! Game data (EN `character_table` / `skill_table` / `battle_equip_table`, read
//! 2026-10-25):
//!
//! - Trait (`slower`): attacks deal Arts damage and Slow. Normal attacks hit one target.
//! - Talent 1 "Data Modeling", `talent1_parameters`: `[0]` `attack_speed` 1, `[1]`
//!   `max_stack_cnt` (12 at E1, 18 at E2). Module `uniequip_002_halo2` overrides it to
//!   23 (level 2) / 25 (level 3), and level 3 adds a hidden talent part (`10_root`,
//!   `atk` 0.12, "+12% ATK at full stacks") that the loader files under
//!   `talent1_module_extra`; it is read from there, not hard-coded.
//! - Talent 2 "Energy Source Analysis" (E2), `talent2_parameters`: `[0]`
//!   `halo2_t_1[weak].damage_scale` 1.10, `[1]` `.interval` 7, `[2]` `.damage_scale_add`
//!   0.04, `[3]` `.damage_scale_max` 1.14 (P5: 1.12 / 1.16). Enemies in her range take
//!   10% Fragile, 14% after 7 s in range.
//! - S1 `skchr_halo2_1`: `[0]` `atk` 1.0, `[1]` `attack@chain.max_target` 3,
//!   `[2]` `attack@projectile_range`.
//! - S2 `skchr_halo2_2` (offensive SP, 3): `[0]` `atk_scale` 1.45, `[1]` `sluggish`,
//!   `[2]` `max_target` 3, `[3]` `force`, `[4]` `ability_range_radius`, `[5]`
//!   `atk_scale_link` 1.45, `[6]` `max_target_attack` 2.
//! - S3 `skchr_halo2_3`: `[0]` `atk` 0.7, `[1]` `base_attack_time` -0.7, `[2]`
//!   `attack@max_target` 2, `[3]` `attack@atk_share` 0.25.
//!
//! Conditionals:
//! - `talent_damage` (`maxSlowStacks`, ON): talent 1 at full stacks, +`attack_speed` x
//!   `max_stack_cnt` ASPD (and, module level 3, +12% ATK). OFF: no stacks. Same reading
//!   as the generated base Astgenne (`aspd = talent1[0] * talent1[2]` under
//!   `talent_dmg`).
//! - `talent2_damage` (`fragileAfter7s`, ON): the enemy has been in range for 7 s,
//!   Fragile = `damage_scale_max` - 1. OFF: `damage_scale` - 1.
//!
//! ASSUMPTIONS and simplifications:
//! - Talent 2 Fragile does not stack with other Fragile: the larger of it and
//!   `buff_fragile` applies, then the engine's own `1 + buff_fragile` is divided back
//!   out (the generated Suzuran convention).
//! - S1 (CN text "攻击可在敌人间重复跳跃，最多跳跃3次"): with 2 or more targets an attack
//!   lands 1 + 3 hits, each at full damage. PRTS (<https://prts.wiki/w/溯光星源>): jump
//!   radius 1.7, a jump prefers a target not yet chosen, then the nearest one, so with
//!   2 enemies the chain goes A-B-A-B. No source names a decay and the blackboard has no
//!   decay key. UPPER BOUND: every jump is assumed to find an enemy within 1.7 tiles.
//!   With one target it lands once.
//! - S2: the skill attack hits `max_target_attack` primary targets at `atk_scale` and
//!   links up to `max_target` further enemies per primary target ("分别链接") at
//!   `atk_scale_link`; every enemy is hit at most once, so the skill attack hits
//!   `min(targets, 2 + 2 * 3)` enemies. One skill attack every `skill_cost` + 1
//!   attacks (offensive SP, 1 SP per attack); the slow and pull are not modelled.
//! - S3: the attack hits `min(targets, 2)` locked enemies; each also receives
//!   `atk_share` of each other locked enemy's hit. PRTS and arknights.wiki.gg: the share
//!   is taken from the hit before RES and before any damage multiplier, and is dealt as
//!   ordinary sourceless Arts damage, so the RECEIVING enemy's RES applies to it once;
//!   shared damage is not shared again. With every target at the enemy's RES this is
//!   `atk_share` x the post-RES hit, which is what the formula computes (the 5% floor
//!   scales the same way). Fragile applies to it on the receiving enemy as to any
//!   hit.
//! - Talent 1 stacks are assumed already at the cap (conditional ON), including S0.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};

// The `Option` is the registry signature every DPS formula shares.
#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let res = enemy.res;
    let sp = |i: usize| unit.skill_parameters.get(i).copied().unwrap_or(0.0);
    let mut atk_interval = f64::from(unit.attack_interval);
    let mut atkbuff: f64 = 0.0;

    // Talent 1: ASPD stacks from inflicting Slow.
    let mut aspd: f64 = 0.0;
    if unit.talent_damage && unit.elite > 0 {
        aspd = unit.talent1_parameters.first().copied().unwrap_or(0.0)
            * unit.talent1_parameters.get(1).copied().unwrap_or(0.0);
        // Module level 3: +ATK at full stacks (hidden talent part of uniequip_002).
        if unit.module_index == 1 && unit.module_level == 3 {
            atkbuff += unit
                .data
                .talent1_module_extra
                .iter()
                .find(|e| e.required_module_level == 3)
                .and_then(|e| e.talent_data.first().copied())
                .unwrap_or(0.0);
        }
    }

    // Talent 2: Fragile on every enemy in range (E2).
    let mut fragile: f64 = 0.0;
    if unit.elite == 2 {
        let idx = if unit.talent2_damage { 3 } else { 0 };
        fragile = unit.talent2_parameters.get(idx).copied().unwrap_or(1.0) - 1.0;
    }
    fragile = fragile.max(unit.buff_fragile);

    let targets = f64::from(unit.targets);
    let mut skill_atk_scale: f64 = 1.0;
    let mut link_scale: f64 = 1.0;
    let mut hits: f64 = 1.0;
    let mut share: f64 = 0.0;
    if skill == 1 {
        atkbuff += sp(0);
        if targets >= 2.0 {
            hits = 1.0 + sp(1);
        }
    }
    if skill == 2 {
        skill_atk_scale = sp(0);
        link_scale = sp(5);
    }
    if skill == 3 {
        atkbuff += sp(0);
        atk_interval += sp(1);
        hits = targets.min(sp(2));
        share = sp(3);
    }

    let final_atk = unit.atk * (1.0 + atkbuff + unit.buff_atk) + unit.buff_atk_flat;
    let arts = |scale: f64| (final_atk * scale * (1.0 - res / 100.0)).max(final_atk * scale * 0.05);
    let hitdmg = arts(1.0);
    let mut avghit = hitdmg * hits * (1.0 + share * (hits - 1.0));

    if skill == 2 {
        let primary = targets.min(sp(6));
        let linked = (targets - primary).max(0.0).min(sp(6) * sp(2));
        let skilldmg = primary * arts(skill_atk_scale) + linked * arts(link_scale);
        let sp_cost = f64::from(unit.skill_cost);
        avghit = (sp_cost * hitdmg + skilldmg) / (sp_cost + 1.0);
    }

    let dps = avghit / atk_interval * (unit.attack_speed + aspd) / 100.0;
    Some(dps * (1.0 + fragile) / (1.0 + unit.buff_fragile))
}
