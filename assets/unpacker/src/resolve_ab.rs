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
    /// Determine Spine asset type from name and atlas content
    /// Uses Python's logic: count animation prefixes in atlas to determine front vs back
    pub fn from_name_and_atlas(name: &str, atlas_content: Option<&str>) -> Self {
        let lower_name = name.to_lowercase();

        // DynIllust: name starts with dyn_
        if lower_name.starts_with("dyn_") {
            return SpineAssetType::DynIllust;
        }

        // Building: name starts with build_ OR has 'Relax' animation
        if lower_name.starts_with("build_") {
            return SpineAssetType::Building;
        }

        // Check atlas for 'Relax' animation (Building indicator)
        if let Some(atlas) = atlas_content {
            if atlas.contains("Relax") {
                return SpineAssetType::Building;
            }

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
    let mut lines = atlas_content.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        // Atlas format: texture filename is on its own line ending with .png
        // followed by size:, format:, filter:, etc.
        if trimmed.ends_with(".png") {
            textures.push(trimmed.to_string());
        }
    }

    textures
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

/// Extract Spine assets from MonoBehaviours
fn extract_spine_assets(env_rc: &Rc<RefCell<Environment>>, destdir: &Path) -> Result<usize> {
    let env = env_rc.borrow();
    let mut saved_count = 0;

    // Collect all assets - use Vec to preserve duplicates with same name
    let mut textassets: Vec<(String, serde_json::Value)> = Vec::new();
    // Store ALL texture objects, including duplicates, keyed by name
    let mut texture_objects_all: std::collections::HashMap<
        String,
        Vec<unity_rs::files::object_reader::ObjectReader<()>>,
    > = std::collections::HashMap::new();

    for mut obj in env.objects() {
        match obj.obj_type {
            ClassIDType::TextAsset => {
                if let Ok(data) = obj.read(false) {
                    if let Some(name) = data.get("m_Name").and_then(|v| v.as_str()) {
                        textassets.push((name.to_string(), data));
                    }
                }
            }
            ClassIDType::Texture2D => {
                if let Ok(data) = obj.read(false) {
                    if let Some(name) = data.get("m_Name").and_then(|v| v.as_str()) {
                        texture_objects_all
                            .entry(name.to_string())
                            .or_insert_with(Vec::new)
                            .push(obj.clone());
                    }
                }
            }
            _ => {}
        }
    }

    // Resolve duplicates by decoding and picking the texture with more colors
    let mut texture_objects: std::collections::HashMap<
        String,
        unity_rs::files::object_reader::ObjectReader<()>,
    > = std::collections::HashMap::new();

    for (name, objects) in texture_objects_all {
        if objects.len() == 1 {
            texture_objects.insert(name, objects.into_iter().next().unwrap());
        } else {
            // Multiple textures with same name - decode all and pick best
            let mut best_obj = None;
            let mut best_colored_pixels = 0usize;

            for mut tex_obj in objects {
                if let Ok(data) = tex_obj.read(false) {
                    if let Ok(mut texture) =
                        serde_json::from_value::<unity_rs::generated::Texture2D>(data)
                    {
                        texture.object_reader = Some(Box::new(tex_obj.clone()));
                        if let Ok(img) = texture_to_image(&texture) {
                            let colored = img
                                .pixels()
                                .filter(|p| p[0] as u32 + p[1] as u32 + p[2] as u32 > 30)
                                .count();
                            if colored > best_colored_pixels {
                                best_colored_pixels = colored;
                                best_obj = Some(tex_obj);
                            }
                        }
                    }
                }
            }

            if let Some(obj) = best_obj {
                log::debug!(
                    "Selected best texture '{}' with {} colored pixels",
                    name,
                    best_colored_pixels
                );
                texture_objects.insert(name, obj);
            }
        }
    }

    // Find Spine asset groups - process each skel+atlas pair separately
    // Key: (base_name, index) to handle multiple instances with same name
    let mut spine_assets: Vec<SpineAsset> = Vec::new();

    // First, collect all skel and atlas files with their content
    let mut skels: Vec<(String, Vec<u8>)> = Vec::new();
    let mut atlases: Vec<(String, String)> = Vec::new();

    for (name, data) in &textassets {
        if name.ends_with(".skel") {
            let base_name = name.trim_end_matches(".skel").to_string();
            if let Some(script) = data.get("m_Script").and_then(|v| v.as_str()) {
                skels.push((base_name, script.as_bytes().to_vec()));
            }
        } else if name.ends_with(".atlas") {
            let base_name = name.trim_end_matches(".atlas").to_string();
            if let Some(script) = data.get("m_Script").and_then(|v| v.as_str()) {
                atlases.push((base_name, script.to_string()));
            }
        }
    }

    // Match skels with atlases by name and create SpineAssets
    // Each skel-atlas pair with the same name creates one SpineAsset
    let mut used_atlas_indices: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for (skel_name, skel_data) in &skels {
        // Find matching atlas (one per skel)
        let mut matched_atlas: Option<(usize, &String)> = None;
        for (idx, (atlas_name, atlas_content)) in atlases.iter().enumerate() {
            if atlas_name == skel_name && !used_atlas_indices.contains(&idx) {
                matched_atlas = Some((idx, atlas_content));
                break;
            }
        }

        if let Some((atlas_idx, atlas_content)) = matched_atlas {
            used_atlas_indices.insert(atlas_idx);

            let asset_type = SpineAssetType::from_name_and_atlas(skel_name, Some(atlas_content));

            spine_assets.push(SpineAsset {
                name: skel_name.clone(),
                asset_type,
                skel_data: Some(skel_data.clone()),
                atlas_data: Some(atlas_content.clone()),
                textures: Vec::new(),
            });
        } else {
            // Skel without atlas
            let asset_type = SpineAssetType::from_name_and_atlas(skel_name, None);
            spine_assets.push(SpineAsset {
                name: skel_name.clone(),
                asset_type,
                skel_data: Some(skel_data.clone()),
                atlas_data: None,
                textures: Vec::new(),
            });
        }
    }

    // Also create assets for atlases without skels
    for (idx, (atlas_name, atlas_content)) in atlases.iter().enumerate() {
        if !used_atlas_indices.contains(&idx) {
            let asset_type = SpineAssetType::from_name_and_atlas(atlas_name, Some(atlas_content));
            spine_assets.push(SpineAsset {
                name: atlas_name.clone(),
                asset_type,
                skel_data: None,
                atlas_data: Some(atlas_content.clone()),
                textures: Vec::new(),
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
    for (_key, asset) in &spine_groups {
        if asset.skel_data.is_none() && asset.atlas_data.is_none() {
            continue;
        }

        // Create directory structure: <type>/<asset_name>/
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

            // Also save associated textures
            let required_textures = parse_atlas_textures(atlas);
            for tex_name in &required_textures {
                let base = tex_name.trim_end_matches(".png");

                // Try to get RGB texture
                let rgb_image = if let Some(tex_obj) = texture_objects.get(base) {
                    let mut tex_obj = tex_obj.clone();
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

                // Try to get alpha texture
                let alpha_name = format!("{}[alpha]", base);
                let alpha_image = if let Some(alpha_obj) = texture_objects.get(&alpha_name) {
                    let mut alpha_obj = alpha_obj.clone();
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

                if asset.asset_type == SpineAssetType::BattleFront
                    || asset.asset_type == SpineAssetType::BattleBack
                {
                    // Combine RGB + alpha for battle sprites
                    if let (Some(ref rgb), Some(ref alpha)) = (&rgb_image, &alpha_image) {
                        let combined = combine_rgba_with_alpha(rgb, alpha);
                        if combined.save(&tex_path).is_ok() {
                            saved_count += 1;
                            log::debug!("Saved combined spine texture: {:?}", tex_path);
                        }
                    } else if let Some(ref rgb) = rgb_image {
                        // No alpha available, save RGB as-is
                        if rgb.save(&tex_path).is_ok() {
                            saved_count += 1;
                            log::debug!("Saved spine RGB texture (no alpha): {:?}", tex_path);
                        }
                    }
                } else {
                    // Building and other types: save RGB only
                    if let Some(ref rgb) = rgb_image {
                        if rgb.save(&tex_path).is_ok() {
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

/// Collect names of textures that should be excluded from root extraction
/// Excludes: alpha textures, building textures, and base spine textures (same name as atlas/skel)
fn collect_spine_texture_names(
    env_rc: &Rc<RefCell<Environment>>,
) -> std::collections::HashSet<String> {
    let env = env_rc.borrow();
    let mut excluded_textures = std::collections::HashSet::new();

    // Collect texture names from atlas/skel files
    for mut obj in env.objects() {
        if obj.obj_type == ClassIDType::TextAsset {
            if let Ok(data) = obj.read(false) {
                if let Some(name) = data.get("m_Name").and_then(|v| v.as_str()) {
                    if name.ends_with(".atlas") {
                        let base_name = name.trim_end_matches(".atlas");

                        // Exclude base spine texture from root (it goes to BattleFront/BattleBack subdirs only)
                        // This prevents chibi-sized sprites from appearing at root
                        excluded_textures.insert(base_name.to_string());

                        // Exclude alpha for atlas base name
                        excluded_textures.insert(format!("{}[alpha]", base_name));

                        // Building textures go ONLY to Building/ subdirectory, not root
                        if base_name.starts_with("build_") {
                            excluded_textures.insert(base_name.to_string());
                        }

                        // Get atlas content and extract texture names (for multi-texture atlases)
                        if let Some(script) = data.get("m_Script").and_then(|v| v.as_str()) {
                            let textures = parse_atlas_textures(script);
                            for tex_name in textures {
                                let base = tex_name.trim_end_matches(".png");
                                // Exclude alpha textures from root
                                excluded_textures.insert(format!("{}[alpha]", base));
                            }
                        }
                    }

                    // Also track skel files for building variants
                    if name.ends_with(".skel") {
                        let base_name = name.trim_end_matches(".skel");
                        // Exclude base spine texture from root
                        excluded_textures.insert(base_name.to_string());
                        // Exclude alpha textures
                        excluded_textures.insert(format!("{}[alpha]", base_name));
                        // Exclude building variant from root
                        if !base_name.starts_with("build_") {
                            excluded_textures.insert(format!("build_{}", base_name));
                        }
                    }
                }
            }
        }
    }

    log::debug!(
        "Collected {} texture names to exclude from root: {:?}",
        excluded_textures.len(),
        excluded_textures
    );
    excluded_textures
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

    // If spine extraction is enabled, first collect spine texture names to exclude from root
    let spine_texture_names: std::collections::HashSet<String> = if do_spine {
        collect_spine_texture_names(&env_rc)
    } else {
        std::collections::HashSet::new()
    };

    // Extract images (Texture2D and Sprite) with alpha/RGB combination
    if do_img {
        saved_count += extract_images_with_combination(
            &env_rc,
            destdir,
            cmb_destdir,
            merge_alpha,
            &spine_texture_names,
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
                                        if !script_bytes.is_empty() {
                                            if std::fs::write(&output_path, &script_bytes).is_ok() {
                                                saved_count += 1;
                                                saved = true;
                                                log::debug!(
                                                    "Saved text asset (bytes): {:?}",
                                                    output_path
                                                );
                                            }
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
                                        if !bytes.is_empty() {
                                            if std::fs::write(&output_path, &bytes).is_ok() {
                                                saved_count += 1;
                                                saved = true;
                                                log::debug!(
                                                    "Saved text asset (m_Bytes): {:?}",
                                                    output_path
                                                );
                                            }
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
            &mut *assets_file_mut,
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
                        let atlas_name = sprite_atlas
                            .m_Name
                            .as_ref()
                            .map(|s| s.as_str())
                            .unwrap_or("SpriteAtlas");

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
    spine_texture_names: &std::collections::HashSet<String>,
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

    // Handle alpha textures based on merge_alpha setting
    // Skip textures that belong to spine assets (they will be handled by spine extraction)
    if merge_alpha {
        // Merge alpha into RGBA images
        // Find RGB+Alpha pairs first
        let mut processed_names: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for name in textures.keys() {
            // Skip spine textures
            if spine_texture_names.contains(name) {
                continue;
            }
            if name.contains("[alpha]") {
                let rgb_name = name.replace("[alpha]", "");
                // Skip if RGB is a spine texture
                if spine_texture_names.contains(&rgb_name) {
                    continue;
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
                            log::debug!("Saved merged texture: {:?}", output_path);
                        }
                    }
                }
            }
        }

        // Save remaining textures that weren't part of pairs
        for (name, tex_data) in &textures {
            // Skip spine textures
            if spine_texture_names.contains(name) {
                log::debug!("Skipping spine texture from root: {}", name);
                continue;
            }
            if !processed_names.contains(name) {
                if let Some(ref img) = tex_data.image {
                    let output_path = destdir.join(format!("{}.png", name));
                    if img.save(&output_path).is_ok() {
                        saved_count += 1;
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
            // Skip spine textures - they will be handled by spine extraction
            if spine_texture_names.contains(name) {
                log::debug!("Skipping spine texture from root: {}", name);
                continue;
            }

            // Skip [alpha] textures - they'll be merged into their RGB counterpart
            if name.contains("[alpha]") {
                log::debug!("Skipping alpha texture (will be merged): {}", name);
                continue;
            }

            // Check if this RGB texture has a matching alpha texture
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
                        log::debug!("Saved texture: {:?}", output_path);
                    }
                }
            }
        }
    }

    // Track which textures were merged (to avoid sprite extraction overwriting them)
    let merged_texture_names: std::collections::HashSet<String> = textures
        .keys()
        .filter(|name| !name.contains("[alpha]"))
        .filter(|name| {
            let alpha_name = format!("{}[alpha]", name);
            textures.contains_key(&alpha_name)
        })
        .cloned()
        .collect();

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
                // Skip sprites that would overwrite merged texture files
                // (sprites with same name as textures that have alpha merged)
                if merged_texture_names.contains(&name) {
                    log::debug!("Skipping sprite {} (already saved as merged texture)", name);
                } else {
                    // Save sprite with its name
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
                                sprite_groups
                                    .entry(texture_name)
                                    .or_insert_with(Vec::new)
                                    .push((name.clone(), sprite.clone(), Rc::clone(&assets_file)));
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
            .chain(large_files.into_iter())
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
        let all: Vec<_> = normal_files
            .into_iter()
            .chain(large_files.into_iter())
            .collect();
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
    let src_dir_name = if src_path.is_dir() {
        src_path.file_name().unwrap_or_default().to_os_string()
    } else {
        std::ffi::OsString::new()
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
                    if file_count % 50 == 0 {
                        if let Err(e) = manifest.lock().unwrap().save(&manifest_path) {
                            log::warn!("Failed to save incremental manifest: {}", e);
                        }
                    }

                    // Log memory usage every 10 files to track leaks
                    if file_count % 10 == 0 {
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
                    if save_counter.fetch_add(1, Ordering::Relaxed) % 50 == 0 {
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
