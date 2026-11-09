//! Ch'en the Dawnstreak (`char_1050_chen3`): `ArknightsDpsCompare` has no class for
//! her. S2's slashes and S3's sword wave are spread over the skill duration.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};
use super::util::module_trait_value;

/// From the skill texts ("Slashes the nearest enemy 10 times", "each attack deals
/// 3 instances"): neither count has a blackboard key.
const S2_SLASHES: f64 = 10.0;
const S3_INSTANCES: f64 = 3.0;

#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let defense = enemy.defense;
    let res = enemy.res;
    let atk_interval = f64::from(unit.attack_interval);
    let param = |i: usize| unit.skill_parameters.get(i).copied().unwrap_or(0.0);

    let weakness = unit.elite > 0;
    let (mut atkbuff, mut aspd) = if unit.elite > 0 {
        (
            unit.talent1_parameters.first().copied().unwrap_or(0.0),
            unit.talent1_parameters.get(1).copied().unwrap_or(0.0),
        )
    } else {
        (0.0, 0.0)
    };
    if unit.module_index == 1 && unit.module_damage {
        aspd += module_trait_value(unit, "attack_speed").unwrap_or(0.0);
    }

    let arts = |dmg: f64| (dmg * (1.0 - res / 100.0)).max(dmg * 0.05);
    // Weakness damage (talent 1): the higher of the Physical and Arts results.
    let attack_hit = |dmg: f64| {
        if weakness {
            arts(dmg).max((dmg - defense).max(dmg * 0.05))
        } else {
            arts(dmg)
        }
    };
    let duration = unit.skill_duration;

    let dps = match skill {
        1 => {
            atkbuff += param(0);
            let final_atk = unit.atk.mul_add(1.0 + atkbuff + unit.buff_atk, unit.buff_atk_flat);
            2.0 * attack_hit(final_atk) / atk_interval * (unit.attack_speed + aspd) / 100.0
        }
        2 => {
            let slash_atk = unit.atk.mul_add(1.0 + atkbuff + unit.buff_atk, unit.buff_atk_flat);
            let slashes = S2_SLASHES * arts(slash_atk * param(0));
            let final_atk =
                unit.atk.mul_add(1.0 + atkbuff + param(1) + unit.buff_atk, unit.buff_atk_flat);
            let attacks = attack_hit(final_atk) / atk_interval * (unit.attack_speed + aspd) / 100.0;
            if duration > 0.0 {
                attacks + slashes / duration
            } else {
                attacks
            }
        }
        3 => {
            let final_atk = unit.atk.mul_add(1.0 + atkbuff + unit.buff_atk, unit.buff_atk_flat);
            let targets = f64::from(unit.targets);
            let attacks = S3_INSTANCES * attack_hit(final_atk * param(2)) / atk_interval
                * (unit.attack_speed + aspd)
                / 100.0
                * targets.min(param(3));
            let wave = arts(final_atk * param(1)) * targets;
            if duration > 0.0 {
                attacks + wave / duration
            } else {
                attacks
            }
        }
        _ => {
            let final_atk = unit.atk.mul_add(1.0 + atkbuff + unit.buff_atk, unit.buff_atk_flat);
            attack_hit(final_atk) / atk_interval * (unit.attack_speed + aspd) / 100.0
        }
    };

    Some(dps)
}
