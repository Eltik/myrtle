//! Viy (`char_4226_veen`): `ArknightsDpsCompare` has no class for her. With stored
//! charges on, every attack is a release: one charge per attack interval, lossless.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};
use super::util::module_talent_value;

/// Trait `merge_cnt`: stored charges per Transposed Energy, also its `DoT` stacks.
const MERGE_CNT: f64 = 3.0;

/// Talent 1 ASPD (no Transposed Energy held) plus the module's (stored attacks).
fn talent_and_module_aspd(unit: &OperatorUnit) -> f64 {
    let stored = unit.trait_damage;
    let transposed = stored && unit.talent_damage;
    let mut aspd: f64 = 0.0;
    if !transposed {
        aspd += unit.talent1_parameters.get(1).copied().unwrap_or(0.0);
    }
    if stored && unit.module_index == 1 {
        aspd += module_talent_value(unit, "10_root", "attack_speed").unwrap_or(0.0);
    }
    aspd
}

/// S3 with stored charges is ammo, one per attack interval; without them it never
/// ends, which is the game-data duration.
pub fn skill_duration(unit: &OperatorUnit) -> Option<f64> {
    if unit.skill_index != 3 || !unit.trait_damage {
        return None;
    }
    let ammo = unit.skill_parameters.get(3).copied().unwrap_or(0.0);
    let aspd = talent_and_module_aspd(unit) + unit.skill_parameters.first().copied().unwrap_or(0.0);
    Some(ammo * f64::from(unit.attack_interval) / ((unit.attack_speed + aspd) / 100.0))
}

#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let res = enemy.res;
    let sp = |i: usize, d: f64| unit.skill_parameters.get(i).copied().unwrap_or(d);
    let t1 = |i: usize| unit.talent1_parameters.get(i).copied().unwrap_or(0.0);
    let t2 = |i: usize| unit.talent2_parameters.get(i).copied().unwrap_or(0.0);

    let stored = unit.trait_damage;
    let transposed = stored && unit.talent_damage;

    let mut atk_interval = f64::from(unit.attack_interval);
    let mut atkbuff: f64 = 0.0;
    let mut aspd = talent_and_module_aspd(unit);
    let mut atk_scale: f64 = 1.0;
    let mut bounce_scale: f64 = 0.0;

    if transposed {
        atkbuff += t1(0);
    }

    if skill == 1 {
        aspd += sp(0, 0.0);
    }
    if skill == 2 {
        atk_interval += sp(0, 0.0);
        if unit.skill_damage {
            let stacks = sp(3, 0.0);
            atkbuff += sp(1, 0.0) * stacks;
            aspd += sp(2, 0.0) * stacks;
        }
    }
    if skill == 3 {
        aspd += sp(0, 0.0);
        if stored {
            atk_scale = sp(5, 1.0);
            bounce_scale = sp(1, 0.0);
        }
    }

    let final_atk = unit
        .atk
        .mul_add(1.0 + atkbuff + unit.buff_atk, unit.buff_atk_flat);
    let arts = |scale: f64| (final_atk * scale * (1.0 - res / 100.0)).max(final_atk * scale * 0.05);
    let hitdmg = arts(atk_scale);
    let bouncedmg = if bounce_scale > 0.0 {
        arts(bounce_scale)
    } else {
        0.0
    };

    let mut dps = (hitdmg + bouncedmg) / atk_interval * (unit.attack_speed + aspd) / 100.0;

    if stored && unit.talent2_damage && unit.elite == 2 {
        let per_release = if transposed { MERGE_CNT } else { 1.0 };
        // Stacks per release are capped by the charges stored during the DoT lockout.
        let effective_interval = atk_interval / ((unit.attack_speed + aspd) / 100.0);
        let stored_in_lockout = t2(0) / effective_interval;
        let stacks = t2(2).min(per_release).min(stored_in_lockout);
        let value = t2(1);
        dps = (value * (1.0 - res / 100.0))
            .max(value * 0.05)
            .mul_add(stacks, dps);
    }

    Some(dps)
}
