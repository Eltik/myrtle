//! THROWAWAY census: for every Material whose shader name contains a substring, the joint
//! distribution of `_MainColorACtrl` and `_MainColor.a`, so the fraction of a family on which
//! a control term is LIVE can be read instead of inferred from the property's presence.
//!
//! Motivation: the `Disturb Anchor` fragments apply `k = _MainColorACtrl * (_MainColor.a - 1) + 1`
//! to all four channels after the x2. That term is the identity whenever the control is 0 or
//! the alpha is 1, so "323 materials declare it" says nothing about how many draw differently.
//! `scan_matprop` counts the property over every shader (residue included) and cannot join it
//! to the shader that reads it.
//!
//! Usage: cargo run --release --example `probe_anchorctrl` -- <shaders.ab> <shader-substr> <bundle.ab>...
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
use unpacker::export::shader_map::{build_shader_map, resolve_shader};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(shaders), Some(want)) = (args.next(), args.next()) else {
        eprintln!("usage: probe_anchorctrl <shaders.ab> <shader-substr> <bundle.ab>...");
        return;
    };
    let map = build_shader_map(&[PathBuf::from(shaders)]);
    let mut total = 0usize;
    let mut ctrl_on = 0usize;
    let mut live = 0usize;
    let mut per_shader: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut skins_live: BTreeMap<String, usize> = BTreeMap::new();
    for path in args {
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            continue;
        };
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
                continue;
            };
            for obj in &sf.objects {
                if obj.class_id != 21 {
                    continue;
                }
                let Ok(mat) = read_object(&sf, obj) else {
                    continue;
                };
                let Some(shader) = mat.get("m_Shader").and_then(|s| {
                    let fid = s.get("m_FileID").and_then(Value::as_i64)?;
                    resolve_shader(&sf.externals, fid, pid(s)?, &map)
                }) else {
                    continue;
                };
                if !shader.contains(&want) {
                    continue;
                }
                total += 1;
                let saved = mat.get("m_SavedProperties");
                let ctrl = saved
                    .and_then(|s| s.get("m_Floats"))
                    .and_then(|f| f.get("_MainColorACtrl"))
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0);
                let alpha = saved
                    .and_then(|s| s.get("m_Colors"))
                    .and_then(|c| c.get("_MainColor"))
                    .and_then(|c| c.get("a"))
                    .and_then(Value::as_f64)
                    .unwrap_or(1.0);
                let e = per_shader.entry(shader.to_string()).or_insert((0, 0));
                e.0 += 1;
                // k = ctrl * (alpha - 1) + 1 differs from 1 only when both terms are away
                // from their identity; a half-step of 8-bit precision is the threshold.
                let is_live = ctrl.abs() > 1e-6 && (alpha - 1.0).abs() > 0.5 / 255.0;
                if ctrl.abs() > 1e-6 {
                    ctrl_on += 1;
                }
                if is_live {
                    live += 1;
                    e.1 += 1;
                    let skin = std::path::Path::new(&path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    *skins_live.entry(skin).or_insert(0) += 1;
                    println!(
                        "  live  {:<58} ctrl {ctrl:.3} alpha {alpha:.3} k {:.3}  {}",
                        shader,
                        ctrl * (alpha - 1.0) + 1.0,
                        mat.get("m_Name").and_then(Value::as_str).unwrap_or("")
                    );
                }
            }
        }
    }
    println!(
        "materials on '{want}': {total}   ctrl != 0: {ctrl_on}   live (ctrl != 0 and alpha != 1): {live}"
    );
    for (s, (n, l)) in &per_shader {
        println!("  {s:<58} {n:>5} materials, {l:>4} live");
    }
    println!("skins with a live material: {}", skins_live.len());
    for (s, n) in &skins_live {
        println!("  {s} {n}");
    }
}
