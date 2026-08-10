//! THROWAWAY diagnostic: dump every scalar field of the `dyn_entrance_*` object tree, so the
//! entrance CONFIG (as opposed to the illust) can be diffed between two skins.
//!
//! Motivation: Civilight Eterna's (`char_4134_cetsyr_epoque#50`) entrance provably renders into
//! exactly 1920x1080 centred in a 2340x1080 screen -- a hard 16:9 aperture, symmetric 210px
//! black pillarbox, coextensive with the cinematic and gone the instant it hands back to the
//! viewer. Reproduced from a cold boot on the emulator, so it is real game behaviour, not a
//! capture artifact. Eyjafjalla the Hvit Aska (`char_1016_agoat2_epoque#34`) is identical on
//! every camera field we export (`aspect` 1.0, `cameraViewPx` 2048.0, `cameraSizePx` 1050) and
//! does NOT pillarbox on the same route.
//!
//! `probe_viewport` already cleared the obvious suspects: both Cameras carry a full
//! `m_NormalizedViewPortRect` (1x1), neither bundle ships a RenderTexture, and no MonoBehaviour
//! anywhere names an aspect/viewport/letterbox field. skin_table carries nothing either. So if
//! anything in the data predicts the aperture it lives on the entrance object itself.
//!
//! Usage: cargo run --release --example probe_entrancecfg -- <bundle.ab> [<bundle.ab> ...]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

/// Flatten to `dotted.path = scalar`, skipping the bulky arrays (curves, meshes) that would
/// drown the diff. Arrays are summarised by length and first element instead.
fn flatten(v: &Value, path: &str, out: &mut Vec<(String, String)>) {
    match v {
        Value::Object(m) => {
            for (k, sub) in m {
                let p = if path.is_empty() { k.clone() } else { format!("{path}.{k}") };
                flatten(sub, &p, out);
            }
        }
        Value::Array(a) => {
            if a.len() > 8 {
                out.push((format!("{path}[]"), format!("len={} first={}", a.len(), a[0].to_string().chars().take(60).collect::<String>())));
            } else {
                for (i, sub) in a.iter().enumerate() {
                    flatten(sub, &format!("{path}[{i}]"), out);
                }
            }
        }
        scalar => out.push((path.to_string(), scalar.to_string())),
    }
}

fn main() {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    assert!(!paths.is_empty(), "usage: probe_entrancecfg <bundle.ab> [...]");

    for path in &paths {
        println!("\n######## {path}");
        let Ok(data) = std::fs::read(path) else { continue };
        let Ok(bundle) = BundleFile::parse(data) else { continue };
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
            let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
            for o in &sf.objects {
                if let Ok(v) = read_object(&sf, o) {
                    all.insert(o.path_id, (o.class_id, v));
                }
            }

            // Anything whose own m_Name, or whose owning GameObject's name, mentions "entrance".
            let mut hits: Vec<(i64, i32, String)> = Vec::new();
            for (pid, (cid, v)) in &all {
                let own = v.get("m_Name").and_then(Value::as_str).unwrap_or("");
                let go = v
                    .get("m_GameObject")
                    .and_then(|g| g.get("m_PathID"))
                    .and_then(Value::as_i64)
                    .and_then(|g| all.get(&g))
                    .and_then(|(_, gv)| gv.get("m_Name"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                if own.to_ascii_lowercase().contains("entrance") || go.to_ascii_lowercase().contains("entrance") {
                    hits.push((*pid, *cid, if own.is_empty() { go.to_string() } else { own.to_string() }));
                }
            }
            hits.sort();
            for (pid, cid, name) in &hits {
                let (_, v) = &all[pid];
                let mut fields = Vec::new();
                flatten(v, "", &mut fields);
                fields.sort();
                println!("  ---- [{name}] class={cid} pid={pid}  ({} fields)", fields.len());
                for (k, val) in &fields {
                    println!("       {k} = {val}");
                }
            }
            if hits.is_empty() {
                println!("  (no object named *entrance* in {})", entry.path);
            }
        }
    }
}
