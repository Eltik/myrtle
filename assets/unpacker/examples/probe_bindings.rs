//! THROWAWAY diagnostic: WHAT does each entrance AnimationClip actually animate?
//!
//! Motivation: Executor's scope rim exports as a static quad, but the capture says its radius is
//! animated (width jumps 305 -> 432 in 0.2 s while the camera scale falls). Her clips are GENERIC
//! (`m_Legacy: false`, every named curve array empty), so the bindings live in
//! `m_ClipBindingConstant.genericBindings` with **CRC32-hashed** paths and attributes. Resolving
//! those hashes against the prefab hierarchy says exactly which objects are driven and by what —
//! and therefore whether the exporter is dropping a per-frame transform it could read.
//!
//! Usage: cargo run --release --example probe_bindings -- <bundle.ab> [path-filter]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

/// Standard CRC32 (IEEE / zlib polynomial) — what Unity hashes binding paths and attributes with.
fn crc32(s: &str) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for b in s.as_bytes() {
        crc ^= u32::from(*b);
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

fn go_path(all: &HashMap<i64, (i32, Value)>, go_pid: i64) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut cur = Some(go_pid);
    for _ in 0..64 {
        let Some(pid) = cur else { break };
        let Some((_, gv)) = all.get(&pid) else { break };
        parts.push(gv.get("m_Name").and_then(Value::as_str).unwrap_or("?").to_string());
        let tf = gv.get("m_Component").and_then(Value::as_array).and_then(|comps| {
            comps.iter().find_map(|c| {
                let p = c.get("component")?.get("m_PathID")?.as_i64()?;
                let (cid, _) = all.get(&p)?;
                if *cid == 4 { Some(p) } else { None }
            })
        });
        cur = tf
            .and_then(|t| all.get(&t))
            .and_then(|(_, tv)| tv.get("m_Father"))
            .and_then(|f| f.get("m_PathID"))
            .and_then(Value::as_i64)
            .filter(|p| *p != 0)
            .and_then(|p| all.get(&p))
            .and_then(|(_, pv)| pv.get("m_GameObject"))
            .and_then(|g| g.get("m_PathID"))
            .and_then(Value::as_i64);
    }
    parts.reverse();
    parts.join("/")
}

fn main() {
    let path = std::env::args().nth(1).expect("usage: probe_bindings <bundle.ab> [filter]");
    let filter = std::env::args().nth(2).unwrap_or_default().to_ascii_lowercase();
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");

    // Attribute hashes worth naming. Unity hashes the property string the same way.
    let attrs = [
        "m_LocalPosition.x", "m_LocalPosition.y", "m_LocalPosition.z",
        "m_LocalScale.x", "m_LocalScale.y", "m_LocalScale.z",
        "m_LocalRotation.x", "m_LocalRotation.y", "m_LocalRotation.z", "m_LocalRotation.w",
        "m_IsActive", "material._MainTex_ST.x", "material._Color.a", "m_Enabled",
    ];
    let attr_map: HashMap<u32, &str> = attrs.iter().map(|a| (crc32(a), *a)).collect();

    for entry in &bundle.files {
        let l = entry.path.to_ascii_lowercase();
        if l.ends_with(".ress") || l.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for o in &sf.objects {
            if let Ok(v) = read_object(&sf, o) {
                all.insert(o.path_id, (o.class_id, v));
            }
        }

        // Every GameObject's hierarchy path, hashed, plus the RELATIVE paths a clip would use
        // (clips bind paths relative to the Animator's root, so try every suffix too).
        let mut hash_to_path: HashMap<u32, String> = HashMap::new();
        for (pid, (cid, _)) in &all {
            if *cid != 1 {
                continue;
            }
            let full = go_path(&all, *pid);
            let segs: Vec<&str> = full.split('/').collect();
            for i in 0..segs.len() {
                let rel = segs[i..].join("/");
                hash_to_path.entry(crc32(&rel)).or_insert(rel);
            }
        }
        hash_to_path.insert(crc32(""), "<root>".to_string());

        let mut clips: Vec<&Value> = all.values().filter(|(cid, _)| *cid == 74).map(|(_, v)| v).collect();
        clips.sort_by_key(|v| v.get("m_Name").and_then(Value::as_str).unwrap_or("").to_string());
        for v in clips {
            let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
            let Some(binds) = v.get("m_ClipBindingConstant").and_then(|c| c.get("genericBindings")).and_then(Value::as_array) else { continue };
            let mut rows: Vec<String> = Vec::new();
            for b in binds {
                let p = b.get("path").and_then(Value::as_u64).unwrap_or(0) as u32;
                let a = b.get("attribute").and_then(Value::as_u64).unwrap_or(0) as u32;
                let ty = b.get("typeID").and_then(Value::as_u64).unwrap_or(0);
                let pname = hash_to_path.get(&p).cloned().unwrap_or_else(|| format!("<unresolved {p:#x}>"));
                let aname = attr_map.get(&a).map_or_else(|| format!("{a:#x}"), |s| (*s).to_string());
                if !filter.is_empty() && !pname.to_ascii_lowercase().contains(&filter) {
                    continue;
                }
                rows.push(format!("{pname}  ::  {aname}  (type {ty})"));
            }
            if rows.is_empty() {
                continue;
            }
            rows.sort();
            rows.dedup();
            println!("== [{name}]  {} binding(s){}", binds.len(), if filter.is_empty() { "" } else { " matching filter" });
            for r in rows.iter().take(50) {
                println!("     {r}");
            }
        }
    }
}
