//! Resolve a material's **external** shader reference to the shader's name.
//!
//! Particle materials in dynchar bundles reference their shader by an external
//! PPtr (`m_Shader = {m_FileID, m_PathID}`) into the shared `[uc]shaders.ab`
//! bundle, so the name isn't present in the dynchar bundle. We pre-scan the
//! shader bundles once into a `(CAB, path_id) -> name` map, then resolve each
//! material's ref via its SerializedFile's `externals` table.
//!
//! Needed to tell apart the HG "flow" shaders that share one pathID: e.g.
//! `Torappu/Particles-L2D/Ram/VertexDisturb(CustomData)` drives both SilverAsh
//! the Reignfrost's procedural targeting-ring reticles (external `_MainTex`) and
//! ordinary slash sprites (in-bundle `_MainTex`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::unity::serialized_file::SerializedFile;

pub type ShaderMap = HashMap<(String, i64), String>;

/// Extract a Shader's name (`m_ParsedForm.m_Name`) by scanning the object bytes.
/// `read_object` overreads on a compiled-shader blob, but the name is a plain
/// early string of the form `Torappu/...`, `Hidden/...` or `Custom/...`.
fn scan_shader_name(data: &[u8], start: usize, size: usize) -> Option<String> {
    let end = (start + size).min(data.len());
    if start >= end {
        return None;
    }
    let seg = &data[start..end];
    let is_name = |b: u8| b.is_ascii_graphic() || b == b' ';
    let mut i = 0usize;
    while i < seg.len() {
        if seg[i].is_ascii_uppercase() {
            let s = &seg[i..];
            let e = s.iter().position(|&b| !is_name(b)).unwrap_or(s.len());
            let run = &s[..e];
            if run.len() >= 10 && run.contains(&b'/') && (run.starts_with(b"Torappu") || run.starts_with(b"Hidden") || run.starts_with(b"Custom")) {
                return Some(String::from_utf8_lossy(run).to_string());
            }
            i += e.max(1);
        } else {
            i += 1;
        }
    }
    None
}

/// True for a filename that looks like a shader bundle (`[uc]shaders.ab`,
/// `[uc]uishaders.ab`, `shaders/other.ab`, …).
fn is_shader_bundle(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_ascii_lowercase();
    if !name.ends_with(".ab") {
        return false;
    }
    name.contains("shaders") || path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()).is_some_and(|d| d.eq_ignore_ascii_case("shaders"))
}

/// Build a `(CAB, path_id) -> shader_name` map from every shader bundle among
/// `files`. Empty when none are present (reticle detection then just stays off).
#[must_use]
pub fn build_shader_map(files: &[PathBuf]) -> ShaderMap {
    let mut map = ShaderMap::new();
    for path in files.iter().filter(|p| is_shader_bundle(p)) {
        let Ok(bytes) = std::fs::read(path) else { continue };
        let Ok(bundle) = crate::unity::bundle::BundleFile::parse(bytes) else { continue };
        for entry in &bundle.files {
            let cab = entry.path.clone();
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
            for obj in &sf.objects {
                if obj.class_id != 48 {
                    continue;
                }
                if let Some(name) = scan_shader_name(&sf.data, obj.byte_start as usize, obj.byte_size as usize) {
                    map.insert((cab.clone(), obj.path_id), name);
                }
            }
        }
    }
    map
}

/// Resolve a material's `m_Shader` PPtr (`file_id`, `path_id`) to a shader name,
/// following the SerializedFile's `externals` table. `file_id == 0` means the
/// shader is in this same file (dynchars never inline shaders, so unresolved).
#[must_use]
pub fn resolve_shader<'a>(externals: &[crate::unity::serialized_file::FileIdentifier], file_id: i64, path_id: i64, map: &'a ShaderMap) -> Option<&'a str> {
    if file_id <= 0 {
        return None;
    }
    let dep = externals.get((file_id - 1) as usize)?;
    map.get(&(dep.cab_name().to_string(), path_id)).map(String::as_str)
}
