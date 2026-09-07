//! Diagnostic: the UV span and position extent of named scene quads' meshes.
//!
//! The scene exporter drops a quad whose bounding extent exceeds ten times the camera frame
//! as an "oversize fill" (`DROP[oversize]`), and its own comment anticipates a tiled sheet
//! being a false drop. Which it is turns on the UV span: a sheet whose UVs run over many
//! repeats tiles under REPEAT and is real scrolling texture, one whose UVs span 0..1 is a
//! stretched fill. The UVs live in the MeshFilter's Mesh, which the drop happens before
//! emitting, so nothing in the JSON carries them.
//!
//! Usage: cargo run --release --example `probe_quaduv` -- <bundle.ab> <go-substr>...
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::export::mesh::parse_mesh;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> i64 {
    v.get("m_PathID").and_then(Value::as_i64).unwrap_or(0)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: probe_quaduv <bundle.ab> <go-substr>...");
        return;
    };
    let filters: Vec<String> = args.collect();
    let Ok(data) = std::fs::read(&path) else {
        return;
    };
    let Ok(bundle) = BundleFile::parse(data) else {
        return;
    };
    let mut resources: HashMap<String, Vec<u8>> = HashMap::new();
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            let filename = entry.path.rsplit('/').next().unwrap_or(&entry.path);
            resources.insert(filename.to_string(), entry.data.clone());
        }
    }
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        let mut gos: HashMap<i64, String> = HashMap::new();
        let mut go_to_mesh: HashMap<i64, i64> = HashMap::new();
        let mut meshes: HashMap<i64, Value> = HashMap::new();
        let mut xforms: HashMap<i64, Value> = HashMap::new();
        for obj in &sf.objects {
            if !matches!(obj.class_id, 1 | 4 | 33 | 43) {
                continue;
            }
            let Ok(v) = read_object(&sf, obj) else {
                continue;
            };
            match obj.class_id {
                1 => {
                    gos.insert(
                        obj.path_id,
                        v.get("m_Name")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                    );
                }
                4 => {
                    if let Some(go) = v.get("m_GameObject").map(pid) {
                        xforms.insert(go, v);
                    }
                }
                33 => {
                    if let (Some(go), Some(mesh)) =
                        (v.get("m_GameObject").map(pid), v.get("m_Mesh").map(pid))
                    {
                        go_to_mesh.insert(go, mesh);
                    }
                }
                _ => {
                    meshes.insert(obj.path_id, v);
                }
            }
        }
        for (go, name) in &gos {
            if !filters.is_empty() && !filters.iter().any(|f| name.contains(f.as_str())) {
                continue;
            }
            let Some(mesh_pid) = go_to_mesh.get(go) else {
                continue;
            };
            let Some(mesh_val) = meshes.get(mesh_pid) else {
                println!("{name}\tmesh {mesh_pid} not in bundle (external or built-in)");
                continue;
            };
            let Some(m) = parse_mesh(mesh_val, &resources) else {
                println!("{name}\tmesh {mesh_pid} failed to parse");
                continue;
            };
            let (mut umin, mut umax, mut vmin, mut vmax) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
            for uv in &m.uvs {
                umin = umin.min(uv[0]);
                umax = umax.max(uv[0]);
                vmin = vmin.min(uv[1]);
                vmax = vmax.max(uv[1]);
            }
            let (mut xmin, mut xmax, mut ymin, mut ymax) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
            for p in &m.positions {
                xmin = xmin.min(p[0]);
                xmax = xmax.max(p[0]);
                ymin = ymin.min(p[1]);
                ymax = ymax.max(p[1]);
            }
            let scale = xforms
                .get(go)
                .and_then(|t| t.get("m_LocalScale"))
                .map(|s| {
                    let g = |k: &str| s.get(k).and_then(Value::as_f64).unwrap_or(1.0);
                    (g("x"), g("y"))
                })
                .unwrap_or((1.0, 1.0));
            println!(
                "{name}\tverts {}\tuv u {umin:.3}..{umax:.3} v {vmin:.3}..{vmax:.3} (span {:.3} x {:.3})\tlocal x {xmin:.3}..{xmax:.3} y {ymin:.3}..{ymax:.3}\tlocalScale ({:.3}, {:.3})",
                m.positions.len(),
                umax - umin,
                vmax - vmin,
                scale.0,
                scale.1
            );
        }
    }
}
