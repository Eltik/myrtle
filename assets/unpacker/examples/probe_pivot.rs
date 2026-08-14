//! THROWAWAY diagnostic: scan every dynchar bundle for `ParticleSystemRenderer.m_Pivot`
//! values that are NOT zero, plus the render mode/alignment that go with them.
//!
//! Motivation: Unity offsets a billboard by `pivot × particleSize`. The exporter reads no
//! pivot at all, so any non-zero one is a silent per-particle misplacement. Before adding a
//! field to the schema, find out whether the corpus actually uses it.
//!
//! Usage: cargo run --release --example `probe_pivot` -- <dir-of-bundles>
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::many_single_char_names
)]

use serde_json::Value;
use std::path::PathBuf;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn main() {
    let dir = std::env::args().nth(1).expect("bundle dir");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("read dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "ab"))
        .collect();
    files.sort();
    let (mut total, mut nonzero) = (0usize, 0usize);
    for path in &files {
        let Ok(data) = std::fs::read(path) else {
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
                if obj.class_id != 199 {
                    continue; // ParticleSystemRenderer
                }
                let Ok(v) = read_object(&sf, obj) else {
                    continue;
                };
                let Some(p) = v.get("m_Pivot") else { continue };
                total += 1;
                let g = |k: &str| p.get(k).and_then(Value::as_f64).unwrap_or(0.0);
                let (x, y, z) = (g("x"), g("y"), g("z"));
                if x.abs() > 1e-6 || y.abs() > 1e-6 || z.abs() > 1e-6 {
                    nonzero += 1;
                    if nonzero <= 25 {
                        println!(
                            "{}  pivot=({x:.4},{y:.4},{z:.4})  renderMode={:?} align={:?}",
                            path.file_name().unwrap().to_string_lossy(),
                            v.get("m_RenderMode").and_then(Value::as_i64),
                            v.get("m_RenderAlignment").and_then(Value::as_i64),
                        );
                    }
                }
            }
        }
    }
    println!("\nParticleSystemRenderers scanned: {total};  NON-ZERO pivot: {nonzero}");
}
