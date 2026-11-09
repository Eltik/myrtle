//! `SilverAsh` the Reignfrost (`char_1045_svash2`): `ArknightsDpsCompare` has no
//! class for him. S2 is cast every time it is ready (upstream instant-skill
//! convention); S3 hits every enemy in the line.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};

/// Upstream SP lockout after an instant skill (Gracebearer S2, Vigil S2).
const SP_LOCKOUT: f64 = 1.2;

#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let defense = enemy.defense;
    let sp = |i: usize, d: f64| unit.skill_parameters.get(i).copied().unwrap_or(d);

    let atk_interval = f64::from(unit.attack_interval);
    let final_atk = unit.atk.mul_add(1.0 + unit.buff_atk, unit.buff_atk_flat);
    let phys = |scale: f64| (final_atk * scale - defense).max(final_atk * scale * 0.05);
    let attack_rate = unit.attack_speed / 100.0 / atk_interval;

    let dps = if skill == 3 {
        let mut fragile = if unit.skill_damage {
            sp(5, 1.0) - 1.0
        } else {
            0.0
        };
        // His Fragile does not stack with external Fragile: the larger applies.
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
