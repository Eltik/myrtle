//! THROWAWAY: why does one material's external shader PPtr fail to resolve?
//! Prints the material's file externals, then looks the pathID up in the shader map both by
//! the expected (CAB, pathID) key and by pathID alone across every CAB.
use serde_json::Value;
use std::path::PathBuf;
use unpacker::export::shader_map::{build_shader_map, resolve_shader};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let (skin, shaders, want) = (&a[0], &a[1], a[2].parse::<i64>().unwrap());
    let map = build_shader_map(&[PathBuf::from(shaders)]);
    println!("shader map entries: {}", map.len());
    let by_pid: Vec<(&String, &String)> = map.iter().filter(|((_, p), _)| *p == want).map(|((c, _), n)| (c, n)).collect();
    println!("entries with pathID {want}: {}", by_pid.len());
    for (c, n) in &by_pid { println!("   CAB '{c}'  ->  '{n}'"); }
    let Ok(d) = std::fs::read(skin) else { return };
    let Ok(b) = BundleFile::parse(d) else { return };
    for e in &b.files {
        let l = e.path.to_ascii_lowercase();
        if l.ends_with(".ress") || l.ends_with(".resource") { continue; }
        let Ok(sf) = SerializedFile::parse(e.data.clone()) else { continue };
        println!("\nskin file '{}' externals ({}):", e.path, sf.externals.len());
        for (i, ext) in sf.externals.iter().enumerate() {
            println!("   [{}] file_id {} -> cab_name '{}'", i, i + 1, ext.cab_name());
        }
        // find a material pointing at `want`
        for o in sf.objects.iter().filter(|o| o.class_id == 21) {
            let Ok(v) = read_object(&sf, o) else { continue };
            let Some(sh) = v.get("m_Shader") else { continue };
            let (fid, pid) = (
                sh.get("m_FileID").and_then(Value::as_i64).unwrap_or(0),
                sh.get("m_PathID").and_then(Value::as_i64).unwrap_or(0),
            );
            if pid != want { continue; }
            let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
            let r = resolve_shader(&sf.externals, fid, pid, &map);
            println!("\n  MAT '{name}'  m_Shader file_id={fid} path_id={pid}");
            println!("     externals[{}] = {:?}", fid - 1, sf.externals.get((fid - 1) as usize).map(|e| e.cab_name().to_string()));
            println!("     resolve_shader -> {r:?}");
            break;
        }
    }
}
