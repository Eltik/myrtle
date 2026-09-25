//! Story asset resolution: a script's raw names -> served URLs.
//!
//! Built once per server on first use (see [`StoryAssetIndex::for_dir`]) from
//! four walks under the server's assets root:
//!
//! * `textures/avg/bg/**/*.png`: backgrounds, keyed by lowercase stem (946 on EN).
//! * `textures/avg/imgs/**/*.png` and `textures/avg/items/**/*.png`: CGs and
//!   item cards, keyed by lowercase stem (1,551 + 96 on EN).
//! * `textures/avg/characters/<sprite>/`: one entry per folder, with the
//!   folder's PNGs keyed by lowercase stem (`[alpha]` companions dropped).
//! * `audio/audio/sound_beta_2/**/*.ogg`: every clip by lowercase stem. Music
//!   lives at `music/<act>/<lowercased base>/<Name>_loop.ogg`, one directory
//!   deeper than the variable's path says, so audio resolves by basename.
//! * `video/**/*.{webm,mp4}`: cutscene clips, keyed by the relative path with
//!   the extension cut and lowercased (`act38side/pv01`). 16 clips on EN, each
//!   transcoded to both containers, so 32 files.
//!
//! Four of those 16 clips (`mixstory/bg_mainline_0..3`) are referenced by no
//! script. Of the 25 distinct `res` values the EN scripts reference over 26
//! `[Video]` commands, 12 have a file and 13 do not, because EN retired those
//! clips; a reference with no file resolves to nothing and the reader skips it.
//!
//! URLs are paths RELATIVE TO THE ASSETS ROOT with a leading slash
//! (`/textures/avg/bg/<hub>/bg_cher_1.png`), the same form `AssetIndex` emits
//! and `frontend/.../assets.ts::asset()` prefixes with `/api/assets`. A `#` in
//! a filename is written `%23` so it survives as a path character.
//!
//! Sprite names are `base#face$body`, both suffixes optional. The folder is
//! `base` (case-insensitive) or `base_1`. The file rules, in the order tried:
//!
//! 1. Hub body `<folder>$<body>.png` (1,407 `avg_*` folders on EN), with the
//!    face patch `<face>$<body>.png`, else `<face>$1.png`.
//! 2. Whole sprite by face: `<stem>_<face>.png` where `stem` is the folder
//!    with its trailing `_N` cut (`char_002_amiya_1#5` -> `char_002_amiya_5`),
//!    then `<folder>_<face>.png`, then `<folder>#<face>.png`
//!    (`char_362_saga#6`), and for face 1 the folder's own `<folder>.png`.
//!    also `<folder>_1#<face>.png` (11 folders such as `avg_npc_135`).
//! 3. The folder's default body `<stem>_1.png`, `<folder>.png`,
//!    `<folder>_1.png`, `<folder>$1.png`, `<folder>#1.png`, `<folder>_1#1.png`.
//!    On either body the OLD hub layout's bare `<face>.png` patch rides along
//!    when the folder has one (`avg_1013_spchen_1`: a 1024px
//!    `avg_1013_spchen_1.png` and 64px `1.png`..`7.png`; 56 folders, plus 5
//!    `avg_npc_139` style folders whose body is `<folder>#1.png`).
//!    No candidate means no sprite: a folder's first PNG is never served as a
//!    body, because in these folders it is a face patch.
//!
//! Face and body indices are read with leading zeros cut (`#01$1` is face 1;
//! 2 names in the EN scripts write them that way). `[alpha]` companions and
//! the bare `alpha.png` (54 old-hub folders) are never candidates.
//!
//! WHERE the face patch sits comes from `hub.json`, which the unpacker writes
//! beside the PNGs out of the bundle's `AVGCharacterSpriteHub(Group)`
//! `MonoBehaviour` (`assets/unpacker/src/export/avg_hub.rs`). The `$M` body
//! index picks the group, the group's `facePos`/`faceSize` become the wire's
//! `facePos {x,y,w,h}` in body pixels, and the sprite list is what `@alias`
//! and an index whose file name the rules above miss resolve through. A
//! folder with no `hub.json` resolves exactly as it did before, by file name
//! alone, and simply carries no `facePos`.
//!
//! `facePos` is measured in the BODY TEXTURE's own pixels, which are NOT the
//! 1024 canvas px the stage draws a body into: `avg_1037_amiya3_1$1` is a
//! 1280x1280 texture, so its `facePos (570,233)` is 570/1280 = 0.44531 of the
//! plate and not 570/1024 = 0.55664. `bodySize` therefore rides beside it on
//! every resolved sprite, read from the hub entry's `size` when the unpacker
//! wrote one and from the PNG's own IHDR otherwise.
//!
//! HOW BIG the plate is drawn is the hub's `root`, on the wire as `plate`: the
//! character's own prefab root rect beats the 1024 slot template, measured at
//! Amiya 1091.7 canvas px against her bundle's 1090 while Dobermann in the
//! same frame measured 958.9 (`docs/story-reader-captures.md` section 2). It
//! is per FOLDER, not per sprite, because the rect is the prefab's.
//!
//! The image trees carry the same kind of file, `sprites.json`, and for the
//! same reason: a `[Background]` with no `screenadapt` is `SetNativeSize` at
//! the sprite's OWN pixels-per-unit against the canvas scaler's reference 100,
//! so `bg_cher_1` at `rect (1024,576)` and `ppu 68.2464` draws 1500.4 x 844.0
//! canvas px and not 1024 x 576. The PPU varies per sprite (68.25, 80, 64 and
//! 100 all ship), so `imageSizes` carries it for every referenced name, from
//! `sprites.json` when the unpacker wrote one and from the PNG's own IHDR at
//! an ASSUMED `ppu` of 100 otherwise, flagged `ppuAssumed`.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::variables::StoryVariables;
use crate::core::gamedata::assets::AssetIndex;

/// Where a face patch sits on its body, in BODY PIXELS: `x`,`y` are the
/// patch's top-left corner measured from the body's left and TOP edge, `w`,`h`
/// the on-body size the face texture is scaled into. Haak reads
/// `{x: 512, y: 120, w: 51, h: 71}` from the hub's `facePos (512,120,0)` /
/// `faceSize (51,71)`. Absent on a legacy hub's `(-1,-1)`/`(0,0)` sentinel, on
/// a whole-body sprite, and on any folder whose `hub.json` the unpacker has
/// not written yet.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct FacePos {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// The body TEXTURE's size in pixels, which is the unit `facePos` is measured
/// in. 1024 for most bodies, 1280 for the oversized ones, and a face patch's
/// fraction of the plate is `facePos / bodySize` in both cases. Absent only
/// when neither the hub nor the PNG header could be read.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct BodySize {
    pub w: f32,
    pub h: f32,
}

/// The plate a character is drawn into, in CANVAS px: the prefab root rect the
/// bundle ships, `x`,`y` its anchored position over the slot origin and
/// `w`,`h` its size. Amiya legacy reads `{x: 0, y: 203, w: 1090, h: 1090}`,
/// Haak `{x: 0, y: 150, w: 955, h: 955}`. It WINS over the slot template's
/// 1024 at y 203, so two characters in one frame are drawn at different sizes.
/// Absent when the folder has no `hub.json`, or one written before the
/// unpacker read the root rect.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct PlateRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// A background or CG plate's own metrics: `w`,`h` the sprite rect in texture
/// px and `ppu` its pixels-per-unit. A command with no `screenadapt` draws it
/// `w * 100 / ppu` by `h * 100 / ppu` canvas px, 100 being the reference
/// pixels-per-unit every AVG `CanvasScaler` carries. `ppu_assumed` says the
/// 100 is a FALLBACK, not a read: the size came from the PNG header because no
/// `sprites.json` covered that name.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ImageSize {
    pub w: f32,
    pub h: f32,
    pub ppu: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub ppu_assumed: Option<bool>,
}

/// A resolved character sprite.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct CharacterSprite {
    pub body_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub face_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub face_pos: Option<FacePos>,
    /// The `body_url` texture's own size. `face_pos` divided by THIS is the
    /// patch's fraction of the body plate.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub body_size: Option<BodySize>,
    /// The canvas rect the whole plate is drawn into, from the character's own
    /// prefab root. It is the bundle's, so every sprite of a folder shares it.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub plate: Option<PlateRect>,
}

/// A music cue: the intro plays once, the loop repeats.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MusicCue {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub intro: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub r#loop: Option<String>,
}

/// The two transcodes of one cutscene clip. `webm` is offered first and `mp4`
/// is the fallback; a clip is only ever missing one of them when a transcode
/// failed, since the unpacker's video step (`backfill-video`, and the tail of
/// every `extract`) writes both.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct VideoSources {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub webm_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub mp4_url: Option<String>,
}

/// The assets one script references, keyed by the raw name as written in the
/// script (`avg_225_haak_1#3$1`, `$escape_loop`). Only resolved names appear.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StoryAssets {
    pub backgrounds: BTreeMap<String, String>,
    pub images: BTreeMap<String, String>,
    pub characters: BTreeMap<String, CharacterSprite>,
    pub music: BTreeMap<String, MusicCue>,
    pub sounds: BTreeMap<String, String>,
    /// `[popupdialog]` head portraits, keyed by the raw `dialoghead` value
    /// (`$avatar_amiya`). 163 distinct values over EN, all 163 resolve.
    pub avatars: BTreeMap<String, String>,
    /// Every name in `backgrounds` and `images`, under the SAME key, with the
    /// plate metrics that size it when the command carries no `screenadapt`.
    pub image_sizes: BTreeMap<String, ImageSize>,
    /// Cutscene clips, keyed by the `[Video]` command's own `res` argument
    /// (`video/act38side/PV01.mp4`) so the engine looks a halt up by the
    /// string it already holds. Only resolved references appear: 12 of the 25
    /// distinct names the EN scripts reference have a file, the other 13 are
    /// clips EN retired and the reader skips them.
    pub videos: BTreeMap<String, VideoSources>,
}

/// A sprite name split into its parts. `face` and `body` are kept as written
/// (`"11"`), since they are file name pieces, not numbers to compute with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteName<'a> {
    pub base: &'a str,
    pub face: Option<&'a str>,
    pub body: Option<&'a str>,
    /// The `@alias` form (`_TryParseAlias`, `LastIndexOf('@')`), which names a
    /// face through the hub group's `sprites[].alias` instead of by index.
    pub alias: Option<&'a str>,
}

/// `base#face$body` -> parts, with `@alias` in place of `#face`. Any suffix
/// may be absent; the client strips them body, alias, index, so the parts are
/// split on the first of each marker.
#[must_use]
pub fn parse_sprite_name(raw: &str) -> SpriteName<'_> {
    const MARKERS: [char; 3] = ['#', '$', '@'];
    let name = raw.trim();
    let (base, rest) = match name.find(MARKERS) {
        Some(i) => (&name[..i], &name[i..]),
        None => (name, ""),
    };
    let mut face = None;
    let mut body = None;
    let mut alias = None;
    let mut rest = rest;
    while !rest.is_empty() {
        let marker = rest.as_bytes()[0];
        let tail = &rest[1..];
        let end = tail.find(MARKERS).unwrap_or(tail.len());
        let value = tail[..end].trim();
        match marker {
            b'#' if face.is_none() => face = Some(value),
            b'$' if body.is_none() => body = Some(value),
            b'@' if alias.is_none() => alias = Some(value),
            _ => {}
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

#[derive(Debug, Default)]
struct SpriteFolder {
    /// The folder name as it is on disk.
    name: String,
    /// The folder on disk, so a body with no hub `size` can be measured from
    /// its own PNG header.
    dir: PathBuf,
    /// Lowercase stem (extension and any `.png.png` / `..png` doubling cut) -> file name on disk.
    files: HashMap<String, String>,
    /// The bundle's `AVGCharacterSpriteHub(Group)`, as the unpacker wrote it
    /// to `hub.json` beside the PNGs. Empty when that file is absent, which
    /// is every folder of an output tree extracted before this landed.
    hub: Vec<HubGroup>,
    /// The prefab's own root rect from the same file, shared by every sprite
    /// in the folder.
    plate: Option<PlateRect>,
}

/// One sprite entry of a hub group, in the order the script's `#N` / `$M`
/// decrement into. Only the three fields the resolver reads are kept.
#[derive(Debug, Clone, Deserialize)]
struct HubSprite {
    /// The exported PNG's stem (`1$1`), lowercased on load.
    #[serde(default)]
    name: String,
    /// The authored alias a `@alias` addresses, lowercased on load.
    #[serde(default)]
    alias: String,
    #[serde(default, rename = "isWholeBody")]
    is_whole_body: bool,
    /// The Sprite's `m_Rect` size in texture pixels, as the unpacker reads it
    /// out of the bundle. Absent on a `hub.json` written before that field.
    #[serde(default)]
    size: Option<HubExtent>,
}

/// One body's sprite list and the single face placement that serves all of
/// them. The group is chosen by the `$M` body index.
#[derive(Debug, Clone, Deserialize)]
struct HubGroup {
    #[serde(rename = "facePos")]
    face_pos: HubPoint,
    #[serde(rename = "faceSize")]
    face_size: HubExtent,
    #[serde(default)]
    sprites: Vec<HubSprite>,
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
struct HubFile {
    #[serde(default)]
    groups: Vec<HubGroup>,
    /// The character prefab's root rect in canvas px. Absent on a `hub.json`
    /// written before the unpacker read it.
    #[serde(default)]
    root: Option<PlateRect>,
}

/// One entry of a folder's `sprites.json`, as the unpacker's image export
/// writes it (`assets/unpacker/src/export/avg_sprites.rs`).
#[derive(Debug, Clone, Copy, Deserialize)]
struct SpriteMetaFile {
    w: f32,
    h: f32,
    ppu: f32,
}

impl HubGroup {
    /// The `FacePos (-1,-1)` / `FaceSize (0,0)` sentinel: whole bodies with
    /// nothing placed on them, so no `facePos` goes on the wire.
    fn is_sentinel(&self) -> bool {
        self.face_size.w <= 0.0
            || self.face_size.h <= 0.0
            || self.face_pos.x < 0.0
            || self.face_pos.y < 0.0
    }

    const fn face_pos(&self) -> FacePos {
        FacePos {
            x: self.face_pos.x,
            y: self.face_pos.y,
            w: self.face_size.w,
            h: self.face_size.h,
        }
    }

    fn index_of(&self, stem: &str) -> Option<usize> {
        self.sprites.iter().position(|s| s.name == stem)
    }
}

/// Which side of an image lookup answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageSource {
    Background,
    Image,
}

impl ImageSource {
    /// The other tree, which is the fallback half of every image lookup.
    const fn fallback(self) -> Self {
        match self {
            Self::Background => Self::Image,
            Self::Image => Self::Background,
        }
    }
}

/// Assets dir -> (the live `AssetIndex` the build was validated against, the build).
type AssetCacheMap = HashMap<PathBuf, (Weak<AssetIndex>, Arc<StoryAssetIndex>)>;

#[derive(Debug, Default)]
pub struct StoryAssetIndex {
    backgrounds: HashMap<String, String>,
    images: HashMap<String, String>,
    /// The same keys as `backgrounds` and `images`, each with the plate
    /// metrics that size it: the sprite's own rect and pixels-per-unit. Two
    /// maps rather than one, because a name in BOTH trees resolves to
    /// whichever the lookup order picks and must be sized by that one.
    background_sizes: HashMap<String, ImageSize>,
    image_sizes: HashMap<String, ImageSize>,
    characters: HashMap<String, SpriteFolder>,
    audio: HashMap<String, String>,
    /// Cutscene clips under `video/`, keyed by the relative path with the
    /// extension cut and lowercased (`act38side/pv01`, `01`). 16 clips on EN,
    /// each with a `.webm` and an `.mp4`.
    videos: HashMap<String, VideoSources>,
    /// Lowercased avatar file stem -> served path, over every
    /// `textures/spritepack/ui_char_avatar_*` folder (1,994 stems on EN).
    avatars: HashMap<String, String>,
    variables: StoryVariables,
    /// How long the walks took, for the boot log and the phase report.
    pub build_ms: u128,
}

fn encode_path_piece(s: &str) -> String {
    s.replace('#', "%23")
}

/// Lowercase stem with the unpacker's doubled extensions cut (`1$1.png.png`,
/// `1$1..png` both key as `1$1`).
fn normalized_stem(file_name: &str) -> String {
    let mut s = file_name.to_ascii_lowercase();
    loop {
        if let Some(cut) = s.strip_suffix(".png") {
            s = cut.to_owned();
        } else if let Some(cut) = s.strip_suffix('.') {
            s = cut.to_owned();
        } else {
            break;
        }
    }
    s
}

/// `x[alpha].png` beside `x.png`, or the old hub layout's bare `alpha.png`:
/// alpha planes the served PNGs already carry, never a body or a face.
fn is_alpha_companion(file_name: &str) -> bool {
    file_name.contains("[alpha]") || normalized_stem(file_name) == "alpha"
}

/// A face or body index with leading zeros cut (`01` -> `1`); all zeros stay `0`.
/// A sprite index with its leading zeros trimmed (`"01"` -> `"1"`), which is
/// how the client reads one. `"0"` stays `"0"`.
#[must_use]
pub(crate) fn trim_index(raw: &str) -> &str {
    let t = raw.trim_start_matches('0');
    if t.is_empty() && !raw.is_empty() {
        "0"
    } else {
        t
    }
}

fn is_png(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("png"))
}

/// A PNG's pixel size straight out of its IHDR: the 8-byte signature, a
/// 4-byte chunk length and the `IHDR` tag, then width and height as
/// big-endian `u32` at offsets 16 and 20. 24 bytes off the front of the file,
/// no decode, so it is cheap enough to run on a body that the hub does not
/// size. `None` on anything that is not a PNG or is larger than 65,535 px.
fn png_size(path: &Path) -> Option<BodySize> {
    use std::io::Read;
    let mut head = [0u8; 24];
    std::fs::File::open(path).ok()?.read_exact(&mut head).ok()?;
    if &head[..8] != b"\x89PNG\r\n\x1a\n" || &head[12..16] != b"IHDR" {
        return None;
    }
    let dim = |o: usize| -> Option<f32> {
        let v = u32::from_be_bytes([head[o], head[o + 1], head[o + 2], head[o + 3]]);
        u16::try_from(v).ok().map(f32::from).filter(|n| *n > 0.0)
    };
    Some(BodySize {
        w: dim(16)?,
        h: dim(20)?,
    })
}

/// The sprite folder's `hub.json`, written by the unpacker's texture export
/// (`assets/unpacker/src/export/avg_hub.rs`). A folder without one resolves
/// exactly as it did before, by file name alone: the hub adds `facePos`, the
/// `@alias` form and a face for an index whose file name the rules miss, and
/// takes nothing away. Sprite names and aliases are lowercased on load so
/// every comparison downstream is on the same footing as the file stems.
fn load_hub(path: &Path) -> (Vec<HubGroup>, Option<PlateRect>) {
    // The kill switch, verified inert: with `STORY_NO_HUB=1` the whole-corpus
    // test reports the same 8,841 of 8,853 names resolved and the same 7,151
    // with a face patch as before the hub existed, and 0 `facePos`.
    if std::env::var_os("STORY_NO_HUB").is_some() {
        return (Vec::new(), None);
    }
    let Ok(text) = std::fs::read_to_string(path) else {
        return (Vec::new(), None);
    };
    let Ok(file) = serde_json::from_str::<HubFile>(&text) else {
        tracing::warn!(path = %path.display(), "hub.json did not parse; sprite resolves by file name only");
        return (Vec::new(), None);
    };
    let mut groups = file.groups;
    for g in &mut groups {
        for sprite in &mut g.sprites {
            sprite.name = sprite.name.to_ascii_lowercase();
            sprite.alias = sprite.alias.to_ascii_lowercase();
        }
    }
    // A zero-sized root is no rect at all: the unpacker skips a stretched one,
    // and a hub written before it read them carries none.
    let plate = file.root.filter(|r| r.w > 0.0 && r.h > 0.0);
    (groups, plate)
}

/// A folder's `sprites.json`, keyed by lowercase sprite name so it lines up
/// with the PNG stems every other lookup here uses. Empty when the file is
/// absent, which is every folder of a tree extracted before it landed: those
/// names fall back to the PNG header at an assumed `ppu` of 100.
fn load_sprite_meta(dir: &Path) -> HashMap<String, SpriteMetaFile> {
    let Ok(text) = std::fs::read_to_string(dir.join("sprites.json")) else {
        return HashMap::new();
    };
    let Ok(file) = serde_json::from_str::<HashMap<String, SpriteMetaFile>>(&text) else {
        tracing::warn!(dir = %dir.display(), "sprites.json did not parse; plates size from the PNG header");
        return HashMap::new();
    };
    file.into_iter()
        .filter(|(_, m)| m.w > 0.0 && m.h > 0.0 && m.ppu > 0.0)
        .map(|(k, m)| (k.to_ascii_lowercase(), m))
        .collect()
}

/// A plate's metrics: the sprite's own rect and PPU when the unpacker wrote
/// them, else the PNG's own IHDR at the reference PPU of 100, flagged as the
/// assumption it is.
fn image_size(meta: Option<&SpriteMetaFile>, path: &Path) -> Option<ImageSize> {
    if let Some(m) = meta {
        return Some(ImageSize {
            w: m.w,
            h: m.h,
            ppu: m.ppu,
            ppu_assumed: None,
        });
    }
    png_size(path).map(|s| ImageSize {
        w: s.w,
        h: s.h,
        ppu: 100.0,
        ppu_assumed: Some(true),
    })
}

impl StoryAssetIndex {
    /// Build the index for a server's assets root. Walks four trees; on EN
    /// the whole thing is one pass over ~17k PNGs and ~73k OGGs.
    #[must_use]
    pub fn build(server_assets_dir: &Path) -> Self {
        let started = Instant::now();
        let mut idx = Self {
            variables: StoryVariables::load(server_assets_dir),
            ..Self::default()
        };

        // One `sprites.json` per folder, read once however many PNGs sit in
        // it, and shared by the `avg` trees and the cutin packs below.
        let mut sprite_meta: HashMap<PathBuf, HashMap<String, SpriteMetaFile>> = HashMap::new();

        let avg = server_assets_dir.join("textures/avg");
        for (sub, target) in [("bg", 0), ("backgrounds", 0), ("imgs", 1), ("items", 1)] {
            let root = avg.join(sub);
            if !root.is_dir() {
                continue;
            }
            for entry in walkdir::WalkDir::new(&root)
                .min_depth(1)
                .sort_by_file_name()
            {
                let Ok(entry) = entry else { continue };
                let path = entry.path();
                if !is_png(path) {
                    continue;
                }
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if name.contains("[alpha]") {
                    continue;
                }
                let Some(rel) = path
                    .strip_prefix(server_assets_dir)
                    .ok()
                    .and_then(Path::to_str)
                else {
                    continue;
                };
                let url = format!("/{}", encode_path_piece(rel));
                let (map, sizes) = if target == 0 {
                    (&mut idx.backgrounds, &mut idx.background_sizes)
                } else {
                    (&mut idx.images, &mut idx.image_sizes)
                };
                let stem = normalized_stem(name);
                if map.contains_key(&stem) {
                    continue;
                }
                let dir = path.parent().unwrap_or(&root).to_path_buf();
                let metas = sprite_meta
                    .entry(dir)
                    .or_insert_with_key(|d| load_sprite_meta(d));
                if let Some(size) = image_size(metas.get(&stem), path) {
                    sizes.insert(stem.clone(), size);
                }
                map.insert(stem, url);
            }
        }

        let chars = avg.join("characters");
        if let Ok(dirs) = std::fs::read_dir(&chars) {
            let mut dirs: Vec<_> = dirs.flatten().collect();
            dirs.sort_by_key(std::fs::DirEntry::file_name);
            for dir in dirs {
                let Ok(folder_name) = dir.file_name().into_string() else {
                    continue;
                };
                if !dir.path().is_dir() {
                    continue;
                }
                let mut folder = SpriteFolder {
                    name: folder_name.clone(),
                    dir: dir.path(),
                    ..SpriteFolder::default()
                };
                let Ok(files) = std::fs::read_dir(dir.path()) else {
                    continue;
                };
                let mut names: Vec<String> = files
                    .flatten()
                    .filter(|f| is_png(&f.path()))
                    .filter_map(|f| f.file_name().into_string().ok())
                    .filter(|n| !is_alpha_companion(n))
                    .collect();
                names.sort();
                for n in names {
                    folder.files.entry(normalized_stem(&n)).or_insert(n);
                }
                let (groups, plate) = load_hub(&dir.path().join("hub.json"));
                folder.hub = groups;
                folder.plate = plate;
                idx.characters
                    .entry(folder_name.to_ascii_lowercase())
                    .or_insert(folder);
            }
        }

        // `[popupdialog(dialoghead="$avatar_amiya")]` draws an operator head:
        // the same `ui_char_avatar_*` sprite packs the rest of the site serves.
        let spritepack = server_assets_dir.join("textures/spritepack");
        if let Ok(dirs) = std::fs::read_dir(&spritepack) {
            for dir in dirs.flatten() {
                let Some(name) = dir.file_name().to_str().map(str::to_owned) else {
                    continue;
                };
                let is_avatar = name.starts_with("ui_char_avatar_")
                    || name.starts_with("ui_player_avatar_list_");
                // `[interlude(name="cutin_char_20")]` names a plate that lives
                // in `spritepack/cutin_char_*`, not under `textures/avg`.
                let is_cutin = name.starts_with("cutin_char_");
                if !is_avatar && !is_cutin {
                    continue;
                }
                let Ok(files) = std::fs::read_dir(dir.path()) else {
                    continue;
                };
                for file in files.flatten() {
                    let path = file.path();
                    if !is_png(&path) {
                        continue;
                    }
                    let Some(fname) = path.file_name().and_then(|s| s.to_str()) else {
                        continue;
                    };
                    let url = format!(
                        "/textures/spritepack/{}/{}",
                        encode_path_piece(&name),
                        encode_path_piece(fname)
                    );
                    let stem = normalized_stem(fname);
                    if is_avatar {
                        idx.avatars.entry(stem).or_insert(url);
                    } else if !idx.images.contains_key(&stem) {
                        let metas = sprite_meta
                            .entry(dir.path())
                            .or_insert_with_key(|d| load_sprite_meta(d));
                        if let Some(size) = image_size(metas.get(&stem), &path) {
                            idx.image_sizes.insert(stem.clone(), size);
                        }
                        idx.images.insert(stem, url);
                    }
                }
            }
        }

        let audio_root = server_assets_dir.join("audio/audio/sound_beta_2");
        for entry in walkdir::WalkDir::new(&audio_root)
            .min_depth(1)
            .sort_by_file_name()
        {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            if !path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("ogg"))
            {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let Some(rel) = path
                .strip_prefix(server_assets_dir)
                .ok()
                .and_then(Path::to_str)
            else {
                continue;
            };
            idx.audio
                .entry(stem.to_ascii_lowercase())
                .or_insert_with(|| format!("/{}", encode_path_piece(rel)));
        }

        let video_root = server_assets_dir.join("video");
        for entry in walkdir::WalkDir::new(&video_root)
            .min_depth(1)
            .sort_by_file_name()
        {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(str::to_ascii_lowercase);
            let Some(ext) = ext else { continue };
            if ext != "webm" && ext != "mp4" {
                continue;
            }
            let stemmed = path.with_extension("");
            let Some(key) = stemmed
                .strip_prefix(&video_root)
                .ok()
                .and_then(Path::to_str)
                .map(str::to_ascii_lowercase)
            else {
                continue;
            };
            let Some(rel) = path
                .strip_prefix(server_assets_dir)
                .ok()
                .and_then(Path::to_str)
            else {
                continue;
            };
            let url = format!("/{}", encode_path_piece(rel));
            let sources = idx.videos.entry(key).or_default();
            let slot = if ext == "webm" {
                &mut sources.webm_url
            } else {
                &mut sources.mp4_url
            };
            if slot.is_none() {
                *slot = Some(url);
            }
        }

        idx.build_ms = started.elapsed().as_millis();
        tracing::info!(
            dir = %server_assets_dir.display(),
            backgrounds = idx.backgrounds.len(),
            images = idx.images.len(),
            sprites = idx.characters.len(),
            audio = idx.audio.len(),
            videos = idx.videos.len(),
            variables = idx.variables.len(),
            ms = idx.build_ms,
            "story asset index built"
        );
        idx
    }

    /// The index for a server, built on first use and shared after. Keyed by
    /// the assets directory and VALIDATED against the server's live
    /// [`AssetIndex`]: a hot reload (`swap_asset_index` after a re-extract)
    /// stores a new `Arc`, the `Weak` held here no longer upgrades to it, and
    /// the next request rebuilds. A `Weak` rather than the pointer, so a
    /// freed-and-reused allocation cannot match a stale entry.
    pub fn for_dir(server_assets_dir: &Path, live: &Arc<AssetIndex>) -> Arc<Self> {
        static CACHE: OnceLock<Mutex<AssetCacheMap>> = OnceLock::new();
        let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        let key = server_assets_dir.to_path_buf();
        if let Some((seen, hit)) = cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&key)
            && seen.upgrade().is_some_and(|old| Arc::ptr_eq(&old, live))
        {
            return Arc::clone(hit);
        }
        // Built outside the lock: a second caller may build too, and the last
        // insert wins. That costs one duplicate walk at most, never a stall on
        // every other server's lookup while this one builds.
        let built = Arc::new(Self::build(server_assets_dir));
        cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(key, (Arc::downgrade(live), Arc::clone(&built)));
        built
    }

    #[must_use]
    pub const fn variables(&self) -> &StoryVariables {
        &self.variables
    }

    #[must_use]
    pub fn counts(&self) -> (usize, usize, usize, usize) {
        (
            self.backgrounds.len(),
            self.images.len(),
            self.characters.len(),
            self.audio.len(),
        )
    }

    /// The two image trees in the order `primary` asks for: the tree the
    /// command kind means first, the other as the fallback. This ordering is
    /// the ONLY thing that separates a `[Background]` lookup from an `[Image]`
    /// one, so both pairs below are this with the pair swapped.
    const fn trees(
        &self,
        primary: ImageSource,
    ) -> (&HashMap<String, String>, &HashMap<String, String>) {
        match primary {
            ImageSource::Background => (&self.backgrounds, &self.images),
            ImageSource::Image => (&self.images, &self.backgrounds),
        }
    }

    const fn size_trees(
        &self,
        primary: ImageSource,
    ) -> (&HashMap<String, ImageSize>, &HashMap<String, ImageSize>) {
        match primary {
            ImageSource::Background => (&self.background_sizes, &self.image_sizes),
            ImageSource::Image => (&self.image_sizes, &self.background_sizes),
        }
    }

    /// One image name resolved with `primary` tried first, saying which tree
    /// answered.
    fn resolve_as(&self, name: &str, primary: ImageSource) -> Option<(String, ImageSource)> {
        let key = name.trim().to_ascii_lowercase();
        let (first, second) = self.trees(primary);
        first.get(&key).map_or_else(
            || second.get(&key).map(|u| (u.clone(), primary.fallback())),
            |u| Some((u.clone(), primary)),
        )
    }

    /// The plate metrics for a name resolved with `primary` tried first, by the
    /// SAME order [`Self::resolve_as`] uses, so the size belongs to the very
    /// file whose URL went on the wire.
    fn size_as(&self, name: &str, primary: ImageSource) -> Option<ImageSize> {
        let key = name.trim().to_ascii_lowercase();
        let (first, second) = self.size_trees(primary);
        first.get(&key).or_else(|| second.get(&key)).copied()
    }

    /// A `Background`/`LargeBg`/`GridBg`/`VerticalBg` image: the `bg/` tree
    /// first, then the CG tree.
    #[must_use]
    pub fn resolve_background(&self, name: &str) -> Option<(String, ImageSource)> {
        self.resolve_as(name, ImageSource::Background)
    }

    /// An `Image`/`CgItem`/`ShowItem` image: the CG tree first, then `bg/`.
    #[must_use]
    pub fn resolve_image(&self, name: &str) -> Option<(String, ImageSource)> {
        self.resolve_as(name, ImageSource::Image)
    }

    /// The plate metrics for a name resolved as a BACKGROUND.
    #[must_use]
    pub fn resolve_background_size(&self, name: &str) -> Option<ImageSize> {
        self.size_as(name, ImageSource::Background)
    }

    /// The plate metrics for a name resolved as an IMAGE, CG tree first.
    #[must_use]
    pub fn resolve_image_size(&self, name: &str) -> Option<ImageSize> {
        self.size_as(name, ImageSource::Image)
    }

    /// How many sized names came from a `sprites.json` and how many from a
    /// PNG header at the assumed `ppu` of 100, over both trees.
    #[must_use]
    pub fn size_sources(&self) -> (usize, usize) {
        let mut read = 0;
        let mut assumed = 0;
        for s in self
            .background_sizes
            .values()
            .chain(self.image_sizes.values())
        {
            if s.ppu_assumed.unwrap_or(false) {
                assumed += 1;
            } else {
                read += 1;
            }
        }
        (read, assumed)
    }

    /// True when a name exists in BOTH image trees, so the lookup order decides.
    #[must_use]
    pub fn image_is_ambiguous(&self, name: &str) -> bool {
        let key = name.trim().to_ascii_lowercase();
        self.backgrounds.contains_key(&key) && self.images.contains_key(&key)
    }

    /// A sound or music reference (`$key` through the variables table, or a
    /// bare logical path), resolved by basename over every clip on disk.
    #[must_use]
    pub fn resolve_audio(&self, reference: &str) -> Option<String> {
        let logical = self.variables.resolve(reference)?;
        let base = logical.rsplit('/').next().unwrap_or(logical).trim();
        if base.is_empty() {
            return None;
        }
        let key = base.to_ascii_lowercase();
        let key = key.strip_suffix(".ogg").unwrap_or(&key);
        self.audio.get(key).cloned()
    }

    /// A `[Video]` command's `res` (`video/act38side/PV01.mp4`) resolved to
    /// the two transcodes on disk. The leading `video/` and the extension are
    /// cut and the rest is lowercased, because the scripts write the stem in
    /// the CASE the CN bundles used and our tree is lowercase throughout.
    #[must_use]
    pub fn resolve_video(&self, reference: &str) -> Option<VideoSources> {
        let lowered = reference.trim().to_ascii_lowercase();
        let rest = lowered
            .trim_start_matches('/')
            .strip_prefix("video/")
            .unwrap_or_else(|| lowered.trim_start_matches('/'));
        let key = Path::new(rest).with_extension("");
        let key = key.to_str()?;
        if key.is_empty() {
            return None;
        }
        self.videos.get(key).cloned()
    }

    /// How many cutscene clips the `video/` walk found (16 on EN).
    #[must_use]
    pub fn video_count(&self) -> usize {
        self.videos.len()
    }

    /// A `[popupdialog]` head. The `$avatar_x` variable names an operator
    /// (`char_002_amiya`), and the avatar sheet carries it under an art
    /// suffix (`char_002_amiya_1+.png`), so the lookup tries the bare id, the
    /// `_1`/`_2`/`_1+` arts, then any stem under that id. All 163 EN values
    /// land.
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
        let mut best: Option<(&String, &String)> = None;
        for (stem, url) in &self.avatars {
            if stem.starts_with(&prefix) && best.is_none_or(|(b, _)| stem < b) {
                best = Some((stem, url));
            }
        }
        best.map(|(_, u)| u.clone())
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

    fn sprite_url(folder: &SpriteFolder, file: &str) -> String {
        format!(
            "/textures/avg/characters/{}/{}",
            encode_path_piece(&folder.name),
            encode_path_piece(file)
        )
    }

    /// The hub group a body belongs to: the one whose sprite list HOLDS that
    /// body (measured true on all 1,636 EN hubs, where group k lists
    /// `N$(k+1)`), falling back to the `$M` index decremented, which is what
    /// the client does.
    fn hub_group<'a>(
        folder: &'a SpriteFolder,
        body_stem: &str,
        body: &str,
    ) -> Option<&'a HubGroup> {
        folder
            .hub
            .iter()
            .find(|g| g.index_of(body_stem).is_some())
            .or_else(|| {
                let m = body.parse::<usize>().ok().filter(|m| *m >= 1).unwrap_or(1);
                folder.hub.get(m - 1)
            })
    }

    /// The face patch a hub names for this body. `@alias` matches
    /// `sprites[].alias` and WINS over the file-name rules, because an alias
    /// carries no index for them to read; otherwise `#N` is 1-based and
    /// decremented, and a MISSING or unparseable index is index 0, the first
    /// face, never "none". The body's own entry is never a face, and a
    /// sentinel group has no patches at all.
    fn hub_face_stem<'a>(
        group: &'a HubGroup,
        body_stem: &str,
        name: &SpriteName<'_>,
    ) -> Option<&'a str> {
        if group.is_sentinel() {
            return None;
        }
        let sprite = if let Some(alias) = name.alias {
            let alias = alias.to_ascii_lowercase();
            group.sprites.iter().find(|s| s.alias == alias)?
        } else {
            let n = name
                .face
                .map(trim_index)
                .and_then(|f| f.parse::<usize>().ok())
                .filter(|n| *n >= 1)
                .map_or(0, |n| n - 1);
            group.sprites.get(n)?
        };
        if sprite.is_whole_body || sprite.name == body_stem || sprite.name.is_empty() {
            return None;
        }
        Some(&sprite.name)
    }

    /// The body TEXTURE's own size, which is the unit `facePos` is measured
    /// in. The hub's `size` for that sprite entry answers first, because it
    /// is the very `m_Rect` the client's uv algebra divides by; a folder
    /// whose `hub.json` predates that field, or holds no entry for this body,
    /// falls back to the PNG's own IHDR. Both read 1280 on
    /// `avg_1037_amiya3_1$1` and 1024 on `avg_225_haak_1$1`.
    fn body_size(folder: &SpriteFolder, body_stem: &str, body_file: &str) -> Option<BodySize> {
        folder
            .hub
            .iter()
            .flat_map(|g| g.sprites.iter())
            .find(|s| s.name == body_stem)
            .and_then(|s| s.size)
            .filter(|e| e.w > 0.0 && e.h > 0.0)
            .map(|e| BodySize { w: e.w, h: e.h })
            .or_else(|| png_size(&folder.dir.join(body_file)))
    }

    /// Resolve `base#face$body` to its body and face files. See the module
    /// doc for the rule order.
    #[must_use]
    pub fn resolve_character(&self, raw: &str) -> Option<CharacterSprite> {
        let parts = parse_sprite_name(raw);
        let folder = self.sprite_folder(parts.base)?;
        let face = trim_index(parts.face.unwrap_or("1"));
        let body = trim_index(parts.body.unwrap_or("1"));
        let fname = folder.name.to_ascii_lowercase();
        let stem = fname
            .rfind('_')
            .filter(|&i| fname[i + 1..].bytes().all(|b| b.is_ascii_digit()) && i + 1 < fname.len())
            .map_or(fname.as_str(), |i| &fname[..i]);

        // 1. Hub body with a face patch.
        if let Some(body_file) = folder.files.get(&format!("{fname}${body}")) {
            let body_stem = format!("{fname}${body}");
            let group = Self::hub_group(folder, &body_stem, body);
            // The hub names the face the index really means, which is how
            // `@alias` and a face whose file the rules below miss still land.
            let hub_face = group
                .and_then(|g| Self::hub_face_stem(g, &body_stem, &parts))
                .and_then(|s| folder.files.get(s));
            let face_file = if parts.alias.is_some() {
                hub_face
            } else {
                folder
                    .files
                    .get(&format!("{face}${body}"))
                    .or_else(|| folder.files.get(&format!("{face}$1")))
                    .or(hub_face)
            };
            let face_pos = face_file.and_then(|f| {
                let g = group?;
                let stem = normalized_stem(f);
                if g.is_sentinel() || g.sprites.iter().any(|s| s.name == stem && s.is_whole_body) {
                    return None;
                }
                Some(g.face_pos())
            });
            return Some(CharacterSprite {
                body_url: Self::sprite_url(folder, body_file),
                face_url: face_file.map(|f| Self::sprite_url(folder, f)),
                face_pos,
                body_size: Self::body_size(folder, &body_stem, body_file),
                plate: folder.plate,
            });
        }

        // 2. Whole sprite selected by face (`char_002_amiya_5`, `avg_npc_135_1#3`).
        let mut by_face = vec![
            format!("{stem}_{face}"),
            format!("{fname}_{face}"),
            format!("{fname}#{face}"),
            format!("{fname}_1#{face}"),
        ];
        if face == "1" {
            by_face.push(fname.clone());
        }
        // 3. Else the folder's default body (`avg_npc_139#1.png` beside bare
        // `2.png` patches). No candidate, no sprite.
        let defaults = [
            format!("{stem}_1"),
            fname.clone(),
            format!("{fname}_1"),
            format!("{fname}$1"),
            format!("{fname}#1"),
            format!("{fname}_1#1"),
        ];
        let file = by_face
            .iter()
            .chain(defaults.iter())
            .find_map(|c| folder.files.get(c))?;
        // The old hub layout's bare `<face>.png` patch rides on either body;
        // a folder with faces baked into the body has no bare patch to find.
        let body_stem = normalized_stem(file);
        let group = Self::hub_group(folder, &body_stem, body);
        let hub_face = group
            .and_then(|g| Self::hub_face_stem(g, &body_stem, &parts))
            .and_then(|s| folder.files.get(s));
        let face_file = if parts.alias.is_some() {
            hub_face
        } else {
            folder.files.get(face).or(hub_face)
        };
        let face_pos = face_file.and_then(|f| {
            let g = group?;
            let stem = normalized_stem(f);
            if g.is_sentinel() || g.sprites.iter().any(|s| s.name == stem && s.is_whole_body) {
                return None;
            }
            Some(g.face_pos())
        });
        Some(CharacterSprite {
            body_url: Self::sprite_url(folder, file),
            face_url: face_file.map(|f| Self::sprite_url(folder, f)),
            face_pos,
            body_size: Self::body_size(folder, &body_stem, file),
            plate: folder.plate,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprite_name_parts() {
        let p = parse_sprite_name("avg_225_haak_1#3$2");
        assert_eq!(p.base, "avg_225_haak_1");
        assert_eq!(p.face, Some("3"));
        assert_eq!(p.body, Some("2"));
        assert_eq!(p.alias, None);
        let p = parse_sprite_name("char_002_amiya_1#5");
        assert_eq!(
            (p.base, p.face, p.body),
            ("char_002_amiya_1", Some("5"), None)
        );
        let p = parse_sprite_name("avg_npc_001");
        assert_eq!((p.base, p.face, p.body), ("avg_npc_001", None, None));
        let p = parse_sprite_name("char_102_texas_1#1 ");
        assert_eq!(
            (p.base, p.face, p.body),
            ("char_102_texas_1", Some("1"), None)
        );
        let p = parse_sprite_name("avg_1014_nearl2_1$1");
        assert_eq!(
            (p.base, p.face, p.body),
            ("avg_1014_nearl2_1", None, Some("1"))
        );
        // `@alias` names a face through the hub instead of by index.
        let p = parse_sprite_name("char_002_amiya_1@open mouth");
        assert_eq!(
            (p.base, p.face, p.alias),
            ("char_002_amiya_1", None, Some("open mouth"))
        );
        let p = parse_sprite_name("avg_1037_amiya3_1@smile$2");
        assert_eq!(
            (p.base, p.alias, p.body),
            ("avg_1037_amiya3_1", Some("smile"), Some("2"))
        );
    }

    #[test]
    fn stems_fold_doubled_extensions() {
        assert_eq!(normalized_stem("1$1.png.png"), "1$1");
        assert_eq!(normalized_stem("1$1..png"), "1$1");
        assert_eq!(normalized_stem("Avg_225_Haak_1$1.png"), "avg_225_haak_1$1");
    }

    fn folder(name: &str, files: &[&str]) -> SpriteFolder {
        let mut f = SpriteFolder {
            name: name.to_owned(),
            ..SpriteFolder::default()
        };
        for n in files.iter().filter(|n| !is_alpha_companion(n)) {
            f.files.insert(normalized_stem(n), (*n).to_owned());
        }
        f
    }

    fn index_with(folders: Vec<SpriteFolder>) -> StoryAssetIndex {
        let mut idx = StoryAssetIndex::default();
        for f in folders {
            idx.characters.insert(f.name.to_ascii_lowercase(), f);
        }
        idx
    }

    /// The `hub.json` shape the unpacker writes, parsed by the same
    /// `serde` path the index uses on disk.
    fn with_hub(mut f: SpriteFolder, json: &str) -> SpriteFolder {
        let file = serde_json::from_str::<HubFile>(json).expect("hub json");
        let mut groups = file.groups;
        for g in &mut groups {
            for s in &mut g.sprites {
                s.name = s.name.to_ascii_lowercase();
                s.alias = s.alias.to_ascii_lowercase();
            }
        }
        f.hub = groups;
        f.plate = file.root;
        f
    }

    const HAAK_HUB: &str = r#"{
      "groups": [{
        "facePos": {"x": 512.0, "y": 120.0},
        "faceSize": {"w": 51.0, "h": 71.0},
        "sprites": [
          {"name": "1$1", "alias": "", "isWholeBody": false},
          {"name": "3$1", "alias": "wry", "isWholeBody": false},
          {"name": "avg_225_haak_1$1", "alias": "", "isWholeBody": false}
        ]
      }],
      "legacy": false
    }"#;

    /// Amiya's hub, with the `size` the unpacker reads off each Sprite's
    /// `m_Rect`: a 1280 px body and 128 px faces.
    const AMIYA_HUB: &str = r#"{
      "groups": [{
        "facePos": {"x": 570.0, "y": 233.0},
        "faceSize": {"w": 117.0, "h": 95.0},
        "sprites": [
          {"name": "1$1", "alias": "", "isWholeBody": false, "size": {"w": 128.0, "h": 128.0}},
          {"name": "avg_1037_amiya3_1$1", "alias": "", "isWholeBody": true, "size": {"w": 1280.0, "h": 1280.0}}
        ]
      }],
      "legacy": false
    }"#;

    /// The body texture is 1280 where the canvas plate is 1024, so the patch
    /// covers 570/1280 = 0.44531 of the plate and NOT 570/1024 = 0.55664.
    #[test]
    fn the_body_texture_size_rides_beside_the_placement() {
        let idx = index_with(vec![with_hub(
            folder("avg_1037_amiya3_1", &["avg_1037_amiya3_1$1.png", "1$1.png"]),
            AMIYA_HUB,
        )]);
        let s = idx.resolve_character("avg_1037_amiya3_1#1$1").unwrap();
        let size = s.body_size.expect("bodySize");
        assert_eq!((size.w, size.h), (1280.0, 1280.0));
        let pos = s.face_pos.expect("facePos");
        assert!((pos.x / size.w - 0.445_312_5).abs() < 1e-6, "{pos:?}");
        assert!((pos.w / size.w - 0.091_406_25).abs() < 1e-6, "{pos:?}");
    }

    /// A `hub.json` written before `size` existed still reaches the wire with
    /// a body size: the PNG's own IHDR answers instead.
    #[test]
    fn a_hub_with_no_size_falls_back_to_the_png_header() {
        let dir = std::env::temp_dir().join(format!(
            "myrtle-bodysize-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        // Signature, IHDR length + tag, then 1280x1280 big-endian.
        let mut png: Vec<u8> = b"\x89PNG\r\n\x1a\n".to_vec();
        png.extend_from_slice(&13u32.to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&1280u32.to_be_bytes());
        png.extend_from_slice(&1280u32.to_be_bytes());
        std::fs::write(dir.join("avg_1037_amiya3_1$1.png"), &png).expect("write png");
        std::fs::write(dir.join("1$1.png"), &png).expect("write png");
        let mut f = with_hub(
            folder("avg_1037_amiya3_1", &["avg_1037_amiya3_1$1.png", "1$1.png"]),
            // The same hub with every `size` stripped.
            &AMIYA_HUB
                .replace(", \"size\": {\"w\": 128.0, \"h\": 128.0}", "")
                .replace(", \"size\": {\"w\": 1280.0, \"h\": 1280.0}", ""),
        );
        f.dir = dir.clone();
        let idx = index_with(vec![f]);
        let s = idx.resolve_character("avg_1037_amiya3_1#1$1").unwrap();
        assert_eq!(
            s.body_size.map(|b| (b.w, b.h)),
            Some((1280.0, 1280.0)),
            "the IHDR fallback should read the texture, never assume 1024"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// The plate is the PREFAB's, so it is the same on every sprite of the
    /// folder, and it is not the 1024 slot template.
    #[test]
    fn the_prefab_plate_rides_on_every_sprite_of_the_folder() {
        let hub = HAAK_HUB.replace(
            "\"legacy\": false",
            "\"legacy\": false, \"root\": {\"x\": 0.0, \"y\": 150.0, \"w\": 955.0, \"h\": 955.0}",
        );
        let idx = index_with(vec![with_hub(
            folder(
                "avg_225_haak_1",
                &["avg_225_haak_1$1.png", "1$1.png", "3$1.png"],
            ),
            &hub,
        )]);
        let want = Some(PlateRect {
            x: 0.0,
            y: 150.0,
            w: 955.0,
            h: 955.0,
        });
        assert_eq!(
            idx.resolve_character("avg_225_haak_1#1$1").unwrap().plate,
            want
        );
        assert_eq!(
            idx.resolve_character("avg_225_haak_1#3$1").unwrap().plate,
            want
        );
        // A hub with no root at all carries no plate rather than a guess.
        let plain = index_with(vec![with_hub(
            folder("avg_225_haak_1", &["avg_225_haak_1$1.png"]),
            HAAK_HUB,
        )]);
        assert_eq!(
            plain.resolve_character("avg_225_haak_1").unwrap().plate,
            None
        );
    }

    /// An absent `screenadapt` sizes a plate at the sprite's OWN ppu, so the
    /// `sprites.json` the unpacker writes beats the PNG header; a name it does
    /// not cover falls back to that header at an ASSUMED ppu of 100.
    #[test]
    fn image_sizes_read_the_sprites_json_and_fall_back_to_the_png_header() {
        let root = std::env::temp_dir().join(format!(
            "myrtle-imagesize-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let dir = root.join("textures/avg/bg/avg_bkg_h1_bg_ch_0");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let png = |w: u32, h: u32| {
            let mut b: Vec<u8> = b"\x89PNG\r\n\x1a\n".to_vec();
            b.extend_from_slice(&13u32.to_be_bytes());
            b.extend_from_slice(b"IHDR");
            b.extend_from_slice(&w.to_be_bytes());
            b.extend_from_slice(&h.to_be_bytes());
            b
        };
        std::fs::write(dir.join("bg_cher_1.png"), png(1024, 576)).expect("png");
        std::fs::write(dir.join("bg_nosprite.png"), png(512, 288)).expect("png");
        std::fs::write(
            dir.join("sprites.json"),
            r#"{"bg_cher_1": {"w": 1024.0, "h": 576.0, "ppu": 68.2464, "pivot": {"x": 0.5, "y": 0.5}}}"#,
        )
        .expect("sprites.json");
        let idx = StoryAssetIndex::build(&root);
        let cher = idx.resolve_background_size("bg_cher_1").expect("bg_cher_1");
        assert_eq!((cher.w, cher.h), (1024.0, 576.0));
        assert!((cher.ppu - 68.2464).abs() < 1e-4, "{cher:?}");
        assert_eq!(cher.ppu_assumed, None);
        // 1024 * 100 / 68.2464 = 1500.4 canvas px, the game's measured 1500.7.
        assert!((cher.w * 100.0 / cher.ppu - 1500.4).abs() < 0.1);
        let other = idx
            .resolve_background_size("bg_nosprite")
            .expect("bg_nosprite");
        assert_eq!((other.w, other.h, other.ppu), (512.0, 288.0, 100.0));
        assert_eq!(other.ppu_assumed, Some(true));
        assert_eq!(idx.size_sources(), (1, 1));
        assert_eq!(idx.resolve_background_size("bg_missing"), None);
        std::fs::remove_dir_all(&root).ok();
    }

    /// Neither a hub size nor a readable PNG: the placement still ships, and
    /// the reader measures the body itself.
    #[test]
    fn no_hub_size_and_no_png_leaves_body_size_absent() {
        let idx = index_with(vec![with_hub(
            folder(
                "avg_225_haak_1",
                &["avg_225_haak_1$1.png", "1$1.png", "3$1.png"],
            ),
            HAAK_HUB,
        )]);
        let s = idx.resolve_character("avg_225_haak_1#3$1").unwrap();
        assert_eq!(s.body_size, None);
        assert!(s.face_pos.is_some());
    }

    #[test]
    fn hub_face_pos_rides_the_face_patch() {
        let idx = index_with(vec![with_hub(
            folder(
                "avg_225_haak_1",
                &["avg_225_haak_1$1.png", "1$1.png", "3$1.png"],
            ),
            HAAK_HUB,
        )]);
        let s = idx.resolve_character("avg_225_haak_1#3$1").unwrap();
        assert_eq!(
            s.face_pos,
            Some(FacePos {
                x: 512.0,
                y: 120.0,
                w: 51.0,
                h: 71.0
            })
        );
        // A MISSING index is index 0, the first face, and carries the same
        // placement.
        let s = idx.resolve_character("avg_225_haak_1").unwrap();
        assert!(s.face_url.as_deref().unwrap().ends_with("1$1.png"));
        assert!(s.face_pos.is_some());
        // `@alias` reaches the face the hub names, by alias, not by index.
        let s = idx.resolve_character("avg_225_haak_1@wry$1").unwrap();
        assert!(s.face_url.as_deref().unwrap().ends_with("3$1.png"));
        assert!(s.face_pos.is_some());
        // No patch file for this index: body only, and NO facePos.
        let s = idx.resolve_character("avg_225_haak_1#9$1").unwrap();
        assert_eq!(s.face_url, None);
        assert_eq!(s.face_pos, None);
    }

    #[test]
    fn a_sentinel_hub_and_a_whole_body_sprite_carry_no_face_pos() {
        let sentinel = r#"{
          "groups": [{
            "facePos": {"x": -1.0, "y": -1.0},
            "faceSize": {"w": 0.0, "h": 0.0},
            "sprites": [
              {"name": "char_002_amiya_1", "alias": "normal", "isWholeBody": false},
              {"name": "char_002_amiya_5", "alias": "angry", "isWholeBody": false}
            ]
          }],
          "legacy": true
        }"#;
        let idx = index_with(vec![with_hub(
            folder(
                "char_002_amiya_1",
                &["char_002_amiya_1.png", "char_002_amiya_5.png"],
            ),
            sentinel,
        )]);
        let s = idx.resolve_character("char_002_amiya_1#5").unwrap();
        assert!(s.body_url.ends_with("char_002_amiya_5.png"));
        assert_eq!(
            s.face_pos, None,
            "the (-1,-1)/(0,0) sentinel places nothing"
        );

        // A real placement, but the indexed sprite is flagged isWholeBody.
        let whole = r#"{
          "groups": [{
            "facePos": {"x": 100.0, "y": 50.0},
            "faceSize": {"w": 60.0, "h": 40.0},
            "sprites": [
              {"name": "1", "alias": "", "isWholeBody": true},
              {"name": "avg_x_1", "alias": "", "isWholeBody": false}
            ]
          }],
          "legacy": true
        }"#;
        let idx = index_with(vec![with_hub(
            folder("avg_x_1", &["avg_x_1.png", "1.png"]),
            whole,
        )]);
        let s = idx.resolve_character("avg_x_1#1").unwrap();
        assert!(s.face_url.as_deref().unwrap().ends_with("1.png"));
        assert_eq!(s.face_pos, None);
    }

    #[test]
    fn the_body_index_picks_the_group() {
        let two = r#"{
          "groups": [
            {"facePos": {"x": 570.0, "y": 233.0}, "faceSize": {"w": 117.0, "h": 95.0},
             "sprites": [{"name": "1$1", "alias": "", "isWholeBody": false},
                         {"name": "avg_a_1$1", "alias": "", "isWholeBody": false}]},
            {"facePos": {"x": 300.0, "y": 111.0}, "faceSize": {"w": 80.0, "h": 60.0},
             "sprites": [{"name": "1$2", "alias": "", "isWholeBody": false},
                         {"name": "avg_a_1$2", "alias": "", "isWholeBody": false}]}
          ],
          "legacy": false
        }"#;
        let idx = index_with(vec![with_hub(
            folder(
                "avg_a_1",
                &["avg_a_1$1.png", "avg_a_1$2.png", "1$1.png", "1$2.png"],
            ),
            two,
        )]);
        let a = idx.resolve_character("avg_a_1#1$1").unwrap();
        let b = idx.resolve_character("avg_a_1#1$2").unwrap();
        assert_eq!(a.face_pos.unwrap().x, 570.0);
        assert_eq!(b.face_pos.unwrap().x, 300.0);
        assert_eq!(b.face_pos.unwrap().h, 60.0);
    }

    #[test]
    fn a_folder_with_no_hub_json_resolves_exactly_as_before() {
        let idx = index_with(vec![folder(
            "avg_225_haak_1",
            &["avg_225_haak_1$1.png", "1$1.png", "3$1.png"],
        )]);
        let s = idx.resolve_character("avg_225_haak_1#3$1").unwrap();
        assert!(s.face_url.as_deref().unwrap().ends_with("3$1.png"));
        assert_eq!(s.face_pos, None);
    }

    #[test]
    fn hub_sprite_resolves_body_and_face() {
        let idx = index_with(vec![folder(
            "avg_225_haak_1",
            &[
                "avg_225_haak_1$1.png",
                "1$1.png",
                "3$1.png",
                "avg_225_haak_1$1[alpha].png",
            ],
        )]);
        let s = idx.resolve_character("avg_225_haak_1#3$1").unwrap();
        assert_eq!(
            s.body_url,
            "/textures/avg/characters/avg_225_haak_1/avg_225_haak_1$1.png"
        );
        assert_eq!(
            s.face_url.as_deref(),
            Some("/textures/avg/characters/avg_225_haak_1/3$1.png")
        );
        assert_eq!(s.face_pos, None);
        // No suffixes: face 1, body 1.
        let s = idx.resolve_character("avg_225_haak_1").unwrap();
        assert_eq!(
            s.face_url.as_deref(),
            Some("/textures/avg/characters/avg_225_haak_1/1$1.png")
        );
        // A face the folder lacks yields the body alone.
        let s = idx.resolve_character("avg_225_haak_1#9$1").unwrap();
        assert_eq!(s.face_url, None);
        // Case-insensitive.
        assert!(idx.resolve_character("AVG_225_Haak_1#1$1").is_some());
    }

    #[test]
    fn legacy_sprite_resolves_by_face_index() {
        let idx = index_with(vec![
            folder(
                "char_002_amiya_1",
                &["char_002_amiya_1.png", "char_002_amiya_5.png"],
            ),
            folder(
                "char_362_saga",
                &["char_362_saga#1.png", "char_362_saga#6.png"],
            ),
            folder(
                "char_130_doberm_ex",
                &["char_130_doberm_ex.png", "char_130_doberm_ex_2.png"],
            ),
        ]);
        let s = idx.resolve_character("char_002_amiya_1#5").unwrap();
        assert_eq!(
            s.body_url,
            "/textures/avg/characters/char_002_amiya_1/char_002_amiya_5.png"
        );
        assert_eq!(s.face_url, None);
        let s = idx.resolve_character("char_002_amiya_1").unwrap();
        assert_eq!(
            s.body_url,
            "/textures/avg/characters/char_002_amiya_1/char_002_amiya_1.png"
        );
        // A missing face falls back to the folder default.
        let s = idx.resolve_character("char_002_amiya_1#9").unwrap();
        assert_eq!(
            s.body_url,
            "/textures/avg/characters/char_002_amiya_1/char_002_amiya_1.png"
        );
        // `#` in a file name is percent-encoded in the URL.
        let s = idx.resolve_character("char_362_Saga#6").unwrap();
        assert_eq!(
            s.body_url,
            "/textures/avg/characters/char_362_saga/char_362_saga%236.png"
        );
        // `base` without a trailing `_1` finds the `_1` folder.
        let s = idx.resolve_character("char_002_amiya#5").unwrap();
        assert!(s.body_url.ends_with("char_002_amiya_5.png"));
        let s = idx.resolve_character("char_130_doberm_ex#2").unwrap();
        assert!(s.body_url.ends_with("char_130_doberm_ex_2.png"));
        assert!(idx.resolve_character("nobody_here").is_none());
        assert!(idx.resolve_character("").is_none());
    }

    #[test]
    fn old_hub_layout_serves_the_body_with_a_bare_face_patch() {
        let idx = index_with(vec![
            folder(
                "avg_1013_spchen_1",
                &[
                    "avg_1013_spchen_1.png",
                    "1.png",
                    "2.png",
                    "alpha.png",
                    "avg_1013_spchen_1[alpha].png",
                ],
            ),
            folder(
                "avg_npc_122",
                &["avg_npc_122_1.png", "1.png", "2.png", "alpha.png"],
            ),
            folder("avg_only_patches", &["1.png", "2.png"]),
            folder(
                "avg_npc_139",
                &["avg_npc_139#1.png", "1.png", "2.png", "alpha.png"],
            ),
            folder(
                "avg_npc_135",
                &["avg_npc_135_1#1.png", "avg_npc_135_1#3.png"],
            ),
        ]);
        // `#1` body variant plus bare patches: the patch rides on the body.
        let s = idx.resolve_character("avg_npc_139#2").unwrap();
        assert_eq!(
            s.body_url,
            "/textures/avg/characters/avg_npc_139/avg_npc_139%231.png"
        );
        assert_eq!(
            s.face_url.as_deref(),
            Some("/textures/avg/characters/avg_npc_139/2.png")
        );
        // `_1#face` whole sprites.
        let s = idx.resolve_character("avg_npc_135#3").unwrap();
        assert_eq!(
            s.body_url,
            "/textures/avg/characters/avg_npc_135/avg_npc_135_1%233.png"
        );
        assert_eq!(s.face_url, None);
        let s = idx.resolve_character("avg_npc_135").unwrap();
        assert!(s.body_url.ends_with("avg_npc_135_1%231.png"));
        let s = idx.resolve_character("avg_1013_spchen_1#2").unwrap();
        assert_eq!(
            s.body_url,
            "/textures/avg/characters/avg_1013_spchen_1/avg_1013_spchen_1.png"
        );
        assert_eq!(
            s.face_url.as_deref(),
            Some("/textures/avg/characters/avg_1013_spchen_1/2.png")
        );
        // `avg_npc_122`: the stem rule cuts to `avg_npc`, the `_1` default carries the face.
        let s = idx.resolve_character("avg_npc_122#2").unwrap();
        assert_eq!(
            s.body_url,
            "/textures/avg/characters/avg_npc_122/avg_npc_122_1.png"
        );
        assert_eq!(
            s.face_url.as_deref(),
            Some("/textures/avg/characters/avg_npc_122/2.png")
        );
        // A face the folder lacks: body alone. `alpha.png` is never a face.
        assert_eq!(
            idx.resolve_character("avg_npc_122#9").unwrap().face_url,
            None
        );
        assert!(!idx.characters["avg_npc_122"].files.contains_key("alpha"));
        // A folder holding only patches yields NO sprite, never a patch as body.
        assert!(idx.resolve_character("avg_only_patches#1").is_none());
    }

    #[test]
    fn zero_padded_indices_resolve() {
        let idx = index_with(vec![folder(
            "avg_172_svrash_1",
            &["avg_172_svrash_1$1.png", "1$1.png", "2$1.png"],
        )]);
        let s = idx.resolve_character("avg_172_svrash_1#01$1").unwrap();
        assert_eq!(
            s.face_url.as_deref(),
            Some("/textures/avg/characters/avg_172_svrash_1/1$1.png")
        );
        let s = idx.resolve_character("avg_172_svrash_1#2$01").unwrap();
        assert_eq!(
            s.body_url,
            "/textures/avg/characters/avg_172_svrash_1/avg_172_svrash_1$1.png"
        );
        assert_eq!(
            s.face_url.as_deref(),
            Some("/textures/avg/characters/avg_172_svrash_1/2$1.png")
        );
        assert_eq!(trim_index("00"), "0");
        assert_eq!(trim_index("10"), "10");
    }

    #[test]
    fn video_index_keys_by_lowercased_relative_path() {
        let dir = std::env::temp_dir().join(format!("story-video-{}", std::process::id()));
        let clips = dir.join("video/main_10");
        std::fs::create_dir_all(&clips).unwrap();
        std::fs::write(clips.join("main_10_enter.webm"), b"w").unwrap();
        std::fs::write(clips.join("main_10_enter.mp4"), b"m").unwrap();

        let idx = StoryAssetIndex::build(&dir);
        assert_eq!(idx.video_count(), 1);
        // The script writes the stem in the bundles' case and names the mp4;
        // both transcodes come back under the one key.
        let v = idx
            .resolve_video("video/main_10/MAIN_10_ENTER.mp4")
            .unwrap();
        assert_eq!(
            v.webm_url.as_deref(),
            Some("/video/main_10/main_10_enter.webm")
        );
        assert_eq!(
            v.mp4_url.as_deref(),
            Some("/video/main_10/main_10_enter.mp4")
        );
        // The same clip named without the folder prefix, and a clip EN retired.
        assert!(idx.resolve_video("main_10/main_10_enter.mp4").is_some());
        assert!(idx.resolve_video("video/act15side/IW01.mp4").is_none());
        assert!(idx.resolve_video("").is_none());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn audio_resolves_through_variables_and_bare_paths() {
        let mut idx = StoryAssetIndex::default();
        idx.audio.insert(
            "m_avg_decisivebattle_loop".to_owned(),
            "/audio/audio/sound_beta_2/music/act25side/m_avg_decisivebattle/m_avg_DecisiveBattle_loop.ogg".to_owned(),
        );
        idx.audio.insert(
            "d_avg_devicebeep".to_owned(),
            "/audio/audio/sound_beta_2/avg_se_3/d_avg_devicebeep.ogg".to_owned(),
        );
        idx.variables = StoryVariables::from_map(HashMap::from([(
            "DecisiveBattle_loop".to_owned(),
            "Sound_Beta_2/Music/act25side/m_avg_DecisiveBattle_loop".to_owned(),
        )]));
        assert!(idx.resolve_audio("$DecisiveBattle_loop").is_some());
        assert!(idx.resolve_audio("$missing").is_none());
        assert!(
            idx.resolve_audio("Sound_Beta_2/AVG/d_avg_devicebeep")
                .is_some()
        );
        assert!(idx.resolve_audio("d_avg_DeviceBeep").is_some());
        assert!(idx.resolve_audio("").is_none());
    }

    #[test]
    fn image_lookup_order_and_ambiguity() {
        let mut idx = StoryAssetIndex::default();
        idx.backgrounds
            .insert("shared".to_owned(), "/bg/shared.png".to_owned());
        idx.images
            .insert("shared".to_owned(), "/imgs/shared.png".to_owned());
        idx.images
            .insert("cg_only".to_owned(), "/imgs/cg_only.png".to_owned());
        assert_eq!(
            idx.resolve_background("Shared").unwrap().1,
            ImageSource::Background
        );
        assert_eq!(idx.resolve_image("shared").unwrap().1, ImageSource::Image);
        assert_eq!(
            idx.resolve_background("cg_only").unwrap().1,
            ImageSource::Image
        );
        assert!(idx.image_is_ambiguous("shared"));
        assert!(!idx.image_is_ambiguous("cg_only"));
        assert!(idx.resolve_image("nothing").is_none());
    }
}
