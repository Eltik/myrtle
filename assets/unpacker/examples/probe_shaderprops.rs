//! THROWAWAY diagnostic: dump the raw property-name strings (and nearby float defaults)
//! of a named compiled Shader inside a shader bundle, so we can tell whether e.g.
//! `Torappu/Particles-L2D/Additive` really declares `_TintColor` (the Unity legacy
//! particle ×2 convention) or whether a material's `_TintColor` is an inert leftover.
//!
//! Usage: cargo run --release --example `probe_shaderprops` -- <shaders.ab> <name-filter>
#![allow(clippy::cast_possible_truncation)]

use unpacker::unity::{bundle::BundleFile, serialized_file::SerializedFile};

fn scan_name(seg: &[u8]) -> Option<String> {
    let is_name = |b: u8| b.is_ascii_graphic() || b == b' ';
    let mut i = 0usize;
    while i < seg.len() {
        if seg[i].is_ascii_uppercase() {
            let s = &seg[i..];
            let e = s.iter().position(|&b| !is_name(b)).unwrap_or(s.len());
            let run = &s[..e];
            if run.len() >= 10
                && run.contains(&b'/')
                && (run.starts_with(b"Torappu")
                    || run.starts_with(b"Hidden")
                    || run.starts_with(b"Custom"))
            {
                return Some(String::from_utf8_lossy(run).to_string());
            }
            i += e.max(1);
        } else {
            i += 1;
        }
    }
    None
}

fn main() {
    let path = std::env::args().nth(1).expect("shaders bundle");
    let filter = std::env::args().nth(2).unwrap_or_default();
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    for entry in &bundle.files {
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        for obj in &sf.objects {
            if obj.class_id != 48 {
                continue;
            }
            let start = obj.byte_start as usize;
            let end = (start + obj.byte_size as usize).min(sf.data.len());
            if start >= end {
                continue;
            }
            let seg = &sf.data[start..end];
            let Some(name) = scan_name(seg) else { continue };
            if !filter.is_empty() && name != filter {
                continue;
            }
            println!("=== {name}  (pid={}, {} bytes)", obj.path_id, seg.len());
            // Property block: a run of length-prefixed ASCII names. Print every
            // `_`-prefixed identifier with the 4 floats that follow its description
            // (SerializedProperty: m_Name, m_Description, m_Attributes, m_Type,
            //  m_Flags, m_DefValue[4], m_DefTexture{...}).
            let mut i = 0usize;
            while i + 4 < seg.len() {
                let len = u32::from_le_bytes([seg[i], seg[i + 1], seg[i + 2], seg[i + 3]]) as usize;
                if (2..=48).contains(&len) && i + 4 + len <= seg.len() && seg[i + 4] == b'_' {
                    let s = &seg[i + 4..i + 4 + len];
                    if s.iter().all(|&b| b.is_ascii_alphanumeric() || b == b'_') {
                        // SerializedProperty: m_Name, m_Description, m_Attributes(vec<string>),
                        // m_Type(i32), m_Flags(u32), m_DefValue[4](f32) — all strings align4.
                        let mut j = i + 4 + len;
                        j = (j + 3) & !3usize;
                        let rd = |p: usize| -> usize {
                            u32::from_le_bytes([seg[p], seg[p + 1], seg[p + 2], seg[p + 3]])
                                as usize
                        };
                        let skip_str = |p: &mut usize| {
                            if *p + 4 > seg.len() {
                                return;
                            }
                            let l = rd(*p);
                            *p = ((*p + 4 + l) + 3) & !3usize;
                        };
                        skip_str(&mut j); // m_Description
                        if j + 4 <= seg.len() {
                            let n = rd(j);
                            j += 4;
                            for _ in 0..n.min(8) {
                                skip_str(&mut j);
                            }
                        }
                        let mut vals = [0f32; 4];
                        let (mut ty, mut flags) = (0i32, 0u32);
                        if j + 24 <= seg.len() {
                            ty = i32::from_le_bytes(seg[j..j + 4].try_into().unwrap());
                            flags = u32::from_le_bytes(seg[j + 4..j + 8].try_into().unwrap());
                            for (k, v) in vals.iter_mut().enumerate() {
                                let p = j + 8 + k * 4;
                                *v = f32::from_le_bytes(seg[p..p + 4].try_into().unwrap());
                            }
                        }
                        println!(
                            "  prop @{i}: {}  type={ty} flags={flags} def=({:.4},{:.4},{:.4},{:.4})",
                            String::from_utf8_lossy(s),
                            vals[0],
                            vals[1],
                            vals[2],
                            vals[3]
                        );
                        i += 4 + len;
                        continue;
                    }
                }
                i += 1;
            }
            // Blend-state names live in the pass's SerializedShaderState as plain
            // strings too; print any that look like fixed-function state.
            for kw in [
                "Blend",
                "SrcBlend",
                "DstBlend",
                "_SrcBlend",
                "_DstBlend",
                "One",
                "SrcAlpha",
            ] {
                let n = seg
                    .windows(kw.len())
                    .filter(|w| *w == kw.as_bytes())
                    .count();
                if n > 0 {
                    println!("  kw {kw}: {n}");
                }
            }
        }
    }
}
