//! THROWAWAY diagnostic: decompress a Shader's `compressedBlob` and print the GLSL source of its
//! subprograms.
//!
//! Motivation: Virtuosa's ten wing particle rigs draw a saturated PINK wisp where the game shows
//! essentially nothing, and EVERY authored input has been verified correct — pink source texture,
//! neutral `_TintColor`, `maxParticles` respected, director-activated, right UV cell, no mesh
//! vertex colour, and a pass state that is exactly the normal blend we use. The only component
//! left is the fragment PROGRAM, which lives compressed inside the Shader object.
//!
//! Layout (Unity 2019+): `platforms[i]` is a `ShaderCompilerPlatform` (9 = `GLES3Plus`, whose
//! subprograms are plain-text GLSL; 18 = Vulkan/SPIR-V, which is not). `offsets[i][j]` /
//! `compressedLengths[i][j]` slice `compressedBlob`, and `decompressedLengths[i][j]` is the LZ4
//! output size. The decompressed blob is a subprogram index (count, then (offset,length) pairs)
//! followed by the entries; rather than parse that, this scans for printable runs, which is
//! enough to read GLSL out.
//!
//! Usage: cargo run --release --example `probe_shadersrc` -- <shaders.ab> <pathID> [min-run]
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::many_single_char_names,
    clippy::too_many_lines
)]

use serde_json::Value;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

/// `read_object` surfaces byte arrays either as a JSON array of numbers or as a base64 string
/// (`base64:` prefixed when the bytes were not valid UTF-8) — accept both.
fn as_bytes(v: &Value) -> Option<Vec<u8>> {
    match v {
        Value::Array(a) => Some(
            a.iter()
                .filter_map(|x| x.as_u64().map(|n| n as u8))
                .collect(),
        ),
        Value::String(s) => {
            use base64::Engine;
            let body = s.strip_prefix("base64:").unwrap_or(s);
            base64::engine::general_purpose::STANDARD.decode(body).ok()
        }
        _ => None,
    }
}

fn nested_u32(v: Option<&Value>) -> Vec<Vec<u32>> {
    let Some(Value::Array(outer)) = v else {
        return Vec::new();
    };
    outer
        .iter()
        .map(|inner| match inner {
            Value::Array(a) => a
                .iter()
                .filter_map(|x| x.as_u64().map(|n| n as u32))
                .collect(),
            other => other.as_u64().map(|n| vec![n as u32]).unwrap_or_default(),
        })
        .collect()
}

const fn platform_name(p: u32) -> &'static str {
    match p {
        0 => "OpenGL",
        4 => "D3D11",
        5 => "GLES20",
        9 => "GLES3Plus",
        14 => "Metal",
        15 => "OpenGLCore",
        18 => "Vulkan",
        _ => "?",
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("shaders.ab");
    let want: i64 = std::env::args()
        .nth(2)
        .expect("pathID")
        .parse()
        .expect("pathID");
    let min_run: usize = std::env::args()
        .nth(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(40);

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
        for obj in &sf.objects {
            if obj.class_id != 48 || obj.path_id != want {
                continue;
            }
            let Ok(v) = read_object(&sf, obj) else {
                continue;
            };
            let name = v
                .get("m_ParsedForm")
                .and_then(|p| p.get("m_Name"))
                .and_then(Value::as_str)
                .unwrap_or("?");
            let Some(blob) = v.get("compressedBlob").and_then(as_bytes) else {
                println!("no compressedBlob");
                return;
            };
            let platforms: Vec<u32> = v
                .get("platforms")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_u64().map(|n| n as u32))
                        .collect()
                })
                .unwrap_or_default();
            let offsets = nested_u32(v.get("offsets"));
            let clens = nested_u32(v.get("compressedLengths"));
            let dlens = nested_u32(v.get("decompressedLengths"));
            println!(
                "SHADER '{name}'  blob {} bytes  platforms {platforms:?}",
                blob.len()
            );

            for (i, p) in platforms.iter().enumerate() {
                let (Some(offs), Some(cl), Some(dl)) = (offsets.get(i), clens.get(i), dlens.get(i))
                else {
                    continue;
                };
                for j in 0..offs.len().min(cl.len()).min(dl.len()) {
                    let (o, c, d) = (offs[j] as usize, cl[j] as usize, dl[j] as usize);
                    if o + c > blob.len() {
                        println!(
                            "  platform {p} ({}) seg {j}: slice out of range",
                            platform_name(*p)
                        );
                        continue;
                    }
                    let out = match lz4::block::decompress(&blob[o..o + c], Some(d as i32)) {
                        Ok(x) => x,
                        Err(e) => {
                            println!(
                                "  platform {p} ({}) seg {j}: lz4 failed: {e}",
                                platform_name(*p)
                            );
                            continue;
                        }
                    };
                    println!(
                        "\n  ===== platform {p} ({}) seg {j}: {} bytes decompressed",
                        platform_name(*p),
                        out.len()
                    );
                    // Printable runs — GLES3Plus subprograms are plain-text GLSL.
                    let mut run: Vec<u8> = Vec::new();
                    let flush = |r: &mut Vec<u8>| {
                        if r.len() >= min_run {
                            println!("{}", String::from_utf8_lossy(r));
                            println!("  ---");
                        }
                        r.clear();
                    };
                    for &b in &out {
                        if b == b'\n' || b == b'\t' || (0x20..0x7f).contains(&b) {
                            run.push(b);
                        } else {
                            flush(&mut run);
                        }
                    }
                    flush(&mut run);
                }
            }
            return;
        }
    }
    println!("shader pathID {want} not found");
}
