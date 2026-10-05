//! One EXPRESSION as a small image: a gallery folder's `#N$M` composed (body
//! plate plus face patch at `facePos`, exactly as `Body` stacks them on the
//! page) and scaled to [`THUMB_H`] px high, made on first request and kept on
//! disk under `derived/sprite-thumbs/<base>/<variant>.<ext>`.
//!
//! Why it exists: the gallery's first page drew 60 cards from 79 full-size
//! body and face PNGs, 25,715,702 bytes (median 285,851). The tier list's
//! `/story-sprite-thumb/{id}` cannot stand in: it crops one default body per
//! CHARACTER, not per folder and expression.
//!
//! Only expressions the gallery LISTS are rendered, looked up in the cached
//! census by key, so the route cannot be driven to compose arbitrary files.
//! The cache file is fresh while it is newer than both sources; a re-extract
//! rewrites every source and so retires every thumb, which then re-renders
//! lazily. `derived/` is never in a subtree the extract writes, so the orphan
//! sweep (`assets/orphans.mjs`) never reaches it; a cache hit still refreshes
//! the file's mtime once it is a day old, the rule `pool_write_png` follows,
//! so a future sweep that did reach it would keep what is served. Not on
//! every hit: the asset route's `ETag` is size plus mtime, and touching per
//! request would turn every 304 into a 200.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::{Duration, SystemTime};

use image::{ImageEncoder, RgbaImage, imageops};
use tokio::sync::Semaphore;

use super::dto::StorySpriteVariant;
use crate::app::error::ApiError;

/// Output height in px; the width follows the plate's aspect.
pub const THUMB_H: u32 = 320;

/// The cache directory under a server's assets root.
const CACHE_DIR: &str = "derived/sprite-thumbs";

/// Bodies composed at once; each is a 1024 to 1384 px decode plus a resize.
const MAX_CONCURRENT_RENDERS: usize = 2;

/// A cached file older than this is touched on a hit, see the module doc.
const TOUCH_AFTER: Duration = Duration::from_hours(24);

static RENDERS: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(MAX_CONCURRENT_RENDERS));

/// The encodings the route can store; see `docs/story-reader.md` for the
/// measurement that picked [`ThumbFormat::Webp`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbFormat {
    /// Lossless WebP, the only WebP encoder the `image` crate ships.
    Webp,
    Png,
}

impl ThumbFormat {
    #[must_use]
    pub const fn ext(self) -> &'static str {
        match self {
            Self::Webp => "webp",
            Self::Png => "png",
        }
    }
}

/// The format the route serves.
pub const FORMAT: ThumbFormat = ThumbFormat::Webp;

/// A variant key as a file name: `#2$1` -> `f2b1`, `@smile` -> `asmile`.
/// Anything else outside `[A-Za-z0-9]` becomes `_`.
#[must_use]
pub fn variant_file_stem(key: &str) -> String {
    key.chars()
        .map(|c| match c {
            '#' => 'f',
            '$' => 'b',
            '@' => 'a',
            c if c.is_ascii_alphanumeric() => c,
            _ => '_',
        })
        .collect()
}

/// The cached thumb's path relative to the assets root. `base` is the folder
/// as the census spells it, lowercased so two spellings share one file.
#[must_use]
pub fn cache_rel_path(base: &str, key: &str, format: ThumbFormat) -> String {
    format!(
        "{CACHE_DIR}/{}/{}.{}",
        base.to_ascii_lowercase(),
        variant_file_stem(key),
        format.ext()
    )
}

/// A wire URL (`/textures/avg/characters/x/avg_x%231.png`) as a path on disk.
fn source_path(root: &Path, url: &str) -> PathBuf {
    root.join(url.trim_start_matches('/').replace("%23", "#"))
}

/// Compose `variant` from its body and face PNGs and scale it to `height`.
///
/// The face patch lands at `facePos` in BODY TEXTURE pixels (`bodySize`),
/// rescaled to the decoded body when the two differ, and is alpha-composited
/// over the body. The resize runs on premultiplied colour so the transparent
/// pixels' arbitrary RGB does not fringe the figure.
pub fn compose(
    body_png: &[u8],
    face_png: Option<&[u8]>,
    variant: &StorySpriteVariant,
    height: u32,
) -> anyhow::Result<RgbaImage> {
    let mut body =
        image::load_from_memory_with_format(body_png, image::ImageFormat::Png)?.to_rgba8();
    let (bw, bh) = body.dimensions();
    if let (Some(face_png), Some(pos)) = (face_png, variant.sprite.face_pos) {
        let face =
            image::load_from_memory_with_format(face_png, image::ImageFormat::Png)?.to_rgba8();
        let (tw, th) = variant
            .sprite
            .body_size
            .map_or((bw as f32, bh as f32), |s| (s.w, s.h));
        let (sx, sy) = (bw as f32 / tw, bh as f32 / th);
        let w = (pos.w * sx).round().max(1.0) as u32;
        let h = (pos.h * sy).round().max(1.0) as u32;
        let mut patch = face;
        if patch.dimensions() != (w, h) {
            premultiply(&mut patch);
            patch = imageops::resize(&patch, w, h, imageops::FilterType::Lanczos3);
            unpremultiply(&mut patch);
        }
        imageops::overlay(
            &mut body,
            &patch,
            i64::from((pos.x * sx).round() as i32),
            i64::from((pos.y * sy).round() as i32),
        );
    }
    let width = ((u64::from(bw) * u64::from(height) + u64::from(bh) / 2) / u64::from(bh.max(1)))
        .max(1) as u32;
    premultiply(&mut body);
    let mut out = imageops::resize(&body, width, height, imageops::FilterType::Lanczos3);
    unpremultiply(&mut out);
    Ok(out)
}

/// Encode a composed thumb.
pub fn encode(img: &RgbaImage, format: ThumbFormat) -> anyhow::Result<Vec<u8>> {
    let (w, h) = img.dimensions();
    let mut bytes = Vec::new();
    match format {
        ThumbFormat::Webp => image::codecs::webp::WebPEncoder::new_lossless(&mut bytes)
            .write_image(img.as_raw(), w, h, image::ExtendedColorType::Rgba8)?,
        ThumbFormat::Png => image::codecs::png::PngEncoder::new_with_quality(
            &mut bytes,
            image::codecs::png::CompressionType::Best,
            image::codecs::png::FilterType::Adaptive,
        )
        .write_image(img.as_raw(), w, h, image::ExtendedColorType::Rgba8)?,
    }
    Ok(bytes)
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

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// Render `variant` of `base` into the cache when it is missing or older than
/// a source, and return the path to serve relative to `assets_dir`.
pub async fn ensure(
    assets_dir: &str,
    base: &str,
    variant: &StorySpriteVariant,
) -> Result<String, ApiError> {
    let root = PathBuf::from(assets_dir);
    let body = source_path(&root, &variant.sprite.body_url);
    let face = variant
        .sprite
        .face_url
        .as_deref()
        .map(|u| source_path(&root, u));
    let rel = cache_rel_path(base, &variant.key, FORMAT);
    let target = root.join(&rel);

    let fresh = |target: &Path| {
        let Some(t) = modified(target) else {
            return false;
        };
        let newer = |s: &Path| modified(s).is_some_and(|m| t >= m);
        newer(&body) && face.as_deref().is_none_or(newer)
    };
    if fresh(&target) {
        touch_if_old(&target);
        return Ok(rel);
    }

    let _permit = RENDERS
        .acquire()
        .await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("sprite thumb render pool closed")))?;
    if fresh(&target) {
        return Ok(rel);
    }
    let tmp_name = format!(
        "{}.{}.{:?}.tmp",
        variant_file_stem(&variant.key),
        std::process::id(),
        std::thread::current().id()
    );
    let v = variant.clone();
    crate::app::cpu::run(
        "story_sprite_variant_thumb",
        move || -> anyhow::Result<()> {
            let body_png = std::fs::read(&body)?;
            let face_png = face.as_deref().map(std::fs::read).transpose()?;
            let img = compose(&body_png, face_png.as_deref(), &v, THUMB_H)?;
            write_atomically(&target, &tmp_name, &encode(&img, FORMAT)?)
        },
    )
    .await?
    .map_err(|e| {
        tracing::warn!(base, key = %variant.key, error = %e, "sprite variant thumb failed");
        ApiError::NotFound
    })?;
    Ok(rel)
}

/// Refresh a served file's mtime once it is [`TOUCH_AFTER`] old.
fn touch_if_old(path: &Path) {
    let old = modified(path)
        .and_then(|m| SystemTime::now().duration_since(m).ok())
        .is_some_and(|age| age > TOUCH_AFTER);
    if old {
        let _ = std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .and_then(|f| f.set_modified(SystemTime::now()));
    }
}

/// Write through a temporary name in the same directory, so the rename is
/// atomic and a reader never sees half a file.
fn write_atomically(target: &Path, tmp_name: &str, bytes: &[u8]) -> anyhow::Result<()> {
    let dir = target
        .parent()
        .ok_or_else(|| anyhow::anyhow!("thumb path has no parent"))?;
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
    use super::*;
    use crate::core::story::assets::BodySize;
    use crate::core::story::{CharacterSprite, FacePos};

    fn png(img: &RgbaImage) -> Vec<u8> {
        let mut out = Vec::new();
        image::DynamicImage::ImageRgba8(img.clone())
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    }

    fn variant(face: Option<FacePos>, body: (f32, f32)) -> StorySpriteVariant {
        StorySpriteVariant {
            key: "#2$1".to_owned(),
            sprite: CharacterSprite {
                body_url: "/b.png".to_owned(),
                face_url: face.map(|_| "/f.png".to_owned()),
                face_pos: face,
                body_size: Some(BodySize {
                    w: body.0,
                    h: body.1,
                }),
                plate: None,
            },
            whole_body: face.is_none(),
            uses: 0,
        }
    }

    #[test]
    fn keys_become_file_names() {
        assert_eq!(variant_file_stem("#2$1"), "f2b1");
        assert_eq!(variant_file_stem("@smile"), "asmile");
        assert_eq!(variant_file_stem("#1/../x"), "f1____x");
        assert_eq!(
            cache_rel_path("AVG_NPC_043_1", "#2$1", ThumbFormat::Webp),
            "derived/sprite-thumbs/avg_npc_043_1/f2b1.webp"
        );
    }

    #[test]
    fn the_face_lands_at_face_pos_in_texture_pixels() {
        // A 200 px transparent body declared as a 400 px texture: the patch at
        // (100, 40, 50, 50) texture px lands at (50, 20, 25, 25) decoded px.
        let body = RgbaImage::from_pixel(200, 200, image::Rgba([0, 0, 255, 255]));
        let face = RgbaImage::from_pixel(64, 64, image::Rgba([255, 0, 0, 255]));
        let v = variant(
            Some(FacePos {
                x: 100.0,
                y: 40.0,
                w: 50.0,
                h: 50.0,
            }),
            (400.0, 400.0),
        );
        let out = compose(&png(&body), Some(&png(&face)), &v, 200).unwrap();
        assert_eq!(out.dimensions(), (200, 200));
        assert_eq!(out.get_pixel(60, 30)[0], 255, "inside the patch");
        assert_eq!(out.get_pixel(40, 30)[2], 255, "left of the patch is body");
        assert_eq!(out.get_pixel(60, 50)[2], 255, "below the patch is body");
    }

    #[test]
    fn the_height_is_fixed_and_the_width_follows_the_plate() {
        let body = RgbaImage::from_pixel(1280, 1024, image::Rgba([10, 20, 30, 255]));
        let out = compose(&png(&body), None, &variant(None, (1280.0, 1024.0)), THUMB_H).unwrap();
        assert_eq!(out.dimensions(), (400, 320));
    }
}
