//! THROWAWAY diagnostic: what draws the circular SCOPE MASK on Executor's entrance?
//!
//! Motivation: the clean 2340x1080 capture of `char_1032_excu2_sale#12` shows the ENTIRE entrance
//! (t≈0→5.5 s) viewed through an expanding circular scope with a black surround; our render has no
//! mask at all and scores 81.6 MADC against the references' 10-18. The effect is in NONE of the
//! exported assets — not the 33 scene textures, not the 461 spine atlas regions, not the 12
//! particle textures — so it is drawn by a component the exporter's quad/spine/particle model does
//! not cover. This dumps every component class present and anything mask-shaped.
//!
//! Usage: cargo run --release --example probe_mask -- <bundle.ab>
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

/// Unity class ids worth calling out by name when they appear.
fn class_name(id: i32) -> &'static str {
    match id {
        1 => "GameObject",
        4 => "Transform",
        20 => "Camera",
        23 => "MeshRenderer",
        33 => "MeshFilter",
        43 => "Mesh",
        materials if materials == 21 => "Material",
        28 => "Texture2D",
        114 => "MonoBehaviour",
        115 => "MonoScript",
        213 => "Sprite",
        222 => "CanvasRenderer",
        223 => "Canvas",
        224 => "RectTransform",
        331 => "SpriteMask",
        _ => "",
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("usage: probe_mask <bundle.ab>");
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
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
        if all.is_empty() {
            continue;
        }
        // 1. census of component classes
        let mut counts: BTreeMap<i32, usize> = BTreeMap::new();
        for (cid, _) in all.values() {
            *counts.entry(*cid).or_default() += 1;
        }
        println!("== {}", entry.path);
        let mut line = String::new();
        for (cid, n) in &counts {
            let nm = class_name(*cid);
            line.push_str(&format!("{cid}{}x{n}  ", if nm.is_empty() { String::new() } else { format!("({nm})") }));
        }
        println!("   classes: {line}");

        // 2. MonoBehaviour script names — this is where a UI Mask / RectMask2D would hide
        let mut scripts: BTreeMap<String, usize> = BTreeMap::new();
        for (_, (cid, v)) in &all {
            if *cid != 114 {
                continue;
            }
            let name = v
                .get("m_Script")
                .and_then(|s| s.get("m_PathID"))
                .and_then(Value::as_i64)
                .and_then(|p| all.get(&p))
                .and_then(|(_, mv)| mv.get("m_ClassName"))
                .and_then(Value::as_str)
                .unwrap_or("<unresolved>")
                .to_string();
            *scripts.entry(name).or_default() += 1;
        }
        println!("   MonoBehaviour scripts:");
        for (n, c) in &scripts {
            println!("      {c:>3}x  {n}");
        }

        // 2b. the single CanvasRenderer / RectTransforms, and every material's SHADER name
        for (pid, (cid, v)) in &all {
            if *cid != 222 && *cid != 224 && *cid != 223 {
                continue;
            }
            let go = v.get("m_GameObject").and_then(|g| g.get("m_PathID")).and_then(Value::as_i64)
                .and_then(|g| all.get(&g)).and_then(|(_, gv)| gv.get("m_Name")).and_then(Value::as_str).unwrap_or("?");
            println!("   UI pathID {pid} class {cid}({}) GO {go}", class_name(*cid));
        }
        // external shader references, by (fileID,pathID) — resolve against [uc]shaders.ab
        let mut ext: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (_, (cid, v)) in &all {
            if *cid != 21 { continue; }
            let sh = v.get("m_Shader");
            let fid = sh.and_then(|s| s.get("m_FileID")).and_then(Value::as_i64).unwrap_or(-1);
            let pid = sh.and_then(|s| s.get("m_PathID")).and_then(Value::as_i64).unwrap_or(0);
            let nm = v.get("m_Name").and_then(Value::as_str).unwrap_or("?").to_string();
            ext.entry(format!("fileID {fid} pathID {pid}")).or_default().push(nm);
        }
        println!("   material -> external shader ref ({} distinct):", ext.len());
        for (k, ns) in &ext {
            println!("      {k}  x{}  e.g. {}", ns.len(), ns.iter().take(3).cloned().collect::<Vec<_>>().join(", "));
        }
        let mut shaders: BTreeMap<String, usize> = BTreeMap::new();
        for (_, (cid, v)) in &all {
            if *cid != 21 { continue; }
            let s = v.get("m_Shader").and_then(|s| s.get("m_PathID")).and_then(Value::as_i64)
                .and_then(|p| all.get(&p)).and_then(|(_, sv)| sv.get("m_ParsedForm").and_then(|f| f.get("m_Name")).or_else(|| sv.get("m_Name")))
                .and_then(Value::as_str).unwrap_or("<external>").to_string();
            let nm = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
            *shaders.entry(format!("{s}  [e.g. {nm}]")).or_default() += 1;
        }
        println!("   materials by shader:");
        for (s, c) in &shaders { println!("      {c:>3}x  {s}"); }

        // 3. anything whose FIELDS mention masking / stencil / radius, whatever the class
        for (pid, (cid, v)) in &all {
            let Some(obj) = v.as_object() else { continue };
            let hits: Vec<&String> = obj
                .keys()
                .filter(|k| {
                    let l = k.to_ascii_lowercase();
                    l.contains("mask") || l.contains("stencil") || l.contains("radius") || l.contains("circle")
                })
                .collect();
            if hits.is_empty() {
                continue;
            }
            let go = v
                .get("m_GameObject")
                .and_then(|g| g.get("m_PathID"))
                .and_then(Value::as_i64)
                .and_then(|g| all.get(&g))
                .and_then(|(_, gv)| gv.get("m_Name"))
                .and_then(Value::as_str)
                .unwrap_or("?");
            println!("   MASK-ISH pathID {pid} class {cid}{} GO {go}", if class_name(*cid).is_empty() { String::new() } else { format!("({})", class_name(*cid)) });
            for h in hits {
                println!("        {h} = {}", serde_json::to_string(&obj[h]).unwrap_or_default().chars().take(160).collect::<String>());
            }
        }
    }
}
