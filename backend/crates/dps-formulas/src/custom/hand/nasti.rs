//! Nasti (`char_4212_nasti`): `ArknightsDpsCompare` has no class for her. Devices
//! deal no damage; S2 stops her attacking.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};

/// `character_table` `BlockCnt`, 2 at every promotion.
const BLOCK: f64 = 2.0;

#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let defense = enemy.defense;
    let atk_interval = f64::from(unit.attack_interval);
    let sp = |i: usize| unit.skill_parameters.get(i).copied().unwrap_or(0.0);

    if skill == 2 {
        return Some(0.0);
    }

    let mut atkbuff: f64 = 0.0;
    let mut targets: f64 = 1.0;
    if skill == 1 {
        atkbuff = sp(1);
        targets = f64::from(unit.targets).min(BLOCK);
    }
    if skill == 3 {
        atkbuff = sp(0);
    }

    let final_atk = unit.atk.mul_add(1.0 + atkbuff + unit.buff_atk, unit.buff_atk_flat);
    let hitdmg = (final_atk - defense).max(final_atk * 0.05);
    Some(hitdmg / atk_interval * unit.attack_speed / 100.0 * targets)
}
