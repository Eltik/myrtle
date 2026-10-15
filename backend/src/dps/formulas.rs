use super::operator_unit::EnemyStats;

/// Apply shreds to enemy stats (flat first, then percentage)
pub fn apply_shreds(enemy: &EnemyStats, shreds: &[f64]) -> EnemyStats {
    let def = ((enemy.defense - shreds[1]).max(0.0)) * shreds[0];
    let res = ((enemy.res - shreds[3]).max(0.0)) * shreds[2];
    EnemyStats {
        defense: def.max(0.0),
        res: res.max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    fn enemy(defense: f64, res: f64) -> EnemyStats {
        EnemyStats { defense, res }
    }

    #[test]
    fn identity_shreds_leave_the_enemy_alone() {
        let out = apply_shreds(&enemy(1000.0, 50.0), &[1.0, 0.0, 1.0, 0.0]);
        assert!(close(out.defense, 1000.0));
        assert!(close(out.res, 50.0));
    }

    #[test]
    fn flat_shred_applies_before_the_percentage() {
        // [def multiplier, flat def, res multiplier, flat res]
        let out = apply_shreds(&enemy(1000.0, 50.0), &[0.5, 100.0, 0.8, 20.0]);
        // (1000 - 100) * 0.5, not 1000 * 0.5 - 100.
        assert!(close(out.defense, 450.0));
        assert!(close(out.res, 24.0));
    }

    #[test]
    fn stats_never_go_negative() {
        let out = apply_shreds(&enemy(100.0, 10.0), &[1.0, 500.0, 1.0, 50.0]);
        assert!(close(out.defense, 0.0));
        assert!(close(out.res, 0.0));
        let negative_multiplier = apply_shreds(&enemy(100.0, 10.0), &[-1.0, 0.0, -0.5, 0.0]);
        assert!(close(negative_multiplier.defense, 0.0));
        assert!(close(negative_multiplier.res, 0.0));
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn fewer_than_four_shred_slots_panics() {
        let _ = apply_shreds(&enemy(100.0, 10.0), &[1.0, 0.0, 1.0]);
    }
}
