use std::collections::HashSet;

use crate::assets::{ImageSource, StoryAssetIndex};
use crate::dto::{StoryArtGallery, StoryArtGroup, StoryArtKind, StoryArtPicture};
use crate::index::StoryLibrary;

pub const MIN_WIDTH: u32 = 1024;
pub const MIN_ASPECT: f64 = 1.6;
pub const MAX_ASPECT: f64 = 1.9;
pub const MIN_BYTES_PER_PX: f64 = 0.2;
pub const CACHE_DIR: &str = "derived/story-art";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exclusion {
    Narrow,
    Aspect,
    Sparse,
}

/// Whether a `w`x`h` source of `bytes` can be a header
pub fn header_fit(w: u32, h: u32, bytes: Option<u64>, min_bpp: f64) -> Result<(), Exclusion> {
    if w < MIN_WIDTH {
        return Err(Exclusion::Narrow);
    }
    let aspect = f64::from(w) / f64::from(h.max(1));
    if !(MIN_ASPECT..=MAX_ASPECT).contains(&aspect) {
        return Err(Exclusion::Aspect);
    }
    if bytes.is_some_and(|b| (b as f64) / (f64::from(w) * f64::from(h)) < min_bpp) {
        return Err(Exclusion::Sparse);
    }
    Ok(())
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StoryArtCensus {
    pub distinct: usize,
    pub passed: usize,
    pub narrow: usize,
    pub aspect: usize,
    pub sparse: usize,
    pub unresolved: usize,
    pub excluded: Vec<(String, u32, u32, u64)>,
}

const fn slot(kind: StoryArtKind) -> usize {
    match kind {
        StoryArtKind::Cg => 0,
        StoryArtKind::Scene => 1,
    }
}

#[must_use]
pub fn resolve_art(
    assets: &StoryAssetIndex,
    kind: StoryArtKind,
    id: &str,
) -> Option<(String, u32, u32)> {
    let (resolved, size, own) = match kind {
        StoryArtKind::Cg => (
            assets.resolve_image(id)?,
            assets.resolve_image_size(id)?,
            ImageSource::Image,
        ),
        StoryArtKind::Scene => (
            assets.resolve_background(id)?,
            assets.resolve_background_size(id)?,
            ImageSource::Background,
        ),
    };
    let px = |v: f32| {
        let n = v.round();
        if n.is_finite() && n > 0.0 {
            n as u32
        } else {
            0
        }
    };
    (resolved.1 == own).then(|| (resolved.0, px(size.w), px(size.h)))
}

#[must_use]
pub fn cache_rel_path(kind: StoryArtKind, id: &str, size: crate::gallery::GallerySize) -> String {
    crate::gallery::cache_rel_path_in(&format!("{CACHE_DIR}/{}", kind.as_str()), id, size)
}

#[must_use]
pub fn build_story_art(
    library: &StoryLibrary,
    assets: &StoryAssetIndex,
    file_len: &dyn Fn(&str) -> Option<u64>,
    min_bpp: f64,
) -> [(StoryArtGallery, StoryArtCensus); 2] {
    let mut out = [StoryArtKind::Cg, StoryArtKind::Scene].map(|kind| {
        (
            StoryArtGallery {
                kind,
                groups: Vec::new(),
            },
            StoryArtCensus::default(),
        )
    });
    let mut seen: HashSet<(StoryArtKind, String)> = HashSet::new();
    for group in &library.index.groups {
        let Some(refs) = library.illustrations.get(&group.id) else {
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
            let Some((_, src)) = resolved else { continue };
            let kind = match src {
                ImageSource::Image => StoryArtKind::Cg,
                ImageSource::Background => StoryArtKind::Scene,
            };
            if !seen.insert((kind, id.clone())) {
                continue;
            }
            let census = &mut out[slot(kind)].1;
            census.distinct += 1;
            let Some((url, width, height)) = resolve_art(assets, kind, &id) else {
                census.unresolved += 1;
                continue;
            };
            let bytes = file_len(&url);
            match header_fit(width, height, bytes, min_bpp) {
                Ok(()) => {
                    census.passed += 1;
                    pictures[slot(kind)].push(StoryArtPicture { id, width, height });
                }
                Err(e) => {
                    *match e {
                        Exclusion::Narrow => &mut census.narrow,
                        Exclusion::Aspect => &mut census.aspect,
                        Exclusion::Sparse => &mut census.sparse,
                    } += 1;
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
