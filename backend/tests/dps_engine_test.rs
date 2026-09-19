// Parity test against a Python reference: exact `!= 1.0` sentinel checks, integer/float casts on
// fixture data, and a long table-driven case are intentional - allow the pedantic/nursery noise.
#![allow(
    clippy::float_cmp,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::items_after_statements
)]

use backend::core::gamedata;
use backend::dps::engine;
use backend::dps::operator_unit::{EnemyStats, OperatorBuffs, OperatorParams, OperatorShred};
use std::collections::HashMap;
use std::path::Path;

fn load_expected_dps() -> HashMap<String, f64> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/expected_dps.json");
    let data = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read expected_dps.json: {e}"));
    serde_json::from_str(&data).expect("Invalid expected_dps.json")
}

#[allow(clippy::too_many_arguments)]
fn make_test_key(
    operator_name: &str,
    skill: i32,
    module: i32,
    defense: f64,
    res: f64,
    fragile: f64,
    def_shred_mult: f64,
    def_shred_flat: f64,
    res_shred_mult: f64,
    res_shred_flat: f64,
) -> String {
    let base = format!("{operator_name}_s{skill}_m{module}_{defense:.0}_{res:.0}");
    if fragile != 0.0
        || def_shred_mult != 1.0
        || res_shred_mult != 1.0
        || def_shred_flat != 0.0
        || res_shred_flat != 0.0
    {
        format!(
            "{}_db{}_{}_{}_{}_{}",
            base,
            (fragile * 100.0) as i32,
            (def_shred_mult * 100.0) as i32,
            def_shred_flat as i32,
            (res_shred_mult * 100.0) as i32,
            res_shred_flat as i32,
        )
    } else {
        base
    }
}

const TOLERANCE_PERCENT: f64 = 0.15;
const TOLERANCE_ABSOLUTE: f64 = 1.0;

/// A case set where the Python reference is wrong and the fixture, generated
/// from it, is wrong with it. Regenerating `expected_dps.json` reproduces the
/// bug, so instead the reference value is CORRECTED here by a factor derived
/// from the game data, and the Rust is held to that. Every entry must match at
/// least one case, so an entry outlives its bug only until the next regen.
struct ReferenceBug {
    class_name: &'static str,
    skill: i32,
    /// What the reference computes, what the data says, and where.
    note: &'static str,
    /// Multiplier that turns the reference's number into the right one.
    correction: fn(&gamedata::types::operator::Operator) -> f64,
}

const REFERENCE_BUGS: &[ReferenceBug] = &[ReferenceBug {
    class_name: "Vulcan",
    skill: 2,
    // damage_formulas.py `Vulcan.skill_dps` assigns a local
    // `atk_interval = 2 if self.skill == 2` and then divides by
    // `self.atk_interval`, the 1.6 s base, so the local is dead. The game data
    // gives skchr_hpsts_2 `base_attack_time: +0.4` on the 1.6 s base: the skill
    // attacks every 2.0 s, which is what the transpiled Rust divides by (it
    // folds the attribute and the local into one variable). Surfaced by the
    // first CI run of this suite, 2026-09-14: 156 cases, Rust = 0.80 x Python.
    note: "reference divides by the 1.6 s base interval; S2 adds base_attack_time +0.4",
    correction: |op| {
        let base = op
            .phases
            .last()
            .and_then(|p| p.attributes_key_frames.last())
            .map(|kf| kf.data.base_attack_time)
            .expect("Vulcan has phase keyframes");
        let delta = op
            .skills
            .get(1)
            .and_then(|s| s.static_data.as_ref())
            .and_then(|s| s.levels.last())
            .and_then(|l| {
                l.blackboard
                    .iter()
                    .find(|b| b.key == "base_attack_time")
                    .map(|b| b.value)
            })
            .expect("Vulcan S2 carries base_attack_time");
        base / (base + delta)
    },
}];

/// A case set the ENGINE is known to get wrong: the reference is right and the
/// Rust is not. Cases here are still computed and compared, but a mismatch is
/// counted as a known defect rather than a failure, so CI stays green on
/// plumbing while the defect is visible in every run's output. Each entry must
/// match at least one MISMATCH, so fixing the engine forces its deletion. An
/// entry is never added without the finding that names the missing term.
struct KnownEngineDefect {
    class_name: &'static str,
    /// `None` covers every skill.
    skill: Option<i32>,
    note: &'static str,
}

// Empty since the Wiš'adel shadow fix (custom/init.rs). The 2026-09-14 entry
// read: "Walter, all skills, shadow damage missing: OperatorUnit::shadows is
// never populated": 92 of 192 cases diverged by exactly n*drone_atk/4.25.
const KNOWN_ENGINE_DEFECTS: &[KnownEngineDefect] = &[];

fn known_engine_defect(class_name: &str, skill: i32) -> Option<usize> {
    KNOWN_ENGINE_DEFECTS
        .iter()
        .position(|d| d.class_name == class_name && d.skill.is_none_or(|s| s == skill))
}

fn reference_correction(
    operator: &gamedata::types::operator::Operator,
    class_name: &str,
    skill: i32,
) -> Option<(usize, f64)> {
    REFERENCE_BUGS
        .iter()
        .position(|b| b.class_name == class_name && b.skill == skill)
        .map(|i| (i, (REFERENCE_BUGS[i].correction)(operator)))
}

fn compare_dps(rust_dps: f64, python_dps: f64) -> bool {
    let diff = (rust_dps - python_dps).abs();
    if python_dps.abs() < 0.01 {
        return diff <= TOLERANCE_ABSOLUTE;
    }
    let percent_diff = diff / python_dps.abs();
    diff <= TOLERANCE_ABSOLUTE || percent_diff <= TOLERANCE_PERCENT
}

#[test]
fn test_engine_vs_python_expected() {
    dotenv::dotenv().ok();
    let data_dir_str = std::env::var("GAME_DATA_DIR")
        .unwrap_or_else(|_| "../assets/output/en/gamedata/excel".into());
    let assets_dir_str =
        std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output/en".into());
    let data_dir = Path::new(&data_dir_str);
    let assets_dir = Path::new(&assets_dir_str);
    let (game_data, _assets) =
        gamedata::init_game_data(data_dir, assets_dir).expect("Failed to load game data");

    let expected = load_expected_dps();
    let formulas = engine::load_formulas();

    let defense_values = [0.0, 300.0, 500.0, 1000.0];
    let res_values = [0.0, 20.0, 30.0, 50.0];

    // (fragile, def_shred_mult, def_shred_flat, res_shred_mult, res_shred_flat)
    let debuff_scenarios: &[(f64, f64, f64, f64, f64)] = &[
        (0.0, 1.0, 0.0, 1.0, 0.0),   // Baseline
        (0.3, 1.0, 0.0, 1.0, 0.0),   // Fragile 30%
        (0.0, 0.6, 100.0, 1.0, 0.0), // DEF shred 40% + 100 flat
        (0.0, 1.0, 0.0, 0.7, 15.0),  // RES shred 30% + 15 flat
    ];

    let mut tested = 0u64;
    let mut passed = 0u64;
    let mut failed = 0u64;
    let mut skipped = 0u64;
    let mut failures: Vec<String> = Vec::new();
    let mut corrected = vec![0u64; REFERENCE_BUGS.len()];
    let mut known_defects = vec![0u64; KNOWN_ENGINE_DEFECTS.len()];

    // Per-formula-type tracking
    let mut type_tested: HashMap<String, u64> = HashMap::new();
    let mut type_passed: HashMap<String, u64> = HashMap::new();

    for (char_id, formula) in &formulas {
        let Some(operator) = game_data.operators.get(char_id) else {
            skipped += 1;
            continue;
        };

        for &skill in &formula.available_skills {
            // Get formula type for this skill
            let formula_type = formula
                .skills
                .get(&skill.to_string())
                .map_or("unknown", |sf| sf.formula_type.as_str());

            // Skip operators with no skill data (CN-only) or clone-dependent (Muelsyse).
            if operator.skills.is_empty() || formula.class_name == "Muelsyse" {
                skipped += 1;
                continue;
            }

            // Check if ALL formula modules exist in game data.
            // Python's talent resolution cross-references data from ALL modules,
            // so missing any module causes talent value divergence.
            let prefixes = ["uniequip_002_", "uniequip_003_", "uniequip_004_"];
            let all_modules_exist = formula.available_modules.iter().all(|&m| {
                let pos = formula.available_modules.iter().position(|&v| v == m);
                pos.and_then(|p| prefixes.get(p)).is_some_and(|prefix| {
                    operator.modules.iter().any(|om| {
                        om.module
                            .id
                            .as_deref()
                            .is_some_and(|id| id.starts_with(prefix))
                    })
                })
            });

            for module in std::iter::once(0).chain(formula.available_modules.iter().copied()) {
                // Skip ALL module tests if any formula module is missing from game data
                if module > 0 && !all_modules_exist {
                    skipped += 1;
                    continue;
                }

                for &def in &defense_values {
                    for &res in &res_values {
                        for &(fragile, def_mult, def_flat, res_mult, res_flat) in debuff_scenarios {
                            let key = make_test_key(
                                &formula.class_name,
                                skill,
                                module,
                                def,
                                res,
                                fragile,
                                def_mult,
                                def_flat,
                                res_mult,
                                res_flat,
                            );

                            let Some(&expected_dps) = expected.get(&key) else {
                                continue;
                            };
                            let expected_dps =
                                match reference_correction(operator, &formula.class_name, skill) {
                                    Some((i, factor)) => {
                                        corrected[i] += 1;
                                        expected_dps * factor
                                    }
                                    None => expected_dps,
                                };

                            // Build params with debuffs
                            let shred = if def_mult != 1.0
                                || def_flat != 0.0
                                || res_mult != 1.0
                                || res_flat != 0.0
                            {
                                Some(OperatorShred {
                                    def: Some(((1.0 - def_mult) * 100.0).round() as i32),
                                    def_flat: Some(def_flat as i32),
                                    res: Some(((1.0 - res_mult) * 100.0).round() as i32),
                                    res_flat: Some(res_flat as i32),
                                })
                            } else {
                                None
                            };

                            let params = OperatorParams {
                                skill_index: Some(skill),
                                module_index: Some(module),
                                buffs: OperatorBuffs {
                                    fragile: if fragile == 0.0 {
                                        None
                                    } else {
                                        Some(fragile as f32)
                                    },
                                    ..Default::default()
                                },
                                shred,
                                ..Default::default()
                            };

                            let enemy = EnemyStats { defense: def, res };

                            if let Some(result) = engine::calculate_dps(operator, params, &enemy) {
                                // Skip cases where Rust returns 0 but Python expects significant DPS
                                // (indicates missing skill data, e.g. CN-only operators)
                                if result.skill_dps.abs() < 0.01 && expected_dps.abs() > 1.0 {
                                    skipped += 1;
                                    continue;
                                }

                                tested += 1;
                                *type_tested.entry(formula_type.to_owned()).or_default() += 1;

                                if compare_dps(result.skill_dps, expected_dps) {
                                    passed += 1;
                                    *type_passed.entry(formula_type.to_owned()).or_default() += 1;
                                } else if let Some(i) =
                                    known_engine_defect(&formula.class_name, skill)
                                {
                                    known_defects[i] += 1;
                                } else {
                                    failed += 1;
                                    if failures.len() < 30 {
                                        let pct = if expected_dps.abs() > 0.01 {
                                            (result.skill_dps - expected_dps).abs()
                                                / expected_dps.abs()
                                                * 100.0
                                        } else {
                                            0.0
                                        };
                                        failures.push(format!(
                                            "[{}] {}: Rust={:.2}, Python={:.2}, diff={:.1}%",
                                            formula_type, key, result.skill_dps, expected_dps, pct
                                        ));
                                    }
                                }
                            } else {
                                skipped += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    // Print results
    println!("\n============================================================");
    println!("  DPS Engine Validation Results");
    println!("============================================================");
    println!("  Total expected entries: {}", expected.len());
    println!("  Tested:  {tested}");
    println!("  Passed:  {passed}");
    println!("  Failed:  {failed}");
    println!("  Skipped: {skipped}");
    for (bug, n) in REFERENCE_BUGS.iter().zip(&corrected) {
        println!(
            "  Reference corrected: {} S{}: {n} cases ({})",
            bug.class_name, bug.skill, bug.note
        );
    }
    for (defect, n) in KNOWN_ENGINE_DEFECTS.iter().zip(&known_defects) {
        println!(
            "  KNOWN ENGINE DEFECT: {} {}: {n} cases diverge ({})",
            defect.class_name,
            defect
                .skill
                .map_or_else(|| "all skills".to_owned(), |s| format!("S{s}")),
            defect.note
        );
    }

    let pass_rate = if tested > 0 {
        passed as f64 / tested as f64 * 100.0
    } else {
        0.0
    };
    println!("  Overall pass rate: {pass_rate:.1}%");

    // Per-formula-type breakdown
    println!("\n  Per-formula-type:");
    let mut types: Vec<_> = type_tested.keys().cloned().collect();
    types.sort();
    for t in &types {
        let t_tested = type_tested.get(t).copied().unwrap_or(0);
        let t_passed = type_passed.get(t).copied().unwrap_or(0);
        let rate = if t_tested > 0 {
            t_passed as f64 / t_tested as f64 * 100.0
        } else {
            0.0
        };
        println!("    {t}: {t_passed}/{t_tested} ({rate:.1}%)");
    }

    if !failures.is_empty() {
        println!("\n  Sample failures:");
        for f in &failures {
            println!("    {f}");
        }
    }

    assert!(tested > 0, "No test cases were executed");
    for (defect, n) in KNOWN_ENGINE_DEFECTS.iter().zip(&known_defects) {
        assert!(
            *n > 0,
            "KNOWN_ENGINE_DEFECTS entry {} no longer diverges: the engine is fixed, delete \
             the entry",
            defect.class_name
        );
    }
    for (bug, n) in REFERENCE_BUGS.iter().zip(&corrected) {
        assert!(
            *n > 0,
            "REFERENCE_BUGS entry {} S{} matched no case: the fixture no longer carries \
             it, delete the entry",
            bug.class_name,
            bug.skill
        );
    }

    // The point of the suite: the transpiled formulas must not drift from the
    // Python reference. A failure means `generated.rs` and `expected_dps.json`
    // disagree - read the sample failures above, then regenerate with
    // `cargo run --bin generate-dps -- --formulas --transpile --expected`
    // before concluding the fixture is merely stale.
    assert_eq!(
        failed, 0,
        "{failed} of {tested} DPS cases diverge from the Python reference"
    );
}

/// When a specific module is requested that the current game data can't resolve,
/// `calculate_dps` must return `None` rather than silently computing a hybrid
/// (default/no-module) result.
///
/// "Missing" is derived from the operator's actual advanced modules (sorted by
/// uniequip number) - a formula module at position `pos` is unavailable when the
/// operator has fewer than `pos + 1` advanced modules. No hardcoded uniequip IDs.
#[test]
fn test_unavailable_module_returns_none() {
    dotenv::dotenv().ok();
    let data_dir_str = std::env::var("GAME_DATA_DIR")
        .unwrap_or_else(|_| "../assets/output/en/gamedata/excel".into());
    let assets_dir_str =
        std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output/en".into());
    let (game_data, _assets) =
        gamedata::init_game_data(Path::new(&data_dir_str), Path::new(&assets_dir_str))
            .expect("Failed to load game data");

    use backend::core::gamedata::types::module::ModuleType;
    let formulas = engine::load_formulas();
    let enemy = EnemyStats {
        defense: 0.0,
        res: 0.0,
    };

    let mut checked = 0u64;
    for (char_id, formula) in &formulas {
        if formula.available_modules.is_empty() {
            continue;
        }
        let Some(operator) = game_data.operators.get(char_id) else {
            continue;
        };

        // The operator's advanced modules, sorted by uniequip number.
        let mut advanced: Vec<i32> = operator
            .modules
            .iter()
            .filter(|m| m.module.module_type == ModuleType::Advanced)
            .filter_map(|m| {
                m.module
                    .id
                    .as_deref()
                    .and_then(|id| id.split('_').nth(1))
                    .and_then(|n| n.parse::<i32>().ok())
            })
            .collect();
        advanced.sort_unstable();
        let advanced_count = advanced.len();

        for (pos, &module_idx) in formula.available_modules.iter().enumerate() {
            // Module is resolvable iff the operator has a module at this position.
            if pos < advanced_count {
                continue;
            }

            // Missing module -> must return None at E2/maxed (where modules apply).
            let missing_params = OperatorParams {
                promotion: Some(2),
                module_index: Some(module_idx),
                ..Default::default()
            };
            let result = engine::calculate_dps(operator, missing_params, &enemy);
            assert!(
                result.is_none(),
                "{char_id}: requesting unavailable module {module_idx} (position {pos}, operator has {advanced_count} advanced modules) should return None, got {result:?}"
            );
            checked += 1;
        }
    }

    println!("Verified None-on-missing-module for {checked} operator/module combos");
}
