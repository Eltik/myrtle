//! Guards the generated TypeScript bindings against silent wire-type drift.
//!
//! `cargo test export_bindings` regenerates `frontend/src/types/generated/`, and
//! the frontend's hand-written aliases are thin wrappers over those files. That
//! makes ts-rs's defaults part of the API contract, and two of them are wrong
//! for this backend:
//!
//! * ts-rs maps `i64`/`u64` to `bigint`, but `serde_json` writes them as plain
//!   JSON numbers, so `JSON.parse` yields `number`. A `bigint` in the bindings
//!   is a type the runtime can never produce — arithmetic against it fails to
//!   compile in correct frontend code. Fix by annotating the Rust field with
//!   `#[ts(type = "number")]` (or `"number | null"` / `"Record<string, number>"`).
//! * A bare `any` disables checking on everything downstream of it. It should
//!   never appear: `serde_json::Value` resolves to ts-rs's recursive `JsonValue`
//!   via the `serde-json-impl` feature. Note that `unknown` is NOT a valid
//!   substitute here — TanStack Start's `createServerFn` rejects it as
//!   unserializable, so a hand-written `#[ts(type = "unknown")]` breaks the
//!   frontend build at every server-fn boundary the type crosses.
//!
//! Neither shows up as a build failure anywhere else: the bindings compile fine,
//! they are just wrong. Hence this test.

use std::fs;
use std::path::PathBuf;

fn generated_dir() -> PathBuf {
    // Mirrors TS_RS_EXPORT_DIR in `backend/.cargo/config.toml`.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../frontend/src/types/generated")
}

/// Scan every generated binding for type constructs that cannot appear on the wire.
///
/// Run `cargo test export_bindings` first if this fails on a stale directory.
#[test]
fn bindings_contain_no_impossible_wire_types() {
    let dir = generated_dir();
    let mut queue = vec![dir.clone()];
    let mut files: Vec<PathBuf> = Vec::new();
    while let Some(next) = queue.pop() {
        let entries = fs::read_dir(&next).unwrap_or_else(|e| {
            panic!(
                "cannot read {}: {e}\nrun `bun run gen:types`",
                next.display()
            )
        });
        for entry in entries {
            let path = entry.expect("readable dir entry").path();
            if path.is_dir() {
                queue.push(path);
            } else {
                files.push(path);
            }
        }
    }

    let mut offenders: Vec<String> = Vec::new();
    let mut checked = 0usize;

    for path in files {
        if path.extension().and_then(|e| e.to_str()) != Some("ts") {
            continue;
        }
        checked += 1;
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let src = fs::read_to_string(&path).expect("readable binding");

        // ts-rs inlines Rust doc comments into the emitted type literal, so
        // strip comments before matching rather than trusting line prefixes.
        let mut code = String::with_capacity(src.len());
        let mut rest = src.as_str();
        loop {
            match rest.find("/*") {
                Some(start) => {
                    code.push_str(&rest[..start]);
                    match rest[start..].find("*/") {
                        Some(end) => rest = &rest[start + end + 2..],
                        None => break,
                    }
                }
                None => {
                    code.push_str(rest);
                    break;
                }
            }
        }
        for line in code.lines() {
            let line = line.split("//").next().unwrap_or(line);
            for (token, hint) in [
                (
                    "bigint",
                    "annotate the i64/u64 field with #[ts(type = \"number\")]",
                ),
                (
                    "any",
                    "a serde_json::Value field lost its JsonValue binding — check the serde-json-impl feature",
                ),
            ] {
                if line.match_indices(token).any(|(at, _)| {
                    let before = line[..at].chars().next_back();
                    let after = line[at + token.len()..].chars().next();
                    let boundary = |c: Option<char>| {
                        c.is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != '$')
                    };
                    boundary(before) && boundary(after)
                }) {
                    offenders.push(format!("{name}: `{token}` — {hint}"));
                    break;
                }
            }
        }
    }

    assert!(checked > 0, "no bindings found in {}", dir.display());
    assert!(
        offenders.is_empty(),
        "{} generated binding(s) declare a type the wire cannot produce:\n  {}",
        offenders.len(),
        offenders.join("\n  ")
    );
}
