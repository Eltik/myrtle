//! `check-layers`: the layer table in the root `Cargo.toml`, enforced from `cargo metadata`

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::process::Command;

use serde_json::Value;

pub struct Report {
    pub crates: usize,
    pub edges: usize,
    pub violations: Vec<String>,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for violation in &self.violations {
            writeln!(f, "{violation}")?;
        }
        let verdict = if self.violations.is_empty() {
            "OK"
        } else {
            "FAILED"
        };
        write!(
            f,
            "check-layers: {verdict}, {} crates, {} internal edges, {} violation(s)",
            self.crates,
            self.edges,
            self.violations.len()
        )
    }
}

/// `--no-deps` skips lockfile resolution: only the members' declared dependencies matter
pub fn metadata(manifest: &str) -> Result<Value, String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let output = Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .args(["--manifest-path", manifest])
        .output()
        .map_err(|e| format!("could not run cargo metadata: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|e| format!("bad cargo metadata JSON: {e}"))
}

pub fn check(metadata: &Value) -> Result<Report, String> {
    let member_ids: BTreeSet<&str> = strs(&metadata["workspace_members"]).collect();
    let members: BTreeMap<&str, &Value> = list(&metadata["packages"])
        .filter(|p| p["id"].as_str().is_some_and(|id| member_ids.contains(id)))
        .filter_map(|p| Some((p["name"].as_str()?, &p["dependencies"])))
        .collect();

    let table = &metadata["metadata"];
    let rules = &table["layer-rules"];
    let layers = table["layers"]
        .as_array()
        .ok_or("Cargo.toml has no [[workspace.metadata.layers]] table")?;
    let tools: BTreeSet<&str> = strs(&rules["tools"]).collect();
    let dev_only: BTreeSet<&str> = strs(&rules["dev-only"]).collect();
    let pure: Vec<&str> = strs(&rules["pure"]).collect();
    let forbidden: BTreeSet<&str> = strs(&rules["pure-forbidden"]).collect();

    let mut violations = Vec::new();
    let mut level: BTreeMap<&str, u64> = BTreeMap::new();
    for layer in layers {
        let lvl = layer["level"]
            .as_u64()
            .ok_or("a layer has no integer `level`")?;
        for krate in strs(&layer["crates"]) {
            if let Some(previous) = level.insert(krate, lvl) {
                violations.push(format!("DUPLICATE  {krate} is in L{previous} and L{lvl}"));
            }
        }
    }
    for tool in &tools {
        if level.contains_key(tool) {
            violations.push(format!(
                "DUPLICATE  {tool} is in a layer and in layer-rules.tools"
            ));
        }
    }
    for named in level.keys().chain(&tools) {
        if !members.contains_key(named) {
            violations.push(format!(
                "STALE      {named} is in the layer table but is not a workspace member"
            ));
        }
    }
    for member in members.keys() {
        if !level.contains_key(member) && !tools.contains(member) {
            violations.push(format!(
                "UNLAYERED  {member} is in no layer and not a tool (add it to [workspace.metadata.layers])"
            ));
        }
    }

    let mut edges = 0;
    for (from, deps) in &members {
        let is_pure = pure.iter().any(|pattern| glob_match(pattern, from));
        for dep in list(deps) {
            let Some(to) = dep["name"].as_str() else {
                continue;
            };
            let kind = dep["kind"].as_str().unwrap_or("normal");
            let dev = kind == "dev";
            if is_pure && !dev && forbidden.contains(&to) {
                violations.push(format!(
                    "IMPURE     {from} -> {to} [{kind}]: pure crates take values and return values"
                ));
            }
            if !members.contains_key(&to) {
                continue;
            }
            edges += 1;
            let Some(&from_level) = level.get(from) else {
                continue; // tools may link anything
            };
            if tools.contains(&to) {
                if !(dev && dev_only.contains(&to)) {
                    violations.push(format!(
                        "TOOL       {from} -> {to} [{kind}]: tools are never linked into a layered crate"
                    ));
                }
            } else if let Some(&to_level) = level.get(&to)
                && to_level > from_level
            {
                violations.push(format!(
                    "UPWARD     {from} (L{from_level}) -> {to} (L{to_level}) [{kind}]: a crate may depend only on its own layer or lower"
                ));
            }
        }
    }

    Ok(Report {
        crates: members.len(),
        edges,
        violations,
    })
}

fn list(value: &Value) -> impl Iterator<Item = &Value> {
    value.as_array().into_iter().flatten()
}

fn strs(value: &Value) -> impl Iterator<Item = &str> {
    list(value).filter_map(Value::as_str)
}

/// A trailing `*` matches any suffix, anything else is exact
fn glob_match(pattern: &str, name: &str) -> bool {
    pattern
        .strip_suffix('*')
        .map_or_else(|| pattern == name, |prefix| name.starts_with(prefix))
}
