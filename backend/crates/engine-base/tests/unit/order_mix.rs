use crate::registry::OrderEffect;
use crate::scoring::order_mix::*;

const PROVISO: OrderEffect = OrderEffect::DefaultedGoldBonus { below: 4, bonus: 2 };
const TEQUILA: OrderEffect = OrderEffect::HighOrderLmdBonus { above: 3, lmd: 500 };

#[test]
fn proviso_is_worth_more_in_lower_posts() {
    assert!((value_pct(&[PROVISO], 1) - 100.0).abs() < 1e-9);
    assert!((value_pct(&[PROVISO], 2) - (2200.0 / 1200.0 - 1.0) * 100.0).abs() < 1e-9);
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
    let v = value_pct(&[strong], 3);
    assert!(
        v > 0.0 && v < 5.0,
        "tailoring throughput at L3 is marginal: {v}"
    );
}

#[test]
fn optimistic_bound_never_undercuts_a_pairing() {
    let alone = value_pct(&[TEQUILA], 3);
    let bound = optimistic_value_pct(&TEQUILA, 3);
    assert!(bound > alone, "bound {bound} must exceed the solo {alone}");
    assert!((optimistic_value_pct(&PROVISO, 3) - value_pct(&[PROVISO], 3)).abs() < 1e-9);
}

#[test]
fn proviso_moves_gold_and_tequila_moves_lmd() {
    assert!((gold_pct(&[PROVISO], 2) - value_pct(&[PROVISO], 2)).abs() < 1e-9);
    assert!((gold_pct(&[PROVISO], 3) - value_pct(&[PROVISO], 3)).abs() < 1e-9);
    assert!(gold_pct(&[TEQUILA], 3).abs() < 1e-9);
    assert!(value_pct(&[TEQUILA], 3) > 0.0);
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
