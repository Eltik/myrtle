//! Angelina the Mellow Wish (`char_1015_aglna2`), Specialist / Skyranger, base
//! interval 1.5 s. Hand-written: `ArknightsDpsCompare` has no class for her.
//!
//! Moved here VERBATIM in arithmetic from `generated.rs` (commit 67f699a3), where a
//! regeneration deleted it. Only the variable declarations changed (each local is
//! bound where it is computed instead of zero-initialised and overwritten); every
//! floating-point operation runs in the same order, so results are bit-identical
//! (checked by `tests/dps_table_dump_test.rs` against CN data).
//!
//! `skill_parameters` are the raw blackboard order:
//! - S1: `[0]` `atk` 1.35, `[1]` `attack@max_target` 2
//! - S2: `[0]` `atk` 1.65, `[1]` `base_attack_time` -0.8, `[2]` `attack@max_target` 5
//!   (Arts), `[3]` `chant_duration` 2.5
//! - S3: `[0]` `attack@atk_scale` 3.80, `[1]` `attack@max_target` 4, `[5]` `atk` 0.30,
//!   `[9]` `attack@trigger_time` 33 (ammo)
//!
//! `talent1_parameters`: `[0]` `atk_scale_hi` (mass <= 3), `[1]` `atk_scale_lo`.
//! `talent2_parameters`: `[0]` `atk`.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};

// The `Option` is the registry signature every DPS formula shares.
#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let defense = enemy.defense;
    let res = enemy.res;
    let mut atk_interval = f64::from(unit.attack_interval);
    let mut atkbuff: f64 = 0.0;
    let mut atk_scale: f64 = 1.0;
    let mut maxtargets: f64 = 1.0;
    let mut arts_attack = false;
    let mut uptime_scale: f64 = 1.0;

    // Talent 1 adds Arts damage to every attack. The mass <= 3 branch is the normal
    // case while she is airborne, because the same talent applies Weightless (-1 mass
    // level) to everything in range; talent_damage = false models a heavy target.
    let talent_scale = if unit.talent_damage {
        unit.talent1_parameters.first().copied().unwrap_or(0.0)
    } else {
        unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
    };

    // Talent 2 buffs every ally in Liftoff, and she is one in all three skills.
    if unit.talent2_damage {
        atkbuff += unit.talent2_parameters.first().copied().unwrap_or(0.0);
    }

    if skill == 1 {
        atkbuff += unit.skill_parameters.first().copied().unwrap_or(0.0);
        maxtargets = unit.skill_parameters.get(1).copied().unwrap_or(1.0);
    }
    if skill == 2 {
        atkbuff += unit.skill_parameters.first().copied().unwrap_or(0.0);
        atk_interval += unit.skill_parameters.get(1).copied().unwrap_or(0.0);
        maxtargets = unit.skill_parameters.get(2).copied().unwrap_or(1.0);
        arts_attack = true;
        // She glides forward for chant_duration before the shortened interval kicks
        // in, so a 22s cast only lands 19.5s of hits. Scaling here keeps the engine's
        // total_damage / average_dps honest; drop this for raw "DPS while attacking".
        let chant = unit.skill_parameters.get(3).copied().unwrap_or(0.0);
        if unit.skill_duration > 0.0 && chant > 0.0 {
            uptime_scale = ((unit.skill_duration - chant) / unit.skill_duration).clamp(0.0, 1.0);
        }
    }
    if skill == 3 {
        atk_scale = unit.skill_parameters.first().copied().unwrap_or(1.0);
        atkbuff += unit.skill_parameters.get(5).copied().unwrap_or(0.0);
        maxtargets = unit.skill_parameters.get(1).copied().unwrap_or(1.0);
    }

    let final_atk = unit.atk * (1.0 + atkbuff + unit.buff_atk) + unit.buff_atk_flat;

    let hitdmg = if arts_attack {
        (final_atk * atk_scale * (1.0 - res / 100.0)).max(final_atk * atk_scale * 0.05)
    } else {
        (final_atk * atk_scale - defense).max(final_atk * atk_scale * 0.05)
    };
    let hitdmgarts =
        (final_atk * talent_scale * (1.0 - res / 100.0)).max(final_atk * talent_scale * 0.05);

    let dps = (hitdmg + hitdmgarts) / atk_interval * unit.attack_speed / 100.0
        * f64::from(unit.targets).min(maxtargets)
        * uptime_scale;

    Some(dps)
}
