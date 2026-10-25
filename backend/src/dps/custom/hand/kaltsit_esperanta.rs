//! Kal'tsit Esperanta (`char_1052_kalts2`), Medic (`watchman`), base attack time
//! 2.85 s, no module. Hand-written: `ArknightsDpsCompare` has no class for her.
//!
//! Game data (EN `character_table` / `skill_table`, read 2026-10-25):
//!
//! - Trait: restores HP of allies (one target per attack) and can Take Off.
//! - Talent 1 "Sentinelo de Polvo": Max HP / DEF / Block only. Talent 2 "Majstraĵo de
//!   Kuracisto": a 30 s 50 HP/s (P5: 60) regeneration buff on allies when they ENTER
//!   her range, doubled on Rhodes Island operators. Neither is modelled (below).
//! - S1 `skchr_kalts2_1`: `[0]` `atk` 0.9, `[1]` `attack@block_radius_scale`, `[2]`
//!   `attack_speed` 50.
//! - S2 `skchr_kalts2_2` (AMMO, 10): `[0]` `atk` 1.5, `[1]` `attack@atk_scale` 3.8
//!   (True damage to all enemies around the target), `[2]` `attack@heal_scale` 2.0
//!   (heal to all allies around the target), `[3]` `attack@sluggish`, `[4]`
//!   `attack@trigger_time` 10 (ammo).
//! - S3 `skchr_kalts2_3`: `[0]` `atk` 1.5, `[1]` `base_attack_time` -1.55, heals 1
//!   additional target (2).
//!
//! DPS: only S2 deals damage (each medical unit, "prioritizing enemies"); every other
//! skill and S0 return 0.
//!
//! ASSUMPTIONS and simplifications:
//! - S2 DPS: every attack targets an enemy; all `targets` enemies stand around it
//!   (AoE, uncapped). True damage, so DEF and RES do not apply.
//! - S2 duration: the game data has none (ammo, `Duration` -1). One ammo per attack, so
//!   `skill_duration` reports the 10 ammo at the S2 attack interval (10 x 2.85 s at
//!   ASPD 100), for DPS and HPS alike: `total_damage` is S2 DPS x that duration, and
//!   the cycle averages cycle over it.
//! - S2 HPS: every attack heals `targets` allies around the target (uncapped) by
//!   `heal_scale` x ATK, and replaces her normal heal. UPPER BOUND: `targets` stands
//!   for the healed-ally count too; PRTS (https://prts.wiki/w/凯尔希·思衡托) gives the
//!   heal a 1.5-tile radius around the enemy the unit targets (first the one she
//!   blocks), so in practice it reaches the allies blocking it, usually fewer than the
//!   enemies hit. The cycle average uses the skill
//!   duration plus `skill_cost` over `1 + sp_boost` of base healing (the generated
//!   healers' average).
//! - Talent 2's regeneration is not modelled: it starts only when an ally enters her
//!   range and lasts 30 s per entry, so it is not a sustained rate. PRTS confirms it is
//!   a timed aura re-granted on each entry (her deployment, an ally deployed in range,
//!   S2's range growth, S3's move), not permanent.
//! - `buff_fragile` scales healing as in the generated HPS formulas.

use super::super::super::engine::HpsResult;
use super::super::super::operator_unit::{EnemyStats, OperatorUnit};

/// S2: `trigger_time` ammo, one spent per attack, at the S2 attack interval (S2
/// changes neither the interval nor the attack speed). Every other skill keeps its
/// game-data duration.
pub fn skill_duration(unit: &OperatorUnit) -> Option<f64> {
    if unit.skill_index != 2 {
        return None;
    }
    let ammo = unit.skill_parameters.get(4).copied().unwrap_or(0.0);
    Some(ammo * f64::from(unit.attack_interval) / (unit.attack_speed / 100.0))
}

// The `Option` is the registry signature every DPS formula shares.
#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, _enemy: &EnemyStats) -> Option<f64> {
    if unit.skill_index != 2 {
        return Some(0.0);
    }
    let sp = |i: usize| unit.skill_parameters.get(i).copied().unwrap_or(0.0);
    let atk_interval = f64::from(unit.attack_interval);
    let final_atk = unit.atk * (1.0 + sp(0) + unit.buff_atk) + unit.buff_atk_flat;
    let hitdmg = final_atk * sp(1);
    Some(hitdmg / atk_interval * unit.attack_speed / 100.0 * f64::from(unit.targets))
}

#[allow(clippy::unnecessary_wraps)]
pub fn hps(unit: &OperatorUnit, _enemy: &EnemyStats) -> Option<HpsResult> {
    let skill = unit.skill_index;
    let sp = |i: usize| unit.skill_parameters.get(i).copied().unwrap_or(0.0);
    let base_interval = f64::from(unit.attack_interval);
    let targets = f64::from(unit.targets);
    let heal_mult = 1.0 + unit.buff_fragile;

    let final_atk = unit.atk * (1.0 + unit.buff_atk) + unit.buff_atk_flat;
    let base_hps =
        final_atk / base_interval * unit.attack_speed / 100.0 * heal_mult * targets.min(1.0);

    if skill == 0 {
        return Some(HpsResult {
            skill_hps: base_hps,
            base_hps,
            avg_hps: base_hps,
        });
    }

    let mut atk_interval = base_interval;
    let mut aspd: f64 = 0.0;
    let mut heal_scale: f64 = 1.0;
    let mut heal_targets = targets.min(1.0);
    if skill == 1 {
        aspd = sp(2);
    }
    if skill == 2 {
        heal_scale = sp(2);
        heal_targets = targets;
    }
    if skill == 3 {
        atk_interval += sp(1);
        heal_targets = targets.min(2.0);
    }

    let final_atk_skill = unit.atk * (1.0 + sp(0) + unit.buff_atk) + unit.buff_atk_flat;
    let skill_hps = final_atk_skill * heal_scale / atk_interval * (unit.attack_speed + aspd)
        / 100.0
        * heal_mult
        * heal_targets;

    let duration = unit.skill_duration;
    let sp_time = f64::from(unit.skill_cost) / (1.0 + f64::from(unit.sp_boost));
    let avg_hps = (skill_hps * duration + base_hps * sp_time) / (duration + sp_time);

    Some(HpsResult {
        skill_hps,
        base_hps,
        avg_hps,
    })
}
