//! THROWAWAY diagnostic: does anything in a dynchar bundle PLACE a sub-clip on a
//! timeline? Whislash-alter's `mask_09` transition plane is sequenced by a colour
//! curve whose times are CLIP-LOCAL to a 0.83 s sub-clip, and the open question is
//! whether the bundle says when that sub-clip runs.
//!
//! Prints: a class census, every `AnimationClip` (74) with its length, and then for
//! the clip whose name matches <needle>, every object in the bundle that REFERENCES
//! its `path_id`, with the JSON path of the reference and the sibling keys around
//! it, since a placement would have to sit next to the reference.
//!
//! Usage: cargo run --release --example `probe_clipplace` -- <bundle.ab> [needle]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

/// Recursively find every JSON location whose `m_PathID` equals `target`.
fn find_refs(prefix: &str, v: &Value, target: i64, out: &mut Vec<(String, String)>) {
    match v {
        Value::Object(o) => {
            if o.get("m_PathID").and_then(Value::as_i64) == Some(target) {
                out.push((prefix.to_string(), String::new()));
                return;
            }
            for (k, vv) in o {
                find_refs(&format!("{prefix}.{k}"), vv, target, out);
            }
        }
        Value::Array(a) => {
            for (i, vv) in a.iter().enumerate() {
                find_refs(&format!("{prefix}[{i}]"), vv, target, out);
            }
        }
        _ => {}
    }
}

/// Walk back down `path` to the object CONTAINING the reference, and describe its
/// scalar fields. A placement (a start time, an offset, an exit time) would be one.
fn container_fields(root: &Value, path: &str) -> String {
    let mut cur = root;
    let segs: Vec<&str> = path.split('.').filter(|s| !s.is_empty()).collect();
    // Drop the last segment: that is the reference field itself.
    for seg in segs.iter().take(segs.len().saturating_sub(1)) {
        let (name, idxs) = match seg.find('[') {
            Some(b) => (&seg[..b], &seg[b..]),
            None => (*seg, ""),
        };
        let Some(next) = cur.get(name) else {
            return "<unresolved>".into();
        };
        cur = next;
        for part in idxs.split('[').filter(|s| !s.is_empty()) {
            let Ok(i) = part.trim_end_matches(']').parse::<usize>() else {
                return "<unresolved>".into();
            };
            let Some(next) = cur.get(i) else {
                return "<unresolved>".into();
            };
            cur = next;
        }
    }
    match cur {
        Value::Object(o) => o
            .iter()
            .filter(|(_, v)| v.is_number() || v.is_boolean() || v.is_string())
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(" "),
        _ => "<not an object>".into(),
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle");
    let needle = std::env::args().nth(2).unwrap_or_else(|| "start".into());
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
        // Skip the heavy leaf classes; none of them can carry a placement.
        let skip: HashSet<i32> = [28, 43, 48, 49, 83, 128, 213].into_iter().collect();
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        let mut census: BTreeMap<i32, usize> = BTreeMap::new();
        for obj in &sf.objects {
            *census.entry(obj.class_id).or_default() += 1;
            if skip.contains(&obj.class_id) {
                continue;
            }
            if let Ok(v) = read_object(&sf, obj) {
                all.insert(obj.path_id, (obj.class_id, v));
            }
        }
        let name_of = |v: &Value| -> String {
            v.get("m_Name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        };
        println!("\n=== {} ===", entry.path);
        println!(
            "census: {}",
            census
                .iter()
                .map(|(c, n)| format!("{c}x{n}"))
                .collect::<Vec<_>>()
                .join(" ")
        );

        let mut clips: Vec<(i64, String)> = all
            .iter()
            .filter(|(_, (c, _))| *c == 74)
            .map(|(p, (_, v))| (*p, name_of(v)))
            .collect();
        clips.sort_by(|a, b| a.1.cmp(&b.1));
        println!("clips ({}):", clips.len());
        for (p, n) in &clips {
            let stop = all[p]
                .1
                .pointer("/m_MuscleClipInfo/m_StopTime")
                .and_then(Value::as_f64);
            println!("   pid={p:<22} {n:<44} stop={stop:?}");
        }

        // `isactive` counts EVERY `m_IsActive` binding in EVERY clip, admitted or not.
        // `active_windows` only scans the entrance/camera set, so its silence is not
        // proof that no clip anywhere toggles a GameObject on.
        if needle == "isactive" {
            const M_IS_ACTIVE_CRC: i64 = 2_086_281_974;
            for (pid, cname) in &clips {
                let bindings = all[pid]
                    .1
                    .pointer("/m_ClipBindingConstant/genericBindings")
                    .and_then(Value::as_array);
                let hits: Vec<i64> = bindings
                    .into_iter()
                    .flatten()
                    .filter(|b| b.get("attribute").and_then(Value::as_i64) == Some(M_IS_ACTIVE_CRC))
                    .filter_map(|b| b.get("path").and_then(Value::as_i64))
                    .collect();
                println!(
                    "   clip {cname:<44} m_IsActive bindings={} paths={hits:?}",
                    hits.len()
                );
            }
            continue;
        }

        // `json:<pid>` prints one object's raw JSON, capped, for the fields that
        // container_fields hides because they are nested structs.
        if let Some(rest) = needle.strip_prefix("json:")
            && let Ok(target) = rest.parse::<i64>()
        {
            if let Some((cid, v)) = all.get(&target) {
                let txt = serde_json::to_string_pretty(v).unwrap_or_default();
                println!("\npid={target} class={cid} ({} bytes)", txt.len());
                println!("{}", &txt[..txt.len().min(6000)]);
            } else {
                println!("pid={target} NOT FOUND");
            }
            continue;
        }

        // `dump:<pid>` prints one object's scalar fields and the GO chain above it,
        // which is how an Animator is traced back to the GameObject it drives.
        if let Some(rest) = needle.strip_prefix("dump:")
            && let Ok(target) = rest.parse::<i64>()
        {
            if let Some((cid, v)) = all.get(&target) {
                println!("\npid={target} class={cid} '{}'", name_of(v));
                println!("   fields: {}", container_fields(v, ".x"));
                let mut go = v
                    .get("m_GameObject")
                    .and_then(|g| g.get("m_PathID"))
                    .and_then(Value::as_i64);
                // Climb Transform parents to print the full prefab path.
                let mut chain = Vec::new();
                while let Some(g) = go.filter(|&g| g != 0) {
                    let Some((_, gv)) = all.get(&g) else { break };
                    chain.push(format!("{}#{g}", name_of(gv)));
                    let t = gv
                        .get("m_Component")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(|c| c.pointer("/component/m_PathID").and_then(Value::as_i64))
                        .find(|p| all.get(p).is_some_and(|(c, _)| *c == 4 || *c == 224));
                    go = t
                        .and_then(|t| all.get(&t))
                        .and_then(|(_, tv)| tv.pointer("/m_Father/m_PathID"))
                        .and_then(Value::as_i64)
                        .and_then(|f| all.get(&f).map(|(_, fv)| fv))
                        .and_then(|fv| fv.pointer("/m_GameObject/m_PathID"))
                        .and_then(Value::as_i64);
                }
                chain.reverse();
                println!("   GO chain: {}", chain.join(" > "));
            } else {
                println!("pid={target} NOT FOUND");
            }
            continue;
        }

        // A bare path_id as the needle searches for THAT object's references instead:
        // the clip's owner is an AnimatorController, and the controller's own placement
        // is the next link in the chain.
        let explicit: Vec<(i64, String)> = needle
            .parse::<i64>()
            .ok()
            .map(|p| vec![(p, all.get(&p).map_or_else(String::new, |(_, v)| name_of(v)))])
            .unwrap_or_default();
        let targets: Vec<(i64, String)> = if explicit.is_empty() {
            clips
                .iter()
                .filter(|(_, n)| n.contains(&needle))
                .cloned()
                .collect()
        } else {
            explicit
        };
        for (pid, cname) in &targets {
            println!("\n--- references to '{cname}' (pid={pid})");
            let mut n_refs = 0;
            for (opid, (ocid, ov)) in &all {
                let mut hits = Vec::new();
                find_refs("", ov, *pid, &mut hits);
                for (p, _) in &hits {
                    n_refs += 1;
                    println!(
                        "   in pid={opid} class={ocid} '{}' at {p}\n      container: {}",
                        name_of(ov),
                        container_fields(ov, p)
                    );
                }
            }
            if n_refs == 0 {
                println!("   NONE, nothing in this file references the clip");
            }
        }
    }
}
