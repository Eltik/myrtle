//! The LMD-income figures behind the max-level card, against real game data:
//! the mission table resolves a live daily period and live weekly chests,
//! and the farming stage's sanity and LMD are where the game data keeps them.
mod common;

use common::load_game_data;

#[test]
fn mission_chests_resolve_for_today() {
    let gd = load_game_data();
    let now = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
    )
    .unwrap_or(0);
    let daily = gd
        .missions
        .daily_lmd_per_day(now)
        .expect("a live daily period");
    let weekly = gd
        .missions
        .weekly_lmd_per_day(now)
        .expect("live weekly chests");
    assert!(
        daily > 0.0 && daily < 100_000.0,
        "daily chests pay LMD: {daily}"
    );
    assert!(
        weekly > 0.0 && weekly < 100_000.0,
        "weekly chests pay LMD a day: {weekly}"
    );
    // The weekday and weekend groups pay the same today; the weighted
    // average is then exactly one group's chests.
    assert!(
        (daily - daily.round()).abs() < 1e-6,
        "a whole-LMD figure, got {daily}"
    );
}

#[test]
fn the_farming_stage_costs_sanity_and_pays_lmd() {
    let gd = load_game_data();
    let ce6 = gd
        .stages
        .values()
        .find(|s| s.code == "CE-6" && !s.stage_id.contains('#'))
        .expect("CE-6 in the stage table");
    assert!(
        ce6.ap_cost > 0 && ce6.gold_gain > 0,
        "sanity {} LMD {}",
        ce6.ap_cost,
        ce6.gold_gain
    );
    let regen = gd.consts.player_ap_regen_speed;
    assert!(regen > 0, "PlayerApRegenSpeed parsed: {regen}");
    #[allow(clippy::cast_precision_loss)]
    let runs = 1440.0 / regen as f64 / f64::from(ce6.ap_cost);
    assert!(
        (6.0..=7.0).contains(&runs),
        "a day's sanity buys {runs} runs"
    );
}
