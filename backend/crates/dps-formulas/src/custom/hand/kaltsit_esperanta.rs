//! Kal'tsit Esperanta (`char_1052_kalts2`): `ArknightsDpsCompare` has no class for
//! her. Only S2 deals damage (True, every enemy around the target). Talent 2's
//! regeneration restarts on each entry into her range, not a rate: not modelled.

use super::super::super::HpsResult;
use super::super::super::operator_unit::{EnemyStats, OperatorUnit};

/// S2 is ammo (`trigger_time` attacks at the S2 interval); other skills keep the
/// game-data duration.
pub fn skill_duration(unit: &OperatorUnit) -> Option<f64> {
    if unit.skill_index != 2 {
        return None;
    }
    let ammo = unit.skill_parameters.get(4).copied().unwrap_or(0.0);
    Some(ammo * f64::from(unit.attack_interval) / (unit.attack_speed / 100.0))
}

#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, _enemy: &EnemyStats) -> Option<f64> {
    if unit.skill_index != 2 {
        return Some(0.0);
    }
    let sp = |i: usize| unit.skill_parameters.get(i).copied().unwrap_or(0.0);
    let atk_interval = f64::from(unit.attack_interval);
    let final_atk = unit.atk.mul_add(1.0 + sp(0) + unit.buff_atk, unit.buff_atk_flat);
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

    let final_atk = unit.atk.mul_add(1.0 + unit.buff_atk, unit.buff_atk_flat);
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

    let final_atk_skill = unit.atk.mul_add(1.0 + sp(0) + unit.buff_atk, unit.buff_atk_flat);
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
