//! THROWAWAY diagnostic: dump every `MonoBehaviour` field that could carry a COLOUR or
//! ALPHA, across all `MonoBehaviours` in a dynchar bundle — looking for a scene/character
//! tint the exporter never reads.
//!
//! Usage: cargo run --release --example `probe_ctrl` -- <bundle.ab>
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::many_single_char_names,
    clippy::needless_collect
)]
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn walk(prefix: &str, v: &Value, out: &mut Vec<String>, depth: usize) {
    if depth > 4 {
        return;
    }
    match v {
        Value::Object(o) => {
            let keys: Vec<&String> = o.keys().collect();
            let looks_color =
                keys.len() == 4 && ["r", "g", "b", "a"].iter().all(|k| o.contains_key(*k));
            if looks_color {
                let f = |k: &str| o.get(k).and_then(Value::as_f64).unwrap_or(f64::NAN);
                let (r, g, b, a) = (f("r"), f("g"), f("b"), f("a"));
                if (r - 1.0).abs() > 1e-4
                    || (g - 1.0).abs() > 1e-4
                    || (b - 1.0).abs() > 1e-4
                    || (a - 1.0).abs() > 1e-4
                {
                    out.push(format!(
                        "{prefix} = rgba({r:.4},{g:.4},{b:.4},{a:.4})  <-- NON-WHITE"
                    ));
                } else {
                    out.push(format!("{prefix} = rgba(1,1,1,1)"));
                }
                return;
            }
            for (k, vv) in o {
                if k.starts_with("m_") && k != "m_Name" {
                    continue;
                }
                walk(&format!("{prefix}.{k}"), vv, out, depth + 1);
            }
        }
        Value::Array(a) => {
            for (i, vv) in a.iter().take(6).enumerate() {
                walk(&format!("{prefix}[{i}]"), vv, out, depth + 1);
            }
        }
        Value::Number(n) => {
            let lk = prefix.to_lowercase();
            if lk.contains("color")
                || lk.contains("alpha")
                || lk.contains("tint")
                || lk.contains("bright")
                || lk.contains("exposure")
                || lk.contains("gamma")
            {
                out.push(format!("{prefix} = {n}"));
            }
        }
        _ => {}
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle");
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
        let skip: HashSet<i32> = [28, 43, 48, 49, 83, 128, 213].into_iter().collect();
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for obj in &sf.objects {
            if skip.contains(&obj.class_id) {
                continue;
            }
            if let Ok(v) = read_object(&sf, obj) {
                all.insert(obj.path_id, (obj.class_id, v));
            }
        }
        let mut names: HashMap<i64, String> = HashMap::new();
        for (p, (cid, v)) in &all {
            if *cid == 1 {
                names.insert(
                    *p,
                    v.get("m_Name").and_then(Value::as_str).unwrap_or("").into(),
                );
            }
        }
        for (pid, (cid, v)) in &all {
            if *cid != 114 {
                continue;
            }
            let go = v
                .get("m_GameObject")
                .and_then(|g| g.get("m_PathID"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let mut out = Vec::new();
            walk("", v, &mut out, 0);
            let interesting: Vec<&String> = out
                .iter()
                .filter(|l| {
                    l.contains("NON-WHITE") || l.contains('=') && !l.ends_with("rgba(1,1,1,1)")
                })
                .collect();
            if interesting.is_empty() {
                continue;
            }
            println!(
                "\nMB pid={pid} on GO '{}'",
                names.get(&go).cloned().unwrap_or_default()
            );
            for l in interesting.iter().take(14) {
                println!("   {l}");
            }
        }
    }
}
