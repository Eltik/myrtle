//! Diagnostic: the MAIN-module clock fields of every ParticleSystem (class 198) on GameObjects
//! whose name contains a substring, printed whole: `startDelay`, `startLifetime`, `lengthInSec`,
//! `simulationSpeed`, `prewarm`, `looping`, `useUnscaledTime`, `playOnAwake`, `startDelay`'s
//! full serialized shape (a MinMaxCurve, not a scalar: the exporter reads only its `scalar`).
//!
//! Usage: cargo run --release --example `probe_psmain` -- <bundle.ab> <go-substr>
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
    let mut args = std::env::args().skip(1);
    let (Some(path), Some(want)) = (args.next(), args.next()) else {
        eprintln!("usage: probe_psmain <bundle.ab> <go-substr>");
        return;
    };
    let Ok(data) = std::fs::read(&path) else {
        return;
    };
    let Ok(bundle) = BundleFile::parse(data) else {
        return;
    };
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        let mut gos: HashMap<i64, String> = HashMap::new();
        let mut systems: Vec<Value> = Vec::new();
        for obj in &sf.objects {
            if obj.class_id != 1 && obj.class_id != 198 {
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
                systems.push(v);
            }
        }
        for v in &systems {
            let go = gos
                .get(&v.get("m_GameObject").map(pid).unwrap_or(0))
                .cloned()
                .unwrap_or_default();
            if !go.contains(&want) {
                continue;
            }
            let f = |k: &str| v.get(k).map_or_else(|| "absent".to_string(), Value::to_string);
            println!("=== GO '{go}'");
            for k in [
                "lengthInSec",
                "simulationSpeed",
                "looping",
                "prewarm",
                "playOnAwake",
                "useUnscaledTime",
                "startDelay",
                "moveWithTransform",
                "simulationSpace",
                "scalingMode",
                "moveWithCustomTransform",
            ] {
                let s = f(k);
                println!("  {k:18} {}", if s.len() > 400 { format!("{}..", &s[..400]) } else { s });
            }
            // The per-particle CustomData module (the vertex-stream payload a Ram material reads
            // its dissolve amount from) and the force module, whole, so a clock or a motion
            // that the export summarises can be checked against its source.
            for k in ["ForceModule", "VelocityModule"] {
                let s = f(k);
                println!("  {k:18} {}", if s.len() > 600 { format!("{}..", &s[..600]) } else { s });
            }
            // CustomData: the vector components only (mode, count, each component's MinMaxCurve
            // with its scalar and key list); the colour gradients are omitted.
            if let Some(cd) = v.get("CustomDataModule").and_then(Value::as_object) {
                let mut keys: Vec<&String> = cd.keys().filter(|k| !k.starts_with("color")).collect();
                keys.sort();
                for k in keys {
                    let s = cd[k].to_string();
                    println!("  CustomData.{k:22} {}", if s.len() > 1500 { format!("{}..", &s[..1500]) } else { s });
                }
            }
            if let Some(im) = v.get("InitialModule") {
                let g = |k: &str| im.get(k).map_or_else(|| "absent".to_string(), Value::to_string);
                for k in ["startLifetime", "startSize", "startSpeed"] {
                    let s = g(k);
                    println!("  Initial.{k:10} {}", if s.len() > 300 { format!("{}..", &s[..300]) } else { s });
                }
            }
        }
    }
}
