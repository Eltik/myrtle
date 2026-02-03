use anyhow::Result;
use image::{ImageBuffer, Rgba, RgbaImage};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::SystemTime;
use sysinfo::{Pid, System};
use unity_rs::export::texture_2d_converter::parse_image_data;
use unity_rs::{BuildTarget, TextureFormat};
use unity_rs::{ClassIDType, Environment};
use walkdir::WalkDir;

use crate::utils::is_ab_file;

/// Manifest entry for tracking extracted files
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManifestEntry {
    modified: u64, // Unix timestamp
    assets_count: usize,
}

/// Extraction manifest for incremental processing
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ExtractionManifest {
    version: u32,
    entries: HashMap<String, ManifestEntry>,
}

impl ExtractionManifest {
    fn load(path: &Path) -> Self {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(manifest) = serde_json::from_str(&content) {
                    return manifest;
                }
            }
        }
        Self {
            version: 1,
            entries: HashMap::new(),
        }
    }

    fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn should_process(&self, file: &Path) -> bool {
        let key = file.to_string_lossy().to_string();

        // Get file modification time
        let modified = match std::fs::metadata(file) {
            Ok(meta) => meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0),
            Err(_) => return true,
        };

        // Check if file is in manifest and hasn't changed
        match self.entries.get(&key) {
            Some(entry) => entry.modified != modified,
            None => true,
        }
    }

    fn update(&mut self, file: &Path, assets_count: usize) {
        let key = file.to_string_lossy().to_string();

        let modified = std::fs::metadata(file)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.entries.insert(
            key,
            ManifestEntry {
                modified,
                assets_count,
            },
        );
    }
}

/// Spine asset types based on Arknights naming conventions
#[derive(Debug, Clone, PartialEq)]
pub enum SpineAssetType {
    Unknown,
    Building,    // [SkeletonGraphic] Building models
    BattleFront, // [SkeletonAnimation] Battle front-facing
    BattleBack,  // [SkeletonAnimation] Battle back-facing
    DynIllust,   // [SkeletonGraphic] Dynamic illustrations
}

impl SpineAssetType {
    /// Determine Spine asset type from name, atlas content, and skeleton data
    /// Uses Python's logic: check skeleton data for "Default" AND "Relax" animations for Building detection
    /// Falls back to atlas content for Front vs Back determination
    pub fn from_name_atlas_and_skel(
        name: &str,
        atlas_content: Option<&str>,
        skel_data: Option<&[u8]>,
    ) -> Self {
        let lower_name = name.to_lowercase();

        // DynIllust: name starts with dyn_
        if lower_name.starts_with("dyn_") {
            return SpineAssetType::DynIllust;
        }

        // Building: name starts with build_
        if lower_name.starts_with("build_") {
            return SpineAssetType::Building;
        }

        // Check skeleton data for "Default" AND "Relax" animations (Building indicator)
        // This matches Python's SpineSkeletonHandler.guess_type() logic:
        // if "default" in anim_names and "relax" in anim_names: return BUILDING
        if let Some(skel) = skel_data {
            let has_default = Self::contains_animation_name(skel, b"Default")
                || Self::contains_animation_name(skel, b"default");
            let has_relax = Self::contains_animation_name(skel, b"Relax")
                || Self::contains_animation_name(skel, b"relax");

            if has_default && has_relax {
                log::debug!(
                    "Detected Building type for '{}' via skeleton animations (Default + Relax)",
                    name
                );
                return SpineAssetType::Building;
            }
        }

        // Fall back to atlas content analysis for Front vs Back
        if let Some(atlas) = atlas_content {
            // Count animation prefixes to determine Front vs Back
            let atlas_lower = atlas.to_lowercase();
            let front_count =
                atlas_lower.matches("\nf_").count() + atlas_lower.matches("\nc_").count();
            let back_count = atlas_lower.matches("\nb_").count();

            if front_count >= back_count {
                return SpineAssetType::BattleFront;
            } else {
                return SpineAssetType::BattleBack;
            }
        }

        SpineAssetType::Unknown
    }

    /// Check if skeleton data contains an animation name
    /// This does a simple byte search which works for both JSON and binary Spine formats
    fn contains_animation_name(skel_data: &[u8], name: &[u8]) -> bool {
        // For binary format, animation names are stored as length-prefixed strings
        // For JSON format, animation names appear in the "animations" object
        // A simple substring search works for both cases
        skel_data.windows(name.len()).any(|window| window == name)
    }

    /// Legacy function for backward compatibility
    pub fn from_name_and_atlas(name: &str, atlas_content: Option<&str>) -> Self {
        Self::from_name_atlas_and_skel(name, atlas_content, None)
    }

    /// Get directory name for this asset type
    pub fn directory_name(&self) -> &'static str {
        match self {
            SpineAssetType::Building => "Building",
            SpineAssetType::BattleFront => "BattleFront",
            SpineAssetType::BattleBack => "BattleBack",
            SpineAssetType::DynIllust => "DynIllust",
            SpineAssetType::Unknown => "",
        }
    }
}

/// Represents a Spine animation asset
pub struct SpineAsset {
    pub name: String,
    pub asset_type: SpineAssetType,
    pub skel_data: Option<Vec<u8>>,
    pub atlas_data: Option<String>,
    pub textures: Vec<SpineTexture>,
    pub skel_path_id: Option<i64>, // For texture path_id matching
}

pub struct SpineTexture {
    pub name: String,
    pub rgb_data: Option<image::RgbaImage>,
    pub alpha_data: Option<image::GrayImage>,
    pub combined: Option<image::RgbaImage>,
}

/// Resource extracted from a Unity Environment
pub struct Resource {
    pub name: String,
    pub sprites: Vec<serde_json::Value>,
    pub texture2ds: Vec<serde_json::Value>,
    pub textassets: Vec<serde_json::Value>,
    pub audioclips: Vec<serde_json::Value>,
    pub materials: Vec<serde_json::Value>,
    pub monobehaviors: Vec<serde_json::Value>,
}

impl Resource {
    /// Load and categorize objects from a Unity environment
    pub fn from_environment(env_rc: &Rc<RefCell<Environment>>) -> Result<Self> {
        let env = env_rc.borrow();
        let objects = env.objects();

        let mut resource = Resource {
            name: String::new(),
            sprites: Vec::new(),
            texture2ds: Vec::new(),
            textassets: Vec::new(),
            audioclips: Vec::new(),
            materials: Vec::new(),
            monobehaviors: Vec::new(),
        };

        for mut obj in objects {
            match obj.obj_type {
                ClassIDType::Sprite => {
                    if let Ok(data) = obj.read(false) {
                        resource.sprites.push(data);
                    }
                }
                ClassIDType::Texture2D => {
                    if let Ok(data) = obj.read(false) {
                        resource.texture2ds.push(data);
                    }
                }
                ClassIDType::TextAsset => {
                    if let Ok(data) = obj.read(false) {
                        resource.textassets.push(data);
                    }
                }
                ClassIDType::AudioClip => {
                    if let Ok(data) = obj.read(false) {
                        resource.audioclips.push(data);
                    }
                }
                ClassIDType::Material => {
                    if let Ok(data) = obj.read(false) {
                        resource.materials.push(data);
                    }
                }
                ClassIDType::MonoBehaviour => {
                    if let Ok(data) = obj.read(false) {
                        resource.monobehaviors.push(data);
                    }
                }
                ClassIDType::AssetBundle => {
                    // Extract bundle name
                    if let Ok(data) = obj.read(false) {
                        if let Some(name) = data.get("m_Name").and_then(|v| v.as_str()) {
                            resource.name = name.to_string();
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(resource)
    }
}

/// Parse atlas file to extract texture filenames
fn parse_atlas_textures(atlas_content: &str) -> Vec<String> {
    let mut textures = Vec::new();
    let lines = atlas_content.lines().peekable();

    for line in lines {
        let trimmed = line.trim();
        // Atlas format: texture filename is on its own line ending with .png
        // followed by size:, format:, filter:, etc.
        if trimmed.ends_with(".png") {
            textures.push(trimmed.to_string());
        }
    }

    textures
}

/// Parse atlas file to extract the expected texture size (width, height)
fn parse_atlas_size(atlas_content: &str) -> Option<(u32, u32)> {
    for line in atlas_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("size:") {
            // Format: "size: 256,256" or "size:256,256"
            let size_str = trimmed.trim_start_matches("size:").trim();
            let parts: Vec<&str> = size_str.split(',').collect();
            if parts.len() == 2 {
                if let (Ok(w), Ok(h)) = (
                    parts[0].trim().parse::<u32>(),
                    parts[1].trim().parse::<u32>(),
                ) {
                    return Some((w, h));
                }
            }
        }
    }
    None
}

/// Validate that all required textures exist
fn validate_spine_textures(
    required: &[String],
    available: &std::collections::HashMap<String, unity_rs::files::object_reader::ObjectReader<()>>,
) -> (Vec<String>, Vec<String>) {
    let mut found = Vec::new();
    let mut missing = Vec::new();

    for tex_name in required {
        // Remove .png extension for matching
        let base = tex_name.trim_end_matches(".png");

        // Check various naming patterns
        let exists = available.contains_key(base)
            || available.contains_key(tex_name)
            || available.contains_key(&format!("{}[alpha]", base))
            || available.keys().any(|k| k.starts_with(base));

        if exists {
            found.push(tex_name.clone());
        } else {
            missing.push(tex_name.clone());
        }
    }

    (found, missing)
}

/// Find CharacterAnimator MonoBehaviours and build a mapping of TextAsset path_id -> SpineAssetType
/// This is the authoritative way to determine front vs back for battle animations
fn find_character_animator_types(
    env_rc: &Rc<RefCell<Environment>>,
) -> (
    std::collections::HashSet<i64>,
    std::collections::HashSet<i64>,
    std::collections::HashMap<i64, i64>, // skel_path_id -> atlas_path_id mapping
    std::collections::HashMap<i64, Vec<i64>>, // skel_path_id -> texture_path_ids mapping
) {
    let mut front_skel_path_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let mut back_skel_path_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let mut skel_to_atlas: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    let mut skel_to_textures: std::collections::HashMap<i64, Vec<i64>> =
        std::collections::HashMap::new();

    let env = env_rc.borrow();

    // Build a map of path_id -> ObjectReader for all objects
    let mut all_objects: std::collections::HashMap<
        i64,
        unity_rs::files::object_reader::ObjectReader<()>,
    > = std::collections::HashMap::new();
    for obj in env.objects() {
        all_objects.insert(obj.path_id, obj);
    }

    // Find CharacterAnimator MonoBehaviours (have _front and _back keys)
    for mut obj in env.objects() {
        if obj.obj_type != ClassIDType::MonoBehaviour {
            continue;
        }

        // Try to read TypeTree
        let node_clone = obj.serialized_type.as_ref().and_then(|st| st.node.clone());
        if node_clone.is_none() {
            continue;
        }

        obj.reset();
        let tree = match obj.read_typetree(node_clone, false, false) {
            Ok(t) => t,
            Err(_) => continue,
        };

        // Check for CharacterAnimator keys
        let has_front = tree.get("_front").is_some();
        let has_back = tree.get("_back").is_some();
        if !has_front || !has_back {
            continue;
        }

        log::debug!("Found CharacterAnimator at path_id {}", obj.path_id);

        // Trace front reference chain: _front.skeleton -> SkeletonAnimation -> skeletonDataAsset -> skeletonJSON
        if let Some(front) = tree.get("_front") {
            if let Some(front_skel_path_id) = trace_to_textasset(
                &all_objects,
                front,
                "skeleton",
                "skeletonDataAsset",
                "skeletonJSON",
            ) {
                log::debug!("  Front skeletonJSON path_id: {}", front_skel_path_id);
                front_skel_path_ids.insert(front_skel_path_id);

                // Also trace to atlas for this front skeleton
                if let Some(front_atlas_path_id) =
                    trace_to_atlas_textasset(&all_objects, front, "skeleton", "skeletonDataAsset")
                {
                    log::debug!("  Front atlas path_id: {}", front_atlas_path_id);
                    skel_to_atlas.insert(front_skel_path_id, front_atlas_path_id);
                }

                // Trace to textures for this front skeleton
                let front_texture_path_ids =
                    trace_to_texture_pathids(&all_objects, front, "skeleton", "skeletonDataAsset");
                if !front_texture_path_ids.is_empty() {
                    log::debug!("  Front texture path_ids: {:?}", front_texture_path_ids);
                    skel_to_textures.insert(front_skel_path_id, front_texture_path_ids);
                }
            }
        }

        // Trace back reference chain
        if let Some(back) = tree.get("_back") {
            if let Some(back_skel_path_id) = trace_to_textasset(
                &all_objects,
                back,
                "skeleton",
                "skeletonDataAsset",
                "skeletonJSON",
            ) {
                log::debug!("  Back skeletonJSON path_id: {}", back_skel_path_id);
                back_skel_path_ids.insert(back_skel_path_id);

                // Also trace to atlas for this back skeleton
                if let Some(back_atlas_path_id) =
                    trace_to_atlas_textasset(&all_objects, back, "skeleton", "skeletonDataAsset")
                {
                    log::debug!("  Back atlas path_id: {}", back_atlas_path_id);
                    skel_to_atlas.insert(back_skel_path_id, back_atlas_path_id);
                }

                // Trace to textures for this back skeleton
                let back_texture_path_ids =
                    trace_to_texture_pathids(&all_objects, back, "skeleton", "skeletonDataAsset");
                if !back_texture_path_ids.is_empty() {
                    log::debug!("  Back texture path_ids: {:?}", back_texture_path_ids);
                    skel_to_textures.insert(back_skel_path_id, back_texture_path_ids);
                }
            }
        }
    }

    log::info!(
        "CharacterAnimator found {} front, {} back skeleton path_ids, {} skel->atlas mappings, {} skel->texture mappings",
        front_skel_path_ids.len(),
        back_skel_path_ids.len(),
        skel_to_atlas.len(),
        skel_to_textures.len()
    );

    (
        front_skel_path_ids,
        back_skel_path_ids,
        skel_to_atlas,
        skel_to_textures,
    )
}

/// Helper to trace the reference chain from CharacterAnimator to TextAsset path_id
/// Chain: _front/_back -> skeleton -> skeletonDataAsset -> skeletonJSON
fn trace_to_textasset(
    all_objects: &std::collections::HashMap<i64, unity_rs::files::object_reader::ObjectReader<()>>,
    direction_value: &serde_json::Value,
    skel_key: &str,
    sda_key: &str,
    json_key: &str,
) -> Option<i64> {
    // Get skeleton reference (SkeletonAnimation MonoBehaviour)
    let skel_anim_path_id = direction_value.get(skel_key)?.get("m_PathID")?.as_i64()?;

    // Read SkeletonAnimation MonoBehaviour
    let mut skel_anim_obj = all_objects.get(&skel_anim_path_id)?.clone();
    let node = skel_anim_obj.serialized_type.as_ref()?.node.clone()?;
    skel_anim_obj.reset();
    let skel_anim_tree = skel_anim_obj.read_typetree(Some(node), false, false).ok()?;

    // Get skeletonDataAsset reference
    let sda_path_id = skel_anim_tree.get(sda_key)?.get("m_PathID")?.as_i64()?;

    // Read SkeletonDataAsset MonoBehaviour
    let mut sda_obj = all_objects.get(&sda_path_id)?.clone();
    let sda_node = sda_obj.serialized_type.as_ref()?.node.clone()?;
    sda_obj.reset();
    let sda_tree = sda_obj.read_typetree(Some(sda_node), false, false).ok()?;

    // Get skeletonJSON TextAsset path_id
    let json_path_id = sda_tree.get(json_key)?.get("m_PathID")?.as_i64()?;

    Some(json_path_id)
}

/// Helper to trace the reference chain from CharacterAnimator to Atlas TextAsset path_id
/// Chain: _front/_back -> skeleton -> skeletonDataAsset -> atlasAssets[0] -> atlasFile
fn trace_to_atlas_textasset(
    all_objects: &std::collections::HashMap<i64, unity_rs::files::object_reader::ObjectReader<()>>,
    direction_value: &serde_json::Value,
    skel_key: &str,
    sda_key: &str,
) -> Option<i64> {
    // Get skeleton reference (SkeletonAnimation MonoBehaviour)
    let skel_anim_path_id = direction_value.get(skel_key)?.get("m_PathID")?.as_i64()?;

    // Read SkeletonAnimation MonoBehaviour
    let mut skel_anim_obj = all_objects.get(&skel_anim_path_id)?.clone();
    let node = skel_anim_obj.serialized_type.as_ref()?.node.clone()?;
    skel_anim_obj.reset();
    let skel_anim_tree = skel_anim_obj.read_typetree(Some(node), false, false).ok()?;

    // Get skeletonDataAsset reference
    let sda_path_id = skel_anim_tree.get(sda_key)?.get("m_PathID")?.as_i64()?;

    // Read SkeletonDataAsset MonoBehaviour
    let mut sda_obj = all_objects.get(&sda_path_id)?.clone();
    let sda_node = sda_obj.serialized_type.as_ref()?.node.clone()?;
    sda_obj.reset();
    let sda_tree = sda_obj.read_typetree(Some(sda_node), false, false).ok()?;

    // Get atlasAssets array and get first element
    let atlas_assets = sda_tree.get("atlasAssets")?.as_array()?;
    if atlas_assets.is_empty() {
        return None;
    }
    let atlas_data_path_id = atlas_assets[0].get("m_PathID")?.as_i64()?;

    // Read AtlasData MonoBehaviour
    let mut atlas_data_obj = all_objects.get(&atlas_data_path_id)?.clone();
    let atlas_data_node = atlas_data_obj.serialized_type.as_ref()?.node.clone()?;
    atlas_data_obj.reset();
    let atlas_data_tree = atlas_data_obj
        .read_typetree(Some(atlas_data_node), false, false)
        .ok()?;

    // Get atlasFile TextAsset path_id
    let atlas_file_path_id = atlas_data_tree
        .get("atlasFile")?
        .get("m_PathID")?
        .as_i64()?;

    Some(atlas_file_path_id)
}

/// Helper to trace the reference chain from CharacterAnimator to Texture2D path_ids
/// Chain: _front/_back -> skeleton -> skeletonDataAsset -> atlasAssets[0] -> AtlasData -> materials -> Material -> mainTexture
fn trace_to_texture_pathids(
    all_objects: &std::collections::HashMap<i64, unity_rs::files::object_reader::ObjectReader<()>>,
    direction_value: &serde_json::Value,
    skel_key: &str,
    sda_key: &str,
) -> Vec<i64> {
    let mut texture_path_ids = Vec::new();

    // Get skeleton reference (SkeletonAnimation MonoBehaviour)
    let skel_anim_path_id = match direction_value
        .get(skel_key)
        .and_then(|v| v.get("m_PathID"))
        .and_then(|v| v.as_i64())
    {
        Some(id) => id,
        None => {
            log::trace!("trace_to_texture_pathids: no skeleton reference found");
            return texture_path_ids;
        }
    };

    // Read SkeletonAnimation MonoBehaviour
    let mut skel_anim_obj = match all_objects.get(&skel_anim_path_id) {
        Some(obj) => obj.clone(),
        None => {
            log::trace!(
                "trace_to_texture_pathids: skeleton object {} not found",
                skel_anim_path_id
            );
            return texture_path_ids;
        }
    };
    let node = match skel_anim_obj
        .serialized_type
        .as_ref()
        .and_then(|st| st.node.clone())
    {
        Some(n) => n,
        None => {
            log::trace!("trace_to_texture_pathids: no node for skeleton object");
            return texture_path_ids;
        }
    };
    skel_anim_obj.reset();
    let skel_anim_tree = match skel_anim_obj.read_typetree(Some(node), false, false) {
        Ok(t) => t,
        Err(e) => {
            log::trace!(
                "trace_to_texture_pathids: failed to read skeleton typetree: {:?}",
                e
            );
            return texture_path_ids;
        }
    };

    // Get skeletonDataAsset reference
    let sda_path_id = match skel_anim_tree
        .get(sda_key)
        .and_then(|v| v.get("m_PathID"))
        .and_then(|v| v.as_i64())
    {
        Some(id) => id,
        None => {
            log::trace!(
                "trace_to_texture_pathids: no skeletonDataAsset reference. Keys: {:?}",
                skel_anim_tree
                    .as_object()
                    .map(|o| o.keys().collect::<Vec<_>>())
            );
            return texture_path_ids;
        }
    };

    log::trace!(
        "trace_to_texture_pathids: skeletonDataAsset path_id: {}",
        sda_path_id
    );

    // Read SkeletonDataAsset MonoBehaviour
    let mut sda_obj = match all_objects.get(&sda_path_id) {
        Some(obj) => obj.clone(),
        None => {
            log::trace!("trace_to_texture_pathids: skeletonDataAsset object not found");
            return texture_path_ids;
        }
    };
    let sda_node = match sda_obj
        .serialized_type
        .as_ref()
        .and_then(|st| st.node.clone())
    {
        Some(n) => n,
        None => {
            log::trace!("trace_to_texture_pathids: no node for skeletonDataAsset");
            return texture_path_ids;
        }
    };
    sda_obj.reset();
    let sda_tree = match sda_obj.read_typetree(Some(sda_node), false, false) {
        Ok(t) => t,
        Err(e) => {
            log::trace!(
                "trace_to_texture_pathids: failed to read skeletonDataAsset typetree: {:?}",
                e
            );
            return texture_path_ids;
        }
    };

    // Get atlasAssets array
    let atlas_assets = match sda_tree.get("atlasAssets").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => {
            log::trace!(
                "trace_to_texture_pathids: no atlasAssets array. Keys: {:?}",
                sda_tree.as_object().map(|o| o.keys().collect::<Vec<_>>())
            );
            return texture_path_ids;
        }
    };

    log::trace!(
        "trace_to_texture_pathids: found {} atlasAssets",
        atlas_assets.len()
    );

    // Process each AtlasData
    for atlas_asset_ref in atlas_assets {
        let atlas_data_path_id = match atlas_asset_ref.get("m_PathID").and_then(|v| v.as_i64()) {
            Some(id) => id,
            None => continue,
        };

        log::trace!(
            "trace_to_texture_pathids: processing AtlasData at path_id {}",
            atlas_data_path_id
        );

        // Read AtlasData MonoBehaviour
        let mut atlas_data_obj = match all_objects.get(&atlas_data_path_id) {
            Some(obj) => obj.clone(),
            None => {
                log::trace!("trace_to_texture_pathids: AtlasData object not found");
                continue;
            }
        };
        let atlas_data_node = match atlas_data_obj
            .serialized_type
            .as_ref()
            .and_then(|st| st.node.clone())
        {
            Some(n) => n,
            None => {
                log::trace!("trace_to_texture_pathids: no node for AtlasData");
                continue;
            }
        };
        atlas_data_obj.reset();
        let atlas_data_tree =
            match atlas_data_obj.read_typetree(Some(atlas_data_node), false, false) {
                Ok(t) => t,
                Err(e) => {
                    log::trace!(
                        "trace_to_texture_pathids: failed to read AtlasData typetree: {:?}",
                        e
                    );
                    continue;
                }
            };

        log::trace!(
            "trace_to_texture_pathids: AtlasData keys: {:?}",
            atlas_data_tree
                .as_object()
                .map(|o| o.keys().collect::<Vec<_>>())
        );

        // Get materials array from AtlasData
        if let Some(materials) = atlas_data_tree.get("materials").and_then(|v| v.as_array()) {
            log::trace!(
                "trace_to_texture_pathids: found {} materials",
                materials.len()
            );
            for mat_ref in materials {
                let mat_path_id = match mat_ref.get("m_PathID").and_then(|v| v.as_i64()) {
                    Some(id) => id,
                    None => continue,
                };

                log::trace!(
                    "trace_to_texture_pathids: processing Material at path_id {}",
                    mat_path_id
                );

                // Read Material
                let mut mat_obj = match all_objects.get(&mat_path_id) {
                    Some(obj) => obj.clone(),
                    None => {
                        log::trace!("trace_to_texture_pathids: Material object not found");
                        continue;
                    }
                };
                let mat_node = match mat_obj
                    .serialized_type
                    .as_ref()
                    .and_then(|st| st.node.clone())
                {
                    Some(n) => n,
                    None => {
                        log::trace!("trace_to_texture_pathids: no node for Material");
                        continue;
                    }
                };
                mat_obj.reset();
                let mat_tree = match mat_obj.read_typetree(Some(mat_node), false, false) {
                    Ok(t) => t,
                    Err(e) => {
                        log::trace!(
                            "trace_to_texture_pathids: failed to read Material typetree: {:?}",
                            e
                        );
                        continue;
                    }
                };

                log::trace!(
                    "trace_to_texture_pathids: Material keys: {:?}",
                    mat_tree.as_object().map(|o| o.keys().collect::<Vec<_>>())
                );

                // Get mainTexture from m_SavedProperties.m_TexEnvs
                if let Some(saved_props) = mat_tree.get("m_SavedProperties") {
                    if let Some(tex_envs) = saved_props.get("m_TexEnvs").and_then(|v| v.as_array())
                    {
                        log::trace!(
                            "trace_to_texture_pathids: found {} m_TexEnvs",
                            tex_envs.len()
                        );
                        for tex_env in tex_envs {
                            // tex_env is an Array: [name, {m_Texture: {...}, m_Scale: {...}, m_Offset: {...}}]
                            let tex_data = if let Some(arr) = tex_env.as_array() {
                                // Array format: [name, texture_data_obj]
                                arr.get(1)
                            } else {
                                // Object format: {first: name, second: texture_data_obj}
                                tex_env.get("second")
                            };

                            if let Some(tex_data) = tex_data {
                                if let Some(tex_ref) = tex_data.get("m_Texture") {
                                    if let Some(tex_path_id) =
                                        tex_ref.get("m_PathID").and_then(|v| v.as_i64())
                                    {
                                        if tex_path_id != 0 {
                                            log::trace!("trace_to_texture_pathids: found texture path_id: {}", tex_path_id);
                                            texture_path_ids.push(tex_path_id);
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        log::trace!("trace_to_texture_pathids: no m_TexEnvs in m_SavedProperties");
                    }
                } else {
                    log::trace!("trace_to_texture_pathids: no m_SavedProperties in Material");
                }
            }
        }
    }

    texture_path_ids
}

/// Extract Spine assets from MonoBehaviours
fn extract_spine_assets(env_rc: &Rc<RefCell<Environment>>, destdir: &Path) -> Result<usize> {
    // First, find CharacterAnimator-based type mappings (including skel->atlas and skel->texture path_id mappings)
    let (front_skel_path_ids, back_skel_path_ids, skel_to_atlas, skel_to_textures) =
        find_character_animator_types(env_rc);

    let env = env_rc.borrow();
    let mut saved_count = 0;

    // Collect all assets - use Vec to preserve duplicates with same name
    // Now also track path_id for CharacterAnimator-based type resolution
    let mut textassets: Vec<(String, i64, serde_json::Value)> = Vec::new();
    // Store ALL texture objects with their sizes, quality, and path_id, keyed by name
    // Each entry: (width, height, colored_pixels, path_id, ObjectReader)
    let mut texture_objects_by_size: std::collections::HashMap<
        String,
        Vec<(
            u32,
            u32,
            usize,
            i64,
            unity_rs::files::object_reader::ObjectReader<()>,
        )>,
    > = std::collections::HashMap::new();

    for mut obj in env.objects() {
        match obj.obj_type {
            ClassIDType::TextAsset => {
                if let Ok(data) = obj.read(false) {
                    if let Some(name) = data.get("m_Name").and_then(|v| v.as_str()) {
                        textassets.push((name.to_string(), obj.path_id, data));
                    }
                }
            }
            ClassIDType::Texture2D => {
                if let Ok(data) = obj.read(false) {
                    if let Some(name) = data.get("m_Name").and_then(|v| v.as_str()) {
                        // Get texture dimensions from metadata
                        let width =
                            data.get("m_Width").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        let height =
                            data.get("m_Height").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        // Decode texture to count colored pixels for quality comparison
                        let mut tex_obj = obj.clone();
                        let colored_pixels = if let Ok(tex_data) = tex_obj.read(false) {
                            if let Ok(mut texture) =
                                serde_json::from_value::<unity_rs::generated::Texture2D>(tex_data)
                            {
                                texture.object_reader = Some(Box::new(tex_obj.clone()));
                                if let Ok(img) = texture_to_image(&texture) {
                                    img.pixels()
                                        .filter(|p| p[0] as u32 + p[1] as u32 + p[2] as u32 > 30)
                                        .count()
                                } else {
                                    0
                                }
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                        texture_objects_by_size
                            .entry(name.to_string())
                            .or_default()
                            .push((width, height, colored_pixels, obj.path_id, obj.clone()));
                    }
                }
            }
            _ => {}
        }
    }

    // Build a simple name->object map for non-spine texture extraction (pick best by colored pixels)
    let mut texture_objects: std::collections::HashMap<
        String,
        unity_rs::files::object_reader::ObjectReader<()>,
    > = std::collections::HashMap::new();

    for (name, objects) in &texture_objects_by_size {
        if objects.len() == 1 {
            texture_objects.insert(name.clone(), objects[0].4.clone());
        } else {
            // Multiple textures with same name - pick best by colored pixels (already computed)
            let mut best_obj = None;
            let mut best_colored_pixels = 0usize;

            for (_, _, colored, _, tex_obj) in objects {
                if *colored > best_colored_pixels {
                    best_colored_pixels = *colored;
                    best_obj = Some(tex_obj.clone());
                }
            }

            if let Some(obj) = best_obj {
                log::debug!(
                    "Selected best texture '{}' with {} colored pixels",
                    name,
                    best_colored_pixels
                );
                texture_objects.insert(name.clone(), obj);
            }
        }
    }

    // Find Spine asset groups - process each skel+atlas pair separately
    // Key: (base_name, index) to handle multiple instances with same name
    let mut spine_assets: Vec<SpineAsset> = Vec::new();

    // First, collect all skel and atlas files with their content AND path_id
    // (name, path_id, data)
    let mut skels: Vec<(String, i64, Vec<u8>)> = Vec::new();
    let mut atlases: Vec<(String, i64, String)> = Vec::new();

    for (name, path_id, data) in &textassets {
        if name.ends_with(".skel") {
            let base_name = name.trim_end_matches(".skel").to_string();
            if let Some(script) = data.get("m_Script").and_then(|v| v.as_str()) {
                // Decode base64 if the skel data is base64-encoded
                let skel_bytes = if script.starts_with("base64:") {
                    use base64::{engine::general_purpose::STANDARD, Engine as _};
                    match STANDARD.decode(&script[7..]) {
                        Ok(decoded) => decoded,
                        Err(e) => {
                            log::warn!("Failed to decode base64 skel data for {}: {}", name, e);
                            script.as_bytes().to_vec()
                        }
                    }
                } else {
                    script.as_bytes().to_vec()
                };
                skels.push((base_name, *path_id, skel_bytes));
            }
        } else if name.ends_with(".atlas") {
            let base_name = name.trim_end_matches(".atlas").to_string();
            if let Some(script) = data.get("m_Script").and_then(|v| v.as_str()) {
                atlases.push((base_name, *path_id, script.to_string()));
            }
        }
    }

    // Match skels with atlases by name and create SpineAssets
    // Each skel-atlas pair with the same name creates one SpineAsset
    // Use CharacterAnimator path_id mappings for authoritative front/back determination
    let mut used_atlas_indices: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for (skel_name, skel_path_id, skel_data) in &skels {
        // Find matching atlas using path_id mapping (authoritative) or fall back to name matching
        let mut matched_atlas: Option<(usize, &String)> = None;

        // First, try to find atlas by path_id mapping from CharacterAnimator
        if let Some(expected_atlas_path_id) = skel_to_atlas.get(skel_path_id) {
            for (idx, (_atlas_name, atlas_path_id, atlas_content)) in atlases.iter().enumerate() {
                if atlas_path_id == expected_atlas_path_id && !used_atlas_indices.contains(&idx) {
                    log::debug!(
                        "Matched atlas for '{}' via path_id mapping: skel_path_id={} -> atlas_path_id={}",
                        skel_name, skel_path_id, atlas_path_id
                    );
                    matched_atlas = Some((idx, atlas_content));
                    break;
                }
            }
        }

        // Fall back to name-based matching if no path_id mapping found
        if matched_atlas.is_none() {
            for (idx, (atlas_name, _atlas_path_id, atlas_content)) in atlases.iter().enumerate() {
                if atlas_name == skel_name && !used_atlas_indices.contains(&idx) {
                    matched_atlas = Some((idx, atlas_content));
                    break;
                }
            }
        }

        // Determine asset type: prioritize CharacterAnimator path_id mapping over atlas heuristic
        let asset_type = if front_skel_path_ids.contains(skel_path_id) {
            log::debug!(
                "Skel '{}' (path_id={}) determined as FRONT via CharacterAnimator",
                skel_name,
                skel_path_id
            );
            SpineAssetType::BattleFront
        } else if back_skel_path_ids.contains(skel_path_id) {
            log::debug!(
                "Skel '{}' (path_id={}) determined as BACK via CharacterAnimator",
                skel_name,
                skel_path_id
            );
            SpineAssetType::BattleBack
        } else {
            // Fall back to atlas + skeleton content analysis
            let atlas_content = matched_atlas.map(|(_, c)| c.as_str());
            SpineAssetType::from_name_atlas_and_skel(skel_name, atlas_content, Some(skel_data))
        };

        if let Some((atlas_idx, atlas_content)) = matched_atlas {
            used_atlas_indices.insert(atlas_idx);

            spine_assets.push(SpineAsset {
                name: skel_name.clone(),
                asset_type,
                skel_data: Some(skel_data.clone()),
                atlas_data: Some(atlas_content.clone()),
                textures: Vec::new(),
                skel_path_id: Some(*skel_path_id),
            });
        } else {
            // Skel without atlas
            spine_assets.push(SpineAsset {
                name: skel_name.clone(),
                asset_type,
                skel_data: Some(skel_data.clone()),
                atlas_data: None,
                textures: Vec::new(),
                skel_path_id: Some(*skel_path_id),
            });
        }
    }

    // Also create assets for atlases without skels
    for (idx, (atlas_name, _atlas_path_id, atlas_content)) in atlases.iter().enumerate() {
        if !used_atlas_indices.contains(&idx) {
            let asset_type = SpineAssetType::from_name_and_atlas(atlas_name, Some(atlas_content));
            spine_assets.push(SpineAsset {
                name: atlas_name.clone(),
                asset_type,
                skel_data: None,
                atlas_data: Some(atlas_content.clone()),
                textures: Vec::new(),
                skel_path_id: None, // No skel for atlas-only assets
            });
        }
    }

    // Convert to HashMap for compatibility with rest of code, using type+name as key
    let mut spine_groups: std::collections::HashMap<String, SpineAsset> =
        std::collections::HashMap::new();
    for asset in spine_assets {
        let key = format!("{}:{}", asset.asset_type.directory_name(), asset.name);
        spine_groups.insert(key, asset);
    }

    // Save complete Spine assets
    for asset in spine_groups.values() {
        if asset.skel_data.is_none() && asset.atlas_data.is_none() {
            continue;
        }

        // Create directory structure: <type>/<asset_name>/
        // Each skin gets its own subdirectory within the type folder
        let type_dir = asset.asset_type.directory_name();
        let asset_dir = if type_dir.is_empty() {
            destdir.join(&asset.name)
        } else {
            destdir.join(type_dir).join(&asset.name)
        };

        // Validate textures if we have atlas data
        if let Some(ref atlas) = asset.atlas_data {
            let required_textures = parse_atlas_textures(atlas);
            let (found, missing) = validate_spine_textures(&required_textures, &texture_objects);

            if !missing.is_empty() {
                log::warn!(
                    "Spine asset '{}' missing textures: {:?}",
                    asset.name,
                    missing
                );
            }

            log::debug!(
                "Spine asset '{}': {}/{} textures available",
                asset.name,
                found.len(),
                required_textures.len()
            );
        }

        std::fs::create_dir_all(&asset_dir)?;

        if let Some(ref skel) = asset.skel_data {
            let path = asset_dir.join(format!("{}.skel", asset.name));
            std::fs::write(&path, skel)?;
            saved_count += 1;
        }

        if let Some(ref atlas) = asset.atlas_data {
            let path = asset_dir.join(format!("{}.atlas", asset.name));
            std::fs::write(&path, atlas)?;
            saved_count += 1;

            // Parse atlas size to find matching textures
            let atlas_size = parse_atlas_size(atlas);

            // Also save associated textures
            let required_textures = parse_atlas_textures(atlas);
            for tex_name in &required_textures {
                let base = tex_name.trim_end_matches(".png");

                // Get expected texture path_ids for this skeleton
                let expected_texture_path_ids: Option<&Vec<i64>> = asset
                    .skel_path_id
                    .and_then(|skel_id| skel_to_textures.get(&skel_id));

                // Helper closure to find best texture
                // force_exact: if true, always use exact size match (for RGB when alpha exists)
                let find_texture_obj = |name: &str,
                                        force_exact: bool|
                 -> Option<
                    unity_rs::files::object_reader::ObjectReader<()>,
                > {
                    if let Some(textures) = texture_objects_by_size.get(name) {
                        // Calculate pixel density (colored / total) to detect nearly-black textures
                        let pixel_density = |w: u32, h: u32, colored: usize| -> f64 {
                            let total = (w as usize) * (h as usize);
                            if total == 0 {
                                0.0
                            } else {
                                colored as f64 / total as f64
                            }
                        };

                        // A texture is "nearly black" if less than 15% of pixels are colored
                        const MIN_DENSITY: f64 = 0.15;

                        // FIRST PRIORITY: If we have expected texture path_ids from CharacterAnimator,
                        // use ONLY textures that match those path_ids
                        if let Some(expected_ids) = expected_texture_path_ids {
                            let path_id_matches: Vec<_> = textures
                                .iter()
                                .filter(|(_, _, _, path_id, _)| expected_ids.contains(path_id))
                                .collect();

                            if !path_id_matches.is_empty() {
                                // Pick the best matching texture (prefer exact atlas size, then most colored pixels)
                                if let Some((atlas_w, atlas_h)) = atlas_size {
                                    // Try exact size match first
                                    if let Some(exact) = path_id_matches
                                        .iter()
                                        .filter(|(w, h, _, _, _)| *w == atlas_w && *h == atlas_h)
                                        .max_by_key(|(_, _, colored, _, _)| *colored)
                                    {
                                        let (w, h, colored, path_id, obj) = *exact;
                                        log::debug!(
                                            "Found texture '{}' via path_id match {}: {}x{} with {} colored pixels",
                                            name, path_id, w, h, colored
                                        );
                                        return Some(obj.clone());
                                    }
                                }
                                // Fall back to best colored pixels among path_id matches
                                if let Some((w, h, colored, path_id, obj)) = path_id_matches
                                    .iter()
                                    .max_by_key(|(_, _, colored, _, _)| *colored)
                                {
                                    log::debug!(
                                        "Found texture '{}' via path_id match {}: {}x{} with {} colored pixels (no exact size)",
                                        name, path_id, w, h, colored
                                    );
                                    return Some((*obj).clone());
                                }
                            }
                        }

                        if let Some((atlas_w, atlas_h)) = atlas_size {
                            // First, try exact size match
                            let exact_matches: Vec<_> = textures
                                .iter()
                                .filter(|(w, h, _, _, _)| *w == atlas_w && *h == atlas_h)
                                .collect();

                            // If we have an exact match, check if we should use it
                            if let Some(best_exact) = exact_matches
                                .iter()
                                .max_by_key(|(_, _, colored, _, _)| *colored)
                            {
                                let (w, h, colored, _, obj) = *best_exact;
                                let density = pixel_density(*w, *h, *colored);

                                // Always use exact match if forced (RGB+alpha pair must match sizes)
                                // or if density is reasonable
                                if force_exact || density >= MIN_DENSITY {
                                    log::debug!(
                                            "Found exact size match for '{}': {}x{} with {} colored pixels ({:.1}% density){}",
                                            name, w, h, colored, density * 100.0,
                                            if force_exact { " [forced for alpha pairing]" } else { "" }
                                        );
                                    return Some(obj.clone());
                                }
                                log::debug!(
                                        "Exact match for '{}' is nearly black ({:.1}% density), looking for alternatives",
                                        name, density * 100.0
                                    );
                            }

                            // Try close matches (within 2x atlas size) that aren't nearly black
                            let close_matches: Vec<_> = textures
                                .iter()
                                .filter(|(w, h, colored, _, _)| {
                                    *w >= atlas_w
                                        && *h >= atlas_h
                                        && *w <= atlas_w * 2
                                        && *h <= atlas_h * 2
                                        && pixel_density(*w, *h, *colored) >= MIN_DENSITY
                                })
                                .collect();

                            // Prefer smallest close match with good density
                            if let Some(best_close) =
                                close_matches.iter().min_by_key(|(w, h, _, _, _)| (*w, *h))
                            {
                                let (w, h, colored, _, obj) = *best_close;
                                log::debug!(
                                        "Found close size match for '{}': {}x{} with {} colored pixels (atlas: {}x{})",
                                        name, w, h, colored, atlas_w, atlas_h
                                    );
                                return Some(obj.clone());
                            }

                            // If no good close match, look for any texture >= atlas size with good density
                            let larger_good: Vec<_> = textures
                                .iter()
                                .filter(|(w, h, colored, _, _)| {
                                    *w >= atlas_w
                                        && *h >= atlas_h
                                        && pixel_density(*w, *h, *colored) >= MIN_DENSITY
                                })
                                .collect();

                            // Prefer smallest one (less downscaling needed)
                            if let Some(best_larger) =
                                larger_good.iter().min_by_key(|(w, h, _, _, _)| (*w, *h))
                            {
                                let (w, h, colored, _, obj) = *best_larger;
                                log::debug!(
                                        "Found larger texture for '{}': {}x{} with {} colored pixels (atlas: {}x{})",
                                        name, w, h, colored, atlas_w, atlas_h
                                    );
                                return Some(obj.clone());
                            }

                            // Last resort: pick any texture with best density
                            let all_with_density: Vec<_> = textures
                                .iter()
                                .map(|(w, h, colored, _, obj)| {
                                    (w, h, colored, pixel_density(*w, *h, *colored), obj)
                                })
                                .collect();

                            if let Some((w, h, colored, density, obj)) =
                                all_with_density.iter().max_by(|a, b| {
                                    a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal)
                                })
                            {
                                log::debug!(
                                        "Using best density texture for '{}': {}x{} with {} colored pixels ({:.1}% density, atlas: {}x{})",
                                        name, w, h, colored, density * 100.0, atlas_w, atlas_h
                                    );
                                return Some((*obj).clone());
                            }
                        }
                        // No size info - pick texture with best density
                        if !textures.is_empty() {
                            let best = textures.iter().max_by(|a, b| {
                                let da = pixel_density(a.0, a.1, a.2);
                                let db = pixel_density(b.0, b.1, b.2);
                                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                            });
                            if let Some((_, _, _, _, obj)) = best {
                                return Some(obj.clone());
                            }
                        }
                    }
                    // Fall back to deduplicated map
                    texture_objects.get(name).cloned()
                };

                // Check if alpha texture exists (to determine if we need to force exact match for RGB)
                let alpha_name = format!("{}[alpha]", base);
                let has_alpha = texture_objects_by_size.contains_key(&alpha_name);

                // Try to get RGB texture matching atlas size
                // If alpha exists, force exact match so RGB and alpha sizes match
                let rgb_image = if let Some(mut tex_obj) = find_texture_obj(base, has_alpha) {
                    if let Ok(data) = tex_obj.read(false) {
                        if let Ok(mut texture) =
                            serde_json::from_value::<unity_rs::generated::Texture2D>(data.clone())
                        {
                            texture.object_reader = Some(Box::new(tex_obj.clone()));
                            texture_to_image(&texture).ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Try to get alpha texture matching atlas size
                let alpha_image = if let Some(mut alpha_obj) = find_texture_obj(&alpha_name, false)
                {
                    if let Ok(data) = alpha_obj.read(false) {
                        if let Ok(mut texture) =
                            serde_json::from_value::<unity_rs::generated::Texture2D>(data.clone())
                        {
                            texture.object_reader = Some(Box::new(alpha_obj.clone()));
                            texture_to_image(&texture).ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                // For BattleFront/BattleBack: combine RGB + alpha into a single texture
                // For Building: save RGB only (no alpha needed)
                // For other types: save RGB + alpha separately

                let tex_path = asset_dir.join(tex_name);

                // Helper to resize image to atlas size if needed
                let resize_to_atlas = |img: image::RgbaImage| -> image::RgbaImage {
                    if let Some((atlas_w, atlas_h)) = atlas_size {
                        if img.width() != atlas_w || img.height() != atlas_h {
                            log::debug!(
                                "Resizing texture from {}x{} to {}x{} to match atlas",
                                img.width(),
                                img.height(),
                                atlas_w,
                                atlas_h
                            );
                            image::imageops::resize(
                                &img,
                                atlas_w,
                                atlas_h,
                                image::imageops::FilterType::Lanczos3,
                            )
                        } else {
                            img
                        }
                    } else {
                        img
                    }
                };

                if asset.asset_type == SpineAssetType::BattleFront
                    || asset.asset_type == SpineAssetType::BattleBack
                {
                    // Combine RGB + alpha for battle sprites
                    if let (Some(ref rgb), Some(ref alpha)) = (&rgb_image, &alpha_image) {
                        let combined = combine_rgba_with_alpha(rgb, alpha);
                        let resized = resize_to_atlas(combined);
                        if resized.save(&tex_path).is_ok() {
                            saved_count += 1;
                            log::debug!("Saved combined spine texture: {:?}", tex_path);
                        }
                    } else if let Some(ref rgb) = rgb_image {
                        // No alpha available, save RGB as-is (resized if needed)
                        let resized = resize_to_atlas(rgb.clone());
                        if resized.save(&tex_path).is_ok() {
                            saved_count += 1;
                            log::debug!("Saved spine RGB texture (no alpha): {:?}", tex_path);
                        }
                    }
                } else {
                    // Building and other types: save RGB only (with resize if needed)
                    if let Some(ref rgb) = rgb_image {
                        let resized = resize_to_atlas(rgb.clone());
                        if resized.save(&tex_path).is_ok() {
                            saved_count += 1;
                            log::debug!("Saved spine RGB texture: {:?}", tex_path);
                        }
                    }
                }

                // Log warning if neither exists
                if rgb_image.is_none() && alpha_image.is_none() {
                    log::warn!("Spine texture '{}' not found", base);
                }
            }
        }

        log::debug!(
            "Saved Spine asset group: {} ({:?})",
            asset.name,
            asset.asset_type
        );
    }

    Ok(saved_count)
}

/// Collect spine texture info: name -> max atlas size
/// Returns map of texture names to their maximum atlas dimensions
/// Textures larger than this are full artwork and should NOT be excluded from root
fn collect_spine_texture_info(
    env_rc: &Rc<RefCell<Environment>>,
) -> std::collections::HashMap<String, (u32, u32)> {
    let env = env_rc.borrow();
    let mut spine_textures: std::collections::HashMap<String, (u32, u32)> =
        std::collections::HashMap::new();

    // Collect texture names and atlas sizes from atlas files
    for mut obj in env.objects() {
        if obj.obj_type == ClassIDType::TextAsset {
            if let Ok(data) = obj.read(false) {
                if let Some(name) = data.get("m_Name").and_then(|v| v.as_str()) {
                    if name.ends_with(".atlas") {
                        let base_name = name.trim_end_matches(".atlas");

                        // Get atlas content to extract size
                        if let Some(script) = data.get("m_Script").and_then(|v| v.as_str()) {
                            // Parse atlas size
                            if let Some(size) = parse_atlas_size(script) {
                                // Track base name with atlas size
                                let entry = spine_textures
                                    .entry(base_name.to_string())
                                    .or_insert((0, 0));
                                if size.0 > entry.0 {
                                    entry.0 = size.0;
                                }
                                if size.1 > entry.1 {
                                    entry.1 = size.1;
                                }

                                // Track alpha variant
                                let alpha_name = format!("{}[alpha]", base_name);
                                let alpha_entry =
                                    spine_textures.entry(alpha_name).or_insert((0, 0));
                                if size.0 > alpha_entry.0 {
                                    alpha_entry.0 = size.0;
                                }
                                if size.1 > alpha_entry.1 {
                                    alpha_entry.1 = size.1;
                                }

                                // Track building variant
                                if base_name.starts_with("build_") {
                                    let build_entry = spine_textures
                                        .entry(base_name.to_string())
                                        .or_insert((0, 0));
                                    if size.0 > build_entry.0 {
                                        build_entry.0 = size.0;
                                    }
                                    if size.1 > build_entry.1 {
                                        build_entry.1 = size.1;
                                    }
                                }
                            }
                        }
                    }

                    // Also track skel files for building variants
                    if name.ends_with(".skel") {
                        let base_name = name.trim_end_matches(".skel");
                        // Use a default max size for skel-only entries (will be overridden by atlas if exists)
                        spine_textures
                            .entry(base_name.to_string())
                            .or_insert((1024, 1024));
                        spine_textures
                            .entry(format!("{}[alpha]", base_name))
                            .or_insert((1024, 1024));
                        if !base_name.starts_with("build_") {
                            spine_textures
                                .entry(format!("build_{}", base_name))
                                .or_insert((2048, 2048));
                        }
                    }
                }
            }
        }
    }

    log::debug!(
        "Collected {} spine texture entries with sizes: {:?}",
        spine_textures.len(),
        spine_textures
    );
    spine_textures
}

fn is_spine_texture(
    name: &str,
    width: u32,
    height: u32,
    spine_info: &std::collections::HashMap<String, (u32, u32)>,
) -> bool {
    if let Some((max_w, max_h)) = spine_info.get(name) {
        // If texture is significantly larger than atlas size (> 1.5x in both dimensions),
        // it's artwork/portrait texture, NOT a spine spritesheet.
        // This is true even if the texture is power-of-2 square, because some portrait
        // textures happen to be 2048x2048 (the character art just fits that size).
        let is_larger = width > max_w + max_w / 2 && height > max_h + max_h / 2;

        if is_larger {
            log::debug!(
                "Texture '{}' ({}x{}) is larger than atlas ({}x{}), treating as artwork/portrait",
                name,
                width,
                height,
                max_w,
                max_h
            );
            return false; // It's artwork, not a spine texture
        }

        // Texture matches atlas size - it's a spine spritesheet texture
        true
    } else {
        false
    }
}

/// Extract a single AB file
fn ab_resolve(
    abfile: &Path,
    destdir: &Path,
    cmb_destdir: &Path,
    do_img: bool,
    do_txt: bool,
    do_aud: bool,
    do_spine: bool,
    merge_alpha: bool,
) -> Result<usize> {
    let mut env = Environment::new();

    // Load the file
    env.load_file(
        unity_rs::helpers::import_helper::FileSource::Path(abfile.to_string_lossy().to_string()),
        None,
        false,
    )
    .ok_or_else(|| anyhow::anyhow!("Failed to load AB file: {:?}", abfile))?;

    // Set environment references for resource resolution
    let env_rc = Rc::new(RefCell::new(env));
    Environment::set_environment_references(&env_rc)?;

    // Debug: log all object types in this bundle
    {
        let env = env_rc.borrow();

        // Debug: Check what files are loaded
        log::debug!("Environment has {} files", env.files.len());
        for (name, file_rc) in &env.files {
            let file_ref = file_rc.borrow();
            log::debug!("  File: {} -> {:?}", name, file_ref);
        }

        let mut type_counts: std::collections::HashMap<ClassIDType, usize> =
            std::collections::HashMap::new();
        for obj in env.objects() {
            *type_counts.entry(obj.obj_type).or_insert(0) += 1;
        }
        if !type_counts.is_empty() {
            log::debug!(
                "Objects in {:?}: {:?}",
                abfile.file_name().unwrap_or_default(),
                type_counts
            );
        } else {
            log::warn!(
                "No objects found in {:?}",
                abfile.file_name().unwrap_or_default()
            );
        }
    }

    let mut saved_count = 0;
    std::fs::create_dir_all(destdir)?;

    // If spine extraction is enabled, first collect spine texture info (name -> atlas size)
    let spine_texture_info: std::collections::HashMap<String, (u32, u32)> = if do_spine {
        collect_spine_texture_info(&env_rc)
    } else {
        std::collections::HashMap::new()
    };

    // Extract images (Texture2D and Sprite) with alpha/RGB combination
    if do_img {
        saved_count += extract_images_with_combination(
            &env_rc,
            destdir,
            cmb_destdir,
            merge_alpha,
            &spine_texture_info,
        )?;
        saved_count += extract_sprite_atlases(&env_rc, destdir)?;
        // Extract SpritePacker MonoBehaviours (used for items/icons, portraits, etc.)
        match extract_sprite_packer(&env_rc, destdir) {
            Ok(count) => saved_count += count,
            Err(e) => log::debug!("SpritePacker extraction: {}", e),
        }
    }

    // Extract text assets
    if do_txt {
        let env = env_rc.borrow();
        for mut obj in env.objects() {
            if obj.obj_type == ClassIDType::TextAsset {
                match obj.read(false) {
                    Ok(data) => {
                        log::debug!(
                            "TextAsset data keys: {:?}",
                            data.as_object().map(|o| o.keys().collect::<Vec<_>>())
                        );

                        match data.get("m_Name").and_then(|v| v.as_str()) {
                            Some(name) if name.is_empty() => {
                                log::debug!("TextAsset has empty name, skipping");
                                continue;
                            }
                            Some(name) => {
                                // Skip spine files if spine extraction is enabled
                                // They will be handled by extract_spine_assets with proper organization
                                if do_spine && (name.ends_with(".skel") || name.ends_with(".atlas"))
                                {
                                    log::debug!("Skipping spine TextAsset (will be handled by spine extraction): {}", name);
                                    continue;
                                }

                                log::debug!("Processing TextAsset: {}", name);

                                let output_path = destdir.join(name);
                                let mut saved = false;

                                // Try m_Script as string first
                                if let Some(script) = data.get("m_Script").and_then(|v| v.as_str())
                                {
                                    // Check if it's base64-encoded binary data
                                    if script.starts_with("base64:") {
                                        use base64::{
                                            engine::general_purpose::STANDARD, Engine as _,
                                        };
                                        if let Ok(decoded) = STANDARD.decode(&script[7..]) {
                                            if std::fs::write(&output_path, &decoded).is_ok() {
                                                saved_count += 1;
                                                saved = true;
                                                log::debug!(
                                                    "Saved text asset (binary): {:?}",
                                                    output_path
                                                );
                                            }
                                        }
                                    } else if std::fs::write(&output_path, script).is_ok() {
                                        saved_count += 1;
                                        saved = true;
                                        log::debug!("Saved text asset (string): {:?}", output_path);
                                    }
                                }

                                // Try m_Script as bytes (for binary data)
                                if !saved {
                                    if let Some(script_bytes) = data.get("m_Script").and_then(|v| {
                                        v.as_array().map(|arr| {
                                            arr.iter()
                                                .filter_map(|b| b.as_u64().map(|n| n as u8))
                                                .collect::<Vec<u8>>()
                                        })
                                    }) {
                                        if !script_bytes.is_empty()
                                            && std::fs::write(&output_path, &script_bytes).is_ok()
                                        {
                                            saved_count += 1;
                                            saved = true;
                                            log::debug!(
                                                "Saved text asset (bytes): {:?}",
                                                output_path
                                            );
                                        }
                                    }
                                }

                                // Try m_Bytes field (alternative binary storage)
                                if !saved {
                                    if let Some(bytes) = data.get("m_Bytes").and_then(|v| {
                                        v.as_array().map(|arr| {
                                            arr.iter()
                                                .filter_map(|b| b.as_u64().map(|n| n as u8))
                                                .collect::<Vec<u8>>()
                                        })
                                    }) {
                                        if !bytes.is_empty()
                                            && std::fs::write(&output_path, &bytes).is_ok()
                                        {
                                            saved_count += 1;
                                            saved = true;
                                            log::debug!(
                                                "Saved text asset (m_Bytes): {:?}",
                                                output_path
                                            );
                                        }
                                    }
                                }

                                if !saved {
                                    log::debug!("TextAsset '{}' has no content to save", name);
                                }
                            }
                            None => {
                                log::debug!("TextAsset has no m_Name or m_Name is not a string");
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to read TextAsset: {}", e);
                    }
                }
            }
        }
    }

    // Extract audio
    if do_aud {
        let env = env_rc.borrow();
        for mut obj in env.objects() {
            if obj.obj_type == ClassIDType::AudioClip {
                // Get container path for proper directory structure
                // This is critical for bundles like extra_custom_X.ab that contain
                // multiple audio clips with the same m_Name (e.g., CN_043 for different characters)
                let container_path = obj.container().ok().flatten();

                match obj.read(false) {
                    Ok(data) => {
                        let name = data
                            .get("m_Name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        match serde_json::from_value::<unity_rs::generated::AudioClip>(data.clone())
                        {
                            Ok(mut audio) => {
                                audio.object_reader = Some(Box::new(obj.clone()));

                                match unity_rs::export::audio_clip_converter::extract_audioclip_samples(
                                    &mut audio, true,
                                ) {
                                    Ok(samples) => {
                                        if samples.is_empty() {
                                            log::warn!("AudioClip '{}' produced no samples", name);
                                        }
                                        for (sample_name, wav_data) in samples {
                                            // Strip .wav if already present to avoid double extension
                                            let clean_name = sample_name.strip_suffix(".wav").unwrap_or(&sample_name);

                                            // Determine output path based on container path
                                            let output_path = if let Some(ref cpath) = container_path {
                                                // Use container path for proper directory structure
                                                // Container paths look like: "dyn/audio/sound_beta_2/voice_custom/char_xxx/cn_043"
                                                // We want to output to: upk/audio/sound_beta_2/voice_custom/char_xxx/CN_043.wav

                                                // Strip common prefixes from container path
                                                let clean_container = cpath
                                                    .strip_prefix("dyn/audio/")
                                                    .or_else(|| cpath.strip_prefix("dyn/"))
                                                    .or_else(|| cpath.strip_prefix("audio/"))
                                                    .unwrap_or(cpath);

                                                let container = std::path::Path::new(clean_container);

                                                // Find the 'upk' directory in destdir path to use as base
                                                // destdir can be: upk/<bundle_name> or upk/<src_folder>/<bundle_name>
                                                // We need to go back to the upk/ level for proper audio path structure
                                                let base_dir = {
                                                    let mut current = destdir;
                                                    let mut found_upk = None;

                                                    // Walk up the path looking for 'upk' directory
                                                    while let Some(parent) = current.parent() {
                                                        if current.file_name().map(|n| n == "upk").unwrap_or(false) {
                                                            found_upk = Some(current);
                                                            break;
                                                        }
                                                        current = parent;
                                                    }

                                                    // Use found upk dir, or fall back to parent of destdir
                                                    found_upk.unwrap_or_else(|| destdir.parent().unwrap_or(destdir))
                                                };

                                                // Get the parent directory from container path (e.g., sound_beta_2/voice_custom/char_xxx)
                                                // Then use the original sample name (with correct case) for the filename
                                                let parent_dir = container.parent().unwrap_or(std::path::Path::new(""));
                                                let audio_path = base_dir.join("audio").join(parent_dir).join(format!("{}.wav", clean_name));

                                                audio_path
                                            } else {
                                                // Fallback to simple name in destdir
                                                destdir.join(format!("{}.wav", clean_name))
                                            };

                                            // Create parent directories
                                            if let Some(parent) = output_path.parent() {
                                                let _ = std::fs::create_dir_all(parent);
                                            }

                                            if std::fs::write(&output_path, wav_data).is_ok() {
                                                saved_count += 1;
                                                log::debug!("Saved audio: {:?}", output_path);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Failed to extract samples from AudioClip '{}': {}", name, e);
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to deserialize AudioClip '{}': {}", name, e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to read AudioClip object: {}", e);
                    }
                }
            }
        }
    }

    // Extract Spine assets
    if do_spine {
        match extract_spine_assets(&env_rc, destdir) {
            Ok(count) => saved_count += count,
            Err(e) => log::warn!("Spine extraction error: {}", e),
        }
    }

    // CRITICAL: Clear environment references to break Rc cycles and free memory
    // Without this, Environment and all loaded files remain in memory indefinitely
    {
        let env = env_rc.borrow();
        for file_rc in env.files.values() {
            let file_ref = file_rc.borrow();
            match &*file_ref {
                unity_rs::files::bundle_file::FileType::SerializedFile(sf_rc) => {
                    if let Ok(mut sf) = sf_rc.try_borrow_mut() {
                        sf.environment = None;
                    }
                }
                unity_rs::files::bundle_file::FileType::BundleFile(bundle) => {
                    for inner_file_rc in bundle.files.values() {
                        if let unity_rs::files::bundle_file::FileType::SerializedFile(sf_rc) =
                            &*inner_file_rc.borrow()
                        {
                            if let Ok(mut sf) = sf_rc.try_borrow_mut() {
                                sf.environment = None;
                            }
                        }
                    }
                }
                unity_rs::files::bundle_file::FileType::WebFile(web) => {
                    for inner_file_rc in web.files.values() {
                        if let unity_rs::files::bundle_file::FileType::SerializedFile(sf_rc) =
                            &*inner_file_rc.borrow()
                        {
                            if let Ok(mut sf) = sf_rc.try_borrow_mut() {
                                sf.environment = None;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Aggressively clear Environment data structures to release memory immediately
    {
        let mut env = env_rc.borrow_mut();
        env.files.clear();
        env.cabs.clear();
        env.local_files.clear();
    }

    // Explicitly drop the Environment to ensure memory is released
    drop(env_rc);

    // Clear FMOD audio cache to prevent memory accumulation
    unity_rs::export::audio_clip_converter::fmod::clear_fmod_cache();

    Ok(saved_count)
}

/// Texture data with decoded image
struct TextureData {
    image: Option<RgbaImage>,
}

/// Convert a Texture2D to an RgbaImage
fn texture_to_image(texture: &unity_rs::generated::Texture2D) -> Result<image::RgbaImage> {
    let width = texture.m_Width.unwrap_or(0) as u32;
    let height = texture.m_Height.unwrap_or(0) as u32;
    let texture_format = TextureFormat::from(texture.m_TextureFormat.unwrap_or(0) as u32);

    // Get image data - either embedded or from external StreamData
    let image_data = if let Some(data) = texture.image_data.clone().filter(|d| !d.is_empty()) {
        // Embedded image data (non-empty)
        data
    } else if let Some(ref stream_data) = texture.m_StreamData {
        // External image data in .resS resource file
        let obj_reader = texture
            .object_reader
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No object_reader to fetch stream data"))?;

        let assets_file_weak = obj_reader
            .as_any()
            .downcast_ref::<unity_rs::files::ObjectReader<()>>()
            .and_then(|r| r.assets_file.as_ref())
            .ok_or_else(|| anyhow::anyhow!("Cannot access assets_file for stream data"))?;

        let assets_file_rc = assets_file_weak
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("SerializedFile was dropped"))?;

        let mut assets_file_mut = assets_file_rc.borrow_mut();

        let source_path = stream_data
            .path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("StreamData has no path"))?;

        unity_rs::helpers::resource_reader::get_resource_data(
            source_path,
            &mut assets_file_mut,
            stream_data.offset.unwrap_or(0) as usize,
            stream_data.size.unwrap_or(0) as usize,
        )
        .map_err(|e| anyhow::anyhow!("Failed to load stream data: {:?}", e))?
    } else {
        return Err(anyhow::anyhow!(
            "Texture has no image data (neither embedded nor streamed)"
        ));
    };

    let platform_blob = texture.m_PlatformBlob.clone();

    // Extract version and platform from SerializedFile
    let (version, platform) = if let Some(reader) = &texture.object_reader {
        if let Some(file) = reader.assets_file() {
            (file.version, file.target_platform)
        } else {
            ((0, 0, 0, 0), BuildTarget::UnknownPlatform)
        }
    } else {
        ((0, 0, 0, 0), BuildTarget::UnknownPlatform)
    };

    parse_image_data(
        image_data,
        width,
        height,
        texture_format,
        version,
        platform,
        platform_blob,
        true, // flip
    )
    .map_err(|e| anyhow::anyhow!("Failed to parse texture: {:?}", e))
}

/// Extract numbered sprites from SpriteAtlas objects
fn extract_sprite_atlases(env_rc: &Rc<RefCell<Environment>>, destdir: &Path) -> Result<usize> {
    use std::collections::HashMap;
    use unity_rs::classes::generated::SpriteAtlas;
    use unity_rs::export::sprite_helper;

    let env = env_rc.borrow();
    let mut saved_count = 0;

    // Find all SpriteAtlas objects
    for mut obj in env.objects() {
        if obj.obj_type == ClassIDType::SpriteAtlas {
            match obj.read(false) {
                Ok(data) => {
                    if let Ok(sprite_atlas) = serde_json::from_value::<SpriteAtlas>(data) {
                        // Get the atlas name
                        let atlas_name = sprite_atlas.m_Name.as_deref().unwrap_or("SpriteAtlas");

                        // Get the packed sprites
                        if let Some(packed_sprites) = &sprite_atlas.m_PackedSprites {
                            if packed_sprites.is_empty() {
                                continue;
                            }

                            log::debug!(
                                "Extracting {} sprites from atlas '{}'",
                                packed_sprites.len(),
                                atlas_name
                            );

                            // Get the assets file
                            let assets_file = if let Some(ref weak) = obj.assets_file {
                                match weak.upgrade() {
                                    Some(f) => f,
                                    None => continue,
                                }
                            } else {
                                continue;
                            };

                            // Extract each sprite with numbered suffix
                            let mut cache = HashMap::new();
                            for (index, sprite_ptr) in packed_sprites.iter().enumerate() {
                                // Skip null sprite pointers
                                if sprite_ptr.m_path_id == 0 {
                                    continue;
                                }

                                // Read the sprite
                                let sprite_result = {
                                    let assets_file_ref = assets_file.borrow();
                                    sprite_ptr.read(&assets_file_ref)
                                };

                                if let Ok(sprite) = sprite_result {
                                    // Get sprite image using sprite_helper
                                    match sprite_helper::get_image_from_sprite(
                                        &sprite,
                                        &assets_file,
                                        &mut cache,
                                    ) {
                                        Ok(sprite_img) => {
                                            // Save with numbered suffix: atlas_name$index.png
                                            let output_path = destdir
                                                .join(format!("{}${}.png", atlas_name, index));
                                            if sprite_img.save(&output_path).is_ok() {
                                                saved_count += 1;
                                                log::debug!(
                                                    "Saved sprite atlas image: {:?}",
                                                    output_path
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            log::debug!(
                                                "Failed to extract sprite {} from atlas '{}': {}",
                                                index,
                                                atlas_name,
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::debug!("Failed to read SpriteAtlas: {}", e);
                }
            }
        }
    }

    Ok(saved_count)
}

/// Extract images with automatic alpha/RGB combination
fn extract_images_with_combination(
    env_rc: &Rc<RefCell<Environment>>,
    destdir: &Path,
    cmb_destdir: &Path,
    merge_alpha: bool,
    spine_texture_info: &std::collections::HashMap<String, (u32, u32)>,
) -> Result<usize> {
    let env = env_rc.borrow();
    let mut saved_count = 0;

    // First pass: collect all textures and decode them
    let mut textures: HashMap<String, TextureData> = HashMap::new();
    let mut sprites: Vec<(
        String,
        serde_json::Value,
        unity_rs::files::object_reader::ObjectReader<()>,
    )> = Vec::new();

    // Debug: count textures found
    let mut texture_count = 0;
    let mut alpha_texture_count = 0;

    for mut obj in env.objects() {
        match obj.obj_type {
            ClassIDType::Texture2D => {
                if let Ok(data) = obj.read(false) {
                    if let Some(name) = data.get("m_Name").and_then(|v| v.as_str()) {
                        if name.is_empty() {
                            continue;
                        }

                        texture_count += 1;

                        // Track alpha textures
                        if name.contains("[alpha]") {
                            alpha_texture_count += 1;
                            log::info!("Found alpha texture: {}", name);
                        }

                        // Decode texture to image
                        let image = if let Ok(mut texture) =
                            serde_json::from_value::<unity_rs::generated::Texture2D>(data.clone())
                        {
                            texture.object_reader = Some(Box::new(obj.clone()));
                            match texture_to_image(&texture) {
                                Ok(img) => {
                                    log::debug!(
                                        "Successfully decoded texture: {} ({}x{})",
                                        name,
                                        img.width(),
                                        img.height()
                                    );
                                    Some(img)
                                }
                                Err(e) => {
                                    log::debug!("Failed to decode texture '{}': {}", name, e);
                                    None
                                }
                            }
                        } else {
                            log::debug!("Failed to deserialize Texture2D: {}", name);
                            None
                        };

                        // Handle duplicate textures - keep the one with more non-black pixels
                        if let Some(existing) = textures.get(name) {
                            if let (Some(ref new_img), Some(ref old_img)) =
                                (&image, &existing.image)
                            {
                                // Count non-black pixels to determine which has more color
                                let new_colored = new_img
                                    .pixels()
                                    .filter(|p| p[0] as u32 + p[1] as u32 + p[2] as u32 > 30)
                                    .count();
                                let old_colored = old_img
                                    .pixels()
                                    .filter(|p| p[0] as u32 + p[1] as u32 + p[2] as u32 > 30)
                                    .count();

                                if new_colored > old_colored {
                                    log::debug!("Replacing texture '{}' with version having more color ({} vs {} colored pixels)", name, new_colored, old_colored);
                                    textures.insert(name.to_string(), TextureData { image });
                                } else {
                                    log::debug!("Keeping existing texture '{}' (has more color: {} vs {} colored pixels)", name, old_colored, new_colored);
                                }
                            } else {
                                // One or both don't have image data - prefer the one with data
                                if image.is_some() && existing.image.is_none() {
                                    textures.insert(name.to_string(), TextureData { image });
                                }
                            }
                        } else {
                            textures.insert(name.to_string(), TextureData { image });
                        }
                        log::debug!("Collected texture: {}", name);
                    }
                }
            }
            ClassIDType::Sprite => {
                if let Ok(data) = obj.read(false) {
                    if let Some(name) = data.get("m_Name").and_then(|v| v.as_str()) {
                        if !name.is_empty() {
                            sprites.push((name.to_string(), data, obj.clone()));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Log summary of textures found
    if texture_count > 0 {
        log::info!(
            "Texture summary: {} total, {} alpha, {} sprites",
            texture_count,
            alpha_texture_count,
            sprites.len()
        );

        // List all texture names for debugging
        let texture_names: Vec<&String> = textures.keys().collect();
        log::debug!("All texture names: {:?}", texture_names);
    }

    // Track which textures were actually saved (to avoid sprite extraction overwriting them)
    let mut saved_texture_names: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    // Track which textures were merged with alpha (only these should skip sprite extraction)
    let mut merged_texture_names: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    // Handle alpha textures based on merge_alpha setting
    // Skip textures that belong to spine assets (they will be handled by spine extraction)
    if merge_alpha {
        // Merge alpha into RGBA images
        // Find RGB+Alpha pairs first
        let mut processed_names: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for name in textures.keys() {
            // Skip spine textures (based on size comparison)
            if let Some(tex_data) = textures.get(name) {
                if let Some(ref img) = tex_data.image {
                    if is_spine_texture(name, img.width(), img.height(), spine_texture_info) {
                        continue;
                    }
                }
            }
            if name.contains("[alpha]") {
                let rgb_name = name.replace("[alpha]", "");
                // Skip if RGB is a spine texture
                if let Some(tex_data) = textures.get(&rgb_name) {
                    if let Some(ref img) = tex_data.image {
                        if is_spine_texture(
                            &rgb_name,
                            img.width(),
                            img.height(),
                            spine_texture_info,
                        ) {
                            continue;
                        }
                    }
                }
                if textures.contains_key(&rgb_name) {
                    processed_names.insert(rgb_name.clone());
                    processed_names.insert(name.clone());

                    let rgb_data = &textures[&rgb_name];
                    let alpha_data = &textures[name];

                    if let (Some(rgb_img), Some(alpha_img)) = (&rgb_data.image, &alpha_data.image) {
                        // Combine RGB with alpha channel
                        let combined = combine_rgba_with_alpha(rgb_img, alpha_img);

                        // Save combined image with RGB name
                        let output_path = destdir.join(format!("{}.png", rgb_name));
                        if combined.save(&output_path).is_ok() {
                            saved_count += 1;
                            saved_texture_names.insert(rgb_name.clone());
                            merged_texture_names.insert(rgb_name.clone());
                            log::debug!("Saved merged texture: {:?}", output_path);
                        }
                    }
                }
            }
        }

        // Save remaining textures that weren't part of pairs
        for (name, tex_data) in &textures {
            // Skip spine textures (based on size comparison)
            if let Some(ref img) = tex_data.image {
                if is_spine_texture(name, img.width(), img.height(), spine_texture_info) {
                    log::debug!("Skipping spine texture from root: {}", name);
                    continue;
                }
            }
            if !processed_names.contains(name) {
                if let Some(ref img) = tex_data.image {
                    let output_path = destdir.join(format!("{}.png", name));
                    if img.save(&output_path).is_ok() {
                        saved_count += 1;
                        saved_texture_names.insert(name.clone());
                        log::debug!("Saved texture: {:?}", output_path);
                    }
                }
            }
        }
    } else {
        // Default: Always merge alpha into RGB textures for usable output
        // First, find all RGB+alpha pairs
        let mut alpha_pairs: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for name in textures.keys() {
            if name.contains("[alpha]") {
                let rgb_name = name.replace("[alpha]", "");
                if textures.contains_key(&rgb_name) {
                    alpha_pairs.insert(rgb_name.clone(), name.clone());
                    log::debug!("Found alpha pair: {} -> {}", rgb_name, name);
                }
            }
        }
        log::info!(
            "Alpha merge: {} textures have separate alpha channels that will be merged",
            alpha_pairs.len()
        );

        // Process textures - merge alpha pairs, save others as-is
        for (name, tex_data) in &textures {
            // Skip [alpha] textures - they'll be merged into their RGB counterpart
            if name.contains("[alpha]") {
                log::debug!("Skipping alpha texture (will be merged): {}", name);
                continue;
            }

            // Check if this RGB texture has a matching alpha texture
            let has_alpha_pair = alpha_pairs.contains_key(name);

            // Skip spine textures ONLY if they don't have an alpha pair
            // Textures with alpha pairs should always be merged and saved for portrait use
            if !has_alpha_pair {
                if let Some(ref img) = tex_data.image {
                    if is_spine_texture(name, img.width(), img.height(), spine_texture_info) {
                        log::debug!("Skipping spine texture from root: {}", name);
                        continue;
                    }
                }
            }

            if let Some(alpha_name) = alpha_pairs.get(name) {
                log::debug!("Merging texture {} with alpha {}", name, alpha_name);
                // Merge RGB + alpha and save
                if let (Some(rgb_img), Some(alpha_img)) = (
                    &tex_data.image,
                    textures.get(alpha_name).and_then(|t| t.image.as_ref()),
                ) {
                    let combined = combine_rgba_with_alpha(rgb_img, alpha_img);
                    let output_path = destdir.join(format!("{}.png", name));
                    if combined.save(&output_path).is_ok() {
                        saved_count += 1;
                        saved_texture_names.insert(name.clone());
                        merged_texture_names.insert(name.clone());
                        log::debug!("Saved merged texture (RGB+alpha): {:?}", output_path);
                    }
                } else {
                    log::warn!("Failed to get images for merge: {} + {}", name, alpha_name);
                }
            } else {
                // No alpha pair - save texture as-is (it already has embedded alpha or doesn't need it)
                log::debug!("Saving texture as-is (no alpha pair): {}", name);
                if let Some(ref img) = tex_data.image {
                    let output_path = destdir.join(format!("{}.png", name));
                    if img.save(&output_path).is_ok() {
                        saved_count += 1;
                        saved_texture_names.insert(name.clone());
                        log::debug!("Saved texture: {:?}", output_path);
                    }
                }
            }
        }
    }

    // Log which textures were actually saved (for debugging)
    log::debug!(
        "Actually saved {} textures to root: {:?}",
        saved_texture_names.len(),
        saved_texture_names
    );

    // Also create combined images in cmb directory for backwards compatibility
    // This creates $0.png files that have RGB+Alpha merged
    let mut combined_pairs: Vec<(String, String)> = Vec::new(); // (rgb_name, alpha_name)

    for name in textures.keys() {
        if name.contains("[alpha]") {
            // Find matching RGB texture
            let rgb_name = name.replace("[alpha]", "");
            if textures.contains_key(&rgb_name) {
                combined_pairs.push((rgb_name.clone(), name.clone()));
            }
        }
    }

    for (rgb_name, alpha_name) in combined_pairs {
        let rgb_data = &textures[&rgb_name];
        let alpha_data = &textures[&alpha_name];

        if let (Some(rgb_img), Some(alpha_img)) = (&rgb_data.image, &alpha_data.image) {
            // Combine RGB with alpha channel
            let combined = combine_rgba_with_alpha(rgb_img, alpha_img);

            // Save as $0.png to cmb directory (matches Python output structure)
            std::fs::create_dir_all(cmb_destdir).ok();
            let output_path = cmb_destdir.join(format!("{}$0.png", rgb_name));
            if combined.save(&output_path).is_ok() {
                saved_count += 1;
                log::debug!("Saved combined texture: {:?}", output_path);
            }
        }
    }

    // Group sprites by texture name for numbered extraction
    let mut sprite_groups: std::collections::HashMap<
        String,
        Vec<(
            String,
            unity_rs::generated::Sprite,
            Rc<RefCell<unity_rs::files::serialized_file::SerializedFile>>,
        )>,
    > = std::collections::HashMap::new();

    // Process sprites - save both named and group for numbered extraction
    for (name, data, obj) in sprites {
        if let Some(assets_file) = obj.assets_file.as_ref().and_then(|weak| weak.upgrade()) {
            if let Ok(sprite) = serde_json::from_value::<unity_rs::generated::Sprite>(data.clone())
            {
                // Check if sprite has alphaTexture reference
                let has_alpha_texture = sprite
                    .m_RD
                    .as_ref()
                    .and_then(|rd| rd.alphaTexture.as_ref())
                    .map(|at| at.m_path_id != 0)
                    .unwrap_or(false);

                // Check if sprite provides cropping (textureRect dimensions differ from texture)
                let provides_cropping = if let Some(ref rd) = sprite.m_RD {
                    if let (Some(ref tr), Some(ref tex)) = (&rd.textureRect, &rd.texture) {
                        // Get texture dimensions
                        let tex_dims = {
                            let assets_file_ref = assets_file.borrow();
                            assets_file_ref
                                .objects
                                .get(&tex.m_path_id)
                                .and_then(|obj| {
                                    let mut obj = obj.clone();
                                    obj.read(false).ok()
                                })
                                .and_then(|data| {
                                    let w = data.get("m_Width").and_then(|v| v.as_i64());
                                    let h = data.get("m_Height").and_then(|v| v.as_i64());
                                    match (w, h) {
                                        (Some(w), Some(h)) => Some((w as f32, h as f32)),
                                        _ => None,
                                    }
                                })
                        };

                        if let Some((tex_w, tex_h)) = tex_dims {
                            let rect_w = tr.width.unwrap_or(0.0);
                            let rect_h = tr.height.unwrap_or(0.0);
                            // Sprite provides cropping if textureRect is smaller than texture
                            (rect_w - tex_w).abs() > 1.0 || (rect_h - tex_h).abs() > 1.0
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };

                // If sprite doesn't have alphaTexture, doesn't provide cropping, and texture was
                // already merged with alpha, skip sprite extraction to preserve the merged version
                if !has_alpha_texture && !provides_cropping && merged_texture_names.contains(&name)
                {
                    log::debug!(
                        "Skipping sprite {} (no alphaTexture, no cropping, merged texture already saved)",
                        name
                    );
                    continue;
                }

                // Extract sprite - it will properly:
                // - Crop according to textureRect
                // - Merge alpha from sprite's alphaTexture reference (if present)
                // - Handle tight packing, rotation, etc.
                let output_path = destdir.join(format!("{}.png", name));

                if unity_rs::export::sprite_helper::save_sprite(
                    &sprite,
                    &assets_file,
                    output_path.to_str().unwrap_or(""),
                )
                .is_ok()
                {
                    saved_count += 1;
                    log::debug!("Saved sprite: {:?}", output_path);
                }

                // Group by texture name for numbered extraction
                if let Some(rd) = &sprite.m_RD {
                    if let Some(texture_ptr) = &rd.texture {
                        if texture_ptr.m_path_id != 0 {
                            // Get texture name
                            let texture_name_opt = {
                                let assets_file_ref = assets_file.borrow();
                                assets_file_ref
                                    .objects
                                    .get(&texture_ptr.m_path_id)
                                    .and_then(|obj_reader| {
                                        let mut obj = obj_reader.clone();
                                        obj.read(false).ok()
                                    })
                                    .and_then(|texture_data| {
                                        texture_data
                                            .get("m_Name")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string())
                                    })
                            };

                            if let Some(texture_name) = texture_name_opt {
                                sprite_groups.entry(texture_name).or_default().push((
                                    name.clone(),
                                    sprite.clone(),
                                    Rc::clone(&assets_file),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // Save numbered sprites grouped by texture
    for (texture_name, mut sprite_list) in sprite_groups {
        // Only create numbered sprites if multiple sprites share the same texture
        if sprite_list.len() > 1 {
            // Sort by sprite name for consistent ordering
            sprite_list.sort_by(|a, b| a.0.cmp(&b.0));

            let mut cache = std::collections::HashMap::new();
            for (index, (_name, sprite, assets_file)) in sprite_list.iter().enumerate() {
                match unity_rs::export::sprite_helper::get_image_from_sprite(
                    sprite,
                    assets_file,
                    &mut cache,
                ) {
                    Ok(sprite_img) => {
                        let output_path = destdir.join(format!("{}${}.png", texture_name, index));
                        if sprite_img.save(&output_path).is_ok() {
                            saved_count += 1;
                            log::debug!("Saved numbered sprite: {:?}", output_path);
                        }
                    }
                    Err(e) => {
                        log::debug!("Failed to extract numbered sprite {}: {}", index, e);
                    }
                }
            }
        }
    }

    // CRITICAL: Explicitly drop large image data structures to free memory immediately
    // Without this, GBs of decoded texture data remain in memory
    // Note: sprite_groups and sprites are already moved/consumed by the for loops above
    drop(textures);

    Ok(saved_count)
}

/// Combine RGB image with alpha channel
fn combine_rgba_with_alpha(rgb: &RgbaImage, alpha: &RgbaImage) -> RgbaImage {
    let (width, height) = rgb.dimensions();

    // Convert alpha to grayscale for alpha channel
    let alpha_gray = image::DynamicImage::ImageRgba8(alpha.clone()).to_luma8();

    // Resize alpha if dimensions don't match
    let alpha_resized = if alpha_gray.dimensions() != (width, height) {
        image::imageops::resize(
            &alpha_gray,
            width,
            height,
            image::imageops::FilterType::Triangle,
        )
    } else {
        alpha_gray
    };

    // Combine
    let mut result = ImageBuffer::new(width, height);
    for (x, y, pixel) in result.enumerate_pixels_mut() {
        let rgb_pixel = rgb.get_pixel(x, y);
        let alpha_value = alpha_resized.get_pixel(x, y).0[0];

        if alpha_value == 0 {
            *pixel = Rgba([0, 0, 0, 0]);
        } else {
            *pixel = Rgba([rgb_pixel[0], rgb_pixel[1], rgb_pixel[2], alpha_value]);
        }
    }

    result
}

/// Size threshold for "large" files that need special handling (50 MB)
const LARGE_FILE_THRESHOLD: u64 = 50 * 1024 * 1024;

/// Maximum file size to process by default (500 MB) - larger files are deferred
const MAX_FILE_SIZE_DEFAULT: u64 = 500 * 1024 * 1024;

/// File with its size for sorting
struct SizedFile {
    path: PathBuf,
    size: u64,
}

/// Main entry point for asset extraction
pub fn main(
    src: &Path,
    destdir: &Path,
    do_del: bool,
    do_img: bool,
    do_txt: bool,
    do_aud: bool,
    do_spine: bool,
    merge_alpha: bool,
    separate: bool,
    force: bool,
    threads: usize,
    skip_large_mb: u64,
) -> Result<()> {
    // Configure thread pool based on user preference
    // threads=1: Sequential processing (lowest memory, slowest)
    // threads=2-4: Moderate parallelism (balanced)
    // threads=8+: High parallelism (fastest, highest memory)
    if threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .ok(); // Ignore error if already initialized

        log::info!("Using {} thread(s) for extraction", threads);
    }

    log::info!("Retrieving file paths...");

    // Collect all AB files with their sizes
    let mut sized_files: Vec<SizedFile> = if src.is_file() {
        let size = std::fs::metadata(src).map(|m| m.len()).unwrap_or(0);
        vec![SizedFile {
            path: src.to_path_buf(),
            size,
        }]
    } else {
        WalkDir::new(src)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
            .filter(|p| is_ab_file(p))
            .map(|p| {
                let size = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                SizedFile { path: p, size }
            })
            .collect()
    };

    if sized_files.is_empty() {
        println!("No AB files found");
        return Ok(());
    }

    // Sort files by size (smallest first) for better progress visibility
    sized_files.sort_by_key(|f| f.size);

    // Separate normal and large files
    let (normal_files, large_files): (Vec<_>, Vec<_>) = sized_files
        .into_iter()
        .partition(|f| f.size < MAX_FILE_SIZE_DEFAULT);

    let _total_normal = normal_files.len();
    let total_large = large_files.len();

    if total_large > 0 {
        log::warn!(
            "Found {} files larger than {} MB that will be processed last:",
            total_large,
            MAX_FILE_SIZE_DEFAULT / 1024 / 1024
        );
        for f in &large_files {
            log::warn!(
                "  {} ({} MB)",
                f.path.file_name().unwrap_or_default().to_string_lossy(),
                f.size / 1024 / 1024
            );
        }
    }

    // Apply skip_large_mb filter if specified
    let skip_threshold = if skip_large_mb > 0 {
        skip_large_mb * 1024 * 1024
    } else {
        u64::MAX // No skip
    };

    let (normal_files, skipped_files): (Vec<_>, Vec<_>) = if skip_large_mb > 0 {
        let (keep, skip): (Vec<_>, Vec<_>) = normal_files
            .into_iter()
            .chain(large_files)
            .partition(|f| f.size < skip_threshold);

        if !skip.is_empty() {
            log::warn!(
                "Skipping {} files larger than {} MB (use --skip-large-mb=0 to include):",
                skip.len(),
                skip_large_mb
            );
            for f in &skip {
                log::warn!(
                    "  {} ({} MB)",
                    f.path.file_name().unwrap_or_default().to_string_lossy(),
                    f.size / 1024 / 1024
                );
            }
        }
        (keep, skip)
    } else {
        // No skip, combine normal and large files
        let all: Vec<_> = normal_files.into_iter().chain(large_files).collect();
        (all, Vec::new())
    };

    // Combine: process normal files first, then large files
    let files: Vec<PathBuf> = normal_files.into_iter().map(|f| f.path).collect();
    let skipped_count = skipped_files.len();

    if do_del && destdir.exists() {
        log::info!("Cleaning destination directory...");
        std::fs::remove_dir_all(destdir)?;
    }

    std::fs::create_dir_all(destdir)?;

    // Create upk and cmb subdirectories to match Python structure
    let upk_dir = destdir.join("upk");
    let cmb_dir = destdir.join("cmb");
    std::fs::create_dir_all(&upk_dir)?;
    std::fs::create_dir_all(&cmb_dir)?;

    // Load manifest for incremental extraction
    let manifest_path = destdir.join(".extraction_manifest.json");
    let manifest = if force {
        ExtractionManifest::default()
    } else {
        ExtractionManifest::load(&manifest_path)
    };
    let manifest = Mutex::new(manifest);

    // Filter files that need processing
    let files_to_process: Vec<_> = if force {
        files.clone()
    } else {
        files
            .iter()
            .filter(|f| manifest.lock().unwrap().should_process(f))
            .cloned()
            .collect()
    };

    let skipped = files.len() - files_to_process.len();
    if skipped > 0 {
        log::info!("Skipping {} unchanged files", skipped);
    }

    if files_to_process.is_empty() {
        println!("All files are up to date (use --force to re-extract)");
        return Ok(());
    }

    // Progress bar
    let pb = ProgressBar::new(files_to_process.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    let total_saved = AtomicUsize::new(0);
    let src_path = src.to_path_buf();
    // Get the input directory name to include in output path (e.g., "chararts")
    // When input is a single file, use parent directory name
    let src_dir_name = if src_path.is_dir() {
        src_path.file_name().unwrap_or_default().to_os_string()
    } else {
        // For single file input, use parent directory name (e.g., "skinpack" from ".../skinpack/char_xxx.ab")
        src_path
            .parent()
            .and_then(|p| p.file_name())
            .unwrap_or_default()
            .to_os_string()
    };
    let save_counter = AtomicUsize::new(0);

    // Initialize memory tracking
    let mut sys = System::new_all();
    let pid = Pid::from_u32(std::process::id());
    sys.refresh_all();

    let start_memory = if let Some(process) = sys.process(pid) {
        process.memory()
    } else {
        0
    };
    log::info!(
        "Starting extraction with {} MB memory",
        start_memory / 1024 / 1024
    );

    if threads == 1 {
        // Sequential processing for minimal memory usage
        for file in &files_to_process {
            let (upk_subdestdir, cmb_subdestdir) = if separate {
                let rel_path = file.strip_prefix(&src_path).unwrap_or(file);
                let parent = rel_path.parent().unwrap_or(Path::new(""));
                let stem = file.file_stem().unwrap_or_default();
                // Include input directory name in output path (e.g., chararts/char_xxx)
                (
                    upk_dir.join(&src_dir_name).join(parent).join(stem),
                    cmb_dir.join(&src_dir_name).join(parent).join(stem),
                )
            } else {
                (upk_dir.clone(), cmb_dir.clone())
            };

            // Track memory before processing this file
            let mem_before = {
                sys.refresh_all();
                sys.process(pid).map(|p| p.memory()).unwrap_or(0)
            };

            match ab_resolve(
                file,
                &upk_subdestdir,
                &cmb_subdestdir,
                do_img,
                do_txt,
                do_aud,
                do_spine,
                merge_alpha,
            ) {
                Ok(count) => {
                    total_saved.fetch_add(count, Ordering::Relaxed);
                    manifest.lock().unwrap().update(file, count);

                    let file_count = save_counter.fetch_add(1, Ordering::Relaxed) + 1;

                    // Track memory after processing this file
                    sys.refresh_all();
                    if let Some(process) = sys.process(pid) {
                        let mem_after = process.memory();
                        let file_growth_mb = (mem_after as i64 - mem_before as i64) / 1024 / 1024;
                        if file_growth_mb.abs() > 50 {
                            log::error!(
                                "Large memory change for file #{} '{}': {} MB",
                                file_count,
                                file.file_name().unwrap_or_default().to_string_lossy(),
                                file_growth_mb
                            );
                        }
                    }

                    // Save manifest every 50 files (more frequent for safety)
                    if file_count.is_multiple_of(50) {
                        if let Err(e) = manifest.lock().unwrap().save(&manifest_path) {
                            log::warn!("Failed to save incremental manifest: {}", e);
                        }
                    }

                    // Log memory usage every 10 files to track leaks
                    if file_count.is_multiple_of(10) {
                        sys.refresh_all();
                        if let Some(process) = sys.process(pid) {
                            let current_memory = process.memory();
                            let memory_mb = current_memory / 1024 / 1024;
                            let growth_mb =
                                (current_memory as i64 - start_memory as i64) / 1024 / 1024;
                            log::warn!(
                                "Memory after {} files: {} MB (growth: {} MB)",
                                file_count,
                                memory_mb,
                                growth_mb
                            );
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to process {:?}: {}", file, e);
                }
            }

            pb.inc(1);
        }
    } else {
        // Hybrid processing: parallel for small files, sequential for large files
        // This prevents memory explosions from processing multiple large files simultaneously

        // Separate files into small and large based on file size
        let (small_files, large_files): (Vec<_>, Vec<_>) = files_to_process.iter().partition(|f| {
            std::fs::metadata(f)
                .map(|m| m.len() < LARGE_FILE_THRESHOLD)
                .unwrap_or(true)
        });

        let small_count = small_files.len();
        let large_count = large_files.len();

        if large_count > 0 {
            log::info!(
                "Processing {} small files in parallel, {} large files sequentially",
                small_count,
                large_count
            );
        }

        // Process small files in parallel
        small_files.par_iter().for_each(|file| {
            let (upk_subdestdir, cmb_subdestdir) = if separate {
                let rel_path = file.strip_prefix(&src_path).unwrap_or(file);
                let parent = rel_path.parent().unwrap_or(Path::new(""));
                let stem = file.file_stem().unwrap_or_default();
                // Include input directory name in output path (e.g., chararts/char_xxx)
                (
                    upk_dir.join(&src_dir_name).join(parent).join(stem),
                    cmb_dir.join(&src_dir_name).join(parent).join(stem),
                )
            } else {
                (upk_dir.clone(), cmb_dir.clone())
            };

            match ab_resolve(
                file,
                &upk_subdestdir,
                &cmb_subdestdir,
                do_img,
                do_txt,
                do_aud,
                do_spine,
                merge_alpha,
            ) {
                Ok(count) => {
                    total_saved.fetch_add(count, Ordering::Relaxed);
                    manifest.lock().unwrap().update(file, count);

                    // Save manifest every 50 files (more frequent for safety)
                    if save_counter
                        .fetch_add(1, Ordering::Relaxed)
                        .is_multiple_of(50)
                    {
                        if let Err(e) = manifest.lock().unwrap().save(&manifest_path) {
                            log::warn!("Failed to save incremental manifest: {}", e);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to process {:?}: {}", file, e);
                }
            }

            pb.inc(1);
        });

        // Process large files sequentially with explicit memory management
        for file in large_files {
            let file_size = std::fs::metadata(file).map(|m| m.len()).unwrap_or(0);
            let file_name = file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            log::warn!(
                "Processing large file '{}' ({} MB) - this may take a while...",
                file_name,
                file_size / 1024 / 1024
            );

            // Check current memory before processing large file
            sys.refresh_all();
            let mem_before = sys.process(pid).map(|p| p.memory()).unwrap_or(0);

            let (upk_subdestdir, cmb_subdestdir) = if separate {
                let rel_path = file.strip_prefix(&src_path).unwrap_or(file);
                let parent = rel_path.parent().unwrap_or(Path::new(""));
                let stem = file.file_stem().unwrap_or_default();
                // Include input directory name in output path (e.g., chararts/char_xxx)
                (
                    upk_dir.join(&src_dir_name).join(parent).join(stem),
                    cmb_dir.join(&src_dir_name).join(parent).join(stem),
                )
            } else {
                (upk_dir.clone(), cmb_dir.clone())
            };

            match ab_resolve(
                file,
                &upk_subdestdir,
                &cmb_subdestdir,
                do_img,
                do_txt,
                do_aud,
                do_spine,
                merge_alpha,
            ) {
                Ok(count) => {
                    total_saved.fetch_add(count, Ordering::Relaxed);
                    manifest.lock().unwrap().update(file, count);
                    save_counter.fetch_add(1, Ordering::Relaxed);

                    // Report memory usage after large file
                    sys.refresh_all();
                    if let Some(process) = sys.process(pid) {
                        let mem_after = process.memory();
                        let file_growth_mb = (mem_after as i64 - mem_before as i64) / 1024 / 1024;
                        log::info!(
                            "Completed '{}': extracted {} assets, memory delta: {} MB",
                            file_name,
                            count,
                            file_growth_mb
                        );
                    }
                }
                Err(e) => {
                    log::error!("Failed to process large file '{}': {}", file_name, e);
                }
            }

            pb.inc(1);

            // Save manifest after each large file
            if let Err(e) = manifest.lock().unwrap().save(&manifest_path) {
                log::warn!("Failed to save manifest: {}", e);
            }
        }
    }

    pb.finish_with_message("Done!");

    // Save manifest
    if let Err(e) = manifest.lock().unwrap().save(&manifest_path) {
        log::warn!("Failed to save manifest: {}", e);
    }

    println!("\nExtraction complete!");
    println!("  Total files found: {}", files.len() + skipped_count);
    println!("  Processed: {}", files_to_process.len());
    println!("  Skipped (unchanged): {}", skipped);
    if skipped_count > 0 {
        println!("  Skipped (too large): {}", skipped_count);
    }
    println!("  Exported {} assets", total_saved.load(Ordering::Relaxed));

    Ok(())
}

/// Extract individual sprites from SpritePacker MonoBehaviours (used for charportraits)
///
/// This handles Arknights' custom SpritePacker format where portraits are packed
/// into texture atlases with a MonoBehaviour describing the sprite positions.
pub fn extract_sprite_packer(env_rc: &Rc<RefCell<Environment>>, destdir: &Path) -> Result<usize> {
    use crate::sprite_packer::{is_sprite_packer, SpriteExtractor, SpritePackerData};
    use std::collections::HashMap;

    let mut saved_count = 0;
    let env = env_rc.borrow();

    // First, collect all textures by PathID
    let mut textures: HashMap<i64, (String, RgbaImage)> = HashMap::new();

    for mut obj in env.objects() {
        if obj.obj_type == ClassIDType::Texture2D {
            if let Ok(data) = obj.read(false) {
                let name = data
                    .get("m_Name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Parse texture using the proper Texture2D struct with stream data support
                if let Ok(mut texture) =
                    serde_json::from_value::<unity_rs::generated::Texture2D>(data.clone())
                {
                    texture.object_reader = Some(Box::new(obj.clone()));
                    if let Ok(img) = texture_to_image(&texture) {
                        textures.insert(obj.path_id, (name, img));
                    }
                }
            }
        }
    }

    // Now find and process SpritePacker MonoBehaviours
    for mut obj in env.objects() {
        if obj.obj_type != ClassIDType::MonoBehaviour {
            continue;
        }

        // Get the TypeTree node
        let node_clone = obj.serialized_type.as_ref().and_then(|st| st.node.clone());
        if node_clone.is_none() {
            continue;
        }

        // Try to read with full TypeTree
        obj.reset();
        let json = match obj.read_typetree(node_clone, false, false) {
            Ok(j) => j,
            Err(_) => continue,
        };

        // Check if this is a SpritePacker
        if !is_sprite_packer(&json) {
            continue;
        }

        // Parse SpritePacker data
        let packer = match SpritePackerData::from_json(&json) {
            Ok(p) => p,
            Err(e) => {
                log::warn!("Failed to parse SpritePacker: {}", e);
                continue;
            }
        };

        log::debug!(
            "Found SpritePacker '{}' with {} sprites",
            packer.name,
            packer.sprites.len()
        );

        // Get the texture and alpha textures
        let texture_path_id = packer.atlas.texture.path_id;
        let alpha_path_id = packer.atlas.alpha.path_id;

        let texture = textures.get(&texture_path_id);
        let alpha = textures.get(&alpha_path_id);

        if texture.is_none() {
            log::warn!(
                "SpritePacker '{}': texture {} not found",
                packer.name,
                texture_path_id
            );
            continue;
        }

        let (_, tex_img) = texture.unwrap();
        let alpha_img = alpha.map(|(_, img)| img.clone());

        // Create extractor
        let extractor = SpriteExtractor::new(tex_img.clone(), alpha_img);

        // Extract each sprite
        for sprite in &packer.sprites {
            match extractor.extract_sprite(sprite) {
                Ok(img) => {
                    // Save the sprite
                    let sprite_path = destdir.join(format!("{}.png", sprite.name));

                    // Ensure parent directory exists
                    if let Some(parent) = sprite_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }

                    if let Err(e) = img.save(&sprite_path) {
                        log::warn!("Failed to save sprite {}: {}", sprite.name, e);
                    } else {
                        saved_count += 1;
                    }
                }
                Err(e) => {
                    log::warn!("Failed to extract sprite {}: {}", sprite.name, e);
                }
            }
        }
    }

    Ok(saved_count)
}

/// Extract individual sprites from Unity Sprite objects in an asset bundle
///
/// Unlike extract_sprite_packer() which handles Arknights' custom SpritePacker format,
/// this handles Unity's native Sprite/SpriteAtlas system used by UI elements.
///
/// This function extracts:
/// 1. Standalone Sprite objects (saved as {sprite_name}.png)
/// 2. Sprites packed in SpriteAtlas objects (saved as {atlas_name}${index}.png)
///
/// # Arguments
/// * `env_rc` - Reference to the Unity Environment containing the loaded asset bundle
/// * `destdir` - Output directory for extracted sprite PNG files
///
/// # Returns
/// Number of successfully extracted sprites
pub fn extract_sprites(env_rc: &Rc<RefCell<Environment>>, destdir: &Path) -> Result<usize> {
    use std::collections::HashMap;
    use unity_rs::classes::generated::SpriteAtlas;
    use unity_rs::export::sprite_helper::get_image_from_sprite;

    let mut saved_count = 0;

    // First, print summary of object types for debugging
    {
        let env = env_rc.borrow();
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for obj in env.objects() {
            *type_counts
                .entry(format!("{:?}", obj.obj_type))
                .or_insert(0) += 1;
        }
        log::info!("Object types in bundle: {:?}", type_counts);
    }

    // 1. Extract standalone Sprite objects
    {
        let env = env_rc.borrow();
        let mut texture_cache: HashMap<(i64, i64), RgbaImage> = HashMap::new();

        for mut obj in env.objects() {
            if obj.obj_type != ClassIDType::Sprite {
                continue;
            }

            // Read sprite data
            let data = match obj.read(false) {
                Ok(d) => d,
                Err(e) => {
                    log::debug!("Failed to read sprite object: {}", e);
                    continue;
                }
            };

            // Get sprite name
            let sprite_name = data
                .get("m_Name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            // Parse as Sprite struct
            let sprite: unity_rs::generated::Sprite = match serde_json::from_value(data.clone()) {
                Ok(s) => s,
                Err(e) => {
                    log::debug!("Failed to parse sprite '{}': {}", sprite_name, e);
                    continue;
                }
            };

            // Get the SerializedFile for this sprite
            let assets_file = match obj.assets_file.as_ref().and_then(|w| w.upgrade()) {
                Some(sf) => sf,
                None => {
                    log::debug!("No assets_file for sprite '{}'", sprite_name);
                    continue;
                }
            };

            // Extract sprite image using the sprite_helper
            let image = match get_image_from_sprite(&sprite, &assets_file, &mut texture_cache) {
                Ok(img) => img,
                Err(e) => {
                    log::warn!("Failed to extract sprite '{}': {}", sprite_name, e);
                    continue;
                }
            };

            // Save sprite
            let output_path = destdir.join(format!("{}.png", sprite_name));

            // Ensure parent directory exists
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            if let Err(e) = image.save(&output_path) {
                log::warn!("Failed to save sprite '{}': {}", sprite_name, e);
            } else {
                saved_count += 1;
            }
        }
    }

    // 2. Extract sprites from SpriteAtlas objects
    {
        let env = env_rc.borrow();
        let mut texture_cache: HashMap<(i64, i64), RgbaImage> = HashMap::new();

        for mut obj in env.objects() {
            if obj.obj_type != ClassIDType::SpriteAtlas {
                continue;
            }

            let data = match obj.read(false) {
                Ok(d) => d,
                Err(e) => {
                    log::debug!("Failed to read SpriteAtlas: {}", e);
                    continue;
                }
            };

            let sprite_atlas: SpriteAtlas = match serde_json::from_value(data) {
                Ok(sa) => sa,
                Err(e) => {
                    log::debug!("Failed to parse SpriteAtlas: {}", e);
                    continue;
                }
            };

            let atlas_name = sprite_atlas.m_Name.as_deref().unwrap_or("SpriteAtlas");
            log::info!("Found SpriteAtlas: {}", atlas_name);

            // Get the assets file
            let assets_file = match obj.assets_file.as_ref().and_then(|w| w.upgrade()) {
                Some(sf) => sf,
                None => continue,
            };

            // Extract from packed sprites if available
            if let Some(packed_sprites) = &sprite_atlas.m_PackedSprites {
                log::info!(
                    "SpriteAtlas '{}' has {} packed sprites",
                    atlas_name,
                    packed_sprites.len()
                );

                for (index, sprite_ptr) in packed_sprites.iter().enumerate() {
                    if sprite_ptr.m_path_id == 0 {
                        continue;
                    }

                    let sprite_result = {
                        let assets_file_ref = assets_file.borrow();
                        sprite_ptr.read(&assets_file_ref)
                    };
                    let sprite = match sprite_result {
                        Ok(s) => s,
                        Err(e) => {
                            log::debug!("Failed to read packed sprite {}: {}", index, e);
                            continue;
                        }
                    };

                    // Get sprite name if available, otherwise use index
                    let sprite_name = sprite
                        .m_Name
                        .as_deref()
                        .filter(|n| !n.is_empty())
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| format!("{}${}", atlas_name, index));

                    match get_image_from_sprite(&sprite, &assets_file, &mut texture_cache) {
                        Ok(img) => {
                            let output_path = destdir.join(format!("{}.png", sprite_name));
                            if let Some(parent) = output_path.parent() {
                                std::fs::create_dir_all(parent)?;
                            }
                            if img.save(&output_path).is_ok() {
                                saved_count += 1;
                            }
                        }
                        Err(e) => {
                            log::debug!("Failed to extract packed sprite '{}': {}", sprite_name, e);
                        }
                    }
                }
            } else {
                log::info!("SpriteAtlas '{}' has no packed sprites", atlas_name);
            }
        }
    }

    Ok(saved_count)
}
