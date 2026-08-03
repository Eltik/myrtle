//! THROWAWAY diagnostic: dump every Material's texture-property (`m_TexEnvs`) slots, with the
//! size of each referenced Texture2D.
//!
//! Motivation: Ch'en the Holungday's sky is one 4-vertex quad whose exported texture is 100%
//! OPAQUE (alphaMin 255), yet the game confines the art to a ragged cut-out. The silhouette is
//! therefore neither in the mesh nor in that texture's alpha. If a material declares a mask /
//! alpha companion under a name the exporter never reads (it only consults `_AlphaTex`), that
//! is where the shape lives.
//!
//! Usage: cargo run --release --example probe_scenemat -- <bundle.ab>

use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

/// Pull the texture-property table out of a Material, tolerating both the tuple encoding
/// (`{first,second}`) and the positional one (`[k, v]`).
fn tex_envs(mat: &Value) -> Vec<(String, i64)> {
    let mut out = Vec::new();
    let Some(te) = mat.get("m_SavedProperties").and_then(|sp| sp.get("m_TexEnvs")) else {
        return out;
    };
    // Unity serialises this as a MAP (property name -> {m_Texture, m_Scale, m_Offset}). An
    // earlier version of this probe assumed the array-of-{first,second} encoding and silently
    // reported zero properties for every material — the bug that made the whole material scan
    // look empty. Handle both.
    if let Some(o) = te.as_object() {
        for (k, v) in o {
            out.push((k.clone(), v.get("m_Texture").and_then(pid).unwrap_or(0)));
        }
    } else if let Some(arr) = te.as_array() {
        for e in arr {
            let k = e
                .get("first")
                .and_then(Value::as_str)
                .or_else(|| e.get(0).and_then(Value::as_str))
                .unwrap_or("?")
                .to_string();
            let tp = e
                .get("second")
                .or_else(|| e.get(1))
                .and_then(|x| x.get("m_Texture"))
                .and_then(pid)
                .unwrap_or(0);
            out.push((k, tp));
        }
    }
    out
}

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else {
            eprintln!("cannot read {path}");
            continue;
        };
        let bundle = match BundleFile::parse(data) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("bundle parse failed: {e}");
                continue;
            }
        };
        println!("\n===== {}", path.rsplit('/').next().unwrap_or(&path));
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let sf = match SerializedFile::parse(entry.data.clone()) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("  serialized parse failed for {}: {e}", entry.path);
                    continue;
                }
            };
            // SELF-CHECK: an earlier version of this probe reported zero materials while a class
            // histogram of the same file counted 36. Print the histogram here so the probe can
            // never again disagree with itself silently.
            let mut hist: BTreeMap<i32, usize> = BTreeMap::new();
            for o in &sf.objects {
                *hist.entry(o.class_id).or_default() += 1;
            }
            println!("  file {} : {} objects", entry.path, sf.objects.len());
            println!("  class histogram: {hist:?}");

            let mut tex: HashMap<i64, (String, i64, i64)> = HashMap::new();
            let mut mats: Vec<(i64, String, Vec<(String, i64)>)> = Vec::new();
            let mut mat_seen = 0usize;
            let mut mat_err = 0usize;
            for obj in &sf.objects {
                match obj.class_id {
                    21 => {
                        mat_seen += 1;
                        match read_object(&sf, obj) {
                            Ok(v) => {
                                let n = v.get("m_Name").and_then(Value::as_str).unwrap_or("?").to_string();
                                mats.push((obj.path_id, n, tex_envs(&v)));
                            }
                            Err(e) => {
                                mat_err += 1;
                                if mat_err <= 3 {
                                    eprintln!("  material pid {} unreadable: {e}", obj.path_id);
                                }
                            }
                        }
                    }
                    28 => {
                        if let Ok(v) = read_object(&sf, obj) {
                            tex.insert(
                                obj.path_id,
                                (
                                    v.get("m_Name").and_then(Value::as_str).unwrap_or("?").to_string(),
                                    v.get("m_Width").and_then(Value::as_i64).unwrap_or(0),
                                    v.get("m_Height").and_then(Value::as_i64).unwrap_or(0),
                                ),
                            );
                        }
                    }
                    _ => {}
                }
            }
            println!("  materials seen: {mat_seen}  read OK: {}  failed: {mat_err}  textures: {}", mats.len(), tex.len());
            // STRUCTURE PROBE: the TexEnvs path came back empty for every material, so print the
            // real shape of one before trusting any accessor.
            if let Some(obj) = sf.objects.iter().find(|o| o.class_id == 21)
                && let Ok(v) = read_object(&sf, obj)
            {
                if let Some(o) = v.as_object() {
                    let mut ks: Vec<&String> = o.keys().collect();
                    ks.sort();
                    println!("  RAW material top-level keys: {ks:?}");
                }
                if let Some(sp) = v.get("m_SavedProperties").and_then(Value::as_object) {
                    let mut ks: Vec<&String> = sp.keys().collect();
                    ks.sort();
                    println!("  m_SavedProperties keys: {ks:?}");
                    if let Some(te) = sp.get("m_TexEnvs") {
                        let s = serde_json::to_string(te).unwrap_or_default();
                        println!("  m_TexEnvs (first 600 chars): {}", &s[..s.len().min(600)]);
                    }
                } else {
                    println!("  m_SavedProperties: ABSENT or not an object -> {}", serde_json::to_string(&v).map(|s| s.chars().take(500).collect::<String>()).unwrap_or_default());
                }
            }

            let mut key_counts: BTreeMap<String, usize> = BTreeMap::new();
            for (_, _, envs) in &mats {
                for (k, _) in envs {
                    *key_counts.entry(k.clone()).or_default() += 1;
                }
            }
            println!("\n  DISTINCT texture-property names ({}):", key_counts.len());
            for (k, n) in &key_counts {
                println!("      {k:<28} on {n} materials");
            }
            // Scalar/colour properties, read with the SAME map-shaped accessor. An earlier probe
            // (probe_skelprops) assumed the array encoding here too and reported "no saved
            // properties at all" for every spine material — a FALSE NEGATIVE that was used to
            // rule out a per-skin material tint.
            println!("\n  NON-DEFAULT colours / floats per material:");
            for obj in sf.objects.iter().filter(|o| o.class_id == 21) {
                let Ok(v) = read_object(&sf, obj) else { continue };
                let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
                let sp = v.get("m_SavedProperties");
                let mut bits: Vec<String> = Vec::new();
                if let Some(cols) = sp.and_then(|x| x.get("m_Colors")).and_then(Value::as_object) {
                    for (k, c) in cols {
                        let g = |n: &str| c.get(n).and_then(Value::as_f64).unwrap_or(1.0);
                        let (r, gg, b, a) = (g("r"), g("g"), g("b"), g("a"));
                        if (r - 1.0).abs() > 1e-3 || (gg - 1.0).abs() > 1e-3 || (b - 1.0).abs() > 1e-3 || (a - 1.0).abs() > 1e-3 {
                            bits.push(format!("{k}=[{r:.3},{gg:.3},{b:.3},{a:.3}]"));
                        }
                    }
                }
                if let Some(fl) = sp.and_then(|x| x.get("m_Floats")).and_then(Value::as_object) {
                    for (k, f) in fl {
                        let x = f.as_f64().unwrap_or(0.0);
                        if x.abs() > 1e-6 {
                            bits.push(format!("{k}={x:.3}"));
                        }
                    }
                }
                if !bits.is_empty() {
                    println!("    '{name}': {}", bits.join("  "));
                }
            }

            // STENCIL / MASK scan: print these ALWAYS, including zeros — `_StencilRef = 0` is
            // meaningful, and the non-default filter above would hide it.
            println!("\n  STENCIL / MASK / CUTOFF properties, with each material's _MainTex:");
            for obj in sf.objects.iter().filter(|o| o.class_id == 21) {
                let Ok(v) = read_object(&sf, obj) else { continue };
                let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
                let sp = v.get("m_SavedProperties");
                let mut hits: Vec<String> = Vec::new();
                for field in ["m_Floats", "m_Ints", "m_Colors"] {
                    if let Some(o) = sp.and_then(|x| x.get(field)).and_then(Value::as_object) {
                        for (k, val) in o {
                            let lk = k.to_ascii_lowercase();
                            if lk.contains("stencil") || lk.contains("mask") || lk.contains("cutoff") || lk.contains("clip") {
                                hits.push(format!("{k}={}", serde_json::to_string(val).unwrap_or_default()));
                            }
                        }
                    }
                }
                let main = tex_envs(&v)
                    .into_iter()
                    .find(|(k, _)| k == "_MainTex")
                    .and_then(|(_, t)| tex.get(&t).cloned())
                    .map_or_else(|| "-".to_string(), |(n, w, h)| format!("{n} {w}x{h}"));
                if !hits.is_empty() {
                    println!("    '{name}'  _MainTex={main}");
                    println!("        {}", hits.join("  "));
                }
            }

            println!("\n  Materials with MORE THAN ONE bound texture (a mask/alpha companion would appear here):");
            for (p, name, envs) in &mats {
                let bound: Vec<&(String, i64)> = envs.iter().filter(|(_, t)| *t != 0).collect();
                if bound.len() < 2 {
                    continue;
                }
                println!("    MAT '{name}' (pid {p})");
                for (k, t) in bound {
                    let d = tex
                        .get(t)
                        .map_or_else(|| format!("pid {t} (external/absent)"), |(n, w, h)| format!("{n} {w}x{h}"));
                    println!("        {k:<24} = {d}");
                }
            }
        }
    }
}
