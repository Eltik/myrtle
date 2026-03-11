use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::Path;

use base64::Engine;
use serde_json::Value;

use super::texture::export_texture;

/// Spine animation category based on asset naming/content
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpineCategory {
    BattleFront,
    BattleBack,
    Building,
    DynIllust,
}

impl fmt::Display for SpineCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpineCategory::BattleFront => write!(f, "BattleFront"),
            SpineCategory::BattleBack => write!(f, "BattleBack"),
            SpineCategory::Building => write!(f, "Building"),
            SpineCategory::DynIllust => write!(f, "DynIllust"),
        }
    }
}

/// A grouped spine asset with properly-paired skel + atlas + textures
pub struct SpineAsset {
    /// Base name (e.g., "char_002_amiya" or "build_char_002_amiya")
    pub name: String,
    /// Raw skel binary data (decoded from base64)
    pub skel_data: Vec<u8>,
    /// Atlas text content
    pub atlas_text: String,
    /// Resolved texture objects (path_id → Value), keyed by texture name
    pub textures: Vec<(String, Value)>,
    /// Classification
    pub category: SpineCategory,
}

/// Check if a bundle path is eligible for spine extraction.
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

/// Extract a path_id from a JSON reference like {"m_FileID": 0, "m_PathID": 12345}
fn get_path_id(val: &Value) -> Option<i64> {
    val.get("m_PathID").and_then(|v| v.as_i64())
}

/// Collect spine assets using MonoBehaviour reference graph traversal.
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
/// Returns (spine_assets, claimed_path_ids) where claimed_path_ids contains
/// path_ids of all objects consumed by spine extraction.
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

        // Follow skeletonDataAsset → SkeletonData MonoBehaviour
        let skel_data_pid = match mecanim_val.get("skeletonDataAsset").and_then(get_path_id) {
            Some(pid) if pid != 0 => pid,
            _ => continue,
        };

        let skel_data_val = match all_objects.get(&skel_data_pid) {
            Some((114, val)) => val,
            _ => continue,
        };

        // Follow skeletonJSON → TextAsset (.skel)
        let skel_text_pid = match skel_data_val.get("skeletonJSON").and_then(get_path_id) {
            Some(pid) if pid != 0 => pid,
            _ => continue,
        };

        let skel_text_val = match all_objects.get(&skel_text_pid) {
            Some((49, val)) => val,
            _ => continue,
        };

        let skel_name = skel_text_val["m_Name"].as_str().unwrap_or("");
        let skel_script = skel_text_val["m_Script"].as_str().unwrap_or("");

        // Decode skel binary from base64
        let skel_bytes = if let Some(b64) = skel_script.strip_prefix("base64:") {
            match base64::engine::general_purpose::STANDARD.decode(b64) {
                Ok(d) => d,
                Err(_) => continue,
            }
        } else {
            continue;
        };

        // Follow atlasAssets[0] → Atlas MonoBehaviour
        let atlas_mono_pid = match skel_data_val
            .get("atlasAssets")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(get_path_id)
        {
            Some(pid) if pid != 0 => pid,
            _ => continue,
        };

        let atlas_mono_val = match all_objects.get(&atlas_mono_pid) {
            Some((114, val)) => val,
            _ => continue,
        };

        // Follow atlasFile → TextAsset (.atlas)
        let atlas_text_pid = match atlas_mono_val.get("atlasFile").and_then(get_path_id) {
            Some(pid) if pid != 0 => pid,
            _ => continue,
        };

        let atlas_text_val = match all_objects.get(&atlas_text_pid) {
            Some((49, val)) => val,
            _ => continue,
        };

        let atlas_script = atlas_text_val["m_Script"].as_str().unwrap_or("");
        if atlas_script.starts_with("base64:") || !atlas_script.contains(".png") {
            continue;
        }
        let atlas_text = atlas_script.to_string();

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
                            let tex_name =
                                tex_val["m_Name"].as_str().unwrap_or("unnamed").to_string();
                            if !texture_pids.contains(&tex_pid) {
                                texture_pids.push(tex_pid);
                                textures.push((tex_name, tex_val.clone()));
                            }
                        }
                    }
                }

                claimed.insert(mat_pid);
            }
        }

        // Classify the spine asset
        let base_name = skel_name.strip_suffix(".skel").unwrap_or(skel_name);
        let category = classify_spine(base_name, anim_name, &skel_bytes, &atlas_text);

        // Claim all path_ids in this spine instance
        claimed.insert(*mecanim_pid);
        claimed.insert(skel_data_pid);
        claimed.insert(skel_text_pid);
        claimed.insert(atlas_mono_pid);
        claimed.insert(atlas_text_pid);
        for pid in &texture_pids {
            claimed.insert(*pid);
        }

        assets.push(SpineAsset {
            name: base_name.to_string(),
            skel_data: skel_bytes,
            atlas_text,
            textures,
            category,
        });
    }

    (assets, claimed)
}

/// Classify a spine asset into a category.
/// Matches the old Python logic:
///   1. skel name starts with "dyn_" → DynIllust
///   2. _animationName == "Relax" OR skel name starts with "build_" → Building
///   3. atlas front count (f_, c_) >= back count (b_) → BattleFront, else BattleBack
fn classify_spine(
    skel_name: &str,
    anim_name: &str,
    _skel_data: &[u8],
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

    // 3. Front vs back based on atlas region prefixes
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
pub fn export_spine_assets(
    spine_assets: &[SpineAsset],
    output_dir: &Path,
    char_name: &str,
    resources: &HashMap<String, Vec<u8>>,
) -> usize {
    let mut count = 0;

    for asset in spine_assets {
        let spine_dir = output_dir
            .join("spine")
            .join(asset.category.to_string())
            .join(char_name);

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

        // Export textures (already resolved by path_id, no name collisions)
        for (_, tex_val) in &asset.textures {
            match export_texture(tex_val, &spine_dir, resources) {
                Ok(()) => count += 1,
                Err(e) => {
                    let name = tex_val["m_Name"].as_str().unwrap_or("?");
                    eprintln!("  error exporting spine texture {name}: {e}");
                }
            }
        }
    }

    count
}

/// Derive the character name from a bundle subdirectory path.
pub fn char_name_from_bundle(bundle_subdir: &Path) -> String {
    bundle_subdir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unity::bundle::BundleFile;
    use crate::unity::object_reader::read_object;
    use crate::unity::serialized_file::SerializedFile;

    type ObjectMap = HashMap<i64, (i32, Value)>;
    type ResourceMap = HashMap<String, Vec<u8>>;

    /// Helper: parse a bundle and collect all objects indexed by path_id
    fn load_all_objects(bundle_path: &Path) -> (ObjectMap, ResourceMap) {
        let data = std::fs::read(bundle_path).expect("failed to read bundle");
        let bundle = BundleFile::parse(data).expect("failed to parse bundle");

        let mut all_objects = HashMap::new();
        let mut resources = HashMap::new();

        for entry in &bundle.files {
            if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
                let filename = entry.path.rsplit('/').next().unwrap_or(&entry.path);
                resources.insert(filename.to_string(), entry.data.clone());
                continue;
            }
            let sf = match SerializedFile::parse(entry.data.clone()) {
                Ok(sf) => sf,
                Err(_) => continue,
            };
            for obj in &sf.objects {
                if let Ok(val) = read_object(&sf, obj) {
                    all_objects.insert(obj.path_id, (obj.class_id, val));
                }
            }
        }

        (all_objects, resources)
    }

    #[test]
    fn test_collect_spine_via_monobehaviour() {
        let bundle_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../downloader/ArkAssets/chararts/char_002_amiya.ab");

        if !bundle_path.exists() {
            eprintln!("SKIP: bundle not found at {}", bundle_path.display());
            return;
        }

        let (all_objects, _resources) = load_all_objects(&bundle_path);
        let (spine_assets, claimed) = collect_spine_assets(&all_objects);

        println!("\n=== collect_spine_assets via MonoBehaviour ===");
        println!("  spine_assets: {}", spine_assets.len());
        println!("  claimed path_ids: {}", claimed.len());

        for sa in &spine_assets {
            println!(
                "  SpineAsset: name={:?}, category={}, skel_len={}, atlas_len={}, textures={}",
                sa.name,
                sa.category,
                sa.skel_data.len(),
                sa.atlas_text.len(),
                sa.textures
                    .iter()
                    .map(|(n, _)| n.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        assert!(
            spine_assets.len() >= 3,
            "char_002_amiya should have 3 spine instances"
        );

        // Verify we have at least one of each expected category
        let categories: Vec<_> = spine_assets.iter().map(|sa| sa.category).collect();
        assert!(
            categories.contains(&SpineCategory::BattleFront),
            "should have BattleFront"
        );
        assert!(
            categories.contains(&SpineCategory::Building),
            "should have Building"
        );
        // BattleBack is only present for certain characters, so don't assert it here
    }

    #[test]
    fn debug_material_textures() {
        let bundle_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../downloader/ArkAssets/chararts/char_002_amiya.ab");

        if !bundle_path.exists() {
            eprintln!("SKIP: bundle not found at {}", bundle_path.display());
            return;
        }

        let (all_objects, _resources) = load_all_objects(&bundle_path);

        let material_pids: &[i64] = &[
            1554564069455891555,
            1806088666543831737,
            8826680919028358421,
        ];

        println!("\n=== debug_material_textures ===");
        println!("Total objects in bundle: {}", all_objects.len());

        // First, list all Material (class_id=21) objects in the bundle
        let materials: Vec<_> = all_objects
            .iter()
            .filter(|(_, (cid, _))| *cid == 21)
            .collect();
        println!(
            "\nAll Material objects in bundle ({} total):",
            materials.len()
        );
        for (pid, (cid, val)) in &materials {
            let name = val.get("m_Name").and_then(|v| v.as_str()).unwrap_or("?");
            println!("  path_id={}, class_id={}, name={}", pid, cid, name);
        }

        for &pid in material_pids {
            println!("\n--- Material path_id={} ---", pid);
            match all_objects.get(&pid) {
                Some((class_id, val)) => {
                    println!("  class_id: {}", class_id);
                    let name = val.get("m_Name").and_then(|v| v.as_str()).unwrap_or("?");
                    println!("  m_Name: {}", name);

                    // Print full m_TexEnvs
                    if let Some(saved_props) = val.get("m_SavedProperties") {
                        if let Some(tex_envs) = saved_props.get("m_TexEnvs") {
                            println!("  m_TexEnvs (full JSON):");
                            println!(
                                "{}",
                                serde_json::to_string_pretty(tex_envs).unwrap_or_default()
                            );

                            // Analyze structure
                            if let Some(arr) = tex_envs.as_array() {
                                println!("  m_TexEnvs is an array with {} entries", arr.len());
                                for (i, entry) in arr.iter().enumerate() {
                                    println!("    entry[{}] type: {}", i, json_type(entry));
                                    if let Some(inner_arr) = entry.as_array() {
                                        println!(
                                            "    entry[{}] is array of len {}",
                                            i,
                                            inner_arr.len()
                                        );
                                        for (j, elem) in inner_arr.iter().enumerate() {
                                            println!(
                                                "      [{}] type={}, preview={}",
                                                j,
                                                json_type(elem),
                                                truncate_json(elem, 200)
                                            );
                                        }
                                    } else if let Some(obj) = entry.as_object() {
                                        println!(
                                            "    entry[{}] is object with keys: {:?}",
                                            i,
                                            obj.keys().collect::<Vec<_>>()
                                        );
                                        // Look for texture refs in object values
                                        for (key, v) in obj {
                                            if let Some(tex) = v.get("m_Texture") {
                                                let tex_pid =
                                                    tex.get("m_PathID").and_then(|p| p.as_i64());
                                                println!(
                                                    "      {}.m_Texture.m_PathID = {:?}",
                                                    key, tex_pid
                                                );
                                                if let Some(tp) = tex_pid {
                                                    match all_objects.get(&tp) {
                                                        Some((cid, tv)) => {
                                                            let tn = tv
                                                                .get("m_Name")
                                                                .and_then(|n| n.as_str())
                                                                .unwrap_or("?");
                                                            println!(
                                                                "        -> exists! class_id={}, name={}",
                                                                cid, tn
                                                            );
                                                        }
                                                        None => println!(
                                                            "        -> NOT FOUND in all_objects"
                                                        ),
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Also try the [name, {m_Texture: ...}] pattern
                                    if let Some(inner_arr) = entry.as_array()
                                        && inner_arr.len() >= 2
                                    {
                                        let tex_name = inner_arr[0].as_str().unwrap_or("?");
                                        if let Some(tex_ref) = inner_arr[1].get("m_Texture") {
                                            let tex_pid =
                                                tex_ref.get("m_PathID").and_then(|p| p.as_i64());
                                            println!(
                                                "    [name={}, m_Texture.m_PathID={:?}]",
                                                tex_name, tex_pid
                                            );
                                            if let Some(tp) = tex_pid {
                                                match all_objects.get(&tp) {
                                                    Some((cid, tv)) => {
                                                        let tn = tv
                                                            .get("m_Name")
                                                            .and_then(|n| n.as_str())
                                                            .unwrap_or("?");
                                                        println!(
                                                            "        -> exists! class_id={}, name={}",
                                                            cid, tn
                                                        );
                                                    }
                                                    None => println!(
                                                        "        -> NOT FOUND in all_objects"
                                                    ),
                                                }
                                            }
                                        }
                                    }
                                }
                            } else if let Some(obj) = tex_envs.as_object() {
                                println!(
                                    "  m_TexEnvs is an OBJECT with keys: {:?}",
                                    obj.keys().collect::<Vec<_>>()
                                );
                            } else {
                                println!("  m_TexEnvs is type: {}", json_type(tex_envs));
                            }
                        } else {
                            println!("  m_SavedProperties exists but NO m_TexEnvs!");
                            println!(
                                "  m_SavedProperties keys: {:?}",
                                saved_props
                                    .as_object()
                                    .map(|o| o.keys().collect::<Vec<_>>())
                            );
                        }
                    } else {
                        println!("  NO m_SavedProperties!");
                        println!(
                            "  top-level keys: {:?}",
                            val.as_object().map(|o| o.keys().collect::<Vec<_>>())
                        );
                    }
                }
                None => {
                    println!("  NOT FOUND in all_objects!");
                }
            }
        }
    }

    fn json_type(v: &Value) -> &'static str {
        match v {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }

    fn truncate_json(v: &Value, max_len: usize) -> String {
        let s = serde_json::to_string(v).unwrap_or_default();
        if s.len() > max_len {
            format!("{}...", &s[..max_len])
        } else {
            s
        }
    }
}
