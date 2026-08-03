//! THROWAWAY diagnostic: for every `renderMode: mesh` ParticleSystem, report WHY its mesh
//! geometry failed to export.
//!
//! Motivation: 1195 of 1818 mesh-particle systems in the corpus (65.7%, across 59 of 92
//! skins) ship with no geometry and are silently dropped by the frontend — including 76 on
//! Virtuosa alone. The exporter only accepts a mesh whose PPtr resolves to a class-43 object
//! IN THE SAME serialized file, so the failures are expected to split into: an EXTERNAL
//! reference (m_FileID != 0), a null PPtr, a missing local object, or a mesh that
//! `parse_mesh` cannot decode (streamed / compressed vertices).
//!
//! Usage: cargo run --release --example probe_meshfail -- <bundle.ab> [<bundle.ab>...]

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}
fn fid(v: &Value) -> i64 {
    v.get("m_FileID").and_then(Value::as_i64).unwrap_or(0)
}

fn main() {
    let mut tally: HashMap<&'static str, usize> = HashMap::new();
    let mut ok = 0usize;
    let mut total = 0usize;
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else { continue };
        let Ok(bundle) = BundleFile::parse(data) else { continue };
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
            let skip: HashSet<i32> = [28, 48, 49, 83, 128, 213].into_iter().collect();
            let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
            for obj in &sf.objects {
                if skip.contains(&obj.class_id) {
                    continue;
                }
                if let Ok(v) = read_object(&sf, obj) {
                    all.insert(obj.path_id, (obj.class_id, v));
                }
            }
            // ParticleSystemRenderer is class 199; find those whose render mode is Mesh (4).
            for (_, (cid, v)) in &all {
                if *cid != 199 {
                    continue;
                }
                let rm = v.get("m_RenderMode").and_then(Value::as_i64).unwrap_or(-1);
                if rm != 4 {
                    continue;
                }
                total += 1;
                let Some(mp) = v.get("m_Mesh") else {
                    *tally.entry("no m_Mesh field").or_default() += 1;
                    continue;
                };
                let f = fid(mp);
                let p = pid(mp).unwrap_or(0);
                if p == 0 {
                    *tally.entry("null PPtr (pathID 0)").or_default() += 1;
                    continue;
                }
                if f != 0 {
                    *tally.entry("EXTERNAL ref (m_FileID != 0)").or_default() += 1;
                    continue;
                }
                match all.get(&p) {
                    None => *tally.entry("local pathID not found in file").or_default() += 1,
                    Some((43, mv)) => {
                        if unpacker::export::mesh::parse_mesh(mv, &HashMap::new()).is_some() {
                            ok += 1;
                        } else {
                            *tally.entry("class 43 but parse_mesh failed (streamed/compressed)").or_default() += 1;
                        }
                    }
                    Some((c, _)) => {
                        eprintln!("  (pathID resolves to class {c}, not 43)");
                        *tally.entry("resolves to wrong class").or_default() += 1;
                    }
                }
            }
        }
    }
    println!("mesh-render renderers seen: {total}   geometry OK: {ok}");
    let mut v: Vec<_> = tally.into_iter().collect();
    v.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    for (k, n) in v {
        println!("  {n:>6}  {k}");
    }
}
