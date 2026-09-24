//! The AVG image sprites: what size a background or CG plate really is.
//!
//! A story `[Background(image="bg_cher_1")]` with no `screenadapt` argument is
//! `SetNativeSize` at the sprite's OWN pixels-per-unit, not a draw into the
//! 1280x720 canvas: the plate is `rect / ppu * referencePixelsPerUnit`, and
//! the reference is 100 on every AVG `CanvasScaler`
//! (`docs/story-reader-avg-prefab.md` section 2). `bg_cher_1` ships
//! `rect (1024,576)` at `ppu 68.2464`, so it draws 1024 * 100 / 68.2464 =
//! 1500.4 by 844.0 canvas px, against the 1500.7 x 843.5 the game was measured
//! at (`docs/story-reader-captures.md` section 1). The PPU is PER SPRITE:
//! 68.25, 80, 64 and 100 all ship inside the same bundles, so a plate cannot
//! be sized from its PNG alone.
//!
//! The export writes this beside the folder's PNGs as `sprites.json`, keyed by
//! the `Sprite`'s `m_Name` (the exported PNG's stem). Like `hub.json` it is a
//! first-class output of the normal texture pass, rewritten on every extract,
//! so the orphan sweep never reaps it, and `unpacker backfill-sprites` writes
//! only these files into an existing output tree.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::unity::bundle::BundleFile;
use crate::unity::object_reader::read_object;
use crate::unity::serialized_file::SerializedFile;

/// A normalized pivot, the fraction of the rect the sprite is placed by.
/// `(0.5,0.5)` on every AVG image sprite read so far.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpritePivot {
    pub x: f32,
    pub y: f32,
}

/// One `Sprite` as `sprites.json` carries it: its `m_Rect` size in texture
/// px, its own `m_PixelsToUnits`, and its pivot.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpriteMeta {
    pub w: f32,
    pub h: f32,
    pub ppu: f32,
    pub pivot: SpritePivot,
}

/// `m_Name` -> its metrics. A `BTreeMap` so two extracts write the same file.
pub type SpriteMetaMap = BTreeMap<String, SpriteMeta>;

fn f32_at(v: &Value, path: &[&str]) -> Option<f32> {
    let mut cur = v;
    for k in path {
        cur = cur.get(k)?;
    }
    cur.as_f64().map(|n| n as f32)
}

/// Read the image sprites out of already-deserialized objects: `path_id ->
/// (class_id, value)`. A sprite with no rect or a non-positive PPU is dropped,
/// because a consumer can do nothing with either.
#[must_use]
pub fn sprites_from_objects(objects: &HashMap<i64, (i32, Value)>) -> SpriteMetaMap {
    let mut out = SpriteMetaMap::new();
    for (class_id, v) in objects.values() {
        if *class_id != 213 {
            continue;
        }
        let Some(name) = v
            .get("m_Name")
            .and_then(Value::as_str)
            .filter(|n| !n.is_empty())
        else {
            continue;
        };
        let (Some(w), Some(h)) = (
            f32_at(v, &["m_Rect", "width"]),
            f32_at(v, &["m_Rect", "height"]),
        ) else {
            continue;
        };
        let Some(ppu) = f32_at(v, &["m_PixelsToUnits"]) else {
            continue;
        };
        if w <= 0.0 || h <= 0.0 || ppu <= 0.0 {
            continue;
        }
        out.insert(
            name.to_owned(),
            SpriteMeta {
                w,
                h,
                ppu,
                pivot: SpritePivot {
                    x: f32_at(v, &["m_Pivot", "x"]).unwrap_or(0.5),
                    y: f32_at(v, &["m_Pivot", "y"]).unwrap_or(0.5),
                },
            },
        );
    }
    out
}

/// Read the image sprites straight out of a parsed bundle. Only `Sprite`
/// (213) objects are deserialized, so this is cheap enough to run on every
/// image bundle of a normal extract.
#[must_use]
pub fn sprites_from_bundle(bundle: &BundleFile) -> SpriteMetaMap {
    let mut objects: HashMap<i64, (i32, Value)> = HashMap::new();
    for entry in &bundle.files {
        // A `.resS`/`.resource` blob is raw texture bytes, not a serialized
        // file: handing it to the parser reads a garbage object count and
        // PANICS rather than erroring, so it is skipped by name.
        if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        for obj in &sf.objects {
            if obj.class_id != 213 {
                continue;
            }
            if let Ok(v) = read_object(&sf, obj) {
                objects.insert(obj.path_id, (obj.class_id, v));
            }
        }
    }
    sprites_from_objects(&objects)
}

/// True for a bundle whose textures are story image plates: the four `avg`
/// image trees the reader resolves `[Background]` and `[Image]` names through,
/// plus the `spritepack/cutin_char_*` bundles an `[interlude]` names.
#[must_use]
pub fn is_avg_image_bundle(bundle_subdir: &Path) -> bool {
    let mut it = bundle_subdir.components();
    let first = it.next().and_then(|c| c.as_os_str().to_str());
    let second = it.next().and_then(|c| c.as_os_str().to_str());
    match (first, second) {
        (Some("avg"), Some(sub)) => matches!(sub, "bg" | "imgs" | "items" | "backgrounds"),
        (Some("spritepack"), Some(sub)) => sub.starts_with("cutin_char_"),
        _ => false,
    }
}

/// Write `sprites.json` next to a folder's PNGs. Pretty-printed so a diff
/// between two extracts reads line by line.
pub fn write_sprites_json(dir: &Path, sprites: &SpriteMetaMap) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let mut json = serde_json::to_string_pretty(sprites)?;
    json.push('\n');
    std::fs::write(dir.join("sprites.json"), json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sprite(name: &str, w: f64, h: f64, ppu: f64) -> Value {
        json!({
            "m_Name": name,
            "m_Rect": {"x": 0.0, "y": 0.0, "width": w, "height": h},
            "m_Offset": {"x": 0.0, "y": 0.0},
            "m_PixelsToUnits": ppu,
            "m_Pivot": {"x": 0.5, "y": 0.5},
        })
    }

    /// The PPU is per SPRITE: the same bundle ships 68.2464 and 80.
    #[test]
    fn each_sprite_keeps_its_own_ppu() {
        let mut m: HashMap<i64, (i32, Value)> = HashMap::new();
        m.insert(1, (213, sprite("bg_cher_1", 1024.0, 576.0, 68.2464)));
        m.insert(2, (213, sprite("bg_cherunder_2", 1024.0, 576.0, 80.0)));
        m.insert(3, (28, json!({"m_Name": "bg_cher_1"})));
        let out = sprites_from_objects(&m);
        assert_eq!(out.len(), 2);
        let bg = out["bg_cher_1"];
        assert_eq!((bg.w, bg.h, bg.ppu), (1024.0, 576.0, 68.2464));
        assert_eq!(bg.pivot, SpritePivot { x: 0.5, y: 0.5 });
        // 1024 * 100 / 68.2464 = 1500.4 canvas px, the game's measured 1500.7.
        assert!((bg.w * 100.0 / bg.ppu - 1500.4).abs() < 0.1);
        assert_eq!(out["bg_cherunder_2"].ppu, 80.0);
    }

    #[test]
    fn a_sprite_with_no_rect_or_no_ppu_is_dropped() {
        let mut m: HashMap<i64, (i32, Value)> = HashMap::new();
        m.insert(
            1,
            (213, json!({"m_Name": "no_rect", "m_PixelsToUnits": 100.0})),
        );
        m.insert(2, (213, sprite("zero_ppu", 100.0, 100.0, 0.0)));
        m.insert(3, (213, sprite("", 100.0, 100.0, 100.0)));
        assert!(sprites_from_objects(&m).is_empty());
    }

    #[test]
    fn only_the_story_image_trees_are_written() {
        for ok in [
            "avg/bg/avg_bkg_h1_bg_ch_0",
            "avg/imgs/avg_img_h1_0",
            "avg/items/avg_item_0",
            "avg/backgrounds/x",
            "spritepack/cutin_char_0",
        ] {
            assert!(is_avg_image_bundle(Path::new(ok)), "{ok}");
        }
        for no in [
            "avg/characters/avg_225_haak_1",
            "avg/effects/x",
            "spritepack/ui_char_avatar_h1_0",
            "chararts/char_002_amiya",
            "avg",
        ] {
            assert!(!is_avg_image_bundle(Path::new(no)), "{no}");
        }
    }
}
