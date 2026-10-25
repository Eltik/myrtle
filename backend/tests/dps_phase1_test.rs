//! Phase 1 of the DPS refactor (docs/dps-refactor.md) on real EN game data:
//! zone mappings against the legacy fields they generalise, MAX stacking,
//! the primary / secondary identity, cycle model v2 per SP type, the module
//! level fix, the Muelsyse clone, and the skill-0 support census.
//!
//! The bit-identity of legacy-only requests is NOT checked here: it is the
//! `DPS_DUMP` / `HPS_DUMP` dumps of the two parity suites compared with `cmp`.
#![allow(
    clippy::float_cmp,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use backend::core::gamedata::{self, types::GameData};
use backend::dps::engine::{
    self, DpsResult, OffSkillSource, SkillCategory, SpRecovery, calculate_dps_with,
};
use backend::dps::operator_data::OperatorData;
use backend::dps::operator_unit::{
    CycleModel, EnemyStats, OperatorBaseBuffs, OperatorBuffs, OperatorParams, OperatorShred,
    OperatorUnit, SecondaryTarget,
};
use backend::dps::zones::{BuffZones, StatReduction};

fn game_data() -> &'static GameData {
    static GD: OnceLock<GameData> = OnceLock::new();
    GD.get_or_init(|| {
        dotenv::dotenv().ok();
        let data_dir = std::env::var("GAME_DATA_DIR")
            .unwrap_or_else(|_| "../assets/output/en/gamedata/excel".into());
        let assets_dir =
            std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output/en".into());
        gamedata::init_game_data(Path::new(&data_dir), Path::new(&assets_dir))
            .expect("Failed to load game data")
            .0
    })
}

fn data(id: &str) -> Arc<OperatorData> {
    Arc::new(OperatorData::new(
        game_data()
            .operators
            .get(id)
            .expect("operator in game data")
            .clone(),
    ))
}

/// Every DPS operator the game data has, sorted, with its default skill.
fn operators() -> Vec<(String, i32)> {
    let mut ops: Vec<(String, i32)> = engine::supported_operators()
        .iter()
        .filter(|(id, _)| {
            game_data()
                .operators
                .get(*id)
                .is_some_and(|o| !o.skills.is_empty())
        })
        .map(|(id, f)| (id.clone(), f.default_skill))
        .collect();
    ops.sort();
    ops
}

fn run(id: &str, params: OperatorParams, defense: f64, res: f64) -> Option<DpsResult> {
    calculate_dps_with(data(id), params, &EnemyStats { defense, res })
}

fn bits(r: &DpsResult) -> [u64; 3] {
    [
        r.skill_dps.to_bits(),
        r.total_damage.to_bits(),
        r.average_dps.to_bits(),
    ]
}

fn skill(s: i32) -> OperatorParams {
    OperatorParams {
        skill_index: Some(s),
        ..Default::default()
    }
}

fn with_zones(s: i32, zones: BuffZones) -> OperatorParams {
    OperatorParams {
        zones: Some(zones),
        ..skill(s)
    }
}

/// Asserts a zone request and a legacy request agree bit for bit on every
/// operator (both `None` counts as agreement), and returns how many computed.
fn assert_same_everywhere(
    name: &str,
    zone: impl Fn(i32) -> OperatorParams,
    legacy: impl Fn(i32) -> OperatorParams,
    skip: impl Fn(&OperatorData) -> bool,
) -> usize {
    let mut n = 0;
    for (id, s) in operators() {
        if skip(&data(&id)) {
            continue;
        }
        let a = run(&id, zone(s), 500.0, 30.0);
        let b = run(&id, legacy(s), 500.0, 30.0);
        match (a, b) {
            (Some(a), Some(b)) => {
                assert_eq!(bits(&a), bits(&b), "{name}: {id} S{s}: {a:?} vs {b:?}");
                n += 1;
            }
            (None, None) => {}
            (a, b) => panic!("{name}: {id} S{s}: {a:?} vs {b:?}"),
        }
    }
    println!("{name}: {n} operators bit-identical");
    assert!(n > 200, "{name}: only {n} operators computed");
    n
}

#[test]
fn empty_zones_are_the_legacy_path() {
    assert_same_everywhere(
        "empty zones",
        |s| with_zones(s, BuffZones::default()),
        skill,
        |_| false,
    );
    // Legacy fields alongside an empty zones object still mean what they meant.
    assert_same_everywhere(
        "legacy fields + empty zones",
        |s| OperatorParams {
            buffs: OperatorBuffs {
                atk: Some(0.5),
                flat_atk: Some(120),
                aspd: Some(20),
                fragile: Some(0.25),
            },
            shred: Some(OperatorShred {
                def: Some(40),
                def_flat: Some(100),
                res: Some(30),
                res_flat: Some(15),
            }),
            sp_boost: Some(0.5),
            ..with_zones(s, BuffZones::default())
        },
        |s| OperatorParams {
            buffs: OperatorBuffs {
                atk: Some(0.5),
                flat_atk: Some(120),
                aspd: Some(20),
                fragile: Some(0.25),
            },
            shred: Some(OperatorShred {
                def: Some(40),
                def_flat: Some(100),
                res: Some(30),
                res_flat: Some(15),
            }),
            sp_boost: Some(0.5),
            ..skill(s)
        },
        |_| false,
    );
}

#[test]
fn each_zone_maps_onto_its_unit_field() {
    // A: flat before % is exactly a base flat ATK buff.
    assert_same_everywhere(
        "zone A",
        |s| {
            with_zones(
                s,
                BuffZones {
                    atk_flat: Some(150.0),
                    ..Default::default()
                },
            )
        },
        |s| OperatorParams {
            base_buffs: OperatorBaseBuffs {
                atk: None,
                flat_atk: Some(150),
            },
            ..skill(s)
        },
        |_| false,
    );
    // B: ATK % is `buffs.atk`.
    assert_same_everywhere(
        "zone B",
        |s| {
            with_zones(
                s,
                BuffZones {
                    atk_percent: Some(0.5),
                    ..Default::default()
                },
            )
        },
        |s| OperatorParams {
            buffs: OperatorBuffs {
                atk: Some(0.5),
                ..Default::default()
            },
            ..skill(s)
        },
        |_| false,
    );
    // C: one Inspire source is `buffs.flatAtk`.
    assert_same_everywhere(
        "zone C",
        |s| {
            with_zones(
                s,
                BuffZones {
                    inspire: vec![250.0],
                    ..Default::default()
                },
            )
        },
        |s| OperatorParams {
            buffs: OperatorBuffs {
                flat_atk: Some(250),
                ..Default::default()
            },
            ..skill(s)
        },
        |_| false,
    );
    // D: with no flat buff, a final multiplier is a base ATK multiplier, for
    // every operator without a summon or clone (D also scales those).
    assert_same_everywhere(
        "zone D",
        |s| {
            with_zones(
                s,
                BuffZones {
                    atk_multiplier: vec![2.0],
                    ..Default::default()
                },
            )
        },
        |s| OperatorParams {
            base_buffs: OperatorBaseBuffs {
                atk: Some(2.0),
                flat_atk: None,
            },
            ..skill(s)
        },
        |d| !d.drone_atk.is_empty() || d.data.id.as_deref() == Some("char_249_mlyss"),
    );
    // ASPD and SP recovery are the legacy slots.
    assert_same_everywhere(
        "zone ASPD",
        |s| {
            with_zones(
                s,
                BuffZones {
                    aspd: Some(30.0),
                    ..Default::default()
                },
            )
        },
        |s| OperatorParams {
            buffs: OperatorBuffs {
                aspd: Some(30),
                ..Default::default()
            },
            ..skill(s)
        },
        |_| false,
    );
    assert_same_everywhere(
        "zone SP",
        |s| {
            with_zones(
                s,
                BuffZones {
                    sp_recovery: Some(0.5),
                    ..Default::default()
                },
            )
        },
        |s| OperatorParams {
            sp_boost: Some(0.5),
            ..skill(s)
        },
        |_| false,
    );
    // F: one generic Fragile source is `buffs.fragile`.
    assert_same_everywhere(
        "zone F",
        |s| {
            with_zones(
                s,
                BuffZones {
                    fragile: vec![0.25],
                    ..Default::default()
                },
            )
        },
        |s| OperatorParams {
            buffs: OperatorBuffs {
                fragile: Some(0.25),
                ..Default::default()
            },
            ..skill(s)
        },
        |_| false,
    );
    // DEF / RES: flat + % reduction is the legacy shred.
    assert_same_everywhere(
        "zone DEF/RES",
        |s| {
            with_zones(
                s,
                BuffZones {
                    def: StatReduction {
                        flat: Some(100.0),
                        percent: Some(0.4),
                        ..Default::default()
                    },
                    res: StatReduction {
                        flat: Some(15.0),
                        percent: Some(0.3),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
        },
        |s| OperatorParams {
            shred: Some(OperatorShred {
                def: Some(40),
                def_flat: Some(100),
                res: Some(30),
                res_flat: Some(15),
            }),
            ..skill(s)
        },
        |_| false,
    );
}

#[test]
fn damage_reduction_and_typed_fragility_multiply_after_the_formula() {
    let mut typed = BTreeMap::new();
    for (id, s) in operators() {
        let Some(base) = run(&id, skill(s), 500.0, 30.0) else {
            continue;
        };
        // Muelsyse with no clone divides by a zero clone interval.
        if !base.skill_dps.is_finite() {
            continue;
        }
        let g = run(
            &id,
            with_zones(
                s,
                BuffZones {
                    damage_reduction: vec![0.25],
                    ..Default::default()
                },
            ),
            500.0,
            30.0,
        )
        .unwrap();
        assert!(
            (g.skill_dps - base.skill_dps * 0.75).abs() <= base.skill_dps.abs() * 1e-12,
            "{id}: G"
        );
        let arts = run(
            &id,
            with_zones(
                s,
                BuffZones {
                    arts_fragile: vec![0.5],
                    ..Default::default()
                },
            ),
            500.0,
            30.0,
        )
        .unwrap();
        let flag = arts
            .approximate_type_split
            .expect("flag set when a typed zone is given");
        let ratio = if base.skill_dps == 0.0 {
            1.0
        } else {
            arts.skill_dps / base.skill_dps
        };
        let applied = (ratio - 1.5).abs() < 1e-12;
        *typed
            .entry((if applied { "arts x1.5" } else { "x1.0" }, flag))
            .or_insert(0) += 1;
    }
    println!("artsFragile 0.5 by (applied, approximateTypeSplit): {typed:?}");
}

#[test]
fn max_groups_keep_only_the_largest_source() {
    let cases: Vec<(&str, BuffZones, BuffZones)> = vec![
        (
            "inspire",
            BuffZones {
                inspire: vec![100.0, 300.0, 200.0],
                ..Default::default()
            },
            BuffZones {
                inspire: vec![300.0],
                ..Default::default()
            },
        ),
        (
            "fragile",
            BuffZones {
                fragile: vec![0.25, 0.5],
                ..Default::default()
            },
            BuffZones {
                fragile: vec![0.5],
                ..Default::default()
            },
        ),
        (
            "spAura",
            BuffZones {
                sp_aura: vec![0.25, 0.5],
                ..Default::default()
            },
            BuffZones {
                sp_aura: vec![0.5],
                ..Default::default()
            },
        ),
        (
            "steal",
            BuffZones {
                def: StatReduction {
                    steal: vec![50.0, 80.0],
                    ..Default::default()
                },
                ..Default::default()
            },
            BuffZones {
                def: StatReduction {
                    steal: vec![80.0],
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
    ];
    for (name, many, one) in cases {
        assert_same_everywhere(
            name,
            |s| with_zones(s, many.clone()),
            |s| with_zones(s, one.clone()),
            |_| false,
        );
    }
    // A legacy flatAtk is one more Inspire source: the larger one wins, no sum.
    assert_same_everywhere(
        "legacy flatAtk in the Inspire MAX",
        |s| OperatorParams {
            buffs: OperatorBuffs {
                flat_atk: Some(300),
                ..Default::default()
            },
            ..with_zones(
                s,
                BuffZones {
                    inspire: vec![100.0],
                    ..Default::default()
                },
            )
        },
        |s| OperatorParams {
            buffs: OperatorBuffs {
                flat_atk: Some(300),
                ..Default::default()
            },
            ..skill(s)
        },
        |_| false,
    );
}

#[test]
fn secondary_equal_to_primary_reproduces_f_of_targets() {
    let mut clamped = Vec::new();
    let mut worst = 0.0f64;
    let mut n = 0;
    for (id, s) in operators() {
        for targets in [2, 3, 6] {
            let Some(whole) = run(
                &id,
                OperatorParams {
                    targets: Some(targets),
                    ..skill(s)
                },
                500.0,
                30.0,
            ) else {
                continue;
            };
            if !whole.skill_dps.is_finite() {
                continue;
            }
            let split = run(
                &id,
                OperatorParams {
                    secondary: Some(SecondaryTarget {
                        defense: 500.0,
                        res: 30.0,
                        count: targets - 1,
                    }),
                    ..skill(s)
                },
                500.0,
                30.0,
            )
            .unwrap();
            let single = run(&id, skill(s), 500.0, 30.0).unwrap();
            let primary = split.primary.unwrap();
            assert_eq!(
                primary.skill_dps.to_bits(),
                single.skill_dps.to_bits(),
                "{id}"
            );
            // Since the phase 1 review, a negative secondary share is clamped
            // at 0 and flagged; the identity holds only where nothing clamped.
            if split.secondary_clamped == Some(true) {
                clamped.push(format!("{id} S{s} x{targets}"));
                continue;
            }
            for (a, b) in [
                (split.skill_dps, whole.skill_dps),
                (split.total_damage, whole.total_damage),
                (split.average_dps, whole.average_dps),
            ] {
                let rel = (a - b).abs() / b.abs().max(1.0);
                worst = worst.max(rel);
                assert!(rel < 1e-9, "{id} S{s} x{targets}: {a} vs {b}");
            }
            n += 1;
        }
    }
    println!(
        "secondary identity: {n} cases, worst relative error {worst:e}; clamped {}: {clamped:?}",
        clamped.len()
    );
}

/// The unit the engine builds for these params, for reading the stats the
/// cycle model used.
fn unit(id: &str, s: i32) -> OperatorUnit {
    let f = engine::get_formula(id).unwrap();
    OperatorUnit::new(
        data(id),
        skill(s),
        f.default_skill,
        f.default_potential,
        f.default_module,
        f.available_skills.clone(),
        f.available_modules.clone(),
    )
}

#[test]
fn cycle_v2_per_sp_type() {
    let v2 = |s: i32, sp: f32| OperatorParams {
        cycle_model: CycleModel::V2,
        sp_boost: Some(sp),
        ..skill(s)
    };
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut sources: BTreeMap<String, usize> = BTreeMap::new();
    for (id, f) in engine::supported_operators() {
        if game_data()
            .operators
            .get(id)
            .is_none_or(|o| o.skills.is_empty())
        {
            continue;
        }
        for &s in &f.available_skills {
            let Some(r) = run(id, v2(s, 0.0), 500.0, 30.0) else {
                continue;
            };
            let legacy = run(id, skill(s), 500.0, 30.0).unwrap();
            if !legacy.skill_dps.is_finite() {
                continue;
            }
            // The skill window does not depend on the cycle model.
            assert_eq!(
                r.skill_dps.to_bits(),
                legacy.skill_dps.to_bits(),
                "{id} S{s}"
            );
            assert_eq!(
                r.total_damage.to_bits(),
                legacy.total_damage.to_bits(),
                "{id} S{s}"
            );

            let u = unit(id, s);
            let cost = f64::from(u.skill_cost);
            // The engine's one duration source: an `__init__` override if the
            // class sets one (Viviana S3 25 s), else the game data.
            let duration =
                backend::dps::custom::skill_duration_override(&u).unwrap_or(u.skill_duration);
            let sp = r.sp_recovery.unwrap();
            let category = r.category.unwrap();
            *seen.entry(format!("{sp:?}/{category:?}")).or_default() += 1;
            if let Some(src) = r.off_skill_source {
                *sources.entry(format!("{src:?}")).or_default() += 1;
            }
            match (sp, category) {
                (SpRecovery::Auto, SkillCategory::Burst) if duration > 0.0 && cost > 0.0 => {
                    let expect = duration + cost;
                    assert!((r.cycle_time.unwrap() - expect).abs() < 1e-9, "{id} S{s}");
                    // An SP aura shortens an auto-recovery charge.
                    let boosted = run(id, v2(s, 0.5), 500.0, 30.0).unwrap();
                    let expect = duration + cost / 1.5;
                    assert!(
                        (boosted.cycle_time.unwrap() - expect).abs() < 1e-6,
                        "{id} S{s}"
                    );
                }
                (SpRecovery::Offensive, SkillCategory::Burst) if duration > 0.0 && cost > 0.0 => {
                    let rate = u.attack_speed / 100.0 / f64::from(u.attack_interval);
                    let expect = duration + cost / rate;
                    assert!((r.cycle_time.unwrap() - expect).abs() < 1e-9, "{id} S{s}");
                    // ...and does nothing to an offensive one.
                    let boosted = run(id, v2(s, 0.5), 500.0, 30.0).unwrap();
                    assert_eq!(boosted.cycle_time, r.cycle_time, "{id} S{s}");
                }
                (SpRecovery::Defensive, _) => {
                    // Not modelled: no timings, and the legacy average.
                    assert!(
                        r.cycle_time.is_none() && r.startup_time.is_none(),
                        "{id} S{s}"
                    );
                    assert_eq!(
                        r.average_dps.to_bits(),
                        legacy.average_dps.to_bits(),
                        "{id} S{s}"
                    );
                }
                (_, SkillCategory::Sustained | SkillCategory::Instant) => {
                    assert_eq!(r.average_dps.to_bits(), r.skill_dps.to_bits(), "{id} S{s}");
                }
                _ => {}
            }
            if r.off_skill_source.is_some() {
                // A cycle average lies between its two phases' DPS: at least
                // it never exceeds the larger of skill DPS and itself.
                assert!(r.average_dps.is_finite(), "{id} S{s}");
            }
        }
    }
    println!("cycle v2 (spRecovery/category): {seen:?}");
    println!("cycle v2 off-skill source per cycling skill: {sources:?}");
    for kind in ["Auto/Burst", "Offensive/Burst", "Auto/Instant"] {
        assert!(seen.contains_key(kind), "no {kind} skill in the data");
    }
}

/// How many of the DPS functions (generated and hand, every id in
/// `engine::supported_operators`) have a real skill-0 path, by the probe the v2 cycle
/// uses (`engine::skill_zero_dps`), over every skill that cycles.
#[test]
fn census_skill_zero_support() {
    let mut all_formula = 0;
    let mut mixed = 0;
    let mut none = 0;
    let mut never = 0;
    let mut fallback_ids = Vec::new();
    for (id, f) in engine::supported_operators() {
        if game_data()
            .operators
            .get(id)
            .is_none_or(|o| o.skills.is_empty())
        {
            never += 1;
            continue;
        }
        let mut formula = 0;
        let mut normal = 0;
        for &s in &f.available_skills {
            let params = OperatorParams {
                cycle_model: CycleModel::V2,
                ..skill(s)
            };
            match run(id, params, 500.0, 30.0).and_then(|r| r.off_skill_source) {
                Some(OffSkillSource::Formula) => formula += 1,
                Some(OffSkillSource::NormalAttack) => normal += 1,
                None => {}
            }
        }
        match (formula, normal) {
            (0, 0) => never += 1,
            (_, 0) => all_formula += 1,
            (0, _) => {
                none += 1;
                fallback_ids.push(format!("{} ({id})", f.class_name));
            }
            _ => mixed += 1,
        }
    }
    fallback_ids.sort();
    println!(
        "skill-0 census: {all_formula} own formula on every cycling skill, {mixed} mixed, \
         {none} normal_attack on every cycling skill, {never} with no cycling skill"
    );
    println!("normal_attack fallback: {fallback_ids:?}");
    // Every operator in the table falls in exactly one bucket.
    assert_eq!(
        all_formula + mixed + none + never,
        engine::supported_operators().len()
    );

    // The same probe on every available skill, cycling or not, called
    // directly: the census of the functions themselves.
    let ela = unit("char_4123_ela", 3);
    let (mut every, mut some, mut no) = (0, 0, 0);
    let mut no_ids = Vec::new();
    for (id, f) in engine::supported_operators() {
        if game_data()
            .operators
            .get(id)
            .is_none_or(|o| o.skills.is_empty())
        {
            no += 1;
            no_ids.push(format!("{} (no game data)", f.class_name));
            continue;
        }
        let (mut yes, mut total) = (0, 0);
        for &s in &f.available_skills {
            let mut u = unit(id, s);
            if id == "char_249_mlyss" {
                u.clone_atk = ela.atk;
                u.clone_atk_interval = f64::from(ela.attack_interval);
                u.clone_is_ranged = ela.is_ranged;
                u.clone_is_physical = ela.is_physical;
            }
            let Some(sf) = f.skills.get(&u.skill_index.to_string()) else {
                continue;
            };
            total += 1;
            let enemy = EnemyStats {
                defense: 500.0,
                res: 30.0,
            };
            yes += usize::from(engine::skill_zero_dps(&mut u, sf, &enemy).is_some());
        }
        match (yes, total) {
            (0, 0) => {
                no += 1;
                no_ids.push(format!("{} (no skill)", f.class_name));
            }
            (y, t) if y == t => every += 1,
            (0, _) => {
                no += 1;
                no_ids.push(f.class_name.clone());
            }
            _ => some += 1,
        }
    }
    no_ids.sort();
    println!(
        "skill-0 probe on every skill: {every} pass on all skills, {some} on some, {no} on none"
    );
    println!("skill-0 probe fails everywhere: {no_ids:?}");
    assert_eq!(every + some + no, engine::supported_operators().len());
}

#[test]
fn module_level_one_is_level_one() {
    // Aak, module 1 (uniequip_002): ATK and crit rate both depend on the level.
    let at = |level| {
        run(
            "char_225_haak",
            OperatorParams {
                module_index: Some(1),
                module_level: Some(level),
                ..skill(3)
            },
            0.0,
            0.0,
        )
        .unwrap()
    };
    let (l1, l2, l3) = (at(1), at(2), at(3));
    println!(
        "Aak S3 mod1 skill_dps: level1 {:?}, level2 {:?}, level3 {:?}",
        l1.skill_dps, l2.skill_dps, l3.skill_dps
    );
    assert!(l1.skill_dps < l2.skill_dps && l2.skill_dps < l3.skill_dps);
    // Out-of-range levels still read as 3, as in the reference.
    assert_eq!(at(0).skill_dps.to_bits(), l3.skill_dps.to_bits());
}

/// Muelsyse against the 192 fixture cases, which the reference computes with
/// its default clone, `Ela(pp)`. The parity suite skips her; this prices the
/// clone parameter against the same 15% bar.
#[test]
fn muelsyse_clone_against_the_reference() {
    let expected: std::collections::HashMap<String, f64> = serde_json::from_str(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/expected_dps.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let ela = data("char_4123_ela");
    let scenarios: &[(f64, f64, f64, f64, f64)] = &[
        (0.0, 1.0, 0.0, 1.0, 0.0),
        (0.3, 1.0, 0.0, 1.0, 0.0),
        (0.0, 0.6, 100.0, 1.0, 0.0),
        (0.0, 1.0, 0.0, 0.7, 15.0),
    ];
    let (mut tested, mut pass_clone, mut pass_none) = (0, 0, 0);
    let mut worst = Vec::new();
    for s in [1, 2, 3] {
        for module in [0, 1, 2] {
            for def in [0.0, 300.0, 500.0, 1000.0] {
                for res in [0.0, 20.0, 30.0, 50.0] {
                    for &(fragile, dm, df, rm, rf) in scenarios {
                        let mut key = format!("Muelsyse_s{s}_m{module}_{def:.0}_{res:.0}");
                        if fragile != 0.0 || dm != 1.0 || rm != 1.0 || df != 0.0 || rf != 0.0 {
                            key = format!(
                                "{key}_db{}_{}_{}_{}_{}",
                                (fragile * 100.0) as i32,
                                (dm * 100.0) as i32,
                                df as i32,
                                (rm * 100.0) as i32,
                                rf as i32
                            );
                        }
                        let Some(&want) = expected.get(&key) else {
                            continue;
                        };
                        let params = |clone: Option<Arc<OperatorData>>| OperatorParams {
                            skill_index: Some(s),
                            module_index: Some(module),
                            buffs: OperatorBuffs {
                                fragile: (fragile != 0.0).then_some(fragile as f32),
                                ..Default::default()
                            },
                            shred: (dm != 1.0 || df != 0.0 || rm != 1.0 || rf != 0.0).then(|| {
                                OperatorShred {
                                    def: Some(((1.0 - dm) * 100.0).round() as i32),
                                    def_flat: Some(df as i32),
                                    res: Some(((1.0 - rm) * 100.0).round() as i32),
                                    res_flat: Some(rf as i32),
                                }
                            }),
                            clone_operator: clone,
                            ..Default::default()
                        };
                        let ok = |got: f64| {
                            let d = (got - want).abs();
                            d <= 1.0 || d / want.abs() <= 0.15
                        };
                        let Some(with) = run("char_249_mlyss", params(Some(ela.clone())), def, res)
                        else {
                            continue;
                        };
                        let without = run("char_249_mlyss", params(None), def, res).unwrap();
                        if tested < 3 {
                            println!("{key}: no clone {without:?}");
                        }
                        tested += 1;
                        pass_clone += usize::from(ok(with.skill_dps));
                        pass_none += usize::from(ok(without.skill_dps));
                        if !ok(with.skill_dps) && worst.len() < 8 {
                            worst.push(format!(
                                "{key}: rust {:.2} python {want:.2}",
                                with.skill_dps
                            ));
                        }
                    }
                }
            }
        }
    }
    println!(
        "Muelsyse vs reference (15% bar): clone=Ela {pass_clone}/{tested}, no clone {pass_none}/{tested}"
    );
    println!("Muelsyse clone=Ela misses: {worst:?}");
    assert!(tested > 0);
    assert!(pass_clone >= pass_none);
}

#[test]
fn request_surface_is_additive_and_bounded() {
    use backend::app::services::dps::CalculateRequest;
    let parse = |v: serde_json::Value| serde_json::from_value::<CalculateRequest>(v).unwrap();

    // A legacy body serializes with no new keys, so its cache hash is unchanged.
    let legacy = serde_json::json!({
        "operatorId": "char_225_haak", "skillIndex": 3, "defense": 500.0,
        "buffs": {"atk": 0.5, "flatAtk": 100, "aspd": 10, "fragile": 0.2},
        "shred": {"def": 40, "defFlat": 100, "res": 30, "resFlat": 15},
        "targets": 2, "spBoost": 0.5
    });
    let out = serde_json::to_value(parse(legacy)).unwrap();
    for key in [
        "zones",
        "secondary",
        "cycleModel",
        "frameTiming",
        "cloneOperatorId",
    ] {
        assert!(out.get(key).is_none(), "{key} serialized on a legacy body");
    }

    let full = parse(serde_json::json!({
        "operatorId": "char_225_haak",
        "zones": {
            "atkFlat": 50, "atkPercent": 0.3, "inspire": [200, 150], "atkMultiplier": [1.2],
            "aspd": 20, "spRecovery": 0.3, "spAura": [0.2], "fragile": [0.3],
            "physicalFragile": [0.2], "artsFragile": [0.1], "damageReduction": [0.2],
            "def": {"flat": 100, "percent": 0.3, "ignoreFlat": 50, "ignorePercent": 0.4, "steal": [80]},
            "res": {"percent": 0.2}
        },
        "secondary": {"def": 300, "res": 20, "count": 2},
        "cycleModel": "v2",
        "frameTiming": false,
        "cloneOperatorId": "char_4123_ela"
    }));
    assert!(full.validate().is_ok());

    let rejects = [
        serde_json::json!({"operatorId": "x", "frameTiming": true}),
        serde_json::json!({"operatorId": "x", "targets": 3, "secondary": {"def": 0, "res": 0, "count": 1}}),
        serde_json::json!({"operatorId": "x", "secondary": {"def": 0, "res": 0, "count": 50}}),
        serde_json::json!({"operatorId": "x", "zones": {"atkMultiplier": [-1]}}),
        serde_json::json!({"operatorId": "x", "zones": {"damageReduction": [1.5]}}),
        serde_json::json!({"operatorId": "x", "zones": {"def": {"ignorePercent": 2}}}),
        serde_json::json!({"operatorId": "x", "zones": {"inspire": [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17]}}),
        serde_json::json!({"operatorId": "x", "cloneOperatorId": "../etc"}),
    ];
    for body in rejects {
        assert!(parse(body.clone()).validate().is_err(), "accepted {body}");
    }
    // An unknown cycle model is a deserialization error (422 at the route).
    assert!(
        serde_json::from_value::<CalculateRequest>(
            serde_json::json!({"operatorId": "x", "cycleModel": "v3"})
        )
        .is_err()
    );
}

#[test]
fn cached_operator_data_computes_what_a_fresh_one_does() {
    use backend::dps::data_cache;
    let id = "char_225_haak";
    let mut gd = GameData::new();
    gd.operators
        .insert(id.into(), game_data().operators[id].clone());
    let gd = Arc::new(gd);
    let a = data_cache::operator_data(&gd, id).unwrap();
    let b = data_cache::operator_data(&gd, id).unwrap();
    assert!(Arc::ptr_eq(&a, &b), "second lookup is a hit");
    assert!(data_cache::operator_data(&gd, "char_does_not_exist").is_none());

    // A reload is a new allocation and a new key.
    let mut reload = GameData::new();
    reload
        .operators
        .insert(id.into(), game_data().operators[id].clone());
    let reload = Arc::new(reload);
    let c = data_cache::operator_data(&reload, id).unwrap();
    assert!(!Arc::ptr_eq(&a, &c));

    let enemy = EnemyStats {
        defense: 500.0,
        res: 30.0,
    };
    let cached = calculate_dps_with(a, skill(3), &enemy).unwrap();
    let fresh = calculate_dps_with(data(id), skill(3), &enemy).unwrap();
    assert_eq!(bits(&cached), bits(&fresh));
}
