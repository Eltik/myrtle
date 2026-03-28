//! All-in-one DPS generator for the Arknights DPS calculator.
//!
//! Reads Python source from ArknightsDpsCompare and generates:
//! - `src/dps/config/operator_formulas.json` — operator metadata (skills, modules, conditionals)
//! - `src/dps/custom/generated.rs` + `mod.rs` — transpiled Rust DPS functions
//! - `tests/fixtures/expected_dps.json` — Python-computed reference DPS values for testing
//!
//! Usage:
//!   cargo run --bin generate-dps                     # Run all steps
//!   cargo run --bin generate-dps -- --formulas       # Only regenerate operator_formulas.json
//!   cargo run --bin generate-dps -- --transpile      # Only transpile Python → Rust
//!   cargo run --bin generate-dps -- --expected       # Only regenerate expected_dps.json
//!   cargo run --bin generate-dps -- --repo <path>    # Custom ArknightsDpsCompare path

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

// ── Shared types ────────────────────────────────────────────────────────

#[derive(Deserialize, Serialize, Clone)]
struct OperatorFormula {
    name: String,
    class_name: String,
    available_skills: Vec<i32>,
    available_modules: Vec<i32>,
    default_skill: i32,
    default_potential: i32,
    default_module: i32,
    skills: HashMap<String, SkillFormula>,
    conditionals: Vec<Conditional>,
}

#[derive(Deserialize, Serialize, Clone)]
struct SkillFormula {
    #[serde(rename = "type")]
    formula_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Conditional {
    #[serde(rename = "type")]
    cond_type: String,
    name: String,
    default: bool,
    skills: Vec<i32>,
    modules: Vec<i32>,
}

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

fn main() {
    dotenv::dotenv().ok();
    let args: Vec<String> = std::env::args().collect();

    let repo_path = if let Some(i) = args.iter().position(|a| a == "--repo") {
        args.get(i + 1)
            .cloned()
            .unwrap_or_else(|| "external/ArknightsDpsCompare".to_string())
    } else {
        // Legacy: first non-flag arg is repo path
        args.iter()
            .skip(1)
            .find(|a| !a.starts_with("--"))
            .cloned()
            .unwrap_or_else(|| "external/ArknightsDpsCompare".to_string())
    };

    let run_formulas = args.contains(&"--formulas".to_string());
    let run_transpile = args.contains(&"--transpile".to_string());
    let run_expected = args.contains(&"--expected".to_string());
    let run_all = !run_formulas && !run_transpile && !run_expected;

    // Load Python source
    let py_src = fs::read_to_string(format!("{repo_path}/damagecalc/damage_formulas.py"))
        .expect("Failed to read damage_formulas.py");

    // Step 1: Generate operator_formulas.json (must run before transpile)
    if run_all || run_formulas {
        generate_formulas_json(&repo_path, &py_src);
    }

    // Load operator_formulas.json (freshly generated or existing)
    let formulas_str = fs::read_to_string("src/dps/config/operator_formulas.json")
        .expect("Failed to read operator_formulas.json");
    let formulas: HashMap<String, OperatorFormula> =
        serde_json::from_str(&formulas_str).expect("Invalid operator_formulas.json");

    // Step 2: Transpile Python → Rust
    if run_all || run_transpile {
        transpile_all(&py_src, &formulas);
    }

    // Step 3: Generate expected_dps.json via Python subprocess
    if run_all || run_expected {
        generate_expected_dps(&repo_path, &formulas);
    }
}

// ── Step 1: Generate operator_formulas.json ─────────────────────────────

fn generate_formulas_json(repo_path: &str, py_src: &str) {
    println!("=== Generating operator_formulas.json ===");

    let id_dict = parse_id_dict(repo_path);
    println!("Parsed {} char_id mappings", id_dict.len());

    let operators = parse_operators(py_src, &id_dict);
    println!("Parsed {} operators", operators.len());

    let json = serde_json::to_string_pretty(&operators).unwrap();
    fs::write("src/dps/config/operator_formulas.json", &json).unwrap();
    println!("Written to src/dps/config/operator_formulas.json");
}

fn parse_id_dict(repo_path: &str) -> HashMap<String, String> {
    let src = fs::read_to_string(format!("{repo_path}/Database/JsonReader.py"))
        .expect("Failed to read JsonReader.py");
    let re = Regex::new(r"'(\w+)'\s*:\s*'(char_\d+_\w+)'").unwrap();
    re.captures_iter(&src)
        .map(|cap| (cap[1].to_string(), cap[2].to_string()))
        .collect()
}

fn parse_operators(
    src: &str,
    id_dict: &HashMap<String, String>,
) -> HashMap<String, OperatorFormula> {
    let class_re = Regex::new(r"class\s+(\w+)\s*\(\s*Operator\s*\)\s*:").unwrap();
    let class_positions: Vec<(usize, String)> = class_re
        .captures_iter(src)
        .map(|cap| (cap.get(0).unwrap().start(), cap[1].to_string()))
        .collect();

    let mut result = HashMap::new();
    for (i, (start, class_name)) in class_positions.iter().enumerate() {
        let end = class_positions
            .get(i + 1)
            .map(|(pos, _)| *pos)
            .unwrap_or(src.len());
        let class_body = &src[*start..end];

        let init_body = extract_init_method(class_body);
        let (name, skills, modules, def_skill, def_pot, def_mod) = parse_super_init(&init_body);
        // Look up by display name first (handles cases like Hibiscus → HibiscusAlter),
        // then fall back to class name
        let char_id = id_dict
            .get(&name)
            .or_else(|| id_dict.get(class_name))
            .cloned()
            .unwrap_or_else(|| format!("unknown_{}", class_name.to_lowercase()));
        let conditionals = extract_conditionals(&init_body);

        let mut skill_formulas = HashMap::new();
        for &skill_idx in &skills {
            skill_formulas.insert(
                skill_idx.to_string(),
                SkillFormula {
                    formula_type: "custom".to_string(),
                },
            );
        }

        result.insert(
            char_id,
            OperatorFormula {
                name,
                class_name: class_name.clone(),
                available_skills: skills,
                available_modules: modules,
                default_skill: def_skill,
                default_potential: def_pot,
                default_module: def_mod,
                skills: skill_formulas,
                conditionals,
            },
        );
    }
    result
}

fn parse_super_init(init_body: &str) -> (String, Vec<i32>, Vec<i32>, i32, i32, i32) {
    let re = Regex::new(
        r#"super\(\)\.__init__\(\s*"([^"]+)"\s*,\s*pp\s*,\s*\[([^\]]*)\]\s*,\s*\[([^\]]*)\](?:\s*,\s*(\d+))?(?:\s*,\s*(\d+))?(?:\s*,\s*(\d+))?"#
    ).unwrap();
    if let Some(cap) = re.captures(init_body) {
        let name = cap[1].to_string();
        let skills: Vec<i32> = cap[2]
            .split(',')
            .filter_map(|v| v.trim().parse().ok())
            .collect();
        let modules: Vec<i32> = cap[3]
            .split(',')
            .filter_map(|v| v.trim().parse().ok())
            .collect();
        let def_skill = cap
            .get(4)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(3);
        let def_pot = cap
            .get(5)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(1);
        let def_mod = cap
            .get(6)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(1);
        (name, skills, modules, def_skill, def_pot, def_mod)
    } else {
        ("Unknown".to_string(), vec![], vec![], 3, 1, 0)
    }
}

fn extract_conditionals(init_body: &str) -> Vec<Conditional> {
    let mut conditionals = Vec::new();
    let cond_types = [
        ("trait_dmg", "trait"),
        ("talent_dmg", "talent"),
        ("talent2_dmg", "talent2"),
        ("skill_dmg", "skill"),
        ("module_dmg", "module"),
    ];
    let label_re = Regex::new(r#"self\.name\s*\+=\s*[f"]?\s*"?\s*(\w+)"#).unwrap();
    for line in init_body.lines() {
        for &(py_field, cond_type) in &cond_types {
            if line.contains(py_field)
                && let Some(cap) = label_re.captures(line)
            {
                conditionals.push(Conditional {
                    cond_type: cond_type.to_string(),
                    name: cap[1].to_string(),
                    default: false,
                    skills: vec![],
                    modules: vec![],
                });
            }
        }
    }
    conditionals.sort_by(|a, b| a.name.cmp(&b.name));
    conditionals.dedup_by(|a, b| a.name == b.name);
    conditionals
}

fn extract_init_method(class_body: &str) -> String {
    let pattern = "def __init__(";
    let mut body = String::new();
    let mut found = false;
    let mut base_indent = 0;
    for line in class_body.lines() {
        if line.trim_start().starts_with(pattern) {
            found = true;
            base_indent = line.len() - line.trim_start().len();
            continue;
        }
        if found {
            let trimmed = line.trim_start();
            if trimmed.is_empty() {
                body.push('\n');
                continue;
            }
            let indent = line.len() - trimmed.len();
            if indent <= base_indent && !trimmed.is_empty() {
                break;
            }
            body.push_str(trimmed);
            body.push('\n');
        }
    }
    body
}

// ── Step 3: Generate expected_dps.json ──────────────────────────────────

/// Load all module IDs from battle_equip_table.json for existence checks.
fn load_equip_keys(path: &str) -> std::collections::HashSet<String> {
    let raw = fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
    let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap_or_default();
    let mut keys = std::collections::HashSet::new();
    if let Some(equips) = parsed.get("Equips").and_then(|v| v.as_array()) {
        for entry in equips {
            if let Some(key) = entry.get("key").and_then(|k| k.as_str()) {
                keys.insert(key.to_string());
            }
        }
    }
    keys
}

/// Check if a specific module exists in game data for the given operator.
fn module_exists_in_game_data(
    equip_keys: &std::collections::HashSet<String>,
    char_id: &str,
    formula_available_modules: &[i32],
    module_value: i32,
) -> bool {
    if module_value == 0 {
        return true;
    }
    let op_id_suffix = char_id.split('_').nth(2).unwrap_or("");
    let prefixes = ["uniequip_002_", "uniequip_003_", "uniequip_004_"];
    let pos = formula_available_modules
        .iter()
        .position(|&m| m == module_value);
    pos.and_then(|p| prefixes.get(p))
        .is_some_and(|prefix| equip_keys.contains(&format!("{prefix}{op_id_suffix}")))
}

fn generate_expected_dps(repo_path: &str, formulas: &HashMap<String, OperatorFormula>) {
    println!("\n=== Generating expected_dps.json ===");

    // Load existing module IDs from game data to filter out non-existent modules
    let game_data_dir =
        std::env::var("GAME_DATA_DIR").unwrap_or_else(|_| "../assets/output/gamedata/excel".into());
    let equip_keys = load_equip_keys(&format!("{game_data_dir}/battle_equip_table.json"));
    println!("Loaded {} module IDs from game data", equip_keys.len());

    let defenses = [0.0, 300.0, 500.0, 1000.0];
    let resistances = [0.0, 20.0, 30.0, 50.0];
    let debuffs: Vec<(f64, f64, f64, f64, f64)> = vec![
        (0.0, 1.0, 0.0, 1.0, 0.0),
        (0.3, 1.0, 0.0, 1.0, 0.0),
        (0.0, 0.6, 100.0, 1.0, 0.0),
        (0.0, 1.0, 0.0, 0.7, 15.0),
    ];

    let mut cases: Vec<TestCase> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut skipped_modules = 0u64;

    for (char_id, formula) in formulas {
        // Check if ALL formula modules exist in game data.
        // Python's talent resolution cross-references talent data from ALL modules
        // (e.g., uniequip_004's talent data can affect talent values when uniequip_003
        // is equipped). If any module is missing, talent data will diverge.
        let all_modules_exist = formula.available_modules.iter().all(|&m| {
            module_exists_in_game_data(&equip_keys, char_id, &formula.available_modules, m)
        });

        for &skill in &formula.available_skills {
            let modules: Vec<i32> = std::iter::once(0)
                .chain(formula.available_modules.iter().copied())
                .collect();
            for &module in &modules {
                // Skip ALL module tests if any formula module is missing from game data
                if module > 0 && !all_modules_exist {
                    skipped_modules += 1;
                    continue;
                }
                for &def in &defenses {
                    for &res in &resistances {
                        for &(fragile, dm, df, rm, rf) in &debuffs {
                            let key = make_test_key(
                                &formula.class_name,
                                skill,
                                module,
                                def,
                                res,
                                fragile,
                                dm,
                                df,
                                rm,
                                rf,
                            );
                            if seen.insert(key.clone()) {
                                cases.push(TestCase {
                                    key,
                                    operator: formula.class_name.clone(),
                                    skill,
                                    module,
                                    defense: def,
                                    res,
                                    fragile,
                                    def_shred_mult: dm,
                                    def_shred_flat: df,
                                    res_shred_mult: rm,
                                    res_shred_flat: rf,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    println!(
        "Generated {} test cases ({} module combos skipped — not in game data)",
        cases.len(),
        skipped_modules
    );

    // Write embedded Python script to temp file
    let py_script = format!(
        r#"#!/usr/bin/env python3
import sys, json, os
os.chdir("{repo_path}")
sys.path.insert(0, ".")
from damagecalc.damage_formulas import *
from damagecalc.utils import PlotParameters

def calc(tc):
    try:
        pp = PlotParameters(skill=tc['skill'], module=tc['module'])
        op = globals()[tc['operator']](pp)
    except Exception:
        return None
    d = max(0, (tc['defense'] - tc['def_shred_flat'])) * tc['def_shred_mult']
    r = max(0, (tc['res'] - tc['res_shred_flat'])) * tc['res_shred_mult']
    try:
        dps = float(op.skill_dps(d, r)) * (1 + tc['fragile'])
        return dps
    except Exception:
        return None

cases = json.load(sys.stdin)
results = {{}}
for tc in cases:
    dps = calc(tc)
    if dps is not None:
        results[tc['key']] = dps
json.dump(results, sys.stdout)
"#
    );

    let tmp_script = std::env::temp_dir().join("myrtle_dps_calc.py");
    fs::write(&tmp_script, &py_script).expect("Failed to write temp Python script");

    println!("Running Python DPS calculator...");
    let cases_json = serde_json::to_string(&cases).unwrap();

    let mut child = Command::new("python3")
        .arg(&tmp_script)
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
        eprintln!(
            "Python failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        std::process::exit(1);
    }

    let results: HashMap<String, f64> =
        serde_json::from_slice(&output.stdout).expect("Failed to parse Python output");

    println!("Received {} DPS values from Python", results.len());

    let output_path = std::path::Path::new("tests/fixtures/expected_dps.json");
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let json = serde_json::to_string_pretty(&results).unwrap();
    fs::write(output_path, &json).unwrap();
    println!("Written to {}", output_path.display());

    // Cleanup
    let _ = fs::remove_file(&tmp_script);
}

#[allow(clippy::too_many_arguments)]
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
            res_flat as i32
        )
    } else {
        base
    }
}

// ── Step 2: Transpile Python → Rust ─────────────────────────────────────

fn transpile_all(py_src: &str, formulas: &HashMap<String, OperatorFormula>) {
    println!("\n=== Transpiling Python → Rust ===");

    let mut custom_ops: Vec<(&str, &str, &OperatorFormula)> = Vec::new();
    for (char_id, formula) in formulas {
        custom_ops.push((char_id.as_str(), formula.class_name.as_str(), formula));
    }
    custom_ops.sort_by_key(|&(_, name, _)| name);

    println!("Found {} operators", custom_ops.len());

    // Parse all Python classes
    let class_re = Regex::new(r"class\s+(\w+)\s*\(\s*Operator\s*\)\s*:").unwrap();
    let class_positions: Vec<(usize, String)> = class_re
        .captures_iter(py_src)
        .map(|cap| (cap.get(0).unwrap().start(), cap[1].to_string()))
        .collect();

    let mut class_bodies: HashMap<String, String> = HashMap::new();
    for (i, (start, name)) in class_positions.iter().enumerate() {
        let end = class_positions
            .get(i + 1)
            .map(|(pos, _)| *pos)
            .unwrap_or(py_src.len());
        class_bodies.insert(name.clone(), py_src[*start..end].to_string());
    }

    // Generate Rust code
    let mut generated = String::new();
    generated.push_str("//! Auto-generated custom DPS implementations.\n");
    generated.push_str("//! Generated by `cargo run --bin generate-custom-dps`\n");
    generated.push_str("//! DO NOT EDIT MANUALLY.\n\n");
    generated.push_str("#![allow(unused_variables, unused_mut, unused_assignments, unused_parens, unreachable_code, path_statements, unused_must_use, non_snake_case, clippy::excessive_precision, clippy::unnecessary_cast, clippy::needless_return, clippy::collapsible_if, clippy::collapsible_else_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op, clippy::get_first, clippy::bool_comparison, clippy::no_effect)]\n\n");
    generated.push_str("use super::super::operator_unit::{EnemyStats, OperatorUnit};\n\n");

    let mut dispatch_arms: Vec<(String, String)> = Vec::new();
    let mut success = 0;
    let mut failed = 0;

    for (char_id, class_name, _formula) in &custom_ops {
        let Some(class_body) = class_bodies.get(*class_name) else {
            eprintln!("  SKIP: {class_name} — not found in Python source");
            failed += 1;
            continue;
        };

        let skill_dps_body = extract_method(class_body, "skill_dps");
        if skill_dps_body.trim().is_empty() {
            eprintln!("  SKIP: {class_name} — no skill_dps method");
            failed += 1;
            continue;
        }

        let init_body = extract_init_method(class_body);
        let init_mutations = extract_and_transpile_init_mutations(&init_body);
        let rust_fn_name = to_snake_case(class_name);
        let rust_body = transpile_skill_dps(&skill_dps_body, &init_mutations);

        let fn_start_pos = generated.len();
        generated.push_str(&format!(
            "/// {class_name} — auto-transpiled from Python\npub fn {rust_fn_name}(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {{\n"
        ));
        generated.push_str(&rust_body);
        generated.push_str("}\n\n");

        // Quick syntax check: look for known bad patterns — only in THIS function
        let fn_body_check = &generated[fn_start_pos..];
        let hardcoded_fallbacks: &[&str] = &[];

        let has_bad_syntax = hardcoded_fallbacks.contains(class_name)
            || fn_body_check.contains("= = ")
            || fn_body_check.contains("! = ");

        if has_bad_syntax {
            // Replace this function with a fallback stub
            generated.truncate(fn_start_pos);
            generated.push_str(&format!(
                "/// {class_name} — fallback (transpiler limitation)\npub fn {rust_fn_name}(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {{\n    None\n}}\n\n"
            ));
            eprintln!("  FALLBACK: {class_name} — syntax check failed");
        }

        dispatch_arms.push((char_id.to_string(), rust_fn_name));
        success += 1;
    }

    println!("Transpiled: {success}, Failed: {failed}");

    // Write generated.rs
    fs::write("src/dps/custom/generated.rs", &generated).expect("Failed to write generated.rs");
    println!("Written to src/dps/custom/generated.rs");

    // Write mod.rs with dispatch
    let mut mod_rs = String::new();
    mod_rs.push_str("use super::operator_unit::{EnemyStats, OperatorUnit};\n\n");
    mod_rs.push_str("mod generated;\n\n");
    mod_rs.push_str("pub fn dispatch(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {\n");
    mod_rs.push_str("    let op_id = unit.data.data.id.as_deref().unwrap_or(\"\");\n");
    mod_rs.push_str("    match op_id {\n");

    for (char_id, fn_name) in &dispatch_arms {
        mod_rs.push_str(&format!(
            "        \"{char_id}\" => generated::{fn_name}(unit, enemy),\n"
        ));
    }

    mod_rs.push_str("        _ => None,\n");
    mod_rs.push_str("    }\n");
    mod_rs.push_str("}\n");

    fs::write("src/dps/custom/mod.rs", &mod_rs).expect("Failed to write mod.rs");
    println!(
        "Written to src/dps/custom/mod.rs ({} dispatch arms)",
        dispatch_arms.len()
    );
}

// ── Transpilation ───────────────────────────────────────────────────────

/// Rust keywords that can't be used as variable names
const RUST_KEYWORDS: &[(&str, &str)] = &[
    ("crate", "crate_val"),
    ("type", "type_val"),
    ("match", "match_val"),
    ("mod", "mod_val"),
    ("ref", "ref_val"),
    ("fn", "fn_val"),
    ("use", "use_val"),
    ("loop", "loop_val"),
    ("move", "move_val"),
    ("self", "self_val"),
    ("super", "super_val"),
    ("trait", "trait_val"),
    ("where", "where_val"),
    ("async", "async_val"),
    ("await", "await_val"),
    ("dyn", "dyn_val"),
    ("impl", "impl_val"),
    ("pub", "pub_val"),
    ("return", "return_val"),
    ("static", "static_val"),
    ("struct", "struct_val"),
    ("enum", "enum_val"),
    ("extern", "extern_val"),
    ("const", "const_val"),
    ("mut", "mut_val"),
    ("unsafe", "unsafe_val"),
];

/// Holds transpiled __init__ mutations: shadow declarations + mutation code + modified fields
struct InitMutations {
    shadow_decls: Vec<String>, // e.g. "let mut drone_atk: f64 = unit.drone_atk;"
    mutations: Vec<String>,    // e.g. "if skill == 1 { drone_atk += 45.0; }"
    modified_fields: std::collections::HashSet<String>, // e.g. {"drone_atk", "trait_damage"}
}

/// Extract stat-modifying lines from __init__ and transpile them with a simple mini-transpiler.
#[allow(dead_code)]
fn extract_and_transpile_init_mutations(init_body: &str) -> InitMutations {
    let mut mutations = Vec::new();
    let mut modified_fields = std::collections::HashSet::new();

    let field_map: &[(&str, &str, &str)] = &[
        // (python_field, rust_field, type)
        ("drone_atk", "drone_atk", "f64"),
        ("attack_speed", "attack_speed", "f64"),
        ("trait_dmg", "trait_damage", "bool"),
        ("talent_dmg", "talent_damage", "bool"),
        ("talent2_dmg", "talent2_damage", "bool"),
        ("skill_dmg", "skill_damage", "bool"),
        ("module_dmg", "module_damage", "bool"),
        ("ammo", "ammo", "f64"),
        ("skill_duration", "skill_duration", "f64"),
    ];

    let mut after_super = false;
    for line in init_body.lines() {
        let trimmed = line.trim();
        if trimmed.contains("super().__init__") {
            after_super = true;
            continue;
        }
        if !after_super || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.contains("self.name") {
            continue;
        }

        for &(py_field, rust_field, _field_type) in field_map {
            let mut rust_line = None;

            // Pattern: self.field = EXPR
            if let Some(rest) = trimmed.strip_prefix(&format!("self.{py_field} =")) {
                let expr = transpile_init_expr(rest.trim_start());
                rust_line = Some(format!("    {rust_field} = {expr};"));
            }
            // Pattern: self.field += EXPR
            else if let Some(rest) = trimmed.strip_prefix(&format!("self.{py_field} +=")) {
                let expr = transpile_init_expr(rest.trim_start());
                rust_line = Some(format!("    {rust_field} += {expr};"));
            }
            // Pattern: self.field -= EXPR
            else if let Some(rest) = trimmed.strip_prefix(&format!("self.{py_field} -=")) {
                let expr = transpile_init_expr(rest.trim_start());
                rust_line = Some(format!("    {rust_field} -= {expr};"));
            }
            // Pattern: if COND: self.field OP= EXPR
            else if trimmed.starts_with("if ")
                && trimmed.contains(&format!(": self.{py_field}"))
                && let Some((cond_part, stmt_part)) =
                    trimmed.strip_prefix("if ").and_then(|s| s.split_once(':'))
            {
                let cond = transpile_init_expr(cond_part.trim());
                let stmt = stmt_part.trim();
                if let Some(rest) = stmt.strip_prefix(&format!("self.{py_field} =")) {
                    let expr = transpile_init_expr(rest.trim_start());
                    rust_line = Some(format!("    if {cond} {{ {rust_field} = {expr}; }}"));
                } else if let Some(rest) = stmt.strip_prefix(&format!("self.{py_field} +=")) {
                    let expr = transpile_init_expr(rest.trim_start());
                    rust_line = Some(format!("    if {cond} {{ {rust_field} += {expr}; }}"));
                } else if let Some(rest) = stmt.strip_prefix(&format!("self.{py_field} -=")) {
                    let expr = transpile_init_expr(rest.trim_start());
                    rust_line = Some(format!("    if {cond} {{ {rust_field} -= {expr}; }}"));
                }
            }

            if let Some(rl) = rust_line {
                // Validate: skip mutations with untranspilable patterns
                // Skip compound boolean mutations (reference multiple shadow vars)
                // Skip complex expressions that can't be simply transpiled
                let valid = !rl.contains("self.")
                    && !rl.contains("module_index_")
                    && !rl.contains("drone_atk_interval")
                    && !rl.contains("aspd")
                    && !rl.contains(".get(")
                    && !rl.contains("pp.")
                    && !rl.contains("&& module_damage")
                    && !rl.contains("&& talent_damage")
                    && !rl.contains("&& skill_damage")
                    && !rl.contains("&& trait_damage")
                    && !rl.contains("(unit.module_level as f64)")
                    && !rl.contains("module_level > ")
                    && !rl.contains("4.0 + 4.0")
                    && !rl.contains("attack_speed -=")
                    && (!rl.trim().starts_with("ammo = 1;") || rl.contains("if "));
                if valid {
                    // Use the main coercion function to handle integer→float conversion
                    let fixed = coerce_arithmetic_literals(&rl);
                    modified_fields.insert(rust_field.to_string());
                    mutations.push(format!("    {}", fixed.trim()));
                }
                break;
            }
        }

        // Handle self.atk separately (substring of other fields)
        if !trimmed.contains("drone_atk") && !trimmed.contains("attack") {
            if let Some(rest) = trimmed.strip_prefix("self.atk +=") {
                let expr = transpile_init_expr(rest.trim_start());
                let line = format!("    atk += {expr};");
                let coerced = coerce_arithmetic_literals(&line);
                modified_fields.insert("atk".to_string());
                mutations.push(format!("    {}", coerced.trim()));
            } else if trimmed.starts_with("if ")
                && trimmed.contains(": self.atk +=")
                && !trimmed.contains("drone")
                && !trimmed.contains("module_lvl")  // Skip nested module_level conditions (lost outer guard)
                && let Some((cond_part, stmt_part)) =
                    trimmed.strip_prefix("if ").and_then(|s| s.split_once(':'))
                && let Some(rest) = stmt_part.trim().strip_prefix("self.atk +=")
            {
                let cond = transpile_init_expr(cond_part.trim());
                let expr = transpile_init_expr(rest.trim_start());
                let line = format!("    if {cond} {{ atk += {expr}; }}");
                let coerced = coerce_arithmetic_literals(&line);
                modified_fields.insert("atk".to_string());
                mutations.push(format!("    {}", coerced.trim()));
            }
        }
    }

    // Build shadow declarations. When ANY init mutation exists, shadow ALL boolean
    // fields unconditionally — this ensures cross-references in mutation conditions
    // (e.g., "if skill_damage && module_damage: talent_damage = false") work.
    let mut shadow_decls = Vec::new();
    let has_any_mutation = !mutations.is_empty();

    let numeric_fields: &[(&str, &str)] = &[
        ("drone_atk", "let mut drone_atk: f64 = unit.drone_atk;"),
        ("atk", "let mut atk: f64 = unit.atk;"),
        (
            "attack_speed",
            "let mut attack_speed: f64 = unit.attack_speed;",
        ),
        ("ammo", "let mut ammo: f64 = unit.ammo;"),
        (
            "skill_duration",
            "let mut skill_duration: f64 = unit.skill_duration;",
        ),
    ];
    let bool_fields: &[(&str, &str)] = &[
        (
            "trait_damage",
            "let mut trait_damage: bool = unit.trait_damage;",
        ),
        (
            "talent_damage",
            "let mut talent_damage: bool = unit.talent_damage;",
        ),
        (
            "talent2_damage",
            "let mut talent2_damage: bool = unit.talent2_damage;",
        ),
        (
            "skill_damage",
            "let mut skill_damage: bool = unit.skill_damage;",
        ),
        (
            "module_damage",
            "let mut module_damage: bool = unit.module_damage;",
        ),
    ];

    // Numeric fields: only shadow if explicitly modified
    for &(field, decl) in numeric_fields {
        if modified_fields.contains(field) {
            shadow_decls.push(format!("    {decl}"));
        }
    }
    // Boolean fields: shadow ALL if any mutation exists (for cross-references)
    if has_any_mutation {
        for &(_field, decl) in bool_fields {
            shadow_decls.push(format!("    {decl}"));
        }
        // Add all bool fields to modified_fields so body replacement works
        for &(field, _) in bool_fields {
            modified_fields.insert(field.to_string());
        }
    }

    InitMutations {
        shadow_decls,
        mutations,
        modified_fields,
    }
}

/// Simple init expression transpiler — no regex, just string replacements.
#[allow(dead_code)]
fn transpile_init_expr(py_expr: &str) -> String {
    let mut s = py_expr.to_string();
    // Field mappings (order matters — longer/specific prefixes first)
    s = s.replace("self.skill_params", "unit.skill_parameters");
    s = s.replace("self.module_lvl", "(unit.module_level as f64)");
    s = s.replace("self.module_dmg", "module_damage");
    s = s.replace("self.module", "unit.module_index");
    s = s.replace("self.elite", "unit.elite");
    s = s.replace("self.trait_dmg", "trait_damage");
    s = s.replace("self.talent2_dmg", "talent2_damage");
    s = s.replace("self.talent_dmg", "talent_damage");
    s = s.replace("self.skill_dmg", "skill_damage");
    s = s.replace("self.atk_interval", "unit.attack_interval as f64");
    s = s.replace("self.attack_speed", "unit.attack_speed");
    s = s.replace("self.ammo", "unit.ammo");
    s = s.replace("self.skill", "skillf");
    s = s.replace("pp.pot", "unit.potential");
    // Boolean
    s = s.replace("True", "true");
    s = s.replace("False", "false");
    s = s.replace(" and ", " && ");
    s = s.replace(" or ", " || ");
    s = s.replace("not ", "!");
    // Python `in [N,M]` → manual expansion (no regex)
    if s.contains(" in [")
        && let Some(start) = s.find(" in [")
    {
        let var_start = s[..start]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
            .map(|i| i + 1)
            .unwrap_or(0);
        let var = &s[var_start..start];
        if let Some(end) = s[start..].find(']') {
            let items_str = &s[start + 5..start + end];
            let items: Vec<String> = items_str
                .split(',')
                .map(|i| format!("{var} == {}", i.trim()))
                .collect();
            s = format!(
                "{}({}){}",
                &s[..var_start],
                items.join(" || "),
                &s[start + end + 1..]
            );
        }
    }
    // Add .0 to integers ONLY in augmented assignments (+=, -=, =) for f64 contexts
    // Do NOT convert comparison values (== N should stay as integer for i32 fields)
    for n in [
        "100", "60", "50", "45", "40", "35", "25", "20", "15", "12", "10", "6", "4", "3", "2", "1",
    ] {
        let patterns = [
            (format!("+= {n};"), format!("+= {n}.0;")),
            (format!("-= {n};"), format!("-= {n}.0;")),
            (format!("+ {n}.0 *"), format!("+ {n}.0 *")), // already has .0
        ];
        for (from, to) in &patterns {
            s = s.replace(from.as_str(), to.as_str());
        }
    }
    s
}

fn transpile_skill_dps(py_body: &str, init_mutations: &InitMutations) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Pass 1: Collect all assigned variable names for hoisting
    let assign_var_re = Regex::new(r"^\t*(\w+)\s*(?:=|\+=|-=|\*=|/=)\s").unwrap();
    let mut all_vars: Vec<String> = Vec::new();
    let mut vec_vars: std::collections::HashSet<String> = std::collections::HashSet::new();
    let builtin_vars = ["dps", "skill", "defense", "res", "self", "atk_interval"];

    // Detect if the Python body mutates self.atk_interval (field write vs local assignment)
    // If yes: map self.atk_interval → atk_interval (the local shadow)
    // If no: map self.atk_interval → unit.attack_interval as f64 (immutable field read)
    let _mutates_atk_interval =
        py_body.contains("self.atk_interval =") || py_body.contains("self.atk_interval=");

    // Also capture variables assigned inside inline if statements
    let inline_assign_re = Regex::new(r"if\s+.+?:\s*(\w+)\s*(?:=|\+=|-=|\*=|/=)\s").unwrap();
    // Detect array assignments: VAR = [...]
    let array_assign_re = Regex::new(r"^\t*(\w+)\s*=\s*\[").unwrap();

    for py_line in py_body.lines() {
        let trimmed = py_line.trim();
        // Skip comments, returns, name assignments
        if trimmed.starts_with('#')
            || trimmed.starts_with("return")
            || trimmed.starts_with("self.name")
            || trimmed.starts_with("try")
            || trimmed.starts_with("except")
            || trimmed.starts_with("print")
            || trimmed.is_empty()
        {
            continue;
        }

        // Detect array variable assignments
        if let Some(cap) = array_assign_re.captures(trimmed) {
            let var = escape_keyword(&cap[1]);
            if !builtin_vars.contains(&var.as_str()) {
                vec_vars.insert(var.clone());
            }
        }

        // Capture vars from inline if: "if COND: VAR = ..."
        if let Some(cap) = inline_assign_re.captures(trimmed) {
            let var = escape_keyword(&cap[1]);
            if !builtin_vars.contains(&var.as_str()) && !all_vars.contains(&var) {
                all_vars.push(var);
            }
        }

        // Skip regular control flow for the main regex
        if trimmed.starts_with("if ")
            || trimmed.starts_with("elif ")
            || trimmed.starts_with("else")
            || trimmed.starts_with("for ")
        {
            continue;
        }
        if let Some(cap) = assign_var_re.captures(trimmed) {
            let var = escape_keyword(&cap[1]);
            if !builtin_vars.contains(&var.as_str()) && !all_vars.contains(&var) {
                all_vars.push(var);
            }
        }
    }

    // Emit standard declarations
    lines.push("    let skill = unit.skill_index;".into());
    lines.push("    let skillf = unit.skill_index as f64;".into());
    lines.push("    let mut defense = enemy.defense;".into());
    lines.push("    let mut res = enemy.res;".into());
    lines.push("    let mut atk_interval: f64 = unit.attack_interval as f64;".into());
    lines.push("    let mut dps: f64 = 0.0;".into());

    // __init__ shadow declarations and mutations (BEFORE hoisted vars)
    for decl in &init_mutations.shadow_decls {
        lines.push(decl.clone());
    }
    for mutation in &init_mutations.mutations {
        lines.push(mutation.clone());
    }

    // Hoist all variable declarations to function scope
    for var in &all_vars {
        if vec_vars.contains(var) {
            lines.push(format!("    let mut {var}: Vec<f64> = Vec::new();"));
        } else {
            lines.push(format!("    let mut {var}: f64 = 0.0;"));
        }
    }
    lines.push(String::new());

    // Mark the split point: everything above is declarations, below is body
    let decl_end_idx = lines.len();

    // Pass 2: Transpile lines — NO `let mut` in the body, just assignments
    let mut declared_vars: std::collections::HashSet<String> = std::collections::HashSet::new();
    // Mark everything as already declared so transpile_expressions won't add `let mut`
    declared_vars.extend(builtin_vars.iter().map(|s| s.to_string()));
    declared_vars.extend(all_vars.iter().cloned());

    let mut indent_stack: Vec<usize> = vec![0];
    let mut in_except_block = false;
    let mut except_indent: usize = 0;
    let mut final_return_expr: Option<String> = None; // Captures "return dps * X" at top level

    for py_line in py_body.lines() {
        let no_comment = py_line.split('#').next().unwrap_or(py_line);
        let trimmed = no_comment.trim();
        if trimmed.is_empty() {
            continue;
        }

        let tab_count = py_line.chars().take_while(|&c| c == '\t').count();

        // Skip try: line (keep its body)
        if trimmed == "try:" {
            continue;
        }
        // Skip except block entirely (including its body)
        if trimmed.starts_with("except") {
            in_except_block = true;
            except_indent = tab_count;
            continue;
        }
        if in_except_block {
            if tab_count > except_indent {
                continue;
            }
            in_except_block = false;
        }

        // Handle standalone "return EXPR" lines
        if trimmed.starts_with("return ") {
            let expr = trimmed.strip_prefix("return ").unwrap().trim();
            if expr == "dps" {
                continue;
            }
            let transpiled = transpile_expressions(expr, &mut declared_vars);
            let coerced = coerce_arithmetic_literals(&transpiled);
            // Top-level return (indent 0): capture as final expression instead of early return
            // This ensures modifiers like "return dps * scale" apply to ALL skill branches
            if tab_count == 0 {
                final_return_expr = Some(coerced);
                continue;
            }
            let rust_indent = "    ".repeat(tab_count + 1);
            lines.push(format!("{rust_indent}return Some({coerced});"));
            continue;
        }
        if trimmed == "return" {
            continue;
        }

        let tab_count = py_line.chars().take_while(|&c| c == '\t').count();
        let py_indent = tab_count;

        // Close braces when indent decreases
        // `else:` and `elif` are continuations (attach to preceding if block).
        let is_continuation = trimmed.starts_with("else:") || trimmed.starts_with("elif ");
        if !is_continuation {
            while py_indent < *indent_stack.last().unwrap_or(&0) {
                indent_stack.pop();
                let rust_indent = "    ".repeat(py_indent + 1);
                lines.push(format!("{rust_indent}}}"));
            }
        } else {
            // For else/elif: close ALL blocks deeper than this indent, then pop one more
            // for the if-block that this else belongs to
            while *indent_stack.last().unwrap_or(&0) > py_indent + 1 {
                indent_stack.pop();
                let depth = *indent_stack.last().unwrap_or(&0);
                let rust_indent = "    ".repeat(depth);
                lines.push(format!("{rust_indent}}}"));
            }
            if py_indent < *indent_stack.last().unwrap_or(&0) {
                indent_stack.pop();
            }
        }

        let raw_line = transpile_line(trimmed, &mut declared_vars);
        if raw_line.is_empty() {
            continue;
        }
        // Apply integer coercion to ALL transpiled lines (including early returns)
        let rust_line = coerce_arithmetic_literals(&raw_line);

        // Merge inline else/elif with preceding inline if:
        // Previous: "if COND { STMT; }"
        // Current:  "} else { STMT; }" or "} else if COND { STMT; }"
        // Result:   "if COND { STMT; } else { STMT; }" or "if COND { STMT; } else if COND { STMT; }"
        if rust_line.trim_start().starts_with("} else ")
            && let Some(prev) = lines.last_mut()
        {
            let pt = prev.trim();
            if pt.ends_with('}') && pt.contains("if ") {
                let else_part = rust_line
                    .trim_start()
                    .strip_prefix("}")
                    .unwrap_or(rust_line.trim_start());
                prev.push_str(else_part);
                // Track else block if it opens a new scope (ends with { but not })
                if else_part.trim_end().ends_with('{')
                    && !else_part.trim_end().ends_with(";}")
                    && !else_part.trim_end().ends_with(" }")
                {
                    indent_stack.push(py_indent + 1);
                }
                continue;
            }
        }

        // Track block opens (if, else, for blocks)
        if rust_line.trim().ends_with('{') || rust_line.contains("for ") && rust_line.contains('{')
        {
            indent_stack.push(py_indent + 1);
        }

        let rust_indent = "    ".repeat(py_indent + 1);
        lines.push(format!("{rust_indent}{}", rust_line.trim()));
    }

    // Close ALL remaining open braces before emitting Some(dps)
    // Count actual braces in the generated body to ensure balance
    let body_so_far = lines.join("\n");
    let opens: i32 = body_so_far.chars().filter(|&c| c == '{').count() as i32;
    let closes: i32 = body_so_far.chars().filter(|&c| c == '}').count() as i32;
    let unclosed = opens - closes;
    for i in (0..unclosed).rev() {
        lines.push(format!("{}}}", "    ".repeat(i as usize + 1)));
    }

    lines.push(String::new());
    if let Some(expr) = final_return_expr {
        lines.push(format!("    Some({expr})"));
    } else {
        lines.push("    Some(dps)".into());
    }
    lines.push(String::new());

    // Two-phase assembly: declarations are untouched, body gets unit.X → shadow replacements
    let decl_section = lines[..decl_end_idx].join("\n");
    let body_section = lines[decl_end_idx..].join("\n");

    let mut processed_body = body_section;

    // Apply unit.X → shadow var replacements ONLY to the body (not declarations)
    // Order: longer/more-specific first to avoid substring collisions
    for field in &init_mutations.modified_fields {
        match field.as_str() {
            "drone_atk" => {
                // Preserve drone_atk_interval before replacing drone_atk
                processed_body =
                    processed_body.replace("unit.drone_atk_interval", "___DRONE_INTERVAL___");
                processed_body = processed_body.replace("unit.drone_atk", "drone_atk");
                processed_body =
                    processed_body.replace("___DRONE_INTERVAL___", "unit.drone_atk_interval");
            }
            "atk" => {
                // Preserve attack_speed and atk_interval before replacing atk
                processed_body = processed_body.replace("unit.attack_speed", "___ASPD___");
                processed_body = processed_body.replace("unit.atk", "atk");
                processed_body = processed_body.replace("___ASPD___", "unit.attack_speed");
            }
            "attack_speed" => {
                processed_body = processed_body.replace("unit.attack_speed", "attack_speed")
            }
            "ammo" => processed_body = processed_body.replace("unit.ammo", "ammo"),
            "trait_damage" => {
                processed_body = processed_body.replace("unit.trait_damage", "trait_damage")
            }
            "talent_damage" => {
                processed_body = processed_body.replace("unit.talent_damage", "talent_damage")
            }
            "talent2_damage" => {
                processed_body = processed_body.replace("unit.talent2_damage", "talent2_damage")
            }
            "skill_damage" => {
                processed_body = processed_body.replace("unit.skill_damage", "skill_damage")
            }
            "module_damage" => {
                processed_body = processed_body.replace("unit.module_damage", "module_damage")
            }
            "skill_duration" => {
                processed_body = processed_body.replace("unit.skill_duration", "skill_duration")
            }
            _ => {}
        }
    }

    format!("{decl_section}\n{processed_body}")
}

/// Converts bare integer literals to f64 in arithmetic contexts.
/// Uses nuclear approach: convert ALL standalone integers to floats,
/// then revert the specific contexts where integers are needed.
fn coerce_arithmetic_literals(line: &str) -> String {
    let mut s = line.to_string();

    // Clean up double-space operators first
    s = s.replace("= =", "==");
    s = s.replace("! =", "!=");

    // Don't touch lines that have no arithmetic (pure braces, comments)
    let t = s.trim();
    if t == "}" || t == "} else {" || t.starts_with("//") {
        return s;
    }

    // Fix self_val references (keyword escaping caught remaining `self`)
    s = s.replace("self_val.", "unit.");

    // Step 1: Convert ALL standalone integer literals to floats
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::with_capacity(s.len() + 100);
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let preceded_by_word = start > 0
                && (chars[start - 1] == '.'
                    || chars[start - 1].is_ascii_alphanumeric()
                    || chars[start - 1] == '_');
            let followed_by_dot_digit = i < chars.len()
                && chars[i] == '.'
                && i + 1 < chars.len()
                && chars[i + 1].is_ascii_digit();
            let followed_by_word =
                i < chars.len() && (chars[i].is_ascii_alphabetic() || chars[i] == '_');
            let num: String = chars[start..i].iter().collect();
            if preceded_by_word || followed_by_dot_digit || followed_by_word {
                result.push_str(&num);
            } else {
                result.push_str(&num);
                result.push_str(".0");
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    s = result;

    // Step 2: Revert contexts where integers are needed

    // .get(N.0) → .get(N)
    let get_fix = Regex::new(r"\.get\((\d+)\.0\)").unwrap();
    s = get_fix.replace_all(&s, ".get($1)").to_string();

    // [N.0] → [N] (array indexing)
    let idx_fix = Regex::new(r"\[(\d+)\.0\]").unwrap();
    s = idx_fix.replace_all(&s, "[$1]").to_string();

    // .len() - N.0 → .len() - N (usize arithmetic)
    let len_sub = Regex::new(r"\.len\(\)\s*-\s*(\d+)\.0").unwrap();
    s = len_sub.replace_all(&s, ".len() - $1").to_string();

    // Slice N.0: → N: (inside brackets)
    let slice_fix = Regex::new(r"\[(\d+)\.0:\]").unwrap();
    s = slice_fix.replace_all(&s, "[$1..]").to_string();
    let slice_fix2 = Regex::new(r"\[:(\d+)\.0\]").unwrap();
    s = slice_fix2.replace_all(&s, "[..$1]").to_string();

    // skillf in comparisons → skill (i32 comparison)
    let skillf_cmp = Regex::new(r"\bskillf\s*(==|!=|<=|>=|<|>)\s*").unwrap();
    s = skillf_cmp.replace_all(&s, "skill $1 ").to_string();
    s = s.replace("if skillf ", "if skill ");

    // i32 fields: comparisons with floats → revert to bare integer
    // NOTE: unit.ammo is f64, not i32 — do NOT include it here
    let i32_float_cmp = Regex::new(r"\b(skill|unit\.elite|unit\.module_index|unit\.module_level|unit\.potential)\s*(==|!=|<=|>=|<|>)\s*(\d+)\.0\b").unwrap();
    s = i32_float_cmp.replace_all(&s, "$1 $2 $3").to_string();

    // Casted comparisons: "(unit.X as f64) > N.0" → "unit.X > N"
    let cast_cmp = Regex::new(r"\(unit\.(elite|module_level|module_index|potential) as f64\)\s*(==|!=|<=|>=|<|>)\s*(\d+)\.0?\b").unwrap();
    s = cast_cmp.replace_all(&s, "unit.$1 $2 $3").to_string();

    // .max(N.0) → .max(N.0_f64) — disambiguate for type inference
    let max_float = Regex::new(r"\.max\((\d+\.\d+)\)").unwrap();
    s = max_float.replace_all(&s, ".max(${1}_f64)").to_string();
    let min_float = Regex::new(r"\.min\((\d+\.\d+)\)").unwrap();
    s = min_float.replace_all(&s, ".min(${1}_f64)").to_string();

    // Ambiguous float RECEIVER: "(N.0).max(" → "(N.0_f64).max("
    let recv_max = Regex::new(r"\((\d+\.\d+)\)\.max\(").unwrap();
    s = recv_max.replace_all(&s, "(${1}_f64).max(").to_string();
    let recv_min = Regex::new(r"\((\d+\.\d+)\)\.min\(").unwrap();
    s = recv_min.replace_all(&s, "(${1}_f64).min(").to_string();

    // Ranges: N.0..N.0 → N..N
    let range_fix = Regex::new(r"(\d+)\.0\.\.(\d+)\.0").unwrap();
    s = range_fix.replace_all(&s, "$1..$2").to_string();
    let range_fix2 = Regex::new(r"(\d+)\.0\.\.").unwrap();
    s = range_fix2.replace_all(&s, "$1..").to_string();
    let range_fix3 = Regex::new(r"\.\.(\d+)\.0([)\s])").unwrap();
    s = range_fix3.replace_all(&s, "..$1$2").to_string();

    // .trunc().0 → .trunc()
    s = s.replace(".trunc().0", ".trunc()");
    // as f64.0 → as f64
    s = s.replace("as f64.0", "as f64");
    // as i32.0 → as i32
    s = s.replace("as i32.0", "as i32");
    // as usize.0 → as usize
    s = s.replace("as usize.0", "as usize");
    // as i64.0 → as i64
    s = s.replace("as i64.0", "as i64");

    // Double float: N.0.0 → N.0
    let double_float = Regex::new(r"(\d+)\.0\.0").unwrap();
    s = double_float.replace_all(&s, "$1.0").to_string();
    s = s.replace(".0.0)", ".0)");

    // Augmented assignments with integers: "*= N;" → "*= N.0;"
    let aug_int = Regex::new(r"(\*=|\+=|-=|/=) (\d+);").unwrap();
    s = aug_int.replace_all(&s, "$1 $2.0;").to_string();

    // Leftover skillf in comparison context
    let skillf_leftover = Regex::new(r"\bskillf\s*(==|!=|<=|>=|<|>)\s*(\d+(?:\.\d+)?)").unwrap();
    s = skillf_leftover
        .replace_all(&s, |caps: &regex::Captures| {
            let op = &caps[1];
            let num = caps[2].trim_end_matches(".0");
            format!("skill {op} {num}")
        })
        .to_string();

    s
}

fn transpile_line(py: &str, declared: &mut std::collections::HashSet<String>) -> String {
    // Skip Python-only constructs
    if py.starts_with("self.name +=")
        || py.starts_with("self.name=")
        || py.starts_with("self.name +=")
        || py.starts_with("print(")
    {
        return String::new();
    }

    // Skip bare attribute reads (no-op in Python): "self.something" or "self.something and ..."
    // These are just side-effect-free expressions that Python evaluates but discards
    let bare_ref_re =
        Regex::new(r"^self\.\w+(\s+(and|or)\s+self\.\w+)*\s*(and\s+self\.\w+)*;?\s*$").unwrap();
    if bare_ref_re.is_match(py.trim()) && !py.contains('=') && !py.contains('(') {
        return String::new();
    }
    // Also skip standalone string literals (""")
    if py.trim() == "\"\"" || py.trim() == "''" || py.trim().starts_with("\"\"\"") {
        return String::new();
    }

    let mut line = py.to_string();

    // Handle inline "try: STMT" — strip the try: prefix, keep the statement
    if line.starts_with("try: ") || line.starts_with("try:") {
        line = line.trim_start_matches("try:").trim().to_string();
    }

    // Handle "if COND: return EXPR" → "if COND { return Some(EXPR); }"
    let if_return_re = Regex::new(r"^if\s+(.+?):\s*return\s+(.+)$").unwrap();
    if let Some(cap) = if_return_re.captures(&line) {
        let cond = transpile_expressions(&cap[1], declared);
        let val = transpile_expressions(&cap[2], declared);
        return format!("if {cond} {{ return Some({val}); }}");
    }

    // Handle self.atk_interval = X (override local atk_interval variable)
    let self_assign_re = Regex::new(r"^self\.atk_interval\s*=\s*(.+)$").unwrap();
    if let Some(cap) = self_assign_re.captures(&line) {
        let val = transpile_expressions(&cap[1], declared);
        return format!("atk_interval = {val};");
    }

    // Handle "if COND: self.atk_interval = X"
    let if_atk_re = Regex::new(r"^if\s+(.+?):\s*self\.atk_interval\s*=\s*(.+)$").unwrap();
    if let Some(cap) = if_atk_re.captures(&line) {
        let cond = transpile_expressions(&cap[1], declared);
        let val = transpile_expressions(&cap[2], declared);
        return format!("if {cond} {{ atk_interval = {val}; }}");
    }

    // Inline if with array index assignment: "if COND: VAR[N] = EXPR"
    let inline_if_arr_re = Regex::new(r"^if\s+(.+?):\s*(\w+)\[([^\]]+)\]\s*=\s*(.+)$").unwrap();
    if let Some(cap) = inline_if_arr_re.captures(&line) {
        let cond = transpile_expressions(&cap[1], declared);
        let var = escape_keyword(&cap[2]);
        let idx = &cap[3];
        let val = transpile_expressions(&cap[4], declared);
        return format!("if {cond} {{ {var}[{idx}] = {val}; }}");
    }

    // Array index assignment: VAR[N] = EXPR
    let arr_idx_assign_re = Regex::new(r"^(\w+)\[([^\]]+)\]\s*=\s*(.+)$").unwrap();
    if let Some(cap) = arr_idx_assign_re.captures(&line) {
        let var = escape_keyword(&cap[1]);
        let idx = &cap[2];
        let val = transpile_expressions(&cap[3], declared);
        return format!("{var}[{idx}] = {val};");
    }

    // Array literal assignment: VAR = [val1, val2, ...]
    let arr_literal_re = Regex::new(r"^(\w+)\s*=\s*\[([^\]]*)\]$").unwrap();
    if let Some(cap) = arr_literal_re.captures(&line) {
        let var = escape_keyword(&cap[1]);
        let elements: Vec<String> = cap[2]
            .split(',')
            .map(|e| transpile_expressions(e.trim(), declared))
            .collect();
        return format!("{var} = vec![{}];", elements.join(", "));
    }

    // Array ternary: VAR = [A,B] if COND else [C,D]
    let arr_ternary_re =
        Regex::new(r"^(\w+)\s*=\s*\[([^\]]*)\]\s+if\s+(.+?)\s+else\s+\[([^\]]*)\]$").unwrap();
    if let Some(cap) = arr_ternary_re.captures(&line) {
        let var = escape_keyword(&cap[1]);
        let true_elems: Vec<String> = cap[2]
            .split(',')
            .map(|e| transpile_expressions(e.trim(), declared))
            .collect();
        let cond = transpile_expressions(&cap[3], declared);
        let false_elems: Vec<String> = cap[4]
            .split(',')
            .map(|e| transpile_expressions(e.trim(), declared))
            .collect();
        return format!(
            "{var} = if {cond} {{ vec![{}] }} else {{ vec![{}] }};",
            true_elems.join(", "),
            false_elems.join(", ")
        );
    }

    // Single-line if with assignment: "if COND: VAR = EXPR"
    let inline_if_re = Regex::new(r"^if\s+(.+?):\s*(\w+)\s*=\s*(.+)$").unwrap();
    if let Some(cap) = inline_if_re.captures(&line) {
        let cond = transpile_expressions(&cap[1], declared);
        let var = escape_keyword(&cap[2]);
        let mut val = cap[3].to_string();
        // Handle array literal in the value: [a, b, c]
        let arr_re = Regex::new(r"^\[([^\]]*)\]$").unwrap();
        if let Some(ac) = arr_re.captures(&val) {
            let elements: Vec<String> = ac[1]
                .split(',')
                .map(|e| transpile_expressions(e.trim(), declared))
                .collect();
            val = format!("vec![{}]", elements.join(", "));
        }
        // Handle nested ternary in the value
        else {
            let inner_ternary = Regex::new(r"^(.+?)\s+if\s+(.+?)\s+else\s+(.+)$").unwrap();
            if let Some(tc) = inner_ternary.captures(&val) {
                let t_val = transpile_expressions(&tc[1], declared);
                let t_cond = transpile_expressions(&tc[2], declared);
                let f_val = transpile_expressions(&tc[3], declared);
                val = format!("if {t_cond} {{ {t_val} }} else {{ {f_val} }}");
            } else {
                val = transpile_expressions(&val, declared);
            }
        }
        return format!("if {cond} {{ {var} = {val}; }}");
    }

    // Handle "if COND: VAR += EXPR" etc
    let inline_if_op_re = Regex::new(r"^if\s+(.+?):\s*(\w+)\s*(\+=|-=|\*=)\s*(.+)$").unwrap();
    if let Some(cap) = inline_if_op_re.captures(&line) {
        let cond = transpile_expressions(&cap[1], declared);
        let var = escape_keyword(&cap[2]);
        let op = &cap[3];
        let val = transpile_expressions(&cap[4], declared);
        return format!("if {cond} {{ {var} {op} {val}; }}");
    }

    // Inline elif: "elif COND: STMT" → close previous block, open new if
    let inline_elif_re = Regex::new(r"^elif\s+(.+?):\s*(.+)$").unwrap();
    if let Some(cap) = inline_elif_re.captures(&line)
        && !cap[2].trim().is_empty()
        && !cap[2].trim().starts_with('#')
    {
        let cond = transpile_expressions(&cap[1], declared);
        let stmt = transpile_expressions(&cap[2], declared);
        return format!("}} else if {cond} {{ {stmt}; }}");
    }

    // Python for-range loop: for i in range(N) → for _i in 0..(N as i32) { let i = _i as f64;
    let for_range_re = Regex::new(r"^for\s+(\w+)\s+in\s+range\((.+)\):?$").unwrap();
    if let Some(cap) = for_range_re.captures(&line) {
        let var = &cap[1];
        let n = transpile_expressions(&cap[2], declared);
        declared.insert(var.to_string());
        return format!("for _{var} in 0..(({n}) as i32) {{ let {var} = _{var} as f64;");
    }

    // if ... in [...]:
    let if_in_re = Regex::new(r"^if\s+self\.skill\s+in\s*\[([^\]]+)\]:?$").unwrap();
    if let Some(cap) = if_in_re.captures(&line) {
        let nums: Vec<String> = cap[1]
            .split(',')
            .map(|s| format!("skill == {}", s.trim()))
            .collect();
        return format!("if {} {{", nums.join(" || "));
    }

    // if/elif with colon — elif becomes separate if (not else if)
    // Also handles if(COND): with parens around condition
    if line.ends_with(':')
        && (line.starts_with("if ") || line.starts_with("if(") || line.starts_with("elif "))
    {
        line = line.trim_end_matches(':').to_string();
        if line.starts_with("elif ") {
            line = format!("}} else if {}", &line[5..]);
        }
        // Strip outer parens from if(COND) → if COND
        if line.starts_with("if(") && line.ends_with(')') {
            line = format!("if {}", &line[3..line.len() - 1]);
        }
        line = transpile_expressions(&line, declared);
        return format!("{line} {{");
    }

    // else: (standalone)
    if line.trim() == "else:" {
        return "} else {".to_string();
    }

    // Inline else: "else: STMT" — merge with preceding inline if
    let inline_else_re = Regex::new(r"^else:\s+(.+)$").unwrap();
    if let Some(cap) = inline_else_re.captures(line.trim()) {
        let raw_stmt = cap[1].trim();
        // Handle "else: return EXPR"
        if raw_stmt.starts_with("return ") {
            let expr = raw_stmt.strip_prefix("return ").unwrap().trim();
            let val = transpile_expressions(expr, declared);
            return format!("}} else {{ return Some({val}); }}");
        }
        let stmt = transpile_expressions(raw_stmt, declared);
        return format!("}} else {{ {stmt}; }}");
    }

    line = transpile_expressions(&line, declared);

    // Add semicolons to statements
    let t = line.trim();
    let is_block_open = t.ends_with('{') && !t.contains("else");
    let is_standalone_close = t == "}";
    let is_comment = t.starts_with("//");
    let is_control_flow = (t.starts_with("if ") || t.starts_with("} else")) && t.ends_with('{');

    if !t.is_empty()
        && !is_block_open
        && !is_standalone_close
        && !is_comment
        && !is_control_flow
        && !t.ends_with(';')
    {
        // Inline if-else expressions ending with } need semicolons:
        // "let mut x = if COND { A } else { B }" → needs ";"
        line.push(';');
    }

    line
}

fn transpile_expressions(line: &str, declared: &mut std::collections::HashSet<String>) -> String {
    let mut s = line.to_string();

    // ** (power) FIRST, before any other transformations
    // Handle all combinations of simple/paren expressions around **
    // 1) (...) ** (...)
    let pow_pp = Regex::new(r"\(([^)]+)\)\s*\*\*\s*\(([^)]+)\)").unwrap();
    s = pow_pp
        .replace_all(&s, "($1 as f64).powf(($2) as f64)")
        .to_string();
    // 2) (...) ** simple
    let pow_ps = Regex::new(r"\(([^)]+)\)\s*\*\*\s*(\d+\.\d+|\w+)").unwrap();
    s = pow_ps
        .replace_all(&s, "($1 as f64).powf($2 as f64)")
        .to_string();
    // 3) simple ** (...)
    let pow_sp = Regex::new(r"(\d+\.\d+|\w+)\s*\*\*\s*\(([^)]+)\)").unwrap();
    s = pow_sp
        .replace_all(&s, "($1 as f64).powf(($2) as f64)")
        .to_string();
    // 4) simple ** simple
    let pow_ss = Regex::new(r"(\d+\.\d+|\w+)\s*\*\*\s*(\d+\.\d+|\w+)").unwrap();
    s = pow_ss
        .replace_all(&s, "($1 as f64).powf($2 as f64)")
        .to_string();

    // X in [A,B,C] → (X == A || X == B || X == C)
    let in_list_re = Regex::new(r"(\w+(?:\.\w+)*)\s+in\s*\[([^\]]+)\]").unwrap();
    s = in_list_re
        .replace_all(&s, |caps: &regex::Captures| {
            let var = &caps[1];
            let items: Vec<String> = caps[2]
                .split(',')
                .map(|i| format!("{var} == {}", i.trim()))
                .collect();
            format!("({})", items.join(" || "))
        })
        .to_string();
    // Also handle self.skill → skill for the above
    s = s.replace("self.skill ==", "skill ==");

    // int(x) → truncate toward zero (stays f64) — uses balanced paren matching
    {
        let int_pat = Regex::new(r"\bint\(").unwrap();
        let mut search_from = 0;
        loop {
            let Some(m) = int_pat.find(&s[search_from..]) else {
                break;
            };
            let abs_start = search_from + m.start();
            let open = search_from + m.end() - 1;
            let mut depth = 1;
            let mut end = open + 1;
            let bytes = s.as_bytes();
            while end < bytes.len() && depth > 0 {
                if bytes[end] == b'(' {
                    depth += 1;
                }
                if bytes[end] == b')' {
                    depth -= 1;
                }
                if depth > 0 {
                    end += 1;
                }
            }
            if depth != 0 {
                break;
            }
            let inner = &s[open + 1..end];
            let replacement = format!("({inner}).trunc()");
            s = format!("{}{replacement}{}", &s[..abs_start], &s[end + 1..]);
            search_from = abs_start + replacement.len();
        }
    }

    // Negative indexing: params[-N] → params[params.len() - N]
    let neg_idx_re =
        Regex::new(r"self\.(skill_params|talent1_params|talent2_params)\[-(\d+)\]").unwrap();
    s = neg_idx_re
        .replace_all(&s, |caps: &regex::Captures| {
            let field = match &caps[1] {
                "skill_params" => "unit.skill_parameters",
                "talent1_params" => "unit.talent1_parameters",
                "talent2_params" => "unit.talent2_parameters",
                _ => "unit.skill_parameters",
            };
            let n = &caps[2];
            format!("{field}[{field}.len() - {n}]")
        })
        .to_string();

    // skill_params[N]
    let sp_re = Regex::new(r"self\.skill_params\[(\d+)\]").unwrap();
    s = sp_re
        .replace_all(&s, "unit.skill_parameters.get($1).copied().unwrap_or(0.0)")
        .to_string();

    // talent1_params[N]
    let t1_re = Regex::new(r"self\.talent1_params\[(\d+)\]").unwrap();
    s = t1_re
        .replace_all(
            &s,
            "unit.talent1_parameters.get($1).copied().unwrap_or(0.0)",
        )
        .to_string();

    // talent2_params[N]
    let t2_re = Regex::new(r"self\.talent2_params\[(\d+)\]").unwrap();
    s = t2_re
        .replace_all(
            &s,
            "unit.talent2_parameters.get($1).copied().unwrap_or(0.0)",
        )
        .to_string();

    // max/min(self.talent_params) — BEFORE self.talent replacement
    s = s.replace(
        "max(self.talent1_params)",
        "unit.talent1_parameters.iter().copied().fold(f64::NEG_INFINITY, f64::max)",
    );
    s = s.replace(
        "min(self.talent1_params)",
        "unit.talent1_parameters.iter().copied().fold(f64::INFINITY, f64::min)",
    );
    s = s.replace(
        "max(self.talent2_params)",
        "unit.talent2_parameters.iter().copied().fold(f64::NEG_INFINITY, f64::max)",
    );
    s = s.replace(
        "min(self.talent2_params)",
        "unit.talent2_parameters.iter().copied().fold(f64::INFINITY, f64::min)",
    );

    // self.params / self.params2
    s = s.replace(
        "self.params2",
        "unit.skill_parameters.get(1).copied().unwrap_or(0.0)",
    );
    s = s.replace(
        "self.params",
        "unit.skill_parameters.get(0).copied().unwrap_or(0.0)",
    );

    // Additional field name fixes — standalone references without index access
    // These catch patterns like "self.talent1_params" used as a whole (e.g., max/min)
    // Must come AFTER the indexed regex replacements above
    if s.contains("self.talent1_params") {
        s = s.replace("self.talent1_params", "unit.talent1_parameters");
    }
    if s.contains("self.talent2_params") {
        s = s.replace("self.talent2_params", "unit.talent2_parameters");
    }
    // max/min on parameter slices — match both self.X_params and unit.X_parameters
    let param_slice_max =
        Regex::new(r"max\((?:self\.(\w+_params)|unit\.(\w+))\[:(\d+)\]\)").unwrap();
    s = param_slice_max
        .replace_all(&s, |caps: &regex::Captures| {
            let field = if let Some(m) = caps.get(1) {
                match m.as_str() {
                    "skill_params" => "unit.skill_parameters",
                    "talent1_params" => "unit.talent1_parameters",
                    "talent2_params" => "unit.talent2_parameters",
                    _ => "unit.skill_parameters",
                }
            } else {
                &format!("unit.{}", &caps[2])
            };
            let n = &caps[3];
            format!("{field}[..{n}].iter().copied().fold(f64::NEG_INFINITY, f64::max)")
        })
        .to_string();
    let param_slice_min =
        Regex::new(r"min\((?:self\.(\w+_params)|unit\.(\w+))\[(\d+):\]\)").unwrap();
    s = param_slice_min
        .replace_all(&s, |caps: &regex::Captures| {
            let field = if let Some(m) = caps.get(1) {
                match m.as_str() {
                    "skill_params" => "unit.skill_parameters",
                    "talent1_params" => "unit.talent1_parameters",
                    "talent2_params" => "unit.talent2_parameters",
                    _ => "unit.skill_parameters",
                }
            } else {
                &format!("unit.{}", &caps[2])
            };
            let n = &caps[3];
            format!("{field}[{n}..].iter().copied().fold(f64::INFINITY, f64::min)")
        })
        .to_string();

    // max/min on full parameter arrays (including skill_params)
    s = s.replace(
        "max(self.skill_params)",
        "unit.skill_parameters.iter().copied().fold(f64::NEG_INFINITY, f64::max)",
    );
    s = s.replace(
        "min(self.skill_params)",
        "unit.skill_parameters.iter().copied().fold(f64::INFINITY, f64::min)",
    );

    s = s.replace("self.pot", "(unit.potential as f64)");
    s = s.replace("self.base_name", "\"\"");
    s = s.replace("self.no_kill", "!unit.skill_damage");
    s = s.replace("self.hits", "unit.ammo");
    // Standalone self.skill_params (no index) → unit.skill_parameters
    if s.contains("self.skill_params") && !s.contains("self.skill_params[") {
        s = s.replace("self.skill_params", "unit.skill_parameters");
    }
    // Standalone skill_params without self. prefix
    if s.contains("skill_params")
        && !s.contains("unit.skill_parameters")
        && !s.contains("self.skill_params")
    {
        s = s.replace("skill_params", "unit.skill_parameters");
    }

    // Clone operator fields (must come before self.atk to avoid partial match)
    s = s.replace("self.cloned_op.atk_interval", "unit.clone_atk_interval");
    s = s.replace("self.cloned_op.atk", "unit.clone_atk");
    s = s.replace("self.cloned_op.ranged", "unit.clone_is_ranged");
    s = s.replace("self.cloned_op.physical", "unit.clone_is_physical");

    // self.X → unit.X field mappings
    s = s.replace("self.drone_atk_interval", "unit.drone_atk_interval as f64");
    s = s.replace("self.atk_interval", "atk_interval");
    s = s.replace("self.drone_atk", "unit.drone_atk");
    s = s.replace("self.attack_speed", "unit.attack_speed");
    s = s.replace("self.buff_atk_flat", "unit.buff_atk_flat");
    s = s.replace("self.buff_fragile", "unit.buff_fragile");
    s = s.replace("self.buff_atk", "unit.buff_atk");
    s = s.replace("self.atk", "unit.atk");
    s = s.replace("self.skill_cost", "unit.skill_cost as f64");
    s = s.replace("self.sp_boost", "unit.sp_boost as f64");
    s = s.replace("self.targets", "unit.targets as f64");
    s = s.replace("self.skill_dmg", "unit.skill_damage");
    s = s.replace("self.potential", "(unit.potential as f64)");
    s = s.replace("self.talent2_dmg", "unit.talent2_damage");
    s = s.replace("self.talent_dmg", "unit.talent_damage");
    s = s.replace("self.module_dmg", "unit.module_damage");
    s = s.replace("self.trait_dmg", "unit.trait_damage");
    // Context-aware self.skill replacement:
    // Comparison context (== != < > <= >= in): use `skill` (i32)
    // Everything else (arithmetic, function args): use `skillf` (f64)
    // Match "self.skill" followed by a non-alphanumeric, non-underscore char (captured)
    let self_skill_re = Regex::new(r"self\.skill([^a-zA-Z_\d])").unwrap();
    {
        let input = s.clone();
        s = self_skill_re.replace_all(&input, |caps: &regex::Captures| {
            let trailing = &caps[1];
            let m = caps.get(0).unwrap();
            let after = &input[m.end()..];
            let is_comparison = trailing == "=" // self.skill == or self.skill =
                || trailing == "!" // before !=
                || trailing == "<"
                || trailing == ">"
                || (trailing == " " && (
                    after.trim_start().starts_with("==")
                    || after.trim_start().starts_with("!=")
                    || after.trim_start().starts_with("<=")
                    || after.trim_start().starts_with(">=")
                    || (after.trim_start().starts_with('<') && !after.trim_start().starts_with("<<"))
                    || (after.trim_start().starts_with('>') && !after.trim_start().starts_with(">>"))
                    || after.trim_start().starts_with("in ")
                ));
            if is_comparison {
                format!("skill{trailing}")
            } else {
                format!("skillf{trailing}")
            }
        }).to_string();
    }
    // Handle self.skill at end of line
    if s.ends_with("self.skill") {
        s = s[..s.len() - 10].to_string() + "skillf";
    }
    // Cast i32 fields to f64 for arithmetic
    s = s.replace("self.elite", "(unit.elite as f64)");
    s = s.replace("self.module_lvl", "(unit.module_level as f64)");
    s = s.replace("self.module", "unit.module_index");
    s = s.replace("self.ammo", "unit.ammo");
    s = s.replace("self.shadows", "unit.shadows");

    // Additional operator property mappings
    s = s.replace("self.below50", "unit.talent2_damage");
    s = s.replace("self.physical", "true");
    s = s.replace("self.ranged", "true");
    s = s.replace("self.freezeRate", "0.0");
    s = s.replace("self.count", "unit.targets as f64");
    s = s.replace("self.duration", "unit.skill_duration");
    s = s.replace("self.sp_cost", "unit.skill_cost as f64");
    s = s.replace("self.shadows", "unit.targets as f64");

    // self.shreds[N] → unit.shreds[N]
    let shreds_re = Regex::new(r"self\.shreds\[(\d+)\]").unwrap();
    s = shreds_re.replace_all(&s, "unit.shreds[$1]").to_string();

    // User-defined array indexing: VARNAME[expr] → VARNAME[(expr) as usize]
    // Only for user variables (lowercase start), NOT unit.X or self.X
    // User-defined array indexing: arr[expr] → arr[(expr) as usize]
    // Skip: unit.X[N], self.X[N], vec![N], range indices [N..M]
    let user_arr_idx_re = Regex::new(r"\b([a-z_]\w*)\[([^\]]+)\]").unwrap();
    s = user_arr_idx_re
        .replace_all(&s, |caps: &regex::Captures| {
            let var = &caps[1];
            let idx = &caps[2];
            if var == "unit" || var == "self" || var == "vec"
            || var.starts_with("skill_param") || var.starts_with("talent")
            || idx.contains("..") // range syntax
            || idx.chars().all(|c| c.is_ascii_digit())
            // literal integer
            {
                return format!("{var}[{idx}]");
            }
            format!("{var}[({idx}) as usize]")
        })
        .to_string();

    // True/False
    s = s.replace(" True", " true");
    s = s.replace(" False", " false");
    s = s.replace("(True", "(true");
    s = s.replace("(False", "(false");

    // and/or/not
    s = s.replace(" and ", " && ");
    s = s.replace(" or ", " || ");
    // "not X == Y" → "X != Y" / "not X" → "!X"
    let not_eq_re = Regex::new(r"\bnot\s+(\S+)\s*==\s*(\S+)").unwrap();
    s = not_eq_re.replace_all(&s, "$1 != $2").to_string();
    let not_re = Regex::new(r"\bnot\s+").unwrap();
    s = not_re.replace_all(&s, "!").to_string();

    // // → floor division (approximate)
    s = s.replace("//", "/ ");

    // Standalone ternary (not assignment): "EXPR if COND else ALT" in expression
    // Must come BEFORE fmax/min/max so ternaries inside function args get converted first
    // Match: capture everything before " if " that isn't an assignment
    let expr_ternary_re = Regex::new(r"(.+)\s+if\s+(.+?)\s+else\s+(.+)").unwrap();
    if !s.contains("let ")
        && !s.contains('{')
        && s.contains(" if ")
        && s.contains(" else ")
        && let Some(cap) = expr_ternary_re.captures(&s.clone())
    {
        let before = cap[1].trim();
        let cond = cap[2].trim();
        let alt = cap[3].trim();
        // Only apply if "before" doesn't contain "=" (not an assignment context)
        // and doesn't already have braces
        if !before.contains('=') && !before.contains('{') {
            s = format!("if {cond} {{ {before} }} else {{ {alt} }}");
        }
    }

    // np.fmax(a, b) → (a).max(b), np.fmin(a, b) → (a).min(b)
    // Use balanced paren matching to handle nested expressions
    for func in &["np.fmax", "np.fmin"] {
        let method = if *func == "np.fmax" { "max" } else { "min" };
        while let Some(pos) = s.find(func) {
            let open = pos + func.len();
            if open >= s.len() || s.as_bytes()[open] != b'(' {
                break;
            }
            // Find matching closing paren
            let mut depth = 1;
            let mut end = open + 1;
            let bytes = s.as_bytes();
            while end < bytes.len() && depth > 0 {
                if bytes[end] == b'(' {
                    depth += 1;
                }
                if bytes[end] == b')' {
                    depth -= 1;
                }
                if depth > 0 {
                    end += 1;
                }
            }
            if depth != 0 {
                break;
            }
            // Find the comma separating the two args (at depth 1)
            let inner = &s[open + 1..end];
            let mut comma_pos = None;
            let mut d = 0;
            for (i, c) in inner.chars().enumerate() {
                if c == '(' {
                    d += 1;
                }
                if c == ')' {
                    d -= 1;
                }
                if c == ',' && d == 0 {
                    comma_pos = Some(i);
                    break;
                }
            }
            if let Some(cp) = comma_pos {
                let arg1 = inner[..cp].trim();
                let arg2 = inner[cp + 1..].trim();
                s = format!("{}({arg1}).{method}({arg2}){}", &s[..pos], &s[end + 1..]);
            } else {
                break;
            }
        }
    }

    // Python min/max two-arg: min(a, b) → (a).min(b)
    // Use balanced paren matching to handle nested expressions
    for func in &["min", "max"] {
        let method = *func;
        let mut search_from = 0;
        loop {
            let pattern = format!("\\b{func}\\(");
            let re = Regex::new(&pattern).unwrap();
            let Some(m) = re.find(&s[search_from..]) else {
                break;
            };
            let abs_start = search_from + m.start();
            let open = search_from + m.end() - 1;
            // Find matching closing paren
            let mut depth = 1;
            let mut end = open + 1;
            let bytes = s.as_bytes();
            while end < bytes.len() && depth > 0 {
                if bytes[end] == b'(' {
                    depth += 1;
                }
                if bytes[end] == b')' {
                    depth -= 1;
                }
                if depth > 0 {
                    end += 1;
                }
            }
            if depth != 0 {
                break;
            }
            let inner = s[open + 1..end].to_string();
            // Find comma at depth 0
            let mut comma_pos = None;
            let mut d = 0;
            for (i, c) in inner.chars().enumerate() {
                if c == '(' {
                    d += 1;
                }
                if c == ')' {
                    d -= 1;
                }
                if c == ',' && d == 0 {
                    comma_pos = Some(i);
                    break;
                }
            }
            if let Some(cp) = comma_pos {
                let arg1 = inner[..cp].trim();
                let arg2 = inner[cp + 1..].trim();
                let replacement = format!("({arg1}).{method}({arg2})");
                s = format!("{}{replacement}{}", &s[..abs_start], &s[end + 1..]);
                search_from = abs_start + replacement.len();
            } else {
                // Single-arg: skip past this occurrence and continue
                search_from = end + 1;
            }
        }
    }

    // Handle inline ternary: "VAR = A if COND else B"
    let ternary_re =
        Regex::new(r"^(\s*(?:let\s+mut\s+)?\w+\s*=\s*)(.+?)\s+if\s+(.+?)\s+else\s+(.+)$").unwrap();
    if let Some(cap) = ternary_re.captures(&s.clone()) {
        let assign = &cap[1];
        let true_val = &cap[2];
        let cond = &cap[3];
        let false_val = &cap[4];
        s = format!("{assign}if {cond} {{ {true_val} }} else {{ {false_val} }}");
    }

    // Augmented assignment ternary: "VAR *= A if COND else B"
    let aug_ternary_re =
        Regex::new(r"^(\s*)(\w+)\s*(\*=|\+=|-=|/=)\s*(.+?)\s+if\s+(.+?)\s+else\s+(.+)$").unwrap();
    if let Some(cap) = aug_ternary_re.captures(&s.clone()) {
        let indent = &cap[1];
        let var = escape_keyword(&cap[2]);
        let op = &cap[3];
        let true_val = &cap[4];
        let cond = &cap[5];
        let false_val = &cap[6];
        s = format!("{indent}{var} {op} if {cond} {{ {true_val} }} else {{ {false_val} }}");
    }

    // Fix double assignment: "x = x = expr" → "x = expr" (Python idiom)
    let double_assign = Regex::new(r"^(\s*)(\w+)\s*=\s*(\w+)\s*=\s*(.+)$").unwrap();
    if let Some(cap) = double_assign.captures(&s.clone())
        && cap[2] == cap[3]
    {
        s = format!("{}{} = {}", &cap[1], &cap[2], &cap[4]);
    }

    // Assignments
    let assign_re = Regex::new(r"^(\s*)(\w+)\s*=\s*(.+)$").unwrap();
    if let Some(cap) = assign_re.captures(&s.clone()) {
        let indent = &cap[1];
        let var = &cap[2];
        let val = &cap[3];

        let safe_var = escape_keyword(var);

        // Don't add `let` for pre-declared variables or dps/skill/defense/res
        if declared.insert(safe_var.clone()) {
            s = format!("{indent}let mut {safe_var} = {val}");
        } else {
            s = format!("{indent}{safe_var} = {val}");
        }
    }

    // Augmented assignments
    let aug_re = Regex::new(r"^(\s*)(\w+)\s*(\+=|-=|\*=|/=)\s*(.+)$").unwrap();
    if let Some(cap) = aug_re.captures(&s.clone()) {
        let indent = &cap[1];
        let var = escape_keyword(&cap[2]);
        let op = &cap[3];
        let val = &cap[4];
        s = format!("{indent}{var} {op} {val}");
    }

    // Escape Rust keywords used as variable names
    for &(kw, replacement) in RUST_KEYWORDS {
        if kw == "mut"
            || kw == "pub"
            || kw == "fn"
            || kw == "use"
            || kw == "impl"
            || kw == "struct"
            || kw == "enum"
            || kw == "const"
            || kw == "static"
            || kw == "return"
            || kw == "unsafe"
            || kw == "extern"
        {
            // These are Rust syntax keywords — only replace if used as a variable name
            // (i.e., in assignment LHS or standalone expression, not after `let`)
            continue;
        }
        let kw_re = Regex::new(&format!(r"\b{kw}\b")).unwrap();
        if s.contains(kw) && !s.contains(&format!("\"{kw}\"")) && !s.contains(&format!("unit.{kw}"))
        {
            s = kw_re.replace_all(&s, replacement).to_string();
        }
    }

    // Clean up artifacts
    s = s.replace(":;", " {");
    s = s.replace(";;", ";");

    s
}

fn escape_keyword(var: &str) -> String {
    for &(kw, replacement) in RUST_KEYWORDS {
        if var == kw {
            return replacement.to_string();
        }
    }
    var.to_string()
}

// ── Helpers ─────────────────────────────────────────────────────────────

fn extract_method(class_body: &str, method_name: &str) -> String {
    let pattern = format!("def {method_name}(");
    let mut body = String::new();
    let mut found = false;
    let mut base_indent = 0;
    let mut body_base_indent = 0;
    let mut first_body_line = true;

    for line in class_body.lines() {
        if line.trim_start().starts_with(&pattern) {
            found = true;
            base_indent = line.len() - line.trim_start().len();
            continue;
        }
        if found {
            let trimmed = line.trim_start();
            if trimmed.is_empty() {
                body.push('\n');
                continue;
            }
            let indent = line.len() - trimmed.len();
            if indent <= base_indent && !trimmed.is_empty() {
                break;
            }
            // Set body base indent from first non-empty content line
            if first_body_line {
                body_base_indent = indent;
                first_body_line = false;
            }
            // Relative indent: each level above body base gets one tab
            let rel = indent.saturating_sub(body_base_indent);
            for _ in 0..rel {
                body.push('\t');
            }
            body.push_str(trimmed);
            body.push('\n');
        }
    }
    body
}

fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            // Don't add underscore between consecutive uppercase (e.g., "DPS" → "dps")
            let prev = name.chars().nth(i - 1).unwrap_or('_');
            let next = name.chars().nth(i + 1);
            if prev.is_lowercase() || (next.is_some() && next.unwrap().is_lowercase()) {
                result.push('_');
            }
        }
        result.push(c.to_ascii_lowercase());
    }
    // Handle names starting with numbers (e.g., "12F" → "op_12f")
    if result.starts_with(|c: char| c.is_ascii_digit()) {
        result = format!("op_{result}");
    }
    result
}
