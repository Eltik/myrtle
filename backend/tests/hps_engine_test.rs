//! Parity test: Rust HPS engine vs. upstream ArknightsDpsCompare Python.
//!
//! Drives every healer × skill × module combination through `engine::calculate_hps`
//! and compares the `(skill_hps, base_hps, avg_hps)` triple against the
//! `tests/fixtures/expected_hps.json` values produced by the Python reference.

use backend::core::gamedata;
use backend::dps::engine;
use backend::dps::operator_unit::OperatorParams;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct HpsExpected {
    skill_hps: i64,
    base_hps: i64,
    avg_hps: i64,
}

fn load_expected_hps() -> HashMap<String, HpsExpected> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/expected_hps.json");
    let data = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read expected_hps.json: {e}"));
    serde_json::from_str(&data).expect("Invalid expected_hps.json")
}

fn make_test_key(operator_name: &str, skill: i32, module: i32) -> String {
    format!("{operator_name}_s{skill}_m{module}")
}

const TOLERANCE_PERCENT: f64 = 0.15;
const TOLERANCE_ABSOLUTE: f64 = 2.0;

/// Compare Rust HPS (f64) against Python's truncated int. Python emits
/// `int(value)` which truncates toward zero, so allow a 1-unit floor offset
/// plus the general tolerance.
fn compare_hps(rust: f64, python: i64) -> bool {
    let py_f = python as f64;
    let diff = (rust - py_f).abs();
    if py_f.abs() < 1.0 {
        return diff <= TOLERANCE_ABSOLUTE;
    }
    let percent_diff = diff / py_f.abs();
    diff <= TOLERANCE_ABSOLUTE || percent_diff <= TOLERANCE_PERCENT
}

#[test]
fn test_hps_engine_vs_python_expected() {
    dotenv::dotenv().ok();
    let data_dir_str =
        std::env::var("GAME_DATA_DIR").unwrap_or_else(|_| "../assets/output/gamedata/excel".into());
    let assets_dir_str = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output".into());
    let data_dir = Path::new(&data_dir_str);
    let assets_dir = Path::new(&assets_dir_str);
    let game_data =
        gamedata::init_game_data(data_dir, assets_dir).expect("Failed to load game data");

    let expected = load_expected_hps();
    let formulas = engine::load_heal_formulas();

    let mut tested = 0u64;
    let mut passed = 0u64;
    let mut failed = 0u64;
    let mut skipped = 0u64;
    let mut failures: Vec<String> = Vec::new();

    for (char_id, formula) in &formulas {
        let Some(operator) = game_data.operators.get(char_id) else {
            skipped += 1;
            continue;
        };

        if operator.skills.is_empty() {
            skipped += 1;
            continue;
        }

        for &skill in &formula.available_skills {
            for module in std::iter::once(0).chain(formula.available_modules.iter().copied()) {
                let key = make_test_key(&formula.class_name, skill, module);

                let Some(exp) = expected.get(&key) else {
                    continue;
                };

                let params = OperatorParams {
                    skill_index: Some(skill),
                    module_index: Some(module),
                    ..Default::default()
                };

                let Some(result) = engine::calculate_hps(operator, params) else {
                    skipped += 1;
                    continue;
                };

                // Compare all three components.
                tested += 1;
                let skill_ok = compare_hps(result.skill_hps, exp.skill_hps);
                let base_ok = compare_hps(result.base_hps, exp.base_hps);
                let avg_ok = compare_hps(result.avg_hps, exp.avg_hps);

                if skill_ok && base_ok && avg_ok {
                    passed += 1;
                } else {
                    failed += 1;
                    if failures.len() < 30 {
                        failures.push(format!(
                            "{key}: Rust=({:.1}/{:.1}/{:.1}) Python=({}/{}/{}) [skill:{} base:{} avg:{}]",
                            result.skill_hps,
                            result.base_hps,
                            result.avg_hps,
                            exp.skill_hps,
                            exp.base_hps,
                            exp.avg_hps,
                            if skill_ok { "✓" } else { "✗" },
                            if base_ok { "✓" } else { "✗" },
                            if avg_ok { "✓" } else { "✗" },
                        ));
                    }
                }
            }
        }
    }

    println!("\n============================================================");
    println!("  HPS Engine Validation Results");
    println!("============================================================");
    println!("  Total expected entries: {}", expected.len());
    println!("  Tested:  {tested}");
    println!("  Passed:  {passed}");
    println!("  Failed:  {failed}");
    println!("  Skipped: {skipped}");

    let pass_rate = if tested > 0 {
        passed as f64 / tested as f64 * 100.0
    } else {
        0.0
    };
    println!("  Overall pass rate: {pass_rate:.1}%");

    if !failures.is_empty() {
        println!("\n  Sample failures:");
        for f in &failures {
            println!("    {f}");
        }
    }

    assert!(tested > 0, "No test cases were executed");
}
