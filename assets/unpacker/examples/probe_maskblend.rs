//! THROWAWAY diagnostic: dump the PASS BLEND STATE of the `Particles-L2D/Mask/Erase` shader.
//!
//! Motivation: Ch'en the Holungday's ragged silhouette is carved by a 200-particle brush field
//! whose material uses `Torappu/Particles-L2D/Mask/Erase`. The exporter's blend classifier is a
//! BOOLEAN (additive / normal) with no erase state, so that system exports as `normal` and paints
//! a haze instead of cutting a hole. Before adding a third state, read the real blend factors
//! (and colour mask) out of the shader rather than assuming `Zero, OneMinusSrcAlpha`.
//!
//! Usage: cargo run --release --example `probe_maskblend` -- <shaders.ab> [name-filter]
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_possible_truncation,
    clippy::similar_names
)]

use serde_json::Value;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

/// Unity's `RenderingCommandBuffer` blend-factor enum, as serialised in `m_State`.
const fn blend_factor(n: i64) -> &'static str {
    match n {
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

/// A blend/colour-mask field is serialised as `{ val: <f>, name: "<prop>" }` — a constant when
/// `name` is empty, otherwise driven by a material property.
fn state_num(v: &Value, key: &str) -> Option<i64> {
    let f = v.get(key)?;
    f.get("val")
        .and_then(Value::as_f64)
        .map(|x| x as i64)
        .or_else(|| f.as_f64().map(|x| x as i64))
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .expect("usage: probe_maskblend <shaders.ab> [filter]");
    let filter = args
        .next()
        .unwrap_or_else(|| "mask/erase".into())
        .to_ascii_lowercase();

    let data = std::fs::read(&path).expect("read bundle");
    let bundle = BundleFile::parse(data).expect("parse bundle");

    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        for obj in &sf.objects {
            if obj.class_id != 48 {
                continue; // Shader
            }
            let Ok(v) = read_object(&sf, obj) else {
                continue;
            };
            let name = v
                .get("m_ParsedForm")
                .and_then(|p| p.get("m_Name"))
                .or_else(|| v.get("m_Name"))
                .and_then(Value::as_str)
                .unwrap_or("");
            if !name.to_ascii_lowercase().contains(&filter) {
                continue;
            }
            println!("\n=== SHADER '{name}'  (pathID {})", obj.path_id);
            // Which colour properties the shader DECLARES. A property absent here is inert
            // residue on the material no matter what value it carries — that is the difference
            // between a live `_TintColor` and a leftover from the shader the asset was authored
            // against, and it decides whether the legacy x2 convention applies at all.
            if let Some(props) = v
                .get("m_ParsedForm")
                .and_then(|p| p.get("m_PropInfo"))
                .and_then(|p| p.get("m_Props"))
                .and_then(Value::as_array)
            {
                let names: Vec<String> = props
                    .iter()
                    .filter_map(|p| p.get("m_Name").and_then(Value::as_str))
                    .filter(|n| {
                        let l = n.to_ascii_lowercase();
                        l.contains("color") || l.contains("tint")
                    })
                    .map(str::to_string)
                    .collect();
                println!("  declared colour props: {names:?}");
            }
            let subs = v
                .get("m_ParsedForm")
                .and_then(|p| p.get("m_SubShaders"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for (si, sub) in subs.iter().enumerate() {
                let passes = sub
                    .get("m_Passes")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                for (pi, pass) in passes.iter().enumerate() {
                    let Some(st) = pass.get("m_State") else {
                        continue;
                    };
                    let rt0 = st
                        .get("rtBlend0")
                        .or_else(|| st.get("m_RtBlend0"))
                        .unwrap_or(st);
                    let src = state_num(rt0, "srcBlend").map_or("-", blend_factor);
                    let dst = state_num(rt0, "destBlend").map_or("-", blend_factor);
                    let srca = state_num(rt0, "srcBlendAlpha").map_or("-", blend_factor);
                    let dsta = state_num(rt0, "destBlendAlpha").map_or("-", blend_factor);
                    let mask = state_num(rt0, "colMask").unwrap_or(-1);
                    let zw = state_num(st, "zWrite").unwrap_or(-1);
                    let pname = st.get("m_Name").and_then(Value::as_str).unwrap_or("");
                    println!(
                        "  sub{si} pass{pi} '{pname}'  Blend {src} {dst}, {srca} {dsta}   ColorMask={mask} ZWrite={zw}"
                    );
                }
            }
        }
    }
}
