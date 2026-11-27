use crate::scoring::yield_model::*;

#[test]
fn a_slashed_order_limit_caps_a_post_at_its_collections() {
    let rate = trading_bars_per_day(1) * productivity_mult(100.0);
    let mut free = BaseFlows::default();
    free.add_room("TRADING", None, 1, 100.0, 0.0, 0.0, 0, None);
    assert!((free.gold_sell_capacity - rate).abs() < 1e-9);
    let mut capped = BaseFlows::default();
    capped.add_room("TRADING", None, 1, 100.0, 0.0, 0.0, 0, Some(1));
    let expect = TRADING_COLLECTIONS_PER_DAY * trading_avg_gold_per_order(1);
    assert!((capped.gold_sell_capacity - expect).abs() < 1e-9);
    let mut roomy = BaseFlows::default();
    roomy.add_room("TRADING", None, 1, 100.0, 0.0, 0.0, 0, Some(100));
    assert!((roomy.gold_sell_capacity - rate).abs() < 1e-9);
}

#[test]
fn posts_without_any_gold_factory_sell_from_stock() {
    let mut stock = BaseFlows::default();
    stock.add_room("TRADING", None, 1, 100.0, 0.0, 0.0, 0, None);
    assert!((stock.realized_lmd() - 40.0 * GOLD_BAR_LMD).abs() < 1e-6);
    let mut idle = BaseFlows::default();
    idle.add_room("TRADING", None, 1, 100.0, 0.0, 0.0, 0, None);
    idle.add_room("MANUFACTURE", Some("F_GOLD"), 3, -100.0, 0.0, 0.0, 0, None);
    assert!(idle.realized_lmd().abs() < 1e-6);
}

#[test]
fn proviso_moves_bars_from_stock_and_tequila_pays_more_per_bar() {
    let mut starved = BaseFlows::default();
    starved.add_room("MANUFACTURE", Some("F_GOLD"), 3, 120.0, 0.0, 0.0, 0, None);
    starved.add_room("TRADING", None, 1, 200.0, 55.0, 55.0, 0, None);
    assert!((starved.realized_lmd() - 44.0 * GOLD_BAR_LMD).abs() < 1e-6);
    let mut rich = BaseFlows::default();
    rich.add_room("MANUFACTURE", Some("F_GOLD"), 3, 4900.0, 0.0, 0.0, 0, None);
    rich.add_room("TRADING", None, 1, 200.0, 55.0, 55.0, 0, None);
    assert!((rich.realized_lmd() - 60.0 * 1.55 * GOLD_BAR_LMD).abs() < 1e-6);

    let mut tequila = BaseFlows::default();
    tequila.add_room("MANUFACTURE", Some("F_GOLD"), 3, 120.0, 0.0, 0.0, 0, None);
    tequila.add_room("TRADING", None, 1, 200.0, 0.0, 24.0, 0, None);
    assert!((tequila.realized_lmd() - 44.0 * 1.24 * GOLD_BAR_LMD).abs() < 1e-6);
}

#[test]
fn drones_fill_the_gold_the_posts_could_still_sell_then_go_to_exp() {
    let mut f = BaseFlows::default();
    f.add_room("TRADING", None, 3, 119.0, 0.0, 55.17, 3, None);
    f.add_room("MANUFACTURE", Some("F_GOLD"), 3, 115.8, 0.0, 0.0, 3, None);
    for eff in [132.0, 132.0, 112.0, 103.0] {
        f.add_room("MANUFACTURE", Some("F_EXP"), 3, eff, 0.0, 0.0, 3, None);
    }
    for pct in [23.5, 20.0, 20.0] {
        f.add_room("POWER", None, 3, pct, 0.0, 0.0, 1, None);
    }
    let d = f.drones();
    assert!((d.per_day - 392.4).abs() < 0.1, "{d:?}");
    let slack = f.gold_sell_capacity - f.gold_produced;
    assert!(slack > 0.0 && d.to_gold > 0.0 && d.to_exp > 0.0, "{d:?}");
    assert!(
        (d.gold_bars - d.to_gold * 2.188 / 24.0).abs() < 1e-6,
        "{d:?}"
    );
    assert!(
        d.gold_bars <= slack + 1e-6,
        "drones never exceed what the posts sell: {d:?}"
    );
    assert!((f.lmd_per_day() - (f.realized_lmd() + d.lmd)).abs() < 1e-6);
    assert!(
        (d.gold_bars - slack).abs() < 1e-6,
        "the slack is filled: {d:?}"
    );
    assert!(
        f.lmd_per_day() > 35_000.0 && f.lmd_per_day() < 36_000.0,
        "{}",
        f.lmd_per_day()
    );
    assert!(f.exp_per_day() > f.exp);
}

#[test]
fn drones_go_to_exp_when_the_posts_are_already_saturated() {
    let mut f = BaseFlows::default();
    f.add_room("TRADING", None, 3, 50.0, 0.0, 0.0, 3, None);
    f.add_room("MANUFACTURE", Some("F_GOLD"), 3, 100.0, 0.0, 0.0, 3, None);
    f.add_room("MANUFACTURE", Some("F_GOLD"), 3, 100.0, 0.0, 0.0, 3, None);
    f.add_room("MANUFACTURE", Some("F_EXP"), 3, 100.0, 0.0, 0.0, 3, None);
    let d = f.drones();
    assert!(d.to_gold == 0.0 && (d.to_exp - 240.0).abs() < 1e-9, "{d:?}");
    assert!(
        (d.exp - 240.0 * 180.0 / 86400.0 * 8000.0 * 2.03).abs() < 1e-6,
        "{d:?}"
    );
    assert!((f.lmd_per_day() - f.realized_lmd()).abs() < 1e-9);
}

#[allow(clippy::float_cmp)] // exact zero is the claim
#[test]
fn drones_are_worth_nothing_without_a_line_to_spend_them_on() {
    let mut f = BaseFlows::default();
    f.add_room("POWER", None, 3, 20.0, 0.0, 0.0, 1, None);
    let d = f.drones();
    assert!(
        (d.per_day - 288.0).abs() < 1e-9 && d.lmd == 0.0 && d.exp == 0.0,
        "{d:?}"
    );
    assert_eq!(f.value_with_drones(), 0.0);
    assert!((f.total_value() - 0.2 * 240.0 * 180.0 / 86400.0 * 20.0 * 500.0).abs() < 1e-6);
}
