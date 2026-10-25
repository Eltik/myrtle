//! Snapshot tests for the hand formula layer (`src/dps/custom/hand/`).
//!
//! A hand operator has no Python reference, so the parity suite cannot cover it.
//! Each one gets a snapshot here: the engine's exact output for a few cases,
//! pinned bit for bit, AND a value derived by hand from the game's own text, which
//! the snapshot must match to 1e-6 relative (the gap is the f32 blackboard values
//! against the decimal percentages the text prints). The derivation is written out
//! next to each case so a reviewer can redo it from the text.
//!
//! Defaults the derivations rely on (`OperatorParams::default()`): E2 at max level,
//! trust 100 (the full trust ATK bonus), potential 1, max skill rank, module 0,
//! every conditional ON, one target.

// Pinned values are full-precision `{:?}` dumps; separators would hide that.
#![allow(clippy::unreadable_literal)]

use backend::core::gamedata;
use backend::core::gamedata::types::GameData;
use backend::dps::engine;
use backend::dps::operator_unit::{EnemyStats, OperatorParams};
use std::path::Path;

fn load(data_dir: &str, assets_dir: &str) -> GameData {
    dotenv::dotenv().ok();
    let (game_data, _assets) = gamedata::init_game_data(Path::new(data_dir), Path::new(assets_dir))
        .expect("Failed to load game data");
    game_data
}

/// `(skill_dps, total_damage, average_dps)` for `op_id` at `skill`, module 0.
fn run(gd: &GameData, op_id: &str, skill: i32, defense: f64, res: f64) -> (f64, f64, f64) {
    let operator = gd
        .operators
        .get(op_id)
        .unwrap_or_else(|| panic!("{op_id} not in this game data"));
    let params = OperatorParams {
        skill_index: Some(skill),
        module_index: Some(0),
        ..Default::default()
    };
    let r = engine::calculate_dps(operator, params, &EnemyStats { defense, res })
        .unwrap_or_else(|| panic!("{op_id} S{skill}: no result"));
    (r.skill_dps, r.total_damage, r.average_dps)
}

fn assert_hand_check(case: &str, engine_value: f64, from_text: f64) {
    let rel = ((engine_value - from_text) / from_text).abs();
    assert!(
        rel < 1e-6,
        "{case}: engine {engine_value:?} vs game-text derivation {from_text:?} (rel {rel:e})"
    );
}

/// Angelina the Mellow Wish (`char_1015_aglna2`). CN-only on 2026-10-25, so this
/// reads the CN tree (`DPS_HAND_CN_DATA_DIR` / `DPS_HAND_CN_ASSETS_DIR` override).
///
/// Game text, CN `character_table` / `skill_table`, checked by hand 2026-10-25:
/// - E2 max level ATK 796, base attack time 1.5; trust 100 adds ATK +40
///   (`FavorKeyFrames` level 50). ATK 836.
/// - Talent 1 at E2, P1: "攻击重量较轻（小于等于3）的敌人时则额外造成相当于攻击力35%的
///   法术伤害" (Arts damage = 35% ATK per attack; the conditional ON is the light
///   target, which her own Weightless makes the normal case).
/// - Talent 2: "所有处于起飞状态的友方干员攻击力+13%" (ATK +13%, she is in Liftoff).
/// - S3 rank 10: "攻击力+30%", "攻击力380%的物理伤害" (ATK +30%, 380% physical).
///
/// Derivations (one target):
/// - S0, DEF 0 RES 0: 836 * (1 + 0.13) = 944.68 ATK; (944.68 + 944.68 * 0.35) / 1.5
///   = 1275.318 / 1.5 = 850.212.
/// - S3, DEF 0 RES 0: 836 * (1 + 0.13 + 0.30) = 1195.48 ATK; physical 1195.48 * 3.8 =
///   4542.824, Arts 1195.48 * 0.35 = 418.418; 4961.242 / 1.5 = 3307.494666...
/// - S3, DEF 1000 RES 50: (4542.824 - 1000) + 418.418 * 0.5 = 3752.033; / 1.5 =
///   2501.355333...
#[test]
#[ignore = "needs CN game data: Angelina the Mellow Wish is not in EN"]
fn angelina_the_mellow_wish_snapshot() {
    let data = std::env::var("DPS_HAND_CN_DATA_DIR")
        .unwrap_or_else(|_| "../assets/output/cn/gamedata/excel".into());
    let assets =
        std::env::var("DPS_HAND_CN_ASSETS_DIR").unwrap_or_else(|_| "../assets/output/cn".into());
    let gd = load(&data, &assets);
    let id = "char_1015_aglna2";

    // Snapshot, bit for bit (values of the formula as moved out of generated.rs).
    assert_eq!(
        run(&gd, id, 0, 0.0, 0.0),
        (850.2119926584561, 850.2119926584561, 850.2119926584561)
    );
    assert_eq!(
        run(&gd, id, 3, 0.0, 0.0),
        (3307.4946404563584, 3307.4946404563584, 3307.4946404563584)
    );
    assert_eq!(
        run(&gd, id, 3, 1000.0, 50.0),
        (2501.355308800618, 2501.355308800618, 2501.355308800618)
    );

    // The same cases against the game-text derivations above.
    assert_hand_check(
        "S0 DEF 0 RES 0",
        run(&gd, id, 0, 0.0, 0.0).0,
        1275.318 / 1.5,
    );
    assert_hand_check(
        "S3 DEF 0 RES 0",
        run(&gd, id, 3, 0.0, 0.0).0,
        4961.242 / 1.5,
    );
    assert_hand_check(
        "S3 DEF 1000 RES 50",
        run(&gd, id, 3, 1000.0, 50.0).0,
        3752.033 / 1.5,
    );
}
