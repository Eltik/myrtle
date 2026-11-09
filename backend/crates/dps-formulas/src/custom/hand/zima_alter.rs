//! Zima the Raging Tide (`char_1051_headb2`): `ArknightsDpsCompare` has no class for
//! her. Splash is modelled as Odda and Pepe are: the target takes the full hit, every
//! other enemy the 50% splash.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};
use super::util::{UNLIMITED_DURATION, module_trait_value};

/// Trait splash as a fraction of ATK; only the module stores it (`attack@atk_scale_2`).
const SPLASH_SCALE: f64 = 0.5;
/// S3 text ("a chain of five hammer blows"); no blackboard key.
const S3_BLOWS: i32 = 5;

/// S2 is unlimited from its second activation on; other skills keep the game-data
/// duration.
pub fn skill_duration(unit: &OperatorUnit) -> Option<f64> {
    (unit.skill_index == 2 && unit.skill_damage).then_some(UNLIMITED_DURATION)
}

#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let defense = enemy.defense;
    let atk_interval = f64::from(unit.attack_interval);
    let param = |i: usize| unit.skill_parameters.get(i).copied().unwrap_or(0.0);
    let others = f64::from(unit.targets - 1);

    let atk_scale = if unit.module_index == 1 && unit.module_damage {
        module_trait_value(unit, "atk_scale_e").unwrap_or(1.0)
    } else {
        1.0
    };
    let splash_dmg_scale = if unit.elite > 0 {
        unit.talent1_parameters.first().copied().unwrap_or(1.0)
    } else {
        1.0
    };
    // Talent 2 applies while a skill is active, doubled because she is in the group.
    let mut atkbuff = if unit.elite == 2 && skill > 0 {
        unit.talent2_parameters.first().copied().unwrap_or(0.0)
            * unit.talent2_parameters.get(2).copied().unwrap_or(1.0)
    } else {
        0.0
    };

    let attack = |final_atk: f64, scale: f64| {
        let main = final_atk * atk_scale * scale;
        let splash = SPLASH_SCALE * main;
        ((splash - defense).max(splash * 0.05) * splash_dmg_scale)
            .mul_add(others, (main - defense).max(main * 0.05))
    };

    let mut aspd = 0.0;
    match skill {
        1 => {
            atkbuff += param(0);
            aspd += param(1);
        }
        2 => {
            atkbuff += if unit.skill_damage {
                param(2)
            } else {
                param(0)
            }
        }
        3 => {
            // No duration: the total of the five blows is the skill DPS (Degenbrecher's
            // convention). The +30% step is granted after each blow, so blow 0 has none.
            let mut total = 0.0;
            for k in 0..S3_BLOWS {
                let blow_buff = f64::from(k).mul_add(param(1), atkbuff + param(4));
                let final_atk = unit
                    .atk
                    .mul_add(1.0 + blow_buff + unit.buff_atk, unit.buff_atk_flat);
                total += attack(final_atk, param(0));
            }
            return Some(total);
        }
        _ => {}
    }

    let final_atk = unit
        .atk
        .mul_add(1.0 + atkbuff + unit.buff_atk, unit.buff_atk_flat);
    Some(attack(final_atk, 1.0) / atk_interval * (unit.attack_speed + aspd) / 100.0)
}
