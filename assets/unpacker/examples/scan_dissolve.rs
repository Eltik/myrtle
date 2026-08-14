//! THROWAWAY scan: which materials use the `Dissolve/` shader family, and do they carry a
//! dissolve that would actually DO anything?
//!
//! The shader's mask is `clamp((tex(_DissolveTex_0N) - _Amount_0N + _BorderWidth_0N*s) /
//! _BorderWidth_0N, 0, 1)` per map, multiplied together. At `_Amount ~ 0` the mask is 1 and the
//! program collapses to plain `2*color*tex`, so a material with no `_Amount_01/02` — or with them
//! at ~0 — renders identically to what we already draw. The exporter reads the SINGLE-name
//! `_DissolveTex` / `_Amount` / `_BorderWidth`, which on these materials are inert residue, so
//! before implementing the real names it is worth knowing how many materials would change.
//!
//! Usage: cargo run --release --example `scan_dissolve` -- <bundle.ab>...
#![allow(clippy::case_sensitive_file_extension_comparisons)]

use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use unpacker::export::shader_map::{build_shader_map, resolve_shader};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}
fn fl(sp: &Value, k: &str) -> Option<f64> {
    sp.get("m_Floats")?.get(k)?.as_f64()
}
fn tex_bound(sp: &Value, k: &str) -> bool {
    sp.get("m_TexEnvs")
        .and_then(|t| t.get(k))
        .and_then(|t| t.get("m_Texture"))
        .and_then(pid)
        .is_some_and(|p| p != 0)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let map = build_shader_map(&[PathBuf::from("../ArkAssets/en/[uc]shaders.ab")]);
    let (mut tot, mut active) = (0usize, 0usize);
    for path in &args {
        let Ok(data) = std::fs::read(path) else {
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            continue;
        };
        let skin = PathBuf::from(path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
                continue;
            };
            let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
            for obj in &sf.objects {
                if let Ok(v) = read_object(&sf, obj) {
                    all.insert(obj.path_id, (obj.class_id, v));
                }
            }
            for (cid, v) in all.values() {
                if *cid != 21 {
                    continue;
                }
                let (Some(fid), Some(sid)) = (
                    v.get("m_Shader")
                        .and_then(|s| s.get("m_FileID"))
                        .and_then(Value::as_i64),
                    v.get("m_Shader").and_then(pid),
                ) else {
                    continue;
                };
                let Some(name) = resolve_shader(&sf.externals, fid, sid, &map) else {
                    continue;
                };
                if !name.contains("/Dissolve/") {
                    continue;
                }
                let Some(sp) = v.get("m_SavedProperties") else {
                    continue;
                };
                let mname = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
                let (a1, a2) = (fl(sp, "_Amount_01"), fl(sp, "_Amount_02"));
                let d1 = tex_bound(sp, "_DissolveTex_01");
                let d2 = tex_bound(sp, "_DissolveTex_02");
                // "active" = at least one map bound AND its amount far enough from 0 to bite
                let bites =
                    (d1 && a1.is_some_and(|x| x > 0.01)) || (d2 && a2.is_some_and(|x| x > 0.01));
                tot += 1;
                if bites {
                    active += 1;
                }
                println!(
                    "{:<34} {:<26} a01={:<8} a02={:<8} bw01={:<7} bw02={:<7} pow={:<6} edge={:<5} tex01={} tex02={}  {}",
                    skin,
                    mname,
                    a1.map_or_else(|| "-".into(), |x| format!("{x:.3}")),
                    a2.map_or_else(|| "-".into(), |x| format!("{x:.3}")),
                    fl(sp, "_BorderWidth_01").map_or_else(|| "-".into(), |x| format!("{x:.2}")),
                    fl(sp, "_BorderWidth_02").map_or_else(|| "-".into(), |x| format!("{x:.2}")),
                    fl(sp, "_pow").map_or_else(|| "-".into(), |x| format!("{x:.2}")),
                    if sp
                        .get("m_Colors")
                        .and_then(|c| c.get("_Edgecolor"))
                        .is_some()
                    {
                        "yes"
                    } else {
                        "-"
                    },
                    d1,
                    d2,
                    if bites { "<-- WOULD CHANGE" } else { "" }
                );
            }
        }
    }
    println!("\n{active} of {tot} Dissolve/ materials carry a dissolve that would actually bite");
}
