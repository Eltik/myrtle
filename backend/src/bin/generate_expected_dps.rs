use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Serialize)]
struct TestCase {
    key: String,
    operator: String,
    skill: i32,
    module: i32,
    defense: f64,
    res: f64,
    fragile: f64,
    def_shred_mult: f64,
    def_shred_flat: f64,
    res_shred_mult: f64,
    res_shred_flat: f64,
}

#[derive(Deserialize)]
struct OperatorFormula {
    class_name: String,
    available_skills: Vec<i32>,
    available_modules: Vec<i32>,
}

fn make_test_key(
    name: &str,
    skill: i32,
    module: i32,
    def: f64,
    res: f64,
    fragile: f64,
    def_mult: f64,
    def_flat: f64,
    res_mult: f64,
    res_flat: f64,
) -> String {
    let base = format!("{name}_s{skill}_m{module}_{:.0}_{:.0}", def, res);
    if fragile != 0.0 || def_mult != 1.0 || res_mult != 1.0 || def_flat != 0.0 || res_flat != 0.0 {
        format!(
            "{base}_db{}_{}_{}_{}_{}",
            (fragile * 100.0) as i32,
            (def_mult * 100.0) as i32,
            def_flat as i32,
            (res_mult * 100.0) as i32,
            res_flat as i32,
        )
    } else {
        base
    }
}

fn main() {
    // Load operator formulas
    let formulas_str = std::fs::read_to_string("src/dps/config/operator_formulas.json")
        .expect("Failed to read operator_formulas.json");
    let formulas: HashMap<String, OperatorFormula> =
        serde_json::from_str(&formulas_str).expect("Invalid operator_formulas.json");

    // Test parameters
    let defenses = [0.0, 300.0, 500.0, 1000.0];
    let resistances = [0.0, 20.0, 30.0, 50.0];
    let debuff_scenarios: Vec<(f64, f64, f64, f64, f64)> = vec![
        (0.0, 1.0, 0.0, 1.0, 0.0),   // Baseline
        (0.3, 1.0, 0.0, 1.0, 0.0),   // Fragile 30%
        (0.0, 0.6, 100.0, 1.0, 0.0), // DEF shred 40% + 100 flat
        (0.0, 1.0, 0.0, 0.7, 15.0),  // RES shred 30% + 15 flat
    ];

    // Generate test cases
    let mut cases: Vec<TestCase> = Vec::new();
    let mut seen_keys = std::collections::HashSet::new();

    for formula in formulas.values() {
        let name = &formula.class_name;

        for &skill in &formula.available_skills {
            let modules: Vec<i32> = std::iter::once(0)
                .chain(formula.available_modules.iter().copied())
                .collect();

            for &module in &modules {
                for &def in &defenses {
                    for &res in &resistances {
                        for &(fragile, def_mult, def_flat, res_mult, res_flat) in &debuff_scenarios
                        {
                            let key = make_test_key(
                                name, skill, module, def, res, fragile, def_mult, def_flat,
                                res_mult, res_flat,
                            );

                            if seen_keys.insert(key.clone()) {
                                cases.push(TestCase {
                                    key,
                                    operator: name.clone(),
                                    skill,
                                    module,
                                    defense: def,
                                    res,
                                    fragile,
                                    def_shred_mult: def_mult,
                                    def_shred_flat: def_flat,
                                    res_shred_mult: res_mult,
                                    res_shred_flat: res_flat,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    println!("Generated {} test cases", cases.len());

    // Call Python in batch mode (one process, all cases)
    let script_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "scripts/dps_calc.py".to_string());

    println!("Running Python DPS calculator...");
    let cases_json = serde_json::to_string(&cases).unwrap();

    let mut child = Command::new("python3")
        .arg(&script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start Python. Is python3 installed?");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(cases_json.as_bytes())
        .expect("Failed to write to Python stdin");

    let output = child
        .wait_with_output()
        .expect("Failed to read Python output");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Python failed:\n{stderr}");
        std::process::exit(1);
    }

    let results: HashMap<String, f64> =
        serde_json::from_slice(&output.stdout).expect("Failed to parse Python output");

    println!("Received {} DPS values from Python", results.len());
    println!(
        "Success rate: {:.1}%",
        results.len() as f64 / cases.len() as f64 * 100.0
    );

    // Write expected_dps.json
    let output_path = std::path::Path::new("tests/fixtures/expected_dps.json");
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let json = serde_json::to_string_pretty(&results).unwrap();
    std::fs::write(output_path, &json).unwrap();
    println!("Written to {}", output_path.display());

    // Print some stats
    if !results.is_empty() {
        let values: Vec<f64> = results.values().copied().collect();
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        println!("\nDPS range: {min:.2} - {max:.2} (mean: {mean:.2})");
    }
}
