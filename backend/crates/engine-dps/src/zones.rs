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
