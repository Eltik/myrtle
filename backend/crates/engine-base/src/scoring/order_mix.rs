//! Trading-post order economics

use myrtle_gamedata_raw::tables::building::BuildingDataFile;

use crate::registry::OrderEffect;
use crate::scoring::yield_model::GOLD_BAR_LMD;

/// One LMD order size: Pure Gold traded and completion time in minutes
#[derive(Clone, Copy)]
struct OrderSize {
    gold: u32,
    minutes: f64,
}

/// Low / medium / high yield
const ORDER_SIZES: [OrderSize; 3] = [
    OrderSize {
        gold: 2,
        minutes: 144.0,
    },
    OrderSize {
        gold: 3,
        minutes: 210.0,
    },
    OrderSize {
        gold: 4,
        minutes: 276.0,
    },
];

/// Draw weights per order size, indexed by `OrderRarity - 1`
const MIX_BY_RARITY: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.6, 0.4, 0.0], [0.3, 0.5, 0.2]];

/// Tailoring mix at rarity 3, indexed by summed steps - 1
const TAILORING_MIX_BY_STEPS: [[f64; 3]; 3] =
    [[0.18, 0.25, 0.57], [0.14, 0.21, 0.65], [0.05, 0.06, 0.89]];

const MAX_RARITY: usize = 3;

#[must_use]
pub fn rarity_for_level(building_data: &BuildingDataFile, level: i32) -> usize {
    let phases = &building_data.trading_data.phases;
    #[allow(clippy::cast_sign_loss)]
    let idx = (level.max(1) as usize - 1).min(phases.len().saturating_sub(1));
    phases.get(idx).map_or(MAX_RARITY, |p| {
        p.order_rarity.clamp(1, MAX_RARITY as i32) as usize
    })
}

fn mix_for(effects: &[OrderEffect], rarity: usize) -> [f64; 3] {
    let base = MIX_BY_RARITY[rarity.clamp(1, MAX_RARITY) - 1];
    if rarity < MAX_RARITY {
        return base;
    }
    let steps: usize = effects
        .iter()
        .map(|e| match e {
            OrderEffect::HigherYieldChance { strong: true } => 2,
            OrderEffect::HigherYieldChance { strong: false } => 1,
            _ => 0,
        })
        .sum();
    if steps == 0 {
        base
    } else {
        TAILORING_MIX_BY_STEPS[steps.min(TAILORING_MIX_BY_STEPS.len()) - 1]
    }
}

fn order_lmd(effects: &[OrderEffect], size: OrderSize) -> f64 {
    let mut gold = f64::from(size.gold);
    let mut flat = 0.0;
    let defaulted_bonus = effects
        .iter()
        .filter_map(|e| match e {
            OrderEffect::DefaultedGoldBonus { below, bonus } if size.gold < *below => Some(*bonus),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    gold += f64::from(defaulted_bonus);
    let high_bonus = effects
        .iter()
        .filter_map(|e| match e {
            OrderEffect::HighOrderLmdBonus { above, lmd } if size.gold > *above => Some(*lmd),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    flat += f64::from(high_bonus);
    gold.mul_add(GOLD_BAR_LMD, flat)
}

fn order_gold(effects: &[OrderEffect], size: OrderSize) -> f64 {
    let defaulted_bonus = effects
        .iter()
        .filter_map(|e| match e {
            OrderEffect::DefaultedGoldBonus { below, bonus } if size.gold < *below => Some(*bonus),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    f64::from(size.gold + defaulted_bonus)
}

fn lmd_per_minute(effects: &[OrderEffect], mix: [f64; 3]) -> f64 {
    per_minute(mix, |size| order_lmd(effects, size))
}

fn gold_per_minute(effects: &[OrderEffect], mix: [f64; 3]) -> f64 {
    per_minute(mix, |size| order_gold(effects, size))
}

fn per_minute(mix: [f64; 3], per_order: impl Fn(OrderSize) -> f64) -> f64 {
    let (total, minutes) = ORDER_SIZES
        .iter()
        .zip(mix)
        .fold((0.0, 0.0), |(t, m), (size, w)| {
            (w.mul_add(per_order(*size), t), w.mul_add(size.minutes, m))
        });
    if minutes <= 0.0 { 0.0 } else { total / minutes }
}

/// Order-VALUE %: LMD/hr with `effects` over a bare post's, minus one
#[must_use]
pub fn value_pct(effects: &[OrderEffect], rarity: usize) -> f64 {
    if effects.is_empty() {
        return 0.0;
    }
    let base = lmd_per_minute(&[], MIX_BY_RARITY[rarity.clamp(1, MAX_RARITY) - 1]);
    if base <= 0.0 {
        return 0.0;
    }
    let boosted = lmd_per_minute(effects, mix_for(effects, rarity));
    (boosted / base - 1.0) * 100.0
}

/// Gold-THROUGHPUT %: bars consumed/hr with `effects` over a bare post's, minus one
#[must_use]
pub fn gold_pct(effects: &[OrderEffect], rarity: usize) -> f64 {
    if effects.is_empty() {
        return 0.0;
    }
    let base = gold_per_minute(&[], MIX_BY_RARITY[rarity.clamp(1, MAX_RARITY) - 1]);
    if base <= 0.0 {
        return 0.0;
    }
    let boosted = gold_per_minute(effects, mix_for(effects, rarity));
    (boosted / base - 1.0) * 100.0
}

/// OPTIMISTIC bound for candidate ranking only, never a final score
#[must_use]
pub fn optimistic_value_pct(effect: &OrderEffect, rarity: usize) -> f64 {
    let solo = value_pct(std::slice::from_ref(effect), rarity);
    let strong = OrderEffect::HigherYieldChance { strong: true };
    let with = value_pct(&[effect.clone(), strong.clone()], rarity);
    let without = value_pct(std::slice::from_ref(&strong), rarity);
    solo.max(with - without)
}

#[must_use]
pub fn optimistic_value_pct_among(
    effect: &OrderEffect,
    rarity: usize,
    companions: &[OrderEffect],
) -> f64 {
    let mut best = optimistic_value_pct(effect, rarity);
    for other in companions.iter().filter(|o| *o != effect) {
        let with = value_pct(&[effect.clone(), other.clone()], rarity);
        let without = value_pct(std::slice::from_ref(other), rarity);
        best = best.max(with - without);
    }
    best
}

/// Bars a bare post sells per day at 100%
#[must_use]
pub fn bars_per_day(rarity: usize) -> f64 {
    let mix = MIX_BY_RARITY[rarity.clamp(1, MAX_RARITY) - 1];
    let (gold, minutes) = ORDER_SIZES
        .iter()
        .zip(mix)
        .fold((0.0, 0.0), |(g, m), (size, w)| {
            (w.mul_add(f64::from(size.gold), g), w.mul_add(size.minutes, m))
        });
    if minutes <= 0.0 {
        0.0
    } else {
        gold / minutes * 1440.0
    }
}

#[must_use]
pub fn avg_gold_per_order(rarity: usize) -> f64 {
    let mix = MIX_BY_RARITY[rarity.clamp(1, MAX_RARITY) - 1];
    ORDER_SIZES
        .iter()
        .zip(mix)
        .map(|(size, w)| w * f64::from(size.gold))
        .sum()
}
