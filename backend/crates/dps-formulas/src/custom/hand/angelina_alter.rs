//! Angelina the Mellow Wish (`char_1015_aglna2`): `ArknightsDpsCompare` has no
//! class for her. Same arithmetic as the transpiled formula, operation for operation.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};

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

    // The talent's own Weightless keeps most targets at mass <= 3; off = a heavy target.
    let talent_scale = if unit.talent_damage {
        unit.talent1_parameters.first().copied().unwrap_or(0.0)
    } else {
        unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
    };

    // She is in Liftoff herself in all three skills.
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
        // The chant glide lands no hits, so a 22 s cast attacks for 19.5 s.
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

    let final_atk = unit
        .atk
        .mul_add(1.0 + atkbuff + unit.buff_atk, unit.buff_atk_flat);

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
