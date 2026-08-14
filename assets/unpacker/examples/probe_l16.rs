//! THROWAWAY diagnostic: for every scene Material, print its SHADER name and every COLOUR
//! property, so a layer's exported tint can be traced to the property it came from.
//!
//! Motivation: Mlynar's uniform +1.98 Cb blue cast localises to ONE sort-10 layer whose exported
//! tint is [0.4118, 0.4645, 1.0000]. Correcting that layer's HUE is worth 0.235 MADC, and the
//! winning ratio is 2:2:1 — i.e. exactly `(tint x 2).clamp` with blue clipping at 1. The Ram
//! family's x2 convention is already in the exporter but gated on the entrance clip naming
//! `_MainColor`. If this material carries `_MainColor` at the un-doubled value, the gate is
//! missing it.
//!
//! Usage: cargo run --release --example `probe_l16` -- <bundle.ab> [colour-substring]
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::format_push_string
)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: probe_l16 <bundle.ab> [substr]");
    let want = std::env::args().nth(2);
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
            if *cid != 21 {
                continue;
            } // Material
            let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("");
            let shader = v
                .get("m_Shader")
                .and_then(|s| s.get("m_PathID"))
                .and_then(Value::as_i64)
                .and_then(|sp| all.get(&sp))
                .and_then(|(_, sv)| sv.get("m_Name").and_then(Value::as_str))
                .unwrap_or("?");
            let cols = v.get("m_SavedProperties").and_then(|s| s.get("m_Colors"));
            let Some(cols) = cols else { continue };
            let mut line = String::new();
            let entries: Vec<(String, String)> = match cols {
                Value::Object(m) => m.iter().map(|(k, c)| (k.clone(), fmt(c))).collect(),
                Value::Array(a) => a
                    .iter()
                    .filter_map(|e| {
                        let k = e.get("first")?.as_str()?.to_string();
                        Some((k, fmt(e.get("second")?)))
                    })
                    .collect(),
                _ => vec![],
            };
            for (k, val) in &entries {
                line.push_str(&format!("  {k}={val}"));
            }
            if let Some(w) = &want
                && !line.contains(w.as_str())
                && !name.contains(w.as_str())
            {
                continue;
            }
            println!("mat pid {pid} name={name:<26} shader={shader}");
            println!("   {line}");
        }
    }
}

fn fmt(c: &Value) -> String {
    let g = |k: &str| c.get(k).and_then(Value::as_f64).unwrap_or(f64::NAN);
    format!("[{:.4},{:.4},{:.4},{:.4}]", g("r"), g("g"), g("b"), g("a"))
}
