//! Merge the DERIVED card placement fields into freshly exported DynIllust scene JSONs.
//!
//! `backdropScale` / `backdropOffsetPx` place the operator illustration in scene space for
//! the Evolved Art card, and the frontend's silhouette clip is data-gated on their
//! presence. The values are per-key registrations SOLVED FROM MEASUREMENT (56
//! character-anchored SIFT solves, 15 masked-NCC solves cross-validated on an art-ablated
//! render; provenance and control statistics in docs/DYNCHAR_GATES.md, 2026-09-01) and are
//! not derivable inside this exporter, which has no renderer. Before this pass existed the
//! values lived only as post-export patches, and an install silently wiped all 56, which is
//! how a fielded card shipped square (chen2_2, 2026-09-01). Re-exports now REGENERATE the
//! fields from the committed `cardfields.json`, so the cost of the solve is paid once and
//! an install can no longer revert them.
//!
//! `DYNCHAR_CARDFIELDS=<path>` overrides the values file; `DYNCHAR_NO_CARDFIELDS=1`
//! (presence, not value) skips the pass and restores the previous exporter output exactly.

use serde_json::Value;
use std::path::Path;

/// Insert the derived fields into `{output}/spine/DynIllust/<key>/*[scene].json` for every
/// key present in the values file. Missing keys and missing scene JSONs are skipped and
/// counted; the summary line is the loud audit that the data made it into the export.
pub fn merge(output_root: impl AsRef<Path>) {
    if std::env::var("DYNCHAR_NO_CARDFIELDS").is_ok() {
        println!("cardfields: skipped (DYNCHAR_NO_CARDFIELDS set)");
        return;
    }
    let values_path = std::env::var("DYNCHAR_CARDFIELDS")
        .unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/cardfields.json").to_string());
    let raw = match std::fs::read_to_string(&values_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cardfields: FAILED to read {values_path}: {e} (derived card placements NOT merged; cards for fielded keys will render square)");
            return;
        }
    };
    let values: Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("cardfields: FAILED to parse {values_path}: {e}");
            return;
        }
    };
    let Some(map) = values.as_object() else {
        eprintln!("cardfields: {values_path} is not a JSON object");
        return;
    };
    let dyn_root = output_root.as_ref().join("spine").join("DynIllust");
    let (mut merged, mut absent, mut failed) = (0usize, 0usize, 0usize);
    for (key, val) in map {
        let dir = dyn_root.join(key);
        if !dir.is_dir() {
            absent += 1;
            continue;
        }
        let scene_json = std::fs::read_dir(&dir).ok().and_then(|rd| {
            rd.filter_map(Result::ok)
                .map(|e| e.path())
                .find(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with("[scene].json")))
        });
        let Some(path) = scene_json else {
            absent += 1;
            continue;
        };
        let merged_ok = (|| -> Option<()> {
            let mut scene: Value = serde_json::from_str(&std::fs::read_to_string(&path).ok()?).ok()?;
            let obj = scene.as_object_mut()?;
            obj.insert("backdropScale".into(), val.get("backdropScale")?.clone());
            obj.insert("backdropOffsetPx".into(), val.get("backdropOffsetPx")?.clone());
            std::fs::write(&path, serde_json::to_string(&scene).ok()?).ok()?;
            Some(())
        })()
        .is_some();
        if merged_ok {
            merged += 1;
        } else {
            failed += 1;
            eprintln!("cardfields: FAILED to merge into {}", path.display());
        }
    }
    println!("cardfields: merged {merged} of {} derived card placements ({absent} keys not in this export, {failed} failed)", map.len());
}

#[cfg(test)]
mod tests {
    use super::merge;
    use serde_json::Value;

    /// End-to-end over a temp tree: a key in the values file gets both fields merged with
    /// the rest of its scene JSON untouched, a key absent from the export is skipped, and
    /// the pass is idempotent.
    #[test]
    fn merges_fields_into_scene_json() {
        let tmp = std::env::temp_dir().join(format!("cardfields_test_{}", std::process::id()));
        let dir = tmp.join("spine/DynIllust/char_9999_test_2");
        std::fs::create_dir_all(&dir).unwrap();
        let scene = dir.join("dyn_illust_char_9999_test2[scene].json");
        std::fs::write(&scene, r#"{"frame":{"cameraSizePx":1000},"layers":[1,2]}"#).unwrap();
        let values = tmp.join("values.json");
        std::fs::write(
            &values,
            r#"{"char_9999_test_2":{"backdropScale":1.25,"backdropOffsetPx":[3.0,-7.5],"src":"test"},
               "char_9999_absent_2":{"backdropScale":2.0,"backdropOffsetPx":[0.0,0.0]}}"#,
        )
        .unwrap();
        // SAFETY: this is the only test in the module and touches no other thread's env.
        unsafe { std::env::set_var("DYNCHAR_CARDFIELDS", &values) };
        merge(&tmp);
        merge(&tmp); // idempotent
        unsafe { std::env::remove_var("DYNCHAR_CARDFIELDS") };
        let out: Value = serde_json::from_str(&std::fs::read_to_string(&scene).unwrap()).unwrap();
        assert_eq!(out["backdropScale"], 1.25);
        assert_eq!(out["backdropOffsetPx"][1], -7.5);
        assert_eq!(out["frame"]["cameraSizePx"], 1000, "unrelated fields preserved");
        assert_eq!(out["layers"][1], 2);
        std::fs::remove_dir_all(&tmp).unwrap();
    }
}
