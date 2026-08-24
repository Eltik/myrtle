//! THROWAWAY: the full transform chain (pos/rot/SCALE) of every GameObject whose name contains
//! $GOCHAIN, so an authored extent can be derived instead of fitted.
//! be recomputed independently of the exporter.
//! MeshFilter resolves a mesh, and whether that mesh is in-bundle. Answers "this skin
//! exported zero scene layers, but the bundle HAS renderers — where did they go?"
//! composite materials) in a dynchar bundle. Motivation: our renderer is measured to be
//! systematically too bright vs the in-game capture, and the per-pixel error fits a pure
//! sRGB gamma whose magnitude differs per skin — which smells like a per-scene colour
//! transform (post-process volume / LUT / camera property) our exporter isn't reading.
//!
//! Usage: cargo run --release --example `probe_grade` -- <bundle.ab> [`shaders_dir`]
//! `shaders_dir` defaults to assets/ArkAssets/en (where `[uc]shaders.ab` lives), used only
//! to resolve external material shader names.
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use unpacker::export::shader_map::build_shader_map;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn walkdir(root: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        if d.is_file() {
            out.push(d);
            continue;
        }
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            stack.push(e.path());
        }
    }
    out
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("bundle path");
    let shader_dir = args
        .next()
        .unwrap_or_else(|| "/Users/eltik/Documents/Coding/myrtle/assets/ArkAssets/en".to_string());

    let shader_files = walkdir(&PathBuf::from(&shader_dir));
    let shader_map = build_shader_map(&shader_files);
    eprintln!("shader map entries: {}", shader_map.len());

    println!("\n================================================================");
    println!("BUNDLE {path}");
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");

    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        println!("-- entry {} : {} objects", entry.path, sf.objects.len());
        println!(
            "   externals: {:?}",
            sf.externals
                .iter()
                .map(unpacker::unity::serialized_file::FileIdentifier::cab_name)
                .collect::<Vec<_>>()
        );

        // Skip only heavy binary-blob classes we don't need (keep Texture2D=28 for
        // name/dims, and everything else, since these bundles are small dynchar scenes).
        let skip: HashSet<i32> = [43, 48, 49, 83, 128, 213].into_iter().collect();
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for obj in &sf.objects {
            if skip.contains(&obj.class_id) {
                continue;
            }
            if let Ok(v) = read_object(&sf, obj) {
                all.insert(obj.path_id, (obj.class_id, v));
            }
        }

        // ---- hierarchy (GameObject names + paths) ----
        let mut go_name: HashMap<i64, String> = HashMap::new();
        let mut tf_go: HashMap<i64, i64> = HashMap::new();
        let mut go_tf: HashMap<i64, i64> = HashMap::new();
        let mut tf_father: HashMap<i64, i64> = HashMap::new();
        for (p, (cid, v)) in &all {
            match cid {
                1 => {
                    go_name.insert(
                        *p,
                        v.get("m_Name")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                    );
                }
                4 | 224 => {
                    if let Some(g) = v.get("m_GameObject").and_then(pid) {
                        tf_go.insert(*p, g);
                        go_tf.insert(g, *p);
                    }
                    if let Some(f) = v.get("m_Father").and_then(pid) {
                        tf_father.insert(*p, f);
                    }
                }
                _ => {}
            }
        }
        let _go_path = |go: i64| -> String {
            let mut parts: Vec<String> = Vec::new();
            let mut cur = go_tf.get(&go).copied();
            for _ in 0..128 {
                let Some(tf) = cur else { break };
                if let Some(g) = tf_go.get(&tf) {
                    parts.push(go_name.get(g).cloned().unwrap_or_else(|| format!("?{g}")));
                }
                cur = tf_father.get(&tf).copied().filter(|&f| f != 0);
            }
            parts.reverse();
            parts.join("/")
        };

        // ---- the chain of every GO whose name matches $GOCHAIN, camera-relative ----
        let want = std::env::var("GOCHAIN").unwrap_or_default();
        if want.is_empty() {
            continue;
        }
        let mut seen: std::collections::HashSet<i64> = std::collections::HashSet::new();
        for (go, nm) in &go_name {
            if !nm.contains(&want) || !seen.insert(*go) {
                continue;
            }
            println!("GO {nm:?} pid={go}");
            let mut cur = go_tf.get(go).copied();
            let mut depth = 0;
            while let Some(tf) = cur {
                if depth > 24 {
                    break;
                }
                let Some((_, v)) = all.get(&tf) else { break };
                let g = tf_go.get(&tf).copied().unwrap_or(0);
                let n2 = go_name.get(&g).cloned().unwrap_or_default();
                let f = |fl: &str, k: &str| {
                    v.get(fl)
                        .and_then(|x| x.get(k))
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0)
                };
                println!(
                    "   {depth:>2} {n2:<28} pos=({:.6},{:.6},{:.6}) rot=({:.6},{:.6},{:.6},{:.6}) scale=({:.6},{:.6},{:.6})",
                    f("m_LocalPosition", "x"),
                    f("m_LocalPosition", "y"),
                    f("m_LocalPosition", "z"),
                    f("m_LocalRotation", "x"),
                    f("m_LocalRotation", "y"),
                    f("m_LocalRotation", "z"),
                    f("m_LocalRotation", "w"),
                    f("m_LocalScale", "x"),
                    f("m_LocalScale", "y"),
                    f("m_LocalScale", "z")
                );
                cur = tf_father
                    .get(&tf)
                    .copied()
                    .filter(|&x| x != 0 && all.contains_key(&x));
                depth += 1;
            }
        }
    }
}
