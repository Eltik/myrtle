//! Generates DPS comparison tests for Rust operators vs Python reference
//!
//! Usage: cargo run --bin generate-dps-tests -- <output_dir>
//!
//! This tool generates:
//! 1. A test configuration JSON file with all test cases
//! 2. Rust integration tests that compare against Python DPS calculations
//! 3. A Python test runner script

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

/// Test case configuration
#[derive(Debug, Clone)]
struct TestCase {
    operator_class: String,
    operator_id: String,
    skill_index: i32,
    module_index: i32,
    defense: f64,
    res: f64,
    expected_min: f64,
    expected_max: f64,
}

/// Operator test configuration
#[derive(Debug, Clone)]
struct OperatorTestConfig {
    class_name: String,
    operator_id: String,
    base_module_index: i32,
    test_cases: Vec<TestCase>,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let output_dir = if args.len() > 1 {
        Path::new(&args[1])
    } else {
        Path::new("tests/dps_comparison")
    };

    println!("Generating DPS comparison tests...");
    println!("Output directory: {}", output_dir.display());

    // Create output directory
    if let Err(e) = fs::create_dir_all(output_dir) {
        eprintln!("Failed to create output directory: {e}");
        std::process::exit(1);
    }

    // Generate test configurations for common operators
    let test_configs = generate_default_test_configs();

    // Generate test configuration JSON
    generate_test_config_json(output_dir, &test_configs);

    // Generate Rust integration test
    generate_rust_integration_test(output_dir, &test_configs);

    // Generate Python comparison script
    generate_python_comparison_script(output_dir, &test_configs);

    println!();
    println!("Generated test files:");
    println!("  - {}/test_config.json", output_dir.display());
    println!("  - {}/dps_comparison_test.rs", output_dir.display());
    println!("  - {}/run_comparison.py", output_dir.display());
    println!();
    println!("To run the comparison tests:");
    println!("  1. cargo test --test dps_comparison");
    println!("  2. python {}/run_comparison.py", output_dir.display());
}

fn generate_default_test_configs() -> Vec<OperatorTestConfig> {
    // Standard test scenarios: (defense, res) pairs
    let scenarios = vec![
        (0.0, 0.0),     // No defense, no res
        (300.0, 0.0),   // Medium defense, no res
        (0.0, 20.0),    // No defense, low res
        (500.0, 30.0),  // High defense, medium res
        (1000.0, 50.0), // Very high defense, high res
    ];

    // Generate test configs for all operators
    // This is a subset - full list would be generated from the Python module
    let operators = vec![
        ("twelveF", "char_009_12fce", -1, vec![0]),
        ("Aak", "char_225_haak", 1, vec![0, 2]),
        ("Absinthe", "char_405_absin", 1, vec![0, 1]),
        ("Aciddrop", "char_366_acdrop", 1, vec![0, 1]),
        ("Adnachiel", "char_211_adnach", -1, vec![0]),
        ("Andreana", "char_218_cuttle", 1, vec![0, 1]),
        ("Angelina", "char_291_aglina", 2, vec![0, 1, 2]),
        ("Aosta", "char_346_aosta", 1, vec![0, 1]),
        ("April", "char_365_aprl", 1, vec![0, 1]),
        ("Archetto", "char_332_archet", 2, vec![0, 1, 2]),
        ("Arene", "char_333_sidero", 1, vec![0, 1]),
        ("Asbestos", "char_378_asbest", 1, vec![0, 1]),
        ("Ascalon", "char_4117_ray", 1, vec![0, 1, 2]),
        ("Ash", "char_423_blemsh", 1, vec![0, 1, 2]),
        ("Ashlock", "char_431_ashlok", 1, vec![0, 1]),
        ("Bagpipe", "char_222_bpipe", 1, vec![0, 1, 2]),
        ("Blaze", "char_017_huang", 1, vec![0, 1]),
        ("BluePoison", "char_129_bluep", 1, vec![0, 1]),
        ("Ceobe", "char_263_skadi", 1, vec![0, 1, 2]),
        ("Chen", "char_010_chen", 1, vec![0, 1, 2]),
        ("Exusiai", "char_103_angel", 2, vec![0, 1, 2]),
        ("Eyjafjalla", "char_180_amgoat", 1, vec![0, 1, 2]),
        ("Ifrit", "char_134_ifrit", 1, vec![0, 1, 2]),
        ("Mlynar", "char_4064_mlynar", 1, vec![0, 1, 2]),
        ("Mountain", "char_264_f12yin", 1, vec![0, 1, 2]),
        ("Mudrock", "char_311_mudrok", 1, vec![0, 1, 2]),
        ("Rosa", "char_197_poca", 1, vec![0, 1, 2]),
        ("Schwarz", "char_340_shwaz", 1, vec![0, 1, 2]),
        ("SilverAsh", "char_172_svrash", 1, vec![0, 1, 2]),
        ("Surtr", "char_350_surtr", 1, vec![0, 1, 2]),
        ("Thorns", "char_293_thorns", 1, vec![0, 1, 2]),
        ("W", "char_113_cqbw", 1, vec![0, 1, 2]),
    ];

    operators
        .into_iter()
        .map(|(class_name, op_id, module_idx, skills)| {
            let mut test_cases = Vec::new();

            for skill in &skills {
                for (defense, res) in &scenarios {
                    test_cases.push(TestCase {
                        operator_class: class_name.to_string(),
                        operator_id: op_id.to_string(),
                        skill_index: *skill,
                        module_index: module_idx,
                        defense: *defense,
                        res: *res,
                        expected_min: 0.0, // Will be filled by Python
                        expected_max: f64::MAX,
                    });
                }
            }

            OperatorTestConfig {
                class_name: class_name.to_string(),
                operator_id: op_id.to_string(),
                base_module_index: module_idx,
                test_cases,
            }
        })
        .collect()
}

fn generate_test_config_json(output_dir: &Path, configs: &[OperatorTestConfig]) {
    let mut json_configs = Vec::new();

    for config in configs {
        for tc in &config.test_cases {
            json_configs.push(serde_json::json!({
                "operator": tc.operator_class,
                "operator_id": tc.operator_id,
                "skill": tc.skill_index,
                "module": tc.module_index,
                "defense": tc.defense,
                "res": tc.res,
            }));
        }
    }

    let json_content = serde_json::to_string_pretty(&json_configs).unwrap();
    let config_path = output_dir.join("test_config.json");

    if let Err(e) = fs::write(&config_path, json_content) {
        eprintln!("Failed to write test config: {e}");
    }
}

fn generate_rust_integration_test(output_dir: &Path, configs: &[OperatorTestConfig]) {
    let mut test_code = String::new();

    test_code.push_str(
        r#"//! DPS Comparison Tests
//!
//! These tests compare Rust DPS calculations against the Python reference implementation.
//!
//! Run with: cargo test --test dps_comparison
//!
//! Note: The Python harness must be set up and ARKDPS_PATH environment variable must be set.

use std::process::Command;
use std::io::Write;

/// Tolerance for DPS comparison (percentage difference allowed)
const TOLERANCE_PERCENT: f64 = 0.01; // 1%

/// Absolute tolerance for very small values
const TOLERANCE_ABSOLUTE: f64 = 1.0;

#[derive(Debug, serde::Deserialize)]
struct PythonDpsResult {
    operator: Option<String>,
    dps: Option<f64>,
    error: Option<String>,
    defense: Option<f64>,
    res: Option<f64>,
}

fn get_python_dps(operator: &str, defense: f64, res: f64, skill: i32, module: i32) -> Result<f64, String> {
    let harness_path = std::env::var("DPS_HARNESS_PATH")
        .unwrap_or_else(|_| "scripts/python_dps_harness.py".to_string());

    let arkdps_path = std::env::var("ARKDPS_PATH")
        .unwrap_or_else(|_| "/Users/eltik/Documents/Coding/ArknightsDpsCompare".to_string());

    let config = serde_json::json!({
        "operator": operator,
        "defense": defense,
        "res": res,
        "skill": skill,
        "module": module,
    });

    let output = Command::new("python3")
        .arg(&harness_path)
        .arg("--batch")
        .env("ARKDPS_PATH", arkdps_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(stdin) = child.stdin.as_mut() {
                writeln!(stdin, "{}", config.to_string())?;
            }
            child.wait_with_output()
        })
        .map_err(|e| format!("Failed to run Python harness: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python harness failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: PythonDpsResult = serde_json::from_str(stdout.trim())
        .map_err(|e| format!("Failed to parse Python output: {} (output: {})", e, stdout))?;

    if let Some(error) = result.error {
        return Err(error);
    }

    result.dps.ok_or_else(|| "No DPS value in response".to_string())
}

fn compare_dps(rust_dps: f64, python_dps: f64, test_name: &str) -> Result<(), String> {
    let diff = (rust_dps - python_dps).abs();
    let percent_diff = if python_dps.abs() > 0.01 {
        diff / python_dps.abs()
    } else {
        diff
    };

    if diff <= TOLERANCE_ABSOLUTE || percent_diff <= TOLERANCE_PERCENT {
        Ok(())
    } else {
        Err(format!(
            "{}: DPS mismatch - Rust: {:.2}, Python: {:.2}, Diff: {:.2} ({:.2}%)",
            test_name, rust_dps, python_dps, diff, percent_diff * 100.0
        ))
    }
}

"#,
    );

    // Generate test module
    test_code.push_str("#[cfg(test)]\nmod tests {\n    use super::*;\n\n");

    for config in configs {
        let test_module_name = to_snake_case(&config.class_name);
        test_code.push_str(&format!("    mod {test_module_name} {{\n"));
        test_code.push_str("        use super::*;\n\n");

        // Group test cases by skill
        let mut by_skill: HashMap<i32, Vec<&TestCase>> = HashMap::new();
        for tc in &config.test_cases {
            by_skill.entry(tc.skill_index).or_default().push(tc);
        }

        for (skill, cases) in &by_skill {
            let skill_name = if *skill < 0 {
                "base".to_string()
            } else {
                format!("s{}", skill + 1)
            };

            for tc in cases {
                let test_name = format!(
                    "test_{}_def{}_res{}",
                    skill_name, tc.defense as i32, tc.res as i32
                );

                test_code.push_str(&format!(
                    r#"        #[test]
        #[ignore] // Enable when Rust implementation is complete
        fn {}() {{
            // TODO: Replace with actual Rust DPS calculation
            let rust_dps = 0.0; // placeholder

            let python_dps = get_python_dps("{}", {:.1}, {:.1}, {}, {})
                .expect("Failed to get Python DPS");

            compare_dps(rust_dps, python_dps, "{} {} def={} res={}")
                .expect("DPS comparison failed");
        }}

"#,
                    test_name,
                    tc.operator_class,
                    tc.defense,
                    tc.res,
                    tc.skill_index,
                    tc.module_index,
                    tc.operator_class,
                    skill_name,
                    tc.defense as i32,
                    tc.res as i32,
                ));
            }
        }

        test_code.push_str("    }\n\n");
    }

    test_code.push_str("}\n");

    let test_path = output_dir.join("dps_comparison_test.rs");
    if let Err(e) = fs::write(&test_path, test_code) {
        eprintln!("Failed to write Rust test file: {e}");
    }
}

fn generate_python_comparison_script(output_dir: &Path, configs: &[OperatorTestConfig]) {
    let mut script = String::new();

    script.push_str(
        r#"#!/usr/bin/env python3
"""
DPS Comparison Test Runner

This script runs DPS calculations for all configured operators and outputs
reference values that can be used in Rust tests.

Usage:
    python run_comparison.py [--output results.json]
"""

import sys
import os
import json
import argparse

# Add paths
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
BACKEND_DIR = os.path.dirname(os.path.dirname(SCRIPT_DIR))
sys.path.insert(0, os.path.join(BACKEND_DIR, 'scripts'))

from python_dps_harness import calculate_dps, list_operators

TEST_CASES = [
"#,
    );

    // Add test cases
    for config in configs {
        for tc in &config.test_cases {
            script.push_str(&format!(
                r#"    {{"operator": "{}", "defense": {:.1}, "res": {:.1}, "skill": {}, "module": {}}},
"#,
                tc.operator_class, tc.defense, tc.res, tc.skill_index, tc.module_index,
            ));
        }
    }

    script.push_str(
        r#"]


def run_all_tests(output_file=None):
    """Run all test cases and collect results."""
    results = []
    errors = []

    print(f"Running {len(TEST_CASES)} test cases...")

    for i, tc in enumerate(TEST_CASES):
        result = calculate_dps(
            operator_name=tc['operator'],
            defense=tc['defense'],
            res=tc['res'],
            skill=tc['skill'],
            module=tc['module'],
        )

        if 'error' in result:
            errors.append(result)
            print(f"  [{i+1}/{len(TEST_CASES)}] {tc['operator']} S{tc['skill']+1} - ERROR: {result['error']}")
        else:
            results.append(result)
            print(f"  [{i+1}/{len(TEST_CASES)}] {tc['operator']} S{tc['skill']+1} def={tc['defense']:.0f} res={tc['res']:.0f} -> DPS: {result['dps']:.2f}")

    print()
    print(f"Completed: {len(results)} successful, {len(errors)} errors")

    if output_file:
        with open(output_file, 'w') as f:
            json.dump({
                'results': results,
                'errors': errors,
            }, f, indent=2)
        print(f"Results saved to {output_file}")

    return results, errors


def generate_rust_expected_values(results):
    """Generate Rust code with expected values from Python results."""
    print("\n// Expected DPS values from Python reference implementation")
    print("// Copy these into your Rust tests\n")

    for r in results:
        skill_name = f"s{r.get('skill', 0) + 1}" if r.get('skill', -1) >= 0 else "base"
        print(f"// {r['operator']} {skill_name} def={r['defense']:.0f} res={r['res']:.0f}")
        print(f"// Expected DPS: {r['dps']:.2f}")
        print()


def main():
    parser = argparse.ArgumentParser(description='Run DPS comparison tests')
    parser.add_argument('--output', '-o', help='Output JSON file for results')
    parser.add_argument('--rust-values', action='store_true', help='Generate Rust expected values')
    args = parser.parse_args()

    results, errors = run_all_tests(args.output)

    if args.rust_values:
        generate_rust_expected_values(results)

    sys.exit(0 if len(errors) == 0 else 1)


if __name__ == '__main__':
    main()
"#,
    );

    let script_path = output_dir.join("run_comparison.py");
    if let Err(e) = fs::write(&script_path, script) {
        eprintln!("Failed to write Python script: {e}");
    }

    // Make the script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(&script_path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(&script_path, perms);
        }
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                let prev = chars[i - 1];
                if !prev.is_uppercase() && prev != '_' {
                    result.push('_');
                }
            }
            result.push(c.to_ascii_lowercase());
        } else if c.is_numeric() {
            if i > 0 && chars[i - 1].is_alphabetic() {
                result.push('_');
            }
            result.push(c);
        } else {
            result.push(c);
        }
    }
    result
}
