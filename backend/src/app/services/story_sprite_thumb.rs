//! Square head-and-shoulders thumbnails of story sprites, made on first
//! request and kept on disk.
//!
//! A story sprite's body is a whole-figure plate, 1024 to 1384 px square and
//! up to ~0.7 MB (`avg_4000_jnight_1`: 689,753 bytes). The tier list pool shows
//! 1,443 of them as 52 to 72 px tiles, so serving the plate and cropping in CSS
//! cost the pool page ~0.7 MB a tile. This crops the head out of the plate once,
//! scales it to [`THUMB_PX`] and keeps the result under the server's
//! `derived/` tree (the same home as `derived/release-art`), where the asset
//! route serves it with the usual `ETag` and week-long `Cache-Control`.
//!
//! Where the crop sits: the hub's face box when the set has one (747 of 1,443
//! entries on EN); otherwise a head found from the alpha (the top of the
//! figure, centred on the opaque columns just under it). Measured against the
//! 250-set sample of face-carrying hubs on EN 2026-10-01, the alpha head lands
//! a median 0.026 of the plate from the hub's face centre (82% within 0.06),
//! where the old fixed `(0.5, 0.2)` guess landed a median 0.086 (20% within
//! 0.06). Its tail is worse (95th percentile 0.191 vs 0.140): a weapon or a
//! hat raised above the head pulls it up. A TRADE, shipped knowingly.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::SystemTime;

use image::{RgbaImage, imageops};
use tokio::sync::Semaphore;

use crate::app::error::ApiError;
pub use crate::app::services::story::sprite_thumbs::ThumbFormat;
use crate::app::services::story::sprite_thumbs::encode;

/// Output side in pixels: the largest tile is 72 px (the board at its widest
/// breakpoint), so 160 covers it at 2x DPR with room for the pool's `1fr`
/// cells to grow past their 52 px minimum.
pub const THUMB_PX: u32 = 160;

/// Bump to invalidate every cached thumbnail when the crop or encoding changes.
const VERSION: &str = "v1";

/// The cache directory under a server's assets root.
const CACHE_DIR: &str = "derived/story-sprite-thumbs";

/// The crop's side as a fraction of the plate's width: head and shoulders.
/// Fixed rather than scaled off the hub's face box, because the box does not
/// measure the same thing on every set: Kal'tsit's is 0.141 of her plate and
/// Nearl's (`avg_1014_nearl2`) 0.062 while their heads are drawn about the same
/// size (~160 and ~128 px), so a box-relative side framed Kal'tsit to the
/// waist and Nearl to the eyes. 0.29 is 3.2 x the median box (0.092).
const SIDE: f32 = 0.29;

/// The crop centre sits this far (in crop sides) below the face centre, so the
/// shoulders show and the hair is not cut.
const DROP: f32 = 0.12;

/// Below the figure's top, the face centre sits a median 0.119 of the plate's
/// height down (IQR 0.098 to 0.142) on the face-carrying sample.
const FACE_BELOW_TOP: f32 = 0.119;

/// How deep, as a fraction of the plate's height, the band under the figure's
/// top is whose opaque columns centre the crop horizontally.
const HEAD_BAND: f32 = 0.15;

/// An alpha at or above this counts as figure.
const OPAQUE: u8 = 64;

/// Opaque pixels a row needs to be the figure's top, so a stray speck above
/// the figure is not taken for it.
const TOP_ROW_MIN_OPAQUE: usize = 3;

/// The crop centre on a plate with no figure in its alpha, as fractions of the
/// plate: the fixed guess the alpha search replaced (see the module doc).
const EMPTY_PLATE_HEAD: (f32, f32) = (0.5, 0.2);

/// Thumbnails rendered at once. Each is ~10 to 50 ms of decode and resize; a
/// cold pool page asks for a few dozen together, and two at a time keeps them
/// off the CPU the rest of the server shares.
const MAX_CONCURRENT_RENDERS: usize = 2;

static RENDERS: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(MAX_CONCURRENT_RENDERS));

/// The crop square in plate pixels: `(left, top, side)`, kept inside the plate.
/// `face` is the hub's face centre as fractions of the plate, when it has one.
fn crop_rect(img: &RgbaImage, face: Option<(f32, f32)>) -> (u32, u32, u32) {
    let (w, h) = img.dimensions();
    let (wf, hf) = (w as f32, h as f32);
    let (cx, cy) = match face {
        Some((x, y)) => (x * wf, y * hf),
        None => alpha_head(img).unwrap_or((EMPTY_PLATE_HEAD.0 * wf, EMPTY_PLATE_HEAD.1 * hf)),
    };
    let side = (SIDE * wf).min(wf).min(hf).max(1.0);
    let cy = cy + DROP * side;
    let left = (cx - side / 2.0).clamp(0.0, wf - side);
    let top = (cy - side / 2.0).clamp(0.0, hf - side);
    (left.round() as u32, top.round() as u32, side.round() as u32)
}

/// The head as found from the alpha: `(x, y)` in plate pixels, or `None` for an
/// empty plate.
fn alpha_head(img: &RgbaImage) -> Option<(f32, f32)> {
    let (w, h) = img.dimensions();
    let opaque_in_row = |y: u32| (0..w).filter(|&x| img.get_pixel(x, y)[3] >= OPAQUE).count();
    let top = (0..h).find(|&y| opaque_in_row(y) >= TOP_ROW_MIN_OPAQUE)?;
    let band_end = (top + (HEAD_BAND * h as f32) as u32).min(h);
    let (mut sum, mut n) = (0u64, 0u64);
    for y in top..band_end {
        for x in 0..w {
            if img.get_pixel(x, y)[3] >= OPAQUE {
                sum += u64::from(x);
                n += 1;
            }
        }
    }
    let x = if n == 0 {
        w as f32 / 2.0
    } else {
        sum as f32 / n as f32
    };
    Some((x, top as f32 + FACE_BELOW_TOP * h as f32))
}

/// Crop `png` (a story sprite body) to its head and scale it to `px` square,
/// encoded as lossless WebP or PNG. Both carry the same pixels.
///
/// The resize runs on premultiplied colour: a sprite's fully transparent
/// pixels carry arbitrary RGB, and filtering them straight bleeds a dark (or
/// coloured) fringe into the figure's edge.
pub fn render(
    png: &[u8],
    face: Option<(f32, f32)>,
    px: u32,
    format: ThumbFormat,
) -> anyhow::Result<Vec<u8>> {
    let img = image::load_from_memory_with_format(png, image::ImageFormat::Png)?.to_rgba8();
    let (left, top, side) = crop_rect(&img, face);
    let mut crop = imageops::crop_imm(&img, left, top, side, side).to_image();
    premultiply(&mut crop);
    let mut out = imageops::resize(&crop, px, px, imageops::FilterType::Lanczos3);
    unpremultiply(&mut out);
    encode(&out, format)
}

fn premultiply(img: &mut RgbaImage) {
    for p in img.pixels_mut() {
        let a = u16::from(p[3]);
        for c in 0..3 {
            p[c] = ((u16::from(p[c]) * a + 127) / 255) as u8;
        }
    }
}

fn unpremultiply(img: &mut RgbaImage) {
    for p in img.pixels_mut() {
        let a = u16::from(p[3]);
        if a == 0 {
            p[0] = 0;
            p[1] = 0;
            p[2] = 0;
            continue;
        }
        for c in 0..3 {
            // Lanczos overshoots; a premultiplied channel above its alpha is ringing.
            let v = u16::from(p[c]).min(a);
            p[c] = ((v * 255 + a / 2) / a) as u8;
        }
    }
}

/// The cached thumbnail's path relative to the assets root: what the asset
/// route serves.
pub fn cache_rel_path(id: &str, format: ThumbFormat) -> String {
    format!("{CACHE_DIR}/{VERSION}/{id}.{}", format.ext())
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// Make sure `assets_dir`'s thumbnail of `id` exists and is no older than its
/// source body, rendering it when it is missing or stale. Returns the path to
/// serve, relative to `assets_dir`.
///
/// `body_rel` is the body PNG relative to `assets_dir` (`/textures/avg/...`).
/// Each format is its own cached file, rendered from the body on its first
/// request.
/// The file is written to a temporary name and renamed into place, so a
/// reader never sees a half-written thumbnail and two racing renders of the
/// same id both leave a whole one.
pub async fn ensure(
    assets_dir: &str,
    id: &str,
    body_rel: &str,
    face: Option<(f32, f32)>,
    format: ThumbFormat,
) -> Result<String, ApiError> {
    if id.is_empty() || id.contains(['/', '\\', '\0']) || id.starts_with('.') {
        return Err(ApiError::NotFound);
    }
    let root = PathBuf::from(assets_dir);
    let source = root.join(body_rel.trim_start_matches('/'));
    let rel = cache_rel_path(id, format);
    let target = root.join(&rel);

    let fresh = |target: &Path| match (modified(target), modified(&source)) {
        (Some(t), Some(s)) => t >= s,
        _ => false,
    };
    if fresh(&target) {
        return Ok(rel);
    }

    let _permit = RENDERS
        .acquire()
        .await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("thumbnail render pool closed")))?;
    // Another request may have rendered it while this one waited.
    if fresh(&target) {
        return Ok(rel);
    }
    // Unique per process and request thread, so racing renders never share
    // a temporary file.
    let tmp_name = format!(
        "{id}.{}.{}.{:?}.tmp",
        format.ext(),
        std::process::id(),
        std::thread::current().id()
    );
    crate::app::cpu::offload("story_sprite_thumb", move || -> anyhow::Result<()> {
        let png = std::fs::read(&source)?;
        let bytes = render(&png, face, THUMB_PX, format)?;
        write_atomically(&target, &tmp_name, &bytes)
    })
    .await?
    .map_err(|e| {
        tracing::warn!(id, error = %e, "story sprite thumbnail failed");
        ApiError::NotFound
    })?;
    Ok(rel)
}

/// Write `bytes` to `target` through `tmp_name` in the same directory, so the
/// rename is atomic. `tmp_name` is cut to `[A-Za-z0-9._]`.
fn write_atomically(target: &Path, tmp_name: &str, bytes: &[u8]) -> anyhow::Result<()> {
    let dir = target
        .parent()
        .ok_or_else(|| anyhow::anyhow!("thumbnail path has no parent"))?;
    std::fs::create_dir_all(dir)?;
    let tmp = dir.join(tmp_name.replace(
        |c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '_',
        "",
    ));
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, target)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use image::ImageEncoder;

    use super::*;

    fn plate(w: u32, h: u32, figure: impl Fn(u32, u32) -> bool) -> RgbaImage {
        RgbaImage::from_fn(w, h, |x, y| {
            if figure(x, y) {
                image::Rgba([200, 100, 50, 255])
            } else {
                // Transparent pixels with loud RGB, the case premultiplying guards.
                image::Rgba([0, 255, 0, 0])
            }
        })
    }

    #[test]
    fn a_hub_face_centres_the_crop_inside_the_plate() {
        let img = plate(1000, 1000, |_, _| true);
        let (left, top, side) = crop_rect(&img, Some((0.6, 0.15)));
        assert_eq!(side, 290);
        assert_eq!(left, 455);
        // 150 + 0.12 * 290 - 145 = 39.8
        assert_eq!(top, 40);
        // A face at the corner is pulled back inside.
        let (l, t, s) = crop_rect(&img, Some((1.0, 1.0)));
        assert_eq!((l + s, t + s), (1000, 1000));
    }

    #[test]
    fn without_a_hub_face_the_crop_finds_the_head_in_the_alpha() {
        // A figure whose top is at y = 100, its head centred on x = 300.
        let img = plate(1000, 1000, |x, y| y >= 100 && (250..350).contains(&x));
        let (left, top, side) = crop_rect(&img, None);
        assert_eq!(side, 290);
        assert_eq!(left, 155);
        // 100 + 119 + 0.12 * 290 - 145 = 108.8
        assert_eq!(top, 109);
    }

    #[test]
    fn the_resize_does_not_bleed_transparent_colour_into_the_figure() {
        let img = plate(400, 400, |x, _| x < 200);
        let mut png = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        let webp = render(&png, Some((0.5, 0.5)), 64, ThumbFormat::Webp).unwrap();
        let out = image::load_from_memory(&webp).unwrap().to_rgba8();
        assert_eq!(out.dimensions(), (64, 64));
        for p in out.pixels().filter(|p| p[3] > 0) {
            assert!(p[1] < 140, "green bled into the edge: {p:?}");
        }
    }

    #[test]
    fn png_and_webp_carry_the_same_pixels() {
        let img = plate(400, 400, |x, y| x < 200 && y > 50);
        let mut png = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        let webp = render(&png, None, 64, ThumbFormat::Webp).unwrap();
        let out_png = render(&png, None, 64, ThumbFormat::Png).unwrap();
        assert_eq!(
            image::guess_format(&out_png).unwrap(),
            image::ImageFormat::Png
        );
        assert_eq!(
            image::load_from_memory(&webp).unwrap().to_rgba8(),
            image::load_from_memory(&out_png).unwrap().to_rgba8()
        );
        assert_ne!(
            cache_rel_path("avg_npc_1", ThumbFormat::Webp),
            cache_rel_path("avg_npc_1", ThumbFormat::Png)
        );
    }

    /// Renders four real sprites from the local EN extract through [`ensure`]
    /// (cold: decode, crop, resize, encode, write; warm: the cached file is
    /// fresh and only its path comes back) and prints sizes and timings. The
    /// sources are copied into a temporary root, so the real `derived/` tree is
    /// never written. Writes PNG previews to `STORY_THUMB_OUT` when set.
    #[tokio::test]
    #[ignore = "reads the local EN extract"]
    async fn real_sprites_render_small_thumbnails() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/output/en");
        let tmp_root = std::env::temp_dir().join(format!("story-thumb-{}", std::process::id()));
        let samples: [(&str, &str); 4] = [
            ("avg_4000_jnight_1", "avg_4000_jnight_1"),
            ("avg_003_kalts_1", "avg_003_kalts_1$1"),
            ("avg_1014_nearl2_1", "avg_1014_nearl2_1$1"),
            ("avg_npc_005", "avg_npc_005"),
        ];
        let out_dir = std::env::var("STORY_THUMB_OUT").ok().map(PathBuf::from);
        for (folder, body) in samples {
            let dir = root.join("textures/avg/characters").join(folder);
            let body_path = dir.join(format!("{body}.png"));
            let Ok(png) = std::fs::read(&body_path) else {
                eprintln!("skip {folder}: no extract");
                continue;
            };
            let face = crate::core::gamedata::story_sprites::face_center(
                &dir.join("hub.json"),
                &body_path,
            );
            let body_rel = format!("/textures/avg/characters/{folder}/{body}.png");
            let copy = tmp_root.join(body_rel.trim_start_matches('/'));
            std::fs::create_dir_all(copy.parent().unwrap()).unwrap();
            std::fs::write(&copy, &png).unwrap();
            let assets_dir = tmp_root.to_string_lossy().into_owned();

            let t0 = std::time::Instant::now();
            let rel = ensure(&assets_dir, folder, &body_rel, face, ThumbFormat::Webp)
                .await
                .unwrap();
            let cold = t0.elapsed();
            let t1 = std::time::Instant::now();
            let rel_again = ensure(&assets_dir, folder, &body_rel, face, ThumbFormat::Webp)
                .await
                .unwrap();
            let warm = t1.elapsed();
            assert_eq!(rel, rel_again);

            let webp = std::fs::read(tmp_root.join(&rel)).unwrap();
            let decoded = image::load_from_memory(&webp).unwrap().to_rgba8();
            assert_eq!(decoded.dimensions(), (THUMB_PX, THUMB_PX));
            let mut png_out = Vec::new();
            image::codecs::png::PngEncoder::new_with_quality(
                &mut png_out,
                image::codecs::png::CompressionType::Best,
                image::codecs::png::FilterType::Adaptive,
            )
            .write_image(
                decoded.as_raw(),
                THUMB_PX,
                THUMB_PX,
                image::ExtendedColorType::Rgba8,
            )
            .unwrap();
            eprintln!(
                "{folder}: source {} B, face {face:?}, webp {} B (png of the same pixels {} B), cold {cold:?}, warm {warm:?}",
                png.len(),
                webp.len(),
                png_out.len(),
            );
            if let Some(dir) = &out_dir {
                std::fs::create_dir_all(dir).unwrap();
                std::fs::write(dir.join(format!("{folder}.png")), &png_out).unwrap();
            }
            assert!(webp.len() < 80_000, "{folder}: {} B", webp.len());
        }
        let _ = std::fs::remove_dir_all(&tmp_root);
    }
}
