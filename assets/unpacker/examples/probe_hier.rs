//! THROWAWAY diagnostic: print the transform hierarchy around the entrance camera.
//!
//! Motivation: Whislash the Decadenza's clip binds `Dummy002/Main Camera` (camera Z) and `<root>`
//! (rig X/Y), and her art binds under `Dummy002/Main Camera/char_01/static_offset/fixed`. Read
//! literally that puts the ART UNDER THE CAMERA, in which case moving either moves both and nothing
//! can reframe — which cannot be right, since the game plainly cuts between shots. Print the real
//! parent chain so the question is settled by the data instead of by the binding strings.
//!
//! Usage: cargo run --release --example probe_hier -- <bundle.ab> [name-substring]
use std::collections::HashMap;
use serde_json::Value;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn pid(v: &Value) -> Option<i64> { v.get("m_PathID").and_then(Value::as_i64) }

fn main() {
    let path = std::env::args().nth(1).expect("bundle");
    let want = std::env::args().nth(2).unwrap_or_default().to_ascii_lowercase();
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    for entry in &bundle.files {
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        let mut name: HashMap<i64, String> = HashMap::new();
        let mut tf_of_go: HashMap<i64, i64> = HashMap::new();
        let mut go_of_tf: HashMap<i64, i64> = HashMap::new();
        let mut father: HashMap<i64, i64> = HashMap::new();
        let mut children: HashMap<i64, Vec<i64>> = HashMap::new();
        let mut cam_gos: Vec<i64> = Vec::new();
        for obj in &sf.objects {
            let Ok(v) = read_object(&sf, obj) else { continue };
            match obj.class_id {
                1 => { name.insert(obj.path_id, v["m_Name"].as_str().unwrap_or("").to_string()); }
                4 | 224 => {
                    let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
                    tf_of_go.insert(go, obj.path_id);
                    go_of_tf.insert(obj.path_id, go);
                    let f = v.get("m_Father").and_then(pid).unwrap_or(0);
                    father.insert(obj.path_id, f);
                    children.entry(f).or_default().push(obj.path_id);
                }
                20 => { if let Some(g) = v.get("m_GameObject").and_then(pid) { cam_gos.push(g); } }
                _ => {}
            }
        }
        for cg in &cam_gos {
            println!("== camera GO '{}'", name.get(cg).cloned().unwrap_or_default());
            let mut chain = Vec::new();
            let mut cur = tf_of_go.get(cg).copied().unwrap_or(0);
            while cur != 0 {
                chain.push(cur);
                cur = father.get(&cur).copied().unwrap_or(0);
            }
            for (i, t) in chain.iter().rev().enumerate() {
                let g = go_of_tf.get(t).copied().unwrap_or(0);
                println!("   {}{}", "  ".repeat(i), name.get(&g).cloned().unwrap_or_default());
            }
            let camtf = tf_of_go.get(cg).copied().unwrap_or(0);
            println!("   -- direct children of the camera GO, with subtree sizes:");
            // Count renderers (class 23 MeshRenderer / 137 SkinnedMesh) under each child, so a
            // rig that holds real ART is distinguishable from one that holds only transforms.
            let mut rend_go: std::collections::HashSet<i64> = std::collections::HashSet::new();
            for obj in &sf.objects {
                if obj.class_id == 23 || obj.class_id == 137 {
                    if let Ok(v) = read_object(&sf, obj) {
                        if let Some(g) = v.get("m_GameObject").and_then(pid) { rend_go.insert(g); }
                    }
                }
            }
            for c in children.get(&camtf).into_iter().flatten() {
                let g = go_of_tf.get(c).copied().unwrap_or(0);
                let n = name.get(&g).cloned().unwrap_or_default();
                if !(want.is_empty() || n.to_ascii_lowercase().contains(&want)) { continue; }
                // BFS the subtree
                let mut stack = vec![*c];
                let (mut nodes, mut rends) = (0usize, 0usize);
                let mut sample: Vec<String> = Vec::new();
                while let Some(t) = stack.pop() {
                    nodes += 1;
                    let gg = go_of_tf.get(&t).copied().unwrap_or(0);
                    if rend_go.contains(&gg) {
                        rends += 1;
                        if sample.len() < 8 { sample.push(name.get(&gg).cloned().unwrap_or_default()); }
                    }
                    for cc in children.get(&t).into_iter().flatten() { stack.push(*cc); }
                }
                println!("        {n}: {nodes} nodes, {rends} renderers");
                if !sample.is_empty() { println!("            e.g. {}", sample.join(", ")); }
            }
        }
    }
}
