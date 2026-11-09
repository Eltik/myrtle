//! Bellone (`char_4037_demetr`): `ArknightsDpsCompare` has no class for her. Talent 1
//! is read at max stacks; an S3 proc replaces the hit rather than adding one.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};
use super::util::module_trait_value;

#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let defense = enemy.defense;
    let atk_interval = f64::from(unit.attack_interval);
    let talent = |i: usize| unit.talent1_parameters.get(i).copied().unwrap_or(0.0);
    let param = |i: usize| unit.skill_parameters.get(i).copied().unwrap_or(0.0);

    // Below E1 the talent slots hold placeholder defaults.
    let (def_per_stack, max_stacks, low_hp_bonus) = if unit.elite > 0 {
        // Always at max stacks: 5 take about 4 s and every hit refreshes them.
        let stacks = if skill == 2 { talent(2) } else { talent(1) };
        (-talent(0), stacks, talent(7))
    } else {
        (0.0, 0.0, 0.0)
    };
    let newdef = defense * def_per_stack.mul_add(-max_stacks, 1.0).max(0.0);
    let dmg_scale = if unit.talent_damage {
        1.0 + low_hp_bonus
    } else {
        1.0
    };

    let mut aspd = if unit.module_index == 1 && unit.module_damage {
        module_trait_value(unit, "attack_speed").unwrap_or(0.0)
    } else {
        0.0
    };

    let hit = |final_atk: f64, scale: f64| {
        final_atk
            .mul_add(scale, -newdef)
            .max(final_atk * scale * 0.05)
    };

    let dps = match skill {
        1 => {
            let final_atk = unit.atk.mul_add(1.0 + unit.buff_atk, unit.buff_atk_flat);
            let sp_cost = f64::from(unit.skill_cost);
            let skillhit = hit(final_atk, param(0)) * 2.0;
            let avghit = sp_cost.mul_add(hit(final_atk, 1.0), skillhit) / (sp_cost + 1.0);
            avghit / atk_interval * (unit.attack_speed + aspd) / 100.0
        }
        2 => {
            aspd += param(2);
            let final_atk = unit.atk.mul_add(1.0 + unit.buff_atk, unit.buff_atk_flat);
            hit(final_atk, param(0)) / atk_interval * (unit.attack_speed + aspd) / 100.0
                * f64::from(unit.targets).min(param(1))
        }
        3 => {
            aspd += param(0);
            let final_atk = unit
                .atk
                .mul_add(1.0 + param(1) + unit.buff_atk, unit.buff_atk_flat);
            let prob = param(3);
            let avghit = (1.0 - prob).mul_add(hit(final_atk, 1.0), prob * hit(final_atk, param(2)));
            avghit / atk_interval * (unit.attack_speed + aspd) / 100.0
        }
        _ => {
            let final_atk = unit.atk.mul_add(1.0 + unit.buff_atk, unit.buff_atk_flat);
            hit(final_atk, 1.0) / atk_interval * (unit.attack_speed + aspd) / 100.0
        }
    };

    Some(dps * dmg_scale)
}
