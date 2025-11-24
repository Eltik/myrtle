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
    let mut texture_objects: std::collections::HashMap<
        String,
        unity_rs::files::object_reader::ObjectReader<()>,
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
                        texture_objects.insert(name.to_string(), obj.clone());
                    }
                }
            }
            _ => {}
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

                // Combine or save as-is
                let final_image = match (rgb_image, alpha_image) {
                    (Some(rgb), Some(alpha)) => {
                        // Combine RGB with alpha channel
                        log::debug!("Combining RGB+Alpha for spine texture: {}", base);
                        Some(combine_spine_rgba_with_alpha(&rgb, &alpha))
                    }
                    (Some(rgb), None) => {
                        // No alpha, use RGB as-is
                        Some(rgb)
                    }
                    (None, Some(alpha)) => {
                        // Only alpha exists (unusual), save it
                        log::warn!("Spine texture '{}' has alpha but no RGB", base);
                        Some(alpha)
                    }
                    (None, None) => {
                        log::warn!("Spine texture '{}' not found", base);
                        None
                    }
                };

                // Save the final image
                if let Some(img) = final_image {
                    let tex_path = asset_dir.join(tex_name);
                    if img.save(&tex_path).is_ok() {
                        saved_count += 1;
                        log::debug!("Saved spine texture: {:?}", tex_path);
                    }
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

/// Check if texture name indicates alpha channel
fn is_alpha_texture(name: &str) -> bool {
    name.contains("[alpha]") || name.ends_with("_alpha") || name.ends_with("a")
}

/// Find matching RGB texture for alpha
fn find_rgb_texture_name(alpha_name: &str) -> Option<String> {
    if alpha_name.contains("[alpha]") {
        Some(alpha_name.replace("[alpha]", ""))
    } else if alpha_name.ends_with("_alpha") {
        Some(alpha_name.trim_end_matches("_alpha").to_string())
    } else {
        None
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

    // Extract images (Texture2D and Sprite) with alpha/RGB combination
    if do_img {
        saved_count += extract_images_with_combination(&env_rc, destdir, cmb_destdir)?;
        saved_count += extract_sprite_atlases(&env_rc, destdir)?;
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
                                            let output_path = destdir.join(format!("{}.wav", clean_name));
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

    Ok(saved_count)
}

/// Combine RGB image with alpha channel for Spine textures
fn combine_spine_rgba_with_alpha(rgb: &RgbaImage, alpha: &RgbaImage) -> RgbaImage {
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

                        textures.insert(name.to_string(), TextureData { image });
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

    // Save all textures separately (including alpha textures)
    for (name, tex_data) in &textures {
        if let Some(ref img) = tex_data.image {
            let output_path = destdir.join(format!("{}.png", name));
            if img.save(&output_path).is_ok() {
                saved_count += 1;
                log::debug!("Saved texture: {:?}", output_path);
            }
        }
    }

    // Create combined $0.png images for RGB+Alpha pairs
    // Find all alpha textures and combine them with their RGB counterparts
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

/// Main entry point for asset extraction
pub fn main(
    src: &Path,
    destdir: &Path,
    do_del: bool,
    do_img: bool,
    do_txt: bool,
    do_aud: bool,
    do_spine: bool,
    separate: bool,
    force: bool,
    threads: usize,
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

    // Collect all AB files
    let files: Vec<PathBuf> = if src.is_file() {
        vec![src.to_path_buf()]
    } else {
        WalkDir::new(src)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
            .filter(|p| is_ab_file(p))
            .collect()
    };

    if files.is_empty() {
        println!("No AB files found");
        return Ok(());
    }

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
    let pb = ProgressBar::new(files.len() as u64);
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
    let save_counter = AtomicUsize::new(0);

    if threads == 1 {
        // Sequential processing for minimal memory usage
        for file in &files_to_process {
            let (upk_subdestdir, cmb_subdestdir) = if separate {
                let rel_path = file.strip_prefix(&src_path).unwrap_or(file);
                let parent = rel_path.parent().unwrap_or(Path::new(""));
                let stem = file.file_stem().unwrap_or_default();
                (
                    upk_dir.join(parent).join(stem),
                    cmb_dir.join(parent).join(stem),
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
            ) {
                Ok(count) => {
                    total_saved.fetch_add(count, Ordering::Relaxed);
                    manifest.lock().unwrap().update(file, count);

                    // Save manifest every 100 files
                    if save_counter.fetch_add(1, Ordering::Relaxed) % 100 == 0 {
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
        }
    } else {
        // Parallel processing for better performance
        files_to_process.par_iter().for_each(|file| {
            let (upk_subdestdir, cmb_subdestdir) = if separate {
                let rel_path = file.strip_prefix(&src_path).unwrap_or(file);
                let parent = rel_path.parent().unwrap_or(Path::new(""));
                let stem = file.file_stem().unwrap_or_default();
                (
                    upk_dir.join(parent).join(stem),
                    cmb_dir.join(parent).join(stem),
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
            ) {
                Ok(count) => {
                    total_saved.fetch_add(count, Ordering::Relaxed);
                    manifest.lock().unwrap().update(file, count);

                    // Save manifest every 100 files
                    if save_counter.fetch_add(1, Ordering::Relaxed) % 100 == 0 {
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
    }

    pb.finish_with_message("Done!");

    // Save manifest
    if let Err(e) = manifest.lock().unwrap().save(&manifest_path) {
        log::warn!("Failed to save manifest: {}", e);
    }

    println!("\nExtraction complete!");
    println!("  Total files: {}", files.len());
    println!("  Processed {} files", files_to_process.len());
    println!("  Skipped (unchanged): {}", skipped);
    println!("  Exported {} assets", total_saved.load(Ordering::Relaxed));

    Ok(())
}
