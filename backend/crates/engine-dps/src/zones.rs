//! Buff zones

use serde::{Deserialize, Serialize};

use myrtle_dps_formulas::operator_unit::EnemyStats;

pub const MAX_SOURCES: usize = 16;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BuffZones {
    /// Added to the operator's base ATK
    pub atk_flat: Option<f64>,
    /// Additive into `buff_atk`
    pub atk_percent: Option<f64>,
    pub inspire: Vec<f64>,
    /// Final ATK multiplier
    /// Resolved as `atk *= D; buff_atk_flat *= D`
    pub atk_multiplier: Vec<f64>,
    /// Additive into `attack_speed`
    pub aspd: Option<f64>,
    /// SP recovery per second, additive into `sp_boost`
    pub sp_recovery: Option<f64>,
    /// Added to `sp_boost` after `sp_recovery`
    pub sp_aura: Vec<f64>,
    pub fragile: Vec<f64>,
    pub physical_fragile: Vec<f64>,
    pub arts_fragile: Vec<f64>,
    pub damage_reduction: Vec<f64>,
    /// Enemy DEF reduction and ignore
    pub def: StatReduction,
    /// Enemy RES reduction and ignore
    pub res: StatReduction,
}

/// Reduction of one enemy stat
/// `((v - flat - max(steal)) * (1 - percent) - ignoreFlat) * (1 - ignorePercent)`
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StatReduction {
    /// Flat reduction
    pub flat: Option<f64>,
    /// Percentage reduction
    pub percent: Option<f64>,
    /// Flat ignore (attacker side)
    pub ignore_flat: Option<f64>,
    /// Percentage ignore (attacker side)
    pub ignore_percent: Option<f64>,
    /// Steal (flat reduction)
    pub steal: Vec<f64>,
}

impl BuffZones {
    #[must_use]
    pub fn lists(&self) -> [(&'static str, &[f64]); 8] {
        [
            ("zones.inspire", &self.inspire),
            ("zones.atkMultiplier", &self.atk_multiplier),
            ("zones.spAura", &self.sp_aura),
            ("zones.fragile", &self.fragile),
            ("zones.physicalFragile", &self.physical_fragile),
            ("zones.artsFragile", &self.arts_fragile),
            ("zones.damageReduction", &self.damage_reduction),
            ("zones.def.steal", &self.def.steal),
        ]
    }
}

pub fn max_source(sources: &[f64]) -> Option<f64> {
    sources.iter().copied().reduce(f64::max)
}

#[must_use]
pub fn product(sources: &[f64]) -> f64 {
    sources.iter().product()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedReduction {
    /// Flat reduction
    pub flat: f64,
    /// Multiplier
    /// `1 - (legacy% / 100 + zone percent)`
    pub ratio: f64,
    pub ignore_flat: f64,
    pub ignore_ratio: f64,
}

impl ResolvedReduction {
    #[must_use]
    pub fn new(zone: &StatReduction, legacy_ratio: f64, legacy_flat: f64) -> Self {
        let steal = max_source(&zone.steal).unwrap_or(0.0);
        let mut flat = legacy_flat;
        if let Some(f) = zone.flat {
            flat += f;
        }
        if !zone.steal.is_empty() {
            flat += steal;
        }
        let ratio = match zone.percent {
            Some(p) => (legacy_ratio - p).max(0.0),
            None => legacy_ratio,
        };
        Self {
            flat,
            ratio,
            ignore_flat: zone.ignore_flat.unwrap_or(0.0),
            ignore_ratio: (1.0 - zone.ignore_percent.unwrap_or(0.0)).max(0.0),
        }
    }

    /// `((v - flat) * ratio - ignoreFlat) * ignoreRatio`
    #[allow(clippy::float_cmp)]
    #[must_use]
    pub fn apply(&self, value: f64) -> f64 {
        let reduced = (value - self.flat).max(0.0) * self.ratio;
        if self.ignore_flat == 0.0 && self.ignore_ratio == 1.0 {
            return reduced.max(0.0);
        }
        ((reduced.max(0.0) - self.ignore_flat).max(0.0) * self.ignore_ratio).max(0.0)
    }
}

#[must_use]
pub fn resolve_enemy(
    enemy: &EnemyStats,
    def: &ResolvedReduction,
    res: &ResolvedReduction,
) -> EnemyStats {
    EnemyStats {
        defense: def.apply(enemy.defense),
        res: res.apply(enemy.res).min(100.0),
    }
}

/// Which typed fragility group
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Physical,
    Arts,
}

#[must_use]
pub fn post_multiplier(
    zones: &BuffZones,
    generic_legacy: Option<f64>,
    typed: Option<DamageType>,
) -> f64 {
    let mut generic = zones.fragile.clone();
    if let Some(f) = generic_legacy {
        generic.push(f);
    }
    let generic = max_source(&generic).unwrap_or(0.0);
    let typed = match typed {
        Some(DamageType::Physical) => max_source(&zones.physical_fragile).unwrap_or(0.0),
        Some(DamageType::Arts) => max_source(&zones.arts_fragile).unwrap_or(0.0),
        None => 0.0,
    };
    let reduction: f64 = zones.damage_reduction.iter().map(|x| 1.0 - x).product();
    let mut m = 1.0 + generic;
    if typed != 0.0 {
        m *= 1.0 + typed;
    }
    if !zones.damage_reduction.is_empty() {
        m *= reduction;
    }
    m
}

#[cfg(test)]
#[allow(clippy::float_cmp)] // the bit-for-bit claims are what these tests check
mod tests {
    use super::*;
    use crate::shreds::apply_shreds;

    fn legacy_pair(pct: i32, flat: i32) -> (f64, f64) {
        (1.0 - (f64::from(pct) / 100.0), f64::from(flat))
    }

    #[test]
    fn legacy_shred_reproduces_apply_shreds_bit_for_bit() {
        // The full legacy range the request validator admits, on a grid.
        let none = StatReduction::default();
        let mut checked = 0;
        for def_pct in (-100..=100).step_by(5) {
            for def_flat in [-500, -1, 0, 1, 100, 450, 999, 5000] {
                for res_pct in (-100..=100).step_by(10) {
                    for res_flat in [-20, 0, 7, 15, 60] {
                        let (dm, df) = legacy_pair(def_pct, def_flat);
                        let (rm, rf) = legacy_pair(res_pct, res_flat);
                        let d = ResolvedReduction::new(&none, dm, df);
                        let r = ResolvedReduction::new(&none, rm, rf);
                        for (def, res) in
                            [(0.0, 0.0), (300.0, 20.0), (1000.0, 50.0), (2500.0, 100.0)]
                        {
                            let enemy = EnemyStats { defense: def, res };
                            let old = apply_shreds(&enemy, &[dm, df, rm, rf]);
                            // `apply`, not `resolve_enemy`: the RES cap is a zones
                            // rule and is the one difference (see below).
                            assert_eq!(
                                old.defense.to_bits(),
                                d.apply(def).to_bits(),
                                "{def_pct} {def_flat} {def}"
                            );
                            assert_eq!(
                                old.res.to_bits(),
                                r.apply(res).to_bits(),
                                "{res_pct} {res_flat} {res}"
                            );
                            if old.res <= 100.0 {
                                let new = resolve_enemy(&enemy, &d, &r);
                                assert_eq!(old.res.to_bits(), new.res.to_bits());
                            }
                            checked += 1;
                        }
                    }
                }
            }
        }
        assert!(checked > 10_000);
    }

    #[test]
    fn zone_percent_adds_to_the_legacy_percent() {
        let zone = StatReduction {
            percent: Some(0.2),
            ..Default::default()
        };
        let (m, f) = legacy_pair(30, 0);
        let r = ResolvedReduction::new(&zone, m, f);
        assert!((r.ratio - 0.5).abs() < 1e-15);
        assert!((r.apply(1000.0) - 500.0).abs() < 1e-9);
        // Floored: 30% + 90% removes everything, never inverts.
        let zone = StatReduction {
            percent: Some(0.9),
            ..Default::default()
        };
        assert_eq!(ResolvedReduction::new(&zone, m, f).apply(1000.0), 0.0);
    }

    #[test]
    fn order_is_flat_then_percent_then_ignore_flat_then_ignore_percent() {
        let zone = StatReduction {
            flat: Some(100.0),
            percent: Some(0.5),
            ignore_flat: Some(50.0),
            ignore_percent: Some(0.4),
            steal: vec![],
        };
        let r = ResolvedReduction::new(&zone, 1.0, 0.0);
        // ((1000 - 100) * 0.5 - 50) * 0.6 = 240
        assert!((r.apply(1000.0) - 240.0).abs() < 1e-9);
        // Each step floors at 0.
        assert_eq!(r.apply(120.0), 0.0);
    }

    #[test]
    fn steal_takes_the_max_and_adds_to_flat() {
        let zone = StatReduction {
            flat: Some(10.0),
            steal: vec![50.0, 80.0, 30.0],
            ..Default::default()
        };
        let r = ResolvedReduction::new(&zone, 1.0, 5.0);
        assert_eq!(r.flat, 95.0);
    }

    #[test]
    fn res_is_capped_at_100() {
        let none = ResolvedReduction::new(&StatReduction::default(), 1.0, 0.0);
        let out = resolve_enemy(
            &EnemyStats {
                defense: 0.0,
                res: 250.0,
            },
            &none,
            &none,
        );
        assert_eq!(out.res, 100.0);
    }

    #[test]
    fn max_groups_take_the_largest_source() {
        assert_eq!(max_source(&[]), None);
        assert_eq!(max_source(&[0.1, 0.3, 0.2]), Some(0.3));
        assert_eq!(max_source(&[-0.2, -0.1]), Some(-0.1));
        assert_eq!(product(&[]), 1.0);
        assert!((product(&[1.2, 1.5]) - 1.8).abs() < 1e-15);
    }

    #[test]
    fn fragile_groups_max_within_and_multiply_across() {
        let zones = BuffZones {
            fragile: vec![0.2, 0.3],
            physical_fragile: vec![0.1, 0.25],
            arts_fragile: vec![0.5],
            damage_reduction: vec![0.2, 0.5],
            ..Default::default()
        };
        // Legacy fragile 0.25 loses the MAX to 0.3.
        let phys = post_multiplier(&zones, Some(0.25), Some(DamageType::Physical));
        assert!((phys - 1.3 * 1.25 * 0.8 * 0.5).abs() < 1e-12);
        let arts = post_multiplier(&zones, None, Some(DamageType::Arts));
        assert!((arts - 1.3 * 1.5 * 0.8 * 0.5).abs() < 1e-12);
        // Legacy fragile wins when it is the largest generic source.
        let legacy = post_multiplier(&zones, Some(0.4), None);
        assert!((legacy - 1.4 * 0.8 * 0.5).abs() < 1e-12);
    }

    #[test]
    fn empty_zones_leave_the_legacy_fragile_multiplier_exact() {
        let zones = BuffZones::default();
        for f in [0.0, 0.1, 0.3, -0.25, 1.7] {
            assert_eq!(
                post_multiplier(&zones, Some(f), None).to_bits(),
                (1.0 + f).to_bits()
            );
        }
        assert_eq!(post_multiplier(&zones, None, Some(DamageType::Arts)), 1.0);
    }
}
