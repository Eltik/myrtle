//! Corpus scan: which spelling do dynchar materials actually use for the Ram DISTURB map?
//!
//! The exporter reads `_DisturbTex` / `_DisturbUSpeed` / `_DisturbVSpeed` (with a `b`).
//! Muelsyse's water-line materials spell the texture slot `_DisturTex` (no `b`) and also
//! carry a SECOND slot `_DisturTex_02`. If a whole shader family uses the short spelling,
//! every disturb lookup for that family is silently read as absent — the classic
//! "not in the data is usually a bug" shape.
//!
//! Prints, per property name, how many materials carry it, so the two populations can be
//! compared instead of guessed at. Nothing is hardcoded per skin: it walks every material
//! in every bundle handed to it.
//!
//! Usage: cargo run --release --example scan_disturb -- <bundle.ab> [<bundle.ab>...]

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn main() {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    // property name -> number of MATERIALS carrying it
    let mut tex_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut float_counts: BTreeMap<String, usize> = BTreeMap::new();
    // bundles where the short spelling appears without the long one
    let mut short_only_bundles: BTreeSet<String> = BTreeSet::new();
    let mut shaders_with_short: BTreeSet<String> = BTreeSet::new();
    let mut n_mats = 0usize;
    let mut per_bundle: BTreeMap<String, usize> = BTreeMap::new();
    let mut effective: BTreeMap<String, usize> = BTreeMap::new();

    for path in &paths {
        let short_name = path.rsplit('/').next().unwrap_or(path).to_string();
        let Ok(data) = std::fs::read(path) else {
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            continue;
        };
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
            for (cid, v) in all.values() {
                if *cid != 21 {
                    continue; // Material
                }
                n_mats += 1;
                let saved = v.get("m_SavedProperties");
                let mut names_here: BTreeSet<String> = BTreeSet::new();
                // `m_TexEnvs`/`m_Floats` deserialise as a MAP (name -> value) here, not as an
                // array of [name, value] pairs. Handle both so the scan can't silently read
                // zero again.
                let mut collect = |slot: &str, counts: &mut BTreeMap<String, usize>| {
                    let Some(node) = saved.and_then(|s| s.get(slot)) else {
                        return;
                    };
                    let keys: Vec<String> = match node {
                        Value::Object(m) => m.keys().cloned().collect(),
                        Value::Array(a) => a
                            .iter()
                            .filter_map(|e| {
                                e.get(0)
                                    .and_then(Value::as_str)
                                    .or_else(|| e.get("first").and_then(Value::as_str))
                                    .map(str::to_string)
                            })
                            .collect(),
                        _ => Vec::new(),
                    };
                    for k in keys {
                        if k.contains("Distur") {
                            *counts.entry(k.clone()).or_default() += 1;
                            names_here.insert(k);
                        }
                    }
                };
                collect("m_TexEnvs", &mut tex_counts);
                collect("m_Floats", &mut float_counts);
                // Does the SHORT-spelling family actually do anything on this material?
                // `_ToggleUseDissolve` / `_ToggleUseDisturb2` gate the two effects in the
                // decompiled `Disturb Anchor` GLSL, so a material with both at 0 draws a plain
                // quad and costs us nothing by being unread.
                let tog = |k: &str| -> f64 {
                    saved
                        .and_then(|s| s.get("m_Floats"))
                        .and_then(|f| f.get(k))
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0)
                };
                let active = tog("_ToggleUseDissolve") != 0.0 || tog("_ToggleUseDisturb2") != 0.0;
                let has_short = names_here.iter().any(|n| n.starts_with("_DisturTex"));
                if has_short && active {
                    *tex_counts.entry("ZZ_SHORT_AND_ACTIVE".to_string()).or_default() += 1;
                    *per_bundle.entry(short_name.clone()).or_default() += 1;
                    // The toggles alone do NOT mean the effect does anything: the decompiled
                    // GLSL makes dissolve a no-op at `_Amount == 0` (roundEven(0.5)==0 makes the
                    // expression >= 1 and it clamps to 1), and the warp vanishes at zero
                    // intensity. Report the magnitudes so the prize can be bounded before any
                    // renderer work.
                    let amount = tog("_Amount");
                    let iu = tog("_IntensityU");
                    let iv = tog("_IntensityV");
                    let iu2 = tog("_IntensityU_02");
                    let iv2 = tog("_IntensityV_02");
                    let warps = iu != 0.0 || iv != 0.0 || iu2 != 0.0 || iv2 != 0.0;
                    let dissolves = tog("_ToggleUseDissolve") != 0.0 && amount != 0.0;
                    if warps {
                        *tex_counts.entry("ZZ_WARP_NONZERO".to_string()).or_default() += 1;
                    }
                    if dissolves {
                        *tex_counts.entry("ZZ_DISSOLVE_NONZERO".to_string()).or_default() += 1;
                    }
                    if warps || dissolves {
                        *effective.entry(short_name.clone()).or_default() += 1;
                    }
                    println!(
                        "   [mat] {short_name} amount={amount} Iu={iu} Iv={iv} Iu2={iu2} Iv2={iv2}"
                    );
                }
                let has_long = names_here.iter().any(|n| n.starts_with("_DisturbTex"));
                if has_short && !has_long {
                    short_only_bundles.insert(short_name.clone());
                    if let Some(sh) = v
                        .get("m_Shader")
                        .and_then(|s| s.get("m_PathID"))
                        .and_then(Value::as_i64)
                    {
                        shaders_with_short.insert(format!("shaderPID={sh}"));
                    }
                }
            }
        }
    }

    println!("materials scanned: {n_mats}");
    println!("\n-- TEXTURE slots containing 'Distur' --");
    for (k, n) in &tex_counts {
        println!("  {n:6}  {k}");
    }
    println!("\n-- FLOATS containing 'Distur' --");
    for (k, n) in &float_counts {
        println!("  {n:6}  {k}");
    }
    println!(
        "\nbundles with a SHORT-spelling material and no long spelling: {}",
        short_only_bundles.len()
    );
    for b in short_only_bundles.iter().take(20) {
        println!("    {b}");
    }
    println!("distinct shaders on short-only materials: {}", shaders_with_short.len());
    println!("\n-- SHORT-spelling materials with dissolve/disturb2 ACTUALLY ENABLED, per bundle --");
    let mut v: Vec<_> = per_bundle.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1));
    for (b, n) in v.iter().take(25) {
        println!("  {n:4}  {b}");
    }
    println!("bundles with at least one ACTIVE short material: {}", per_bundle.len());
    println!("\n-- materials where the effect is NON-TRIVIAL (nonzero warp or dissolve amount) --");
    let mut e: Vec<_> = effective.iter().collect();
    e.sort_by(|a, b| b.1.cmp(a.1));
    for (b, n) in e.iter().take(20) {
        println!("  {n:4}  {b}");
    }
    println!("bundles with a NON-TRIVIAL short material: {}", effective.len());
}
