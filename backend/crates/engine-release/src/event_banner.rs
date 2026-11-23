use image::{ImageEncoder, ImageError, ImageFormat, Rgba, RgbaImage, imageops};
use myrtle_gamedata::assets::AssetKind;
use myrtle_gamedata::model::GameModel;

pub const WIDTH: u32 = 568;
pub const HEIGHT: u32 = 336;
pub const CACHE_DIR: &str = "derived/event-banners";

const TITLE_MAX_W: f32 = 0.5;
const TITLE_MAX_H: f32 = 0.45;
const TITLE_RIGHT_MARGIN: f32 = 0.042;
const JPEG_QUALITY: u8 = 88;
/// Bump to invalidate every cached banner when the layout or encoding changes
const VERSION: &str = "v1";

/// The key visual and title paths in the art tree
#[must_use]
pub fn sources(m: &GameModel, act_id: &str) -> Option<(String, String)> {
    let set = m
        .storyline_story_sets
        .values()
        .find(|s| s.covers_activity(act_id))?;
    let background = m
        .assets
        .path(AssetKind::StorylineArt, set.background_id()?)?;
    let title = m
        .assets
        .path(AssetKind::StorylineArt, set.title_image_id.as_deref()?)?;
    Some((background.to_owned(), title.to_owned()))
}

/// Served path relative to the server's assets root
#[must_use]
pub fn cache_rel_path(act_id: &str) -> String {
    format!("{CACHE_DIR}/{VERSION}/{act_id}.jpg")
}

/// JPEG of the key visual cut from its left edge, title trimmed to its ink and centred at the right
pub fn render(background_png: &[u8], title_png: &[u8]) -> Result<Vec<u8>, ImageError> {
    let bg = image::load_from_memory_with_format(background_png, ImageFormat::Png)?.to_rgba8();
    let scale = (WIDTH as f32 / bg.width() as f32).max(HEIGHT as f32 / bg.height() as f32);
    let (sw, sh) = (
        ((bg.width() as f32 * scale).round() as u32).max(WIDTH),
        ((bg.height() as f32 * scale).round() as u32).max(HEIGHT),
    );
    let bg = imageops::resize(&bg, sw, sh, imageops::FilterType::Lanczos3);
    let mut out = RgbaImage::from_pixel(WIDTH, HEIGHT, Rgba([0, 0, 0, 255]));
    let top = (sh - HEIGHT) / 2;
    imageops::overlay(
        &mut out,
        &imageops::crop_imm(&bg, 0, top, WIDTH, HEIGHT).to_image(),
        0,
        0,
    );

    let title = image::load_from_memory_with_format(title_png, ImageFormat::Png)?.to_rgba8();
    if let Some((x, y, w, h)) = opaque_box(&title) {
        let mut title = imageops::crop_imm(&title, x, y, w, h).to_image();
        let fit =
            (WIDTH as f32 * TITLE_MAX_W / w as f32).min(HEIGHT as f32 * TITLE_MAX_H / h as f32);
        let (tw, th) = (
            ((w as f32 * fit).round() as u32).max(1),
            ((h as f32 * fit).round() as u32).max(1),
        );
        premultiply(&mut title); // so transparent pixels' RGB does not fringe the edges
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

/// `(x, y, w, h)` of every pixel with any alpha
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
