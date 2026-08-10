//! THROWAWAY: Texture2D dimensions in the bundle, to compare against what we exported.
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};
fn main() {
    let path = std::env::args().nth(1).expect("usage: probe_texsize <bundle.ab> [substr]");
    let want = std::env::args().nth(2).unwrap_or_default().to_lowercase();
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") { continue; }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        let mut all: HashMap<i64,(i32,Value)> = HashMap::new();
        for o in &sf.objects { if let Ok(v)=read_object(&sf,o) { all.insert(o.path_id,(o.class_id,v)); } }
        for (pid,(cid,v)) in &all {
            if *cid != 28 { continue; }
            let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
            if !want.is_empty() && !name.to_lowercase().contains(&want) { continue; }
            let w = v.get("m_Width").and_then(Value::as_i64).unwrap_or(-1);
            let h = v.get("m_Height").and_then(Value::as_i64).unwrap_or(-1);
            let fmt = v.get("m_TextureFormat").and_then(Value::as_i64).unwrap_or(-1);
            let mips = v.get("m_MipCount").and_then(Value::as_i64).unwrap_or(-1);
            println!("  tex pid {pid} {name:<34} {w}x{h} fmt={fmt} mips={mips}");
        }
    }
}
