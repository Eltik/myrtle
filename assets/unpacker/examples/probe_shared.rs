//! THROWAWAY: do OTHER clips animate the same transform paths as the camera clip?
//! If so, two clocks compose into one baked camera curve.
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};
fn main() {
    let path = std::env::args().nth(1).unwrap();
    let target = std::env::args().nth(2).unwrap_or_else(|| "epoque#28_Start".into());
    let data = std::fs::read(&path).unwrap();
    let bundle = BundleFile::parse(data).unwrap();
    for entry in &bundle.files {
        let l = entry.path.to_ascii_lowercase();
        if l.ends_with(".ress") || l.ends_with(".resource") { continue; }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        let skip: HashSet<i32> = [28,43,48,49,83,128,213].into_iter().collect();
        let mut clips: Vec<(String, f64, Vec<(i64,u32)>)> = Vec::new();
        for obj in &sf.objects {
            if obj.class_id != 74 || skip.contains(&obj.class_id) { continue; }
            let Ok(v) = read_object(&sf, obj) else { continue };
            let n = v.get("m_Name").and_then(Value::as_str).unwrap_or("").to_string();
            let stop = v.get("m_MuscleClip").and_then(|m| m.get("m_StopTime")).and_then(Value::as_f64).unwrap_or(f64::NAN);
            let mut bs2 = Vec::new();
            if let Some(bs) = v.get("m_ClipBindingConstant").and_then(|b| b.get("genericBindings")).and_then(Value::as_array) {
                for b in bs {
                    let ty = b.get("typeID").and_then(Value::as_i64).unwrap_or(-1);
                    let p = b.get("path").and_then(Value::as_i64).unwrap_or(0) as u32;
                    bs2.push((ty, p));
                }
            }
            clips.push((n, stop, bs2));
        }
        let Some(cam) = clips.iter().find(|c| c.0 == target) else { println!("clip '{target}' not found"); return };
        let cam_paths: HashSet<u32> = cam.2.iter().filter(|(t,_)| *t == 4 || *t == 20).map(|(_,p)| *p).collect();
        println!("camera clip '{}' (stop={:.4}) animates {} distinct transform/camera paths:", cam.0, cam.1, cam_paths.len());
        for (t,p) in &cam.2 { if *t==4 || *t==20 { println!("    typeID {t:>3}  path {p}"); } }
        println!("\nOTHER clips touching any of those paths:");
        let mut any = false;
        for c in &clips {
            if c.0 == cam.0 { continue; }
            let shared: Vec<(i64,u32)> = c.2.iter().filter(|(_,p)| cam_paths.contains(p)).cloned().collect();
            if shared.is_empty() { continue; }
            any = true;
            println!("  '{}' (stop={:.4}) shares {} binding(s): {:?}", c.0, c.1, shared.len(), shared);
        }
        if !any { println!("  NONE — no other clip animates the camera's transform paths."); }
        return;
    }
}
