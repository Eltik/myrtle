//! THROWAWAY: for every `MeshFilter` (class 33), can its `m_Mesh` be resolved IN-BUNDLE?
//! An unresolved reference makes the scene exporter substitute `unit_quad()` — a 4-vertex
//! RECTANGLE — for whatever the real (possibly ragged) geometry was.
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};
fn main() {
    for path in std::env::args().skip(1) {
        let Ok(d) = std::fs::read(&path) else {
            continue;
        };
        let Ok(b) = BundleFile::parse(d) else {
            continue;
        };
        for e in &b.files {
            let l = e.path.to_ascii_lowercase();
            if l.ends_with(".ress") || l.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(e.data.clone()) else {
                continue;
            };
            let local: HashSet<i64> = sf
                .objects
                .iter()
                .filter(|o| o.class_id == 43)
                .map(|o| o.path_id)
                .collect();
            let mut ok = 0;
            let mut ext = 0;
            let mut null = 0;
            let mut missing = 0;
            let mut verts: HashMap<i64, usize> = HashMap::new();
            for o in sf.objects.iter().filter(|o| o.class_id == 43) {
                if let Ok(v) = read_object(&sf, o)
                    && let Some(m) = unpacker::export::mesh::parse_mesh(&v, &HashMap::new())
                {
                    verts.insert(o.path_id, m.positions.len());
                }
            }
            for o in sf.objects.iter().filter(|o| o.class_id == 33) {
                let Ok(v) = read_object(&sf, o) else { continue };
                let Some(mp) = v.get("m_Mesh") else {
                    null += 1;
                    continue;
                };
                let fid = mp.get("m_FileID").and_then(Value::as_i64).unwrap_or(0);
                let pid = mp.get("m_PathID").and_then(Value::as_i64).unwrap_or(0);
                if pid == 0 {
                    null += 1;
                } else if fid != 0 {
                    ext += 1;
                } else if local.contains(&pid) {
                    ok += 1;
                } else {
                    missing += 1;
                }
            }
            let total = ok + ext + null + missing;
            println!(
                "{:<34} MeshFilters {total:>3} | resolved {ok:>3}  EXTERNAL {ext:>3}  null {null:>3}  missing {missing:>3}  -> unit_quad for {}",
                path.rsplit('/').next().unwrap_or("?"),
                ext + null + missing
            );
            let mut vs: Vec<usize> = verts.values().copied().collect();
            vs.sort_unstable();
            println!("      in-bundle mesh vertex counts: {vs:?}");
        }
    }
}
