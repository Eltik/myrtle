//! Every Archives gallery picture as one catalogue, and each picture as two
//! smaller JPEGs, for the profile header's background picker.
//!
//! The pictures are `story_review_meta_table.ActArchiveResData.Pics`: 324 on
//! EN 2026-10-06, every one a 1600x900 RGBA PNG under `textures/avg/imgs`,
//! 2,049,755 bytes on average (664,120,699 in all). The group is the archive
//! component that lists the picture, the first in key order, so a live act
//! (`act13side`) and its retro twin (`act13sre`), which list the same ids,
//! become one group: 8 on EN (3 events, 5 Integrated Strategies themes).
//!
//! The variants are rendered on first request and kept under
//! `derived/archive-pics/<size>/<id>.jpg`, the `sprite_thumbs` pattern: a
//! variant is fresh while newer than its source, `derived/` is outside what
//! the extract writes so the orphan sweep never reaches it, and a hit older
//! than a day has its mtime refreshed (the `pool_write_png` rule).
//!
//! Why JPEG: the alpha channel carries nothing. 286 of the 324 are opaque
//! everywhere, and the other 38 have at most 0.289% of their pixels below
//! 255 (`act25side_pic_11`), stray edge pixels, so the variants drop it onto
//! black. What this encoder writes, over all 324 on EN 2026-10-06 (the
//! `profile_layout_real_data_test` gallery census, `GALLERY_ALL=1`): the
//! 320x180 thumb is 13,630 bytes on average (median 12,934, max 30,686), the
//! 1600x900 header 224,384 (median 212,452, max 560,923), against the PNG's
//! 2,049,755: 150x and 9.1x smaller. The header keeps the full 1600 px: the
//! header measured 1440x182 CSS px at a 1500 px viewport, so its right 62%
//! (893 px) already wants 1786 device px on a 2x screen.
//! Ruled out by measurement on 24 pictures (seed 1, PIL encoders, so only
//! the ratios carry over): lossless WebP, the only WebP the `image` crate
//! encodes, at 64,506 bytes for the thumb and 875,145 for the header against
//! 11,761 and 193,438 for JPEG q82; a 1280 px header at 129,686, which saves
//! a third of the bytes but drops below what a 2x header shows.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Weak};

use image::{ImageEncoder, RgbImage, imageops};
use tokio::sync::Semaphore;

use super::archive::slot_ids;
use super::cache::{ServerCache, StoryTrees};
use super::dto::{GalleryGroup, GalleryPicture, StoryGallery};
use super::sprite_thumbs::{modified, touch_if_old, write_atomically};
use crate::app::{cpu, error::ApiError, state::AppState};
use crate::core::gamedata::assets::AssetIndex;
use crate::core::gamedata::types::GameData;
use crate::core::hypergryph::constants::Server;
use crate::core::story::assets::{StoryAssetIndex, served_path};

/// The cache directory under a server's art root.
const CACHE_DIR: &str = "derived/archive-pics";

/// JPEG quality of both variants.
const QUALITY: u8 = 82;

/// Pictures rendered at once; each is a 1600x900 PNG decode plus a resize.
const MAX_CONCURRENT_RENDERS: usize = 2;

static RENDERS: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(MAX_CONCURRENT_RENDERS));

/// Which variant of a picture to serve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GallerySize {
    /// 320 px wide, for a picker tile.
    Thumb,
    /// The full 1600 px width, re-encoded, for the profile header.
    Header,
}

impl GallerySize {
    /// Output width in px; the height follows the picture's aspect.
    #[must_use]
    pub const fn width(self) -> u32 {
        match self {
            Self::Thumb => 320,
            Self::Header => 1600,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Thumb => "thumb",
            Self::Header => "header",
        }
    }
}

/// Whether `gd` lists `id` as a gallery picture. Exact: the ids are the
/// table's own keys (`act13side_pic_0`, `pic_rogue_1_KV1`).
#[must_use]
pub fn known(gd: &GameData, id: &str) -> bool {
    gd.story_archives.act_archive_res_data.pics.contains_key(id)
}

/// The variant's path relative to the art root. The id is lowercased and
/// anything outside `[a-z0-9_]` becomes `_`, so a path never leaves the
/// cache directory; ids are checked against the table before this is built.
#[must_use]
pub fn cache_rel_path(id: &str, size: GallerySize) -> String {
    cache_rel_path_in(CACHE_DIR, id, size)
}

/// [`cache_rel_path`] under another cache directory.
#[must_use]
pub fn cache_rel_path_in(dir: &str, id: &str, size: GallerySize) -> String {
    let stem: String = id
        .to_ascii_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("{dir}/{}/{stem}.jpg", size.as_str())
}

/// A group's display name: the event's name, else the Integrated Strategies
/// theme's, else the component id itself.
fn group_name(gd: &GameData, component: &str) -> String {
    gd.activities
        .get(component)
        .map(|a| a.name.trim().to_owned())
        .filter(|n| !n.is_empty())
        .or_else(|| {
            gd.roguelike
                .entries
                .get(component)
                .map(|e| e.name.trim().to_owned())
                .filter(|n| !n.is_empty())
        })
        .unwrap_or_else(|| component.to_owned())
}

/// The catalogue: groups in component key order, each picture once, in the
/// order its archive shows it. A picture whose file is missing is still
/// listed, with no `url`, so a stored background never vanishes from the
/// picker while the data still names it.
#[must_use]
pub fn build_gallery(gd: &GameData, story_assets: &StoryAssetIndex) -> StoryGallery {
    let res = &gd.story_archives.act_archive_res_data;
    let mut keys: Vec<&String> = gd
        .story_archives
        .act_archive_data
        .components
        .keys()
        .collect();
    keys.sort();
    let mut seen = HashSet::new();
    let mut groups = Vec::new();
    for key in keys {
        let component = &gd.story_archives.act_archive_data.components[key];
        let pictures: Vec<GalleryPicture> = slot_ids(component.pic.as_ref())
            .into_iter()
            .filter(|id| seen.insert(id.clone()))
            .filter_map(|id| res.pics.get(&id))
            .map(|p| GalleryPicture {
                id: p.id.clone(),
                title: p.desc.trim().to_owned(),
                description: super::opt(&p.pic_description),
                picture_type: p.type_.clone(),
                url: story_assets.resolve_image(&p.asset_path).map(|(u, _)| u),
            })
            .collect();
        if !pictures.is_empty() {
            groups.push(GalleryGroup {
                id: key.clone(),
                name: group_name(gd, key),
                pictures,
            });
        }
    }
    StoryGallery { groups }
}

/// The catalogue plus the asset index it resolved against, so an asset hot
/// reload rebuilds it as a game-data reload does.
struct Cached {
    gallery: StoryGallery,
    assets: Weak<AssetIndex>,
}

static GALLERY: ServerCache<Cached> = ServerCache::new();

/// `server`'s catalogue, built once per game data and asset tree.
///
/// # Errors
/// `404` for a server that is not loaded, `503` when the CPU pool sheds.
pub async fn get_gallery(state: &AppState, server: Server) -> Result<StoryGallery, ApiError> {
    let sd = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let gd = sd.game_data.load_full();
    let trees = StoryTrees::of(&sd);
    let build_gd = Arc::clone(&gd);
    let live = Arc::clone(&trees.live);
    let cached = GALLERY
        .get_or_build(
            server,
            gd,
            |hit| hit.assets.upgrade().is_some_and(|a| Arc::ptr_eq(&a, &live)),
            || async move {
                cpu::run("story_gallery", move || {
                    let story_assets = trees.story_assets();
                    Arc::new(Cached {
                        gallery: build_gallery(&build_gd, &story_assets),
                        assets: Arc::downgrade(&trees.live),
                    })
                })
                .await
            },
        )
        .await?;
    Ok(cached.gallery.clone())
}

/// Scale `png` to `width` (height by its aspect) as RGB on black, JPEG.
///
/// # Errors
/// A PNG that does not decode.
pub fn render(png: &[u8], width: u32) -> anyhow::Result<Vec<u8>> {
    let src = image::load_from_memory_with_format(png, image::ImageFormat::Png)?.to_rgba8();
    let (sw, sh) = src.dimensions();
    // Onto black, so the near-opaque edge pixels darken by at most their alpha.
    let mut rgb = RgbImage::new(sw, sh);
    for (d, s) in rgb.pixels_mut().zip(src.pixels()) {
        let a = u16::from(s[3]);
        for c in 0..3 {
            d[c] = u8::try_from((u16::from(s[c]) * a + 127) / 255).unwrap_or(u8::MAX);
        }
    }
    let width = width.min(sw).max(1);
    let height = u32::try_from(
        (u64::from(sh) * u64::from(width) + u64::from(sw) / 2) / u64::from(sw.max(1)),
    )
    .unwrap_or(1)
    .max(1);
    let out = if (width, height) == (sw, sh) {
        rgb
    } else {
        imageops::resize(&rgb, width, height, imageops::FilterType::Lanczos3)
    };
    let mut bytes = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, QUALITY).write_image(
        out.as_raw(),
        out.width(),
        out.height(),
        image::ExtendedColorType::Rgb8,
    )?;
    Ok(bytes)
}

/// The file to serve for `size` of picture `id`, as `(art_dir, rel)`: the
/// cached variant, rendered when missing or older than the source. When the
/// render fails the original PNG is served instead, so a header never goes
/// blank over an encoder fault. The default server is tried after `server`,
/// as the sprite thumbnail route does.
///
/// # Errors
/// `404` when no loaded server lists the id or has its file.
pub async fn ensure_picture(
    state: &AppState,
    server: Server,
    id: &str,
    size: GallerySize,
) -> Result<(String, String), ApiError> {
    for srv in with_default_fallback(state, server) {
        let Ok(gallery) = get_gallery(state, srv).await else {
            continue;
        };
        let Some(url) = gallery
            .groups
            .iter()
            .flat_map(|g| &g.pictures)
            .find(|p| p.id == id)
            .and_then(|p| p.url.clone())
        else {
            continue;
        };
        let Some(sd) = state.try_server_data(srv) else {
            continue;
        };
        // The url is relative to the art tree, so the variant is derived from
        // and cached beside its source there, not under a text-only tree.
        let rel = cache_rel_path(id, size);
        return ensure_variant(sd.art_dir.clone(), &url, rel, size).await;
    }
    Err(ApiError::NotFound)
}

/// `server`, then the default server when that is another: the order a
/// variant route tries.
pub fn with_default_fallback(state: &AppState, server: Server) -> Vec<Server> {
    let mut servers = vec![server];
    if server != state.default_server {
        servers.push(state.default_server);
    }
    servers
}

/// The variant at `rel` (under `art_dir`) of the PNG a wire `url` names,
/// as `(art_dir, rel)` for [`serve_file`](crate::app::routes::assets::serve_file):
/// served from the cache while it is newer than its source, else rendered
/// under the shared two-render semaphore and written atomically. A failed
/// render serves the original PNG, so a header never goes blank. Shared by
/// the Archives gallery and the story CG and scene galleries.
///
/// # Errors
/// `500` when the render pool is closed, `503` when the CPU pool sheds.
pub(super) async fn ensure_variant(
    art_dir: String,
    url: &str,
    rel: String,
    size: GallerySize,
) -> Result<(String, String), ApiError> {
    let root = PathBuf::from(&art_dir);
    let source = served_path(&root, url);
    let original = url.trim_start_matches('/').replace("%23", "#");
    let target = root.join(&rel);
    let fresh =
        |target: &Path| modified(target).is_some_and(|t| modified(&source).is_some_and(|m| t >= m));
    if fresh(&target) {
        touch_if_old(&target);
        return Ok((art_dir, rel));
    }
    let _permit = RENDERS
        .acquire()
        .await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("gallery render pool closed")))?;
    if fresh(&target) {
        return Ok((art_dir, rel));
    }
    let tmp_name = format!(
        "{}.{}.{:?}.tmp",
        size.as_str(),
        std::process::id(),
        std::thread::current().id()
    );
    let width = size.width();
    let rendered = cpu::run("story_gallery_variant", move || -> anyhow::Result<()> {
        let png = std::fs::read(&source)?;
        write_atomically(&target, &tmp_name, &render(&png, width)?)
    })
    .await?;
    match rendered {
        Ok(()) => Ok((art_dir, rel)),
        Err(e) => {
            tracing::warn!(rel, error = %e, "gallery variant failed");
            Ok((art_dir, original))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(img: &image::RgbaImage) -> Vec<u8> {
        let mut out = Vec::new();
        image::DynamicImage::ImageRgba8(img.clone())
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    }

    #[test]
    fn ids_become_file_names_inside_the_cache() {
        assert_eq!(
            cache_rel_path("pic_rogue_1_KV1", GallerySize::Thumb),
            "derived/archive-pics/thumb/pic_rogue_1_kv1.jpg"
        );
        assert_eq!(
            cache_rel_path("../x/y", GallerySize::Header),
            "derived/archive-pics/header/___x_y.jpg"
        );
    }

    #[test]
    fn a_variant_keeps_the_aspect_and_never_upscales() {
        let src = image::RgbaImage::from_pixel(1600, 900, image::Rgba([200, 100, 50, 255]));
        let thumb = image::load_from_memory(&render(&png(&src), 320).unwrap()).unwrap();
        assert_eq!((thumb.width(), thumb.height()), (320, 180));
        let header = image::load_from_memory(&render(&png(&src), 1600).unwrap()).unwrap();
        assert_eq!((header.width(), header.height()), (1600, 900));
        let small = image::RgbaImage::from_pixel(800, 450, image::Rgba([0, 0, 0, 255]));
        let capped = image::load_from_memory(&render(&png(&small), 1600).unwrap()).unwrap();
        assert_eq!((capped.width(), capped.height()), (800, 450));
    }

    #[test]
    fn transparency_lands_on_black() {
        let src = image::RgbaImage::from_pixel(64, 36, image::Rgba([255, 255, 255, 0]));
        let out = image::load_from_memory(&render(&png(&src), 64).unwrap())
            .unwrap()
            .to_rgb8();
        assert!(out.pixels().all(|p| p[0] < 8 && p[1] < 8 && p[2] < 8));
    }
}
