//! THROWAWAY diagnostic: hunt for any colour-transform data (post-processing / colour
//! grading `MonoBehaviours`, Camera HDR/clear-flags, LUT-like textures, grading/tonemap/
//! composite materials) in a dynchar bundle. Motivation: our renderer is measured to be
//! systematically too bright vs the in-game capture, and the per-pixel error fits a pure
//! sRGB gamma whose magnitude differs per skin — which smells like a per-scene colour
//! transform (post-process volume / LUT / camera property) our exporter isn't reading.
//!
//! Usage: cargo run --release --example `probe_grade` -- <bundle.ab> [`shaders_dir`]
//! `shaders_dir` defaults to assets/ArkAssets/en (where `[uc]shaders.ab` lives), used only
//! to resolve external material shader names.
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use unpacker::export::shader_map::{build_shader_map, resolve_shader};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

const KEYWORDS: &[&str] = &[
    "postprocess",
    "postprocessing",
    "colorgrading",
    "colorcorrection",
    "grading",
    "tonemap",
    "tonemapping",
    "lut",
    "colorlookup",
    "bloom",
    "vignette",
    "grade",
    "coloradjust",
    "exposure",
    "gamma",
    "brightness",
    "contrast",
    "saturation",
];

fn matches_keyword(s: &str) -> bool {
    let low = s.to_ascii_lowercase();
    KEYWORDS.iter().any(|k| low.contains(k))
}

fn walkdir(root: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        if d.is_file() {
            out.push(d);
            continue;
        }
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            stack.push(e.path());
        }
    }
    out
}

fn pptr(v: Option<&Value>) -> (i64, i64) {
    let fid = v
        .and_then(|s| s.get("m_FileID"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let pid = v
        .and_then(|s| s.get("m_PathID"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    (fid, pid)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("bundle path");
    let shader_dir = args
        .next()
        .unwrap_or_else(|| "/Users/eltik/Documents/Coding/myrtle/assets/ArkAssets/en".to_string());

    let shader_files = walkdir(&PathBuf::from(&shader_dir));
    let shader_map = build_shader_map(&shader_files);
    eprintln!("shader map entries: {}", shader_map.len());

    println!("\n================================================================");
    println!("BUNDLE {path}");
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
        println!("-- entry {} : {} objects", entry.path, sf.objects.len());
        println!(
            "   externals: {:?}",
            sf.externals
                .iter()
                .map(unpacker::unity::serialized_file::FileIdentifier::cab_name)
                .collect::<Vec<_>>()
        );

        // Skip only heavy binary-blob classes we don't need (keep Texture2D=28 for
        // name/dims, and everything else, since these bundles are small dynchar scenes).
        let skip: HashSet<i32> = [43, 48, 49, 83, 128, 213].into_iter().collect();
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for obj in &sf.objects {
            if skip.contains(&obj.class_id) {
                continue;
            }
            if let Ok(v) = read_object(&sf, obj) {
                all.insert(obj.path_id, (obj.class_id, v));
            }
        }

        // ---- hierarchy (GameObject names + paths) ----
        let mut go_name: HashMap<i64, String> = HashMap::new();
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
                }
                4 | 224 => {
                    if let Some(g) = v.get("m_GameObject").and_then(pid) {
                        tf_go.insert(*p, g);
                        go_tf.insert(g, *p);
                    }
                    if let Some(f) = v.get("m_Father").and_then(pid) {
                        tf_father.insert(*p, f);
                    }
                }
                _ => {}
            }
        }
        let go_path = |go: i64| -> String {
            let mut parts: Vec<String> = Vec::new();
            let mut cur = go_tf.get(&go).copied();
            for _ in 0..128 {
                let Some(tf) = cur else { break };
                if let Some(g) = tf_go.get(&tf) {
                    parts.push(go_name.get(g).cloned().unwrap_or_else(|| format!("?{g}")));
                }
                cur = tf_father.get(&tf).copied().filter(|&f| f != 0);
            }
            parts.reverse();
            parts.join("/")
        };

        // ---- scene root GameObjects (transforms with no in-file father) ----
        let mut roots: Vec<i64> = Vec::new();
        for (tp, (cid, _)) in &all {
            if *cid != 4 && *cid != 224 {
                continue;
            }
            match tf_father.get(tp) {
                Some(f) if *f != 0 && all.contains_key(f) => {}
                _ => roots.push(*tp),
            }
        }
        roots.sort_unstable();
        for tp in &roots {
            if let Some(g) = tf_go.get(tp) {
                println!(
                    "   ROOT GO: '{}' (go pid={g})",
                    go_name.get(g).cloned().unwrap_or_default()
                );
            }
        }

        // ---- MonoScript class names (internal only; external ones we can't resolve) ----
        let mut script_name: HashMap<i64, String> = HashMap::new();
        for (p, (cid, v)) in &all {
            if *cid == 115 {
                script_name.insert(
                    *p,
                    v.get("m_ClassName")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                );
            }
        }

        // ---- every MonoBehaviour: resolve script identity, flag keyword matches ----
        let mut distinct: HashSet<String> = HashSet::new();
        let mut listing: Vec<String> = Vec::new();
        for (p, (cid, v)) in &all {
            if *cid != 114 {
                continue;
            }
            let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
            let (fid, spid) = pptr(v.get("m_Script"));
            let ident = if fid == 0 {
                script_name
                    .get(&spid)
                    .cloned()
                    .unwrap_or_else(|| format!("<local script pid {spid} not found>"))
            } else {
                let cab = sf
                    .externals
                    .get((fid - 1).max(0) as usize)
                    .map(|d| d.cab_name().to_string())
                    .unwrap_or_default();
                format!("EXTERNAL[{cab}]:pid{spid}")
            };
            distinct.insert(ident.clone());
            let name_field = v.get("m_Name").and_then(Value::as_str).unwrap_or("");
            let keys: Vec<String> = v
                .as_object()
                .map(|o| o.keys().cloned().collect())
                .unwrap_or_default();
            let key_flag = keys.iter().any(|k| matches_keyword(k));
            let ident_flag = matches_keyword(&ident);
            if ident_flag || key_flag {
                println!(
                    "   *** KEYWORD MATCH *** MonoBehaviour pid={p} on '{}' script='{ident}' m_Name='{name_field}'",
                    go_path(go)
                );
                println!(
                    "       fields: {}",
                    serde_json::to_string(v).unwrap_or_default()
                );
            }
            listing.push(format!(
                "pid={p} go='{}' script='{ident}' m_Name='{name_field}'",
                go_path(go)
            ));
        }
        listing.sort();
        println!("   -- all MonoBehaviours ({}) --", listing.len());
        for l in &listing {
            println!("      {l}");
        }
        let mut dv: Vec<String> = distinct.into_iter().collect();
        dv.sort();
        println!(
            "   >> distinct MonoBehaviour script identities ({}): {dv:?}",
            dv.len()
        );

        // ---- Camera components: dump every field verbatim ----
        for (p, (cid, v)) in &all {
            if *cid != 20 {
                continue;
            }
            let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
            println!("   CAMERA pid={p} on '{}'", go_path(go));
            println!(
                "     {}",
                serde_json::to_string_pretty(v).unwrap_or_default()
            );
            // Any MonoBehaviours (image effects) on the same GameObject.
            for l in &listing {
                if l.contains(&format!("go='{}'", go_path(go))) {
                    println!("     (co-located MonoBehaviour: {l})");
                }
            }
        }

        // ---- Texture2D: flag LUT-like names, list all names for context ----
        let mut all_tex: Vec<String> = Vec::new();
        for (p, (cid, v)) in &all {
            if *cid != 28 {
                continue;
            }
            let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("");
            let w = v.get("m_Width").and_then(Value::as_i64).unwrap_or(-1);
            let h = v.get("m_Height").and_then(Value::as_i64).unwrap_or(-1);
            let fmt = v
                .get("m_TextureFormat")
                .and_then(Value::as_i64)
                .unwrap_or(-1);
            all_tex.push(format!("{name} ({w}x{h} fmt{fmt})"));
            let low = name.to_ascii_lowercase();
            if low.contains("lut")
                || low.contains("ramp")
                || low.contains("grade")
                || low.contains("curve")
            {
                println!(
                    "   *** LUT-CANDIDATE TEXTURE *** pid={p} name='{name}' {w}x{h} fmt={fmt}"
                );
            }
        }
        all_tex.sort();
        println!("   -- all Texture2D names ({}) --", all_tex.len());
        for t in &all_tex {
            println!("      {t}");
        }

        // ---- Materials: flag grading/post/composite-ish shader names ----
        for (p, (cid, v)) in &all {
            if *cid != 21 {
                continue;
            }
            let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("");
            let (fid, spid) = pptr(v.get("m_Shader"));
            let shader_name = resolve_shader(&sf.externals, fid, spid, &shader_map)
                .unwrap_or("<unresolved-or-local>");
            let low = shader_name.to_ascii_lowercase();
            for k in [
                "grad",
                "lut",
                "tone",
                "post",
                "blit",
                "composite",
                "copy",
                "screen",
                "final",
            ] {
                if low.contains(k) {
                    println!(
                        "   *** SHADER KEYWORD MATCH *** Material pid={p} name='{name}' shader='{shader_name}'"
                    );
                    break;
                }
            }
        }
    }
}
