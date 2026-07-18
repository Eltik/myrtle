//! Resolve a particle material's **external** `_MainTex` reference to the actual
//! texture, pulled from the shared FX texture bundles (`refs/fx/texture/*.ab`:
//! star, trail, smoke, flow, ray, …).
//!
//! Dynchar prefabs reference these shared sprites externally, so the emitter
//! exports `tex: null` and renders nothing — the sparkles, streaks, smoke and
//! flow effects visible in-game are simply missing. We pre-load the FX bundles
//! once into a `(CAB, path_id) -> (Texture2D, resources)` map and decode on
//! demand, so *any* skin's external-textured emitters render their real sprite.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::Value;

use crate::export::texture::{DecodedTexture, decode_texture_object};
use crate::unity::bundle::BundleFile;
use crate::unity::object_reader::read_object;
use crate::unity::serialized_file::{FileIdentifier, SerializedFile};

struct FxEntry {
    value: Value,
    /// The FX bundle's streamed-resource files (`.resS`/`.resource`), keyed by
    /// filename — the texture's `m_StreamData` points into these.
    res: Arc<HashMap<String, Vec<u8>>>,
}

/// Material JSON + that bundle's external-CAB names (for resolving the
/// material's own further-external `_MainTex`).
type MaterialEntry = (Value, Arc<Vec<String>>);

/// Alpha above which a texel counts as opaque for the sprite gate.
const SPRITE_GATE_OPAQUE_ALPHA: u8 = 230;
/// Percentage of opaque texels at/above which a texture is treated as a data map.
const SPRITE_GATE_OPAQUE_PCT: usize = 85;

/// Sprite gate: true when the texture is essentially fully opaque. Genuine
/// effect SPRITES have real transparency (soft edges: star, trail, smoke); a
/// fully-opaque texture is a distortion / normal / dark field map (no sprite
/// shape) — billboarded it stamps a hard box, so callers skip it.
fn is_opaque_data_map(decoded: &DecodedTexture) -> bool {
    let opaque = decoded.rgba.iter().skip(3).step_by(4).filter(|&&a| a > SPRITE_GATE_OPAQUE_ALPHA).count();
    let total = (decoded.width * decoded.height).max(1) as usize;
    opaque * 100 / total >= SPRITE_GATE_OPAQUE_PCT
}

#[derive(Default)]
pub struct FxTextures {
    map: HashMap<(String, i64), FxEntry>,
    /// External MATERIALS (class 21) from the FX bundles, keyed by `(CAB, path_id)` →
    /// `(material JSON, that bundle's external-CAB names)`. A dynchar emitter can
    /// reference its whole material externally (into `dynchars/effect.ab`), not just
    /// the texture — then the emitter has no blend AND no sprite (`tex: null`,
    /// invisible: Virtuosa's gold-mist/rain). We resolve the material JSON here and
    /// inline it; its `_MainTex` may sit in the SAME bundle (`file_id == 0`) or a
    /// FURTHER bundle it references (`file_id > 0` → that bundle via the stored CAB
    /// names), so the emitter gets both its blend class and its real sprite.
    materials: HashMap<(String, i64), MaterialEntry>,
}

/// A shared FX texture bundle. Two sources supply the external sprites a dynchar's
/// particle emitters reference:
///  - `refs/fx/` (…/texture/star.ab, etc.) — the global effect-texture library.
///  - `arts/dynchars/effect.ab` — the shared DYNCHAR effect bundle (sparks, streaks,
///    the `rainbow` rain sheets, sand). Emitters cross-reference its textures the same
///    way; without it those refs stay `tex: null` (e.g. Virtuosa's standing-window
///    rain). It's a sibling of the per-skin `arts/dynchars/char_*.ab`, so a full
///    extraction always has it available.
fn is_fx_bundle(path: &Path) -> bool {
    let p = path.to_string_lossy().to_ascii_lowercase();
    p.ends_with(".ab") && (p.contains("refs/fx/") || (p.contains("dynchars/") && p.ends_with("/effect.ab")))
}

impl FxTextures {
    /// Pre-load every FX bundle among `files` into the `(CAB, path_id)` map.
    /// Empty (feature off) when none are present.
    #[must_use]
    pub fn build(files: &[PathBuf]) -> Self {
        let mut map = HashMap::new();
        let mut materials = HashMap::new();
        for path in files.iter().filter(|p| is_fx_bundle(p)) {
            let Ok(bytes) = std::fs::read(path) else { continue };
            let Ok(bundle) = BundleFile::parse(bytes) else { continue };
            for entry in &bundle.files {
                if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
                    continue;
                }
                let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
                let cab = entry.path.clone();
                // This SerializedFile's external dependency CAB names (for resolving a
                // material's own further-external `_MainTex`).
                let ext_cabs: Arc<Vec<String>> = Arc::new(sf.externals.iter().map(|d| d.cab_name().to_string()).collect());
                // This SerializedFile's streamed resources.
                let res: Arc<HashMap<String, Vec<u8>>> = Arc::new(
                    bundle
                        .files
                        .iter()
                        .filter(|f| f.path.ends_with(".resS") || f.path.ends_with(".resource"))
                        .map(|f| (f.path.rsplit('/').next().unwrap_or(&f.path).to_string(), f.data.clone()))
                        .collect(),
                );
                for obj in &sf.objects {
                    match obj.class_id {
                        28 => {
                            if let Ok(v) = read_object(&sf, obj) {
                                map.insert((cab.clone(), obj.path_id), FxEntry { value: v, res: Arc::clone(&res) });
                            }
                        }
                        21 => {
                            if let Ok(v) = read_object(&sf, obj) {
                                materials.insert((cab.clone(), obj.path_id), (v, Arc::clone(&ext_cabs)));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Self { map, materials }
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Resolve an external MATERIAL ref (`file_id`,`path_id`, via the referencing
    /// file's `externals`) → `(the material's own bundle CAB, its JSON)`. The CAB
    /// lets the caller decode the material's `_MainTex`, which lives in that SAME FX
    /// bundle, via {@link texture_by_cab}. `None` when in-bundle or unresolvable.
    #[must_use]
    pub fn resolve_material_ref(&self, externals: &[FileIdentifier], file_id: i64, path_id: i64) -> Option<(String, Value, Arc<Vec<String>>)> {
        if file_id <= 0 {
            return None;
        }
        let dep = externals.get((file_id - 1) as usize)?;
        let cab = dep.cab_name().to_string();
        let (mat, ext_cabs) = self.materials.get(&(cab.clone(), path_id))?;
        Some((cab, mat.clone(), Arc::clone(ext_cabs)))
    }

    /// Decode an external material's own `_MainTex` given the MATERIAL's bundle CAB +
    /// its external-CAB names: the texture is either in the SAME bundle (`file_id==0`
    /// → `mat_cab`) or in a FURTHER bundle the material references (`file_id>0` →
    /// `mat_ext_cabs[file_id-1]`). Gated like {@link resolve_decode} (opaque field
    /// maps stay out so they never stamp a grey box). `None` if the bundle isn't loaded.
    #[must_use]
    pub fn material_texture(&self, mat_cab: &str, mat_ext_cabs: &[String], file_id: i64, path_id: i64) -> Option<DecodedTexture> {
        let cab = if file_id == 0 { mat_cab.to_string() } else { mat_ext_cabs.get((file_id - 1) as usize)?.clone() };
        let entry = self.map.get(&(cab, path_id))?;
        let decoded = decode_texture_object(&entry.value, &entry.res).ok().flatten()?;
        if is_opaque_data_map(&decoded) {
            return None;
        }
        Some(decoded)
    }

    /// Resolve + decode an external texture ref (`file_id`, `path_id`) via the
    /// referencing file's `externals` table, with NO content gating. Backs both
    /// the gated public entry point and the Ram-family bypass.
    fn resolve_raw(&self, externals: &[FileIdentifier], file_id: i64, path_id: i64) -> Option<DecodedTexture> {
        if file_id <= 0 {
            return None;
        }
        let dep = externals.get((file_id - 1) as usize)?;
        let entry = self.map.get(&(dep.cab_name().to_string(), path_id))?;
        decode_texture_object(&entry.value, &entry.res).ok().flatten()
    }

    /// Resolve + decode an external texture ref (`file_id`, `path_id`) via the
    /// referencing file's `externals` table. Returns the decoded RGBA texture.
    #[must_use]
    pub fn resolve_decode(&self, externals: &[FileIdentifier], file_id: i64, path_id: i64) -> Option<DecodedTexture> {
        let decoded = self.resolve_raw(externals, file_id, path_id)?;
        // Gated by the sprite gate: a rejected (opaque) texture leaves the
        // emitter `tex: null` rather than stamping a hard box.
        if is_opaque_data_map(&decoded) {
            return None;
        }
        Some(decoded)
    }

    /// Resolve + decode an external texture ref for a **Ram-family** material
    /// slot, bypassing the opaque-sprite gate. The Ram shaders (`Ram/Disturb`,
    /// `Ram/VertexDisturb`) legitimately sample opaque data maps (ramp / disturb /
    /// dissolve field textures, e.g. `flow_129_*`, `trail_113_2`) that the sprite
    /// gate would otherwise reject, so they must resolve regardless of opacity.
    #[must_use]
    pub fn resolve_decode_ram(&self, externals: &[FileIdentifier], file_id: i64, path_id: i64) -> Option<DecodedTexture> {
        self.resolve_raw(externals, file_id, path_id)
    }
}
