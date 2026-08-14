//! THROWAWAY diagnostic: enumerate EVERY renderer (class 23 `MeshRenderer` / 137 `SkinnedMesh`)
//! in a dynchar bundle with its GO name, static active flag, sorting order, material shader,
//! material colour properties, and whether any `AnimationClip` binds its `m_IsActive` or a
//! material colour channel.
//!
//! Motivation: Skadi2 fades to full black at tt11/tt14.8 and the `BG_black_01..04` planes were
//! dismissed as "no colour channels" — but a plane can produce a TIMED blackout purely by
//! `m_IsActive` toggling, or by an untextured solid-colour material the mesh exporter drops.
//!
//! Usage: cargo run --release --example `probe_black` -- <bundle.ab> [name-filter]
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_possible_truncation,
    clippy::for_kv_map,
    clippy::format_push_string,
    clippy::too_many_lines
)]

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in bytes {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle path");
    let filter = std::env::args().nth(2).unwrap_or_default().to_lowercase();
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
        let skip: HashSet<i32> = [28, 43, 48, 49, 83, 128, 213].into_iter().collect();
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for obj in &sf.objects {
            if skip.contains(&obj.class_id) {
                continue;
            }
            if let Ok(v) = read_object(&sf, obj) {
                all.insert(obj.path_id, (obj.class_id, v));
            }
        }

        // Hierarchy: GO name/active, transform parentage for full paths.
        let mut go_name: HashMap<i64, String> = HashMap::new();
        let mut go_active: HashMap<i64, i64> = HashMap::new();
        let mut tf_go: HashMap<i64, i64> = HashMap::new();
        let mut go_tf: HashMap<i64, i64> = HashMap::new();
        let mut tf_father: HashMap<i64, i64> = HashMap::new();
        for (p, (cid, v)) in &all {
            match cid {
                1 => {
                    go_name.insert(
                        *p,
                        v.get("m_Name")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                    );
                    go_active.insert(*p, v.get("m_IsActive").and_then(Value::as_i64).unwrap_or(1));
                }
                4 | 224 => {
                    if let Some(g) = v.get("m_GameObject").and_then(pid) {
                        tf_go.insert(*p, g);
                        go_tf.insert(g, *p);
                    }
                    if let Some(f) = v.get("m_Father").and_then(pid)
                        && f != 0
                    {
                        tf_father.insert(*p, f);
                    }
                }
                _ => {}
            }
        }
        let full_path = |go: i64| -> String {
            let mut parts = Vec::new();
            let mut cur = go_tf.get(&go).copied();
            while let Some(tf) = cur {
                if let Some(g) = tf_go.get(&tf) {
                    parts.push(go_name.get(g).cloned().unwrap_or_default());
                }
                cur = tf_father.get(&tf).copied();
            }
            parts.reverse();
            parts.join("/")
        };

        // Which GO subpath-hashes are bound by clips, and for what.
        // Unity binds by crc32 of the animator-relative path; collect every bound
        // (pathHash, attribute) so we can report what drives a given GO.
        let mut bound: HashMap<u32, Vec<String>> = HashMap::new();
        for (cid, v) in all.values() {
            if *cid != 74 {
                continue;
            }
            let clip_name = v
                .get("m_Name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            for key in [
                "m_PositionCurves",
                "m_ScaleCurves",
                "m_RotationCurves",
                "m_EulerCurves",
                "m_FloatCurves",
                "m_PPtrCurves",
            ] {
                let Some(arr) = v.get(key).and_then(Value::as_array) else {
                    continue;
                };
                for c in arr {
                    let p = c.get("path").and_then(Value::as_str).unwrap_or("");
                    let h = c
                        .get("path")
                        .and_then(Value::as_u64)
                        .map_or_else(|| crc32(p.as_bytes()), |n| n as u32);
                    let attr = c
                        .get("attribute")
                        .and_then(Value::as_str)
                        .map_or_else(|| key.to_string(), str::to_string);
                    bound
                        .entry(h)
                        .or_default()
                        .push(format!("{clip_name}:{attr}"));
                }
            }
            // Generic (streamed/dense) bindings live in m_ClipBindingConstant.
            if let Some(gb) = v
                .get("m_ClipBindingConstant")
                .and_then(|b| b.get("genericBindings"))
                .and_then(Value::as_array)
            {
                for b in gb {
                    let h = b.get("path").and_then(Value::as_u64).unwrap_or(0) as u32;
                    let attr = b.get("attribute").and_then(Value::as_u64).unwrap_or(0);
                    let tid = b
                        .get("typeID")
                        .or_else(|| b.get("classID"))
                        .and_then(Value::as_i64)
                        .unwrap_or(-1);
                    // attribute 2086281974 == crc32("m_IsActive") for typeID 1 (GameObject).
                    let label = if tid == 1 {
                        "m_IsActive".to_string()
                    } else {
                        format!("tid{tid}/attr{attr}")
                    };
                    bound
                        .entry(h)
                        .or_default()
                        .push(format!("{clip_name}:{label}"));
                }
            }
        }

        println!("\n########## {} ##########", entry.path);
        let mut rows: Vec<String> = Vec::new();
        for (_p, (cid, v)) in &all {
            if *cid != 23 && *cid != 137 {
                continue;
            }
            let Some(go) = v.get("m_GameObject").and_then(pid) else {
                continue;
            };
            let name = go_name.get(&go).cloned().unwrap_or_default();
            let fp = full_path(go);
            if !filter.is_empty()
                && !name.to_lowercase().contains(&filter)
                && !fp.to_lowercase().contains(&filter)
            {
                continue;
            }
            let active = go_active.get(&go).copied().unwrap_or(1);
            let sort = v.get("m_SortingOrder").and_then(Value::as_i64).unwrap_or(0);
            let enabled = v.get("m_Enabled").and_then(Value::as_i64).unwrap_or(1);

            // Material → shader + colour props + textures.
            let mut mat_desc = String::new();
            if let Some(mats) = v.get("m_Materials").and_then(Value::as_array) {
                for m in mats {
                    let Some(mp) = pid(m) else { continue };
                    let Some((_, mv)) = all.get(&mp) else {
                        mat_desc.push_str(&format!(" [EXTERNAL mat pathID={mp}]"));
                        continue;
                    };
                    let mname = mv.get("m_Name").and_then(Value::as_str).unwrap_or("");
                    let shader = mv
                        .get("m_Shader")
                        .and_then(pid)
                        .and_then(|s| all.get(&s))
                        .and_then(|(_, sv)| sv.get("m_Name").and_then(Value::as_str))
                        .unwrap_or("<ext>");
                    let saved = mv.get("m_SavedProperties");
                    let cols: Vec<String> = saved
                        .and_then(|s| s.get("m_Colors"))
                        .and_then(Value::as_array)
                        .map(|a| {
                            a.iter()
                                .filter_map(|kv| {
                                    let k = kv.get("first")?.as_str()?;
                                    let c = kv.get("second")?;
                                    Some(format!(
                                        "{k}=({:.2},{:.2},{:.2},{:.2})",
                                        c.get("r")?.as_f64()?,
                                        c.get("g")?.as_f64()?,
                                        c.get("b")?.as_f64()?,
                                        c.get("a")?.as_f64()?
                                    ))
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    let texs: Vec<String> = saved
                        .and_then(|s| s.get("m_TexEnvs"))
                        .and_then(Value::as_array)
                        .map(|a| {
                            a.iter()
                                .filter_map(|kv| {
                                    let k = kv.get("first")?.as_str()?;
                                    let t = kv.get("second")?.get("m_Texture")?;
                                    let tp = pid(t).unwrap_or(0);
                                    let res = if tp == 0 {
                                        "NULL".to_string()
                                    } else if all.contains_key(&tp) {
                                        "in-bundle".to_string()
                                    } else {
                                        "EXTERNAL".to_string()
                                    };
                                    Some(format!("{k}={res}"))
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    mat_desc.push_str(&format!(
                        "\n      mat='{mname}' shader='{shader}' colors=[{}] texs=[{}]",
                        cols.join(", "),
                        texs.join(", ")
                    ));
                }
            }

            // Is this GO (or an ancestor) animated?
            let mut drivers: Vec<String> = Vec::new();
            let mut probe = Some(go);
            let mut depth = 0;
            while let Some(g) = probe {
                if depth > 8 {
                    break;
                }
                // Try the animator-relative subpath suffixes.
                let fpg = full_path(g);
                for start in 0..fpg.split('/').count() {
                    let sub = fpg.split('/').skip(start).collect::<Vec<_>>().join("/");
                    if let Some(b) = bound.get(&crc32(sub.as_bytes())) {
                        for x in b {
                            drivers.push(format!("{sub}→{x}"));
                        }
                    }
                }
                probe = go_tf
                    .get(&g)
                    .and_then(|t| tf_father.get(t))
                    .and_then(|f| tf_go.get(f))
                    .copied();
                depth += 1;
            }
            drivers.sort();
            drivers.dedup();

            rows.push(format!(
                "  GO='{name}' active={active} enabled={enabled} sort={sort}\n    path={fp}{mat_desc}\n    drivers={}",
                if drivers.is_empty() {
                    "NONE".to_string()
                } else {
                    drivers.join(" | ")
                }
            ));
        }
        rows.sort();
        println!("renderers matched: {}", rows.len());
        for r in rows {
            println!("{r}");
        }
    }
}
