//! `cargo run -p xtask -- check-layers`
//!
//! Reads `[workspace.metadata.layers]` and `[workspace.metadata.layer-rules]` from
//! the workspace manifest (through `cargo metadata`) and fails when the dependency
//! graph breaks one of these rules:
//!
//! 1. Every workspace member is in exactly one layer, or listed under `tools`.
//! 2. A layered crate's normal and build dependencies on other layered crates point
//!    at the same level or lower, never higher.
//! 3. A `dev-only` crate appears only under `[dev-dependencies]`.
//! 4. A `pure` crate reaches no `pure-forbidden` crate through its normal
//!    dependencies, directly or transitively (dev-dependencies do not count).
//!
//! Same-level edges are allowed (engine-dps on dps-formulas, both level 3) and
//! printed, so a new one is visible in review without failing the run.

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::process::{Command, ExitCode};

use serde_json::Value;

fn main() -> ExitCode {
    if std::env::args().nth(1).as_deref() == Some("check-layers") {
        check_layers()
    } else {
        eprintln!("usage: cargo run -p xtask -- check-layers");
        ExitCode::FAILURE
    }
}

fn glob_match(pattern: &str, name: &str) -> bool {
    pattern
        .strip_suffix('*')
        .map_or_else(|| pattern == name, |prefix| name.starts_with(prefix))
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

#[allow(clippy::too_many_lines)]
fn check_layers() -> ExitCode {
    let out = match Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["metadata", "--format-version", "1"])
        .output()
    {
        Ok(o) if o.status.success() => o.stdout,
        Ok(o) => {
            eprintln!(
                "cargo metadata failed: {}",
                String::from_utf8_lossy(&o.stderr)
            );
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("cargo metadata: {e}");
            return ExitCode::FAILURE;
        }
    };
    let meta: Value = match serde_json::from_slice(&out) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("cargo metadata output: {e}");
            return ExitCode::FAILURE;
        }
    };

    let ws_meta = &meta["metadata"];
    let mut level_of: HashMap<String, i64> = HashMap::new();
    for layer in ws_meta["layers"].as_array().into_iter().flatten() {
        let level = layer["level"].as_i64().unwrap_or(-1);
        for c in strings(&layer["crates"]) {
            level_of.insert(c, level);
        }
    }
    let rules = &ws_meta["layer-rules"];
    let tools = strings(&rules["tools"]);
    let dev_only = strings(&rules["dev-only"]);
    let pure = strings(&rules["pure"]);
    let forbidden = strings(&rules["pure-forbidden"]);

    // id -> name, and the packages by name
    let mut name_of: HashMap<String, String> = HashMap::new();
    let mut pkg_by_name: HashMap<String, &Value> = HashMap::new();
    for p in meta["packages"].as_array().into_iter().flatten() {
        let id = p["id"].as_str().unwrap_or_default().to_owned();
        let name = p["name"].as_str().unwrap_or_default().to_owned();
        name_of.insert(id, name.clone());
        pkg_by_name.entry(name).or_insert(p);
    }
    let members: BTreeSet<String> = strings(&meta["workspace_members"])
        .into_iter()
        .filter_map(|id| name_of.get(&id).cloned())
        .collect();

    let mut errors: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    // Rule 1
    for m in &members {
        let in_layer = level_of.contains_key(m);
        let is_tool = tools.iter().any(|t| glob_match(t, m));
        if !in_layer && !is_tool {
            errors.push(format!("rule 1: {m} is in no layer and not a tool"));
        }
    }
    let mut undeclared: Vec<&String> = level_of.keys().filter(|c| !members.contains(*c)).collect();
    undeclared.sort();
    for c in undeclared {
        notes.push(format!(
            "{c} is declared in a layer but is not a workspace member yet"
        ));
    }

    // Rules 2 and 3, from each member's declared dependencies
    for m in &members {
        let Some(pkg) = pkg_by_name.get(m) else {
            continue;
        };
        for dep in pkg["dependencies"].as_array().into_iter().flatten() {
            let dname = dep["name"].as_str().unwrap_or_default();
            let kind = dep["kind"].as_str().unwrap_or("normal");
            if dev_only.iter().any(|d| glob_match(d, dname)) && kind != "dev" {
                errors.push(format!(
                    "rule 3: {m} depends on dev-only {dname} as a {kind} dependency"
                ));
            }
            if kind == "dev" {
                continue;
            }
            if let (Some(&from), Some(&to)) = (level_of.get(m), level_of.get(dname)) {
                if to > from {
                    errors.push(format!(
                        "rule 2: {m} (level {from}) depends upward on {dname} (level {to})"
                    ));
                } else if to == from {
                    notes.push(format!("same-level edge: {m} -> {dname} (level {from})"));
                }
            }
        }
    }

    // Rule 4, over the resolved graph, normal and build edges only
    let mut edges: HashMap<String, Vec<String>> = HashMap::new();
    for node in meta["resolve"]["nodes"].as_array().into_iter().flatten() {
        let id = node["id"].as_str().unwrap_or_default();
        let from = name_of.get(id).cloned().unwrap_or_default();
        for d in node["deps"].as_array().into_iter().flatten() {
            let runtime = d["dep_kinds"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|k| k["kind"].is_null() || k["kind"].as_str() == Some("build"));
            if runtime && let Some(to) = d["pkg"].as_str().and_then(|p| name_of.get(p)) {
                edges.entry(from.clone()).or_default().push(to.clone());
            }
        }
    }
    for m in &members {
        if !pure.iter().any(|p| glob_match(p, m)) {
            continue;
        }
        // BFS with parents, so the report shows the path
        let mut parent: BTreeMap<String, String> = BTreeMap::new();
        let mut queue = VecDeque::from([m.clone()]);
        let mut seen = BTreeSet::from([m.clone()]);
        while let Some(cur) = queue.pop_front() {
            if cur != *m && forbidden.iter().any(|f| glob_match(f, &cur)) {
                let mut path = vec![cur.clone()];
                let mut at = cur.clone();
                while let Some(p) = parent.get(&at) {
                    path.push(p.clone());
                    at = p.clone();
                }
                path.reverse();
                errors.push(format!(
                    "rule 4: pure {m} reaches {cur}: {}",
                    path.join(" -> ")
                ));
                continue;
            }
            for next in edges.get(&cur).into_iter().flatten() {
                if seen.insert(next.clone()) {
                    parent.insert(next.clone(), cur.clone());
                    queue.push_back(next.clone());
                }
            }
        }
    }

    for n in &notes {
        println!("note: {n}");
    }
    if errors.is_empty() {
        println!("check-layers: {} members, 0 violations", members.len());
        ExitCode::SUCCESS
    } else {
        for e in &errors {
            eprintln!("error: {e}");
        }
        eprintln!("check-layers: {} violation(s)", errors.len());
        ExitCode::FAILURE
    }
}
