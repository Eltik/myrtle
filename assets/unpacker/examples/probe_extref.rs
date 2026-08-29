//! THROWAWAY: classify every material TEXTURE slot in a bundle as INTERNAL or EXTERNAL, and for
//! the external ones name the CAB the reference points into.
//!
//! Motivation: `spine_objects` in `main.rs` is built from the entries of ONE bundle, and
//! `mat_texenv` looks a texture pid up in exactly that map, so a PPtr with `m_FileID != 0` can
//! never resolve however much is staged. Kal'tsit's layer 47 is the case that matters: same
//! GameObject and shader as layer 48, but its `_DissolveTex` points outside the bundle, so it
//! draws unmasked and costs -1.517 MADC when removed.
//!
//! A pid alone cannot answer this: `probe_matprops` prints `external/absent` for anything not in
//! the one file it opened, which conflates "lives in a sibling CAB" with "not shipped at all".
//! The discriminator is `m_FileID`, which indexes the SerializedFile's own `externals` table.
//!
//! Usage: cargo run --release --example probe_extref -- <bundle.ab> [slot-substr]
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn main() {
    let path = std::env::args().nth(1).expect("bundle.ab");
    let want = std::env::args().nth(2).unwrap_or_default();
    let Ok(data) = std::fs::read(&path) else { return };
    let Ok(bundle) = BundleFile::parse(data) else { return };
    let mut internal = 0usize;
    let mut external: HashMap<String, usize> = HashMap::new();
    let mut unbound = 0usize;
    let mut missing = 0usize;
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") { continue }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        let here: HashSet<i64> = sf.objects.iter().map(|o| o.path_id).collect();
        for obj in &sf.objects {
            if obj.class_id != 21 { continue }
            let Ok(mat) = read_object(&sf, obj) else { continue };
            let name = format!("{} [{}]", mat.get("m_Name").and_then(Value::as_str).unwrap_or("?"), obj.path_id);
            let Some(te) = mat.get("m_SavedProperties").and_then(|s| s.get("m_TexEnvs")).and_then(Value::as_object) else { continue };
            for (slot, v) in te {
                let Some(t) = v.get("m_Texture") else { continue };
                let fid = t.get("m_FileID").and_then(Value::as_i64).unwrap_or(0);
                let pid = t.get("m_PathID").and_then(Value::as_i64).unwrap_or(0);
                let show = want.is_empty() || slot.contains(&want) || name.contains(&want);
                if pid == 0 {
                    unbound += 1;
                    if show && !want.is_empty() { println!("  {name:<40} {slot:<18} UNBOUND"); }
                } else if fid == 0 {
                    if here.contains(&pid) {
                        internal += 1;
                        if show && !want.is_empty() { println!("  {name:<40} {slot:<18} INTERNAL  pid {pid}"); }
                    } else {
                        missing += 1;
                        if show { println!("  {name:<40} {slot:<18} MISSING-IN-FILE  pid {pid}"); }
                    }
                } else {
                    let cab = sf.externals.get((fid - 1) as usize).map_or("<out of range>", |e| e.path.as_str());
                    *external.entry(cab.to_string()).or_default() += 1;
                    if show { println!("  {name:<40} {slot:<18} EXTERNAL fid {fid} pid {pid} -> {cab}"); }
                }
            }
        }
    }
    println!("TOTALS internal={internal} external={} unbound={unbound} missing_in_file={missing}", external.values().sum::<usize>());
    let mut v: Vec<_> = external.into_iter().collect();
    v.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    for (cab, n) in v.iter().take(8) { println!("  EXT {n:>5}  {cab}"); }
}
