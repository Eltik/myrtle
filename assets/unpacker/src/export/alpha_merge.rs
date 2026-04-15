use std::collections::HashMap;
use std::path::Path;

use super::texture::{DecodedTexture, save_decoded_texture};

const ALPHA_SUFFIX: &str = "[alpha]";

/// Detect `[alpha]` pairs among decoded textures, merge them, and export all
/// results to `output_dir`. Returns the number of files written.
///
/// For each pair (`foo` + `foo[alpha]`):
///   - Saves merged RGBA as `foo.png`
///   - Saves the alpha texture as `foo[alpha].png`
///
/// Textures without a pair are saved as-is.
pub fn merge_and_export(textures: HashMap<String, DecodedTexture>, output_dir: &Path) -> usize {
    let mut exported = 0;

    // Partition into base and alpha sets
    let alpha_names: Vec<String> = textures
        .keys()
        .filter(|n| n.ends_with(ALPHA_SUFFIX))
        .cloned()
        .collect();

    let mut consumed: std::collections::HashSet<String> = std::collections::HashSet::new();

    for alpha_name in &alpha_names {
        let base_name = &alpha_name[..alpha_name.len() - ALPHA_SUFFIX.len()];

        let (Some(base), Some(alpha)) =
            (textures.get(base_name), textures.get(alpha_name.as_str()))
        else {
            continue;
        };

        // Merge and save combined RGBA
        let merged = combine_with_alpha(base, alpha);
        match save_decoded_texture(&merged, output_dir) {
            Ok(()) => exported += 1,
            Err(e) => eprintln!("  error saving merged {}: {e}", merged.name),
        }

        // Save the alpha texture as-is
        match save_decoded_texture(alpha, output_dir) {
            Ok(()) => exported += 1,
            Err(e) => eprintln!("  error saving alpha {}: {e}", alpha.name),
        }

        consumed.insert(base_name.to_string());
        consumed.insert(alpha_name.clone());
    }

    // Export remaining textures that weren't part of a pair
    for (name, tex) in &textures {
        if consumed.contains(name.as_str()) {
            continue;
        }
        match save_decoded_texture(tex, output_dir) {
            Ok(()) => exported += 1,
            Err(e) => eprintln!("  error saving {name}: {e}"),
        }
    }

    exported
}

/// Combine an RGB base texture with a separate alpha texture.
///
/// The alpha texture is converted to grayscale (R channel) and used as the
/// alpha channel of the output. If dimensions differ, the alpha texture is
/// resized to match the base using bilinear interpolation.
///
/// Pixels with alpha == 0 are set to `[0,0,0,0]` to prevent color bleeding.
pub fn combine_with_alpha(rgb: &DecodedTexture, alpha: &DecodedTexture) -> DecodedTexture {
    let (w, h) = (rgb.width, rgb.height);

    // Extract grayscale alpha values, resizing if needed
    let alpha_values = if alpha.width == w && alpha.height == h {
        alpha.rgba.chunks(4).map(|px| px[0]).collect::<Vec<u8>>()
    } else {
        let resized = resize_rgba(&alpha.rgba, alpha.width, alpha.height, w, h);
        resized.chunks(4).map(|px| px[0]).collect::<Vec<u8>>()
    };

    let pixel_count = (w * h) as usize;
    let mut result = vec![0u8; pixel_count * 4];

    for i in 0..pixel_count {
        let a = alpha_values.get(i).copied().unwrap_or(0);
        let off = i * 4;
        if a == 0 {
            // Color bleeding fix: fully transparent → black
            result[off] = 0;
            result[off + 1] = 0;
            result[off + 2] = 0;
            result[off + 3] = 0;
        } else {
            result[off] = rgb.rgba[off]; // R
            result[off + 1] = rgb.rgba[off + 1]; // G
            result[off + 2] = rgb.rgba[off + 2]; // B
            result[off + 3] = a;
        }
    }

    DecodedTexture {
        name: rgb.name.clone(),
        width: w,
        height: h,
        rgba: result,
    }
}

/// Resize RGBA pixel data using bilinear interpolation (Triangle filter).
fn resize_rgba(data: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Vec<u8> {
    let img = image::RgbaImage::from_raw(src_w, src_h, data.to_vec())
        .expect("RGBA buffer size mismatch in resize");
    let resized =
        image::imageops::resize(&img, dst_w, dst_h, image::imageops::FilterType::Triangle);
    resized.into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combine_synthetic_2x2() {
        // 2x2 red RGB texture (fully opaque)
        let rgb = DecodedTexture {
            name: "test_rgb".into(),
            width: 2,
            height: 2,
            rgba: vec![
                255, 0, 0, 255, 0, 255, 0, 255, // row 0: red, green
                0, 0, 255, 255, 255, 255, 0, 255, // row 1: blue, yellow
            ],
        };

        // 2x2 alpha: top-left transparent, rest opaque
        let alpha = DecodedTexture {
            name: "test_rgb[alpha]".into(),
            width: 2,
            height: 2,
            rgba: vec![
                0, 0, 0, 255, 200, 200, 200, 255, // row 0: alpha=0, alpha=200
                128, 128, 128, 255, 255, 255, 255, 255, // row 1: alpha=128, alpha=255
            ],
        };

        let merged = combine_with_alpha(&rgb, &alpha);
        assert_eq!(merged.width, 2);
        assert_eq!(merged.height, 2);
        assert_eq!(merged.name, "test_rgb");

        // Pixel 0: alpha=0 → all zeros (color bleeding fix)
        assert_eq!(&merged.rgba[0..4], &[0, 0, 0, 0]);

        // Pixel 1: alpha=200, RGB from green
        assert_eq!(&merged.rgba[4..8], &[0, 255, 0, 200]);

        // Pixel 2: alpha=128, RGB from blue
        assert_eq!(&merged.rgba[8..12], &[0, 0, 255, 128]);

        // Pixel 3: alpha=255, RGB from yellow
        assert_eq!(&merged.rgba[12..16], &[255, 255, 0, 255]);
    }

    #[test]
    fn combine_different_dimensions() {
        // 4x4 base (all white, opaque)
        let rgb = DecodedTexture {
            name: "base".into(),
            width: 4,
            height: 4,
            rgba: vec![255u8; 4 * 4 * 4],
        };

        // 2x2 alpha (all mid-gray)
        let alpha = DecodedTexture {
            name: "base[alpha]".into(),
            width: 2,
            height: 2,
            rgba: vec![
                128, 128, 128, 255, 128, 128, 128, 255, // row 0
                128, 128, 128, 255, 128, 128, 128, 255, // row 1
            ],
        };

        let merged = combine_with_alpha(&rgb, &alpha);
        assert_eq!(merged.width, 4);
        assert_eq!(merged.height, 4);

        // All pixels should have alpha around 128 (resized from 2x2 uniform)
        for chunk in merged.rgba.chunks(4) {
            assert!(chunk[3] > 100 && chunk[3] < 160, "alpha={}", chunk[3]);
        }
    }

    #[test]
    fn merge_and_export_pairs_and_orphans() {
        let dir = std::path::PathBuf::from(format!(
            "{}/test_output/alpha_merge_unit",
            env!("CARGO_MANIFEST_DIR")
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let mut textures = HashMap::new();

        // Paired: "foo" + "foo[alpha]"
        textures.insert(
            "foo".into(),
            DecodedTexture {
                name: "foo".into(),
                width: 1,
                height: 1,
                rgba: vec![255, 0, 0, 255],
            },
        );
        textures.insert(
            "foo[alpha]".into(),
            DecodedTexture {
                name: "foo[alpha]".into(),
                width: 1,
                height: 1,
                rgba: vec![128, 128, 128, 255],
            },
        );

        // Orphan base: "bar" (no alpha)
        textures.insert(
            "bar".into(),
            DecodedTexture {
                name: "bar".into(),
                width: 1,
                height: 1,
                rgba: vec![0, 255, 0, 255],
            },
        );

        let count = merge_and_export(textures, &dir);
        assert_eq!(count, 3); // foo (merged) + foo[alpha] + bar

        assert!(dir.join("foo.png").exists(), "merged foo.png should exist");
        assert!(
            dir.join("foo[alpha].png").exists(),
            "alpha texture should be preserved"
        );
        assert!(
            dir.join("bar.png").exists(),
            "orphan base should be exported"
        );
    }
}
