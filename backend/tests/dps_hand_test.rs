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
//!
//! Layout: the shared helpers sit at the top (`en`, `params`, `with`, `dps`, `hps`,
//! `assert_hand_check`); each batch of hand operators gets one `mod` named after their
//! id suffixes, importing what it uses from `super`, with one `#[ignore]`d
//! `<operator>_snapshot` test per operator (run with `-- --include-ignored`).

// Pinned values are full-precision `{:?}` dumps; separators would hide that.
#![allow(clippy::unreadable_literal)]

use backend::core::gamedata;
use backend::core::gamedata::types::GameData;
use backend::dps::engine;
use backend::dps::operator_unit::{EnemyStats, OperatorConditionals, OperatorParams};
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

/// EN game data (`GAME_DATA_DIR` / `ASSETS_DIR` override), for the operators on the EN
/// server.
fn en() -> GameData {
    let data = std::env::var("GAME_DATA_DIR")
        .unwrap_or_else(|_| "../assets/output/en/gamedata/excel".into());
    let assets = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output/en".into());
    load(&data, &assets)
}

/// Defaults (every conditional ON, potential 1, trust 100, E2 90, rank 10) at
/// `skill`, `module` (level 3 when not 0) and `targets`.
fn params(skill: i32, module: i32, targets: i32) -> OperatorParams {
    OperatorParams {
        skill_index: Some(skill),
        module_index: Some(module),
        module_level: Some(3),
        targets: Some(targets),
        ..Default::default()
    }
}

/// `p` with `conditionals` set.
fn with(conditionals: OperatorConditionals, p: OperatorParams) -> OperatorParams {
    OperatorParams {
        conditionals: Some(conditionals),
        ..p
    }
}

/// `(skill_dps, total_damage, average_dps)`, or `(skill_hps, base_hps, avg_hps)`.
type Metrics = (f64, f64, f64);

/// `(skill_dps, total_damage, average_dps)`.
fn dps(gd: &GameData, id: &str, p: OperatorParams, defense: f64, res: f64) -> Metrics {
    let op = gd
        .operators
        .get(id)
        .unwrap_or_else(|| panic!("{id} not in game data"));
    let r = engine::calculate_dps(op, p, &EnemyStats { defense, res })
        .unwrap_or_else(|| panic!("{id}: no result"));
    (r.skill_dps, r.total_damage, r.average_dps)
}

/// `(skill_hps, base_hps, avg_hps)`.
fn hps(gd: &GameData, id: &str, p: OperatorParams) -> Metrics {
    let op = gd
        .operators
        .get(id)
        .unwrap_or_else(|| panic!("{id} not in game data"));
    let r = engine::calculate_hps(op, p).unwrap_or_else(|| panic!("{id}: no HPS"));
    (r.skill_hps, r.base_hps, r.avg_hps)
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

/// Astgenne the Lightchaser, Nasti, Kal'tsit Esperanta (EN game data).
mod halo2_nasti_kalts2 {
    use super::{assert_hand_check, dps, en, hps, params};
    use backend::dps::engine;
    use backend::dps::operator_unit::{EnemyStats, OperatorConditionals, OperatorParams};

    /// Astgenne the Lightchaser (`char_1047_halo2`).
    ///
    /// Game text, EN `character_table` / `skill_table` / `battle_equip_table`, checked
    /// by hand 2026-10-25:
    /// - E2 90 ATK 533, base attack time 1.9; trust 100 ATK +75. ATK 608.
    /// - Talent 1 (E2): "When inflicting Slow on an enemy, +1 ASPD, stacking up to 18
    ///   times" (ON: +18 ASPD, ASPD 118).
    /// - Talent 2 (E2, P1): "Enemies within attack range gain 10% Fragile, increasing to
    ///   14% Fragile after ... more than 7 seconds" (ON: x1.14).
    /// - S1 rank 10: "+100% ATK, attacks can jump between enemies ... up to 3 times".
    /// - S2 rank 10, SP 3 per attack: "next attack ... furthest 2 enemies ... 145% ATK
    ///   as Arts damage ... links to up to 3 enemies around the target ... the same
    ///   amount of Arts damage".
    /// - S3 rank 10: "+70% ATK, ATK interval decreases" (`base_attack_time` -0.7, so
    ///   1.2 s), "lock onto 2 enemies ... deal 25% of the Arts damage taken to other
    ///   chained enemies".
    /// - Module `uniequip_002_halo2` level 3: ATK +50, "stacking up to 25 times; +12%
    ///   ATK at full stacks".
    ///
    /// Derivations (DPS = hit / interval x ASPD / 100 x 1.14):
    /// - S0, RES 0: 608 / 1.9 x 1.18 x 1.14 = 430.464. RES 50: 304 / 1.9 x 1.18 x 1.14 =
    ///   215.232.
    /// - S1, 1 target: 1216 / 1.9 x 1.18 x 1.14 = 860.928. 3 targets, 1 + 3 hits: 4 x
    ///   1216 / 1.9 x 1.18 x 1.14 = 3443.712.
    /// - S2, 1 target: skill hit 608 x 1.45 = 881.6; average hit (3 x 608 + 881.6) / 4 =
    ///   676.4; / 1.9 x 1.18 x 1.14 = 478.8912. RES 50: half, 239.4456. 6 targets
    ///   (2 primary + 4 linked): (1824 + 6 x 881.6) / 4 = 1778.4; / 1.9 x 1.18 x 1.14 =
    ///   1259.1072.
    /// - S3, 1 target: 608 x 1.7 = 1033.6; / 1.2 x 1.18 x 1.14 = 1158.6656. 2 targets,
    ///   2 hits each + 25% shared: 1033.6 x 2.5 / 1.2 x 1.18 x 1.14 = 2896.664. RES 50:
    ///   1448.332. Talent 2 OFF (10%), 1 target: 1033.6 / 1.2 x 1.18 x 1.10 =
    ///   1118.0106666...
    /// - S3, module level 3, 1 target: ATK 658, x (1 + 0.7 + 0.12) = 1197.56; / 1.2 x
    ///   1.25 x 1.14 = 1422.1025.
    #[test]
    #[ignore = "needs EN game data"]
    fn astgenne_the_lightchaser_snapshot() {
        let gd = en();
        let id = "char_1047_halo2";
        let off2 = || OperatorParams {
            conditionals: Some(OperatorConditionals {
                talent2_damage: Some(false),
                ..Default::default()
            }),
            ..params(3, 0, 1)
        };

        let cases: [(&str, OperatorParams, f64, f64, f64); 12] = [
            ("S0 RES 0", params(0, 0, 1), 0.0, 0.0, 430.464),
            ("S0 DEF 500 RES 50", params(0, 0, 1), 500.0, 50.0, 215.232),
            ("S1 1 target", params(1, 0, 1), 0.0, 0.0, 860.928),
            ("S1 3 targets", params(1, 0, 3), 0.0, 0.0, 3443.712),
            ("S2 1 target", params(2, 0, 1), 0.0, 0.0, 478.8912),
            ("S2 RES 50", params(2, 0, 1), 0.0, 50.0, 239.4456),
            ("S2 6 targets", params(2, 0, 6), 0.0, 0.0, 1259.1072),
            ("S3 1 target", params(3, 0, 1), 0.0, 0.0, 1158.6656),
            ("S3 2 targets", params(3, 0, 2), 0.0, 0.0, 2896.664),
            (
                "S3 2 targets RES 50",
                params(3, 0, 2),
                300.0,
                50.0,
                1448.332,
            ),
            (
                "S3 talent 2 OFF",
                off2(),
                0.0,
                0.0,
                1033.6 / 1.2 * 1.18 * 1.10,
            ),
            ("S3 module 1 level 3", params(3, 1, 1), 0.0, 0.0, 1422.1025),
        ];
        // Snapshot, bit for bit: (skill_dps, total_damage, average_dps), same order.
        let pins = [
            (430.464, 430.464, 430.464),
            (215.232, 215.232, 215.232),
            (860.928, 17218.56, 536.3712024092825),
            (3443.712, 68874.24, 1569.4848024092828),
            (478.8912051315308, 478.8912051315308, 478.8912051315308),
            (239.4456025657654, 239.4456025657654, 239.4456025657654),
            (1259.1072307891845, 1259.1072307891845, 1259.1072307891845),
            (1158.6655888460477, 28966.639721151194, 669.4439976948779),
            (2896.6639721151196, 72416.59930287799, 1393.6099907236578),
            (1448.3319860575598, 36208.299651438996, 696.8049953618289),
            (1118.0106941654947, 27950.267354137366, 652.5044582446475),
            (1422.1024848717575, 35552.56212179394, 794.5602484245343),
        ];
        for ((case, p, d, r, from_text), pin) in cases.into_iter().zip(pins) {
            let got = dps(&gd, id, p, d, r);
            println!("halo2 {case}: {got:?}");
            assert_eq!(got, pin, "{case}");
            assert_hand_check(case, got.0, from_text);
        }

        // Module level 2: ATK +45 (653), 23 stacks (ASPD 123), no +12% ATK yet.
        // S3, 1 target: 653 x 1.7 = 1110.1; / 1.2 x 1.23 x 1.14 = 1297.1518...
        let l2 = OperatorParams {
            module_level: Some(2),
            ..params(3, 1, 1)
        };
        let got = dps(&gd, id, l2, 0.0, 0.0);
        println!("halo2 S3 module 1 level 2: {got:?}");
        assert_eq!(
            got,
            (1297.1518375129028, 32428.79593782257, 740.962390953117)
        );
        assert_hand_check("S3 module 1 level 2", got.0, 1110.1 / 1.2 * 1.23 * 1.14);
    }

    /// Nasti (`char_4212_nasti`).
    ///
    /// Game text, EN tables, checked by hand 2026-10-25:
    /// - E2 90 ATK 614, base attack time 1.5, Block 2; trust adds no ATK; P1 (the P3
    ///   ATK +25 not taken).
    /// - S1 rank 10: "+80% ATK ... and simultaneously attacks all blocked enemies".
    /// - S2: "Stops attacking".
    /// - S3 rank 10: "+160% ATK".
    /// - Module `uniequip_002_nasti` level 3: ATK +40.
    ///
    /// Derivations (physical, one hit per attack per target):
    /// - S0: 614 / 1.5 = 409.333... DEF 300: 314 / 1.5 = 209.333...
    /// - S1, 2 targets: 614 x 1.8 = 1105.2; x 2 / 1.5 = 1473.6. DEF 300: 805.2 x 2 / 1.5
    ///   = 1073.6. 5 targets still hit only the 2 blocked: 1473.6.
    /// - S2: 0.
    /// - S3: 614 x 2.6 = 1596.4; / 1.5 = 1064.2666... DEF 500: 1096.4 / 1.5 =
    ///   730.9333...
    /// - S3 module level 3: 654 x 2.6 / 1.5 = 1133.6.
    #[test]
    #[ignore = "needs EN game data"]
    fn nasti_snapshot() {
        let gd = en();
        let id = "char_4212_nasti";
        assert_eq!(
            dps(&gd, id, params(2, 0, 1), 0.0, 0.0).0,
            0.0,
            "S2 stops attacking"
        );
        let cases: [(&str, OperatorParams, f64, f64, f64); 8] = [
            ("S0 DEF 0", params(0, 0, 1), 0.0, 0.0, 614.0 / 1.5),
            ("S0 DEF 300", params(0, 0, 1), 300.0, 0.0, 314.0 / 1.5),
            ("S1 2 targets", params(1, 0, 2), 0.0, 0.0, 1473.6),
            ("S1 2 targets DEF 300", params(1, 0, 2), 300.0, 20.0, 1073.6),
            ("S1 5 targets", params(1, 0, 5), 0.0, 0.0, 1473.6),
            ("S3 DEF 0", params(3, 0, 1), 0.0, 0.0, 1596.4 / 1.5),
            ("S3 DEF 500", params(3, 0, 1), 500.0, 0.0, 1096.4 / 1.5),
            ("S3 module 1 level 3", params(3, 1, 1), 0.0, 0.0, 1133.6),
        ];
        // Snapshot, bit for bit: (skill_dps, total_damage, average_dps), same order.
        let pins = [
            (409.33333333333326, 409.33333333333326, 409.33333333333326),
            (209.33333333333337, 209.33333333333337, 209.33333333333337),
            (1473.6000097592669, 58944.000390370675, 941.4666715463002),
            (1073.600009759267, 42944.00039037068, 700.5333382129669),
            (1473.6000097592669, 58944.000390370675, 941.4666715463002),
            (1064.2666764259338, 15964.000146389008, 654.9333369930584),
            (730.9333430926004, 10964.000146389006, 529.9333369930584),
            (1133.60001039505, 17004.00015592575, 697.6000038981438),
        ];
        for ((case, p, d, r, from_text), pin) in cases.into_iter().zip(pins) {
            let got = dps(&gd, id, p, d, r);
            println!("nasti {case}: {got:?}");
            assert_eq!(got, pin, "{case}");
            assert_hand_check(case, got.0, from_text);
        }
    }

    /// Kal'tsit Esperanta (`char_1052_kalts2`), DPS and HPS.
    ///
    /// Game text, EN tables, checked by hand 2026-10-25:
    /// - E2 90 ATK 510, base attack time 2.85; trust adds no ATK; no module.
    /// - S1 rank 10: "ATK +90%, ASPD +50"; 35 s, SP 35.
    /// - S2 rank 10: "ATK +150%, ... medical units ... deal 380% ATK as True damage to
    ///   all enemies around the target ... restore HP equal to 200% ATK to all allied
    ///   Operators around the target"; 10 ammo, SP 35.
    /// - S3 rank 10: "ATK +150%, Attack Interval decreases significantly"
    ///   (`base_attack_time` -1.55, so 1.30 s), "heals 1 additional target"; 35 s, SP 50.
    ///
    /// DPS derivations (True damage, DEF and RES do not apply):
    /// - S2: 510 x 2.5 = 1275; x 3.8 = 4845; / 2.85 = 1700 per target. DEF 1000 RES 50:
    ///   1700. 3 targets: 5100.
    /// - S2 duration (`kaltsit_esperanta::skill_duration`): 10 ammo, one per attack, 10 x
    ///   2.85 = 28.5 s. Total damage 1700 x 28.5 = 48450 (3 targets: 145350). Average
    ///   (legacy cycle: the off-skill damage is the engine's `normal_attack`, her 510 ATK
    ///   as Arts, 178.947... per s at RES 0, over 35 SP): (48450 + 178.947... x 35) /
    ///   63.5 = 861.624534...
    ///
    /// HPS derivations (base: 510 / 2.85 = 178.947368...):
    /// - S0: 178.947368... for all three fields.
    /// - S1: 969 / 2.85 x 1.5 = 510; average (510 x 35 + 178.947... x 35) / 70 =
    ///   344.473684...
    /// - S2, 1 ally: 1275 x 2 / 2.85 = 894.736842...; 10 ammo x 2.85 = 28.5 s; average
    ///   (25500 + 178.947... x 35) / 63.5 = 500.207210...
    /// - S3, 2 allies: 2 x 1275 / 1.3 = 1961.538461...; average (1961.538... x 35 +
    ///   178.947... x 50) / 85 = 912.955465...
    #[test]
    #[ignore = "needs EN game data"]
    fn kaltsit_esperanta_snapshot() {
        let gd = en();
        let id = "char_1052_kalts2";
        let dps_cases: [(&str, OperatorParams, f64, f64, f64); 3] = [
            ("S2 1 target", params(2, 0, 1), 0.0, 0.0, 4845.0 / 2.85),
            ("S2 DEF 1000 RES 50", params(2, 0, 1), 1000.0, 50.0, 1700.0),
            ("S2 3 targets", params(2, 0, 3), 0.0, 0.0, 5100.0),
        ];
        let dps_pins = [
            (1700.000035553649, 48449.99939203262, 861.6245404418593),
            (1700.000035553649, 48449.99939203262, 812.3083341554069),
            (5100.0001066609475, 145349.9981760979, 2387.608796179769),
        ];
        for ((case, p, d, r, from_text), pin) in dps_cases.into_iter().zip(dps_pins) {
            let got = dps(&gd, id, p, d, r);
            println!("kalts2 DPS {case}: {got:?}");
            assert_eq!(got, pin, "{case}");
            assert_hand_check(case, got.0, from_text);
            assert_hand_check(case, got.1, from_text * 28.5);
        }
        assert_hand_check(
            "S2 average",
            dps(&gd, id, params(2, 0, 1), 0.0, 0.0).2,
            (48450.0 + 510.0 / 2.85 * 35.0) / 63.5,
        );
        // Her normal attacks heal, so S0 deals 0; S1 and S3 are not in the DPS table.
        assert_eq!(dps(&gd, id, params(0, 0, 1), 0.0, 0.0).0, 0.0, "S0");
        let op = &gd.operators[id];
        for skill in [1, 3] {
            let r = engine::calculate_dps(op, params(skill, 0, 1), &EnemyStats::default());
            assert!(r.is_none(), "S{skill} has no DPS formula");
        }

        let base = 510.0 / 2.85;
        let hps_cases: [(&str, OperatorParams, f64, f64); 4] = [
            ("S0", params(0, 0, 1), base, base),
            ("S1", params(1, 0, 1), 510.0, (510.0 + base) / 2.0),
            (
                "S2",
                params(2, 0, 1),
                2550.0 / 2.85,
                (25500.0 + base * 35.0) / 63.5,
            ),
            (
                "S3 2 allies",
                params(3, 0, 2),
                2.0 * 1275.0 / 1.3,
                (2.0 * 1275.0 / 1.3 * 35.0 + base * 50.0) / 85.0,
            ),
        ];
        // Snapshot, bit for bit: (skill_hps, base_hps, avg_hps), same order.
        let hps_pins = [
            (178.94737440903563, 178.94737440903563, 178.94737440903563),
            (510.0000106660947, 178.94737440903563, 344.47369253756517),
            (894.7368720451781, 178.94737440903563, 500.2072217535595),
            (1961.5385334872665, 178.94737440903563, 912.955498735366),
        ];
        for ((case, p, skill_hps, avg_hps), pin) in hps_cases.into_iter().zip(hps_pins) {
            let got = hps(&gd, id, p);
            println!("kalts2 HPS {case}: {got:?}");
            assert_eq!(got, pin, "{case}");
            assert_hand_check(case, got.0, skill_hps);
            assert_hand_check(case, got.1, base);
            assert_hand_check(case, got.2, avg_hps);
        }
    }
}

/// Viy, Mantra, SilverAsh the Reignfrost, Closure (EN game data).
///
/// Every derivation below uses the game text's decimal percentages; the engine reads
/// the f32 blackboard values, hence the 1e-6 relative check. Defaults: E2 90, trust
/// 100, potential 1, max skill rank, every conditional ON, module level 3 when a
/// module is chosen.
mod veen_mantra_svash2_closur {
    use super::{assert_hand_check, dps, en, params};
    use backend::core::gamedata::types::GameData;
    use backend::dps::operator_unit::OperatorParams;

    /// `skill_dps` at module 0, one target.
    fn plain(gd: &GameData, id: &str, skill: i32, defense: f64, res: f64) -> f64 {
        dps(gd, id, params(skill, 0, 1), defense, res).0
    }

    /// Viy (`char_4226_veen`).
    ///
    /// Game text, EN `character_table` / `skill_table` / `battle_equip_table`, checked
    /// by hand 2026-10-25:
    /// - E2 90 ATK 1410, base attack time 3.0; trust 100 ATK +100. ATK 1510. Arts.
    /// - Talent 1 (E2, P1): "ATK +10% when in possession of Transposed Energy, and ASPD
    ///   +15 when not" (`transposedEnergy` ON: ATK +10%, no ASPD).
    /// - Talent 2 (E2): "Attacking with stored charges makes the target suffer 90 Arts
    ///   damage per second for 5s (stacks up to 3 times; Transposed Energy inflicts 3
    ///   stacks)" (ON: 3 x 90 = 270 per s, RES applies).
    /// - S1 rank 10: "ASPD +100". S2 rank 10: "Attack Interval decreases slightly"
    ///   (-0.5 s), per stack "ATK +15% and ASPD +8 (max 9 stacks)" (`maxStacks` ON: ATK
    ///   +135%, ASPD +72). S3 rank 10: "ASPD +180; stored charges will bounce ... once
    ///   ... dealing 165% ATK as Arts damage" (charge 100% + bounce 165% = 265%).
    /// - Module MSC-Y level 3: ATK +120, "ASPD +30 when there are stored attacks",
    ///   talent 2 "120 Arts damage per second ... stacks up to 4 times" (a TE gives 3).
    ///
    /// - Talent 2 lockout (PRTS https://prts.wiki/w/维伊, Mystic note; checked
    ///   2026-10-25): she cannot attack the target under the DoT, so each attack interval
    ///   stores a charge for the 5 s; a release carries 5 / (3.0 / (ASPD / 100)) charges,
    ///   so DoT stacks = min(3, 3 per TE, that count).
    ///
    /// Derivations (every attack a stored charge released as TE; DPS = hit / interval x
    /// ASPD / 100 + DoT):
    /// - S0, RES 0: 1510 x 1.1 = 1661; 1661 / 3 x 1.00 = 553.666...; charges in the 5 s
    ///   lockout 5 / 3.0 = 1.666... stacks, DoT 90 x 1.666... = 150; 703.666... RES 50:
    ///   830.5 / 3 + 75 = 351.833...
    /// - S0, module level 3: ATK 1630 x 1.1 = 1793; ASPD 100 + 30 = 130; 1793 / 3 x 1.3
    ///   = 776.9666...; lockout 5 / (3 / 1.3) = 2.1666... stacks (cap 4) x 120 = 260;
    ///   1036.9666...
    /// - S1 to S3 store 3 or more charges in 5 s (5 / 1.5, 5 / 1.4535, 5 / 1.0714), so
    ///   their DoT keeps 3 stacks.
    /// - S1, RES 0: 1661 / 3 x 2.00 + 270 = 1377.333... RES 30: 1162.7 / 3 x 2 + 189 =
    ///   964.1333...
    /// - S2, RES 0: 1510 x (1 + 0.10 + 1.35) = 3699.5; / 2.5 x 1.72 + 270 = 2815.256.
    ///   RES 40: 2219.7 / 2.5 x 1.72 + 162 = 1689.1536.
    /// - S3, RES 0: 1661 x 2.65 = 4401.65; / 3 x 2.80 + 270 = 4378.20666... RES 50:
    ///   half, 2189.10333...
    /// - S3, module level 3: ATK 1630 x 1.1 = 1793; x 2.65 = 4751.45; / 3 x 3.10 (ASPD
    ///   100 + 180 + 30) + 3 x 120 = 5269.83166...
    /// - S3, every conditional OFF (plain attacks: no TE, no bounce, no DoT): 1510 / 3 x
    ///   2.95 (ASPD 100 + 180 + 15) = 1484.8333...
    /// - S3 duration (`viy::skill_duration`): "Skill activation grants 27 ammo; 1 ammo is
    ///   spent per stored charge released, and 3 ammo per Transposed Energy released",
    ///   one charge per attack interval, so 27 x 3 / 2.80 = 28.928571... s; total
    ///   4378.20666... x 28.928571... = 126655.26... With the module, 27 x 3 / 3.10 =
    ///   26.129032... s. Every conditional OFF: no charge is released, no ammo is spent,
    ///   the game-data duration (-1, unlimited) stays and total = skill DPS.
    #[test]
    #[ignore = "needs EN game data"]
    fn viy_snapshot() {
        let gd = en();
        let id = "char_4226_veen";
        // Snapshot, bit for bit: (skill_dps, total_damage, average_dps), module 0.
        assert_eq!(
            dps(&gd, id, params(0, 0, 1), 0.0, 0.0),
            (703.6666674166918, 703.6666674166918, 703.6666674166918)
        );
        assert_eq!(
            dps(&gd, id, params(0, 1, 1), 0.0, 0.0),
            (1036.9666677191854, 1036.9666677191854, 1036.9666677191854)
        );
        assert_eq!(
            dps(&gd, id, params(1, 0, 1), 0.0, 0.0),
            (1377.3333348333836, 24792.000027000904, 869.1937990775403)
        );
        assert_eq!(
            dps(&gd, id, params(2, 0, 1), 0.0, 0.0),
            (2815.256057277918, 78827.1696037817, 1582.230604507473)
        );
        assert_eq!(
            dps(&gd, id, params(3, 0, 1), 0.0, 0.0),
            (4378.206635270616, 126655.2633774714, 2019.5881036566182)
        );
        assert_eq!(
            dps(&gd, id, params(3, 0, 1), 0.0, 50.0),
            (2189.103317635308, 63327.6316887357, 1009.7940518283091)
        );
        let off = || OperatorParams {
            all_cond: Some(false),
            ..params(3, 0, 1)
        };
        let s3_total = (4401.65 / 3.0 * 2.8 + 270.0) * 27.0 * 3.0 / 2.8;
        assert_hand_check(
            "S3 total",
            dps(&gd, id, params(3, 0, 1), 0.0, 0.0).1,
            s3_total,
        );
        assert_hand_check(
            "S3 module 3 total",
            dps(&gd, id, params(3, 1, 1), 0.0, 0.0).1,
            (4751.45 / 3.0 * 3.1 + 360.0) * 27.0 * 3.0 / 3.1,
        );
        let all_off = dps(&gd, id, off(), 0.0, 0.0);
        assert_eq!(
            all_off.1, all_off.0,
            "S3 all OFF: unlimited, total = skill DPS"
        );
        let cases = [
            (
                "S0 RES 0",
                plain(&gd, id, 0, 0.0, 0.0),
                1661.0 / 3.0 + 90.0 * 5.0 / 3.0,
            ),
            (
                "S0 module 3",
                dps(&gd, id, params(0, 1, 1), 0.0, 0.0).0,
                1793.0 / 3.0 * 1.3 + 120.0 * 5.0 / (3.0 / 1.3),
            ),
            (
                "S0 RES 50",
                plain(&gd, id, 0, 0.0, 50.0),
                830.5 / 3.0 + 45.0 * 5.0 / 3.0,
            ),
            (
                "S1 RES 0",
                plain(&gd, id, 1, 0.0, 0.0),
                1661.0 / 3.0 * 2.0 + 270.0,
            ),
            (
                "S1 RES 30",
                plain(&gd, id, 1, 0.0, 30.0),
                1162.7 / 3.0 * 2.0 + 189.0,
            ),
            (
                "S2 RES 0",
                plain(&gd, id, 2, 0.0, 0.0),
                3699.5 / 2.5 * 1.72 + 270.0,
            ),
            (
                "S2 RES 40",
                plain(&gd, id, 2, 0.0, 40.0),
                2219.7 / 2.5 * 1.72 + 162.0,
            ),
            (
                "S3 RES 0",
                plain(&gd, id, 3, 0.0, 0.0),
                4401.65 / 3.0 * 2.8 + 270.0,
            ),
            (
                "S3 RES 50",
                plain(&gd, id, 3, 0.0, 50.0),
                2200.825 / 3.0 * 2.8 + 135.0,
            ),
            (
                "S3 module 3",
                dps(&gd, id, params(3, 1, 1), 0.0, 0.0).0,
                4751.45 / 3.0 * 3.1 + 360.0,
            ),
            (
                "S3 all OFF",
                dps(&gd, id, off(), 0.0, 0.0).0,
                1510.0 / 3.0 * 2.95,
            ),
        ];
        for (case, got, want) in cases {
            println!("viy {case}: {got:?}");
            assert_hand_check(case, got, want);
        }
    }

    /// Mantra (`char_4204_mantra`).
    ///
    /// Game text, EN tables, checked by hand 2026-10-25:
    /// - E2 90 ATK 695, base attack time 1.6; trust 100 ATK +60. ATK 755. Arts.
    /// - S1 rank 10 (2 SP per attack): "The next attack deals 325% ATK as Arts damage
    ///   and 30% of damage as Nervous Impairment; if target is under the burst effect
    ///   ..., deals an additional 200% ATK as Elemental Damage".
    /// - S2 rank 10: "Attack Interval reduces" (-0.8 s, so 0.8 s), "attacks deal 240% ATK
    ///   as Arts damage ... an additional 18% of damage as Nervous Impairment; ... burst
    ///   ..., deals an additional 25% ATK as Elemental Damage".
    /// - S3 rank 10: "ATK +275% and attacks 2 targets at once".
    /// - Module PRI-X level 3: ATK +87; "Damage dealt to enemies under an Elemental Burst
    ///   effect increased to 110%".
    /// - Nervous Impairment (upstream constants): gauge 1000 (2000 `vsBoss`), burst 6000
    ///   Elemental damage over 10 s; average = (DPS x T + burst DPS x 10 + 6000 + P) /
    ///   (T + 10), T = 1000 / Nervous Impairment per s.
    /// - Talent 1 (E2, P1): "Enemies take 135% of Mantra's ATK as Elemental Damage when
    ///   they trigger Paralysis; ... a 10% chance to not consume a Paralysis stack";
    ///   `gamedata_const` `ba.dt.neural2`: a burst inflicts "3 stacks of Paralysis";
    ///   PRTS (https://prts.wiki/w/真言, checked 2026-10-25): the talent fires on each
    ///   interrupted attack. `targetAttacks` ON: P = 3 / (1 - 0.10) x 1.35 x ATK per
    ///   burst = 4.5 x 755 = 3397.5 (module level 3: talent 160% / 15%, P = 3 / 0.85 x
    ///   1.6 x 842 = 4754.8235...). Elemental damage: no RES.
    ///
    /// Derivations:
    /// - S0: 755 / 1.6 = 471.875. RES 30: 528.5 / 1.6 = 330.3125.
    /// - S1, RES 0: skill hit 755 x 3.25 = 2453.75; 3-attack cycle (2453.75 + 2 x 755) /
    ///   3 / 1.6 = 825.78125 DPS; Nervous Impairment 2453.75 x 0.30 / 3 / 1.6 =
    ///   153.359375 per s, T = 6.52063...; burst adds 755 x 2.0 / 3 / 1.6 = 314.58333 per
    ///   s; (825.78125 T + 1140.36458 x 10 + 6000 + 3397.5) / (T + 10) = 1585.03389...
    /// - S1, RES 30: skill hit 1717.625; (1717.625 + 2 x 528.5) / 3 / 1.6 = 578.046875;
    ///   Nervous Impairment 107.3515625 per s, T = 9.31519...; burst DPS 578.046875 +
    ///   314.58333 = 892.63021; average (with P 3397.5) 1227.44947...
    /// - S2, RES 0: hit 755 x 2.4 = 1812; / 0.8 = 2265 DPS; Nervous Impairment 1812 x
    ///   0.18 / 0.8 = 407.7 per s, T = 2.45278...; burst adds 755 x 0.25 / 0.8 = 235.9375;
    ///   (2265 T + 2500.9375 x 10 + 6000 + 3397.5) / (T + 10) = 3209.11619...
    ///   `targetAttacks` OFF (P = 0): 2936.28564...
    /// - S2, RES 30: hit 1268.4, 1585.5 DPS, 285.39 per s, T = 3.50398...; burst DPS
    ///   1821.4375; average 2456.12314...
    /// - S2, `vsBoss` OFF-flag (gauge 2000): T = 4.90557...; average 3053.75727...
    /// - S2, module level 3: ATK 842; hit 2020.8, 2526 DPS, 454.68 per s, T = 2.19935...;
    ///   burst DPS (2526 + 263.125) x 1.1 = 3068.0375; average (P 4754.8235...)
    ///   3851.90684...
    /// - S3: 755 x 3.75 = 2831.25; / 1.6 = 1769.53125. RES 30: 1238.671875.
    #[test]
    #[ignore = "needs EN game data"]
    fn mantra_snapshot() {
        let gd = en();
        let id = "char_4204_mantra";
        let no_interrupts = || OperatorParams {
            conditionals: Some(backend::dps::operator_unit::OperatorConditionals {
                talent_damage: Some(false),
                ..Default::default()
            }),
            ..params(2, 0, 1)
        };
        // Snapshot, bit for bit: (skill_dps, total_damage, average_dps), module 0.
        assert_eq!(
            dps(&gd, id, params(0, 0, 1), 0.0, 0.0),
            (471.8749929685146, 471.8749929685146, 471.8749929685146)
        );
        assert_eq!(
            dps(&gd, id, params(1, 0, 1), 0.0, 0.0),
            (1585.0338929986542, 1585.0338929986542, 1585.0338929986542)
        );
        assert_eq!(
            dps(&gd, id, params(2, 0, 1), 0.0, 0.0),
            (3209.1162687564533, 80227.90671891133, 1840.4956308624837)
        );
        assert_eq!(
            dps(&gd, id, params(2, 0, 1), 0.0, 30.0),
            (2456.1232058215764, 61403.08014553941, 1393.2178504497683)
        );
        assert_eq!(
            dps(&gd, id, no_interrupts(), 0.0, 0.0),
            (2936.2857006309187, 73407.14251577297, 1704.0803467997166)
        );
        assert_eq!(
            dps(&gd, id, params(3, 0, 1), 0.0, 0.0),
            (1769.5312236319298, 70781.2489452772, 1120.7031083002223)
        );
        let paralysis = 3.0 / (1.0 - 0.10) * 1.35 * 755.0;
        let nerv_p = |dps: f64, per_s: f64, burst: f64, gauge: f64, p: f64| {
            let t = gauge / per_s;
            (dps * t + burst * 10.0 + 6000.0 + p) / (t + 10.0)
        };
        let nerv = |dps: f64, per_s: f64, burst: f64, gauge: f64| {
            nerv_p(dps, per_s, burst, gauge, paralysis)
        };
        let boss = OperatorParams {
            conditionals: Some(backend::dps::operator_unit::OperatorConditionals {
                trait_damage: Some(false),
                ..Default::default()
            }),
            ..params(2, 0, 1)
        };
        let cases = [
            ("S0 RES 0", plain(&gd, id, 0, 0.0, 0.0), 755.0 / 1.6),
            ("S0 RES 30", plain(&gd, id, 0, 0.0, 30.0), 528.5 / 1.6),
            (
                "S1 RES 0",
                plain(&gd, id, 1, 0.0, 0.0),
                nerv(
                    (2453.75 + 2.0 * 755.0) / 3.0 / 1.6,
                    2453.75 * 0.30 / 3.0 / 1.6,
                    (2453.75 + 2.0 * 755.0) / 3.0 / 1.6 + 755.0 * 2.0 / 3.0 / 1.6,
                    1000.0,
                ),
            ),
            (
                "S1 RES 30",
                plain(&gd, id, 1, 0.0, 30.0),
                nerv(
                    (1717.625 + 2.0 * 528.5) / 3.0 / 1.6,
                    1717.625 * 0.30 / 3.0 / 1.6,
                    (1717.625 + 2.0 * 528.5) / 3.0 / 1.6 + 755.0 * 2.0 / 3.0 / 1.6,
                    1000.0,
                ),
            ),
            (
                "S2 RES 0",
                plain(&gd, id, 2, 0.0, 0.0),
                nerv(
                    2265.0,
                    1812.0 * 0.18 / 0.8,
                    2265.0 + 755.0 * 0.25 / 0.8,
                    1000.0,
                ),
            ),
            (
                "S2 RES 30",
                plain(&gd, id, 2, 0.0, 30.0),
                nerv(
                    1585.5,
                    1268.4 * 0.18 / 0.8,
                    1585.5 + 755.0 * 0.25 / 0.8,
                    1000.0,
                ),
            ),
            (
                "S2 vsBoss",
                dps(&gd, id, boss, 0.0, 0.0).0,
                nerv(
                    2265.0,
                    1812.0 * 0.18 / 0.8,
                    2265.0 + 755.0 * 0.25 / 0.8,
                    2000.0,
                ),
            ),
            (
                "S2 module 3",
                dps(&gd, id, params(2, 1, 1), 0.0, 0.0).0,
                nerv_p(
                    2526.0,
                    2020.8 * 0.18 / 0.8,
                    (2526.0 + 842.0 * 0.25 / 0.8) * 1.1,
                    1000.0,
                    3.0 / (1.0 - 0.15) * 1.6 * 842.0,
                ),
            ),
            (
                "S2 targetAttacks OFF",
                dps(&gd, id, no_interrupts(), 0.0, 0.0).0,
                nerv_p(
                    2265.0,
                    1812.0 * 0.18 / 0.8,
                    2265.0 + 755.0 * 0.25 / 0.8,
                    1000.0,
                    0.0,
                ),
            ),
            ("S3 RES 0", plain(&gd, id, 3, 0.0, 0.0), 2831.25 / 1.6),
            ("S3 RES 30", plain(&gd, id, 3, 0.0, 30.0), 1981.875 / 1.6),
        ];
        for (case, got, want) in cases {
            println!("mantra {case}: {got:?}");
            assert_hand_check(case, got, want);
        }
    }

    /// SilverAsh the Reignfrost (`char_1045_svash2`). No module.
    ///
    /// Game text, EN tables, checked by hand 2026-10-25:
    /// - E2 90 ATK 618, base attack time 1.2; trust 100 ATK +35. ATK 653. Physical.
    /// - S1: DP effects only (normal attacks).
    /// - S2 rank 10 (17 SP): "Deals 380% ATK as Physical damage to 6 enemies in front";
    ///   cast every 17 + 1.2 s (upstream SP lockout).
    /// - S3 rank 10: "attacks deal 200% ATK as Physical damage and inflict 30% Fragile to
    ///   targets in a straight line" (`withFragile` ON: x1.3).
    ///
    /// Derivations:
    /// - S0 / S1, DEF 0: 653 / 1.2 = 544.1666... DEF 300: 353 / 1.2 = 294.1666...
    /// - S2, DEF 0: 544.1666 + 653 x 3.8 / 18.2 = 544.1666 + 2481.4 / 18.2 = 680.50732...
    ///   DEF 300: 294.1666 + 2181.4 / 18.2 = 414.02380...
    /// - S3, DEF 0: 653 x 2 x 1.3 / 1.2 = 1414.8333... DEF 500: (1306 - 500) x 1.3 / 1.2
    ///   = 873.1666... 3 targets in the line: 3 x 1414.8333 = 4244.5.
    #[test]
    #[ignore = "needs EN game data"]
    fn silverash_the_reignfrost_snapshot() {
        let gd = en();
        let id = "char_1045_svash2";
        // Snapshot, bit for bit: (skill_dps, total_damage, average_dps), module 0.
        assert_eq!(
            dps(&gd, id, params(0, 0, 1), 0.0, 0.0),
            (544.166645043427, 544.166645043427, 544.166645043427)
        );
        assert_eq!(
            dps(&gd, id, params(1, 0, 1), 0.0, 0.0),
            (544.166645043427, 544.166645043427, 544.166645043427)
        );
        assert_eq!(
            dps(&gd, id, params(2, 0, 1), 0.0, 0.0),
            (680.5073026732366, 680.5073026732366, 680.5073026732366)
        );
        assert_eq!(
            dps(&gd, id, params(3, 0, 1), 0.0, 0.0),
            (1414.833225217135, 67911.99481042247, 949.9141775515627)
        );
        assert_eq!(
            dps(&gd, id, params(3, 0, 1), 500.0, 0.0),
            (873.1665999425809, 41911.99679724388, 474.995111830992)
        );
        let cases = [
            ("S0 DEF 0", plain(&gd, id, 0, 0.0, 0.0), 653.0 / 1.2),
            ("S0 DEF 300", plain(&gd, id, 0, 300.0, 0.0), 353.0 / 1.2),
            ("S1 DEF 0", plain(&gd, id, 1, 0.0, 0.0), 653.0 / 1.2),
            ("S1 DEF 300", plain(&gd, id, 1, 300.0, 0.0), 353.0 / 1.2),
            (
                "S2 DEF 0",
                plain(&gd, id, 2, 0.0, 0.0),
                653.0 / 1.2 + 2481.4 / 18.2,
            ),
            (
                "S2 DEF 300",
                plain(&gd, id, 2, 300.0, 0.0),
                353.0 / 1.2 + 2181.4 / 18.2,
            ),
            ("S3 DEF 0", plain(&gd, id, 3, 0.0, 0.0), 1306.0 * 1.3 / 1.2),
            (
                "S3 DEF 500",
                plain(&gd, id, 3, 500.0, 0.0),
                806.0 * 1.3 / 1.2,
            ),
            (
                "S3 3 targets",
                dps(&gd, id, params(3, 0, 3), 0.0, 0.0).0,
                3.0 * 1306.0 * 1.3 / 1.2,
            ),
        ];
        for (case, got, want) in cases {
            println!("svash2 {case}: {got:?}");
            assert_hand_check(case, got, want);
        }
    }

    /// Closure (`char_4228_closur`).
    ///
    /// Game text, EN tables, checked by hand 2026-10-25:
    /// - E2 90 ATK 480, base attack time 1.0; trust 100 ATK +60. ATK 540. Physical.
    /// - Trait: "ATK is increased to 150% when attacking enemies blocked by
    ///   Reinforcements" (`vsBlocked` ON: x1.5).
    /// - Talent 2 (E2, P1): "[Rhodes Island] Operator ATK +4%"; she is Rhodes Island.
    /// - S1: DP and a Shield only (normal attacks).
    /// - S2 rank 10: "Closure's ATK +80%, attacks 2 targets simultaneously".
    /// - S3 rank 10 (30 s): "Attack Interval decreases" (-0.5 s, so 0.5 s), "dealing 250%
    ///   of Closure's ATK as Physical damage ...; target count +1 after every 9 attacks
    ///   (can trigger up to 6 times)".
    /// - Module TAC-X level 3: ATK +30, talent 2 "[Rhodes Island] Operator ATK +8%".
    ///
    /// Derivations:
    /// - S0 / S1, DEF 0: 540 x 1.04 x 1.5 = 842.4 per 1.0 s. DEF 300: 542.4.
    /// - S2, DEF 0: 540 x 1.84 x 1.5 = 1490.4. DEF 400: 1090.4. 2 targets: 2980.8.
    /// - S3, DEF 0: 561.6 x 1.5 x 2.5 = 2106 per 0.5 s = 4212. DEF 500: 1606 / 0.5 = 3212.
    /// - S3, 7 targets: 30 s x 2 attacks per s = 60 attacks; targets 1..6 for 9 attacks
    ///   each, 7 for the last 6: (9 x 21 + 6 x 7) / 60 = 3.85; 4212 x 3.85 = 16216.2.
    ///   3 targets: (9 x 1 + 9 x 2 + 42 x 3) / 60 = 2.55; 4212 x 2.55 = 10740.6.
    /// - S3, module level 3: 570 x 1.08 x 1.5 x 2.5 / 0.5 = 4617.
    /// - S3, trait OFF: 561.6 x 2.5 / 0.5 = 2808.
    #[test]
    #[ignore = "needs EN game data"]
    fn closure_snapshot() {
        let gd = en();
        let id = "char_4228_closur";
        // Snapshot, bit for bit: (skill_dps, total_damage, average_dps), module 0.
        assert_eq!(
            dps(&gd, id, params(0, 0, 1), 0.0, 0.0),
            (842.3999992758036, 842.3999992758036, 842.3999992758036)
        );
        assert_eq!(
            dps(&gd, id, params(1, 0, 1), 0.0, 0.0),
            (842.3999992758036, 6739.1999942064285, 691.1999996379018)
        );
        assert_eq!(
            dps(&gd, id, params(2, 0, 1), 0.0, 0.0),
            (1490.400008931756, 44712.00026795268, 947.3142895421812)
        );
        assert_eq!(
            dps(&gd, id, params(3, 0, 1), 0.0, 0.0),
            (4211.999996379018, 126359.99989137053, 2375.999998189509)
        );
        assert_eq!(
            dps(&gd, id, params(3, 0, 1), 500.0, 0.0),
            (3211.999996379018, 96359.99989137053, 1625.999998189509)
        );
        let trait_off = OperatorParams {
            conditionals: Some(backend::dps::operator_unit::OperatorConditionals {
                trait_damage: Some(false),
                ..Default::default()
            }),
            ..params(3, 0, 1)
        };
        let cases = [
            ("S0 DEF 0", plain(&gd, id, 0, 0.0, 0.0), 842.4),
            ("S0 DEF 300", plain(&gd, id, 0, 300.0, 0.0), 542.4),
            ("S1 DEF 0", plain(&gd, id, 1, 0.0, 0.0), 842.4),
            ("S1 DEF 300", plain(&gd, id, 1, 300.0, 0.0), 542.4),
            ("S2 DEF 0", plain(&gd, id, 2, 0.0, 0.0), 1490.4),
            ("S2 DEF 400", plain(&gd, id, 2, 400.0, 0.0), 1090.4),
            (
                "S2 2 targets",
                dps(&gd, id, params(2, 0, 2), 0.0, 0.0).0,
                2980.8,
            ),
            ("S3 DEF 0", plain(&gd, id, 3, 0.0, 0.0), 4212.0),
            ("S3 DEF 500", plain(&gd, id, 3, 500.0, 0.0), 3212.0),
            (
                "S3 7 targets",
                dps(&gd, id, params(3, 0, 7), 0.0, 0.0).0,
                16216.2,
            ),
            (
                "S3 3 targets",
                dps(&gd, id, params(3, 0, 3), 0.0, 0.0).0,
                10740.6,
            ),
            (
                "S3 module 3",
                dps(&gd, id, params(3, 1, 1), 0.0, 0.0).0,
                4617.0,
            ),
            ("S3 trait OFF", dps(&gd, id, trait_off, 0.0, 0.0).0, 2808.0),
        ];
        for (case, got, want) in cases {
            println!("closur {case}: {got:?}");
            assert_hand_check(case, got, want);
        }
    }
}

/// Bellone, Zima the Raging Tide, Ch'en the Dawnstreak, Wang (EN game data).
mod demetr_headb2_chen3_wang {
    use super::{Metrics, assert_hand_check, dps, en, params, with};
    use backend::dps::operator_unit::{OperatorConditionals, OperatorParams};

    /// Bellone (`char_4037_demetr`).
    ///
    /// Game text, EN tables, checked by hand 2026-10-25:
    /// - E2 90 ATK 605, base attack time 0.78; trust 100 ATK +60. ATK 665.
    /// - Talent 1 (E2, P1): "Attacks reduce the target's DEF by 7% for 10s (stacks up to
    ///   5 times)"; S2: "stacks up to 8 times"; "Bellone's damage dealt increases, to a
    ///   maximum of 128% when target HP is below 20%" (vsLowHp ON: x1.28). DEF taken as
    ///   max stacks: x0.65, x0.44 in S2.
    /// - S1 rank 10, SP 3 per attack: "The next attack deals 250% ATK as Physical
    ///   damage twice".
    /// - S2 rank 10: "ASPD +80, attacks deal 200% ATK as Physical damage to 3 targets".
    /// - S3 rank 10: "ATK +170%, ASPD +50, attacks have a 50% chance to deal 185% ATK".
    /// - Module `uniequip_002_demetr` level 3: ATK +80, "+10 ASPD when above 50% HP",
    ///   talent "DEF by 8% ... maximum of 142%".
    ///
    /// Derivations (DPS = hit / 0.78 x ASPD / 100 x 1.28):
    /// - S0, DEF 0: 665 x 1.28 / 0.78 = 1091.282051... DEF 500 (325 after 5 stacks):
    ///   340 x 1.28 / 0.78 = 557.948717... vsLowHp OFF, DEF 0: 665 / 0.78 = 852.564102...
    /// - S1, DEF 0: (3 x 665 + 2 x 1662.5) / 4 = 1330; x 1.28 / 0.78 = 2182.564102...
    ///   DEF 500: (3 x 340 + 2 x 1337.5) / 4 = 923.75; x 1.28 / 0.78 = 1515.897435...
    /// - S2, DEF 0: 1330 x 1.8 x 1.28 / 0.78 = 3928.615384... DEF 500 (220 after 8
    ///   stacks): 1110 x 1.8 x 1.28 / 0.78 = 3278.769230...
    /// - S3, DEF 0: ATK 665 x 2.7 = 1795.5; average hit 0.5 x 1795.5 + 0.5 x 3321.675 =
    ///   2558.5875; x 1.5 x 1.28 / 0.78 = 6298.061538... DEF 500: 0.5 x 1470.5 + 0.5 x
    ///   2996.675 = 2233.5875; x 1.5 x 1.28 / 0.78 = 5498.061538...
    /// - S3, module level 3, DEF 0: ATK 745 x 2.7 = 2011.5; average hit 0.5 x 2011.5 +
    ///   0.5 x 3721.275 = 2866.3875; x 1.6 x 1.42 / 0.78 = 8349.272307...
    #[test]
    #[ignore = "needs EN game data"]
    fn bellone_snapshot() {
        let gd = en();
        let id = "char_4037_demetr";
        let off = OperatorConditionals {
            talent_damage: Some(false),
            ..Default::default()
        };

        let cases: [(&str, OperatorParams, f64, f64, Metrics); 10] = [
            (
                "S0 DEF 0",
                params(0, 0, 1),
                0.0,
                1091.282051282051,
                (1091.2820923263757, 1091.2820923263757, 1091.2820923263757),
            ),
            (
                "S0 DEF 500",
                params(0, 0, 1),
                500.0,
                435.2 / 0.78,
                (557.9487401564454, 557.9487401564454, 557.9487401564454),
            ),
            (
                "S0 DEF 0 vsLowHp off",
                with(off, params(0, 0, 1)),
                0.0,
                665.0 / 0.78,
                (852.5641338359687, 852.5641338359687, 852.5641338359687),
            ),
            (
                "S1 DEF 0",
                params(1, 0, 1),
                0.0,
                1702.4 / 0.78,
                (2182.5641846527515, 2182.5641846527515, 2182.5641846527515),
            ),
            (
                "S1 DEF 500",
                params(1, 0, 1),
                500.0,
                1182.4 / 0.78,
                (1515.8974944403383, 1515.8974944403383, 1515.8974944403383),
            ),
            (
                "S2 DEF 0",
                params(2, 0, 1),
                0.0,
                3064.32 / 0.78,
                (3928.6155323749517, 86429.54171224893, 2390.58983310546),
            ),
            (
                "S2 DEF 500",
                params(2, 0, 1),
                500.0,
                2557.44 / 0.78,
                (3278.769357608625, 72132.92586738974, 1745.1539134531354),
            ),
            (
                "S3 DEF 0",
                params(3, 0, 1),
                0.0,
                4912.488 / 0.78,
                (6298.061939253208, 188941.85817759624, 3365.87081325931),
            ),
            (
                "S3 DEF 500",
                params(3, 0, 1),
                500.0,
                4288.488 / 0.78,
                (5498.061910998314, 164941.85732994942, 2651.472365467185),
            ),
            (
                "S3 mod3 DEF 0",
                params(3, 1, 1),
                0.0,
                6512.4324 / 0.78,
                (8349.272754140093, 250478.18262420277, 4367.810323536558),
            ),
        ];
        for (case, p, defense, expected, pinned) in cases {
            let got = dps(&gd, id, p, defense, 0.0);
            assert_eq!(got, pinned, "{case}: snapshot");
            assert_hand_check(case, got.0, expected);
        }
    }

    /// Zima the Raging Tide (`char_1051_headb2`).
    ///
    /// Game text, EN tables, checked by hand 2026-10-25:
    /// - E2 90 ATK 1262, base attack time 1.8; trust 100 ATK +50. ATK 1312.
    /// - Trait: "Attacks deal 50% ATK as AOE Physical damage to enemies around the
    ///   target".
    /// - Talent 1 (E2): "Physical damage caused by Trait splash effect increases by 24%"
    ///   (x1.24 on the splash).
    /// - Talent 2 (E2, P1): "When skill is active, all Operators gain +14% ATK ...
    ///   [Ursus Student Self-Government Group] Operators receive double the effect"; she
    ///   is in it (`TeamId` `student`): +28% during any skill.
    /// - S1 rank 10: "ATK +50%; ASPD +50".
    /// - S2 rank 10: "ATK +90%", second activation onward "double": ATK +180%
    ///   (2ndActivation ON).
    /// - S3 rank 10: "ATK +100%, ... five hammer blows ..., each blow dealing 240% ATK as
    ///   Physical damage and granting an additional +30% ATK" (blow k has +30% x k).
    /// - Module `uniequip_002_headb2` level 3: ATK +83, "ATK of the current instance of
    ///   attack increases to 115%" (3inRange ON), talent splash +40%.
    ///
    /// Derivations (splash target = each of the other targets):
    /// - S0, 1 target, DEF 0: 1312 / 1.8 = 728.888... 3 targets, DEF 300: 1012 + 2 x
    ///   (656 - 300) x 1.24 = 1894.88; / 1.8 = 1052.711...
    /// - S1, 1 target, DEF 0: ATK 1312 x 1.78 = 2335.36; x 1.5 / 1.8 = 1946.133...
    ///   3 targets, DEF 300: 2035.36 + 2 x 867.68 x 1.24 = 4187.2064; x 1.5 / 1.8 =
    ///   3489.338666...
    /// - S2, 1 target, DEF 0: ATK 1312 x 3.08 = 4040.96; / 1.8 = 2244.977... 3 targets,
    ///   DEF 500: 3540.96 + 2 x 1520.48 x 1.24 = 7311.7504; / 1.8 = 4062.083555...
    ///   2ndActivation OFF, 1 target, DEF 0: 1312 x 2.18 / 1.8 = 1588.977...
    /// - S3 (total of five blows), 1 target, DEF 0: 1312 x 2.4 x (2.28 + 2.58 + 2.88 +
    ///   3.18 + 3.48 = 14.4) = 45342.72. 2 targets, DEF 500: blows 45342.72 - 5 x 500 =
    ///   42842.72; splash (22671.36 - 2500) x 1.24 = 25012.4864; 67855.2064.
    /// - S1, module level 3, 3 targets, DEF 300: ATK 1395 x 1.78 x 1.15 = 2855.565;
    ///   2555.565 + 2 x (1427.7825 - 300) x 1.40 = 5713.356; x 1.5 / 1.8 = 4761.13.
    /// - S2 duration (`zima_alter::skill_duration`): "Upon the second activation of this
    ///   skill and afterwards, this skill has unlimited duration": 2ndActivation ON is
    ///   unlimited (total damage and average = skill DPS); OFF keeps the 16 s game-data
    ///   duration and its cycle.
    #[test]
    #[ignore = "needs EN game data"]
    fn zima_the_raging_tide_snapshot() {
        let gd = en();
        let id = "char_1051_headb2";
        let first = OperatorConditionals {
            skill_damage: Some(false),
            ..Default::default()
        };

        let cases: [(&str, OperatorParams, f64, f64, Metrics); 10] = [
            (
                "S0 1t DEF 0",
                params(0, 0, 1),
                0.0,
                1312.0 / 1.8,
                (728.8889081978509, 728.8889081978509, 728.8889081978509),
            ),
            (
                "S0 3t DEF 300",
                params(0, 0, 3),
                300.0,
                1894.88 / 1.8,
                (1052.7111427707446, 1052.7111427707446, 1052.7111427707446),
            ),
            (
                "S1 1t DEF 0",
                params(1, 0, 1),
                0.0,
                2335.36 * 1.5 / 1.8,
                (1946.1333861916169, 58384.001585748505, 1337.511147194734),
            ),
            (
                "S1 3t DEF 300",
                params(1, 0, 3),
                300.0,
                4187.2064 * 1.5 / 1.8,
                (3489.338775813494, 104680.16327440483, 2025.7805064647596),
            ),
            (
                "S2 1t DEF 0",
                params(2, 0, 1),
                0.0,
                4040.96 / 1.8,
                (2244.9778033621524, 2244.9778033621524, 2244.9778033621524),
            ),
            (
                "S2 3t DEF 500",
                params(2, 0, 3),
                500.0,
                7311.7504 / 1.8,
                (4062.083603368219, 4062.083603368219, 4062.083603368219),
            ),
            (
                "S2 1st 1t DEF 0",
                with(first, params(2, 0, 1)),
                0.0,
                2860.16 / 1.8,
                (1588.9778033621524, 25423.644853794438, 937.3953070255604),
            ),
            (
                "S3 1t DEF 0",
                params(3, 0, 1),
                0.0,
                45342.72,
                (45342.72219589235, 45342.72219589235, 45342.72219589235),
            ),
            (
                "S3 2t DEF 500",
                params(3, 0, 2),
                500.0,
                67855.2064,
                (67855.2101497147, 67855.2101497147, 67855.2101497147),
            ),
            (
                "S1 mod3 3t DEF 300",
                params(1, 1, 3),
                300.0,
                5713.356 * 1.5 / 1.8,
                (4761.129966734218, 142833.89900202653, 2684.7316580914407),
            ),
        ];
        for (case, p, defense, expected, pinned) in cases {
            let got = dps(&gd, id, p, defense, 0.0);
            assert_eq!(got, pinned, "{case}: snapshot");
            assert_hand_check(case, got.0, expected);
        }
        let first_again = OperatorConditionals {
            skill_damage: Some(false),
            ..Default::default()
        };
        let first_s2 = dps(&gd, id, with(first_again, params(2, 0, 1)), 0.0, 0.0);
        assert_hand_check("S2 1st total", first_s2.1, first_s2.0 * 16.0);
    }

    /// Ch'en the Dawnstreak (`char_1050_chen3`).
    ///
    /// Game text, EN tables, checked by hand 2026-10-25:
    /// - E2 90 ATK 670, base attack time 1.25; trust 100 ATK +100. ATK 770.
    /// - Talent 1 (E2, P1): "ATK +13%, ASPD +13, and attacks deal Weakness damage"
    ///   (`ba.weaknessatk`: the type changes "depending on the target's DEF and RES";
    ///   taken as the larger of Physical and Arts). ATK 870.1, ASPD 113.
    /// - S1 rank 10: "ATK +120%, attacks hit twice".
    /// - S2 rank 10 (6 s): "Slashes the nearest enemy 10 times, each slash dealing 480%
    ///   ATK as Arts damage ...; afterward, ATK +300%".
    /// - S3 rank 10 (20 s): sword wave "minimum of 580% ATK as Arts damage"; "each attack
    ///   deals 3 instances of 210% ATK as Arts damage to up to 4 ground enemies".
    /// - Module `uniequip_002_chen3` level 3: ATK +60, "ASPD +8 when not blocking an
    ///   enemy" (NotBlocking ON).
    ///
    /// Derivations (x 1.13 / 1.25 = x 0.904):
    /// - S0, DEF 0 RES 0: 870.1 x 0.904 = 786.5704. DEF 300 RES 50: Physical 570.1 beats
    ///   Arts 435.05; 515.3704. DEF 1000 RES 20: Arts 696.08 beats Physical's 5% floor;
    ///   629.25632.
    /// - S1, DEF 0: ATK 770 x 2.33 = 1794.1; 2 x 1794.1 x 0.904 = 3243.7328. DEF 300 RES
    ///   50: 2 x 1494.1 x 0.904 = 2701.3328.
    /// - S2, DEF 0 RES 0: slashes 10 x 870.1 x 4.8 / 6 = 6960.8; attacks ATK 770 x 4.13 =
    ///   3180.1, x 0.904 = 2874.8104; 9835.6104. DEF 300 RES 50: slashes half, 3480.4;
    ///   attacks Physical 2880.1 x 0.904 = 2603.6104; 6084.0104.
    /// - S3, 1 target, DEF 0 RES 0: 3 x 1827.21 x 0.904 = 4955.39352; wave 5046.58 / 20 =
    ///   252.329; 5207.72252. 2 targets, DEF 300 RES 50: Physical 1527.21 per instance,
    ///   3 x 1527.21 x 0.904 x 2 = 8283.58704; wave 2523.29 x 2 / 20 = 252.329;
    ///   8535.91604.
    /// - S0, module level 3, DEF 0 RES 0: ATK 830 x 1.13 = 937.9; x 1.21 / 1.25 =
    ///   907.8872.
    #[test]
    #[ignore = "needs EN game data"]
    fn chen_the_dawnstreak_snapshot() {
        let gd = en();
        let id = "char_1050_chen3";

        let cases: [(&str, OperatorParams, f64, f64, f64, Metrics); 11] = [
            (
                "S0 DEF 0 RES 0",
                params(0, 0, 1),
                0.0,
                0.0,
                786.5704,
                (786.5703966808319, 786.5703966808319, 786.5703966808319),
            ),
            (
                "S0 DEF 300 RES 50",
                params(0, 0, 1),
                300.0,
                50.0,
                515.3704,
                (515.370396680832, 515.370396680832, 515.370396680832),
            ),
            (
                "S0 DEF 1000 RES 20",
                params(0, 0, 1),
                1000.0,
                20.0,
                629.25632,
                (629.2563173446656, 629.2563173446656, 629.2563173446656),
            ),
            (
                "S1 DEF 0 RES 0",
                params(1, 0, 1),
                0.0,
                0.0,
                3243.7328,
                (3243.7328597450255, 58387.19147541046, 1860.7155651423802),
            ),
            (
                "S1 DEF 300 RES 50",
                params(1, 0, 1),
                300.0,
                50.0,
                2701.3328,
                (2701.3328597450254, 48623.991475410454, 1441.683986195012),
            ),
            (
                "S2 DEF 0 RES 0",
                params(2, 0, 1),
                0.0,
                0.0,
                9835.6104,
                (9835.610643905004, 59013.66386343002, 2743.60245628577),
            ),
            (
                "S2 DEF 300 RES 50",
                params(2, 0, 1),
                300.0,
                50.0,
                6084.0104,
                (6084.010520292917, 36504.0631217575, 1640.9255046829808),
            ),
            (
                "S3 1t DEF 0 RES 0",
                params(3, 0, 1),
                0.0,
                0.0,
                5207.72252,
                (5207.722281282789, 104154.44562565578, 2656.765458347906),
            ),
            (
                "S3 2t DEF 300 RES 50",
                params(3, 0, 2),
                300.0,
                50.0,
                8535.91604,
                (8535.915555332433, 170718.31110664865, 3964.8513579255255),
            ),
            (
                "S0 mod3 DEF 0 RES 0",
                params(0, 1, 1),
                0.0,
                0.0,
                907.8872,
                (907.8871961688995, 907.8871961688995, 907.8871961688995),
            ),
            (
                "S1 mod3 DEF 0 RES 0",
                params(1, 1, 1),
                0.0,
                0.0,
                2.0 * 830.0 * 2.33 * 1.21 / 1.25,
                (3744.0304689598083, 67392.54844127655, 2122.961801086225),
            ),
        ];
        for (case, p, defense, res, expected, pinned) in cases {
            let got = dps(&gd, id, p, defense, res);
            assert_eq!(got, pinned, "{case}: snapshot");
            assert_hand_check(case, got.0, expected);
        }
    }

    /// Wang (`char_2027_wang`).
    ///
    /// Game text, EN tables, checked by hand 2026-10-25:
    /// - E2 90 ATK 589, base attack time 0.85, Physical; trust 100 ATK +80. ATK 669.
    /// - Talent 1 (E2, P1): "when manually deploying a stone, Wang will deploy an
    ///   additional stone" (2 stones per stone gained, the model's assumption).
    /// - Talent 2 (E2, P1): "for every stone in the same straight chain, all stones
    ///   within that chain deal an additional 10% damage and ignore 9 RES (stacks up to 3
    ///   times)" (maxTalent2 ON: x1.3, RES -27).
    /// - S1 rank 10 (SP 16): "When a stone is triggered, target enemy ... receives 135% of
    ///   Wang's ATK as Arts damage every second for 6.5s"; "Gain 2 stones".
    /// - S2 rank 10 (SP 15): "When a stone is triggered, deals 580% ATK as Arts damage to
    ///   all enemies within 3 tiles"; "Gain 2 stones".
    /// - S3 rank 10: "dealing 380% ATK as Arts damage"; "grants 8 stones", "20 ammo";
    ///   "Talent 1 deploys up to 3 additional stones at the cost of an equal amount of
    ///   ammo"; "Stops attacking". 28 stone activations. Duration: ceil(20 / 3) = 7
    ///   manual placements, one per stone redeploy time (`token_10064_wang_stone1`
    ///   `RespawnTime` 2 at E2): 14 s. SP cost 50.
    /// - S1 ticks: "every second for 6.5s", read as 6 whole ticks (first tick 1 s after
    ///   the trigger; PRTS https://prts.wiki/w/望 has no tick note, checked 2026-10-25).
    /// - Module `uniequip_002_wang` level 3: ATK +60, talent 2 "13% damage and ignore 12
    ///   RES" (x1.39, RES -36).
    ///
    /// Derivations (attacks 669 / 0.85 = 787.058823...; stones per second 4 / SP cost):
    /// - S0, DEF 0: 787.058823...
    /// - S1, DEF 0 RES 0: stone 669 x 1.35 x 1.3 x 6 = 7044.57; x 4 / 16 = 1761.1425;
    ///   + 787.058823... = 2548.201323... DEF 300 RES 50 (RES 23): 903.15 x 0.77 x 1.3
    ///   x 6 = 5424.3189; x 0.25 = 1356.079725; + 369 / 0.85 = 1790.197372...
    /// - S2, 1 target, DEF 0 RES 0: 3880.2 x 1.3 x 4 / 15 = 1345.136; + 787.058823... =
    ///   2132.194823... 3 targets, DEF 300 RES 50: 3880.2 x 0.77 x 1.3 x 3 x 4 / 15 =
    ///   3107.26416; + 434.117647... = 3541.381807... maxTalent2 OFF, 1 target, RES 50:
    ///   1940.1 x 4 / 15 = 517.36; + 787.058823... = 1304.418823...
    /// - S3, 1 target, RES 0: total 28 x 2542.2 x 1.3 = 92536.08; skill DPS / 14 =
    ///   6609.72; average (92536.08 + 787.058823... x 50) / (14 + 50) = 2060.765955...
    ///   2 targets, RES 50: total 28 x 2542.2 x 0.77 x 1.3 x 2 = 142505.5632; / 14 =
    ///   10178.9688.
    /// - S2, module level 3, 1 target, DEF 0 RES 50 (RES 14): ATK 729; 4228.2 x 0.86 x
    ///   1.39 x 4 / 15 = 1347.837408; + 729 / 0.85 = 2205.484466...
    #[test]
    #[ignore = "needs EN game data"]
    fn wang_snapshot() {
        let gd = en();
        let id = "char_2027_wang";
        let off2 = OperatorConditionals {
            talent2_damage: Some(false),
            ..Default::default()
        };
        let attack = 669.0 / 0.85;

        let cases: [(&str, OperatorParams, f64, f64, f64, Metrics); 10] = [
            (
                "S0 DEF 0 RES 0",
                params(0, 0, 1),
                0.0,
                0.0,
                attack,
                (787.0588014530069, 787.0588014530069, 787.0588014530069),
            ),
            (
                "S0 DEF 300 RES 50",
                params(0, 0, 1),
                300.0,
                50.0,
                369.0 / 0.85,
                (434.1176348821518, 434.1176348821518, 434.1176348821518),
            ),
            (
                "S1 DEF 0 RES 0",
                params(1, 0, 1),
                0.0,
                0.0,
                1761.1425 + attack,
                (2548.201338611995, 2548.201338611995, 2548.201338611995),
            ),
            (
                "S1 DEF 300 RES 50",
                params(1, 0, 1),
                300.0,
                50.0,
                1356.079725 + 369.0 / 0.85,
                (1790.197388494573, 1790.197388494573, 1790.197388494573),
            ),
            (
                "S2 1t DEF 0 RES 0",
                params(2, 0, 1),
                0.0,
                0.0,
                1345.136 + attack,
                (2132.1948503137955, 2132.1948503137955, 2132.1948503137955),
            ),
            (
                "S2 3t DEF 300 RES 50",
                params(2, 0, 3),
                300.0,
                50.0,
                3107.26416 + 369.0 / 0.85,
                (3541.381907750573, 3541.381907750573, 3541.381907750573),
            ),
            (
                "S2 1t RES 50 maxTalent2 off",
                with(off2, params(2, 0, 1)),
                0.0,
                50.0,
                517.36 + attack,
                (1304.4188184665568, 1304.4188184665568, 1304.4188184665568),
            ),
            (
                "S3 1t RES 0",
                params(3, 0, 1),
                0.0,
                0.0,
                92536.08 / 14.0,
                (6609.719939787984, 92536.07915703178, 2060.7659254637833),
            ),
            (
                "S3 2t RES 50",
                params(3, 0, 2),
                0.0,
                50.0,
                142505.5632 / 14.0,
                (10178.968707273496, 142505.56190182894, 2841.539093351239),
            ),
            (
                "S2 mod3 1t RES 50",
                params(2, 1, 1),
                0.0,
                50.0,
                1347.837408 + 729.0 / 0.85,
                (2205.484473220043, 2205.484473220043, 2205.484473220043),
            ),
        ];
        for (case, p, defense, res, expected, pinned) in cases {
            let got = dps(&gd, id, p, defense, res);
            assert_eq!(got, pinned, "{case}: snapshot");
            assert_hand_check(case, got.0, expected);
        }
        let s3 = dps(&gd, id, params(3, 0, 1), 0.0, 0.0);
        assert_hand_check("S3 total", s3.1, 92536.08);
        assert_hand_check("S3 average", s3.2, (92536.08 + attack * 50.0) / 64.0);
    }
}
