use std::collections::HashSet;

use image::{ImageEncoder, ImageError, RgbImage, imageops};
use myrtle_gamedata::model::GameModel;

use crate::archive::slot_ids;
use crate::assets::StoryAssetIndex;
use crate::dto::{GalleryGroup, GalleryPicture, StoryGallery};
use crate::opt;

const CACHE_DIR: &str = "derived/archive-pics";
const QUALITY: u8 = 82;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GallerySize {
    Thumb,
    Header,
}

impl GallerySize {
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

#[must_use]
pub fn known(m: &GameModel, id: &str) -> bool {
    m.story_archives.act_archive_res_data.pics.contains_key(id)
}

#[must_use]
pub fn cache_rel_path(id: &str, size: GallerySize) -> String {
    cache_rel_path_in(CACHE_DIR, id, size)
}

#[must_use]
pub fn cache_rel_path_in(dir: &str, id: &str, size: GallerySize) -> String {
    let stem: String = id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    format!("{dir}/{}/{stem}.jpg", size.as_str())
}

fn group_name(m: &GameModel, component: &str) -> String {
    let named = |n: &str| Some(n.trim().to_owned()).filter(|n| !n.is_empty());
    m.activities
        .get(component)
        .and_then(|a| named(&a.name))
        .or_else(|| {
            m.roguelike
                .entries
                .get(component)
                .and_then(|e| named(&e.name))
        })
        .unwrap_or_else(|| component.to_owned())
}

#[must_use]
pub fn build_gallery(m: &GameModel, assets: &StoryAssetIndex) -> StoryGallery {
    let archives = &m.story_archives;
    let mut keys: Vec<&String> = archives.act_archive_data.components.keys().collect();
    keys.sort();
    let mut seen = HashSet::new();
    let groups = keys
        .into_iter()
        .filter_map(|key| {
            let component = &archives.act_archive_data.components[key];
            let pictures: Vec<GalleryPicture> = slot_ids(component.pic.as_ref())
                .into_iter()
                .filter(|id| seen.insert(*id))
                .filter_map(|id| archives.act_archive_res_data.pics.get(id))
                .map(|p| GalleryPicture {
                    id: p.id.clone(),
                    title: p.desc.trim().to_owned(),
                    description: opt(&p.pic_description),
                    picture_type: p.type_.clone(),
                    url: assets.resolve_image(&p.asset_path).map(|(u, _)| u),
                })
                .collect();
            (!pictures.is_empty()).then(|| GalleryGroup {
                id: key.clone(),
                name: group_name(m, key),
                pictures,
            })
        })
        .collect();
    StoryGallery { groups }
}

pub fn render_jpeg(png: &[u8], width: u32) -> Result<Vec<u8>, ImageError> {
    let src = image::load_from_memory_with_format(png, image::ImageFormat::Png)?.to_rgba8();
    let (sw, sh) = src.dimensions();
    let mut rgb = RgbImage::new(sw, sh);
    for (d, s) in rgb.pixels_mut().zip(src.pixels()) {
        let a = u16::from(s[3]);
        for c in 0..3 {
            d[c] = ((u16::from(s[c]) * a + 127) / 255) as u8;
        }
    }
    let width = width.min(sw).max(1);
    let height = ((u64::from(sh) * u64::from(width) + u64::from(sw) / 2) / u64::from(sw.max(1)))
        .max(1) as u32;
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
