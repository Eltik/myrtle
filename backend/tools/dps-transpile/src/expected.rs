use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};

use crate::formulas::OperatorFormula;
use crate::hand::{python_reference_corrections, write_output};
use crate::paths::{HEAL_FALLBACKS, Mode};

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

#[derive(Serialize)]
struct HpsTestCase {
    key: String,
    operator: String,
    skill: i32,
    module: i32,
}

#[derive(Deserialize, Serialize)]
struct HpsExpected {
    skill_hps: i64,
    base_hps: i64,
    avg_hps: i64,
}

fn load_equip_keys(path: &Path) -> HashSet<String> {
    let raw = fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
    let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap_or_default();
    let mut keys = HashSet::new();
    if let Some(equips) = parsed.get("Equips").and_then(|v| v.as_array()) {
        for entry in equips {
            if let Some(key) = entry.get("key").and_then(|k| k.as_str()) {
                keys.insert(key.to_string());
            }
        }
    }
    keys
}

fn module_exists_in_game_data(
    equip_keys: &HashSet<String>,
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

pub fn generate_expected_dps(
    repo_path: &Path,
    formulas: &HashMap<String, OperatorFormula>,
    python: &str,
    excel_dir: &Path,
) -> anyhow::Result<()> {
    let out_path = Mode::Damage.expected_json_path();
    println!("\n=== Generating {} ===", out_path.display());

    let equip_keys = load_equip_keys(&excel_dir.join("battle_equip_table.json"));
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
    let mut seen = HashSet::new();
    let mut skipped_modules = 0u64;

    for (char_id, formula) in formulas {
        let all_modules_exist = formula.available_modules.iter().all(|&m| {
            module_exists_in_game_data(&equip_keys, char_id, &formula.available_modules, m)
        });

        for &skill in &formula.available_skills {
            let modules: Vec<i32> = std::iter::once(0)
                .chain(formula.available_modules.iter().copied())
                .collect();
            for &module in &modules {
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
        "Generated {} test cases ({} module combos skipped - not in game data)",
        cases.len(),
        skipped_modules
    );

    let repo_path = repo_path.display().to_string();
    let corrections = python_reference_corrections(&repo_path);
    let py_script = format!(
        r#"#!/usr/bin/env python3
import sys, json, os
os.chdir("{repo_path}")
sys.path.insert(0, ".")
{corrections}
from damagecalc.damage_formulas import *
from damagecalc.utils import PlotParameters
from collections import defaultdict

cases = json.load(sys.stdin)

# Five operators read `shreds` at build time to order a flat DEF/RES change against it
grouped = defaultdict(list)
for tc in cases:
    grouped[(tc['operator'], tc['skill'], tc['module'], tc['def_shred_mult'], tc['def_shred_flat'], tc['res_shred_mult'], tc['res_shred_flat'])].append(tc)

results = {{}}
for (op_name, skill, module, dm, df, rm, rf), tcs in grouped.items():
    try:
        pp = PlotParameters(skill=skill, module=module)
        op = globals()[op_name](pp, shreds=[dm, df, rm, rf])
    except Exception:
        continue
    for tc in tcs:
        d = max(0, (tc['defense'] - tc['def_shred_flat'])) * tc['def_shred_mult']
        r = max(0, (tc['res'] - tc['res_shred_flat'])) * tc['res_shred_mult']
        try:
            dps = float(op.skill_dps(d, r)) * (1 + tc['fragile'])
            results[tc['key']] = dps
        except Exception:
            pass

json.dump(results, sys.stdout)
"#
    );

    println!("Running Python DPS calculator...");
    let stdout = run_python(python, "myrtle_dps_calc.py", &py_script, &cases)?;
    let results: HashMap<String, f64> =
        serde_json::from_slice(&stdout).context("parse Python output")?;
    println!("Received {} DPS values from Python", results.len());

    let sorted: BTreeMap<&String, _> = results.iter().collect();
    write_output(&out_path, serde_json::to_string_pretty(&sorted)?.as_bytes())?;
    println!("Written to {}", out_path.display());
    Ok(())
}

#[allow(clippy::too_many_arguments, clippy::float_cmp)]
pub fn make_test_key(
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
    let base = format!("{name}_s{skill}_m{module}_{def:.0}_{res:.0}");
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

fn make_hps_test_key(name: &str, skill: i32, module: i32) -> String {
    format!("{name}_s{skill}_m{module}")
}

pub fn generate_expected_hps(
    repo_path: &Path,
    formulas: &HashMap<String, OperatorFormula>,
    python: &str,
) -> anyhow::Result<()> {
    let out_path = Mode::Healing.expected_json_path();
    println!("\n=== Generating {} (HPS) ===", out_path.display());

    let mut cases: Vec<HpsTestCase> = Vec::new();
    let mut seen = HashSet::new();

    for formula in formulas.values() {
        if HEAL_FALLBACKS.contains(&formula.class_name.as_str()) {
            continue;
        }
        let modules: Vec<i32> = std::iter::once(0)
            .chain(formula.available_modules.iter().copied())
            .collect();
        for &skill in &formula.available_skills {
            for &module in &modules {
                let key = make_hps_test_key(&formula.class_name, skill, module);
                if seen.insert(key.clone()) {
                    cases.push(HpsTestCase {
                        key,
                        operator: formula.class_name.clone(),
                        skill,
                        module,
                    });
                }
            }
        }
    }

    println!("Generated {} test cases", cases.len());

    let repo_path = repo_path.display().to_string();
    let corrections = python_reference_corrections(&repo_path);
    let py_script = format!(
        r#"#!/usr/bin/env python3
import sys, json, os, io, contextlib
os.chdir("{repo_path}")
sys.path.insert(0, ".")
# Silence stray prints from upstream code so only the final JSON reaches stdout.
_real_stdout = sys.stdout
sys.stdout = io.StringIO()
{corrections}
from damagecalc.healing_formulas import *
from damagecalc.utils import PlotParameters

cases = json.load(sys.stdin)

# Healers with required kwargs - pass safe defaults so they don't KeyError.
# Operators that derive a default in __init__ are omitted so Python uses that
# default (the Rust engine mirrors it).
KWARGS_BY_OPERATOR = {{
    "Blemishine": {{"hits": 0}},
    "Hung": {{"hits": 0}},
}}

# Canonical local names + accepted aliases. Some healers use single-word
# variants (skillhps, avghps) or domain-specific synonyms.
SKILL_KEYS = ("skill_hps", "skillhps")
BASE_KEYS  = ("base_hps", "skilldown_hps")
AVG_KEYS   = ("avg_hps", "avghps")

def _pick(local_dict, keys):
    for k in keys:
        if k in local_dict:
            try:
                return float(local_dict[k])
            except (TypeError, ValueError):
                continue
    return 0.0

class LocalsCapture:
    "Capture the locals of skill_hps() at the moment it returns."
    def __init__(self):
        self.locals = {{}}
        self.target_code = None
    def install(self, method_code):
        self.locals = {{}}
        self.target_code = method_code
        sys.settrace(self._trace)
    def uninstall(self):
        sys.settrace(None)
    def _trace(self, frame, event, arg):
        # Restrict to the target function; return self to keep tracing inside it.
        if event == 'call':
            if frame.f_code is self.target_code:
                return self._trace
            return None
        if event == 'return' and frame.f_code is self.target_code:
            self.locals = dict(frame.f_locals)
        return self._trace

results = {{}}
for tc in cases:
    op_name = tc['operator']
    if op_name not in globals():
        continue
    cap = LocalsCapture()
    try:
        pp = PlotParameters(skill=tc['skill'], module=tc['module'])
        kwargs = KWARGS_BY_OPERATOR.get(op_name, {{}})
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            op = globals()[op_name](pp, **kwargs)
            cap.install(op.skill_hps.__func__.__code__)
            try:
                op.skill_hps(**kwargs)
            finally:
                cap.uninstall()
    except Exception:
        cap.uninstall()
        continue
    locs = cap.locals
    skill_h = int(_pick(locs, SKILL_KEYS))
    base_h  = int(_pick(locs, BASE_KEYS))
    avg_h   = int(_pick(locs, AVG_KEYS))
    results[tc['key']] = {{"skill_hps": skill_h, "base_hps": base_h, "avg_hps": avg_h}}

# Restore real stdout and write the JSON.
sys.stdout = _real_stdout
json.dump(results, sys.stdout)
"#
    );

    println!("Running Python HPS calculator...");
    let stdout = run_python(python, "myrtle_hps_calc.py", &py_script, &cases)?;
    let results: HashMap<String, HpsExpected> =
        serde_json::from_slice(&stdout).context("parse Python output")?;
    println!("Received {} HPS triples from Python", results.len());

    let sorted: BTreeMap<&String, _> = results.iter().collect();
    write_output(&out_path, serde_json::to_string_pretty(&sorted)?.as_bytes())?;
    println!("Written to {}", out_path.display());
    Ok(())
}

fn run_python(
    python: &str,
    script_name: &str,
    script: &str,
    cases: &impl Serialize,
) -> anyhow::Result<Vec<u8>> {
    let tmp_script = std::env::temp_dir().join(script_name);
    fs::write(&tmp_script, script).context("write temp Python script")?;
    let cases_json = serde_json::to_string(cases)?;
    let output = Command::new(python)
        .arg(&tmp_script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(cases_json.as_bytes())?;
            }
            child.wait_with_output()
        });
    let _ = fs::remove_file(&tmp_script);
    let output = output.with_context(|| {
        format!("run Python ({python}); needs 3.10+ with upstream's dependencies, set --python or [tools] python_bin")
    })?;
    if !output.status.success() {
        bail!(
            "Python failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(output.stdout)
}
