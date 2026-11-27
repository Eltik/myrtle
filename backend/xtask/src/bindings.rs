//! `gen-ts`: the frontend's TypeScript bindings, exported by ts-rs from every crate that derives `TS`

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use crate::{WORKSPACE_MANIFEST, layers};

const EXPORT_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../target/ts-bindings");
const FRONTEND: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../frontend");
pub const GENERATED: &str = "src/types/generated";

pub fn exporting_crates(metadata: &Value) -> Vec<&str> {
    let depends_on_ts_rs = |package: &&Value| {
        package["dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|dep| dep["name"] == "ts-rs")
    };
    metadata["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(depends_on_ts_rs)
        .filter_map(|package| package["name"].as_str())
        .collect()
}

/// Types the wire cannot produce: `serde_json` writes an `i64` as a number, and an `any` is a `Value` that lost `JsonValue`
pub fn impossible_types(binding: &str) -> Vec<&'static str> {
    let mut code = String::with_capacity(binding.len());
    let mut rest = binding;
    while let Some(start) = rest.find("/*") {
        code.push_str(&rest[..start]);
        rest = rest[start..]
            .find("*/")
            .map_or("", |end| &rest[start + end + 2..]);
    }
    code.push_str(rest);
    let words: BTreeSet<&str> = code
        .lines()
        .map(|line| line.split("//").next().unwrap_or(line))
        .flat_map(|line| line.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$')))
        .collect();
    ["bigint", "any"]
        .into_iter()
        .filter(|token| words.contains(token))
        .collect()
}

pub fn files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut found = Vec::new();
    let mut queue = vec![dir.to_path_buf()];
    while let Some(next) = queue.pop() {
        for entry in fs::read_dir(&next).map_err(|e| format!("{}: {e}", next.display()))? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.is_dir() {
                queue.push(path);
            } else if path.extension().is_some_and(|ext| ext == "ts") {
                found.push(path);
            }
        }
    }
    found.sort();
    Ok(found)
}

fn run(command: &mut Command) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|e| format!("could not run {command:?}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{command:?} failed with {status}"))
    }
}

/// With `check`, fails when the regenerated bindings differ from what git has
pub fn generate(check: bool) -> Result<String, String> {
    let metadata = layers::metadata(WORKSPACE_MANIFEST)?;
    let crates = exporting_crates(&metadata);
    let export = Path::new(EXPORT_DIR);
    if export.exists() {
        fs::remove_dir_all(export).map_err(|e| format!("{EXPORT_DIR}: {e}"))?;
    }

    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let mut export_tests = Command::new(cargo);
    export_tests
        .args(["test", "--quiet", "--lib"])
        .args(["--manifest-path", WORKSPACE_MANIFEST])
        .args(crates.iter().flat_map(|name| ["-p", name]))
        .arg("export_bindings")
        .env("TS_RS_EXPORT_DIR", EXPORT_DIR);
    run(&mut export_tests)?;

    let exported = files(export)?;
    let mut offenders = Vec::new();
    let mut kept = BTreeSet::new();
    let generated = Path::new(FRONTEND).join(GENERATED);
    for path in &exported {
        let binding = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let relative = path.strip_prefix(export).map_err(|e| e.to_string())?;
        for token in impossible_types(&binding) {
            offenders.push(format!("{}: `{token}`", relative.display()));
        }
        let target = generated.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        fs::write(&target, binding).map_err(|e| format!("{}: {e}", target.display()))?;
        kept.insert(target);
    }
    if !offenders.is_empty() {
        return Err(format!(
            "bindings declare a type the wire cannot produce (annotate the field with #[ts(type = \"number\")]):\n  {}",
            offenders.join("\n  ")
        ));
    }
    for stale in files(&generated)?
        .iter()
        .filter(|path| !kept.contains(*path))
    {
        fs::remove_file(stale).map_err(|e| format!("{}: {e}", stale.display()))?;
    }

    run(Command::new("bunx")
        .args(["--bun", "biome", "check", "--write", GENERATED])
        .current_dir(FRONTEND))?;

    if check {
        let status = Command::new("git")
            .args(["status", "--porcelain", "--", GENERATED])
            .current_dir(FRONTEND)
            .output()
            .map_err(|e| format!("could not run git status: {e}"))?;
        let changed = String::from_utf8_lossy(&status.stdout);
        if !changed.trim().is_empty() {
            return Err(format!(
                "generated bindings are stale, run `cargo run -p xtask -- gen-ts` and commit:\n{changed}"
            ));
        }
    }
    Ok(format!(
        "gen-ts: {} bindings from {} crates",
        exported.len(),
        crates.len()
    ))
}
