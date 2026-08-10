//! THROWAWAY scan: map each ParticleSystem to its RENDERER's material — shader name,
//! `_MainColor` / `_TintColor`, and the `_MainTex` dimensions — in the SAME order the
//! exporter emits systems, so a system index in `[particles].json` can be tied to the
//! material that actually draws it.
//!
//! Motivation: Virtuosa's magenta haze wisps (systems 3/10/27/46 of her `_Start` set) render
//! R−G ≈ +40 where the game reads +5. Their texture is a UNIFORM hot pink (253,82,141) with a
//! shaped alpha — the signature of a shader WARP INPUT rather than a drawable sprite. The
//! particle exporter also ignores `_MainColor` outright (`particle_tint` excludes every
//! sub-namespaced family), so a material that modulates its sprite through `_MainColor` is
//! drawn un-modulated. Both questions need the system→material link, which no exported file
//! carries.
//!
//! Usage: cargo run --release --example scan_psmat -- <bundle.ab>

use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use unpacker::export::shader_map::{build_shader_map, resolve_shader};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn col(mat: &Value, key: &str) -> String {
    mat.get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Colors"))
        .and_then(|c| c.get(key))
        .map(|c| {
            let g = |k: &str| c.get(k).and_then(Value::as_f64).unwrap_or(-1.0);
            format!("({:.3},{:.3},{:.3},{:.3})", g("r"), g("g"), g("b"), g("a"))
        })
        .unwrap_or_else(|| "-".into())
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle path");
    let map = build_shader_map(&[PathBuf::from("../ArkAssets/en/[uc]shaders.ab")]);
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");

    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for obj in &sf.objects {
            if let Ok(v) = read_object(&sf, obj) {
                all.insert(obj.path_id, (obj.class_id, v));
            }
        }
        // GameObject -> its ParticleSystemRenderer (class 199), in ascending pathID order so
        // the listing matches the exporter's own traversal.
        let mut ordered: Vec<(&i64, &(i32, Value))> = all.iter().collect();
        ordered.sort_by_key(|(p, _)| **p);

        let mut n = 0usize;
        for (p, (cid, v)) in &ordered {
            if *cid != 199 {
                continue;
            }
            let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
            let name = all
                .get(&go)
                .and_then(|(_, g)| g.get("m_Name").and_then(Value::as_str))
                .unwrap_or("?");
            let mat_pid = v
                .get("m_Materials")
                .and_then(Value::as_array)
                .and_then(|a| a.first())
                .and_then(pid)
                .unwrap_or(0);
            let Some((_, mat)) = all.get(&mat_pid) else {
                println!("  ps#{n:<3} '{name}' renderer={p} material UNRESOLVED");
                n += 1;
                continue;
            };
            let shader = mat
                .get("m_Shader")
                .and_then(|s| {
                    let fid = s.get("m_FileID").and_then(Value::as_i64)?;
                    let sid = pid(s)?;
                    resolve_shader(&sf.externals, fid, sid, &map)
                })
                .unwrap_or_else(|| "?".into());
            // `_MainTex` dimensions — a uniform-colour texture is a warp input, not a sprite.
            let tex = mat
                .get("m_SavedProperties")
                .and_then(|sp| sp.get("m_TexEnvs"))
                .and_then(|t| t.get("_MainTex"))
                .and_then(|t| t.get("m_Texture"))
                .and_then(pid)
                .and_then(|tp| all.get(&tp))
                .map(|(_, t)| {
                    format!(
                        "{}x{}",
                        t.get("m_Width").and_then(Value::as_i64).unwrap_or(-1),
                        t.get("m_Height").and_then(Value::as_i64).unwrap_or(-1)
                    )
                })
                .unwrap_or_else(|| "none".into());
            println!(
                "  ps#{n:<3} '{:<28}' {:<52} main={:<26} tint={:<26} tex={tex}",
                &name[..name.len().min(28)],
                &shader[..shader.len().min(52)],
                col(mat, "_MainColor"),
                col(mat, "_TintColor")
            );
            n += 1;
        }
        if n > 0 {
            println!("\n{n} ParticleSystemRenderers in {}", entry.path);
        }
    }
}
