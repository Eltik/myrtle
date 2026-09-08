//! Trading-post order economics: which LMD orders a post of a given level
//! can draw, what each is worth and how long it takes, and how order-VALUE
//! skills change the post's LMD per hour.
//!
//! The registry stores an order-value skill as its SHAPE (`OrderEffect`), not
//! a guessed percentage, and the scorer resolves the room's whole set of
//! shapes here against the post's `OrderRarity` (gamedata `TradingData`).
//! That is what makes Proviso worth +100% in a level-1 post, +83% in a
//! level-2 post and +55% in a level-3 post: her "+2 gold on orders below 4"
//! fires on every order a low post can draw, but 4-gold orders - only a
//! level-3 post draws them - earn her nothing.
//!
//! Order sizes, durations and per-rarity draw weights are game mechanics
//! (Trading Post reference tables, cross-checked against the wiki), not
//! per-operator values; LMD per gold bar is the same `GOLD_BAR_LMD` the yield
//! model uses (gamedata `GoldItems` 3003 = 500).

use crate::core::gamedata::types::building::BuildingDataFile;

use super::buff_registry::OrderEffect;
use super::yield_model::GOLD_BAR_LMD;

/// One LMD order size: Pure Gold traded and completion time in minutes.
#[derive(Clone, Copy)]
struct OrderSize {
    gold: u32,
    minutes: f64,
}

/// The three LMD order sizes in rarity order (low / medium / high yield).
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

/// Draw weights per order size, indexed by `OrderRarity - 1`: a level-1
/// post draws only 2-gold orders, level-2 adds 3-gold, level-3 adds 4-gold.
const MIX_BY_RARITY: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.6, 0.4, 0.0], [0.3, 0.5, 0.2]];

/// The Tailoring family's mix at rarity 3: "increased slightly" (α tiers)
/// and "increased" (β tiers). Below rarity 3 no high-yield order exists to
/// shift toward, and the game publishes no shifted mix there - never guess.
const TAILORING_MIX_SLIGHT: [f64; 3] = [0.15, 0.30, 0.55];
const TAILORING_MIX_STRONG: [f64; 3] = [0.05, 0.10, 0.85];

/// Highest supported rarity (the size table has three entries).
const MAX_RARITY: usize = 3;

/// The order rarity a trading post of `level` draws from, from gamedata.
/// Clamped to the supported table; a level past the data uses the top tier.
pub fn rarity_for_level(building_data: &BuildingDataFile, level: i32) -> usize {
    let phases = &building_data.trading_data.phases;
    #[allow(clippy::cast_sign_loss)]
    let idx = (level.max(1) as usize - 1).min(phases.len().saturating_sub(1));
    phases.get(idx).map_or(MAX_RARITY, |p| {
        p.order_rarity.clamp(1, MAX_RARITY as i32) as usize
    })
}

/// The draw mix for `rarity` once the room's mix-shifting effects apply
/// (the strongest Tailoring tier present wins - they don't stack).
fn mix_for(effects: &[OrderEffect], rarity: usize) -> [f64; 3] {
    let base = MIX_BY_RARITY[rarity.clamp(1, MAX_RARITY) - 1];
    if rarity < MAX_RARITY {
        return base;
    }
    let strongest = effects
        .iter()
        .filter_map(|e| match e {
            OrderEffect::HigherYieldChance { strong } => Some(*strong),
            _ => None,
        })
        .max();
    match strongest {
        Some(true) => TAILORING_MIX_STRONG,
        Some(false) => TAILORING_MIX_SLIGHT,
        None => base,
    }
}

/// LMD an order of `size` pays once the room's per-order effects apply.
/// Same-kind effects take the strongest (two Provisos cannot coexist; the
/// rule keeps the arithmetic honest anyway); different kinds compose, since
/// they target disjoint orders (below-4 vs above-3).
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
    gold * GOLD_BAR_LMD + flat
}

/// Pure Gold an order of `size` consumes once the room's per-order effects
/// apply. Proviso's defaulted bonus is gold DRAWN FROM STOCK (base expert,
/// 2026-09-08: "it just is more gold consumed per order"); Tequila's flat
/// LMD rider consumes none.
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

/// LMD per minute of a post drawing `mix` with `effects` applied.
fn lmd_per_minute(effects: &[OrderEffect], mix: [f64; 3]) -> f64 {
    per_minute(mix, |size| order_lmd(effects, size))
}

/// Pure Gold per minute of a post drawing `mix` with `effects` applied.
fn gold_per_minute(effects: &[OrderEffect], mix: [f64; 3]) -> f64 {
    per_minute(mix, |size| order_gold(effects, size))
}

fn per_minute(mix: [f64; 3], per_order: impl Fn(OrderSize) -> f64) -> f64 {
    let (total, minutes) = ORDER_SIZES
        .iter()
        .zip(mix)
        .fold((0.0, 0.0), |(t, m), (size, w)| {
            (t + w * per_order(*size), m + w * size.minutes)
        });
    if minutes <= 0.0 { 0.0 } else { total / minutes }
}

/// The order-VALUE percentage a room's set of order effects is worth at
/// `rarity`: LMD per hour with the effects over LMD per hour of a bare post
/// of the same level, minus one. Zero for an empty set.
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

/// The gold-THROUGHPUT percentage the same set of effects is worth at
/// `rarity`: Pure Gold consumed per hour with the effects over a bare post's,
/// minus one. Proviso's bonus gold is throughput (more bars per order, drawn
/// from stock); Tequila's LMD rider is not (same bars, more LMD per bar).
/// The yield model couples the two: bars moved are bounded by the gold the
/// factories make, LMD per bar is not.
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

/// An OPTIMISTIC bound on one shape's worth at `rarity`, for candidate
/// ranking only: its solo value, or its marginal inside a room whose mix is
/// already shifted by a strong Tailoring companion - whichever is larger.
/// Tequila's "+500 above 3 gold" is worth 7 points alone at level 3 but 24
/// next to a strong Tailoring, and the ranker must not cut her before the
/// scorer can try that pairing. Never a final score (the ledger is).
pub fn optimistic_value_pct(effect: &OrderEffect, rarity: usize) -> f64 {
    let solo = value_pct(std::slice::from_ref(effect), rarity);
    let strong = OrderEffect::HigherYieldChance { strong: true };
    let with = value_pct(&[effect.clone(), strong.clone()], rarity);
    let without = value_pct(std::slice::from_ref(&strong), rarity);
    solo.max(with - without)
}

/// Average Pure Gold per order a bare post of `rarity` draws - the fill
/// model's orders-per-day basis (a level-1 post fills its buffer with more,
/// smaller orders than a level-3 post).
pub fn avg_gold_per_order(rarity: usize) -> f64 {
    let mix = MIX_BY_RARITY[rarity.clamp(1, MAX_RARITY) - 1];
    ORDER_SIZES
        .iter()
        .zip(mix)
        .map(|(size, w)| w * f64::from(size.gold))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROVISO: OrderEffect = OrderEffect::DefaultedGoldBonus { below: 4, bonus: 2 };
    const TEQUILA: OrderEffect = OrderEffect::HighOrderLmdBonus { above: 3, lmd: 500 };

    #[test]
    fn proviso_is_worth_more_in_lower_posts() {
        // Level 1: every order is a 2-gold order, all defaulted: 2 -> 4 gold.
        assert!((value_pct(&[PROVISO], 1) - 100.0).abs() < 1e-9);
        // Level 2: 60/40 low/med, all below 4: (0.6*2000 + 0.4*2500) / 1200.
        assert!((value_pct(&[PROVISO], 2) - (2200.0 / 1200.0 - 1.0) * 100.0).abs() < 1e-9);
        // Level 3: the 4-gold fifth earns nothing: 2250 / 1450.
        assert!((value_pct(&[PROVISO], 3) - (2250.0 / 1450.0 - 1.0) * 100.0).abs() < 1e-9);
    }

    #[test]
    fn tequila_needs_high_orders() {
        assert!(
            value_pct(&[TEQUILA], 2).abs() < 1e-9,
            "no 4-gold orders below level 3"
        );
        assert!((value_pct(&[TEQUILA], 3) - (1550.0 / 1450.0 - 1.0) * 100.0).abs() < 1e-9);
    }

    #[test]
    fn proviso_and_tequila_compose_on_disjoint_orders() {
        let both = value_pct(&[PROVISO, TEQUILA], 3);
        let alone = value_pct(&[PROVISO], 3);
        assert!(
            both > alone,
            "Tequila still boosts the 4-gold orders Proviso leaves alone"
        );
        assert!((both - (2350.0 / 1450.0 - 1.0) * 100.0).abs() < 1e-9);
    }

    #[test]
    fn tailoring_shifts_the_mix_only_where_high_orders_exist() {
        let strong = OrderEffect::HigherYieldChance { strong: true };
        assert!(value_pct(std::slice::from_ref(&strong), 2).abs() < 1e-9);
        // Bigger orders take proportionally longer: the throughput gain is small.
        let v = value_pct(&[strong], 3);
        assert!(
            v > 0.0 && v < 5.0,
            "tailoring throughput at L3 is marginal: {v}"
        );
    }

    #[test]
    fn optimistic_bound_never_undercuts_a_pairing() {
        // Tequila alone is a sliver; next to strong Tailoring she is the point.
        let alone = value_pct(&[TEQUILA], 3);
        let bound = optimistic_value_pct(&TEQUILA, 3);
        assert!(bound > alone, "bound {bound} must exceed the solo {alone}");
        // Proviso loses defaulted orders under a high-order mix: her bound is her solo.
        assert!((optimistic_value_pct(&PROVISO, 3) - value_pct(&[PROVISO], 3)).abs() < 1e-9);
    }

    #[test]
    fn proviso_moves_gold_and_tequila_moves_lmd() {
        // Proviso: every extra LMD is an extra bar from stock - gold and LMD
        // throughput rise together.
        assert!((gold_pct(&[PROVISO], 2) - value_pct(&[PROVISO], 2)).abs() < 1e-9);
        assert!((gold_pct(&[PROVISO], 3) - value_pct(&[PROVISO], 3)).abs() < 1e-9);
        // Tequila: the same bars pay more - no extra gold consumed at all.
        assert!(gold_pct(&[TEQUILA], 3).abs() < 1e-9);
        assert!(value_pct(&[TEQUILA], 3) > 0.0);
        // Beside strong Tailoring the mix shifts (bigger, slower orders): gold
        // throughput barely moves while Tequila's LMD rider fires on 85%.
        let strong = OrderEffect::HigherYieldChance { strong: true };
        let g = gold_pct(&[TEQUILA, strong.clone()], 3);
        let v = value_pct(&[TEQUILA, strong], 3);
        assert!(g.abs() < 5.0 && v > 20.0, "gold {g} vs value {v}");
    }

    #[test]
    fn average_gold_per_order_grows_with_rarity() {
        assert!((avg_gold_per_order(1) - 2.0).abs() < 1e-9);
        assert!((avg_gold_per_order(2) - 2.4).abs() < 1e-9);
        assert!((avg_gold_per_order(3) - 2.9).abs() < 1e-9);
    }
}
