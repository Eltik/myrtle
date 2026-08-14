//! THROWAWAY diagnostic: dump the raw `CustomDataModule` of named `ParticleSystems`.
//!
//! Motivation: the Ram shader's dissolve threshold is `vs_TEXCOORD2.x + _Amount`, and
//! `vs_TEXCOORD2` is a Unity Custom Vertex Stream — so the `CustomData` value decides how much of
//! the quad survives the dissolve. Civilight Eterna's `guangyun01` lens flare exports a threshold
//! curve of 0.392 -> 0.465 against a dissolve texture averaging 0.622, which renders the flare at
//! ~20% intensity where the game saturates it.
//!
//! The exporter reads `vector{stream}_0.maxCurve` WITHOUT checking `minMaxState`. That is exactly
//! the shape of the `frameOverTime` bug (a constant read as a curve), so print the state, the
//! scalar and both curves for every component and let the data say which it is.
//!
//! Usage: cargo run --release --example `probe_customdata` -- <bundle.ab> [name-substring]

use std::collections::HashMap;

use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn f(v: &serde_json::Value, k: &str) -> String {
    v.get(k)
        .and_then(serde_json::Value::as_f64)
        .map_or_else(|| "-".into(), |n| format!("{n:.4}"))
}

/// `minMaxState`: 0 = constant (uses `scalar`), 1 = curve, 2 = two curves, 3 = two constants.
const fn state_name(s: i64) -> &'static str {
    match s {
        0 => "CONSTANT (scalar)",
        1 => "curve",
        2 => "two curves",
        3 => "TWO CONSTANTS",
        _ => "?",
    }
}

fn dump_mmcurve(label: &str, v: &serde_json::Value) {
    let state = v
        .get("minMaxState")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(-1);
    println!(
        "      {label:<12} state={state} ({}) scalar={} minScalar={}",
        state_name(state),
        f(v, "scalar"),
        f(v, "minScalar"),
    );
    for key in ["maxCurve", "minCurve"] {
        let Some(c) = v.get(key) else { continue };
        let Some(keys) = c.get("m_Curve").and_then(serde_json::Value::as_array) else {
            continue;
        };
        let pts: Vec<String> = keys
            .iter()
            .take(8)
            .map(|k| format!("{}:{}", f(k, "time"), f(k, "value")))
            .collect();
        println!("         {key:<9} {} key(s)  {}", keys.len(), pts.join(" "));
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("bundle path");
    let want = args.next().unwrap_or_default().to_ascii_lowercase();

    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");

    for entry in &bundle.files {
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        // GameObject pathID -> name, and GO -> ParticleSystemRenderer vertex streams.
        let mut go_name: HashMap<i64, String> = HashMap::new();
        let mut go_streams: HashMap<i64, String> = HashMap::new();
        for obj in &sf.objects {
            let Ok(v) = read_object(&sf, obj) else {
                continue;
            };
            if obj.class_id == 1 {
                go_name.insert(obj.path_id, v["m_Name"].as_str().unwrap_or("").to_string());
            } else if obj.class_id == 199
                && let Some(g) = v
                    .get("m_GameObject")
                    .and_then(|x| x.get("m_PathID"))
                    .and_then(serde_json::Value::as_i64)
            {
                let st = v
                    .get("m_VertexStreams")
                    .map_or_else(|| "-".into(), |s| format!("{s}"));
                go_streams.insert(g, st);
            }
        }
        for obj in &sf.objects {
            if obj.class_id != 198 {
                continue; // ParticleSystem
            }
            let Ok(ps) = read_object(&sf, obj) else {
                continue;
            };
            let gid = ps
                .get("m_GameObject")
                .and_then(|x| x.get("m_PathID"))
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0);
            let name = go_name
                .get(&gid)
                .cloned()
                .unwrap_or_else(|| ps["m_Name"].as_str().unwrap_or("").to_string());
            if !want.is_empty() && !name.to_ascii_lowercase().contains(&want) {
                continue;
            }
            let Some(cdm) = ps.get("CustomDataModule") else {
                continue;
            };
            let enabled = cdm
                .get("enabled")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_else(|| {
                    cdm.get("enabled")
                        .and_then(serde_json::Value::as_i64)
                        .unwrap_or(0)
                        != 0
                });
            println!("\n=== {name}   CustomDataModule enabled={enabled}");
            println!(
                "   renderer m_VertexStreams: {}",
                go_streams.get(&gid).map_or("-", String::as_str)
            );
            for stream in 0..2 {
                let mode = cdm
                    .get(format!("mode{stream}").as_str())
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(0);
                let count = cdm
                    .get(format!("vectorComponentCount{stream}").as_str())
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(0);
                println!(
                    "   stream {stream}: mode={mode} ({}) componentCount={count}",
                    if mode == 1 { "Vector" } else { "Colour/off" }
                );
                for comp in 0..4 {
                    let Some(v) = cdm.get(format!("vector{stream}_{comp}").as_str()) else {
                        continue;
                    };
                    dump_mmcurve(&format!("vector{stream}_{comp}"), v);
                }
            }
        }
    }
}
