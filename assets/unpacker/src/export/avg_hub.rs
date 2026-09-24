//! The AVG character sprite hub: where a face patch sits on its body.
//!
//! An `avg/characters/*.ab` bundle carries one `MonoBehaviour` that says how its
//! PNGs compose. Two classes ship, and the type tree names their fields
//! differently:
//!
//! * `AVGCharacterSpriteHubGroup` (script 3518600802999480713, the modern
//!   `avg_*` bundles): `spriteGroups: SpriteConfigGroup[]`, each group being
//!   `sprites: SpriteConfig[]` of `{sprite: PPtr<Sprite>, alphaTex, alias,
//!   isWholeBody}` plus `facePos: Vector3f` and `faceSize: Vector2f`.
//! * `AVGCharacterSpriteHub` (script -1280753076610887924, the legacy
//!   `char_*` bundles): a flat `sprites` list with `FacePos`/`FaceSize`
//!   capitalised, and the sentinel `FacePos (-1,-1)` / `FaceSize (0,0)`
//!   meaning the sprites are whole bodies with no patch to place.
//!
//! `facePos` is the face patch's TOP-LEFT corner in body pixels, x from the
//! left and y from the TOP; `faceSize` is the on-body size the face texture is
//! scaled into. Haak reads `facePos (512,120,0)` / `faceSize (51,71)`, which
//! the IL2CPP uv algebra in `docs/story-reader-il2cpp-characters.md` section 6
//! solves to a patch covering body x 512..563 and y 120..191. The placement is
//! PER GROUP, one pair for every face in it, and the group is chosen by the
//! `$M` body index (`avg_1037_amiya3_1` has 4 groups of 18 sprites).
//!
//! Each entry also carries its `Sprite`'s own `m_Rect` size as `size {w,h}`,
//! in TEXTURE pixels. It is NOT decoration: `facePos` is measured in the BODY
//! TEXTURE's pixels, not in the 1024 canvas px the story stage draws a body
//! into, and the two differ. `avg_1037_amiya3_1$1` is a 1280x1280 texture, so
//! its `facePos (570,233)` is 570/1280 = 0.44531 of the plate and NOT
//! 570/1024 = 0.55664. Every consumer divides by this size.
//!
//! The bundle also carries the character's OWN root `RectTransform`, written
//! as `root {x,y,w,h}` in canvas px. It is not decoration either: the slot
//! template's 1024 at y 203 LOSES to it. One game frame measured Amiya's
//! plate at 1091.7 canvas px against her bundle's 1090, and Dobermann's at
//! 958.9 in the same frame, which no shared template can produce
//! (`docs/story-reader-captures.md` section 2).
//!
//! The export writes this beside the folder's PNGs as `hub.json`. It is a
//! first-class output of the normal texture pass, rewritten on every extract,
//! so the orphan sweep (`assets/orphans.mjs`, which deletes anything in a
//! touched subtree older than the run) never reaps it.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::unity::bundle::BundleFile;
use crate::unity::object_reader::read_object;
use crate::unity::serialized_file::SerializedFile;

/// One entry of a group's sprite list, in declaration order: the index the
/// script's `#N` / `$M` decrements into.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HubSprite {
    /// The `Sprite` object's `m_Name`, which is the exported PNG's stem
    /// (`1$1`, `avg_225_haak_1$1`). Empty when the reference points outside
    /// the bundle, so the entry keeps its index either way.
    pub name: String,
    /// The authored alias a script may address with `@alias` (`normal`,
    /// `open mouth`). Empty on every modern hub read so far.
    pub alias: String,
    #[serde(rename = "isWholeBody")]
    pub is_whole_body: bool,
    /// The referenced `Sprite`'s `m_Rect` size in TEXTURE pixels. Absent when
    /// the reference points outside the bundle, and on a hub written before
    /// this field existed, so a reader must still have a fallback.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<HubSize>,
}

/// A point in body pixels: the face patch's top-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HubPoint {
    pub x: f32,
    pub y: f32,
}

/// The on-body size a face texture is scaled into, in body pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HubSize {
    pub w: f32,
    pub h: f32,
}

/// The character prefab's own root `RectTransform`, in CANVAS px: `x`,`y` is
/// the anchored position (y up, measured from the slot origin the way the slot
/// template's own `(0,203)` reads) and `w`,`h` the `sizeDelta` times the local
/// scale. Every EN character bundle anchors its root at `(0.5,0.5)` with pivot
/// `(0.5,0.5)` and scale `(1,1,1)`, so `sizeDelta` IS the size; a stretched
/// root, where it would not be, is skipped rather than guessed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HubRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// One body's sprite list and its single face placement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HubGroup {
    #[serde(rename = "facePos")]
    pub face_pos: HubPoint,
    #[serde(rename = "faceSize")]
    pub face_size: HubSize,
    pub sprites: Vec<HubSprite>,
}

impl HubGroup {
    /// The `FacePos (-1,-1)` / `FaceSize (0,0)` sentinel: this group's sprites
    /// are whole bodies and NOTHING is placed on them.
    #[must_use]
    pub fn is_sentinel(&self) -> bool {
        self.face_size.w <= 0.0
            || self.face_size.h <= 0.0
            || self.face_pos.x < 0.0
            || self.face_pos.y < 0.0
    }
}

/// A bundle's whole hub, as `hub.json` carries it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hub {
    pub groups: Vec<HubGroup>,
    /// True when the bundle carries the older flat `AVGCharacterSpriteHub`.
    pub legacy: bool,
    /// The prefab's own root rect. Absent when the bundle carries no root
    /// `RectTransform` with a positive size, and on a `hub.json` written
    /// before this field existed, so a reader must still have a fallback.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<HubRect>,
}

/// A serialized `Vector2f`/`Vector3f`'s x and y, each falling back when the
/// field is absent.
fn xy(v: Option<&Value>, fallback: (f32, f32)) -> (f32, f32) {
    let read = |k: &str, d: f32| {
        v.and_then(|o| o.get(k))
            .and_then(Value::as_f64)
            .map_or(d, |n| n as f32)
    };
    (read("x", fallback.0), read("y", fallback.1))
}

/// The hub's own vectors, where an absent field is the `(-1,-1)` sentinel.
fn vec_xy(v: Option<&Value>) -> (f32, f32) {
    xy(v, (-1.0, -1.0))
}

/// The prefab's root rect: the `RectTransform` whose `m_Father` is null. A
/// bundle that somehow ships several takes the lowest path ID, so the output
/// is deterministic across runs, and a stretched root (`anchorMin` away from
/// `anchorMax`, where `sizeDelta` is an inset rather than a size) is skipped.
fn root_rect(objects: &HashMap<i64, (i32, Value)>) -> Option<HubRect> {
    let mut roots: Vec<(&i64, &Value)> = objects
        .iter()
        .filter(|(_, (class_id, v))| {
            *class_id == 224
                && v.get("m_Father")
                    .and_then(|f| f.get("m_PathID"))
                    .and_then(Value::as_i64)
                    .unwrap_or(0)
                    == 0
        })
        .map(|(pid, (_, v))| (pid, v))
        .collect();
    roots.sort_by_key(|(pid, _)| **pid);
    roots.into_iter().find_map(|(_, v)| {
        let (min_x, min_y) = xy(v.get("m_AnchorMin"), (0.5, 0.5));
        let (max_x, max_y) = xy(v.get("m_AnchorMax"), (0.5, 0.5));
        if (min_x - max_x).abs() > f32::EPSILON || (min_y - max_y).abs() > f32::EPSILON {
            return None;
        }
        let (x, y) = xy(v.get("m_AnchoredPosition"), (0.0, 0.0));
        let (dw, dh) = xy(v.get("m_SizeDelta"), (0.0, 0.0));
        let (sx, sy) = xy(v.get("m_LocalScale"), (1.0, 1.0));
        let (w, h) = (dw * sx, dh * sy);
        (w > 0.0 && h > 0.0).then_some(HubRect { x, y, w, h })
    })
}

/// A `Sprite`'s `m_Rect` as a size. Every AVG character sprite is a FULL-RECT
/// sprite (`rect (0,0,w,h)`, `offset (0,0)`), so this is the whole texture:
/// 1280x1280 on `avg_1037_amiya3_1$1`, 128x128 on its face patches.
fn rect_size(v: Option<&Value>) -> Option<HubSize> {
    let r = v?;
    let w = r.get("width").and_then(Value::as_f64)? as f32;
    let h = r.get("height").and_then(Value::as_f64)? as f32;
    (w > 0.0 && h > 0.0).then_some(HubSize { w, h })
}

fn sprite_entries(
    list: Option<&Value>,
    sprite_meta: &HashMap<i64, (String, Option<HubSize>)>,
) -> Vec<HubSprite> {
    let Some(arr) = list.and_then(Value::as_array) else {
        return Vec::new();
    };
    arr.iter()
        .map(|e| {
            let meta = e
                .get("sprite")
                .filter(|p| p.get("m_FileID").and_then(Value::as_i64).unwrap_or(0) == 0)
                .and_then(|p| p.get("m_PathID"))
                .and_then(Value::as_i64)
                .and_then(|pid| sprite_meta.get(&pid));
            HubSprite {
                name: meta.map(|m| m.0.clone()).unwrap_or_default(),
                alias: e
                    .get("alias")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                is_whole_body: e
                    .get("isWholeBody")
                    .and_then(Value::as_i64)
                    .is_some_and(|b| b != 0),
                size: meta.and_then(|m| m.1),
            }
        })
        .collect()
}

/// Read the hub out of already-deserialized objects: `path_id -> (class_id,
/// value)`. Returns `None` when the bundle carries no hub `MonoBehaviour`.
#[must_use]
pub fn hub_from_objects(objects: &HashMap<i64, (i32, Value)>) -> Option<Hub> {
    let mut sprite_names: HashMap<i64, (String, Option<HubSize>)> = HashMap::new();
    for (pid, (class_id, v)) in objects {
        if *class_id == 213
            && let Some(n) = v.get("m_Name").and_then(Value::as_str)
        {
            sprite_names.insert(*pid, (n.to_owned(), rect_size(v.get("m_Rect"))));
        }
    }
    // A bundle holds exactly one hub; take the lowest path_id when a future
    // one holds more, so the output is deterministic across runs.
    let mut candidates: Vec<(&i64, &Value)> = objects
        .iter()
        .filter(|(_, (class_id, v))| {
            *class_id == 114 && (v.get("spriteGroups").is_some() || v.get("FacePos").is_some())
        })
        .map(|(pid, (_, v))| (pid, v))
        .collect();
    candidates.sort_by_key(|(pid, _)| **pid);
    let (_, hub) = candidates.first()?;

    let root = root_rect(objects);
    if let Some(groups) = hub.get("spriteGroups").and_then(Value::as_array) {
        return Some(Hub {
            groups: groups
                .iter()
                .map(|g| {
                    let (px, py) = vec_xy(g.get("facePos"));
                    let (sw, sh) = vec_xy(g.get("faceSize"));
                    HubGroup {
                        face_pos: HubPoint { x: px, y: py },
                        face_size: HubSize { w: sw, h: sh },
                        sprites: sprite_entries(g.get("sprites"), &sprite_names),
                    }
                })
                .collect(),
            legacy: false,
            root,
        });
    }
    let (px, py) = vec_xy(hub.get("FacePos"));
    let (sw, sh) = vec_xy(hub.get("FaceSize"));
    Some(Hub {
        groups: vec![HubGroup {
            face_pos: HubPoint { x: px, y: py },
            face_size: HubSize { w: sw, h: sh },
            sprites: sprite_entries(hub.get("sprites"), &sprite_names),
        }],
        legacy: true,
        root,
    })
}

/// Read the hub straight out of a parsed bundle. Only `MonoBehaviour` (114),
/// `Sprite` (213) and `RectTransform` (224) objects are deserialized, so this
/// is cheap enough to run on every `avg/characters` bundle of a normal
/// extract.
#[must_use]
pub fn hub_from_bundle(bundle: &BundleFile) -> Option<Hub> {
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
            if !matches!(obj.class_id, 114 | 213 | 224) {
                continue;
            }
            if let Ok(v) = read_object(&sf, obj) {
                objects.insert(obj.path_id, (obj.class_id, v));
            }
        }
    }
    hub_from_objects(&objects)
}

/// True for a bundle whose textures land under `textures/avg/characters/`.
#[must_use]
pub fn is_avg_character_bundle(bundle_subdir: &Path) -> bool {
    let mut it = bundle_subdir.components();
    let first = it.next().and_then(|c| c.as_os_str().to_str());
    let second = it.next().and_then(|c| c.as_os_str().to_str());
    first == Some("avg") && second == Some("characters")
}

/// Write `hub.json` next to a sprite folder's PNGs. Pretty-printed so a diff
/// between two extracts reads line by line.
pub fn write_hub_json(dir: &Path, hub: &Hub) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let mut json = serde_json::to_string_pretty(hub)?;
    json.push('\n');
    std::fs::write(dir.join("hub.json"), json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// `(path_id, name, texture px)`: a square `m_Rect`, or 0 for a Sprite
    /// that carries none, which is what a hub written before `size` existed
    /// reads like.
    fn objects(hub: Value, sprites: &[(i64, &str, f64)]) -> HashMap<i64, (i32, Value)> {
        let mut m: HashMap<i64, (i32, Value)> = HashMap::new();
        m.insert(1, (114, hub));
        for (pid, name, px) in sprites {
            let mut v = json!({ "m_Name": name });
            if *px > 0.0 {
                v["m_Rect"] = json!({"x": 0.0, "y": 0.0, "width": px, "height": px});
            }
            m.insert(*pid, (213, v));
        }
        m
    }

    #[test]
    fn modern_hub_reads_groups_in_order() {
        let hub = json!({
            "spriteGroups": [{
                "facePos": {"x": 512.0, "y": 120.0, "z": 0.0},
                "faceSize": {"x": 51.0, "y": 71.0},
                "sprites": [
                    {"alias": "", "isWholeBody": 0, "sprite": {"m_FileID": 0, "m_PathID": 10}},
                    {"alias": "", "isWholeBody": 0, "sprite": {"m_FileID": 0, "m_PathID": 11}},
                    {"alias": "", "isWholeBody": 1, "sprite": {"m_FileID": 0, "m_PathID": 12}},
                ],
            }],
        });
        let h = hub_from_objects(&objects(
            hub,
            &[
                (10, "1$1", 128.0),
                (11, "2$1", 128.0),
                (12, "avg_225_haak_1$1", 1024.0),
            ],
        ))
        .unwrap();
        assert!(!h.legacy);
        assert_eq!(h.groups.len(), 1);
        let g = &h.groups[0];
        assert_eq!(g.face_pos, HubPoint { x: 512.0, y: 120.0 });
        assert_eq!(g.face_size, HubSize { w: 51.0, h: 71.0 });
        assert!(!g.is_sentinel());
        assert_eq!(
            g.sprites
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>(),
            ["1$1", "2$1", "avg_225_haak_1$1"]
        );
        assert!(g.sprites[2].is_whole_body);
        // Each entry carries its OWN texture size: 128 px faces on a 1024 body.
        assert_eq!(g.sprites[0].size, Some(HubSize { w: 128.0, h: 128.0 }));
        assert_eq!(
            g.sprites[2].size,
            Some(HubSize {
                w: 1024.0,
                h: 1024.0
            })
        );
    }

    /// `facePos` is in BODY TEXTURE pixels, and Amiya's body texture is 1280,
    /// not the 1024 the stage draws it into: the size is what makes the two
    /// separable.
    #[test]
    fn a_body_bigger_than_the_canvas_reports_its_own_size() {
        let hub = json!({
            "spriteGroups": [{
                "facePos": {"x": 570.0, "y": 233.0, "z": 0.0},
                "faceSize": {"x": 117.0, "y": 95.0},
                "sprites": [
                    {"alias": "", "isWholeBody": 0, "sprite": {"m_FileID": 0, "m_PathID": 30}},
                    {"alias": "", "isWholeBody": 1, "sprite": {"m_FileID": 0, "m_PathID": 31}},
                    // Out of the bundle: no name and no size either.
                    {"alias": "", "isWholeBody": 0, "sprite": {"m_FileID": 2, "m_PathID": 32}},
                ],
            }],
        });
        let h = hub_from_objects(&objects(
            hub,
            &[
                (30, "1$1", 128.0),
                (31, "avg_1037_amiya3_1$1", 1280.0),
                (32, "elsewhere", 512.0),
            ],
        ))
        .unwrap();
        let g = &h.groups[0];
        assert_eq!(
            g.sprites[1].size,
            Some(HubSize {
                w: 1280.0,
                h: 1280.0
            })
        );
        assert_eq!(g.face_pos.x / g.sprites[1].size.unwrap().w, 570.0 / 1280.0);
        assert_eq!(g.sprites[2].size, None);
    }

    /// A Sprite with no `m_Rect` at all leaves `size` absent rather than 0,
    /// so a reader can tell "unknown" from "zero" and fall back.
    #[test]
    fn a_sprite_with_no_rect_carries_no_size() {
        let hub = json!({
            "spriteGroups": [{
                "facePos": {"x": 1.0, "y": 2.0, "z": 0.0},
                "faceSize": {"x": 3.0, "y": 4.0},
                "sprites": [{"alias": "", "isWholeBody": 0, "sprite": {"m_FileID": 0, "m_PathID": 40}}],
            }],
        });
        let h = hub_from_objects(&objects(hub, &[(40, "1$1", 0.0)])).unwrap();
        assert_eq!(h.groups[0].sprites[0].size, None);
        let json = serde_json::to_string(&h).unwrap();
        assert!(!json.contains("size"), "{json}");
    }

    #[test]
    fn legacy_hub_is_one_sentinel_group_with_aliases() {
        let hub = json!({
            "FacePos": {"x": -1.0, "y": -1.0, "z": 0.0},
            "FaceSize": {"x": 0.0, "y": 0.0},
            "sprites": [
                {"alias": "normal", "isWholeBody": 0, "sprite": {"m_FileID": 0, "m_PathID": 20}},
                {"alias": "smile", "isWholeBody": 0, "sprite": {"m_FileID": 1, "m_PathID": 21}},
            ],
        });
        let h = hub_from_objects(&objects(hub, &[(20, "char_002_amiya_1", 1024.0)])).unwrap();
        assert!(h.legacy);
        assert!(h.groups[0].is_sentinel());
        assert_eq!(h.groups[0].sprites[0].alias, "normal");
        // An out-of-bundle sprite keeps its slot with an empty name.
        assert_eq!(h.groups[0].sprites[1].name, "");
    }

    /// The prefab's own root rect, which BEATS the 1024 slot template.
    #[test]
    fn the_root_recttransform_is_read_in_canvas_px() {
        let hub = json!({
            "FacePos": {"x": -1.0, "y": -1.0, "z": 0.0},
            "FaceSize": {"x": 0.0, "y": 0.0},
            "sprites": [{"alias": "", "isWholeBody": 0, "sprite": {"m_FileID": 0, "m_PathID": 20}}],
        });
        let mut m = objects(hub, &[(20, "char_002_amiya_1", 1024.0)]);
        m.insert(
            50,
            (
                224,
                json!({
                    "m_Father": {"m_FileID": 0, "m_PathID": 0},
                    "m_AnchorMin": {"x": 0.5, "y": 0.5},
                    "m_AnchorMax": {"x": 0.5, "y": 0.5},
                    "m_AnchoredPosition": {"x": 0.0, "y": 203.0},
                    "m_SizeDelta": {"x": 1090.0, "y": 1090.0},
                    "m_Pivot": {"x": 0.5, "y": 0.5},
                    "m_LocalScale": {"x": 1.0, "y": 1.0, "z": 1.0},
                }),
            ),
        );
        // A child rect is never the root, however large it is.
        m.insert(
            51,
            (
                224,
                json!({
                    "m_Father": {"m_FileID": 0, "m_PathID": 50},
                    "m_AnchoredPosition": {"x": 0.0, "y": 0.0},
                    "m_SizeDelta": {"x": 4096.0, "y": 4096.0},
                }),
            ),
        );
        let h = hub_from_objects(&m).unwrap();
        assert_eq!(
            h.root,
            Some(HubRect {
                x: 0.0,
                y: 203.0,
                w: 1090.0,
                h: 1090.0
            })
        );
    }

    /// `sizeDelta` is scaled, and a STRETCHED root has no size to read at all.
    #[test]
    fn a_scaled_root_multiplies_and_a_stretched_one_is_skipped() {
        let hub = json!({
            "spriteGroups": [{
                "facePos": {"x": 1.0, "y": 2.0, "z": 0.0},
                "faceSize": {"x": 3.0, "y": 4.0},
                "sprites": [],
            }],
        });
        let rt = |father: i64, min: f64, max: f64, scale: f64| {
            json!({
                "m_Father": {"m_FileID": 0, "m_PathID": father},
                "m_AnchorMin": {"x": min, "y": min},
                "m_AnchorMax": {"x": max, "y": max},
                "m_AnchoredPosition": {"x": -20.0, "y": 165.0},
                "m_SizeDelta": {"x": 800.0, "y": 400.0},
                "m_LocalScale": {"x": scale, "y": scale, "z": scale},
            })
        };
        let mut m = objects(hub.clone(), &[]);
        m.insert(60, (224, rt(0, 0.5, 0.5, 0.5)));
        assert_eq!(
            hub_from_objects(&m).unwrap().root,
            Some(HubRect {
                x: -20.0,
                y: 165.0,
                w: 400.0,
                h: 200.0
            })
        );
        let mut m = objects(hub, &[]);
        m.insert(60, (224, rt(0, 0.0, 1.0, 1.0)));
        assert_eq!(hub_from_objects(&m).unwrap().root, None);
    }

    #[test]
    fn no_hub_monobehaviour_is_none() {
        let mut m: HashMap<i64, (i32, Value)> = HashMap::new();
        m.insert(1, (114, json!({ "m_Name": "something else" })));
        assert!(hub_from_objects(&m).is_none());
    }

    #[test]
    fn only_avg_character_bundles_are_hubs() {
        assert!(is_avg_character_bundle(Path::new(
            "avg/characters/avg_225_haak_1"
        )));
        assert!(!is_avg_character_bundle(Path::new("avg/bg/bg_cher")));
        assert!(!is_avg_character_bundle(Path::new(
            "chararts/char_002_amiya"
        )));
    }
}
