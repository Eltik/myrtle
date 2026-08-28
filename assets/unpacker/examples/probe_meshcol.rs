//! THROWAWAY diagnostic: does a scene mesh carry a VERTEX COLOUR channel in the source bundle?
//!
//! Motivation. Kal'tsit's entrance breaks structurally from t=2 to t=6 (r 0.593, worst 0.279 at
//! t=5) and an alpha sweep on scene layers 43 and 45 puts the MADC minimum at alpha 0.25, with
//! alpha 0.00 WORSE. An intermediate optimum means the content belongs and we draw it too
//! strongly. Their immediate neighbours, scene indices 42 and 46, carry authored `col` arrays
//! averaging 0.444 and 0.400 vertex alpha, which is suspiciously close to the measured optimum.
//!
//! The question this settles: is that colour AUTHORED and lost on the way out, or genuinely
//! absent? `spine.rs` emits `col` only when some vertex differs from opaque white by >0.004, and
//! `mesh.rs` reads colour from vertex channel 3, defaulting to `[1,1,1,1]` when the channel is
//! missing. So "no col array" is consistent with both "no channel" and "channel present, all
//! white", and only the bundle can tell them apart.
//!
//! ⚠️ A format hazard worth checking while here: `read_scalar` handles formats 0 (f32), 1 (half),
//! 2 (UNorm8) and 4 (UNorm16), and returns 0.0 for anything else. An unhandled colour format would
//! therefore yield [0,0,0,0], which is NOT what a missing channel yields, so the two are
//! distinguishable in the output below.
//!
//! Meshes are fingerprinted by TRIANGLE INDEX COUNT, because that is what the exported scene JSON
//! records: Kal'tsit's layers 43, 44 and 45 have 48, 33 and 27 indices.
//!
//! Usage: cargo run --release --example probe_meshcol -- <bundle.ab> [<bundle.ab> ...]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else {
            eprintln!("cannot read {path}");
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            eprintln!("not a bundle: {path}");
            continue;
        };
        println!("=== {path} ===");
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
                continue;
            };
            for obj in &sf.objects {
                if obj.class_id != 43 {
                    continue; // Mesh
                }
                let Ok(v) = read_object(&sf, obj) else {
                    continue;
                };
                let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
                let vd = v.get("m_VertexData");
                let vcount = vd
                    .and_then(|d| d.get("m_VertexCount"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                // index count, the fingerprint the scene JSON records
                let idx_count = v
                    .get("m_IndexBuffer")
                    .and_then(Value::as_array)
                    .map_or(0, |a| a.len() / 2);
                // channel 3 is Color
                let ch = vd
                    .and_then(|d| d.get("m_Channels"))
                    .and_then(Value::as_array);
                let col_ch = ch.and_then(|c| c.get(3));
                let dim = col_ch
                    .and_then(|c| c.get("dimension"))
                    .and_then(Value::as_i64)
                    .unwrap_or(0);
                let fmt = col_ch
                    .and_then(|c| c.get("format"))
                    .and_then(Value::as_i64)
                    .unwrap_or(-1);
                let parsed = super_parse(&v);
                println!(
                    "  mesh {name:<28} verts {vcount:<5} idx {idx_count:<5} colorChannel dim={dim} fmt={fmt}  {parsed}"
                );
            }
        }
    }
}

/// Run the real parser and report the alpha range it produces, so the probe measures what the
/// exporter actually sees rather than re-implementing it.
fn super_parse(v: &Value) -> String {
    let empty: HashMap<String, Vec<u8>> = HashMap::new();
    match unpacker::export::mesh::parse_mesh(v, &empty) {
        None => "parse_mesh -> None (unreadable or streamed)".to_string(),
        Some(m) => {
            let a: Vec<f32> = m.colors.iter().map(|c| c[3]).collect();
            if a.is_empty() {
                return "parse_mesh -> 0 verts".to_string();
            }
            let mn = a.iter().copied().fold(f32::INFINITY, f32::min);
            let mx = a.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let mean = a.iter().sum::<f32>() / a.len() as f32;
            let white = m.colors.iter().all(|c| {
                (c[0] - 1.0).abs() <= 0.004
                    && (c[1] - 1.0).abs() <= 0.004
                    && (c[2] - 1.0).abs() <= 0.004
                    && (c[3] - 1.0).abs() <= 0.004
            });
            format!(
                "alpha {mn:.3}..{mx:.3} mean {mean:.3}  {}",
                if white {
                    "ALL OPAQUE WHITE -> col omitted"
                } else {
                    "has vertex colour -> col emitted"
                }
            )
        }
    }
}
