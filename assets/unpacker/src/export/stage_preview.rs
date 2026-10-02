//! Unsquash the squashed `stage_mappreview_h2_*` previews.
//!
//! Arknights ships stage map thumbnails as 512×512 ASTC squares, and the
//! Sprite's `m_Rect` is 512×512 too. The source renders are captures from the
//! 16:9 battle camera, and the game stretches the squares back out on screen,
//! so every preview is resized to 16:9 at its own height.
//!
//! Ruled out by inspection: deriving the aspect from the level's tile grid
//! (cols / rows of `MapData.Map` in the `level_<stage_id>` `TextAsset`). It
//! sized 1183 of 2470 previews off 16:9 at 512 tall (1157 narrower, down to
//! 626 px; 26 wider), and the narrow ones read squashed: on `act12side_01` at 640×512 the
//! deploy-point cubes are taller than wide and floor tiles under a tilted
//! camera are narrower than tall, both of which are only right at 910×512.
//! The grid measures the playfield, not the camera frame, which also shows
//! the scenery around it.

use std::path::Path;

use super::texture::DecodedTexture;

/// Width / height of the battle camera every preview was captured from.
const PREVIEW_ASPECT: f32 = 16.0 / 9.0;

const MAPPREVIEW_PATH_MARKER: &str = "stage_mappreview_h2_";

/// True if a bundle's relative path identifies it as a mappreview h2 bundle
/// whose textures need unsquashing.
#[must_use]
pub fn detect_mappreview_bundle(bundle_subdir: &Path) -> bool {
    bundle_subdir
        .to_string_lossy()
        .contains(MAPPREVIEW_PATH_MARKER)
}

/// Resize a mappreview texture in place from its squashed (typically square)
/// form to 16:9 at its own height.
pub fn unsquash_mappreview_texture(tex: &mut DecodedTexture) {
    let target_h = tex.height;
    let target_w = ((target_h as f32) * PREVIEW_ASPECT).round() as u32;
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
    fn unsquash_stretches_to_16_9() {
        let mut tex = DecodedTexture {
            name: "a001_01".to_string(),
            width: 512,
            height: 512,
            rgba: vec![0u8; 512 * 512 * 4],
        };
        unsquash_mappreview_texture(&mut tex);
        // 16/9 * 512 ≈ 910
        assert_eq!(tex.height, 512);
        assert_eq!(tex.width, 910);
        assert_eq!(tex.rgba.len(), (910 * 512 * 4) as usize);
    }
}
