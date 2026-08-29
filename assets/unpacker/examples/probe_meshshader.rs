//! THROWAWAY: for every MeshRenderer in a bundle, print its GameObject, its material's SHADER
//! (resolved through the shared shader bundle) and its `_MainTex` size.
//!
//! Motivation: a scene LAYER's shader decides whether the vertex colour it carries is even read.
//! kalts's layer 48 is a plain quad with no ram block and a 0.5625 mean vertex alpha, and whether
//! that multiply is faithful depends entirely on which family it is on. `probe_matprops` cannot
//! answer it: `_shaderName` is injected by the exporter and absent from the raw material.
//!
//! Usage: cargo run --release --example probe_meshshader -- <bundle.ab> <shaders.ab>
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use unpacker::export::shader_map::{build_shader_map, resolve_shader};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle.ab");
    let shaders = std::env::args().nth(2).expect("shaders.ab");
    let map = build_shader_map(&[PathBuf::from(shaders)]);
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
        for obj in &sf.objects {
            if let Ok(v) = read_object(&sf, obj) {
                all.insert(obj.path_id, (obj.class_id, v));
            }
        }
        let names: HashMap<i64, String> = all
            .iter()
            .filter(|(_, (c, _))| *c == 1)
            .map(|(k, (_, v))| (*k, v["m_Name"].as_str().unwrap_or("").to_string()))
            .collect();
        for (_, (cid, v)) in &all {
            if *cid != 23 {
                continue;
            } // MeshRenderer
            let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
            let gname = names.get(&go).cloned().unwrap_or_default();
            let mat_pid = v
                .get("m_Materials")
                .and_then(Value::as_array)
                .and_then(|a| a.first())
                .and_then(pid)
                .unwrap_or(0);
            let Some((_, mat)) = all.get(&mat_pid) else {
                println!("  {gname:<28} material UNRESOLVED");
                continue;
            };
            let shader = mat
                .get("m_Shader")
                .and_then(|s| {
                    let fid = s.get("m_FileID").and_then(Value::as_i64)?;
                    resolve_shader(&sf.externals, fid, pid(s)?, &map)
                })
                .unwrap_or("?");
            let tex = mat
                .get("m_SavedProperties")
                .and_then(|sp| sp.get("m_TexEnvs"))
                .and_then(|t| t.get("_MainTex"))
                .and_then(|t| t.get("m_Texture"))
                .and_then(pid)
                .and_then(|tp| all.get(&tp))
                .map_or_else(
                    || "none".to_string(),
                    |(_, t)| {
                        format!(
                            "{}x{}",
                            t.get("m_Width").and_then(Value::as_i64).unwrap_or(0),
                            t.get("m_Height").and_then(Value::as_i64).unwrap_or(0)
                        )
                    },
                );
            println!("  {gname:<28} {shader:<52} mainTex {tex}");
        }
    }
}
