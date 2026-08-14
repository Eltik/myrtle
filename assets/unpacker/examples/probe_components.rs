//! THROWAWAY diagnostic: what COMPONENT TYPES does the entrance prefab attach that we may not read?
//!
//! Motivation: the entrance post-process volume (`pp` + `PostProcessVolume` + profile + an animated
//! weight) was an entire subsystem sitting in the bundle that nothing in our pipeline touched, and
//! it was worth 4.2 MADC on the one skin that opens it. That is unlikely to be the only one. Group
//! every `MonoBehaviour` on the entrance prefab by its script, so an unhandled subsystem shows up as
//! a script we have never mentioned.
//!
//! Usage: cargo run --release --example `probe_components` -- <bundle.ab>...
#![allow(clippy::or_fun_call, clippy::too_many_lines)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            continue;
        };
        let short = path.rsplit('/').next().unwrap_or(&path).to_string();
        for entry in &bundle.files {
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
                continue;
            };
            let mut names: HashMap<i64, String> = HashMap::new();
            for obj in &sf.objects {
                if obj.class_id == 1
                    && let Ok(v) = read_object(&sf, obj)
                {
                    names.insert(obj.path_id, v["m_Name"].as_str().unwrap_or("").to_string());
                }
            }
            // script pathID -> (count, sample GO name, sample field list)
            let mut by_script: HashMap<i64, (usize, String, String)> = HashMap::new();
            for obj in &sf.objects {
                if obj.class_id != 114 {
                    continue;
                }
                let Ok(v) = read_object(&sf, obj) else {
                    continue;
                };
                let sp = v.get("m_Script").and_then(pid).unwrap_or(0);
                let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
                let gname = names.get(&go).cloned().unwrap_or_default();
                let fields = v
                    .as_object()
                    .map(|m| {
                        let mut k: Vec<&str> = m
                            .keys()
                            .map(String::as_str)
                            .filter(|s| !s.starts_with("m_"))
                            .collect();
                        k.sort_unstable();
                        k.join(",")
                    })
                    .unwrap_or_default();
                let e = by_script
                    .entry(sp)
                    .or_insert((0, gname.clone(), fields.clone()));
                e.0 += 1;
                if e.1.is_empty() {
                    e.1 = gname;
                }
                if e.2.is_empty() {
                    e.2 = fields;
                }
            }
            if by_script.is_empty() {
                continue;
            }
            println!("== {short}");
            let mut rows: Vec<_> = by_script.into_iter().collect();
            rows.sort_by_key(|(_, (c, _, _))| std::cmp::Reverse(*c));
            // Full dump for a script of interest (2nd CLI arg = script pathID).
            if let Some(want) = std::env::args().nth(2).and_then(|x| x.parse::<i64>().ok()) {
                for obj in &sf.objects {
                    if obj.class_id != 114 {
                        continue;
                    }
                    let Ok(v) = read_object(&sf, obj) else {
                        continue;
                    };
                    if v.get("m_Script").and_then(pid) != Some(want) {
                        continue;
                    }
                    let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
                    println!(
                        "   --- on '{}':",
                        names.get(&go).cloned().unwrap_or_default()
                    );
                    if let Some(m) = v.as_object() {
                        for (k, vv) in m {
                            if k.starts_with("m_") {
                                continue;
                            }
                            let t = format!("{vv}");
                            println!(
                                "        {k} = {}",
                                if t.len() > 120 {
                                    format!("{}…", &t[..120])
                                } else {
                                    t
                                }
                            );
                        }
                    }
                }
                continue;
            }
            for (sp, (c, go, fields)) in rows.iter().take(14) {
                let f = if fields.len() > 110 {
                    format!("{}…", &fields[..110])
                } else {
                    fields.clone()
                };
                println!("   script {sp:>21}  x{c:<3} on '{go}'  fields: {f}");
            }
        }
    }
}
