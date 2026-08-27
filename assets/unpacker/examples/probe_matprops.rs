//! THROWAWAY diagnostic: dump EVERY saved property of the materials on a named GameObject.
//!
//! Motivation. Unity's `Ram/Disturb(CustomData)` vertex stage rotates each sampler's UV about
//! (0.5, 0.5) by its own `_RotationN` vector, and our port applies no rotation at all. Whether
//! that matters turns entirely on whether those vectors are identity, which no dump so far has
//! printed: `probe_layerblend` shows floats and colours, and a Vector property is neither.
//!
//! Usage: cargo run --release --example probe_matprops -- <bundle.ab> <go-name-substr>
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

/// Print a `m_SavedProperties` sub-table, tolerating both the map encoding and the
/// array-of-pairs one that older serialisations use.
fn dump(label: &str, node: Option<&Value>) {
    let Some(node) = node else { return };
    if let Some(map) = node.as_object() {
        for (k, v) in map {
            println!("      {label:<9} {k:<26} {}", compact(v));
        }
    } else if let Some(arr) = node.as_array() {
        for e in arr {
            let (k, v) = match (e.get("first"), e.get("second")) {
                (Some(k), Some(v)) => (k.clone(), v.clone()),
                _ => match e.as_array() {
                    Some(a) if a.len() == 2 => (a[0].clone(), a[1].clone()),
                    _ => continue,
                },
            };
            let ks = k.as_str().map(str::to_string).unwrap_or_else(|| k.to_string());
            println!("      {label:<9} {ks:<26} {}", compact(&v));
        }
    }
}

fn compact(v: &Value) -> String {
    let s = serde_json::to_string(v).unwrap_or_default();
    if s.len() > 120 { format!("{}…", &s[..120]) } else { s }
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle.ab");
    let want = std::env::args().nth(2).unwrap_or_default().to_ascii_lowercase();
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    let mut objs: HashMap<i64, (i32, Value)> = HashMap::new();
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        for obj in &sf.objects {
            if let Ok(v) = read_object(&sf, obj) {
                objs.insert(obj.path_id, (obj.class_id, v));
            }
        }
    }
    let mut go_components: HashMap<i64, Vec<i64>> = HashMap::new();
    for (id, (cid, v)) in &objs {
        if *cid == 1 {
            let comps = v
                .get("m_Component")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|c| c.get("component").and_then(pid).or_else(|| pid(c))).collect())
                .unwrap_or_default();
            go_components.insert(*id, comps);
        }
    }
    let mut seen: std::collections::HashSet<i64> = std::collections::HashSet::new();
    for (go, comps) in &go_components {
        let name = objs.get(go).and_then(|(_, g)| g.get("m_Name")).and_then(Value::as_str).unwrap_or("");
        if !name.to_ascii_lowercase().contains(&want) {
            continue;
        }
        for c in comps {
            // 23 MeshRenderer, 199 ParticleSystemRenderer
            let Some((cid, r)) = objs.get(c) else { continue };
            if *cid != 23 && *cid != 199 {
                continue;
            }
            // The INDEX within `m_Materials` matters: a ParticleSystemRenderer draws with slot 0
            // and may carry a trail material after it, so "this GO has a material with X" is not
            // the same claim as "the emitter draws with X".
            let cname = if *cid == 23 { "MeshRenderer" } else { "ParticleSystemRenderer" };
            for (mi, mp) in r.get("m_Materials").and_then(Value::as_array).into_iter().flatten().enumerate() {
                let Some(mp) = pid(mp) else { continue };
                if !seen.insert(mp) {
                    continue;
                }
                let Some((_, mat)) = objs.get(&mp) else { continue };
                println!("=== GO {name:?}  {cname} m_Materials[{mi}]  material {mp}");
                let sp = mat.get("m_SavedProperties");
                dump("float", sp.and_then(|s| s.get("m_Floats")));
                dump("color", sp.and_then(|s| s.get("m_Colors")));
                dump("vector", sp.and_then(|s| s.get("m_Vectors")));
                // TEXTURE slots, with the referenced Texture2D resolved. Whether a sampler is
                // BOUND at all decides what the game samples: an unbound slot falls back to the
                // shader's declared default (usually "white", i.e. 1.0), which is a very different
                // value from whatever our exporter chose to put in that index.
                if let Some(te) = sp.and_then(|s| s.get("m_TexEnvs")) {
                    let mut rows: Vec<(String, String)> = Vec::new();
                    let mut push = |k: String, v: &Value| {
                        let tp = v.get("m_Texture").and_then(pid).unwrap_or(0);
                        let sc = v.get("m_Scale").map(compact).unwrap_or_default();
                        let of = v.get("m_Offset").map(compact).unwrap_or_default();
                        let named = objs
                            .get(&tp)
                            .and_then(|(_, t)| t.get("m_Name"))
                            .and_then(Value::as_str)
                            .unwrap_or(if tp == 0 { "UNBOUND" } else { "external/absent" });
                        rows.push((k, format!("{named:<28} pid {tp:<22} scale {sc} offset {of}")));
                    };
                    if let Some(map) = te.as_object() {
                        for (k, v) in map {
                            push(k.clone(), v);
                        }
                    } else if let Some(arr) = te.as_array() {
                        for e in arr {
                            let (k, v) = match (e.get("first"), e.get("second")) {
                                (Some(k), Some(v)) => (k.clone(), v.clone()),
                                _ => match e.as_array() {
                                    Some(a) if a.len() == 2 => (a[0].clone(), a[1].clone()),
                                    _ => continue,
                                },
                            };
                            let ks = k.as_str().map(str::to_string).unwrap_or_else(|| k.to_string());
                            push(ks, &v);
                        }
                    }
                    rows.sort();
                    for (k, v) in rows {
                        println!("      texenv    {k:<26} {v}");
                    }
                }
            }
        }
    }
}
