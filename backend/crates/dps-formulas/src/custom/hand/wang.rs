//! Wang (`char_2027_wang`): `ArknightsDpsCompare` has no class for him. Every stone
//! triggers exactly once (Dorothy's mine model) and arrives from the skill alone.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};

/// S3 text ("up to 3 additional stones" per placement); no blackboard key.
const S3_EXTRA_PER_PLACEMENT: f64 = 3.0;
/// `token_10064_wang_stone1` `RespawnTime` at E2, seconds.
const STONE_REDEPLOY_E2: f64 = 2.0;

/// S3 has no timer: one placement per stone redeploy until the ammo is spent. Other
/// skills keep the game-data duration.
pub fn skill_duration(unit: &OperatorUnit) -> Option<f64> {
    if unit.skill_index != 3 {
        return None;
    }
    let ammo = unit.skill_parameters.get(1).copied().unwrap_or(0.0);
    Some((ammo / S3_EXTRA_PER_PLACEMENT).ceil() * STONE_REDEPLOY_E2)
}

#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let defense = enemy.defense;
    let res = enemy.res;
    let atk_interval = f64::from(unit.attack_interval);
    let param = |i: usize| unit.skill_parameters.get(i).copied().unwrap_or(0.0);
    let talent2 = |i: usize| unit.talent2_parameters.get(i).copied().unwrap_or(0.0);
    let targets = f64::from(unit.targets);

    let final_atk = unit.atk.mul_add(1.0 + unit.buff_atk, unit.buff_atk_flat);
    let attack_dps =
        (final_atk - defense).max(final_atk * 0.05) / atk_interval * unit.attack_speed / 100.0;

    // Below E2 the talent slots hold placeholder defaults.
    let stacks = if unit.elite == 2 && unit.talent2_damage {
        talent2(2)
    } else {
        0.0
    };
    let stone_scale = 1.0 + stacks * talent2(0);
    let stone_res = (res - stacks * talent2(1)).max(0.0);
    let stone_hit = |scale: f64| {
        (final_atk * scale * (1.0 - stone_res / 100.0)).max(final_atk * scale * 0.05) * stone_scale
    };

    // Talent 1 (E1, `max_spawn_cnt` above 0) adds a stone to every placement.
    let extra_stone = if unit.talent1_parameters.get(3).copied().unwrap_or(0.0) > 0.0 {
        2.0
    } else {
        1.0
    };
    let cycle = f64::from(unit.skill_cost) / (1.0 + f64::from(unit.sp_boost));
    let stones_per_second = if cycle > 0.0 {
        param(0) * extra_stone / cycle
    } else {
        0.0
    };

    let dps = match skill {
        1 => (stone_hit(param(1)) * param(2).floor()).mul_add(stones_per_second, attack_dps),
        2 => (stone_hit(param(1)) * targets).mul_add(stones_per_second, attack_dps),
        3 => {
            let duration = unit.skill_duration;
            let total = (param(0) + param(1)) * stone_hit(param(2)) * targets;
            if duration > 0.0 {
                total / duration
            } else {
                total
            }
        }
        _ => attack_dps,
    };

    Some(dps)
}
