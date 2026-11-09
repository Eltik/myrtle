//! Closure (`char_4228_closur`): `ArknightsDpsCompare` has no class for her. S3's
//! target count ramps every 9 attacks and is averaged over the skill window.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};

/// Trait blackboard `atk_scale`: ATK x1.5 against enemies her Reinforcements block.
const TRAIT_ATK_SCALE: f64 = 1.5;

#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let defense = enemy.defense;
    let sp = |i: usize, d: f64| unit.skill_parameters.get(i).copied().unwrap_or(d);

    let atk_scale = if unit.trait_damage {
        TRAIT_ATK_SCALE
    } else {
        1.0
    };
    let mut atkbuff = if unit.elite == 2 {
        unit.talent2_parameters.get(1).copied().unwrap_or(0.0)
    } else {
        0.0
    };
    let mut atk_interval = f64::from(unit.attack_interval);
    let mut skill_scale: f64 = 1.0;
    let mut targets: f64 = 1.0;

    if skill == 2 {
        atkbuff += sp(4, 0.0);
        targets = f64::from(unit.targets).min(sp(5, 1.0));
    }
    if skill == 3 {
        atk_interval += sp(5, 0.0);
        skill_scale = sp(4, 1.0);
    }

    let final_atk = unit.atk.mul_add(1.0 + atkbuff + unit.buff_atk, unit.buff_atk_flat);
    let scale = atk_scale * skill_scale;
    let hitdmg = (final_atk * scale - defense).max(final_atk * scale * 0.05);
    let attack_rate = unit.attack_speed / 100.0 / atk_interval;

    if skill == 3 {
        targets = s3_average_targets(
            attack_rate * unit.skill_duration,
            f64::from(unit.targets),
            sp(2, 1.0),
            sp(3, 0.0),
        );
    }

    Some(hitdmg * attack_rate * targets)
}

/// Mean target count over `attacks` attacks (continuous) when attack k hits
/// 1 + min(`max_trigger`, floor(k / `per_trigger`)) targets, capped at `enemies`.
#[allow(clippy::while_float)]
fn s3_average_targets(attacks: f64, enemies: f64, per_trigger: f64, max_trigger: f64) -> f64 {
    if attacks <= 0.0 || per_trigger <= 0.0 {
        return enemies.min(1.0 + max_trigger).max(1.0);
    }
    let mut sum = 0.0;
    let mut step = 0.0;
    while step <= max_trigger {
        let start = step * per_trigger;
        if start >= attacks {
            break;
        }
        let end = if step < max_trigger {
            ((step + 1.0) * per_trigger).min(attacks)
        } else {
            attacks
        };
        sum = (end - start).mul_add(enemies.min(1.0 + step), sum);
        step += 1.0;
    }
    sum / attacks
}
