//! THROWAWAY diagnostic: dump the ENTRANCE director's `_params` in full.
//!
//! Motivation: `ENTRANCE_FADE_IN` (0.85 s) and `ENTRANCE_FADE_HOLD` (0.2 s) are hand-picked
//! GLOBAL constants — the renderer's own comment says 0.85 was "measured on Virtuosa alone".
//! They are now the dominant residual on Wiš'adel (her t=12 beat scores 102.8 against 22-40
//! everywhere else) and shortening them globally costs Executor +13.3. If the director ships a
//! fade DURATION alongside `fadeColor`, the ramp becomes per-skin data instead of a compromise.
//!
//! Usage: cargo run --release --example `probe_fadecfg` -- <bundle.ab>
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn walk(prefix: &str, v: &Value, out: &mut Vec<String>) {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                walk(&format!("{prefix}.{k}"), x, out);
            }
        }
        Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                walk(&format!("{prefix}[{i}]"), x, out);
            }
        }
        _ => out.push(format!("{prefix} = {v}")),
    }
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: probe_fadecfg <bundle.ab>");
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
        let mut pids: Vec<i64> = all.keys().copied().collect();
        pids.sort_unstable();
        for pid in pids {
            let (cid, v) = &all[&pid];
            if *cid != 114 || v.get("_mainCamera").is_none() {
                continue;
            }
            let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
            println!("== director MonoBehaviour pid={pid} name={name}");
            let mut out = Vec::new();
            if let Some(p) = v.get("_params") {
                walk("_params", p, &mut out);
            }
            for k in ["_duration", "_fadeTime", "_fadeDuration", "_delay"] {
                if let Some(x) = v.get(k) {
                    walk(k, x, &mut out);
                }
            }
            out.sort();
            for l in out {
                println!("   {l}");
            }
        }
    }
}
