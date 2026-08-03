//! THROWAWAY diagnostic: for every MeshRenderer in a dynchar bundle, print the GO name, its
//! material name + shader, and the material fields the SCENE export path does NOT carry —
//! `_RamTex`, `_MainColor`, `_TintColor`.
//!
//! Motivation: Mlynar's t=4 warm deficit and t=13 bottom-centre deficit both attribute to
//! "a localised brightening the exported scene does not contain". But per-layer ABLATION only
//! measures what we DO draw: a layer we render at ~zero looks identical to a missing one.
//! The scene ram block (`export/spine.rs`) emits only dissolve/disturb — the Ram family's
//! per-pixel COLOUR RAMP (`_RamTex`) and its `_MainColor` are dropped for scene layers, while
//! the PARTICLE path exports both. So a scene layer whose colour comes from a ramp would draw
//! with whatever `_MainTex` holds and no palette at all.
//!
//! Usage: cargo run --release --example probe_ramscene -- <bundle.ab>

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

/// `m_TexEnvs` is a list of `{first: name, second: {m_Texture: {m_PathID}, ...}}`.
fn texenv<'a>(mat: &'a Value, name: &str) -> Option<&'a Value> {
    // `m_TexEnvs` / `m_Colors` / `m_Floats` deserialise as MAPS here, not as the
    // {first, second} pair lists the Unity docs describe.
    mat.get("m_SavedProperties")?.get("m_TexEnvs")?.get(name)?.get("m_Texture")
}

fn color(mat: &Value, name: &str) -> Option<Vec<f64>> {
    let c = mat.get("m_SavedProperties")?.get("m_Colors")?.get(name)?;
    Some(vec![c.get("r")?.as_f64()?, c.get("g")?.as_f64()?, c.get("b")?.as_f64()?, c.get("a")?.as_f64()?])
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle path");
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");

    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
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
        // GameObject names, so each renderer can be reported by the object it sits on.
        let mut go_name: HashMap<i64, String> = HashMap::new();
        for (p, (cls, v)) in &all {
            if *cls == 1 {
                if let Some(n) = v.get("m_Name").and_then(Value::as_str) {
                    go_name.insert(*p, n.to_string());
                }
            }
        }

        // class 23 = MeshRenderer, 33 = MeshFilter, 137 = SkinnedMeshRenderer
        for (_p, (cls, v)) in &all {
            if *cls != 23 && *cls != 137 {
                continue;
            }
            let name = v
                .get("m_GameObject")
                .and_then(pid)
                .and_then(|g| go_name.get(&g))
                .cloned()
                .unwrap_or_else(|| "?".into());
            let Some(mats) = v.get("m_Materials").and_then(Value::as_array) else { continue };
            for mr in mats {
                let Some(mp) = pid(mr) else { continue };
                let Some((21, mat)) = all.get(&mp) else { continue };
                let mname = mat.get("m_Name").and_then(Value::as_str).unwrap_or("?");
                // The raw material carries only an external shader REF; the name is resolved
                // elsewhere. Bindings are what matter here, so report the ref's pathID.
                let shader = mat.get("m_Shader").and_then(pid).map(|p| p.to_string()).unwrap_or_else(|| "?".into());
                let ram = texenv(mat, "_RamTex").and_then(pid).filter(|&p| p != 0);
                let main = texenv(mat, "_MainTex").and_then(pid).filter(|&p| p != 0);
                println!(
                    "GO {:<40} mat {:<38} sh {:<22} _RamTex {:<8} _MainTex {:<8} _MainColor {:?} _TintColor {:?}",
                    name,
                    mname,
                    shader,
                    ram.map(|p| p.to_string()).unwrap_or_else(|| "-".into()),
                    main.map(|p| p.to_string()).unwrap_or_else(|| "-".into()),
                    color(mat, "_MainColor").map(|c| c.iter().map(|x| (x * 100.0).round() / 100.0).collect::<Vec<_>>()),
                    color(mat, "_TintColor").map(|c| c.iter().map(|x| (x * 100.0).round() / 100.0).collect::<Vec<_>>()),
                );
            }
        }
    }
}
