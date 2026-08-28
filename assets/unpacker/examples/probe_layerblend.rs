//! THROWAWAY diagnostic: what BLEND does a scene quad's material actually declare?
//!
//! Motivation. Two dynchar anomalies have the signature of a blend-mode mismatch rather than a
//! transparency error. Kal'tsit's scene layers 43/44/45 leave a residual whose per-channel
//! regression slopes read R 0.471, G 0.250, B -0.107: an alpha multiply is FLAT across channels
//! and cannot produce a negative one. Whislash-alter's particle foreground sweeps non-monotonically
//! (0.25 worse than 0.50, optimum at 0), which a simple opacity scale also cannot produce.
//!
//! The exporter reduces every material to ONE bit, `additive`, chosen in `export_scene` from the
//! shader family plus texture statistics. `BgQuad` carries `src_blend`/`dst_blend` but they are
//! only used for blend-CLASS heuristics and are never emitted, so the scene JSON cannot say
//! whether a layer is One/OneMinusSrcAlpha (straight alpha), One/One (additive), DstColor/Zero
//! (multiply) or One/OneMinusSrcColor (screen). This prints what the source declares.
//!
//! Unity `BlendMode` enum, the values `_SrcBlend`/`_DstBlend` hold:
//!   0 Zero  1 One  2 DstColor  3 SrcColor  4 OneMinusDstColor  5 SrcAlpha
//!   6 OneMinusSrcColor  7 DstAlpha  8 OneMinusDstAlpha  9 SrcAlphaSaturate  10 OneMinusSrcAlpha
//!
//! Meshes are fingerprinted by TRIANGLE INDEX COUNT, which is what the exported scene JSON
//! records, so a row here can be matched back to a `layers[i]` entry without guessing.
//!
//! Usage: cargo run --release --example probe_layerblend -- <bundle.ab> [<bundle.ab> ...]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::export::shader_map::{build_shader_map, resolve_shader};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pptr_ids(v: &Value) -> Option<(i64, i64)> {
    Some((
        v.get("m_FileID").and_then(Value::as_i64)?,
        v.get("m_PathID").and_then(Value::as_i64)?,
    ))
}

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

/// `m_SavedProperties.m_Floats` is a MAP keyed by property name. `spine.rs:1937` reads it with a
/// direct `.get(k)`, so mirror that exactly rather than inventing a second shape for it.
fn float_of(mat: &Value, k: &str) -> Option<f64> {
    mat.get("m_SavedProperties")
        .and_then(|s| s.get("m_Floats"))
        .and_then(|f| f.get(k))
        .and_then(Value::as_f64)
}

/// `_TintColor` / `_MainColor` live in `m_Colors`, not `m_Floats`. Which one a material carries
/// decides whether `legacy_tint_scale` (spine.rs:2994) applies its x2, so printing the value is
/// the only way to tell an authored 0.5 (x2 neutral white) from an un-doubled literal grey.
fn color_of(mat: &Value, k: &str) -> Option<[f64; 4]> {
    let c = mat
        .get("m_SavedProperties")
        .and_then(|s| s.get("m_Colors"))
        .and_then(|f| f.get(k))?;
    Some([
        c.get("r").and_then(Value::as_f64)?,
        c.get("g").and_then(Value::as_f64)?,
        c.get("b").and_then(Value::as_f64)?,
        c.get("a").and_then(Value::as_f64)?,
    ])
}

fn blend_name(v: f64) -> &'static str {
    match v as i64 {
        0 => "Zero",
        1 => "One",
        2 => "DstColor",
        3 => "SrcColor",
        4 => "OneMinusDstColor",
        5 => "SrcAlpha",
        6 => "OneMinusSrcColor",
        7 => "DstAlpha",
        8 => "OneMinusDstAlpha",
        9 => "SrcAlphaSaturate",
        10 => "OneMinusSrcAlpha",
        _ => "?",
    }
}

fn main() {
    // A material references its shader in ANOTHER bundle, so the name only resolves against the
    // external map. Built from the whole asset tree exactly as `main.rs:158` does; the builder
    // already filters to shader bundles, so the walk is cheap.
    let root = std::path::Path::new("/Users/eltik/Documents/Coding/myrtle/assets/ArkAssets/en");
    let files: Vec<std::path::PathBuf> = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();
    let shader_map = build_shader_map(&files);
    eprintln!(
        "shader map: {} entries from {} files",
        shader_map.len(),
        files.len()
    );
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else {
            eprintln!("cannot read {path}");
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            eprintln!("not a bundle: {path}");
            continue;
        };
        println!("=== {path} ===");
        // One pass to index everything by path_id, because a renderer reaches its mesh through
        // its GameObject's MeshFilter and its material through m_Materials, and both are
        // arbitrary forward references within the file.
        let mut objs: HashMap<i64, (i32, Value)> = HashMap::new();
        let mut shader_of: HashMap<i64, String> = HashMap::new();
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
                continue;
            };
            for obj in &sf.objects {
                if let Ok(v) = read_object(&sf, obj) {
                    // Resolve HERE, where this SerializedFile's `externals` are in scope, the same
                    // constraint that forces `main.rs:414` to do it inside its own walk.
                    if obj.class_id == 21
                        && let Some((fid, spid)) = v.get("m_Shader").and_then(pptr_ids)
                        && let Some(n) = resolve_shader(&sf.externals, fid, spid, &shader_map)
                    {
                        shader_of.insert(obj.path_id, n.to_string());
                    }
                    objs.insert(obj.path_id, (obj.class_id, v));
                }
            }
        }
        // GameObject path_id -> its components, so a renderer can find its sibling MeshFilter.
        let mut go_components: HashMap<i64, Vec<i64>> = HashMap::new();
        for (id, (cid, v)) in &objs {
            if *cid == 1 {
                let comps = v
                    .get("m_Component")
                    .and_then(Value::as_array)
                    .map(|a| {
                        a.iter()
                            .filter_map(|c| c.get("component").and_then(pid).or_else(|| pid(c)))
                            .collect()
                    })
                    .unwrap_or_default();
                go_components.insert(*id, comps);
            }
        }
        let idx_count_of = |mesh_pid: i64| -> usize {
            objs.get(&mesh_pid)
                .and_then(|(_, m)| m.get("m_IndexBuffer"))
                .and_then(Value::as_array)
                .map_or(0, |a| a.len() / 2)
        };
        let mut rows: Vec<(usize, String, String, String, String, String)> = Vec::new();
        // 23 = MeshRenderer (scene quads), 199 = ParticleSystemRenderer (emitters). Whislash-alter's
        // anomaly is on the PARTICLE side, and `is_additive` is shared by both paths, so both have
        // to be printed or only half the question is answered.
        for (cid, v) in objs.values() {
            if *cid != 23 && *cid != 199 {
                continue;
            }
            let kind = if *cid == 23 { "mesh" } else { "PART" };
            let Some(go) = v.get("m_GameObject").and_then(pid) else {
                continue;
            };
            let go_name = objs
                .get(&go)
                .and_then(|(_, g)| g.get("m_Name"))
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_string();
            // sibling MeshFilter (class 33) -> Mesh
            let mut idx = 0usize;
            for c in go_components.get(&go).into_iter().flatten() {
                if let Some((33, mf)) = objs.get(c) {
                    if let Some(mp) = mf.get("m_Mesh").and_then(pid) {
                        idx = idx_count_of(mp);
                    }
                }
            }
            let mats = v.get("m_Materials").and_then(Value::as_array);
            let Some(mp) = mats.and_then(|a| a.first()).and_then(pid) else {
                continue;
            };
            let Some((_, mat)) = objs.get(&mp) else {
                continue;
            };
            let shader = shader_of.get(&mp).cloned().unwrap_or_else(|| "?".into());
            let g = |k: &str| float_of(mat, k);
            let src = g("_SrcBlend").map_or("-".into(), |v| format!("{}({v:.0})", blend_name(v)));
            let dst = g("_DstBlend").map_or("-".into(), |v| format!("{}({v:.0})", blend_name(v)));
            let extra = [
                "_BlendMode",
                "_ZWrite",
                "_Cull",
                "_Mode",
                "_AlphaPremultiply",
            ]
            .iter()
            .filter_map(|k| g(k).map(|v| format!("{k}={v:.2}")))
            .collect::<Vec<_>>()
            .join(" ");
            let cols = ["_TintColor", "_MainColor", "_Color"]
                .iter()
                .filter_map(|k| {
                    color_of(mat, k)
                        .map(|c| format!("{k}=[{:.3} {:.3} {:.3} {:.3}]", c[0], c[1], c[2], c[3]))
                })
                .collect::<Vec<_>>()
                .join("  ");
            rows.push((
                idx,
                format!("[{kind}] {go_name}"),
                shader,
                src,
                dst,
                format!("{extra}  {cols}"),
            ));
        }
        rows.sort();
        for (idx, go, sh, src, dst, extra) in rows {
            println!("  idx {idx:<5} {go:<26} src={src:<22} dst={dst:<22} {extra}");
            println!("        shader {sh}");
        }
    }
}
