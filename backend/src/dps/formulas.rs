use super::operator_unit::{EnemyStats, OperatorUnit};

/// Apply damage reduction (physical = flat DEF, arts = % RES)
pub fn apply_damage(atk: f64, enemy: &EnemyStats, is_physical: bool) -> f64 {
    if is_physical {
        (atk - enemy.defense).max(atk * 0.05)
    } else {
        (atk * (1.0 - enemy.res / 100.0)).max(atk * 0.05)
    }
}

/// Apply shreds to enemy stats (flat first, then percentage)
pub fn apply_shreds(enemy: &EnemyStats, shreds: &[f64]) -> EnemyStats {
    let def = ((enemy.defense - shreds[1]).max(0.0)) * shreds[0];
    let res = ((enemy.res - shreds[3]).max(0.0)) * shreds[2];
    EnemyStats {
        defense: def.max(0.0),
        res: res.max(0.0),
    }
}

/// ATK buff skill
/// final_atk = atk * (1 + skill_scale + buff_atk) + buff_atk_flat
pub fn atk_buff(unit: &OperatorUnit, enemy: &EnemyStats, atk_scale: f64, hits: f64) -> f64 {
    let final_atk = unit.atk * (1.0 + atk_scale + unit.buff_atk) + unit.buff_atk_flat;
    let hit_dmg = apply_damage(final_atk, enemy, unit.is_physical);
    (hits * hit_dmg) / unit.attack_interval as f64 * unit.attack_speed / 100.0
}

/// Multi-hit
/// N hits per attack cycle, each mitigated separately
pub fn multi_hit(
    unit: &OperatorUnit,
    enemy: &EnemyStats,
    atk_scale: f64,
    hits: f64,
    targets: f64,
) -> f64 {
    let final_atk = unit.atk * (1.0 + atk_scale + unit.buff_atk) + unit.buff_atk_flat;
    let hit_dmg = apply_damage(final_atk, enemy, unit.is_physical);
    (hits * hit_dmg * targets) / unit.attack_interval as f64 * unit.attack_speed / 100.0
}

/// ASPD talent
/// talent adds attack speed during skill
pub fn atk_buff_with_aspd(
    unit: &OperatorUnit,
    enemy: &EnemyStats,
    atk_scale: f64,
    extra_aspd: f64,
) -> f64 {
    let final_atk = unit.atk * (1.0 + atk_scale + unit.buff_atk) + unit.buff_atk_flat;
    let hit_dmg = apply_damage(final_atk, enemy, unit.is_physical);
    hit_dmg / unit.attack_interval as f64 * (unit.attack_speed + extra_aspd) / 100.0
}

/// True damage
/// ignores DEF and RES
pub fn true_damage(unit: &OperatorUnit, atk_scale: f64) -> f64 {
    let final_atk = unit.atk * (1.0 + atk_scale + unit.buff_atk) + unit.buff_atk_flat;
    final_atk / unit.attack_interval as f64 * unit.attack_speed / 100.0
}

/// DEF/RES ignore
/// reduces enemy defense by a percentage before damage calc
pub fn damage_with_ignore(
    unit: &OperatorUnit,
    enemy: &EnemyStats,
    atk_scale: f64,
    def_ignore: f64,
    res_ignore: f64,
) -> f64 {
    let final_atk = unit.atk * (1.0 + atk_scale + unit.buff_atk) + unit.buff_atk_flat;
    let modified_enemy = EnemyStats {
        defense: enemy.defense * (1.0 - def_ignore),
        res: (enemy.res - res_ignore).max(0.0),
    };
    let hit_dmg = apply_damage(final_atk, &modified_enemy, unit.is_physical);
    hit_dmg / unit.attack_interval as f64 * unit.attack_speed / 100.0
}

/// AoE — DPS multiplied by min(targets, cap)
pub fn aoe(unit: &OperatorUnit, enemy: &EnemyStats, atk_scale: f64, targets: f64) -> f64 {
    let final_atk = unit.atk * (1.0 + atk_scale + unit.buff_atk) + unit.buff_atk_flat;
    let hit_dmg = apply_damage(final_atk, enemy, unit.is_physical);
    let target_count = (unit.targets as f64).min(targets);
    hit_dmg / unit.attack_interval as f64 * unit.attack_speed / 100.0 * target_count
}
