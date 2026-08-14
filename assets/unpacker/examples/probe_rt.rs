//! THROWAWAY diagnostic: is the dynchar scene rendered into an ALPHA-BEARING target?
//!
//! Motivation: every `Particles-L2D` port writes alpha as `Zero OneMinusSrcAlpha`
//! (`dstA = dstA * (1 - srcA)` — drawing REMOVES destination alpha), with `Mask/Erase` the one
//! corpus-wide exception. If the scene composites into an alpha-bearing `RenderTexture` that the UI
//! then draws over its white backdrop, that carved alpha IS Ch'en's ragged silhouette and needs no
//! mask geometry. That only holds if the target actually carries alpha AND the camera clears to a
//! transparent colour — an opaque target makes every alpha write inert.
//!
//! Prints, for each Camera (class 20): clear flags, background colour (ALPHA is the tell),
//! HDR/MSAA, and any `m_TargetTexture`; plus every `RenderTexture` (class 84) with its format and
//! depth, and every GameObject-owned Canvas/RawImage-ish `MonoBehaviour` is ignored (out of scope).
//!
//! Usage: cargo run --release --example `probe_rt` -- <bundle.ab>
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::many_single_char_names
)]

use serde_json::Value;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

/// Unity `Camera.clearFlags`.
const fn clear_flags(n: i64) -> &'static str {
    match n {
        1 => "Skybox",
        2 => "SolidColor",
        3 => "DepthOnly",
        4 => "Nothing",
        _ => "?",
    }
}

/// The subset of `RenderTextureFormat` that can appear here; the tell is whether the format
/// carries an alpha channel at all.
const fn rt_format(n: i64) -> &'static str {
    match n {
        0 => "ARGB32 (alpha)",
        1 => "Depth",
        2 => "ARGBHalf (alpha)",
        7 => "RGB565 (NO alpha)",
        8 => "ARGB4444 (alpha)",
        9 => "ARGB1555 (alpha)",
        12 => "Default (ARGB32, alpha)",
        14 => "ARGBFloat (alpha)",
        19 => "DefaultHDR (alpha)",
        20 => "R8 (NO alpha)",
        22 => "RGB111110Float (NO alpha)",
        _ => "other",
    }
}

fn f(v: &Value, k: &str) -> Option<f64> {
    v.get(k).and_then(Value::as_f64)
}

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else {
            eprintln!("cannot read {path}");
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            eprintln!("bundle parse failed for {path}");
            continue;
        };
        println!("\n===== {}", path.rsplit('/').next().unwrap_or(&path));
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
                continue;
            };
            for obj in &sf.objects {
                match obj.class_id {
                    20 => {
                        let Ok(v) = read_object(&sf, obj) else {
                            continue;
                        };
                        let cf = v.get("m_ClearFlags").and_then(Value::as_i64).unwrap_or(-1);
                        let bg = v.get("m_BackGroundColor");
                        let (r, g, b, a) = bg.map_or((-1.0, -1.0, -1.0, -1.0), |c| {
                            (
                                f(c, "r").unwrap_or(-1.0),
                                f(c, "g").unwrap_or(-1.0),
                                f(c, "b").unwrap_or(-1.0),
                                f(c, "a").unwrap_or(-1.0),
                            )
                        });
                        let tt = v
                            .get("m_TargetTexture")
                            .and_then(|t| t.get("m_PathID"))
                            .and_then(Value::as_i64)
                            .unwrap_or(0);
                        let hdr = v.get("m_AllowHDR").and_then(Value::as_i64).unwrap_or(-1);
                        println!(
                            "  CAMERA pid={}  clear={} ({cf})  bg=({r:.3},{g:.3},{b:.3},A={a:.3})  \
                             targetTexture={tt}  allowHDR={hdr}",
                            obj.path_id,
                            clear_flags(cf)
                        );
                        if a == 0.0 && cf == 2 {
                            println!(
                                "      -> clears to TRANSPARENT: target is alpha-bearing and composited by the UI"
                            );
                        } else if cf == 2 {
                            println!(
                                "      -> clears to an OPAQUE colour: alpha writes never reach a backdrop"
                            );
                        }
                    }
                    84 => {
                        let Ok(v) = read_object(&sf, obj) else {
                            continue;
                        };
                        let n = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
                        let fmt = v.get("m_ColorFormat").and_then(Value::as_i64).unwrap_or(-1);
                        println!(
                            "  RENDERTEXTURE '{n}'  {}x{}  format={} ({fmt})  depth={}",
                            v.get("m_Width").and_then(Value::as_i64).unwrap_or(-1),
                            v.get("m_Height").and_then(Value::as_i64).unwrap_or(-1),
                            rt_format(fmt),
                            v.get("m_DepthFormat").and_then(Value::as_i64).unwrap_or(-1)
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}
