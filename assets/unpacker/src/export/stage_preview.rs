//! Unsquash the squashed `stage_mappreview_h2_*` previews.
//!
//! Arknights ships stage map thumbnails as 512×512 ASTC squares, but the
//! source renders are wider. The Sprite metadata reports `m_Rect = 512×512`
//! and the game stretches them at runtime via UI layout we don't have. We
//! approximate the original aspect from the level's tile grid (cols × rows
//! in the `level_<stage_id>` `TextAsset`'s `MapData.Map`), which matches the
//! camera-space proportions closely enough that square tiles read as square.
//!
//! The aspect map is built once up front by a parallel pre-pass over the
//! input bundles, then consulted per-texture in the main extraction loop.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::prelude::*;
use serde_json::Value;

use crate::unity::bundle::BundleFile;
use crate::unity::object_reader::read_object;
use crate::unity::serialized_file::SerializedFile;

use super::text_asset;
use super::texture::DecodedTexture;

/// `stage_id` (e.g. "`a001_01`") → (columns, rows) of the level's tile grid.
pub type StageAspectMap = Arc<HashMap<String, (u32, u32)>>;

/// Fallback aspect (width / height) when a stage isn't found in the map —
/// keeps the result closer to in-game than a raw square.
const FALLBACK_ASPECT: f32 = 16.0 / 9.0;

const MAPPREVIEW_PATH_MARKER: &str = "stage_mappreview_h2_";

/// True if a bundle's relative path identifies it as a mappreview h2 bundle
/// whose textures need unsquashing.
#[must_use]
pub fn detect_mappreview_bundle(bundle_subdir: &Path) -> bool {
    bundle_subdir
        .to_string_lossy()
        .contains(MAPPREVIEW_PATH_MARKER)
}

/// Fast check on a raw input path so we can skip the pre-pass when no
/// mappreview bundles are present.
#[must_use]
pub fn input_has_mappreview_bundles(files: &[PathBuf]) -> bool {
    files
        .iter()
        .any(|p| p.to_string_lossy().contains(MAPPREVIEW_PATH_MARKER))
}

/// Pre-scan every input bundle for `level_<stage_id>` `TextAssets` and build
/// the `stage_id` → (cols, rows) map. The scan parses each bundle's serialized
/// files but only deserializes class 49 (`TextAsset`) objects, so non-text
/// bundles cost only their `BundleFile::parse`.
#[must_use]
pub fn build_stage_aspect_map(files: &[PathBuf]) -> StageAspectMap {
    let entries: Vec<(String, (u32, u32))> = files
        .par_iter()
        .flat_map(|file| scan_bundle_for_levels(file).unwrap_or_default())
        .collect();
    Arc::new(entries.into_iter().collect())
}

fn scan_bundle_for_levels(file: &Path) -> Option<Vec<(String, (u32, u32))>> {
    let data = std::fs::read(file).ok()?;
    let bundle = BundleFile::parse(data).ok()?;
    let mut out = Vec::new();
    for entry in &bundle.files {
        if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        // Quick reject: skip serialized files with no TextAssets at all.
        if !sf.objects.iter().any(|o| o.class_id == 49) {
            continue;
        }
        for obj in &sf.objects {
            if obj.class_id != 49 {
                continue;
            }
            let Ok(val) = read_object(&sf, obj) else {
                continue;
            };
            let name = val["m_Name"].as_str().unwrap_or("");
            let Some(stage_id) = name.strip_prefix("level_") else {
                continue;
            };
            if let Some(dims) = extract_grid_dims(&val) {
                out.push((stage_id.to_string(), dims));
            }
        }
    }
    Some(out)
}

fn extract_grid_dims(text_obj: &Value) -> Option<(u32, u32)> {
    let json = text_asset::parse_text_asset_json(text_obj)?;
    let map = json.get("MapData")?.get("Map")?;
    let cols = map.get("Column_size")?.as_u64()? as u32;
    let matrix = map.get("Matrix_data")?.as_array()?;
    if cols == 0 {
        return None;
    }
    let rows = (matrix.len() as u32) / cols;
    if rows == 0 {
        return None;
    }
    Some((cols, rows))
}

/// Resize a mappreview texture in place from its squashed (typically square)
/// form to its natural aspect, derived from the level's tile grid. Falls
/// back to a fixed 16:9 stretch when the stage isn't in the map.
///
/// Texture name (e.g. "`a001_01`") is matched against the keys in `aspects`
/// — this is the same identifier the level `TextAsset` uses as `level_a001_01`.
pub fn unsquash_mappreview_texture(tex: &mut DecodedTexture, aspects: &StageAspectMap) {
    let aspect = match aspects.get(&tex.name) {
        Some(&(cols, rows)) => (cols as f32) / (rows as f32),
        None => FALLBACK_ASPECT,
    };
    let target_h = tex.height;
    let target_w = ((target_h as f32) * aspect).round() as u32;
    if target_w == tex.width && target_h == tex.height {
        return;
    }
    let buf = std::mem::take(&mut tex.rgba);
    let img = image::RgbaImage::from_raw(tex.width, tex.height, buf)
        .expect("RGBA buffer size mismatch in mappreview resize");
    let resized = image::imageops::resize(
        &img,
        target_w,
        target_h,
        image::imageops::FilterType::Triangle,
    );
    tex.width = target_w;
    tex.height = target_h;
    tex.rgba = resized.into_raw();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_mappreview_path() {
        assert!(detect_mappreview_bundle(Path::new(
            "arts/ui/stage_mappreview_h2_a001_1_0"
        )));
        assert!(detect_mappreview_bundle(Path::new(
            "arts/ui/stage_mappreview_h2_act11d0_a_0"
        )));
        assert!(!detect_mappreview_bundle(Path::new(
            "spritepack/sandbox_1_stage_mappreview_0"
        )));
        assert!(!detect_mappreview_bundle(Path::new(
            "arts/ui/stage/[pack]campaignrules"
        )));
    }

    #[test]
    fn unsquash_uses_grid_aspect() {
        let mut map = HashMap::new();
        map.insert("test_stage".to_string(), (10u32, 7u32));
        let aspects: StageAspectMap = Arc::new(map);

        let mut tex = DecodedTexture {
            name: "test_stage".to_string(),
            width: 512,
            height: 512,
            rgba: vec![128u8; 512 * 512 * 4],
        };
        unsquash_mappreview_texture(&mut tex, &aspects);
        // 10/7 * 512 ≈ 731
        assert_eq!(tex.height, 512);
        assert_eq!(tex.width, 731);
        assert_eq!(tex.rgba.len(), (731 * 512 * 4) as usize);
    }

    #[test]
    fn unsquash_falls_back_to_16_9() {
        let aspects: StageAspectMap = Arc::new(HashMap::new());
        let mut tex = DecodedTexture {
            name: "unknown_stage".to_string(),
            width: 512,
            height: 512,
            rgba: vec![0u8; 512 * 512 * 4],
        };
        unsquash_mappreview_texture(&mut tex, &aspects);
        // 16/9 * 512 ≈ 910
        assert_eq!(tex.height, 512);
        assert_eq!(tex.width, 910);
    }
}
