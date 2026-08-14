//! THROWAWAY diagnostic: RAW Unity mesh UVs vs what the exporter writes.
//!
//! Virtuosa's scene layer 113 is a white wash over 63.7% of the frame, correctly REGISTERED
//! (best shift 0,0) but correlating only 0.315 with the game's luminance across its own
//! footprint. Its texture carries plenty of alpha (p90 0.67) — the amplitude is landing in the
//! wrong places. With 19 verts / 17 tris covering most of the frame, UV interpolation is what
//! distributes it, and `spine.rs` BAKES the material's `_MainTex_ST` into the UVs at export.
//! So: print the raw mesh UVs and the ST, to see whether the baked result is faithful.
//!
//! Usage: cargo run --release --example `probe_uv` -- <bundle.ab> [vertcount]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::export::mesh::parse_mesh;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("usage: probe_uv <bundle.ab> [vertcount]");
    let want: Option<usize> = a.next().and_then(|s| s.parse().ok());
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
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for o in &sf.objects {
            if let Ok(v) = read_object(&sf, o) {
                all.insert(o.path_id, (o.class_id, v));
            }
        }
        for (pid, (cid, v)) in &all {
            if *cid != 43 {
                continue;
            }
            let Some(m) = parse_mesh(v, &HashMap::new()) else {
                continue;
            };
            if let Some(w) = want
                && m.uvs.len() != w
            {
                continue;
            }
            if m.uvs.is_empty() {
                continue;
            }
            let (mut u0, mut u1, mut v0, mut v1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
            for uv in &m.uvs {
                u0 = u0.min(uv[0]);
                u1 = u1.max(uv[0]);
                v0 = v0.min(uv[1]);
                v1 = v1.max(uv[1]);
            }
            let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
            println!(
                "  mesh pid {pid} name={name:<28} verts={:<4} tris={:<4} rawUV u[{u0:.3},{u1:.3}] v[{v0:.3},{v1:.3}]",
                m.uvs.len(),
                m.indices.len() / 3
            );
            if want.is_some() {
                for (i, uv) in m.uvs.iter().enumerate() {
                    println!("      v{i:<3} uv ({:.4}, {:.4})", uv[0], uv[1]);
                }
            }
        }
    }
}
