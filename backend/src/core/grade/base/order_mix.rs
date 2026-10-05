//! Trading-post order economics. Order-value skills are stored as SHAPES
//! (`OrderEffect`), never a guessed %, and priced here against the post's
//! `OrderRarity` (gamedata `TradingData`). So Proviso's "+2 gold on orders below
//! 4" is +100% at level 1, +83% at level 2, +55% at level 3, where 4-gold orders
//! earn her nothing.
//!
//! Order sizes, durations and draw weights: Trading Post reference tables,
//! cross-checked against the wiki. LMD per bar = `GOLD_BAR_LMD` (`GoldItems`
//! 3003 = 500).

use crate::core::gamedata::types::building::BuildingDataFile;

use super::buff_registry::OrderEffect;
use super::yield_model::GOLD_BAR_LMD;

/// One LMD order size: Pure Gold traded and completion time in minutes.
#[derive(Clone, Copy)]
struct OrderSize {
    gold: u32,
    minutes: f64,
}

/// Low / medium / high yield.
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

/// Tailoring mix at rarity 3, by the room's summed STEPS: "increased slightly"
/// (α: Shamare, E0 Bibeak) = 1, "increased" (β: E2 Bibeak) = 2; tailors stack
/// (no strongest-only clause). Index = steps - 1, capped. Below rarity 3 there's
/// no high-yield order to shift toward and no published mix: never guess.
///
/// Fitted to the community sheet's Shamare-squad table, eight level-3 figures
/// (Shamare + other 91.6; E0/E2 Tequila + other 108.07/124.39; with E0 Bibeak
/// 110.08/128.12; E2 Bibeak + other 92.8, with E0/E2 Tequila 115.52/138.21): all
/// eight within 0.05 (least squares, 0.01 grid, 2026-09-20). Earlier guesses
/// (0.15/0.30/0.55, 0.05/0.10/0.85) read Tequila 0.6-1.7 low and couldn't stack
/// two α tiers.
const TAILORING_MIX_BY_STEPS: [[f64; 3]; 3] =
    [[0.18, 0.25, 0.57], [0.14, 0.21, 0.65], [0.05, 0.06, 0.89]];

const MAX_RARITY: usize = 3;

/// Order rarity for a post of `level`. A level past the data uses the top tier.
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

/// Same-kind effects take the strongest; different kinds compose, since they hit
/// disjoint orders (below-4 vs above-3).
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

/// Proviso's bonus is gold DRAWN FROM STOCK (base expert, 2026-09-08: "it just
/// is more gold consumed per order"); Tequila's flat LMD rider consumes none.
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
            (t + w * per_order(*size), m + w * size.minutes)
        });
    if minutes <= 0.0 { 0.0 } else { total / minutes }
}

/// Order-VALUE %: LMD/hr with `effects` over a bare post's at `rarity`, minus one.
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

/// Gold-THROUGHPUT %: bars consumed/hr with `effects` over a bare post's, minus
/// one. Proviso's bonus is throughput; Tequila's rider is not (same bars, more LMD
/// each). Matters because the yield model caps bars moved at factory gold output.
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

/// OPTIMISTIC bound for candidate ranking only, never a final score: max of solo
/// value and the marginal beside a strong Tailoring. Tequila's "+500 above 3 gold"
/// is 7 points alone at level 3 but 24 beside strong Tailoring; the ranker must
/// not cut her before the scorer tries the pairing.
pub fn optimistic_value_pct(effect: &OrderEffect, rarity: usize) -> f64 {
    let solo = value_pct(std::slice::from_ref(effect), rarity);
    let strong = OrderEffect::HigherYieldChance { strong: true };
    let with = value_pct(&[effect.clone(), strong.clone()], rarity);
    let without = value_pct(std::slice::from_ref(&strong), rarity);
    solo.max(with - without)
}

/// [`optimistic_value_pct`] plus the best marginal beside any shape the roster
/// fields. Bibeak's Tailoring is a sliver alone but makes Tequila's rider pay
/// (+17 beside him); the ranker must not cut her before the scorer tries the trio.
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

/// Bars a bare post sells per day at 100% (level 3: 2.9 gold per 203.4 min =
/// 20.53; level 1: exactly 20).
pub fn bars_per_day(rarity: usize) -> f64 {
    let mix = MIX_BY_RARITY[rarity.clamp(1, MAX_RARITY) - 1];
    let (gold, minutes) = ORDER_SIZES
        .iter()
        .zip(mix)
        .fold((0.0, 0.0), |(g, m), (size, w)| {
            (g + w * f64::from(size.gold), m + w * size.minutes)
        });
    if minutes <= 0.0 {
        0.0
    } else {
        gold / minutes * 1440.0
    }
}

/// The fill model's orders-per-day basis (a level-1 post fills its buffer with
/// more, smaller orders than a level-3 one).
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
        // Proviso: every extra LMD is an extra bar from stock.
        assert!((gold_pct(&[PROVISO], 2) - value_pct(&[PROVISO], 2)).abs() < 1e-9);
        assert!((gold_pct(&[PROVISO], 3) - value_pct(&[PROVISO], 3)).abs() < 1e-9);
        // Tequila: same bars pay more, no extra gold.
        assert!(gold_pct(&[TEQUILA], 3).abs() < 1e-9);
        assert!(value_pct(&[TEQUILA], 3) > 0.0);
        // Strong Tailoring: gold throughput barely moves, Tequila's rider fires on
        // most orders (65% at two steps, measured).
        let strong = OrderEffect::HigherYieldChance { strong: true };
        let g = gold_pct(&[TEQUILA, strong.clone()], 3);
        let v = value_pct(&[TEQUILA, strong], 3);
        assert!(g.abs() < 5.0 && v > 15.0, "gold {g} vs value {v}");
    }

    #[test]
    fn average_gold_per_order_grows_with_rarity() {
        assert!((avg_gold_per_order(1) - 2.0).abs() < 1e-9);
        assert!((avg_gold_per_order(2) - 2.4).abs() < 1e-9);
        assert!((avg_gold_per_order(3) - 2.9).abs() < 1e-9);
    }
}
