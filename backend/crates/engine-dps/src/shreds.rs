use myrtle_dps_formulas::operator_unit::EnemyStats;

/// Apply shreds to enemy stats (flat first, then percentage)
#[must_use]
pub fn apply_shreds(enemy: &EnemyStats, shreds: &[f64]) -> EnemyStats {
    let def = ((enemy.defense - shreds[1]).max(0.0)) * shreds[0];
    let res = ((enemy.res - shreds[3]).max(0.0)) * shreds[2];
    EnemyStats {
        defense: def.max(0.0),
        res: res.max(0.0),
    }
}
