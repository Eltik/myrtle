//! The story scripts' own art as profile-header backgrounds: every CG
//! (`textures/avg/imgs`) and scene plate (`textures/avg/bg`) a library
//! group's scripts draw, each once, wide enough and close enough to 16:9 to
//! fill the header. Built off the illustration names the story index already
//! caches per group ([`StoryIndexCache::illustrations`]), so no script is read
//! here; each name resolves through the same [`StoryAssetIndex`] the reader
//! uses, and the file it lands in decides the kind.
//!
//! The 320 px thumb and the header copy (1600 px, capped at the source width,
//! so a 1024 px scene stays 1024) go through the Archives gallery's JPEG
//! pipeline ([`super::gallery::ensure_variant`]) under
//! `derived/story-art/<kind>/<size>/<id>.jpg`.
//!
//! EN 2026-10-06 (`story_art_lists_each_header_sized_cg_and_scene_once`):
//! the library's 451 groups (Main 17, Side 50, Vignette 20, Record 364; the
//! EN index has no IS, Reclamation or side-content group) name 1,335
//! distinct CG files and 929 scene files.
//! - CGs: 1,230 pass (902 are 1600x900, 197 1280x720, 69 1500x844, 13
//!   1024x576). 45 are narrow (the `cgitem_*` item close-ups and strips up to
//!   1185 px, the 500 px `item_*` squares, the 533x702 `avg_6d5_*`
//!   portraits); 35 off-aspect (16 1024x1024 `cutin_char_*` cut-ins, the
//!   1280x230 to 1280x291 letterbox strips, `ac6_9` at 1280x2048, and some
//!   real paintings: `69_i12_*` 1600x1060, `58_mini06` 1600x1350,
//!   `cgitem_70_i17_*` ~1150x900, `44_i01` 1164x576); 25 sparse (see
//!   [`MIN_BYTES_PER_PX`]: the `avg_ep*` episode cards, `ac3_title*` and
//!   `ac6_2`/`ac6_5` title logos, `blocker_movi`, transparent splatter
//!   overlays `cgitem_70_i02_2`/`_3`).
//! - Scenes: 898 pass, every one 1024x576. 11 narrow (920x720
//!   `bg_beach`/`bg_falls`, the 677/704x800 `50_g2x` panels), 18 off-aspect
//!   (the 1024x384 and 1024x500 overlay halves, the near-square `38_g1x` and
//!   `48_i08` plates), 2 sparse (`bg_black`, `bg_white`).
//!
//! What the filter does NOT catch, by eye on a 42-tile sample of each and
//! the sparse tail: five title cards above the floor (named at
//! [`MIN_BYTES_PER_PX`]); phone and terminal UI shots (`pic_rogue_2_47`,
//! `bg_1_call`, 0.50 bytes per px, inside the painting range); a few
//! near-black rain overlays among the scenes (`66_i15_1` 0.32,
//! `66_i16_1`, `66_i17_1`), which sit beside plain-sky plates
//! (`38_g20_skyblue_r1` 0.315) that are real scenes.
//! The CG PNGs the library never names (1,551 under `textures/avg/imgs`
//! against 1,335 named, part of them IS stories the EN index does not list)
//! are not offered: the index is the catalogue, so a picture is always one a
//! story shows.
//! JSON: 59,576 bytes for the CGs and 54,787 for the scenes, so one route
//! per kind rather than one 114 KB payload. Variants on a 41-picture sample
//! of each: a CG thumb is 14,614 bytes and its header 192,742 against the
//! PNG's 1,616,954 (111x and 8.4x smaller); a scene thumb 13,020 and its
//! header (1024 px, the source width) 90,472 against 782,336 (60x, 8.6x).

use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Weak};

use super::cache::{ServerCache, StoryTrees, cached_index};
use super::dto::{StoryArtGallery, StoryArtGroup, StoryArtKind, StoryArtPicture};
use super::gallery::{GallerySize, cache_rel_path_in, ensure_variant, with_default_fallback};
use super::index::StoryIndexCache;
use crate::app::{cpu, error::ApiError, state::AppState};
use crate::core::gamedata::assets::AssetIndex;
use crate::core::hypergryph::constants::Server;
use crate::core::story::assets::{ImageSource, StoryAssetIndex, served_path};

/// The narrowest source a header takes, px.
pub const MIN_WIDTH: u32 = 1024;
/// The aspect band a header takes, width over height. 16:9 is 1.778; the
/// band admits the 1280x712 CGs (1.798) and keeps out a 4:3 or a 2:1 strip.
pub const MIN_ASPECT: f64 = 1.6;
pub const MAX_ASPECT: f64 = 1.9;

/// The PNG bytes per pixel below which a header-sized picture is a title
/// card or a fill rather than art: a TRADE, shipped knowingly. On EN
/// 2026-10-06 the CG tree's floor is a run of black episode cards, logos on
/// transparency and blockers (`blocker_movi` 0.021, `ac6_2` 0.036, the
/// `avg_ep*` cards 0.044 to 0.172), and the scene tree's is `bg_black` and
/// `bg_white` at 0.022 with nothing else below 0.315. 0.2 removes those and
/// no painting seen; it keeps five cards above it (`cc_start` 0.223,
/// `avg_ep10` 0.259, `avg_ep08` 0.304, `ac5_title1` 0.36, `avg_ep14` 0.399)
/// because `cg_firework`, a real sparse painting, is 0.396, so no floor
/// separates the two. `STORY_ART_MIN_BPP=0` turns it off, restoring the
/// dimension filter alone.
pub const MIN_BYTES_PER_PX: f64 = 0.2;

/// [`MIN_BYTES_PER_PX`], or the `STORY_ART_MIN_BPP` override when it parses
/// as a finite number (a MISSING variable is the default, never 0).
#[must_use]
pub fn min_bytes_per_px() -> f64 {
    std::env::var("STORY_ART_MIN_BPP")
        .ok()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite())
        .unwrap_or(MIN_BYTES_PER_PX)
}

/// Why a resolved picture is left out, for the census.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exclusion {
    /// Narrower than [`MIN_WIDTH`].
    Narrow,
    /// Wide enough, but outside the aspect band.
    Aspect,
    /// Header-shaped, but under the bytes-per-pixel floor.
    Sparse,
}

/// Whether a `w`x`h` px source of `bytes` (when its size is known) can be a
/// header under a `min_bpp` floor, and why not.
///
/// # Errors
/// The [`Exclusion`] that keeps it out.
pub fn header_fit(w: u32, h: u32, bytes: Option<u64>, min_bpp: f64) -> Result<(), Exclusion> {
    if w < MIN_WIDTH {
        return Err(Exclusion::Narrow);
    }
    let aspect = f64::from(w) / f64::from(h.max(1));
    if !(MIN_ASPECT..=MAX_ASPECT).contains(&aspect) {
        return Err(Exclusion::Aspect);
    }
    #[allow(clippy::cast_precision_loss)]
    let sparse = bytes.is_some_and(|b| (b as f64) / (f64::from(w) * f64::from(h)) < min_bpp);
    if sparse {
        Err(Exclusion::Sparse)
    } else {
        Ok(())
    }
}

/// What a build saw, per kind: distinct files, how many passed, how many each
/// [`Exclusion`] kept out, and names with no file or no size.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StoryArtCensus {
    pub distinct: usize,
    pub passed: usize,
    pub narrow: usize,
    pub aspect: usize,
    pub sparse: usize,
    pub unresolved: usize,
    /// Every picture kept out, as `(id, width, height, PNG bytes)`.
    pub excluded: Vec<(String, u32, u32, u64)>,
}

/// The cache directory of a kind's variants, under a server's art root.
const CACHE_DIR: &str = "derived/story-art";

/// The two kinds, in the order [`build_story_art`] returns them.
const KINDS: [StoryArtKind; 2] = [StoryArtKind::Cg, StoryArtKind::Scene];

/// `kind`'s index in [`KINDS`].
const fn slot(kind: StoryArtKind) -> usize {
    match kind {
        StoryArtKind::Cg => 0,
        StoryArtKind::Scene => 1,
    }
}

/// The tree a kind lives in.
const fn source_of(kind: StoryArtKind) -> ImageSource {
    match kind {
        StoryArtKind::Cg => ImageSource::Image,
        StoryArtKind::Scene => ImageSource::Background,
    }
}

/// The resolved URL and px size of `id` IN `kind`'s own tree, `None` when it
/// resolves elsewhere or not at all.
fn resolve(assets: &StoryAssetIndex, kind: StoryArtKind, id: &str) -> Option<(String, u32, u32)> {
    let (url, src, size) = match kind {
        StoryArtKind::Cg => {
            let (u, s) = assets.resolve_image(id)?;
            (u, s, assets.resolve_image_size(id)?)
        }
        StoryArtKind::Scene => {
            let (u, s) = assets.resolve_background(id)?;
            (u, s, assets.resolve_background_size(id)?)
        }
    };
    if src != source_of(kind) {
        return None;
    }
    // `ImageSize.w`/`h` are the texture's px (the plate's world size is
    // `w / ppu`), the same numbers the PNG header carries.
    let px = |v: f32| -> u32 {
        let n = v.round();
        if n.is_finite() && n > 0.0 {
            // Bounded by the PNG header's u32 dimensions.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let out = n as u32;
            out
        } else {
            0
        }
    };
    Some((url, px(size.w), px(size.h)))
}

/// Both catalogues and their censuses, in library group order: a picture is
/// filed under the first group whose scripts draw it, by either command
/// (`Background` or `Image`), and lands in the kind its FILE belongs to.
#[must_use]
pub fn build_story_art(
    cache: &StoryIndexCache,
    assets: &StoryAssetIndex,
    art_dir: &Path,
    min_bpp: f64,
) -> [(StoryArtGallery, StoryArtCensus); 2] {
    let mut out = KINDS.map(|kind| {
        (
            StoryArtGallery {
                kind,
                groups: Vec::new(),
            },
            StoryArtCensus::default(),
        )
    });
    let mut seen: HashSet<(StoryArtKind, String)> = HashSet::new();
    for group in &cache.index.groups {
        let Some(refs) = cache.illustrations.get(&group.id) else {
            continue;
        };
        let mut pictures: [Vec<StoryArtPicture>; 2] = [Vec::new(), Vec::new()];
        let names = refs.images.iter().map(|r| (ImageSource::Image, r)).chain(
            refs.backgrounds
                .iter()
                .map(|r| (ImageSource::Background, r)),
        );
        for (primary, r) in names {
            let id = r.name.trim().to_ascii_lowercase();
            let resolved = match primary {
                ImageSource::Image => assets.resolve_image(&id),
                ImageSource::Background => assets.resolve_background(&id),
            };
            let Some((_, src)) = resolved else {
                continue;
            };
            let kind = if src == ImageSource::Image {
                StoryArtKind::Cg
            } else {
                StoryArtKind::Scene
            };
            let at = slot(kind);
            if !seen.insert((kind, id.clone())) {
                continue;
            }
            let census = &mut out[at].1;
            census.distinct += 1;
            let Some((url, width, height)) = resolve(assets, kind, &id) else {
                census.unresolved += 1;
                continue;
            };
            let bytes = std::fs::metadata(served_path(art_dir, &url))
                .ok()
                .map(|m| m.len());
            match header_fit(width, height, bytes, min_bpp) {
                Ok(()) => {
                    census.passed += 1;
                    pictures[at].push(StoryArtPicture { id, width, height });
                }
                Err(e) => {
                    match e {
                        Exclusion::Narrow => census.narrow += 1,
                        Exclusion::Aspect => census.aspect += 1,
                        Exclusion::Sparse => census.sparse += 1,
                    }
                    census
                        .excluded
                        .push((id, width, height, bytes.unwrap_or(0)));
                }
            }
        }
        for (at, list) in pictures.into_iter().enumerate() {
            if !list.is_empty() {
                out[at].0.groups.push(StoryArtGroup {
                    id: group.id.clone(),
                    name: group.name.trim().to_owned(),
                    category: group.category,
                    pictures: list,
                });
            }
        }
    }
    out
}

/// Both catalogues plus the asset index they resolved against.
struct Cached {
    galleries: [(StoryArtGallery, StoryArtCensus); 2],
    assets: Weak<AssetIndex>,
}

impl Cached {
    const fn gallery(&self, kind: StoryArtKind) -> &StoryArtGallery {
        &self.galleries[slot(kind)].0
    }

    fn lists(&self, kind: StoryArtKind, id: &str) -> bool {
        self.gallery(kind)
            .groups
            .iter()
            .any(|g| g.pictures.iter().any(|p| p.id == id))
    }
}

static STORY_ART: ServerCache<Cached> = ServerCache::new();

async fn cached(state: &AppState, server: Server) -> Result<Arc<Cached>, ApiError> {
    let sd = state.require_server_data(server)?;
    let gd = sd.game_data.load_full();
    let trees = StoryTrees::of(&sd);
    let index = cached_index(state, server).await?;
    let live = Arc::clone(&trees.live);
    STORY_ART
        .get_or_build(
            server,
            gd,
            |hit| hit.assets.upgrade().is_some_and(|a| Arc::ptr_eq(&a, &live)),
            || async move {
                cpu::run("story_art_gallery", move || {
                    let assets = trees.story_assets();
                    Arc::new(Cached {
                        galleries: build_story_art(
                            &index,
                            &assets,
                            &trees.art_dir,
                            min_bytes_per_px(),
                        ),
                        assets: Arc::downgrade(&trees.live),
                    })
                })
                .await
            },
        )
        .await
}

/// `server`'s catalogue of `kind`, built once per game data and asset tree.
///
/// # Errors
/// `404` for a server that is not loaded, `503` when the CPU pool sheds.
pub async fn get_story_art(
    state: &AppState,
    server: Server,
    kind: StoryArtKind,
) -> Result<StoryArtGallery, ApiError> {
    Ok(cached(state, server).await?.gallery(kind).clone())
}

/// `server`'s census of `kind`, for the real-data test.
///
/// # Errors
/// As [`get_story_art`].
pub async fn story_art_census(
    state: &AppState,
    server: Server,
    kind: StoryArtKind,
) -> Result<StoryArtCensus, ApiError> {
    Ok(cached(state, server).await?.galleries[slot(kind)].1.clone())
}

/// Whether any loaded server's catalogue of `kind` lists `id` (exact: ids
/// are lowercase on the wire). A server whose catalogue fails to build
/// counts as not listing it.
pub async fn story_art_known(state: &AppState, kind: StoryArtKind, id: &str) -> bool {
    let mut servers: Vec<Server> = state
        .servers
        .iter()
        .filter(|(_, sd)| sd.loaded.load(std::sync::atomic::Ordering::Acquire))
        .map(|(s, _)| *s)
        .collect();
    servers.sort_by_key(|s| *s != state.default_server);
    for server in servers {
        if let Ok(c) = cached(state, server).await
            && c.lists(kind, id)
        {
            return true;
        }
    }
    false
}

/// The variant to serve for `size` of `kind` picture `id`, `server` first
/// and then the default server.
///
/// # Errors
/// `404` when no loaded server lists the id.
pub async fn ensure_story_art(
    state: &AppState,
    server: Server,
    kind: StoryArtKind,
    id: &str,
    size: GallerySize,
) -> Result<(String, String), ApiError> {
    for srv in with_default_fallback(state, server) {
        let Ok(c) = cached(state, srv).await else {
            continue;
        };
        if !c.lists(kind, id) {
            continue;
        }
        let Some(sd) = state.try_server_data(srv) else {
            continue;
        };
        let assets = StoryTrees::of(&sd).story_assets();
        let Some((url, _, _)) = resolve(&assets, kind, id) else {
            continue;
        };
        let rel = cache_rel_path_in(&format!("{CACHE_DIR}/{}", kind.as_str()), id, size);
        return ensure_variant(sd.art_dir.clone(), &url, rel, size).await;
    }
    Err(ApiError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_header_band_takes_16_9_and_the_1280x712_cgs_only() {
        let fit = |w, h| header_fit(w, h, None, MIN_BYTES_PER_PX);
        assert_eq!(fit(1600, 900), Ok(()));
        assert_eq!(fit(1280, 712), Ok(()));
        assert_eq!(fit(1024, 576), Ok(()));
        assert_eq!(fit(1023, 576), Err(Exclusion::Narrow));
        assert_eq!(fit(512, 1024), Err(Exclusion::Narrow));
        assert_eq!(fit(1024, 768), Err(Exclusion::Aspect));
        assert_eq!(fit(2048, 1024), Err(Exclusion::Aspect));
        assert_eq!(fit(1024, 0), Err(Exclusion::Aspect));
    }

    #[test]
    fn a_title_card_is_too_sparse_and_the_floor_switches_off() {
        // `bg_black`: 1024x576 in 12,790 bytes is 0.022 per px.
        let px = 1024 * 576;
        assert_eq!(
            header_fit(1024, 576, Some(12_790), MIN_BYTES_PER_PX),
            Err(Exclusion::Sparse)
        );
        assert_eq!(
            header_fit(1024, 576, Some(px / 5 + 1), MIN_BYTES_PER_PX),
            Ok(())
        );
        assert_eq!(header_fit(1024, 576, Some(12_790), 0.0), Ok(()));
        // A size that could not be read is no evidence either way.
        assert_eq!(header_fit(1024, 576, None, MIN_BYTES_PER_PX), Ok(()));
        // The dimension checks come first, so the census names the shape.
        assert_eq!(
            header_fit(500, 500, Some(1), MIN_BYTES_PER_PX),
            Err(Exclusion::Narrow)
        );
    }

    #[test]
    fn story_art_ids_land_inside_their_cache() {
        assert_eq!(
            cache_rel_path_in("derived/story-art/cg", "avg_1_1", GallerySize::Thumb),
            "derived/story-art/cg/thumb/avg_1_1.jpg"
        );
        assert_eq!(
            cache_rel_path_in("derived/story-art/scene", "../bg#2", GallerySize::Header),
            "derived/story-art/scene/header/___bg_2.jpg"
        );
    }
}
