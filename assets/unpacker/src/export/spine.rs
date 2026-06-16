use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::Path;

use base64::Engine;
use serde_json::Value;

use super::alpha_merge;
use super::texture::decode_texture_object;

/// Spine animation category based on asset naming/content
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpineCategory {
    BattleFront,
    BattleBack,
    Building,
    DynIllust,
    Enemy,
}

impl fmt::Display for SpineCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BattleFront => write!(f, "BattleFront"),
            Self::BattleBack => write!(f, "BattleBack"),
            Self::Building => write!(f, "Building"),
            Self::DynIllust => write!(f, "DynIllust"),
            Self::Enemy => write!(f, "Enemy"),
        }
    }
}

/// A grouped spine asset with properly-paired skel + atlas + textures
pub struct SpineAsset {
    /// Base name (e.g., "`char_002_amiya`" or "`build_char_002_amiya`")
    pub name: String,
    /// Raw skel binary data (decoded from base64)
    pub skel_data: Vec<u8>,
    /// Atlas text content
    pub atlas_text: String,
    /// Resolved texture objects (`path_id` → Value), keyed by texture name
    pub textures: Vec<(String, Value)>,
    /// Classification
    pub category: SpineCategory,
}

/// Check if a bundle path is eligible for spine extraction.
#[must_use]
pub fn detect_spine_bundle(bundle_subdir: &Path, input_dir: &Path) -> bool {
    let full_path = input_dir.join(bundle_subdir);
    let path_str = full_path.to_string_lossy();
    let path_lower = path_str.to_lowercase();

    for segment in &[
        "/chararts/",
        "/skinpack/",
        "/npcpack/",
        "/building/vault/characters/",
        "/arts/dynchars",
        "/arts/dynavatars",
    ] {
        if path_lower.contains(segment) {
            return true;
        }
    }

    for segment in &[
        "chararts/",
        "skinpack/",
        "npcpack/",
        "building/vault/characters/",
        "arts/dynchars",
        "arts/dynavatars",
    ] {
        if path_lower.starts_with(segment) {
            return true;
        }
    }

    false
}

/// Check if a bundle path is an enemy spine art bundle.
///
/// Enemy spine data (`SkeletonData` + .skel/.atlas `TextAssets` + textures) lives
/// in `refs/arts/enm_art_*.ab` pack bundles, ~60 enemies per bundle. The
/// `battle/enm_pfb_*.ab` prefab bundles only hold `GameObject` wiring with
/// external references into these art bundles, so they are not needed here.
#[must_use]
pub fn detect_enemy_spine_bundle(bundle_subdir: &Path, input_dir: &Path) -> bool {
    let full_path = input_dir.join(bundle_subdir);
    let path_lower = full_path.to_string_lossy().to_lowercase();

    path_lower.contains("/refs/arts/enm_art") || path_lower.starts_with("refs/arts/enm_art")
}

/// Extract a `path_id` from a JSON reference like {"`m_FileID"`: 0, "`m_PathID"`: 12345}
fn get_path_id(val: &Value) -> Option<i64> {
    val.get("m_PathID").and_then(serde_json::Value::as_i64)
}

/// Collect spine assets using `MonoBehaviour` reference graph traversal.
///
/// Reference chain:
/// ```text
/// SkeletonMecanim (has skeletonDataAsset + _animationName)
///   → SkeletonData (has skeletonJSON + atlasAssets)
///     → skel: TextAsset (.skel binary)
///     → Atlas MonoBehaviour (has atlasFile + materials)
///       → atlas: TextAsset (.atlas text)
///       → Material (has m_SavedProperties.m_TexEnvs)
///         → _MainTex → Texture2D
///         → _AlphaTex → Texture2D (optional)
/// ```
///
/// Returns (`spine_assets`, `claimed_path_ids`) where `claimed_path_ids` contains
/// `path_ids` of all objects consumed by spine extraction.
pub fn collect_spine_assets(
    all_objects: &HashMap<i64, (i32, Value)>,
) -> (Vec<SpineAsset>, HashSet<i64>) {
    let mut claimed = HashSet::new();
    let mut assets = Vec::new();

    // Find SkeletonMecanim MonoBehaviours (class_id=114 with skeletonDataAsset field)
    let skeleton_mecanimss: Vec<(i64, &Value)> = all_objects
        .iter()
        .filter(|(_, (class_id, val))| *class_id == 114 && val.get("skeletonDataAsset").is_some())
        .map(|(pid, (_, val))| (*pid, val))
        .collect();

    for (mecanim_pid, mecanim_val) in &skeleton_mecanimss {
        // Get _animationName for classification
        let anim_name = mecanim_val
            .get("_animationName")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Get the owning GameObject's name ("Front"/"Back" for battle spines).
        // Front and back skeletons can share one atlas, so the GameObject name
        // is the only reliable front/back discriminator.
        let game_object_name = mecanim_val
            .get("m_GameObject")
            .and_then(get_path_id)
            .and_then(|pid| all_objects.get(&pid))
            .and_then(|(class_id, go_val)| (*class_id == 1).then_some(go_val))
            .and_then(|go_val| go_val.get("m_Name"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Follow skeletonDataAsset → SkeletonData MonoBehaviour
        let skel_data_pid = match mecanim_val.get("skeletonDataAsset").and_then(get_path_id) {
            Some(pid) if pid != 0 => pid,
            _ => continue,
        };

        let Some(chain) = follow_skeleton_data(all_objects, skel_data_pid) else {
            continue;
        };

        // Classify the spine asset
        let base_name = chain
            .skel_name
            .strip_suffix(".skel")
            .unwrap_or(&chain.skel_name);
        let category = classify_spine(base_name, anim_name, game_object_name, &chain.atlas_text);

        // Claim all path_ids in this spine instance
        claimed.insert(*mecanim_pid);
        claimed.extend(&chain.claimed);

        assets.push(SpineAsset {
            name: base_name.to_string(),
            skel_data: chain.skel_bytes,
            atlas_text: chain.atlas_text,
            textures: chain.textures,
            category,
        });
    }

    (assets, claimed)
}

/// Collect enemy spine assets from a `refs/arts/enm_art_*` pack bundle.
///
/// Unlike operator bundles, enemy art bundles carry no `SkeletonMecanim`
/// components (those live in the `battle/enm_pfb_*` prefab bundles and point
/// here via external references), so traversal starts directly at each
/// `SkeletonData` `MonoBehaviour` (class 114 with `skeletonJSON` + `atlasAssets`).
/// All assets are categorized as [`SpineCategory::Enemy`].
#[must_use]
pub fn collect_enemy_spine_assets(
    all_objects: &HashMap<i64, (i32, Value)>,
) -> (Vec<SpineAsset>, HashSet<i64>) {
    let mut claimed = HashSet::new();
    let mut assets = Vec::new();

    let mut skel_data_pids: Vec<i64> = all_objects
        .iter()
        .filter(|(_, (class_id, val))| {
            *class_id == 114
                && val.get("skeletonJSON").is_some()
                && val.get("atlasAssets").is_some()
        })
        .map(|(pid, _)| *pid)
        .collect();
    // Deterministic processing order across runs
    skel_data_pids.sort_unstable();

    for skel_data_pid in skel_data_pids {
        let Some(chain) = follow_skeleton_data(all_objects, skel_data_pid) else {
            continue;
        };

        let base_name = chain
            .skel_name
            .strip_suffix(".skel")
            .unwrap_or(&chain.skel_name);

        claimed.extend(&chain.claimed);

        assets.push(SpineAsset {
            name: base_name.to_string(),
            skel_data: chain.skel_bytes,
            atlas_text: chain.atlas_text,
            textures: chain.textures,
            category: SpineCategory::Enemy,
        });
    }

    (assets, claimed)
}

/// Everything resolved by following a `SkeletonData` `MonoBehaviour`'s references.
struct SkeletonDataChain {
    skel_name: String,
    skel_bytes: Vec<u8>,
    atlas_text: String,
    textures: Vec<(String, Value)>,
    /// All `path_ids` consumed while walking the chain (including `skel_data_pid`)
    claimed: Vec<i64>,
}

/// Follow a `SkeletonData` `MonoBehaviour`'s reference chain:
/// ```text
/// SkeletonData (skeletonJSON + atlasAssets)
///   → skel: TextAsset (.skel binary, base64)
///   → Atlas MonoBehaviour (atlasFile + materials)
///     → atlas: TextAsset (.atlas text)
///     → Material → _MainTex/_AlphaTex → Texture2D
/// ```
/// Returns `None` when any required link is missing or malformed.
fn follow_skeleton_data(
    all_objects: &HashMap<i64, (i32, Value)>,
    skel_data_pid: i64,
) -> Option<SkeletonDataChain> {
    let skel_data_val = match all_objects.get(&skel_data_pid) {
        Some((114, val)) => val,
        _ => return None,
    };

    // Follow skeletonJSON → TextAsset (.skel)
    let skel_text_pid = match skel_data_val.get("skeletonJSON").and_then(get_path_id) {
        Some(pid) if pid != 0 => pid,
        _ => return None,
    };

    let skel_text_val = match all_objects.get(&skel_text_pid) {
        Some((49, val)) => val,
        _ => return None,
    };

    let skel_name = skel_text_val["m_Name"].as_str().unwrap_or("").to_string();
    let skel_script = skel_text_val["m_Script"].as_str().unwrap_or("");

    // Decode skel binary from base64
    let skel_bytes = base64::engine::general_purpose::STANDARD
        .decode(skel_script.strip_prefix("base64:")?)
        .ok()?;

    // Follow atlasAssets[0] → Atlas MonoBehaviour
    let atlas_mono_pid = match skel_data_val
        .get("atlasAssets")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(get_path_id)
    {
        Some(pid) if pid != 0 => pid,
        _ => return None,
    };

    let atlas_mono_val = match all_objects.get(&atlas_mono_pid) {
        Some((114, val)) => val,
        _ => return None,
    };

    // Follow atlasFile → TextAsset (.atlas)
    let atlas_text_pid = match atlas_mono_val.get("atlasFile").and_then(get_path_id) {
        Some(pid) if pid != 0 => pid,
        _ => return None,
    };

    let atlas_text_val = match all_objects.get(&atlas_text_pid) {
        Some((49, val)) => val,
        _ => return None,
    };

    let atlas_script = atlas_text_val["m_Script"].as_str().unwrap_or("");
    if atlas_script.starts_with("base64:") || !atlas_script.contains(".png") {
        return None;
    }
    let atlas_text = atlas_script.to_string();

    let mut claimed = vec![skel_data_pid, skel_text_pid, atlas_mono_pid, atlas_text_pid];

    // Follow materials → Material → textures
    let mut textures: Vec<(String, Value)> = Vec::new();
    let mut texture_pids: Vec<i64> = Vec::new();

    if let Some(materials) = atlas_mono_val.get("materials").and_then(|v| v.as_array()) {
        for mat_ref in materials {
            let mat_pid = match get_path_id(mat_ref) {
                Some(pid) if pid != 0 => pid,
                _ => continue,
            };

            let mat_val = match all_objects.get(&mat_pid) {
                Some((21, val)) => val,
                _ => continue,
            };

            // Extract textures from m_SavedProperties.m_TexEnvs
            // m_TexEnvs is a JSON object: {"_MainTex": {m_Texture: ...}, "_AlphaTex": ...}
            if let Some(tex_envs) = mat_val
                .get("m_SavedProperties")
                .and_then(|sp| sp.get("m_TexEnvs"))
                .and_then(|te| te.as_object())
            {
                for (tex_type, tex_data) in tex_envs {
                    if tex_type != "_MainTex" && tex_type != "_AlphaTex" {
                        continue;
                    }
                    let tex_pid = match tex_data.get("m_Texture").and_then(get_path_id) {
                        Some(pid) if pid != 0 => pid,
                        _ => continue,
                    };

                    if let Some((28, tex_val)) = all_objects.get(&tex_pid) {
                        let tex_name = tex_val["m_Name"].as_str().unwrap_or("unnamed").to_string();
                        if !texture_pids.contains(&tex_pid) {
                            texture_pids.push(tex_pid);
                            textures.push((tex_name, tex_val.clone()));
                        }
                    }
                }
            }

            claimed.push(mat_pid);
        }
    }

    claimed.extend(&texture_pids);

    Some(SkeletonDataChain {
        skel_name,
        skel_bytes,
        atlas_text,
        textures,
        claimed,
    })
}

/// Classify a spine asset into a category.
///   1. skel name starts with "dyn_" → `DynIllust`
///   2. _animationName == "Relax" OR skel name starts with "build_" → Building
///   3. owning `GameObject` named "Front"/"Back" → `BattleFront`/`BattleBack`
///   4. fallback: atlas front count (f_, c_) >= back count (b_) → `BattleFront`, else `BattleBack`
///
/// Step 3 is required for correctness: front and back battle skeletons often
/// share a single atlas (e.g. `char_1048_orchd2`), so the atlas heuristic
/// classifies both the same way and one overwrites the other on export.
fn classify_spine(
    skel_name: &str,
    anim_name: &str,
    game_object_name: &str,
    atlas_text: &str,
) -> SpineCategory {
    let name_lower = skel_name.to_lowercase();

    // 1. Dynamic illustration
    if name_lower.starts_with("dyn_") {
        return SpineCategory::DynIllust;
    }

    // 2. Building: "Relax" animation or build_ prefix
    if anim_name == "Relax" || name_lower.starts_with("build_") {
        return SpineCategory::Building;
    }

    // 3. Battle spines hang off GameObjects literally named "Front"/"Back"
    if game_object_name.eq_ignore_ascii_case("front") {
        return SpineCategory::BattleFront;
    }
    if game_object_name.eq_ignore_ascii_case("back") {
        return SpineCategory::BattleBack;
    }

    // 4. Fallback: front vs back based on atlas region prefixes
    let atlas_lower = atlas_text.to_lowercase();
    let front_count = atlas_lower.matches("\nf_").count() + atlas_lower.matches("\nc_").count();
    let back_count = atlas_lower.matches("\nb_").count();

    if front_count >= back_count {
        SpineCategory::BattleFront
    } else {
        SpineCategory::BattleBack
    }
}

/// Export organized spine assets. Returns count of exported files.
///
/// `char_name` is the destination directory for every asset (operator bundles
/// hold one character). Pass `None` for multi-enemy pack bundles: each asset
/// then derives its own directory from its skel name via [`enemy_dir_name`],
/// grouping form variants (`enemy_1000_gopro_2`) under the base enemy id.
#[must_use]
pub fn export_spine_assets(
    spine_assets: &[SpineAsset],
    output_dir: &Path,
    char_name: Option<&str>,
    resources: &HashMap<String, Vec<u8>>,
) -> usize {
    let mut count = 0;

    for asset in spine_assets {
        let dir_name = char_name.map_or_else(
            || enemy_dir_name(&asset.name),
            std::string::ToString::to_string,
        );
        let spine_dir = output_dir
            .join("spine")
            .join(asset.category.to_string())
            .join(dir_name);

        if std::fs::create_dir_all(&spine_dir).is_err() {
            continue;
        }

        // Write skel
        let skel_path = spine_dir.join(format!("{}.skel", asset.name));
        if std::fs::write(&skel_path, &asset.skel_data).is_ok() {
            count += 1;
        }

        // Write atlas
        let atlas_path = spine_dir.join(format!("{}.atlas", asset.name));
        if std::fs::write(&atlas_path, &asset.atlas_text).is_ok() {
            count += 1;
        }

        // Decode textures and apply alpha merging
        let mut decoded = HashMap::new();
        for (_, tex_val) in &asset.textures {
            match decode_texture_object(tex_val, resources) {
                Ok(Some(tex)) => {
                    decoded.insert(tex.name.clone(), tex);
                }
                Ok(None) => {}
                Err(e) => {
                    let name = tex_val["m_Name"].as_str().unwrap_or("?");
                    eprintln!("  error decoding spine texture {name}: {e}");
                }
            }
        }
        count += alpha_merge::merge_and_export(decoded, &spine_dir);
    }

    count
}

/// Derive the output directory for an enemy spine asset from its skel stem.
///
/// Enemy skel names follow `enemy_<num>_<tag>[_<form>]` where a trailing
/// `_<digits>` suffix marks an alternate form of the same enemy (e.g.
/// `enemy_1000_gopro_2` is form 2 of `enemy_1000_gopro`). Forms are grouped
/// under the base enemy id so downstream skin resolution can expose them as
/// variants of one enemy.
#[must_use]
pub fn enemy_dir_name(asset_name: &str) -> String {
    if let Some((base, suffix)) = asset_name.rsplit_once('_')
        && base.starts_with("enemy_")
        && !suffix.is_empty()
        && suffix.chars().all(|c| c.is_ascii_digit())
    {
        return base.to_string();
    }
    asset_name.to_string()
}

/// Derive the character name from a bundle subdirectory path.
#[must_use]
pub fn char_name_from_bundle(bundle_subdir: &Path) -> String {
    bundle_subdir.file_name().map_or_else(
        || "unknown".to_string(),
        |n| n.to_string_lossy().to_string(),
    )
}
