//! THROWAWAY diagnostic: what is the ASPECT of the quad the entrance root renders through?
//!
//! Motivation: Civilight Eterna's entrance provably renders into exactly 1920x1080 inside a
//! 2340x1080 screen (hard 16:9 aperture, symmetric 210px black pillarbox, coextensive with the
//! cinematic, gone the instant it hands back). Reproduced from a cold emulator boot, so it is
//! real game behaviour. Eyjafjalla the Hvit Aska is identical on every camera field we export
//! and does NOT pillarbox on the same route.
//!
//! Already cleared: Camera `m_NormalizedViewPortRect` is 1x1 on both, neither bundle ships a
//! RenderTexture, no MonoBehaviour names an aspect/viewport/letterbox field, skin_table has
//! nothing, and the entrance director itself carries only `_effects`, `_mainCamera` and
//! `_params{charVoiceOffset,duration,fadeColor}` -- zero rendering params.
//!
//! Remaining candidate: the entrance GameObject also carries a MeshFilter + MeshRenderer, i.e.
//! the entrance draws through a QUAD. If that quad is authored 16:9 for Civilight Eterna and
//! wider for Eyjafjalla, then the aperture is simply "the entrance quad's aspect, height-fit"
//! -- a scalable rule rather than a per-skin constant. This dumps the quad's local AABB.
//!
//! Usage: cargo run --release --example probe_entmesh -- <bundle.ab> [<bundle.ab> ...]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn f(v: Option<&Value>) -> f64 {
    v.and_then(Value::as_f64).unwrap_or(f64::NAN)
}

fn main() {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    assert!(!paths.is_empty(), "usage: probe_entmesh <bundle.ab> [...]");

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

            // Every mesh in the bundle, with its AABB aspect -- the entrance quad will stand out
            // as a large, near-flat, single-quad mesh.
            let mut meshes: Vec<(i64, &Value)> =
                all.iter().filter(|(_, (cid, _))| *cid == 43).map(|(p, (_, v))| (*p, v)).collect();
            meshes.sort_unstable_by_key(|(p, _)| *p);
            println!("  {} mesh(es)", meshes.len());
            for (pid, v) in &meshes {
                let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
                let ext = v.get("m_LocalAABB").and_then(|a| a.get("m_Extent"));
                let (ex, ey, ez) = (
                    f(ext.and_then(|e| e.get("x"))),
                    f(ext.and_then(|e| e.get("y"))),
                    f(ext.and_then(|e| e.get("z"))),
                );
                let (w, h) = (ex * 2.0, ey * 2.0);
                println!(
                    "   [mesh pid={pid}] {name:32} size={w:9.3} x {h:9.3} x {ez:7.3}  aspect={:.4}",
                    w / h
                );
            }

            // Which mesh does each *entrance* GameObject actually point at?
            for (pid, (cid, v)) in &all {
                if *cid != 33 {
                    continue; // MeshFilter
                }
                let go = v.get("m_GameObject").and_then(|g| g.get("m_PathID")).and_then(Value::as_i64);
                let go_name = go
                    .and_then(|g| all.get(&g))
                    .and_then(|(_, gv)| gv.get("m_Name"))
                    .and_then(Value::as_str)
                    .unwrap_or("?");
                if !go_name.to_ascii_lowercase().contains("entrance") {
                    continue;
                }
                let mesh_pid = v.get("m_Mesh").and_then(|m| m.get("m_PathID")).and_then(Value::as_i64);
                println!("   ==> MeshFilter pid={pid} on {go_name} -> mesh pid={mesh_pid:?}");
            }
        }
    }
}
