//! THROWAWAY diagnostic: which MESH does each mesh-render `ParticleSystem` actually reference?
//!
//! Motivation: Civilight Eterna's entrance is confined to a hard 1920x1080 window in game while
//! ours fills the full 2340. Attribution on our own render says the bar columns are painted by a
//! handful of systems, dominated by a single huge quad (sys30: one particle, 374k px^2, box 1054
//! px wide on a 900 px canvas). Its exported mesh is the generic SQUARE `Plane001`
//! (0.254 x 0.254, aspect 1.0) — and six sibling background planes carry the same square.
//!
//! But the bundle also ships a mesh named `bg_02` at 0.210 x 0.118 — **aspect 1.7779, i.e. 16:9**
//! — and that mesh appears NOWHERE in the exported particle JSON (aspect histogram across all 27
//! mesh systems: 1.0 x24, 1.4944, 0.8469, 0.5476). `bg_01` (0.8469) IS exported, so the exporter
//! resolves some of these correctly and not others.
//!
//! If the `bg_*` systems should be drawing `bg_02` and we hand them a square instead, we paint a
//! square backdrop where the game paints a 16:9 one — which is exactly the shape of the missing
//! aperture, and a corpus-wide data-fidelity bug rather than a per-skin constant.
//!
//! Usage: cargo run --release --example `probe_psmesh` -- <bundle.ab>
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn name_of(all: &HashMap<i64, (i32, Value)>, pid: Option<i64>) -> String {
    pid.and_then(|p| all.get(&p))
        .and_then(|(_, v)| v.get("m_Name"))
        .and_then(Value::as_str)
        .unwrap_or("?")
        .to_string()
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: probe_psmesh <bundle.ab>");
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

        // Mesh pathID -> (name, aspect), so a renderer's reference can be reported by NAME.
        let mut meshes: HashMap<i64, (String, f64, f64)> = HashMap::new();
        for (pid, (cid, v)) in &all {
            if *cid != 43 {
                continue;
            }
            let n = v
                .get("m_Name")
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_string();
            let ext = v.get("m_LocalAABB").and_then(|a| a.get("m_Extent"));
            let g = |k: &str| {
                ext.and_then(|e| e.get(k))
                    .and_then(Value::as_f64)
                    .unwrap_or(f64::NAN)
            };
            meshes.insert(*pid, (n, g("x") * 2.0, g("y") * 2.0));
        }

        // ParticleSystemRenderer is class 199; its `m_Mesh` is the mesh drawn in Mesh render mode.
        let mut rows: Vec<(String, String, String, f64)> = Vec::new();
        for (cid, v) in all.values() {
            if *cid != 199 {
                continue;
            }
            let go = v
                .get("m_GameObject")
                .and_then(|g| g.get("m_PathID"))
                .and_then(Value::as_i64);
            let go_name = name_of(&all, go);
            let mode = v.get("m_RenderMode").and_then(Value::as_i64).unwrap_or(-1);
            if mode != 4 {
                continue; // 4 == Mesh
            }
            let mesh_pid = v
                .get("m_Mesh")
                .and_then(|m| m.get("m_PathID"))
                .and_then(Value::as_i64);
            let (mname, w, h) = mesh_pid
                .and_then(|p| meshes.get(&p).cloned())
                .unwrap_or_else(|| ("<unresolved>".into(), f64::NAN, f64::NAN));
            rows.push((go_name, mname, format!("{w:.4}x{h:.4}"), w / h));
        }
        rows.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
        println!(
            "{} mesh-mode ParticleSystemRenderer(s) in {}",
            rows.len(),
            entry.path
        );
        println!("  {:38} {:22} {:18} aspect", "GameObject", "mesh", "size");
        for (go, m, size, a) in &rows {
            let flag = if (a - 1.7778).abs() < 0.01 {
                "  <== 16:9"
            } else {
                ""
            };
            println!("  {go:38} {m:22} {size:18} {a:.4}{flag}");
        }
    }
}
