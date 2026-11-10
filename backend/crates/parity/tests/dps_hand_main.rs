#![allow(clippy::unreadable_literal)]
#![allow(clippy::doc_markdown, clippy::float_cmp)]

use std::collections::HashMap;
use std::path::Path;

use myrtle_dps_formulas::operator_unit::{EnemyStats, OperatorConditionals, OperatorParams};
use myrtle_gamedata_raw::tables::operator::Operator;

mod engine {
    use myrtle_dps_formulas::HpsResult;
    use myrtle_dps_formulas::operator_unit::{EnemyStats, OperatorParams};
    use myrtle_engine_dps::{DpsRequest, DpsResult};
    use myrtle_gamedata_raw::tables::operator::Operator;

    pub fn calculate_dps(
        op: &Operator,
        p: OperatorParams,
        enemy: &EnemyStats,
    ) -> Option<DpsResult> {
        myrtle_engine_dps::calculate_dps(op, DpsRequest::from(p), enemy)
    }

    pub fn calculate_hps(op: &Operator, p: OperatorParams) -> Option<HpsResult> {
        myrtle_engine_dps::calculate_hps(op, p)
    }
}

pub struct GameData {
    pub operators: HashMap<String, Operator>,
}

fn load(data_dir: &str, _assets_dir: &str) -> GameData {
    GameData {
        operators: myrtle_testkit::real_data::dps_operators(Path::new(data_dir))
            .expect("Failed to load game data"),
    }
}

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

fn en() -> GameData {
    let data = myrtle_testkit::real_data::excel_dir();
    let assets = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output/en".into());
    load(&data.to_string_lossy(), &assets)
}

fn params(skill: i32, module: i32, targets: i32) -> OperatorParams {
    OperatorParams {
        skill_index: Some(skill),
        module_index: Some(module),
        module_level: Some(3),
        targets: Some(targets),
        ..Default::default()
    }
}

fn with(conditionals: OperatorConditionals, p: OperatorParams) -> OperatorParams {
    OperatorParams {
        conditionals: Some(conditionals),
        ..p
    }
}

type Metrics = (f64, f64, f64);

fn dps(gd: &GameData, id: &str, p: OperatorParams, defense: f64, res: f64) -> Metrics {
    let op = gd
        .operators
        .get(id)
        .unwrap_or_else(|| panic!("{id} not in game data"));
    let r = engine::calculate_dps(op, p, &EnemyStats { defense, res })
        .unwrap_or_else(|| panic!("{id}: no result"));
    (r.skill_dps, r.total_damage, r.average_dps)
}

fn hps(gd: &GameData, id: &str, p: OperatorParams) -> Metrics {
    let op = gd
        .operators
        .get(id)
        .unwrap_or_else(|| panic!("{id} not in game data"));
    let r = engine::calculate_hps(op, p).unwrap_or_else(|| panic!("{id}: no HPS"));
    (r.skill_hps, r.base_hps, r.avg_hps)
}

#[test]
#[ignore = "needs CN game data: Angelina the Mellow Wish is not in EN"]
fn angelina_the_mellow_wish_snapshot() {
    let cn = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../assets/output/cn");
    let data =
        std::env::var("DPS_HAND_CN_DATA_DIR").unwrap_or_else(|_| format!("{cn}/gamedata/excel"));
    let assets = std::env::var("DPS_HAND_CN_ASSETS_DIR").unwrap_or_else(|_| cn.into());
    let gd = load(&data, &assets);
    let id = "char_1015_aglna2";

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

mod halo2_nasti_kalts2 {
    use super::engine;
    use super::{assert_hand_check, dps, en, hps, params};
    use myrtle_dps_formulas::operator_unit::{EnemyStats, OperatorConditionals, OperatorParams};

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

mod veen_mantra_svash2_closur {
    use super::GameData;
    use super::{assert_hand_check, dps, en, params};
    use myrtle_dps_formulas::operator_unit::OperatorParams;

    /// `skill_dps` at module 0, one target.
    fn plain(gd: &GameData, id: &str, skill: i32, defense: f64, res: f64) -> f64 {
        dps(gd, id, params(skill, 0, 1), defense, res).0
    }

    #[test]
    #[ignore = "needs EN game data"]
    fn viy_snapshot() {
        let gd = en();
        let id = "char_4226_veen";
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

    #[test]
    #[ignore = "needs EN game data"]
    fn mantra_snapshot() {
        let gd = en();
        let id = "char_4204_mantra";
        let no_interrupts = || OperatorParams {
            conditionals: Some(myrtle_dps_formulas::operator_unit::OperatorConditionals {
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
            conditionals: Some(myrtle_dps_formulas::operator_unit::OperatorConditionals {
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
            conditionals: Some(myrtle_dps_formulas::operator_unit::OperatorConditionals {
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

mod demetr_headb2_chen3_wang {
    use super::{Metrics, assert_hand_check, dps, en, params, with};
    use myrtle_dps_formulas::operator_unit::{OperatorConditionals, OperatorParams};

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
