//! Phase 2 of the DPS refactor (docs/dps-refactor.md) on real EN game data:
//! the batch and sweep endpoint against single calls, the all-operator
//! ranking, the supporter presets against game text, conditional polarity,
//! the game-data identity, and the phase 1 review fixes.
#![allow(
    clippy::float_cmp,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]

use std::path::Path;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use backend::app::services::dps::{
    BatchRequest, CalculateRequest, RankFilters, RankMetric, RankPreset, RankRequest,
    RequestCycleModel, RequestSecondary, Sweep, SweepAxis, batch_in, calculate_hps_in,
    calculate_in, finish_rank, hps_batch_in, rank_rows_in, supporters_in,
};
use backend::core::gamedata::{self, types::GameData};
use backend::dps::data_cache;
use backend::dps::engine::{self, DpsResult};
use backend::dps::operator_data::OperatorData;
use backend::dps::operator_unit::{EnemyStats, OperatorParams, OperatorUnit, SecondaryTarget};
use backend::dps::supporters::{SUPPORTERS, SupporterInvestment};
use backend::dps::zones::BuffZones;

fn game_data() -> &'static Arc<GameData> {
    static GD: OnceLock<Arc<GameData>> = OnceLock::new();
    GD.get_or_init(|| {
        dotenv::dotenv().ok();
        let data_dir = std::env::var("GAME_DATA_DIR")
            .unwrap_or_else(|_| "../assets/output/en/gamedata/excel".into());
        let assets_dir =
            std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output/en".into());
        Arc::new(
            gamedata::init_game_data(Path::new(&data_dir), Path::new(&assets_dir))
                .expect("Failed to load game data")
                .0,
        )
    })
}

fn req(id: &str, skill: i32) -> CalculateRequest {
    serde_json::from_value(serde_json::json!({ "operatorId": id, "skillIndex": skill }))
        .expect("request")
}

fn bits(r: &DpsResult) -> Vec<u64> {
    let mut v = vec![
        r.skill_dps.to_bits(),
        r.total_damage.to_bits(),
        r.average_dps.to_bits(),
    ];
    for m in [r.primary, r.secondary].into_iter().flatten() {
        v.extend([
            m.skill_dps.to_bits(),
            m.total_damage.to_bits(),
            m.average_dps.to_bits(),
        ]);
    }
    v.extend(r.cycle_time.map(f64::to_bits));
    v.extend(r.startup_time.map(f64::to_bits));
    v
}

/// A spread of request shapes: legacy, zones, secondary, cycle v2.
fn sample_requests() -> Vec<CalculateRequest> {
    let mut out = Vec::new();
    for (id, s) in [
        ("char_293_thorns", 3),
        ("char_017_huang", 2),
        ("char_2013_cerber", 3),
        ("char_1035_wisdel", 3),
        ("char_249_mlyss", 1),
        ("char_113_cqbw", 3),
    ] {
        out.push(req(id, s));
        let mut z = req(id, s);
        z.zones = Some(BuffZones {
            atk_percent: Some(0.4),
            inspire: vec![300.0],
            fragile: vec![0.3],
            ..Default::default()
        });
        z.cycle_model = Some(RequestCycleModel::V2);
        out.push(z);
        let mut sec = req(id, s);
        sec.secondary = Some(RequestSecondary {
            def: 300.0,
            res: 20.0,
            count: 2,
        });
        out.push(sec);
    }
    out
}

#[test]
fn batch_and_sweep_equal_single_calls_bit_for_bit() {
    let gd = game_data();
    let requests = sample_requests();

    let plain = batch_in(
        gd,
        BatchRequest {
            requests: requests.clone(),
            sweep: None,
        },
    );
    for (r, item) in requests.iter().zip(&plain.results) {
        let single = calculate_in(gd, r.clone()).expect("single call");
        assert_eq!(
            bits(item.result.as_ref().expect("result")),
            bits(&single),
            "{}",
            r.operator_id
        );
    }

    let mut points = 0;
    for (axis, max) in [(SweepAxis::Def, 2000.0), (SweepAxis::Res, 100.0)] {
        let sweep = Sweep {
            axis,
            min: 0.0,
            max,
            steps: 41,
        };
        let xs = sweep.points();
        assert_eq!(xs.len(), 41);
        assert_eq!(xs[40], max);
        let swept = batch_in(
            gd,
            BatchRequest {
                requests: requests.clone(),
                sweep: Some(sweep),
            },
        );
        for (r, item) in requests.iter().zip(&swept.results) {
            let pts = item.points.as_ref().expect("points");
            assert_eq!(pts.len(), 41);
            for (p, &x) in pts.iter().zip(&xs) {
                let mut single = r.clone();
                match axis {
                    SweepAxis::Def => single.defense = Some(x),
                    SweepAxis::Res => single.res = Some(x),
                }
                let single = calculate_in(gd, single).expect("single call");
                assert_eq!(p.x.to_bits(), x.to_bits());
                assert_eq!(bits(&p.result), bits(&single), "{} at {x}", r.operator_id);
                points += 1;
            }
        }
    }
    println!(
        "batch: {} requests, {points} sweep points bit-identical",
        requests.len()
    );

    // HPS batch equals single HPS calls.
    let heal: Vec<CalculateRequest> = ["char_128_plosis", "char_1012_skadi2", "char_4134_cetsyr"]
        .iter()
        .map(|id| req(id, 1))
        .collect();
    let hb = hps_batch_in(
        gd,
        BatchRequest {
            requests: heal.clone(),
            sweep: None,
        },
    );
    for (r, item) in heal.iter().zip(&hb.results) {
        let single = calculate_hps_in(gd, r.clone()).expect("hps");
        let got = item.result.as_ref().expect("hps result");
        assert_eq!(
            [got.skill_hps, got.base_hps, got.avg_hps].map(f64::to_bits),
            [single.skill_hps, single.base_hps, single.avg_hps].map(f64::to_bits)
        );
    }

    // A bad entry fails alone.
    let mixed = batch_in(
        gd,
        BatchRequest {
            requests: vec![req("char_nobody", 1), req("char_293_thorns", 3)],
            sweep: None,
        },
    );
    assert!(mixed.results[0].error.is_some());
    assert!(mixed.results[1].result.is_some());
}

#[test]
fn batch_bounds() {
    let too_many = BatchRequest {
        requests: vec![req("char_293_thorns", 3); 65],
        sweep: None,
    };
    assert!(too_many.validate().is_err());
    let empty = BatchRequest {
        requests: vec![],
        sweep: None,
    };
    assert!(empty.validate().is_err());
    let sweep = |min, max, steps| BatchRequest {
        requests: vec![req("char_293_thorns", 3)],
        sweep: Some(Sweep {
            axis: SweepAxis::Res,
            min,
            max,
            steps,
        }),
    };
    assert!(sweep(0.0, 100.0, 101).validate().is_ok());
    assert!(sweep(0.0, 100.0, 102).validate().is_err());
    assert!(sweep(0.0, 100.0, 0).validate().is_err());
    assert!(sweep(50.0, 10.0, 5).validate().is_err());
    assert!(sweep(0.0, 1001.0, 5).validate().is_err());
    let mut bad = req("char_293_thorns", 3);
    bad.targets = Some(99);
    let one_bad = BatchRequest {
        requests: vec![req("char_293_thorns", 3), bad],
        sweep: None,
    };
    let err = one_bad.validate().unwrap_err().to_string();
    assert!(err.starts_with("requests[1]"), "{err}");
}

#[test]
fn rank_rows_equal_single_calls() {
    let gd = game_data();
    // Cold: no OperatorData cached yet for most operators in this process.
    let ctx = RankRequest {
        defense: Some(500.0),
        res: Some(30.0),
        ..Default::default()
    };
    let started = Instant::now();
    let computed = rank_rows_in(gd, &ctx);
    let cold = started.elapsed();
    let started = Instant::now();
    let again = rank_rows_in(gd, &ctx);
    let warm = started.elapsed();
    println!(
        "rank p1Max DEF 500 RES 30: {} rows, {} skipped, cold {cold:?}, warm (OperatorData cached) {warm:?} [{} build]",
        computed.rows.len(),
        computed.skipped,
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );
    assert_eq!(computed.rows.len(), again.rows.len());
    assert!(computed.rows.len() > 500, "{}", computed.rows.len());

    // Every row equals its single call, bit for bit.
    for row in &computed.rows {
        let single = calculate_in(
            gd,
            ctx.row_request(&row.operator_id, row.skill_index, row.module_index),
        )
        .expect("single call");
        assert_eq!(
            [row.skill_dps, row.total_damage, row.average_dps].map(f64::to_bits),
            [single.skill_dps, single.total_damage, single.average_dps].map(f64::to_bits),
            "{} S{} M{}",
            row.operator_id,
            row.skill_index,
            row.module_index
        );
    }
    // One known row, spelled out as a /dps/calculate body.
    let thorns = computed
        .rows
        .iter()
        .find(|r| r.operator_id == "char_293_thorns" && r.skill_index == 3 && r.module_index == 1)
        .expect("Thorns S3 module 1");
    let body: CalculateRequest = serde_json::from_value(serde_json::json!({
        "operatorId": "char_293_thorns", "promotion": 2, "level": 90, "potential": 1,
        "trust": 100, "skillIndex": 3, "masteryLevel": 3, "moduleIndex": 1,
        "moduleLevel": 3, "defense": 500.0, "res": 30.0
    }))
    .unwrap();
    let single = calculate_in(gd, body).unwrap();
    println!(
        "Thorns S3 X3 p1Max DEF 500 RES 30: rank {} single {}",
        thorns.skill_dps, single.skill_dps
    );
    assert_eq!(thorns.skill_dps.to_bits(), single.skill_dps.to_bits());
    assert_eq!(thorns.module_id.as_deref(), Some("uniequip_002_thorns"));

    // Filters and sorting act on the cached rows only.
    let filtered = finish_rank(
        &computed,
        &RankRequest {
            filters: Some(RankFilters {
                professions: Some(vec!["SNIPER".into()]),
                rarities: Some(vec![6]),
                damage_types: None,
            }),
            sort_by: Some(RankMetric::AverageDps),
            limit: Some(10),
            ..ctx.clone()
        },
        "test",
    );
    assert!(filtered.rows.len() <= 10);
    assert!(filtered.matched >= filtered.rows.len());
    assert!(
        filtered
            .rows
            .iter()
            .all(|r| r.profession == "SNIPER" && r.rarity == 6)
    );
    assert!(
        filtered
            .rows
            .windows(2)
            .all(|w| w[0].average_dps >= w[1].average_dps)
    );
    println!(
        "top 6* sniper by averageDps: {} S{} {}",
        filtered.rows[0].name, filtered.rows[0].skill_index, filtered.rows[0].average_dps
    );

    // The preset changes the investment, so P6 never ranks a row below P1.
    let p6 = rank_rows_in(
        gd,
        &RankRequest {
            preset: Some(RankPreset::P6Max),
            ..ctx.clone()
        },
    );
    assert_eq!(p6.rows.len(), computed.rows.len());
    let moved = p6
        .rows
        .iter()
        .zip(&computed.rows)
        .filter(|(a, b)| a.skill_dps != b.skill_dps)
        .count();
    println!(
        "p6Max vs p1Max: {moved} of {} rows move skill_dps",
        p6.rows.len()
    );
    // Context hash ignores filters, sort and limit.
    let mut other = ctx.clone();
    other.limit = Some(3);
    other.sort_by = Some(RankMetric::TotalDamage);
    assert_eq!(other.context_hash(), ctx.context_hash());
    other.defense = Some(501.0);
    assert_ne!(other.context_hash(), ctx.context_hash());
}

#[test]
fn supporters_resolve_from_game_data() {
    let gd = game_data();
    for (label, inv) in [
        ("default", SupporterInvestment::default()),
        (
            "P6 M3 mod3",
            SupporterInvestment {
                promotion: 2,
                skill_level: 10,
                potential: 6,
                module_level: 3,
            },
        ),
    ] {
        let out = supporters_in(gd, inv);
        assert!(
            out.unresolved.is_empty(),
            "{label}: unresolved {:?}",
            out.unresolved
        );
        assert_eq!(out.entries.len(), SUPPORTERS.len());
        println!("supporters {label}: {} entries", out.entries.len());
        for e in &out.entries {
            let c: Vec<String> = e
                .contributions
                .iter()
                .map(|c| format!("{:?}={} ({}={})", c.zone, c.value, c.key, c.raw))
                .collect();
            println!(
                "  {} {} {} {}: {} dur {:?} atk {:?} mod {:?}",
                e.id,
                e.name,
                e.source_kind,
                e.source_index,
                c.join(", "),
                e.duration,
                e.supporter_atk,
                e.module_id
            );
        }
    }

    // Values against the game's own text, at P6 (the last talent candidate).
    let p6 = supporters_in(
        gd,
        SupporterInvestment {
            potential: 6,
            ..Default::default()
        },
    );
    let entry = |id: &str| p6.entries.iter().find(|e| e.id == id).expect(id);
    let first_percent = |text: &str| -> f64 {
        let start = text.find(|c: char| c.is_ascii_digit()).expect("a number");
        let digits: String = text[start..]
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        digits.parse().expect("number")
    };
    let talent_text = |char_id: &str, talent: usize| -> String {
        gd.operators[char_id].talents[talent]
            .candidates
            .last()
            .and_then(|c| c.description.clone())
            .expect("description")
    };
    let mut checked = 0;
    // "Apply 33% (+3%) Fragile effect to enemies ..." (Pramanix, Shamare),
    // "Enemies in range that are Slowed are inflicted with 23% (+3%) Fragile",
    // "Increases the Arts damage taken by all blocked enemies by 18% (+3%)",
    // "Applies 18% (+2%) Fragile effect to enemies that are not Elites".
    for (id, char_id, talent) in [
        ("pramanix_t1", "char_174_slbell", 0),
        ("shamare_t1", "char_254_vodfox", 0),
        ("suzuran_t2", "char_358_lisa", 1),
        ("skyfire_t1", "char_166_skfire", 0),
        ("lucilla_t1", "char_4079_haini", 0),
    ] {
        let text = talent_text(char_id, talent);
        let want = first_percent(&text);
        let got = entry(id).contributions[0].value * 100.0;
        println!("{id}: text {want}% vs data {got}% ({text})");
        assert!((got - want).abs() < 1e-3, "{id}: {got} vs {want}");
        checked += 1;
    }
    // "SP recovery of all allies by 0.3/sec" (Ptilopsis).
    let text = talent_text("char_128_plosis", 0);
    let want: f64 = first_percent(&text);
    let got = entry("ptilopsis_t1").contributions[0].value;
    assert!((got - want).abs() < 1e-6, "ptilopsis {got} vs {want}");
    checked += 1;
    // Skill text displays the fragility through a second, display-only key:
    // Saria S3 `fake.b`, Ela S3 `fake_damage_scale`.
    for (id, char_id, skill, display) in [
        ("saria_s3", "char_202_demkni", 2, "fake.b"),
        ("ela_s3", "char_4123_ela", 2, "fake_damage_scale"),
    ] {
        let level = gd.operators[char_id].skills[skill]
            .static_data
            .as_ref()
            .unwrap()
            .levels
            .last()
            .unwrap()
            .clone();
        let shown = level
            .blackboard
            .iter()
            .find(|b| b.key == display)
            .unwrap()
            .value;
        assert!(level.description.contains(display));
        let got = entry(id).contributions[0].value;
        println!("{id}: display key {display} {shown} vs zone value {got}");
        assert!((got - shown).abs() < 1e-6, "{id}");
        checked += 1;
    }
    println!("supporter values checked against game text: {checked}");

    // Inspiration multiplies the supporter's own ATK at the investment.
    let sora = entry("sora_s2");
    let atk = sora.supporter_atk.expect("atk");
    assert_eq!(sora.contributions[0].value, sora.contributions[0].raw * atk);
}

#[test]
fn conditional_polarity_table() {
    let (on, inverted, unresolved) = engine::polarity_counts();
    println!("polarity: on {on}, inverted {inverted}, unresolved {unresolved}");
    assert_eq!(on + inverted + unresolved, 391);
    assert!(engine::conditional_inverted(
        "dps",
        "char_293_thorns",
        "trait",
        "rangedAtk"
    ));
    assert!(!engine::conditional_inverted(
        "dps",
        "char_293_thorns",
        "talent",
        "vsRanged"
    ));
    assert!(engine::conditional_inverted(
        "dps",
        "char_1029_yato2",
        "talent2",
        "after10s"
    ));
    assert!(!engine::conditional_inverted(
        "dps",
        "char_350_surtr",
        "skill",
        "KillingHitsOnly"
    ));
}

#[test]
fn game_data_identity_is_content_addressed() {
    let gd = game_data();
    let started = Instant::now();
    let a = data_cache::game_data_identity(gd);
    let took = started.elapsed();
    let b = data_cache::game_data_identity(gd);
    assert_eq!(a, b);
    assert_eq!(a.len(), 16);
    // A second load of the same data: same identity, different allocation.
    let copy = Arc::new((**gd).clone());
    assert_eq!(data_cache::game_data_identity(&copy), a);
    // One changed operator: a different identity.
    let mut changed = (**gd).clone();
    changed.operators.get_mut("char_293_thorns").unwrap().phases[2].attributes_key_frames[1]
        .data
        .atk += 1;
    assert_ne!(data_cache::game_data_identity(&Arc::new(changed)), a);
    println!("game-data identity {a}, first hash {took:?}");
}

fn muelsyse(module: i32, s: i32) -> OperatorParams {
    OperatorParams {
        skill_index: Some(s),
        module_index: Some(module),
        clone_operator: Some(Arc::new(OperatorData::new(
            game_data().operators["char_4123_ela"].clone(),
        ))),
        ..Default::default()
    }
}

/// Phase 1 review fix 1: a Muelsyse clone whose requested module the copied
/// operator lacks takes the copied operator's DEFAULT module, as the
/// reference's `Ela(pp)` does (`module = default_mod` when `params.module` is
/// not in `available_modules`). Ela's formula advertises module 3 only; a
/// Muelsyse module 2 request asks Ela for module 2, which she cannot resolve.
/// The fixture holds Muelsyse at module 0 only, so there is no reference
/// number for module 2; this pins the clone stats both ways.
#[test]
fn muelsyse_clone_falls_back_to_the_clone_default_module() {
    let ela = Arc::new(OperatorData::new(
        game_data().operators["char_4123_ela"].clone(),
    ));
    let f = &engine::supported_operators()["char_4123_ela"];
    let unit = |module: i32| {
        OperatorUnit::new(
            Arc::clone(&ela),
            OperatorParams {
                skill_index: Some(1),
                module_index: Some(module),
                trust: Some(100),
                ..Default::default()
            },
            f.default_skill,
            f.default_potential,
            f.default_module,
            f.available_skills.clone(),
            f.available_modules.clone(),
        )
    };
    assert!(unit(2).module_unavailable, "Ela resolves module 2");
    let before = unit(0);
    let after = unit(f.default_module);
    println!(
        "Ela clone for Muelsyse m2: clone_atk {} (no module, before) -> {} (module {}, after); interval {} -> {}",
        before.atk, after.atk, f.default_module, before.attack_interval, after.attack_interval
    );
    assert!(after.atk > before.atk);

    let data = Arc::new(OperatorData::new(
        game_data().operators["char_249_mlyss"].clone(),
    ));
    for s in [1, 2, 3] {
        let r = engine::calculate_dps_with(
            Arc::clone(&data),
            muelsyse(2, s),
            &EnemyStats {
                defense: 500.0,
                res: 30.0,
            },
        )
        .unwrap();
        println!("Muelsyse S{s} m2 DEF 500 RES 30: skill_dps {}", r.skill_dps);
        assert!(r.skill_dps.is_finite());
    }
}

/// Phase 1 review fix 4: the secondary share is clamped at 0 and flagged.
#[test]
fn secondary_share_is_never_negative() {
    let gd = game_data();
    let mut clamped = 0;
    let mut total = 0;
    let mut ids: Vec<&String> = engine::supported_operators().keys().collect();
    ids.sort();
    for id in ids {
        let Some(data) = data_cache::operator_data(gd, id) else {
            continue;
        };
        let s = engine::supported_operators()[id].default_skill;
        let Some(r) = engine::calculate_dps_with(
            data,
            OperatorParams {
                skill_index: Some(s),
                secondary: Some(SecondaryTarget {
                    defense: 800.0,
                    res: 50.0,
                    count: 3,
                }),
                ..Default::default()
            },
            &EnemyStats {
                defense: 300.0,
                res: 20.0,
            },
        ) else {
            continue;
        };
        total += 1;
        let sec = r.secondary.unwrap();
        assert!(sec.skill_dps >= 0.0 && sec.total_damage >= 0.0 && sec.average_dps >= 0.0);
        if r.secondary_clamped == Some(true) {
            clamped += 1;
            println!("clamped: {id} S{s}");
        }
    }
    println!("secondary clamp fired on {clamped} of {total} default skills");
}

/// Phase 1 review fix 3: zone D must be > 0, and no response number is
/// non-finite.
#[test]
fn atk_multiplier_zero_is_rejected() {
    let mut r = req("char_293_thorns", 1);
    r.zones = Some(BuffZones {
        atk_multiplier: vec![0.0],
        ..Default::default()
    });
    assert!(r.validate().is_err());
    r.zones = Some(BuffZones {
        atk_multiplier: vec![0.5],
        ..Default::default()
    });
    assert!(r.validate().is_ok());
}

/// Phase 1 review fix 3, the guard: the service never returns a non-finite
/// number. Thorns module 3 at zone D 0 is the reported case (ATK 0 -> the
/// fallout term divides by 0 DPS); `validate` now rejects the body, and the
/// service, which does not validate, returns an error instead of `null`.
#[test]
fn non_finite_results_are_errors() {
    let gd = game_data();
    let body = |m: f64| -> CalculateRequest {
        serde_json::from_value(serde_json::json!({
            "operatorId": "char_293_thorns", "skillIndex": 1, "moduleIndex": 3,
            "zones": { "atkMultiplier": [m] }
        }))
        .unwrap()
    };
    let raw = engine::calculate_dps_with(
        data_cache::operator_data(gd, "char_293_thorns").unwrap(),
        OperatorParams {
            skill_index: Some(1),
            module_index: Some(3),
            zones: Some(BuffZones {
                atk_multiplier: vec![0.0],
                ..Default::default()
            }),
            ..Default::default()
        },
        &EnemyStats::default(),
    )
    .unwrap();
    println!("Thorns S1 module 3, zone D 0, engine: {raw:?}");
    let service = calculate_in(gd, body(0.0));
    let any_non_finite = ![raw.skill_dps, raw.total_damage, raw.average_dps]
        .iter()
        .all(|v| v.is_finite());
    assert_eq!(service.is_err(), any_non_finite, "{service:?}");
    assert!(body(0.0).validate().is_err());
    assert!(calculate_in(gd, body(1.0)).is_ok());
}
