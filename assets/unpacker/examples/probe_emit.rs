//! THROWAWAY diagnostic: dump the RAW `EmissionModule` of every ParticleSystem whose GameObject
//! name matches a filter — `rateOverTime` / `rateOverDistance` MinMaxCurve state, scalar,
//! minScalar and the raw curve key values, plus `startLifetime` and `maxNumParticles`.
//!
//! Motivation: Skadi's crown shoal renders 9 live particles where the game shows >=12 visible.
//! The exported rate curve peaks at 2.0/s which, against the 4.2 s lifetime, cannot produce 12.
//! Either the scalar is being applied once where Unity applies it differently, or the curve is
//! normalised on a different axis. This prints the unprocessed fields so the exported form can be
//! checked against them term by term.
//!
//! Usage: cargo run --release --example probe_emit -- <bundle.ab> <name-substr>

use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn pid(v: &Value) -> Option<i64> { v.get("m_PathID").and_then(Value::as_i64) }
fn f(v: &Value, k: &str) -> f64 { v.get(k).and_then(Value::as_f64).unwrap_or(f64::NAN) }

fn dump_mm(tag: &str, mm: &Value) {
    let state = mm.get("minMaxState").and_then(Value::as_i64).unwrap_or(-1);
    println!("    {tag}: minMaxState={state} scalar={:.4} minScalar={:.4}", f(mm, "scalar"), f(mm, "minScalar"));
    for which in ["maxCurve", "minCurve"] {
        if let Some(c) = mm.get(which).and_then(|c| c.get("m_Curve")).and_then(Value::as_array) {
            let vals: Vec<String> = c.iter().take(8)
                .map(|k| format!("{:.3}@{:.2}", f(k, "value"), f(k, "time"))).collect();
            let hi = c.iter().map(|k| f(k, "value")).fold(f64::MIN, f64::max);
            println!("      {which}: {} keys, max value {hi:.4}  first: {}", c.len(), vals.join(" "));
        }
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle");
    let want = std::env::args().nth(2).unwrap_or_default();
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") { continue; }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for obj in &sf.objects {
            if let Ok(v) = read_object(&sf, obj) { all.insert(obj.path_id, (obj.class_id, v)); }
        }
        for (_p, (cid, v)) in &all {
            if *cid != 198 { continue; }
            let name = v.get("m_GameObject").and_then(pid)
                .and_then(|g| all.get(&g))
                .and_then(|(_, g)| g.get("m_Name").and_then(Value::as_str))
                .unwrap_or("?");
            if !want.is_empty() && !name.contains(&want) { continue; }
            println!("\n  PS '{name}'  maxNumParticles={:?}  lengthInSec={:.3}  looping={:?}  prewarm={:?}",
                v.get("InitialModule").and_then(|i| i.get("maxNumParticles")).and_then(Value::as_i64),
                f(v, "lengthInSec"),
                v.get("looping").and_then(Value::as_bool),
                v.get("prewarm").and_then(Value::as_bool));
            if let Some(im) = v.get("InitialModule") {
                if let Some(l) = im.get("startLifetime") { dump_mm("startLifetime", l); }
                if let Some(s) = im.get("startSize") { dump_mm("startSize", s); }
            }
            if let Some(em) = v.get("EmissionModule") {
                println!("    emission enabled={:?}", em.get("enabled").and_then(Value::as_bool));
                for k in ["rateOverTime", "rateOverDistance"] {
                    if let Some(r) = em.get(k) { dump_mm(k, r); }
                }
                if let Some(b) = em.get("m_Bursts").and_then(Value::as_array) {
                    println!("    bursts: {}", b.len());
                }
            }
        }
    }
}
