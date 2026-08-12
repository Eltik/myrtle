//! THROWAWAY diagnostic: is the entrance Main Camera ORTHOGRAPHIC or PERSPECTIVE?
//!
//! Motivation: Whislash the Decadenza's `_Start` clip animates `Main Camera` position on Z ONLY
//! (3.0 -> 1.22), leaving X/Y at 0, and animates the RIG ROOT in X/Y. Under an orthographic camera
//! a Z move is pure depth and cannot reframe anything — so the exporter reads no zoom and her
//! entrance renders at a single static framing. Under a PERSPECTIVE camera the same Z move IS the
//! dolly, and on-screen scale goes as 1/z: 3.0/1.22 = 2.46x, which is what her render is off by.
//!
//! Usage: cargo run --release --example probe_camtype -- <bundle.ab>...
use serde_json::Value;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else { continue };
        let Ok(bundle) = BundleFile::parse(data) else { continue };
        println!("== {}", path.rsplit('/').next().unwrap_or(&path));
        for entry in &bundle.files {
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
            let mut names = std::collections::HashMap::new();
            for obj in &sf.objects {
                if obj.class_id == 1 {
                    if let Ok(v) = read_object(&sf, obj) {
                        names.insert(obj.path_id, v["m_Name"].as_str().unwrap_or("").to_string());
                    }
                }
            }
            for obj in &sf.objects {
                if obj.class_id != 20 { continue; }
                let Ok(v) = read_object(&sf, obj) else { continue };
                let go = v.get("m_GameObject").and_then(|g| g.get("m_PathID")).and_then(Value::as_i64).unwrap_or(0);
                let nm = names.get(&go).cloned().unwrap_or_default();
                let f = |k: &str| v.get(k).and_then(Value::as_f64);
                let ortho = v.get("orthographic").and_then(Value::as_i64)
                    .or_else(|| v.get("orthographic").and_then(|x| x.as_bool()).map(i64::from));
                println!(
                    "   camera '{nm}'  orthographic={:?}  orthoSize={:?}  fov={:?}  near={:?}  far={:?}",
                    ortho, f("orthographic size"), f("field of view"), f("near clip plane"), f("far clip plane")
                );
            }
        }
    }
}
