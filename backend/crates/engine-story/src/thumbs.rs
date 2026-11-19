use image::{ImageEncoder, ImageError, RgbaImage, imageops};

use crate::assets::CharacterSprite;

pub const THUMB_H: u32 = 320;
pub const HEAD_PX: u32 = 160;

const VARIANT_DIR: &str = "derived/sprite-thumbs";
const HEAD_DIR: &str = "derived/story-sprite-thumbs/v1";

const SIDE: f32 = 0.29;
const DROP: f32 = 0.12;
const FACE_BELOW_TOP: f32 = 0.119;
const HEAD_BAND: f32 = 0.15;
const OPAQUE: u8 = 64;
const TOP_ROW_MIN_OPAQUE: usize = 3;
const EMPTY_PLATE_HEAD: (f32, f32) = (0.5, 0.2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbFormat {
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

pub const FORMAT: ThumbFormat = ThumbFormat::Webp;

/// `#2$1` -> `f2b1`, `@smile` -> `asmile`, etc.
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

#[must_use]
pub fn variant_rel_path(base: &str, key: &str, format: ThumbFormat) -> String {
    format!(
        "{VARIANT_DIR}/{}/{}.{}",
        base.to_ascii_lowercase(),
        variant_file_stem(key),
        format.ext()
    )
}

#[must_use]
pub fn head_rel_path(id: &str, format: ThumbFormat) -> String {
    format!("{HEAD_DIR}/{id}.{}", format.ext())
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
            p.0[..3].fill(0);
            continue;
        }
        for c in 0..3 {
            let v = u16::from(p[c]).min(a);
            p[c] = ((v * 255 + a / 2) / a) as u8;
        }
    }
}

fn resize(mut img: RgbaImage, w: u32, h: u32) -> RgbaImage {
    premultiply(&mut img);
    let mut out = imageops::resize(&img, w, h, imageops::FilterType::Lanczos3);
    unpremultiply(&mut out);
    out
}

fn decode(png: &[u8]) -> Result<RgbaImage, ImageError> {
    Ok(image::load_from_memory_with_format(png, image::ImageFormat::Png)?.to_rgba8())
}

pub fn compose(
    body_png: &[u8],
    face_png: Option<&[u8]>,
    sprite: &CharacterSprite,
    height: u32,
) -> Result<RgbaImage, ImageError> {
    let mut body = decode(body_png)?;
    let (bw, bh) = body.dimensions();
    if let (Some(face_png), Some(pos)) = (face_png, sprite.face_pos) {
        let (tw, th) = sprite
            .body_size
            .map_or((bw as f32, bh as f32), |s| (s.w, s.h));
        let (sx, sy) = (bw as f32 / tw, bh as f32 / th);
        let w = (pos.w * sx).round().max(1.0) as u32;
        let h = (pos.h * sy).round().max(1.0) as u32;
        let mut patch = decode(face_png)?;
        if patch.dimensions() != (w, h) {
            patch = resize(patch, w, h);
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
    Ok(resize(body, width, height))
}

pub fn encode(img: &RgbaImage, format: ThumbFormat) -> Result<Vec<u8>, ImageError> {
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

fn alpha_head(img: &RgbaImage) -> Option<(f32, f32)> {
    let (w, h) = img.dimensions();
    let opaque = |x: u32, y: u32| img.get_pixel(x, y)[3] >= OPAQUE;
    let top = (0..h).find(|&y| (0..w).filter(|&x| opaque(x, y)).count() >= TOP_ROW_MIN_OPAQUE)?;
    let band_end = (top + (HEAD_BAND * h as f32) as u32).min(h);
    let (mut sum, mut n) = (0u64, 0u64);
    for y in top..band_end {
        for x in (0..w).filter(|&x| opaque(x, y)) {
            sum += u64::from(x);
            n += 1;
        }
    }
    let x = if n == 0 {
        w as f32 / 2.0
    } else {
        sum as f32 / n as f32
    };
    Some((x, top as f32 + FACE_BELOW_TOP * h as f32))
}

#[must_use]
pub fn crop_rect(img: &RgbaImage, face: Option<(f32, f32)>) -> (u32, u32, u32) {
    let (w, h) = img.dimensions();
    let (wf, hf) = (w as f32, h as f32);
    let (cx, cy) = face.map_or_else(
        || alpha_head(img).unwrap_or((EMPTY_PLATE_HEAD.0 * wf, EMPTY_PLATE_HEAD.1 * hf)),
        |(x, y)| (x * wf, y * hf),
    );
    let side = (SIDE * wf).min(wf).min(hf).max(1.0);
    let cy = cy + DROP * side;
    let left = (cx - side / 2.0).clamp(0.0, wf - side);
    let top = (cy - side / 2.0).clamp(0.0, hf - side);
    (left.round() as u32, top.round() as u32, side.round() as u32)
}

pub fn render_head(
    png: &[u8],
    face: Option<(f32, f32)>,
    px: u32,
    format: ThumbFormat,
) -> Result<Vec<u8>, ImageError> {
    let img = decode(png)?;
    let (left, top, side) = crop_rect(&img, face);
    let crop = imageops::crop_imm(&img, left, top, side, side).to_image();
    encode(&resize(crop, px, px), format)
}

pub fn is_blank(png: &[u8]) -> bool {
    image::load_from_memory(png).is_ok_and(|img| img.to_rgba8().pixels().all(|p| p[3] == 0))
}
