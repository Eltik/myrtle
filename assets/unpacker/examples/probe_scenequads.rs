//! THROWAWAY diagnostic: every `MeshRenderer` under the ENTRANCE prefab root, vs what we export.
//!
//! Motivation: Civilight Eterna's t=2 region needs ~+50 luma of a pale veil that nothing in her
//! exported scene supplies — the candidates were excluded on magnitude. Before concluding "the
//! game draws something the data does not contain", check the obvious alternative: the exporter
//! dropped it. This lists the raw renderers so the two counts can be compared directly.
//!
//! Usage: cargo run --release --example `probe_scenequads` -- <bundle.ab> [root-substring]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle");
    let want = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "dyn_entrance".into())
        .to_ascii_lowercase();
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    for entry in &bundle.files {
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        let mut name: HashMap<i64, String> = HashMap::new();
        let mut tf_of_go: HashMap<i64, i64> = HashMap::new();
        let mut go_of_tf: HashMap<i64, i64> = HashMap::new();
        let mut father: HashMap<i64, i64> = HashMap::new();
        let mut rend: HashMap<i64, bool> = HashMap::new(); // go -> renderer enabled
        for obj in &sf.objects {
            let Ok(v) = read_object(&sf, obj) else {
                continue;
            };
            match obj.class_id {
                1 => {
                    name.insert(obj.path_id, v["m_Name"].as_str().unwrap_or("").to_string());
                }
                4 | 224 => {
                    let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
                    tf_of_go.insert(go, obj.path_id);
                    go_of_tf.insert(obj.path_id, go);
                    father.insert(obj.path_id, v.get("m_Father").and_then(pid).unwrap_or(0));
                }
                23 => {
                    if let Some(g) = v.get("m_GameObject").and_then(pid) {
                        let en = v.get("m_Enabled").and_then(Value::as_i64).unwrap_or(1) != 0;
                        rend.insert(g, en);
                    }
                }
                _ => {}
            }
        }
        // root of each GO
        let root_of = |go: i64| -> String {
            let mut cur = tf_of_go.get(&go).copied().unwrap_or(0);
            let mut last = cur;
            while cur != 0 {
                last = cur;
                cur = father.get(&cur).copied().unwrap_or(0);
            }
            name.get(&go_of_tf.get(&last).copied().unwrap_or(0))
                .cloned()
                .unwrap_or_default()
        };
        let mut total = 0usize;
        let mut enabled = 0usize;
        let mut rows: Vec<(String, bool)> = Vec::new();
        for (go, en) in &rend {
            let r = root_of(*go).to_ascii_lowercase();
            if !r.contains(&want) {
                continue;
            }
            total += 1;
            if *en {
                enabled += 1;
            }
            rows.push((name.get(go).cloned().unwrap_or_default(), *en));
        }
        if total == 0 {
            continue;
        }
        println!("root filter '{want}': {total} MeshRenderers, {enabled} enabled");
        rows.sort();
        for (n, en) in rows.iter().take(60) {
            println!(
                "   {}{}",
                n,
                if *en { "" } else { "   (renderer DISABLED)" }
            );
        }
        if rows.len() > 60 {
            println!("   … and {} more", rows.len() - 60);
        }
    }
}
