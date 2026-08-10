//! THROWAWAY scan: which `Torappu/Particles-L2D/*` shader families carry a `_TintColor`, and how
//! many materials each covers across the dynchar corpus.
//!
//! Motivation: every Torappu particle/scene shader read out of `[uc]shaders.ab` so far computes
//! `tex * (vs_COLOR0 + vs_COLOR0)` — a ×2 — with the vertex stage building
//! `vs_COLOR0 = in_COLOR0 * _TintColor`. The exporter only applies that ×2 to the PLAIN blend
//! modes (`Particles-L2D/<Mode>` with nothing below), so sub-namespaced families like `Dissolve/`
//! export at HALF amplitude. Before widening the gate, enumerate exactly which families it would
//! newly admit so the change can be justified per-family rather than assumed.
//!
//! Usage: cargo run --release --example scan_l2dtint -- <bundle.ab>...

use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use unpacker::export::shader_map::{build_shader_map, resolve_shader};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn has_color(v: &Value, key: &str) -> bool {
    v.get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Colors"))
        .and_then(|c| c.as_object())
        .is_some_and(|c| c.contains_key(key))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let map = build_shader_map(&[PathBuf::from("../ArkAssets/en/[uc]shaders.ab")]);
    // shader name -> (materials, with _TintColor, with _MainColor, example skins)
    let mut tally: BTreeMap<String, (usize, usize, usize, Vec<String>)> = BTreeMap::new();

    for path in &args {
        let Ok(data) = std::fs::read(path) else { continue };
        let Ok(bundle) = BundleFile::parse(data) else { continue };
        let skin = PathBuf::from(path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
            let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
            for obj in &sf.objects {
                if let Ok(v) = read_object(&sf, obj) {
                    all.insert(obj.path_id, (obj.class_id, v));
                }
            }
            for (_p, (cid, v)) in &all {
                if *cid != 21 {
                    continue;
                }
                let (Some(fid), Some(sid)) = (
                    v.get("m_Shader").and_then(|s| s.get("m_FileID")).and_then(Value::as_i64),
                    v.get("m_Shader").and_then(pid),
                ) else {
                    continue;
                };
                let Some(name) = resolve_shader(&sf.externals, fid, sid, &map) else { continue };
                if std::env::var("VERBOSE").is_ok() {
                    let g = |k: &str| {
                        v.get("m_SavedProperties")
                            .and_then(|sp| sp.get("m_Colors"))
                            .and_then(|c| c.get(k))
                            .map(|c| {
                                format!(
                                    "({:.3},{:.3},{:.3},{:.3})",
                                    c.get("r").and_then(Value::as_f64).unwrap_or(-1.0),
                                    c.get("g").and_then(Value::as_f64).unwrap_or(-1.0),
                                    c.get("b").and_then(Value::as_f64).unwrap_or(-1.0),
                                    c.get("a").and_then(Value::as_f64).unwrap_or(-1.0)
                                )
                            })
                            .unwrap_or_else(|| "-".into())
                    };
                    println!(
                        "  MAT {:<30} {:<52} tint={:<26} main={}",
                        v.get("m_Name").and_then(Value::as_str).unwrap_or("?"),
                        name,
                        g("_TintColor"),
                        g("_MainColor")
                    );
                }
                let e = tally.entry(name.to_string()).or_default();
                e.0 += 1;
                if has_color(v, "_TintColor") {
                    e.1 += 1;
                }
                if has_color(v, "_MainColor") {
                    e.2 += 1;
                }
                if !e.3.contains(&skin) && e.3.len() < 4 {
                    e.3.push(skin.clone());
                }
            }
        }
    }

    println!("{:<62} {:>5} {:>6} {:>6}  skins", "shader", "mats", "_Tint", "_Main");
    for (name, (n, t, m, skins)) in &tally {
        // The gate question is only about the L2D family; `/Particles/` is already covered.
        let rest = name.rfind("Particles-L2D/").map(|i| &name[i + "Particles-L2D/".len()..]);
        let mark = match rest {
            Some(r) if r.contains('/') => "<-- SUB-NAMESPACED",
            Some(_) => "(plain, already ×2)",
            None => "",
        };
        println!("{name:<62} {n:>5} {t:>6} {m:>6}  {} {mark}", skins.join(","));
    }
}
