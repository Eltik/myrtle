//! THROWAWAY census: every animation-clip binding that targets a MATERIAL property on a
//! dynchar prefab, with the property resolved by name.
//!
//! Motivation: the IL2CPP setter census (register, ninth pass) found no C# on the dynchar path
//! that writes `_MainColor`, `_TintColor`, `_Strength` or `_Anchor*`, so the runtime behind the
//! three refuted shader-family terms is either an animation clip binding a material property
//! or nothing. The exporter reads only COLOUR-channel bindings (`customType` 22, top nibble
//! 4..7) on the `_Start` clips (`anim.rs`, `entrance_material_color_channels`); this prints
//! every material binding on every clip, colour channels and channel-less floats alike.
//!
//! A material binding encodes its target as `customType` 22 with `attribute` =
//! `crc32(propName)` for a float, or `(crc32(propName) & 0x0FFF_FFFF) | ((4 + channel) << 28)`
//! for a colour channel (r, g, b, a = 0..3). Names are resolved against every property name any
//! Material in the bundle saves (`m_SavedProperties`), plus the names the register's terms use,
//! so an unresolved hash is printed as a hash and counted, never guessed.
//!
//! A vector property's components use the same low-28 hash with top nibble 0..3 (x, y, z, w;
//! for `_MainTex_ST` scale x, scale y, offset x, offset y). Shader bundles among the arguments
//! (`[uc]shaders.ab`, `[uc]uishaders.ab`) contribute every DECLARED property name through
//! `build_shader_props`, so a property no material saves still resolves.
//!
//! Prints one TSV row per binding: skin, clip, path, typeID, kind (float | r | g | b | a |
//! x | y | z | w), property (or `?0x...`), varying (1 when the decoded curve moves, 0 when it
//! is constant, ? when it could not be decoded), then a summary per property to stderr.
//!
//! Usage: cargo run --release --example `probe_matbindings` -- [uc]shaders.ab <bundle.ab>...
#![allow(clippy::cast_possible_truncation, clippy::too_many_lines)]
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use unpacker::export::anim::decode_curve_any;
use unpacker::export::shader_map::build_shader_props;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

/// Standard CRC32 (IEEE), what Unity hashes binding paths and attributes with.
fn crc32(s: &str) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for b in s.as_bytes() {
        crc ^= u32::from(*b);
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

fn go_path(all: &HashMap<i64, (i32, Value)>, go_pid: i64) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut cur = Some(go_pid);
    for _ in 0..64 {
        let Some(pid) = cur else { break };
        let Some((_, gv)) = all.get(&pid) else { break };
        parts.push(
            gv.get("m_Name")
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_string(),
        );
        let tf = gv
            .get("m_Component")
            .and_then(Value::as_array)
            .and_then(|comps| {
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

/// Every property name a Material saves, whatever shape the typetree gives the arrays.
fn material_prop_names(mat: &Value, out: &mut HashSet<String>) {
    let Some(sp) = mat.get("m_SavedProperties") else {
        return;
    };
    for key in ["m_Floats", "m_Colors", "m_TexEnvs", "m_Ints"] {
        match sp.get(key) {
            Some(Value::Array(items)) => {
                for it in items {
                    if let Some(n) = it.get("first").and_then(Value::as_str) {
                        out.insert(n.to_string());
                    } else if let Some(arr) = it.as_array()
                        && let Some(n) = arr.first().and_then(Value::as_str)
                    {
                        out.insert(n.to_string());
                    }
                }
            }
            Some(Value::Object(map)) => {
                for k in map.keys() {
                    out.insert(k.clone());
                }
            }
            _ => {}
        }
    }
}

/// Curves one generic binding expands to (Unity's `FindBinding` accumulation): Transform
/// position, scale and euler are 3, quaternion rotation 4, everything else 1.
const fn binding_curve_count(type_id: i64, attribute: i64) -> usize {
    if type_id == 4 {
        match attribute {
            1 | 3 | 4 => 3,
            2 => 4,
            _ => 1,
        }
    } else {
        1
    }
}

fn main() {
    println!("skin\tclip\tpath\ttypeID\tkind\tproperty\tvarying");
    // (bindings, varying, skins, clips) per property.kind
    let mut summary: BTreeMap<String, (usize, usize, HashSet<String>, HashSet<String>)> =
        BTreeMap::new();
    let mut unresolved = 0usize;
    let mut total = 0usize;
    let args: Vec<std::path::PathBuf> = std::env::args().skip(1).map(Into::into).collect();
    let (declared, _) = build_shader_props(&args);
    let declared_names: HashSet<String> = declared.values().flatten().cloned().collect();
    eprintln!("declared shader property names: {}", declared_names.len());
    for path in std::env::args().skip(1) {
        if path.contains("shaders.ab") {
            continue;
        }
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            continue;
        };
        let skin = std::path::Path::new(&path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        for entry in &bundle.files {
            let l = entry.path.to_ascii_lowercase();
            if l.ends_with(".ress") || l.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
                continue;
            };
            let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
            for o in &sf.objects {
                if let Ok(v) = read_object(&sf, o) {
                    all.insert(o.path_id, (o.class_id, v));
                }
            }
            // Property names: every Material's saved properties plus the register's terms.
            let mut names: HashSet<String> = [
                "_MainColor",
                "_TintColor",
                "_Strength",
                "_Anchor",
                "_Anchor0",
                "_Anchor1",
                "_Anchor2",
                "_Anchor3",
                "_AnchorX",
                "_AnchorY",
                "_Amount",
                "_Color",
                "_MainTex",
                "_MainTex_ST",
                "_DissolveTex",
                "_DisturbTex",
                "_Alpha",
                "_Intensity",
            ]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
            for (cid, v) in all.values() {
                if *cid == 21 {
                    material_prop_names(v, &mut names);
                }
            }
            names.extend(declared_names.iter().cloned());
            let float_map: HashMap<u32, String> =
                names.iter().map(|n| (crc32(n), n.clone())).collect();
            let color_map: HashMap<u32, String> = names
                .iter()
                .map(|n| (crc32(n) & 0x0FFF_FFFF, n.clone()))
                .collect();
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
            let mut clips: Vec<&Value> = all
                .values()
                .filter(|(cid, _)| *cid == 74)
                .map(|(_, v)| v)
                .collect();
            clips.sort_by_key(|v| {
                v.get("m_Name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string()
            });
            for v in clips {
                let clip = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
                let Some(binds) = v
                    .get("m_ClipBindingConstant")
                    .and_then(|c| c.get("genericBindings"))
                    .and_then(Value::as_array)
                else {
                    continue;
                };
                let mut gidx = 0usize;
                for b in binds {
                    let custom = b.get("customType").and_then(Value::as_i64).unwrap_or(0);
                    let ty = b
                        .get("typeID")
                        .or_else(|| b.get("classID"))
                        .and_then(Value::as_i64)
                        .unwrap_or(0);
                    let a64 = b.get("attribute").and_then(Value::as_i64).unwrap_or(0);
                    let my_idx = gidx;
                    gidx += binding_curve_count(ty, a64);
                    if custom != 22 {
                        continue;
                    }
                    total += 1;
                    let p = b.get("path").and_then(Value::as_u64).unwrap_or(0) as u32;
                    let a = a64 as u32;
                    let nibble = a >> 28;
                    let (kind, prop) = if (4..=7).contains(&nibble) {
                        (
                            ["r", "g", "b", "a"][(nibble - 4) as usize].to_string(),
                            color_map.get(&(a & 0x0FFF_FFFF)).cloned(),
                        )
                    } else if nibble <= 3 && color_map.contains_key(&(a & 0x0FFF_FFFF)) {
                        (
                            ["x", "y", "z", "w"][nibble as usize].to_string(),
                            color_map.get(&(a & 0x0FFF_FFFF)).cloned(),
                        )
                    } else {
                        ("float".to_string(), float_map.get(&a).cloned())
                    };
                    let varying = match decode_curve_any(v, my_idx) {
                        Some(c) if c.len() > 1 => {
                            let lo = c.iter().map(|s| s.1).fold(f32::INFINITY, f32::min);
                            let hi = c.iter().map(|s| s.1).fold(f32::NEG_INFINITY, f32::max);
                            if hi - lo > 1e-6 { "1" } else { "0" }
                        }
                        Some(_) => "0",
                        None => "?",
                    };
                    let prop = prop.unwrap_or_else(|| {
                        unresolved += 1;
                        format!("?{a:#010x}")
                    });
                    let pname = hash_to_path
                        .get(&p)
                        .cloned()
                        .unwrap_or_else(|| format!("<unresolved {p:#x}>"));
                    println!("{skin}\t{clip}\t{pname}\t{ty}\t{kind}\t{prop}\t{varying}");
                    let e = summary.entry(format!("{prop}.{kind}")).or_insert((
                        0,
                        0,
                        HashSet::new(),
                        HashSet::new(),
                    ));
                    e.0 += 1;
                    e.1 += usize::from(varying == "1");
                    e.2.insert(skin.clone());
                    e.3.insert(clip.to_string());
                }
            }
        }
    }
    eprintln!("material bindings: {total}, unresolved property hashes: {unresolved}");
    for (k, (n, var, skins, clips)) in &summary {
        eprintln!(
            "  {k:36} bindings {n:5}  varying {var:5}  skins {:3}  clips {:4}",
            skins.len(),
            clips.len()
        );
    }
}
