use std::collections::{BTreeMap, HashMap, HashSet};
use std::io;
use std::sync::OnceLock;

use myrtle_gamedata_raw::tables::story::{PNG_HEAD_LEN, StoryVariables};
use myrtle_gamedata_raw::{DirSource, TableSource};
use serde::{Deserialize, Serialize};

/// Face patch placement in body texture pixels
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FacePos {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BodySize {
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PlateRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ImageSize {
    pub w: f32,
    pub h: f32,
    pub ppu: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub ppu_assumed: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CharacterSprite {
    pub body_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub face_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub face_pos: Option<FacePos>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub body_size: Option<BodySize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub plate: Option<PlateRect>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MusicCue {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub intro: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub r#loop: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct VideoSources {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub webm_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub mp4_url: Option<String>,
}

/// Resolved assets of one script
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct StoryAssets {
    pub backgrounds: BTreeMap<String, String>,
    pub images: BTreeMap<String, String>,
    pub characters: BTreeMap<String, CharacterSprite>,
    pub music: BTreeMap<String, MusicCue>,
    pub sounds: BTreeMap<String, String>,
    pub avatars: BTreeMap<String, String>,
    pub image_sizes: BTreeMap<String, ImageSize>,
    pub videos: BTreeMap<String, VideoSources>,
}

/// `base#face$body` or `base@alias$body`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteName<'a> {
    pub base: &'a str,
    pub face: Option<&'a str>,
    pub body: Option<&'a str>,
    pub alias: Option<&'a str>,
}

#[must_use]
pub fn parse_sprite_name(raw: &str) -> SpriteName<'_> {
    const MARKERS: [char; 3] = ['#', '$', '@'];
    let name = raw.trim();
    let (base, mut rest) = name.find(MARKERS).map_or((name, ""), |i| name.split_at(i));
    let (mut face, mut body, mut alias) = (None, None, None);
    while let Some(marker) = rest.chars().next() {
        let tail = &rest[1..];
        let end = tail.find(MARKERS).unwrap_or(tail.len());
        let value = tail[..end].trim();
        let slot = match marker {
            '#' => &mut face,
            '$' => &mut body,
            _ => &mut alias,
        };
        if slot.is_none() {
            *slot = Some(value);
        }
        rest = &tail[end..];
    }
    SpriteName {
        base: base.trim(),
        face: face.filter(|f| !f.is_empty()),
        body: body.filter(|b| !b.is_empty()),
        alias: alias.filter(|a| !a.is_empty()),
    }
}

#[must_use]
pub fn trim_index(raw: &str) -> &str {
    let t = raw.trim_start_matches('0');
    if t.is_empty() && !raw.is_empty() {
        "0"
    } else {
        t
    }
}

#[must_use]
pub fn served_rel(url: &str) -> String {
    url.trim_start_matches('/').replace("%23", "#")
}

fn url_of(rel: &str) -> String {
    format!("/{}", rel.replace('#', "%23"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageSource {
    Background,
    Image,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct HubPoint {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct HubExtent {
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct HubSprite {
    #[serde(default)]
    name: String,
    #[serde(default)]
    alias: String,
    #[serde(default, rename = "isWholeBody")]
    is_whole_body: bool,
    #[serde(default)]
    size: Option<HubExtent>,
}

#[derive(Debug, Clone, Deserialize)]
struct HubGroup {
    #[serde(rename = "facePos")]
    face_pos: HubPoint,
    #[serde(rename = "faceSize")]
    face_size: HubExtent,
    #[serde(default)]
    sprites: Vec<HubSprite>,
}

#[derive(Debug, Clone, Deserialize)]
struct HubFile {
    #[serde(default)]
    groups: Vec<HubGroup>,
    #[serde(default)]
    root: Option<PlateRect>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct SpriteMeta {
    w: f32,
    h: f32,
    ppu: f32,
}

impl HubGroup {
    fn is_sentinel(&self) -> bool {
        self.face_size.w <= 0.0
            || self.face_size.h <= 0.0
            || self.face_pos.x < 0.0
            || self.face_pos.y < 0.0
    }

    fn is_whole(&self, stem: &str) -> bool {
        self.sprites
            .iter()
            .any(|s| s.name == stem && s.is_whole_body)
    }

    fn face_stem(&self, body_stem: &str, name: &SpriteName<'_>) -> Option<&str> {
        if self.is_sentinel() {
            return None;
        }
        let sprite = if let Some(alias) = name.alias {
            let alias = alias.to_ascii_lowercase();
            self.sprites.iter().find(|s| s.alias == alias)?
        } else {
            let n = name
                .face
                .and_then(|f| trim_index(f).parse::<usize>().ok())
                .filter(|n| *n >= 1)
                .map_or(0, |n| n - 1);
            self.sprites.get(n)?
        };
        (!sprite.is_whole_body && sprite.name != body_stem && !sprite.name.is_empty())
            .then_some(sprite.name.as_str())
    }
}

#[derive(Debug, Default)]
struct SpriteFolder {
    name: String,
    /// Lowercase stem -> file name on disk
    files: HashMap<String, String>,
    hub: Vec<HubGroup>,
    plate: Option<PlateRect>,
    /// PNG header sizes
    head_sizes: HashMap<String, OnceLock<BodySize>>,
}

impl SpriteFolder {
    fn url(&self, file: &str) -> String {
        url_of(&format!("textures/avg/characters/{}/{file}", self.name))
    }

    fn hub_size(&self, stem: &str) -> Option<BodySize> {
        self.hub
            .iter()
            .flat_map(|g| &g.sprites)
            .find(|s| s.name == stem)
            .and_then(|s| s.size)
            .filter(|e| e.w > 0.0 && e.h > 0.0)
            .map(|e| BodySize { w: e.w, h: e.h })
    }

    fn body_size(&self, art: Option<&DirSource>, stem: &str) -> Option<BodySize> {
        self.hub_size(stem).or_else(|| {
            let (art, file, cell) = (art?, self.files.get(stem)?, self.head_sizes.get(stem)?);
            if let Some(size) = cell.get() {
                return Some(*size);
            }
            let size = head_size(art, &format!("{CHARACTERS}{}/{file}", self.name))?;
            Some(*cell.get_or_init(|| size))
        })
    }

    fn hub_group(&self, body_stem: &str, body: &str) -> Option<&HubGroup> {
        self.hub
            .iter()
            .find(|g| g.sprites.iter().any(|s| s.name == body_stem))
            .or_else(|| {
                let m = body.parse::<usize>().ok().filter(|m| *m >= 1).unwrap_or(1);
                self.hub.get(m - 1)
            })
    }

    fn sprite(
        &self,
        art: Option<&DirSource>,
        name: &SpriteName<'_>,
        body: &str,
        body_stem: &str,
        body_file: &str,
        by_file_name: Option<&String>,
    ) -> CharacterSprite {
        let group = self.hub_group(body_stem, body);
        let hub_face = group
            .and_then(|g| g.face_stem(body_stem, name))
            .and_then(|s| self.files.get(s));
        let face = if name.alias.is_some() {
            hub_face
        } else {
            by_file_name.or(hub_face)
        };
        if let Some(file) = face {
            let stem = normalized_stem(file);
            if group.is_some_and(|g| g.is_whole(&stem)) {
                return CharacterSprite {
                    body_url: self.url(file),
                    face_url: None,
                    face_pos: None,
                    body_size: self.body_size(art, &stem),
                    plate: self.plate,
                };
            }
        }
        CharacterSprite {
            body_url: self.url(body_file),
            face_url: face.map(|f| self.url(f)),
            face_pos: face
                .and(group)
                .filter(|g| !g.is_sentinel())
                .map(|g| FacePos {
                    x: g.face_pos.x,
                    y: g.face_pos.y,
                    w: g.face_size.w,
                    h: g.face_size.h,
                }),
            body_size: self.body_size(art, body_stem),
            plate: self.plate,
        }
    }
}

#[derive(Debug, Default)]
pub struct StoryAssetIndex {
    backgrounds: HashMap<String, String>,
    images: HashMap<String, String>,
    background_sizes: HashMap<String, ImageSize>,
    image_sizes: HashMap<String, ImageSize>,
    characters: HashMap<String, SpriteFolder>,
    audio: HashMap<String, String>,
    videos: HashMap<String, VideoSources>,
    avatars: HashMap<String, String>,
    /// `act_archive*` spritepacks, then `textures/ui`
    archive_art: HashMap<String, String>,
    /// Paths under `sound_beta_2` in a `voice*` directory
    voices: HashSet<String>,
    variables: StoryVariables,
    /// The art tree sprite headers are read from
    art: Option<DirSource>,
}

fn normalized_stem(file_name: &str) -> String {
    let mut s = file_name.to_ascii_lowercase();
    while let Some(cut) = s.strip_suffix(".png").or_else(|| s.strip_suffix('.')) {
        s = cut.to_owned();
    }
    s
}

fn is_ext(name: &str, ext: &str) -> bool {
    name.rsplit_once('.')
        .is_some_and(|(_, e)| e.eq_ignore_ascii_case(ext))
}

fn file_stem(name: &str) -> &str {
    name.rsplit_once('.').map_or(name, |(s, _)| s)
}

fn png_size(head: &[u8]) -> Option<BodySize> {
    let head = head.get(..24)?;
    if &head[..8] != b"\x89PNG\r\n\x1a\n" || &head[12..16] != b"IHDR" {
        return None;
    }
    let dim = |o: usize| {
        let v = u32::from_be_bytes([head[o], head[o + 1], head[o + 2], head[o + 3]]);
        u16::try_from(v).ok().map(f32::from).filter(|n| *n > 0.0)
    };
    Some(BodySize {
        w: dim(16)?,
        h: dim(20)?,
    })
}

fn head_size(art: &dyn TableSource, rel: &str) -> Option<BodySize> {
    png_size(&art.read_asset_head(rel, PNG_HEAD_LEN).ok()??)
}

fn read_json<T: serde::de::DeserializeOwned>(src: &dyn TableSource, rel: &str) -> Option<T> {
    serde_json::from_slice(&src.read_asset(rel).ok()??).ok()
}

fn sprite_metas(art: &dyn TableSource, dir: &str) -> HashMap<String, SpriteMeta> {
    read_json::<HashMap<String, SpriteMeta>>(art, &format!("{dir}/sprites.json"))
        .unwrap_or_default()
        .into_iter()
        .filter(|(_, m)| m.w > 0.0 && m.h > 0.0 && m.ppu > 0.0)
        .map(|(k, m)| (k.to_ascii_lowercase(), m))
        .collect()
}

fn image_size(meta: Option<&SpriteMeta>, art: &dyn TableSource, rel: &str) -> Option<ImageSize> {
    if let Some(m) = meta {
        return Some(ImageSize {
            w: m.w,
            h: m.h,
            ppu: m.ppu,
            ppu_assumed: None,
        });
    }
    head_size(art, rel).map(|s| ImageSize {
        w: s.w,
        h: s.h,
        ppu: 100.0,
        ppu_assumed: Some(true),
    })
}

const AUDIO_ROOT: &str = "audio/audio/sound_beta_2/";
const CHARACTERS: &str = "textures/avg/characters/";
const SPRITEPACK: &str = "textures/spritepack/";

/// The trees story media lives in
const SUBTREES: [&str; 9] = [
    "textures/avg/bg",
    "textures/avg/backgrounds",
    "textures/avg/imgs",
    "textures/avg/items",
    "textures/avg/characters",
    "textures/spritepack",
    "textures/ui",
    "audio/audio/sound_beta_2",
    "video",
];

impl StoryAssetIndex {
    pub fn build(
        art: &DirSource,
        text: &dyn TableSource,
        list: &dyn Fn(&str) -> Vec<String>,
    ) -> io::Result<Self> {
        let paths: Vec<String> = SUBTREES
            .iter()
            .flat_map(|rel| list(rel).into_iter().map(move |p| format!("{rel}/{p}")))
            .collect();
        let mut idx = Self {
            art: Some(art.clone()),
            variables: text
                .read_asset("gamedata/story/story_variables.json.json")
                .ok()
                .flatten()
                .and_then(|b| StoryVariables::parse(&b).ok())
                .unwrap_or_default(),
            ..Self::default()
        };
        let mut metas: HashMap<&str, HashMap<String, SpriteMeta>> = HashMap::new();
        for (sub, background) in [
            ("bg", true),
            ("backgrounds", true),
            ("imgs", false),
            ("items", false),
        ] {
            let prefix = format!("textures/avg/{sub}/");
            for rel in paths.iter().filter(|p| p.starts_with(&prefix)) {
                let Some((dir, name)) = rel.rsplit_once('/') else {
                    continue;
                };
                if !is_ext(name, "png") || name.contains("[alpha]") {
                    continue;
                }
                let (map, sizes) = if background {
                    (&mut idx.backgrounds, &mut idx.background_sizes)
                } else {
                    (&mut idx.images, &mut idx.image_sizes)
                };
                let stem = normalized_stem(name);
                if map.contains_key(&stem) {
                    continue;
                }
                let meta = metas.entry(dir).or_insert_with(|| sprite_metas(art, dir));
                if let Some(size) = image_size(meta.get(&stem), art, rel) {
                    sizes.insert(stem.clone(), size);
                }
                map.insert(stem, url_of(rel));
            }
        }

        let mut folders: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for rel in &paths {
            if let Some((folder, file)) = rel
                .strip_prefix(CHARACTERS)
                .and_then(|r| r.split_once('/'))
                .filter(|(_, f)| !f.contains('/'))
            {
                folders.entry(folder).or_default().push(file);
            }
        }
        for (name, files) in folders {
            idx.characters
                .entry(name.to_ascii_lowercase())
                .or_insert_with(|| load_folder(art, name, &files));
        }

        for rel in &paths {
            let Some((pack, file)) = rel
                .strip_prefix(SPRITEPACK)
                .and_then(|r| r.split_once('/'))
                .filter(|(_, f)| !f.contains('/') && is_ext(f, "png"))
            else {
                continue;
            };
            let stem = normalized_stem(file);
            if pack.starts_with("ui_char_avatar_") || pack.starts_with("ui_player_avatar_list_") {
                idx.avatars.entry(stem).or_insert_with(|| url_of(rel));
            } else if pack.starts_with("cutin_char_") && !idx.images.contains_key(&stem) {
                let dir = &rel[..rel.len() - file.len() - 1];
                let meta = metas.entry(dir).or_insert_with(|| sprite_metas(art, dir));
                if let Some(size) = image_size(meta.get(&stem), art, rel) {
                    idx.image_sizes.insert(stem.clone(), size);
                }
                idx.images.insert(stem, url_of(rel));
            }
        }

        let archive_roots = paths
            .iter()
            .filter(|p| {
                p.strip_prefix(SPRITEPACK)
                    .is_some_and(|r| r.starts_with("act_archive"))
            })
            .chain(paths.iter().filter(|p| p.starts_with("textures/ui/")));
        for rel in archive_roots {
            let name = rel.rsplit('/').next().unwrap_or(rel);
            if is_ext(name, "png") {
                idx.archive_art
                    .entry(file_stem(name).to_ascii_lowercase())
                    .or_insert_with(|| url_of(rel));
            }
        }

        for rel in &paths {
            if let Some(sub) = rel.strip_prefix(AUDIO_ROOT) {
                let name = sub.rsplit('/').next().unwrap_or(sub);
                if !is_ext(name, "ogg") {
                    continue;
                }
                idx.audio
                    .entry(file_stem(name).to_ascii_lowercase())
                    .or_insert_with(|| url_of(rel));
                if sub.starts_with("voice") {
                    idx.voices.insert(sub.to_owned());
                }
            } else if let Some(sub) = rel.strip_prefix("video/") {
                let Some((key, ext)) = sub.rsplit_once('.') else {
                    continue;
                };
                let webm = ext.eq_ignore_ascii_case("webm");
                if !webm && !ext.eq_ignore_ascii_case("mp4") {
                    continue;
                }
                let sources = idx.videos.entry(key.to_ascii_lowercase()).or_default();
                let slot = if webm {
                    &mut sources.webm_url
                } else {
                    &mut sources.mp4_url
                };
                if slot.is_none() {
                    *slot = Some(url_of(rel));
                }
            }
        }
        Ok(idx)
    }

    fn resolve_as(&self, name: &str, primary: ImageSource) -> Option<(String, ImageSource)> {
        let key = name.trim().to_ascii_lowercase();
        let (first, second, fallback) = match primary {
            ImageSource::Background => (&self.backgrounds, &self.images, ImageSource::Image),
            ImageSource::Image => (&self.images, &self.backgrounds, ImageSource::Background),
        };
        first.get(&key).map_or_else(
            || second.get(&key).map(|u| (u.clone(), fallback)),
            |u| Some((u.clone(), primary)),
        )
    }

    fn size_as(&self, name: &str, primary: ImageSource) -> Option<ImageSize> {
        let key = name.trim().to_ascii_lowercase();
        let (first, second) = match primary {
            ImageSource::Background => (&self.background_sizes, &self.image_sizes),
            ImageSource::Image => (&self.image_sizes, &self.background_sizes),
        };
        first.get(&key).or_else(|| second.get(&key)).copied()
    }

    #[must_use]
    pub fn resolve_background(&self, name: &str) -> Option<(String, ImageSource)> {
        self.resolve_as(name, ImageSource::Background)
    }

    #[must_use]
    pub fn resolve_image(&self, name: &str) -> Option<(String, ImageSource)> {
        self.resolve_as(name, ImageSource::Image)
    }

    #[must_use]
    pub fn resolve_background_size(&self, name: &str) -> Option<ImageSize> {
        self.size_as(name, ImageSource::Background)
    }

    #[must_use]
    pub fn resolve_image_size(&self, name: &str) -> Option<ImageSize> {
        self.size_as(name, ImageSource::Image)
    }

    #[must_use]
    pub fn resolve_audio(&self, reference: &str) -> Option<String> {
        let logical = self.variables.resolve(reference)?;
        let base = logical.rsplit('/').next().unwrap_or(logical).trim();
        let key = base.to_ascii_lowercase();
        let key = key.strip_suffix(".ogg").unwrap_or(&key);
        if key.is_empty() {
            return None;
        }
        self.audio.get(key).cloned()
    }

    /// A `[Video]` `res` (`video/act38side/PV01.mp4`) to both transcodes
    #[must_use]
    pub fn resolve_video(&self, reference: &str) -> Option<VideoSources> {
        let lowered = reference
            .trim()
            .trim_start_matches('/')
            .to_ascii_lowercase();
        let rest = lowered.strip_prefix("video/").unwrap_or(&lowered);
        let name_at = rest.rfind('/').map_or(0, |i| i + 1);
        let key = rest[name_at..]
            .rfind('.')
            .map_or(rest, |dot| &rest[..name_at + dot]);
        if key.is_empty() {
            return None;
        }
        self.videos.get(key).cloned()
    }

    /// A `[popupdialog]` head: the bare id, its `_1`/`_2`/`_1+` arts, then the first stem under it
    #[must_use]
    pub fn resolve_avatar(&self, reference: &str) -> Option<String> {
        let logical = self
            .variables
            .resolve(reference)?
            .trim()
            .to_ascii_lowercase();
        if logical.is_empty() {
            return None;
        }
        for suffix in ["", "_1", "_2", "_1+"] {
            if let Some(u) = self.avatars.get(&format!("{logical}{suffix}")) {
                return Some(u.clone());
            }
        }
        let prefix = format!("{logical}_");
        self.avatars
            .iter()
            .filter(|(stem, _)| stem.starts_with(&prefix))
            .min_by_key(|(stem, _)| *stem)
            .map(|(_, u)| u.clone())
    }

    /// Archive plate, masthead, or picture
    #[must_use]
    pub fn archive_art(&self, stem: &str) -> Option<&str> {
        self.archive_art
            .get(&stem.trim().to_ascii_lowercase())
            .map(String::as_str)
    }

    #[must_use]
    pub fn has_voice(&self, dir: &str, char_id: &str, voice_id: &str) -> bool {
        self.voices
            .contains(&format!("{dir}/{char_id}/{voice_id}.ogg"))
    }

    fn sprite_folder(&self, base: &str) -> Option<&SpriteFolder> {
        let key = base.to_ascii_lowercase();
        if key.is_empty() {
            return None;
        }
        self.characters
            .get(&key)
            .or_else(|| self.characters.get(&format!("{key}_1")))
    }

    #[must_use]
    pub fn sprite_folder_name(&self, raw: &str) -> Option<&str> {
        self.sprite_folder(parse_sprite_name(raw).base)
            .map(|f| f.name.as_str())
    }

    #[must_use]
    pub fn sprite_folder_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.characters.values().map(|f| f.name.as_str()).collect();
        names.sort_unstable();
        names
    }

    /// Every `#N$M` a folder offers by its hub, else one `#N$1` per PNG
    #[must_use]
    pub fn sprite_variant_keys(&self, folder: &str) -> Vec<String> {
        let Some(f) = self.sprite_folder(folder) else {
            return Vec::new();
        };
        if f.hub.is_empty() {
            return (1..=f.files.len().max(1))
                .map(|n| format!("#{n}$1"))
                .collect();
        }
        f.hub
            .iter()
            .enumerate()
            .flat_map(|(k, g)| (1..=g.sprites.len().max(1)).map(move |n| format!("#{n}${}", k + 1)))
            .collect()
    }

    /// `base#face$body` to its body and face files
    #[must_use]
    pub fn resolve_character(&self, raw: &str) -> Option<CharacterSprite> {
        let name = parse_sprite_name(raw);
        let folder = self.sprite_folder(name.base)?;
        let face = trim_index(name.face.unwrap_or("1"));
        let body = trim_index(name.body.unwrap_or("1"));
        let fname = folder.name.to_ascii_lowercase();
        let stem = fname
            .rfind('_')
            .filter(|&i| i + 1 < fname.len() && fname[i + 1..].bytes().all(|b| b.is_ascii_digit()))
            .map_or(fname.as_str(), |i| &fname[..i]);

        let hub_body = format!("{fname}${body}");
        if let Some(file) = folder.files.get(&hub_body) {
            let by_name = folder
                .files
                .get(&format!("{face}${body}"))
                .or_else(|| folder.files.get(&format!("{face}$1")));
            return Some(folder.sprite(self.art.as_ref(), &name, body, &hub_body, file, by_name));
        }

        let mut by_face = vec![
            format!("{stem}_{face}"),
            format!("{fname}_{face}"),
            format!("{fname}#{face}"),
            format!("{fname}_1#{face}"),
        ];
        if face == "1" {
            by_face.push(fname.clone());
        }
        let defaults = [
            format!("{stem}_1"),
            fname.clone(),
            format!("{fname}_1"),
            format!("{fname}$1"),
            format!("{fname}#1"),
            format!("{fname}_1#1"),
        ];
        let slot = face.parse::<usize>().ok().and_then(|n| n.checked_sub(1));
        let legacy_hub_order = folder
            .hub
            .first()
            .filter(|g| {
                folder.hub.len() == 1
                    && g.is_sentinel()
                    && g.sprites.len() >= 2
                    && g.sprites
                        .iter()
                        .all(|s| !s.name.bytes().all(|b| b.is_ascii_digit()))
            })
            .and_then(|g| folder.files.get(g.sprites.get(slot?)?.name.as_str()));
        let hub_fallback = || {
            let g = folder.hub.first()?;
            g.sprites
                .get(slot.unwrap_or(0))
                .into_iter()
                .chain(g.sprites.first())
                .find_map(|s| folder.files.get(s.name.as_str()))
        };
        let file = legacy_hub_order
            .or_else(|| {
                by_face
                    .iter()
                    .chain(&defaults)
                    .find_map(|c| folder.files.get(c))
            })
            .or_else(hub_fallback)?;
        Some(folder.sprite(
            self.art.as_ref(),
            &name,
            body,
            &normalized_stem(file),
            file,
            folder.files.get(face),
        ))
    }
}

fn load_folder(art: &dyn TableSource, name: &str, files: &[&str]) -> SpriteFolder {
    let mut folder = SpriteFolder {
        name: name.to_owned(),
        ..SpriteFolder::default()
    };
    let mut pngs: Vec<&str> = files
        .iter()
        .copied()
        .filter(|f| is_ext(f, "png") && !f.contains("[alpha]") && normalized_stem(f) != "alpha")
        .collect();
    pngs.sort_unstable();
    for file in pngs {
        folder
            .files
            .entry(normalized_stem(file))
            .or_insert_with(|| file.to_owned());
    }
    if files.contains(&"hub.json")
        && let Some(hub) = read_json::<HubFile>(art, &format!("{CHARACTERS}{name}/hub.json"))
    {
        folder.hub = hub.groups;
        for s in folder.hub.iter_mut().flat_map(|g| &mut g.sprites) {
            s.name.make_ascii_lowercase();
            s.alias.make_ascii_lowercase();
        }
        folder.plate = hub.root.filter(|r| r.w > 0.0 && r.h > 0.0);
    }
    folder.head_sizes = folder
        .files
        .keys()
        .filter(|stem| folder.hub_size(stem).is_none())
        .map(|stem| (stem.clone(), OnceLock::new()))
        .collect();
    folder
}
