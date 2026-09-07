//! Diagnostic: every Camera (typeID 20) in a dynchar bundle with its clear flags and the FULL
//! `m_BackGroundColor`, alpha included. The exporter keeps only the entrance camera's rgb
//! (`entranceClearColor`), so the alpha the render target starts from, which the shader
//! blend states (`srcBlendAlpha`/`destBlendAlpha`) then act on, was never read.
//!
//! Usage: cargo run --release --example `probe_camclear` -- <bundle.ab> [<bundle.ab>...]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> i64 {
    v.get("m_PathID").and_then(Value::as_i64).unwrap_or(0)
}

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            continue;
        };
        let short = path.rsplit('/').next().unwrap_or(&path).to_string();
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
                continue;
            };
            let mut gos: HashMap<i64, String> = HashMap::new();
            let mut cams: Vec<Value> = Vec::new();
            for obj in &sf.objects {
                if obj.class_id != 1 && obj.class_id != 20 {
                    continue;
                }
                let Ok(v) = read_object(&sf, obj) else {
                    continue;
                };
                if obj.class_id == 1 {
                    gos.insert(
                        obj.path_id,
                        v.get("m_Name")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                    );
                } else {
                    cams.push(v);
                }
            }
            for cam in cams {
                let go = gos
                    .get(&cam.get("m_GameObject").map(pid).unwrap_or(0))
                    .cloned()
                    .unwrap_or_default();
                let c = cam.get("m_BackGroundColor");
                let f = |k: &str| {
                    c.and_then(|o| o.get(k))
                        .and_then(Value::as_f64)
                        .unwrap_or(-1.0)
                };
                println!(
                    "{short}\tcam '{go}'\tclearFlags {}\tortho {}\tsize {}\tdepth {}\tbg rgba ({:.4}, {:.4}, {:.4}, {:.4})\ttargetTex {}",
                    cam.get("m_ClearFlags")
                        .and_then(Value::as_i64)
                        .unwrap_or(-1),
                    cam.get("orthographic")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    cam.get("orthographic size")
                        .and_then(Value::as_f64)
                        .unwrap_or(-1.0),
                    cam.get("m_Depth").and_then(Value::as_f64).unwrap_or(-1.0),
                    f("r"),
                    f("g"),
                    f("b"),
                    f("a"),
                    cam.get("m_TargetTexture").map(pid).unwrap_or(0)
                );
            }
        }
    }
}
