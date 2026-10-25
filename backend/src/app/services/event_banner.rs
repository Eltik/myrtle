//! Wide banners for events the client no longer ships a home-entry banner
//! for, made on first request and kept on disk.
//!
//! The home screen's banner (`[uc]homeentry/{act_id}.png`, 284x168) ships only
//! while an event runs; the release-art archive keeps the ones it saw. Older
//! events (102 of 141 rankable ones on EN 2026-10-25) had only the Archives
//! cover, a portrait collage with a film-reel badge, which sat in a wide tile
//! as a thumbnail inside a frame. The Story Collection carries every story
//! event's own art instead: a 1660x936 key visual (`storybg_*`, fading to
//! black on the right) and a transparent title logotype (`title_*`). Laid
//! together the way that screen lays them, title in the faded right side,
//! they make a banner in the home banner's shape.
//!
//! Events outside the Story Collection (Contingency Contract, Trials for
//! Navigator, the collaborations) have no key visual and keep the cover.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::SystemTime;

use image::{ImageEncoder, RgbaImage, imageops};
use tokio::sync::Semaphore;

use crate::app::error::ApiError;
use crate::app::services::story_sprite_thumb::{premultiply, unpremultiply, write_atomically};
use crate::core::gamedata::assets::{AssetIndex, AssetKind};
use crate::core::gamedata::types::GameData;

/// Output size: twice the home banner's 284x168, so a generated banner and a
/// real one sit side by side in a tile at the same sharpness.
pub const WIDTH: u32 = 568;
pub const HEIGHT: u32 = 336;

/// The title's box, as fractions of the banner: it fits inside this and keeps
/// its own aspect.
const TITLE_MAX_W: f32 = 0.5;
const TITLE_MAX_H: f32 = 0.45;

/// Gap between the title and the banner's right edge, as a fraction of the
/// banner's width.
const TITLE_RIGHT_MARGIN: f32 = 0.042;

const JPEG_QUALITY: u8 = 88;

/// Bump to invalidate every cached banner when the layout or encoding changes.
const VERSION: &str = "v1";

/// The cache directory under a server's assets root.
const CACHE_DIR: &str = "derived/event-banners";

/// Banners rendered at once: each is ~30 ms of decode, resize and encode, and
/// a cold tier-list pool asks for dozens together.
const MAX_CONCURRENT_RENDERS: usize = 2;

static RENDERS: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(MAX_CONCURRENT_RENDERS));

/// The key visual and title an event's banner is made from, as served paths
/// under the art tree. A rerun shares its original's art.
#[must_use]
pub fn sources(gd: &GameData, assets: &AssetIndex, act_id: &str) -> Option<(String, String)> {
    let set = gd
        .storyline_story_sets
        .values()
        .find(|s| s.covers_activity(act_id))?;
    let background = assets.path(AssetKind::StorylineArt, set.background_id()?)?;
    let title = assets.path(AssetKind::StorylineArt, set.title_image_id.as_deref()?)?;
    Some((background.to_owned(), title.to_owned()))
}

/// Lay `title_png` over `background_png` as a banner, JPEG-encoded.
///
/// The key visual covers the banner's height and is cut from its left edge,
/// where the art is; the title is trimmed to its opaque box and centred
/// vertically against the right edge, over the faded side.
pub fn render(background_png: &[u8], title_png: &[u8]) -> anyhow::Result<Vec<u8>> {
    let bg =
        image::load_from_memory_with_format(background_png, image::ImageFormat::Png)?.to_rgba8();
    let scale = (WIDTH as f32 / bg.width() as f32).max(HEIGHT as f32 / bg.height() as f32);
    let (sw, sh) = (
        ((bg.width() as f32 * scale).round() as u32).max(WIDTH),
        ((bg.height() as f32 * scale).round() as u32).max(HEIGHT),
    );
    let bg = imageops::resize(&bg, sw, sh, imageops::FilterType::Lanczos3);
    let mut out = RgbaImage::from_pixel(WIDTH, HEIGHT, image::Rgba([0, 0, 0, 255]));
    let top = (sh - HEIGHT) / 2;
    imageops::overlay(
        &mut out,
        &imageops::crop_imm(&bg, 0, top, WIDTH, HEIGHT).to_image(),
        0,
        0,
    );

    let title = image::load_from_memory_with_format(title_png, image::ImageFormat::Png)?.to_rgba8();
    if let Some((x, y, w, h)) = opaque_box(&title) {
        let mut title = imageops::crop_imm(&title, x, y, w, h).to_image();
        let fit =
            (WIDTH as f32 * TITLE_MAX_W / w as f32).min(HEIGHT as f32 * TITLE_MAX_H / h as f32);
        let (tw, th) = (
            ((w as f32 * fit).round() as u32).max(1),
            ((h as f32 * fit).round() as u32).max(1),
        );
        premultiply(&mut title);
        let mut title = imageops::resize(&title, tw, th, imageops::FilterType::Lanczos3);
        unpremultiply(&mut title);
        let left = WIDTH - tw - (WIDTH as f32 * TITLE_RIGHT_MARGIN).round() as u32;
        imageops::overlay(
            &mut out,
            &title,
            i64::from(left),
            i64::from((HEIGHT - th) / 2),
        );
    }

    let rgb = image::DynamicImage::ImageRgba8(out).to_rgb8();
    let mut bytes = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, JPEG_QUALITY).write_image(
        rgb.as_raw(),
        WIDTH,
        HEIGHT,
        image::ExtendedColorType::Rgb8,
    )?;
    Ok(bytes)
}

/// The smallest box holding every pixel with any alpha: `(x, y, w, h)`.
fn opaque_box(img: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for (x, y, p) in img.enumerate_pixels() {
        if p[3] > 0 {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
    }
    (x0 <= x1).then(|| (x0, y0, x1 - x0 + 1, y1 - y0 + 1))
}

/// The cached banner's path relative to the assets root: what the asset route
/// serves.
#[must_use]
pub fn cache_rel_path(act_id: &str) -> String {
    format!("{CACHE_DIR}/{VERSION}/{act_id}.jpg")
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// Render `act_id`'s banner if it is missing or older than either source;
/// returns the path to serve, relative to `art_dir`. The sources are served
/// paths under `art_dir` (from [`sources`]).
///
/// An empty source (an evicted stub, `S3_EVICT_LOCAL`) fails to decode and
/// answers `NotFound`, so the caller falls back to the Archives cover.
pub async fn ensure(
    art_dir: &str,
    act_id: &str,
    background_rel: &str,
    title_rel: &str,
) -> Result<String, ApiError> {
    if act_id.is_empty() || act_id.contains(['/', '\\', '\0']) || act_id.starts_with('.') {
        return Err(ApiError::NotFound);
    }
    let root = PathBuf::from(art_dir);
    let background = root.join(background_rel.trim_start_matches('/'));
    let title = root.join(title_rel.trim_start_matches('/'));
    let rel = cache_rel_path(act_id);
    let target = root.join(&rel);

    let fresh = |target: &Path| match (modified(target), modified(&background), modified(&title)) {
        (Some(t), Some(b), Some(n)) => t >= b && t >= n,
        _ => false,
    };
    if fresh(&target) {
        return Ok(rel);
    }

    let _permit = RENDERS
        .acquire()
        .await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("event banner render pool closed")))?;
    // Another request may have rendered it while this one waited.
    if fresh(&target) {
        return Ok(rel);
    }
    let tmp_name = crate::utils::fs::unique_tmp_name(&format!("{act_id}.jpg"));
    crate::app::cpu::offload("event_banner", move || -> anyhow::Result<()> {
        let bytes = render(&std::fs::read(&background)?, &std::fs::read(&title)?)?;
        write_atomically(&target, &tmp_name, &bytes)
    })
    .await?
    .map_err(|e| {
        tracing::warn!(act_id, error = %e, "event banner failed");
        ApiError::NotFound
    })?;
    Ok(rel)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(img: &RgbaImage) -> Vec<u8> {
        let mut bytes = Vec::new();
        image::codecs::png::PngEncoder::new(&mut bytes)
            .write_image(
                img.as_raw(),
                img.width(),
                img.height(),
                image::ExtendedColorType::Rgba8,
            )
            .unwrap();
        bytes
    }

    #[test]
    fn the_banner_takes_the_left_of_the_key_visual_and_lays_the_title_right() {
        // A key visual red on its left half and blue on its right, and a white
        // title with a transparent margin.
        let bg = RgbaImage::from_fn(1660, 936, |x, _| {
            if x < 830 {
                image::Rgba([255, 0, 0, 255])
            } else {
                image::Rgba([0, 0, 255, 255])
            }
        });
        let title = RgbaImage::from_fn(516, 260, |x, y| {
            if (100..416).contains(&x) && (80..180).contains(&y) {
                image::Rgba([255, 255, 255, 255])
            } else {
                image::Rgba([0, 0, 0, 0])
            }
        });
        let jpeg = render(&png(&bg), &png(&title)).unwrap();
        let out = image::load_from_memory(&jpeg).unwrap().to_rgb8();
        assert_eq!(out.dimensions(), (WIDTH, HEIGHT));
        // Left edge: the key visual's left, untouched by the title.
        let left = out.get_pixel(10, HEIGHT / 2);
        assert!(left[0] > 200 && left[2] < 60, "{left:?}");
        // Right of centre, mid-height: the title.
        let title_px = out.get_pixel(WIDTH - 60, HEIGHT / 2);
        assert!(title_px.0.iter().all(|&c| c > 200), "{title_px:?}");
        // The title is trimmed to its opaque box, so its 316 px of ink fill
        // the width budget rather than its 516 px canvas.
        let ink_left = (0..WIDTH)
            .find(|&x| out.get_pixel(x, HEIGHT / 2).0.iter().all(|&c| c > 200))
            .unwrap();
        assert!(ink_left < WIDTH / 2 + 20, "title starts at {ink_left}");
    }

    #[test]
    fn a_fully_transparent_title_leaves_the_key_visual_alone() {
        let bg = RgbaImage::from_pixel(1660, 936, image::Rgba([0, 255, 0, 255]));
        let title = RgbaImage::new(516, 260);
        let out = image::load_from_memory(&render(&png(&bg), &png(&title)).unwrap())
            .unwrap()
            .to_rgb8();
        let px = out.get_pixel(WIDTH - 60, HEIGHT / 2);
        assert!(px[1] > 200 && px[0] < 60, "{px:?}");
    }

    #[test]
    fn an_empty_source_does_not_render() {
        assert!(render(&[], &[]).is_err());
    }

    /// Every story event's banner from the real EN extract: each one resolves
    /// and renders. Writes the JPEGs to `EVENT_BANNER_OUT` when set.
    #[test]
    #[ignore = "reads the local EN extract"]
    fn real_story_events_render_banners() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/output/en");
        let (gd, assets) =
            crate::core::gamedata::init_game_data(&root.join("gamedata/excel"), &root)
                .expect("EN game data loads");
        let out_dir = std::env::var("EVENT_BANNER_OUT").ok().map(PathBuf::from);
        let mut rendered = 0;
        for id in gd.activities.keys() {
            let Some((bg, title)) = sources(&gd, &assets, id) else {
                continue;
            };
            let read = |rel: &str| std::fs::read(root.join(rel.trim_start_matches('/'))).unwrap();
            let jpeg = render(&read(&bg), &read(&title)).unwrap_or_else(|e| panic!("{id}: {e}"));
            if let Some(dir) = &out_dir {
                std::fs::create_dir_all(dir).unwrap();
                std::fs::write(dir.join(format!("{id}.jpg")), &jpeg).unwrap();
            }
            rendered += 1;
        }
        println!("rendered {rendered} event banners");
        assert!(rendered > 90, "only {rendered} events found Story Collection art");
    }
}
