//! THROWAWAY diagnostic: list every PARTICLE material in a dynchar bundle with its shader,
//! its blend classification, and its `_TintColor`.
//!
//! Motivation: `particle_tint` early-returns `None` for any material `is_additive` rejects, so
//! an ALPHABLEND legacy port never exports its `_TintColor` — yet Unity's legacy particle
//! shaders multiply by `_TintColor * 2` for alphablend exactly as they do for additive. Virtuosa's
//! nested diamond frames (`sys55`: normal blend, no `_MainTex`, vertex-coloured mesh) render
//! visibly dimmer than the game's. Before touching that gate, read what the materials actually
//! carry: if every non-additive tint is the 0.5 neutral, the gate costs nothing.
//!
//! Usage: cargo run --release --example probe_ptint -- <bundle.ab> <shaders-dir>

use serde_json::Value;
use std::collections::HashMap;
use unpacker::export::shader_map::{build_shader_map, resolve_shader};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn col(mat: &Value, key: &str) -> Option<[f64; 4]> {
    let c = mat.get("m_SavedProperties")?.get("m_Colors")?.get(key)?;
    let g = |n: &str| c.get(n).and_then(Value::as_f64).unwrap_or(f64::NAN);
    Some([g("r"), g("g"), g("b"), g("a")])
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("usage: probe_ptint <bundle.ab> <shaders-dir>");
    let shaders = args.next().expect("shaders dir");
    let smap = build_shader_map(&walkdir(&std::path::PathBuf::from(&shaders)));

    let data = std::fs::read(&path).expect("read bundle");
    let bundle = BundleFile::parse(data).expect("parse bundle");

    let mut n_total = 0usize;
    let mut n_nonneutral_nonadd = 0usize;
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        let mut tex: HashMap<i64, String> = HashMap::new();
        for o in &sf.objects {
            if o.class_id == 28
                && let Ok(v) = read_object(&sf, o)
            {
                tex.insert(o.path_id, v.get("m_Name").and_then(Value::as_str).unwrap_or("?").to_string());
            }
        }
        for o in sf.objects.iter().filter(|o| o.class_id == 21) {
            let Ok(mat) = read_object(&sf, o) else { continue };
            let shader = mat
                .get("m_Shader")
                .and_then(|s| Some((s.get("m_FileID")?.as_i64()?, s.get("m_PathID")?.as_i64()?)))
                .and_then(|(f, p)| resolve_shader(&sf.externals, f, p, &smap))
                .unwrap_or_else(|| "<unresolved>".into());
            let sl = shader.to_ascii_lowercase();
            if !sl.contains("particles") {
                continue;
            }
            n_total += 1;
            // The exporter's own rule: blend is fixed by the shader NAME, not `_DstBlend`.
            let additive = sl.contains("additive");
            let Some(t) = col(&mat, "_TintColor") else {
                println!(
                    "MAT {:<28} add={:<5} shader={}   _TintColor ABSENT",
                    mat.get("m_Name").and_then(Value::as_str).unwrap_or("?"),
                    additive,
                    shader
                );
                continue;
            };
            let neutral = t.iter().all(|c| (c - 0.5).abs() <= 1.5 / 255.0);
            if !additive && !neutral {
                n_nonneutral_nonadd += 1;
            }
            let main = mat
                .get("m_SavedProperties")
                .and_then(|sp| sp.get("m_TexEnvs"))
                .and_then(|te| te.get("_MainTex"))
                .and_then(|e| e.get("m_Texture"))
                .and_then(|t| t.get("m_PathID"))
                .and_then(Value::as_i64)
                .and_then(|t| tex.get(&t))
                .cloned()
                .unwrap_or_else(|| "-".into());
            println!(
                "MAT {:<28} add={:<5} neutral={:<5} tint=({:.3},{:.3},{:.3},{:.3}) x2=({:.3},{:.3},{:.3},{:.3}) tex={} shader={}",
                mat.get("m_Name").and_then(Value::as_str).unwrap_or("?"),
                additive,
                neutral,
                t[0], t[1], t[2], t[3],
                t[0] * 2.0, t[1] * 2.0, t[2] * 2.0, t[3] * 2.0,
                main,
                shader
            );
        }
    }
    println!("\nparticle materials: {n_total}   NON-additive with a NON-neutral _TintColor: {n_nonneutral_nonadd}");
}

fn walkdir(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        if d.is_file() {
            out.push(d);
            continue;
        }
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            stack.push(e.path());
        }
    }
    out
}
