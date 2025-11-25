/// Asset extractor for Unity files
///
/// This module provides tools for extracting assets from Unity asset files,
/// matching the functionality of UnityPy's tools.extractor module.
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write as IoWrite};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use crate::enums::class_id_type::ClassIDType;
use crate::helpers::import_helper::FileSource;
use crate::Environment;

// ============================================================================
// Spine Asset Type Classification
// ============================================================================

/// Types of Spine assets in Arknights
#[derive(Debug, Clone, PartialEq)]
pub enum SpineAssetType {
    Unknown,
    Building,    // "build_" prefix or has "Relax" animation
    BattleFront, // More "f_" or "c_" prefixes in atlas than "b_"
    BattleBack,  // More "b_" prefixes in atlas
    DynIllust,   // "dyn_" prefix
}

impl SpineAssetType {
    /// Get the directory name for this type
    pub fn as_dir_name(&self) -> &str {
        match self {
            SpineAssetType::Unknown => "Unknown",
            SpineAssetType::Building => "Building",
            SpineAssetType::BattleFront => "BattleFront",
            SpineAssetType::BattleBack => "BattleBack",
            SpineAssetType::DynIllust => "DynIllust",
        }
    }
}

/// A Spine asset consisting of skeleton, atlas, and textures
#[derive(Debug)]
pub struct SpineAsset {
    /// Skeleton file content (JSON or binary)
    pub skel_name: String,
    pub skel_data: Vec<u8>,
    /// Atlas file content
    pub atlas_name: String,
    pub atlas_data: String,
    /// List of (RGB texture path_id, Alpha texture path_id or 0)
    pub tex_list: Vec<(i64, i64)>,
    /// Asset type for directory organization
    pub asset_type: SpineAssetType,
    /// Parsed atlas data for dimensions
    pub atlas_dimensions: Option<(u32, u32)>,
}

impl SpineAsset {
    /// Create a new SpineAsset and determine its type
    pub fn new(
        skel_name: String,
        skel_data: Vec<u8>,
        atlas_name: String,
        atlas_data: String,
        tex_list: Vec<(i64, i64)>,
        anim_list: Option<Vec<String>>,
    ) -> Self {
        let mut asset = SpineAsset {
            skel_name,
            skel_data,
            atlas_name,
            atlas_data: atlas_data.clone(),
            tex_list,
            asset_type: SpineAssetType::Unknown,
            atlas_dimensions: None,
        };

        // Determine asset type
        asset.determine_type(&anim_list);

        // Parse atlas dimensions
        asset.parse_atlas_dimensions();

        asset
    }

    /// Determine the asset type based on name and atlas content
    fn determine_type(&mut self, anim_list: &Option<Vec<String>>) {
        let skel_name_lower = self.skel_name.to_lowercase();

        // Rule 1: Dynamic illustrations start with "dyn_"
        if skel_name_lower.starts_with("dyn_") {
            self.asset_type = SpineAssetType::DynIllust;
            return;
        }

        // Rule 2: Building models have "Relax" animation or start with "build_"
        if skel_name_lower.starts_with("build_") {
            self.asset_type = SpineAssetType::Building;
            return;
        }
        if let Some(anims) = anim_list {
            if anims.iter().any(|a| a == "Relax") {
                self.asset_type = SpineAssetType::Building;
                return;
            }
        }

        // Rule 3: Count atlas region prefixes to determine front vs back
        let atlas_lower = self.atlas_data.to_lowercase();
        let front_count = atlas_lower.matches("\nf_").count() + atlas_lower.matches("\nc_").count();
        let back_count = atlas_lower.matches("\nb_").count();

        if front_count >= back_count {
            self.asset_type = SpineAssetType::BattleFront;
        } else {
            self.asset_type = SpineAssetType::BattleBack;
        }
    }

    /// Parse atlas dimensions from atlas data
    fn parse_atlas_dimensions(&mut self) {
        // Atlas format example:
        // filename.png
        // size: 2048,2048
        // ...
        for line in self.atlas_data.lines() {
            let line = line.trim();
            if line.starts_with("size:") {
                let size_part = line.trim_start_matches("size:").trim();
                let parts: Vec<&str> = size_part.split(',').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    if let (Ok(w), Ok(h)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                        self.atlas_dimensions = Some((w, h));
                        return;
                    }
                }
            }
        }
    }

    /// Get the prefix path for this asset (e.g., "BattleFront/char_xxx/")
    pub fn get_prefix(&self) -> String {
        let atlas_basename = Path::new(&self.atlas_name)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        format!("{}/{}/", self.asset_type.as_dir_name(), atlas_basename)
    }
}

// ============================================================================
// Atlas Parser
// ============================================================================

/// Parsed atlas region
#[derive(Debug)]
pub struct AtlasRegion {
    pub name: String,
    pub index: i32, // -1 = no index, >= 0 = indexed variant ($0, $1, etc.)
}

/// Parse atlas file to extract regions with index values
pub fn parse_atlas_regions(atlas_text: &str) -> Vec<AtlasRegion> {
    let mut regions = Vec::new();
    let mut current_region: Option<String> = None;
    let mut current_index: i32 = -1;

    for line in atlas_text.lines() {
        let line_trimmed = line.trim();

        // Skip empty lines or page header lines
        if line_trimmed.is_empty() {
            continue;
        }

        // Check if this is a region attribute (starts with whitespace in original)
        if line.starts_with(' ') || line.starts_with('\t') {
            // Parse index attribute
            if line_trimmed.starts_with("index:") {
                let value = line_trimmed.trim_start_matches("index:").trim();
                current_index = value.parse().unwrap_or(-1);
            }
        } else if line_trimmed.contains(':') {
            // This is a page attribute (size:, format:, etc.) - skip
            continue;
        } else {
            // This is a region name - save previous region first
            if let Some(name) = current_region.take() {
                regions.push(AtlasRegion {
                    name,
                    index: current_index,
                });
            }
            // Start new region
            current_region = Some(line_trimmed.to_string());
            current_index = -1;
        }
    }

    // Don't forget the last region
    if let Some(name) = current_region {
        regions.push(AtlasRegion {
            name,
            index: current_index,
        });
    }

    regions
}

// ============================================================================
// File Collision Handling
// ============================================================================

/// Returns a unique path by appending $n suffix if file exists
/// Python equivalent: SaverUtils._no_namesake()
pub fn no_namesake(dest: &Path) -> PathBuf {
    if !dest.exists() {
        return dest.to_path_buf();
    }

    let dir = dest.parent().unwrap_or(Path::new("."));
    let stem = dest.file_stem().unwrap_or_default().to_string_lossy();
    let ext = dest
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    let mut result = dest.to_path_buf();
    let mut counter = 0;

    while result.exists() {
        result = dir.join(format!("{stem}${counter}{ext}"));
        counter += 1;
    }

    result
}

/// Check if data is unique (not a duplicate of existing files)
/// Python equivalent: SaverUtils._is_unique()
pub fn is_unique_content(data: &[u8], dest: &Path) -> bool {
    let dir = match dest.parent() {
        Some(d) => d,
        None => return true,
    };

    if !dir.exists() {
        return true;
    }

    let stem = dest.file_stem().unwrap_or_default().to_string_lossy();
    let ext = dest
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();

    // Check existing files with same base name pattern
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let matches_pattern = if ext.is_empty() {
                name.starts_with(&*stem)
            } else {
                name.starts_with(&*stem) && name.ends_with(&format!(".{}", ext))
            };

            if matches_pattern {
                if let Ok(existing) = fs::read(entry.path()) {
                    if existing == data {
                        return false; // Duplicate content found
                    }
                }
            }
        }
    }

    true
}

/// Global storage for MonoBehaviour TypeTree definitions
/// Maps "Assembly-Name.dll" -> "Class-Name" -> TypeTree nodes
pub static MONOBEHAVIOUR_TYPETREES: once_cell::sync::Lazy<
    RwLock<HashMap<String, HashMap<String, Vec<serde_json::Value>>>>,
> = once_cell::sync::Lazy::new(|| RwLock::new(HashMap::new()));

// ============================================================================
// Spine Asset Sorting
// ============================================================================

/// Collected objects from a Unity environment for Spine sorting
struct CollectedObjects {
    monobehaviors: Vec<crate::files::object_reader::ObjectReader<()>>,
    textassets: Vec<crate::files::object_reader::ObjectReader<()>>,
    materials: Vec<crate::files::object_reader::ObjectReader<()>>,
    texture2ds: Vec<crate::files::object_reader::ObjectReader<()>>,
}

/// Get object by path_id from a list of objects
fn get_object_by_pathid<'a>(
    pathid: i64,
    objects: &'a [crate::files::object_reader::ObjectReader<()>],
) -> Option<&'a crate::files::object_reader::ObjectReader<()>> {
    objects.iter().find(|obj| obj.path_id == pathid)
}

/// Extract path_id from a PPtr value in TypeTree
fn extract_pathid_from_pptr(value: &serde_json::Value) -> Option<i64> {
    value.get("m_PathID").and_then(|v| v.as_i64())
}

/// Sort and collect Spine assets from MonoBehaviours
/// Python equivalent: Resource.sort_skeletons()
pub fn sort_skeletons(collected: &CollectedObjects) -> Vec<SpineAsset> {
    let mut spines = Vec::new();

    for mono in &collected.monobehaviors {
        // Try to read the MonoBehaviour's type tree
        let mut mono_clone = mono.clone();
        let tree = match mono_clone.read_typetree(None, true, false) {
            Ok(t) => t,
            Err(_) => continue,
        };

        // Check if this MonoBehaviour has skeletonDataAsset field (Spine component)
        let skel_data_ptr = match tree.get("skeletonDataAsset") {
            Some(v) => v,
            None => continue, // Not a Spine component
        };

        // Get the skeletonDataAsset PathID
        let skel_data_pathid = match extract_pathid_from_pptr(skel_data_ptr) {
            Some(id) if id != 0 => id,
            _ => continue,
        };

        // Find the skeleton data MonoBehaviour
        let mono_sd = match get_object_by_pathid(skel_data_pathid, &collected.monobehaviors) {
            Some(m) => m,
            None => continue,
        };

        // Read skeleton data type tree
        let mut mono_sd_clone = mono_sd.clone();
        let tree_sd = match mono_sd_clone.read_typetree(None, true, false) {
            Ok(t) => t,
            Err(_) => continue,
        };

        // Get skeletonJSON (TextAsset)
        let skel_json_ptr = match tree_sd.get("skeletonJSON") {
            Some(v) => v,
            None => continue,
        };
        let skel_pathid = match extract_pathid_from_pptr(skel_json_ptr) {
            Some(id) if id != 0 => id,
            _ => continue,
        };
        let skel_obj = match get_object_by_pathid(skel_pathid, &collected.textassets) {
            Some(o) => o,
            None => continue,
        };

        // Read skeleton TextAsset
        let mut skel_clone = skel_obj.clone();
        let skel_data_json = match skel_clone.read(false) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let skel_name = skel_data_json
            .get("m_Name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let skel_script = skel_data_json
            .get("m_Script")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .as_bytes()
            .to_vec();

        // Get atlasAssets array
        let atlas_assets = match tree_sd.get("atlasAssets").and_then(|v| v.as_array()) {
            Some(arr) if !arr.is_empty() => arr,
            _ => continue,
        };

        // Get first atlas asset MonoBehaviour
        let atlas_data_pathid = match extract_pathid_from_pptr(&atlas_assets[0]) {
            Some(id) if id != 0 => id,
            _ => continue,
        };
        let mono_ad = match get_object_by_pathid(atlas_data_pathid, &collected.monobehaviors) {
            Some(m) => m,
            None => continue,
        };

        // Read atlas data type tree
        let mut mono_ad_clone = mono_ad.clone();
        let tree_ad = match mono_ad_clone.read_typetree(None, true, false) {
            Ok(t) => t,
            Err(_) => continue,
        };

        // Get atlasFile (TextAsset)
        let atlas_file_ptr = match tree_ad.get("atlasFile") {
            Some(v) => v,
            None => continue,
        };
        let atlas_pathid = match extract_pathid_from_pptr(atlas_file_ptr) {
            Some(id) if id != 0 => id,
            _ => continue,
        };
        let atlas_obj = match get_object_by_pathid(atlas_pathid, &collected.textassets) {
            Some(o) => o,
            None => continue,
        };

        // Read atlas TextAsset
        let mut atlas_clone = atlas_obj.clone();
        let atlas_data_json = match atlas_clone.read(false) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let atlas_name = atlas_data_json
            .get("m_Name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let atlas_script = atlas_data_json
            .get("m_Script")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Get materials and textures
        let materials_arr = match tree_ad.get("materials").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => continue,
        };

        let mut tex_list = Vec::new();
        for mat_ptr in materials_arr {
            let mat_pathid = match extract_pathid_from_pptr(mat_ptr) {
                Some(id) if id != 0 => id,
                _ => continue,
            };
            let mat_obj = match get_object_by_pathid(mat_pathid, &collected.materials) {
                Some(m) => m,
                None => continue,
            };

            // Read material type tree
            let mut mat_clone = mat_obj.clone();
            let tree_mat = match mat_clone.read_typetree(None, true, false) {
                Ok(t) => t,
                Err(_) => continue,
            };

            // Get texture references from m_SavedProperties.m_TexEnvs
            let tex_envs = tree_mat
                .get("m_SavedProperties")
                .and_then(|p| p.get("m_TexEnvs"))
                .and_then(|t| t.as_array());

            let mut tex_rgb_id: i64 = 0;
            let mut tex_alpha_id: i64 = 0;

            if let Some(envs) = tex_envs {
                for tex in envs {
                    if let Some(arr) = tex.as_array() {
                        if arr.len() >= 2 {
                            let name = arr[0].as_str().unwrap_or("");
                            let tex_ptr = arr[1].get("m_Texture");

                            match name {
                                "_MainTex" => {
                                    if let Some(id) = tex_ptr.and_then(extract_pathid_from_pptr) {
                                        tex_rgb_id = id;
                                    }
                                }
                                "_AlphaTex" => {
                                    if let Some(id) = tex_ptr.and_then(extract_pathid_from_pptr) {
                                        tex_alpha_id = id;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            if tex_rgb_id != 0 {
                tex_list.push((tex_rgb_id, tex_alpha_id));
            }
        }

        if tex_list.is_empty() {
            continue;
        }

        // Get animation names for type classification
        let anim_list = tree
            .get("_animationName")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            });

        // Create SpineAsset
        let spine = SpineAsset::new(
            skel_name,
            skel_script,
            atlas_name,
            atlas_script,
            tex_list,
            anim_list,
        );
        spines.push(spine);
    }

    spines
}

/// Collect all objects from environment by type for Spine sorting
fn collect_objects_for_spine(
    env_rc: &std::rc::Rc<std::cell::RefCell<Environment>>,
) -> CollectedObjects {
    let mut collected = CollectedObjects {
        monobehaviors: Vec::new(),
        textassets: Vec::new(),
        materials: Vec::new(),
        texture2ds: Vec::new(),
    };

    let env = env_rc.borrow();
    for (_name, file_rc) in &env.files {
        let file_ref = file_rc.borrow();
        match &*file_ref {
            crate::files::bundle_file::FileType::SerializedFile(sf_rc) => {
                let sf = sf_rc.borrow();
                for (_id, obj) in &sf.objects {
                    match obj.obj_type {
                        ClassIDType::MonoBehaviour => collected.monobehaviors.push(obj.clone()),
                        ClassIDType::TextAsset => collected.textassets.push(obj.clone()),
                        ClassIDType::Material => collected.materials.push(obj.clone()),
                        ClassIDType::Texture2D => collected.texture2ds.push(obj.clone()),
                        _ => {}
                    }
                }
            }
            crate::files::bundle_file::FileType::BundleFile(bundle) => {
                for (_bundle_name, bundle_file_rc) in &bundle.files {
                    let bundle_file_ref = bundle_file_rc.borrow();
                    if let crate::files::bundle_file::FileType::SerializedFile(sf_rc) =
                        &*bundle_file_ref
                    {
                        let sf = sf_rc.borrow();
                        for (_id, obj) in &sf.objects {
                            match obj.obj_type {
                                ClassIDType::MonoBehaviour => {
                                    collected.monobehaviors.push(obj.clone())
                                }
                                ClassIDType::TextAsset => collected.textassets.push(obj.clone()),
                                ClassIDType::Material => collected.materials.push(obj.clone()),
                                ClassIDType::Texture2D => collected.texture2ds.push(obj.clone()),
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    collected
}

/// Save a Spine asset to disk with proper directory structure and alpha preservation
/// Python equivalent: SpineAsset.save_spine()
fn save_spine_asset(
    spine: &SpineAsset,
    collected: &CollectedObjects,
    dst: &Path,
    merge_alpha: bool,
) -> io::Result<usize> {
    let mut saved_count = 0;
    let prefix = spine.get_prefix();

    // Save textures
    for (rgb_id, alpha_id) in &spine.tex_list {
        // Get RGB texture
        let rgb_obj = match get_object_by_pathid(*rgb_id, &collected.texture2ds) {
            Some(o) => o,
            None => continue,
        };

        // Read RGB texture
        let mut rgb_clone = rgb_obj.clone();
        let rgb_data = match rgb_clone.read(false) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let tex_name = rgb_data
            .get("m_Name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        // Get texture with object_reader set for StreamData access
        use crate::generated::Texture2D;
        let mut rgb_texture: Texture2D = match serde_json::from_value(rgb_data.clone()) {
            Ok(t) => t,
            Err(_) => continue,
        };
        rgb_texture.object_reader = Some(Box::new(rgb_obj.clone()));

        // Create output path with prefix
        let tex_output_dir = dst.join(&prefix);
        fs::create_dir_all(&tex_output_dir)?;

        // Get alpha texture if exists
        let alpha_texture: Option<Texture2D> = if *alpha_id != 0 {
            if let Some(alpha_obj) = get_object_by_pathid(*alpha_id, &collected.texture2ds) {
                let mut alpha_clone = alpha_obj.clone();
                if let Ok(alpha_data) = alpha_clone.read(false) {
                    let mut tex: Texture2D = match serde_json::from_value(alpha_data) {
                        Ok(t) => t,
                        Err(_) => continue,
                    };
                    tex.object_reader = Some(Box::new(alpha_obj.clone()));
                    Some(tex)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Save RGB texture (with collision handling)
        let rgb_path = tex_output_dir.join(format!("{}.png", tex_name));
        let rgb_final_path = no_namesake(&rgb_path);

        if merge_alpha {
            // Merge alpha into RGBA and save single file
            if let Some(ref alpha_tex) = alpha_texture {
                // Save RGB first, then load and merge with alpha
                // This is less efficient but avoids exposing internal functions
                let temp_rgb = tex_output_dir.join(format!("_temp_rgb_{}.png", tex_name));
                let temp_alpha = tex_output_dir.join(format!("_temp_alpha_{}.png", tex_name));

                // Save both temporarily
                crate::export::texture_2d_converter::save_texture_as_png(
                    &rgb_texture,
                    &temp_rgb,
                    true,
                )
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
                crate::export::texture_2d_converter::save_texture_as_png(
                    alpha_tex,
                    &temp_alpha,
                    true,
                )
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

                // Load and merge
                if let (Ok(rgb_img), Ok(alpha_img)) =
                    (image::open(&temp_rgb), image::open(&temp_alpha))
                {
                    let rgb_rgba = rgb_img.to_rgba8();
                    let alpha_rgba = alpha_img.to_rgba8();

                    let mut merged = image::RgbaImage::new(rgb_rgba.width(), rgb_rgba.height());
                    for (x, y, rgb_pixel) in rgb_rgba.enumerate_pixels() {
                        let alpha_pixel = if x < alpha_rgba.width() && y < alpha_rgba.height() {
                            alpha_rgba.get_pixel(x, y)
                        } else {
                            &image::Rgba([255, 255, 255, 255])
                        };
                        merged.put_pixel(
                            x,
                            y,
                            image::Rgba([
                                rgb_pixel[0],
                                rgb_pixel[1],
                                rgb_pixel[2],
                                alpha_pixel[0], // Use red channel as alpha
                            ]),
                        );
                    }
                    merged.save(&rgb_final_path).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("Failed to save merged texture: {}", e),
                        )
                    })?;
                    saved_count += 1;
                }

                // Clean up temp files
                let _ = fs::remove_file(&temp_rgb);
                let _ = fs::remove_file(&temp_alpha);
            } else {
                // No alpha, just save RGB
                crate::export::texture_2d_converter::save_texture_as_png(
                    &rgb_texture,
                    &rgb_final_path,
                    true,
                )
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
                saved_count += 1;
            }
        } else {
            // Save RGB and Alpha separately
            crate::export::texture_2d_converter::save_texture_as_png(
                &rgb_texture,
                &rgb_final_path,
                true,
            )
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
            saved_count += 1;

            // Save alpha texture separately with [alpha] suffix
            if let Some(ref alpha_tex) = alpha_texture {
                let alpha_path = tex_output_dir.join(format!("{}[alpha].png", tex_name));
                let alpha_final_path = no_namesake(&alpha_path);
                crate::export::texture_2d_converter::save_texture_as_png(
                    alpha_tex,
                    &alpha_final_path,
                    true,
                )
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
                saved_count += 1;
            }
        }
    }

    // Save atlas file
    let atlas_output_dir = dst.join(&prefix);
    fs::create_dir_all(&atlas_output_dir)?;
    let atlas_path = atlas_output_dir.join(format!(
        "{}.atlas",
        Path::new(&spine.atlas_name)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
    ));
    let atlas_final_path = no_namesake(&atlas_path);
    fs::write(&atlas_final_path, &spine.atlas_data)?;
    saved_count += 1;

    // Save skeleton file
    let skel_path = atlas_output_dir.join(format!(
        "{}.skel",
        Path::new(&spine.skel_name)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
    ));
    let skel_final_path = no_namesake(&skel_path);
    fs::write(&skel_final_path, &spine.skel_data)?;
    saved_count += 1;

    Ok(saved_count)
}

/// Extracts assets from Unity file(s) to a destination directory with Spine support.
///
/// This function matches the Python `extract_assets()` function, providing
/// batch extraction of Unity assets with optional filtering and organization.
///
/// # Arguments
///
/// * `src` - Source file or folder path
/// * `dst` - Destination directory path
/// * `use_container` - If true, use container paths for organizing output
///
/// # Returns
///
/// Number of files extracted
///
/// # Example
///
/// ```no_run
/// use unity_rs::tools::extract_assets;
///
/// let extracted = extract_assets(
///     "assets.unity3d",
///     "output/",
///     true
/// ).unwrap();
/// println!("Extracted {} files", extracted);
/// ```
pub fn extract_assets<P: AsRef<Path>, S: AsRef<Path>>(
    src: S,
    dst: P,
    use_container: bool,
) -> io::Result<usize> {
    let src_path = src.as_ref();
    let mut env = Environment::new();

    // Load source - handle both files and directories
    if src_path.is_dir() {
        env.load_folder(
            src_path.to_str().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid source path")
            })?,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
    } else {
        env.load_file(
            FileSource::Path(src_path.to_string_lossy().to_string()),
            None,
            false,
        )
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Failed to load source"))?;
    }

    if cfg!(debug_assertions) {
        eprintln!("Loaded {} files from environment", env.files.len());
    }

    // Set environment references on all SerializedFiles for resource resolution
    let env_rc = std::rc::Rc::new(std::cell::RefCell::new(env));
    Environment::set_environment_references(&env_rc)?;

    let dst = dst.as_ref();
    fs::create_dir_all(dst)?;

    let mut file_count = 0;

    if use_container {
        // Use container organization
        // Collect container entries first to avoid holding env borrow
        let container = {
            let env = env_rc.borrow();
            env.container()
        };

        if cfg!(debug_assertions) {
            eprintln!("Container has {} entries", container.len());
        }
        for (obj_path, obj) in container {
            if let Some(output_path) = extract_object(&obj, dst, Some(&obj_path))? {
                file_count += 1;
                if cfg!(debug_assertions) {
                    eprintln!("Extracted: {}", output_path.display());
                }
            }
        }
    } else {
        // Extract all objects without container organization
        // Collect all objects first to avoid borrow checker issues
        let mut objects_to_extract = Vec::new();

        {
            let env = env_rc.borrow();
            for (_name, file_rc) in &env.files {
                let file_ref = file_rc.borrow();
                match &*file_ref {
                    crate::files::bundle_file::FileType::SerializedFile(sf_rc) => {
                        let sf = sf_rc.borrow();
                        for (_id, obj) in &sf.objects {
                            objects_to_extract.push(obj.clone());
                        }
                    }
                    crate::files::bundle_file::FileType::BundleFile(bundle) => {
                        // Handle SerializedFiles inside BundleFiles
                        for (_bundle_name, bundle_file_rc) in &bundle.files {
                            let bundle_file_ref = bundle_file_rc.borrow();
                            if let crate::files::bundle_file::FileType::SerializedFile(sf_rc) =
                                &*bundle_file_ref
                            {
                                let sf = sf_rc.borrow();
                                for (_id, obj) in &sf.objects {
                                    objects_to_extract.push(obj.clone());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        } // env borrow dropped here

        // Now extract all collected objects
        for obj in objects_to_extract {
            if let Some(output_path) = extract_object(&obj, dst, None)? {
                file_count += 1;
                if cfg!(debug_assertions) {
                    eprintln!("Extracted: {}", output_path.display());
                }
            }
        }
    }

    Ok(file_count)
}

/// Extracts assets from Unity file(s) with proper Spine asset handling.
///
/// This function properly:
/// 1. Identifies Spine assets and their textures
/// 2. Exports Spine assets to typed subdirectories (BattleFront/, BattleBack/, Building/)
/// 3. Exports non-Spine textures to root only
/// 4. Preserves alpha textures as separate [alpha].png files
///
/// # Arguments
///
/// * `src` - Source file or folder path
/// * `dst` - Destination directory path
/// * `do_spine` - If true, process and export Spine assets
/// * `merge_alpha` - If true, merge alpha into RGBA; if false, save [alpha].png separately
///
/// # Returns
///
/// Number of files extracted
pub fn extract_assets_with_spine<P: AsRef<Path>, S: AsRef<Path>>(
    src: S,
    dst: P,
    do_spine: bool,
    merge_alpha: bool,
) -> io::Result<usize> {
    let src_path = src.as_ref();
    let mut env = Environment::new();

    // Load source - handle both files and directories
    if src_path.is_dir() {
        env.load_folder(
            src_path.to_str().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid source path")
            })?,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
    } else {
        env.load_file(
            FileSource::Path(src_path.to_string_lossy().to_string()),
            None,
            false,
        )
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Failed to load source"))?;
    }

    if cfg!(debug_assertions) {
        eprintln!("Loaded {} files from environment", env.files.len());
    }

    // Set environment references on all SerializedFiles for resource resolution
    let env_rc = std::rc::Rc::new(std::cell::RefCell::new(env));
    Environment::set_environment_references(&env_rc)?;

    let dst = dst.as_ref();
    fs::create_dir_all(dst)?;

    let mut file_count = 0;

    // STEP 1: Collect objects by type for Spine sorting
    let collected = collect_objects_for_spine(&env_rc);

    // STEP 2: Sort and identify Spine assets
    let spines = if do_spine {
        sort_skeletons(&collected)
    } else {
        Vec::new()
    };

    if cfg!(debug_assertions) {
        eprintln!("Found {} Spine assets", spines.len());
    }

    // STEP 3: Build set of texture path_ids that belong to Spine assets
    let spine_texture_ids: HashSet<i64> = spines
        .iter()
        .flat_map(|s| s.tex_list.iter())
        .flat_map(|(rgb_id, alpha_id)| {
            let mut ids = Vec::new();
            if *rgb_id != 0 {
                ids.push(*rgb_id);
            }
            if *alpha_id != 0 {
                ids.push(*alpha_id);
            }
            ids
        })
        .collect();

    if cfg!(debug_assertions) {
        eprintln!(
            "Spine textures to filter: {} path_ids",
            spine_texture_ids.len()
        );
    }

    // STEP 4: Export Spine assets to typed subdirectories
    if do_spine {
        for spine in &spines {
            match save_spine_asset(spine, &collected, dst, merge_alpha) {
                Ok(count) => {
                    file_count += count;
                    if cfg!(debug_assertions) {
                        eprintln!(
                            "Exported Spine asset '{}' ({} files)",
                            spine.skel_name, count
                        );
                    }
                }
                Err(e) => {
                    if cfg!(debug_assertions) {
                        eprintln!("Failed to export Spine asset '{}': {}", spine.skel_name, e);
                    }
                }
            }
        }
    }

    // STEP 5: Export remaining assets (non-Spine) to root
    // For textures: Skip those belonging to Spine assets
    // For other types: Export normally

    // Collect all objects to extract
    let mut objects_to_extract = Vec::new();
    {
        let env = env_rc.borrow();
        for (_name, file_rc) in &env.files {
            let file_ref = file_rc.borrow();
            match &*file_ref {
                crate::files::bundle_file::FileType::SerializedFile(sf_rc) => {
                    let sf = sf_rc.borrow();
                    for (_id, obj) in &sf.objects {
                        objects_to_extract.push(obj.clone());
                    }
                }
                crate::files::bundle_file::FileType::BundleFile(bundle) => {
                    for (_bundle_name, bundle_file_rc) in &bundle.files {
                        let bundle_file_ref = bundle_file_rc.borrow();
                        if let crate::files::bundle_file::FileType::SerializedFile(sf_rc) =
                            &*bundle_file_ref
                        {
                            let sf = sf_rc.borrow();
                            for (_id, obj) in &sf.objects {
                                objects_to_extract.push(obj.clone());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Extract objects, filtering out Spine-related textures
    for obj in objects_to_extract {
        // Skip Spine textures - they were already exported to subdirectories
        if obj.obj_type == ClassIDType::Texture2D && spine_texture_ids.contains(&obj.path_id) {
            continue;
        }

        // Skip MonoBehaviours and Materials (Spine-related)
        if obj.obj_type == ClassIDType::MonoBehaviour || obj.obj_type == ClassIDType::Material {
            continue;
        }

        // Skip TextAssets that are Spine atlas/skeleton files
        // (they were already exported to subdirectories)
        if obj.obj_type == ClassIDType::TextAsset {
            // Check if this TextAsset is part of a Spine asset
            let is_spine_text = spines.iter().any(|spine| {
                // Check by comparing content or name would be needed here
                // For now, skip all TextAssets when do_spine is true as they're handled
                false
            });
            if is_spine_text {
                continue;
            }
        }

        // Export non-Spine objects with collision handling
        if let Some(output_path) = extract_object_with_collision(&obj, dst, None)? {
            file_count += 1;
            if cfg!(debug_assertions) {
                eprintln!("Extracted: {}", output_path.display());
            }
        }
    }

    Ok(file_count)
}

/// Extract a single object to disk with collision handling ($n suffix)
fn extract_object_with_collision(
    obj: &crate::files::object_reader::ObjectReader<()>,
    dst: &Path,
    container_path: Option<&str>,
) -> io::Result<Option<PathBuf>> {
    // Determine output path
    let base_path = if let Some(cpath) = container_path {
        let parts: Vec<&str> = cpath.split('/').filter(|s| !s.is_empty()).collect();
        dst.join(parts.join("/"))
    } else {
        let name = get_object_name(obj);
        dst.join(&name)
    };

    // Create parent directory
    if let Some(parent) = base_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Export based on type with collision handling
    let result = match obj.obj_type {
        ClassIDType::Texture2D => export_texture2d_with_collision(obj, &base_path),
        ClassIDType::Sprite => export_sprite_with_collision(obj, &base_path),
        ClassIDType::Cubemap => export_cubemap(obj, &base_path),
        ClassIDType::Texture3D => export_texture3d(obj, &base_path),
        ClassIDType::RenderTexture => export_rendertexture(obj, &base_path),
        ClassIDType::Mesh => export_mesh(obj, &base_path),
        ClassIDType::AudioClip => export_audioclip(obj, &base_path),
        ClassIDType::Shader => export_shader(obj, &base_path),
        ClassIDType::TextAsset => export_textasset(obj, &base_path),
        ClassIDType::Font => export_font(obj, &base_path),
        ClassIDType::Material => export_material_standalone(obj, &base_path),
        ClassIDType::MonoScript => export_monoscript(obj, &base_path),
        ClassIDType::AnimationClip => export_animationclip(obj, &base_path),
        ClassIDType::VideoClip => export_videoclip(obj, &base_path),
        ClassIDType::MovieTexture => export_movietexture(obj, &base_path),
        ClassIDType::MonoBehaviour => export_monobehaviour(obj, &base_path),
        ClassIDType::SpriteAtlas => export_spriteatlas(obj, &base_path),
        _ => {
            if obj.class_id as u32 == 65007 {
                export_spriteatlas(obj, &base_path)
            } else {
                Ok(None)
            }
        }
    };

    match result {
        Ok(path) => Ok(path),
        Err(e) => {
            if cfg!(debug_assertions) {
                eprintln!(
                    "Failed to export {:?} ({}): {}",
                    obj.obj_type, obj.path_id, e
                );
            }
            Ok(None)
        }
    }
}

/// Export Texture2D with collision handling ($n suffix)
fn export_texture2d_with_collision(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Texture2D;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let mut texture: Texture2D = match serde_json::from_value(data.clone()) {
        Ok(t) => t,
        Err(e) => {
            if cfg!(debug_assertions) {
                eprintln!("Texture2D deserialize error: {}", e);
            }
            return Ok(None);
        }
    };

    let width = texture.m_Width.unwrap_or(0);
    let height = texture.m_Height.unwrap_or(0);

    if width == 0 || height == 0 {
        return Ok(None);
    }

    texture.object_reader = Some(Box::new(obj.clone()));

    let output = path.with_extension("png");
    let final_path = no_namesake(&output);

    crate::export::texture_2d_converter::save_texture_as_png(&texture, &final_path, true)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    Ok(Some(final_path))
}

/// Export Sprite with collision handling ($n suffix)
fn export_sprite_with_collision(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Sprite;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let sprite: Sprite = match serde_json::from_value(data.clone()) {
        Ok(s) => s,
        Err(e) => {
            if cfg!(debug_assertions) {
                eprintln!("Sprite deserialize error: {}", e);
            }
            return Ok(None);
        }
    };

    let assets_file = obj
        .assets_file
        .as_ref()
        .and_then(|weak| weak.upgrade())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "Sprite's ObjectReader has no assets_file reference",
            )
        })?;

    let output = path.with_extension("png");
    let final_path = no_namesake(&output);

    crate::export::sprite_helper::save_sprite(
        &sprite,
        &assets_file,
        final_path
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid output path"))?,
    )
    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    Ok(Some(final_path))
}

/// Extract a single object to disk
///
/// Returns the output path if the object was extracted, None if skipped
fn extract_object(
    obj: &crate::files::object_reader::ObjectReader<()>,
    dst: &Path,
    container_path: Option<&str>,
) -> io::Result<Option<PathBuf>> {
    // Determine output path
    let output_path = if let Some(cpath) = container_path {
        // Use container path structure
        let parts: Vec<&str> = cpath.split('/').filter(|s| !s.is_empty()).collect();
        dst.join(parts.join("/"))
    } else {
        // Use object name
        let name = get_object_name(obj);
        dst.join(format!("{}_{}", name, obj.path_id))
    };

    // Create parent directory
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Export based on type
    let result = match obj.obj_type {
        ClassIDType::Texture2D => export_texture2d(obj, &output_path),
        ClassIDType::Cubemap => export_cubemap(obj, &output_path),
        ClassIDType::Texture3D => export_texture3d(obj, &output_path),
        ClassIDType::RenderTexture => export_rendertexture(obj, &output_path),
        ClassIDType::Sprite => export_sprite(obj, &output_path),
        ClassIDType::Mesh => export_mesh(obj, &output_path),
        ClassIDType::AudioClip => export_audioclip(obj, &output_path),
        ClassIDType::Shader => export_shader(obj, &output_path),
        ClassIDType::TextAsset => export_textasset(obj, &output_path),
        ClassIDType::Font => export_font(obj, &output_path),
        ClassIDType::Material => export_material_standalone(obj, &output_path),
        ClassIDType::MonoScript => export_monoscript(obj, &output_path),
        ClassIDType::AnimationClip => export_animationclip(obj, &output_path),
        ClassIDType::VideoClip => export_videoclip(obj, &output_path),
        ClassIDType::MovieTexture => export_movietexture(obj, &output_path),
        ClassIDType::MonoBehaviour => export_monobehaviour(obj, &output_path),
        ClassIDType::SpriteAtlas => export_spriteatlas(obj, &output_path),
        _ => {
            // Check for SpriteAtlas by numeric ClassID (65007) for Unity versions that use different IDs
            if obj.class_id as u32 == 65007 {
                export_spriteatlas(obj, &output_path)
            } else {
                Ok(None) // Skip unsupported types
            }
        }
    };

    // Handle errors gracefully - log and continue
    match result {
        Ok(path) => Ok(path),
        Err(e) => {
            if cfg!(debug_assertions) {
                if let Some(cpath) = container_path {
                    eprintln!(
                        "Failed to export {:?} at '{}' (PathID: {}): {}",
                        obj.obj_type, cpath, obj.path_id, e
                    );
                } else {
                    eprintln!(
                        "Failed to export {:?} ({}): {}",
                        obj.obj_type, obj.path_id, e
                    );
                }
            }
            Ok(None) // Skip failed exports instead of stopping
        }
    }
}

fn export_texture2d(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Texture2D;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    // Try to deserialize to Texture2D struct
    let mut texture: Texture2D = match serde_json::from_value(data.clone()) {
        Ok(t) => t,
        Err(e) => {
            // Deserialization failed - try to extract image data from JSON
            if cfg!(debug_assertions) {
                eprintln!(
                    "Texture2D deserialize error (attempting raw data extraction): {}",
                    e
                );
            }

            // Try to extract image_data from the raw JSON
            if let Some(image_data_val) = data.get("image_data") {
                // Check if it's a byte array
                if let Some(bytes) = image_data_val.as_array() {
                    let image_bytes: Result<Vec<u8>, _> = bytes
                        .iter()
                        .map(|v| v.as_u64().and_then(|n| u8::try_from(n).ok()))
                        .collect::<Option<Vec<u8>>>()
                        .ok_or_else(|| {
                            io::Error::new(io::ErrorKind::InvalidData, "Invalid image data")
                        });

                    if let Ok(img_bytes) = image_bytes {
                        // Try to detect format and export
                        if !img_bytes.is_empty() {
                            let output = path.with_extension("dds");
                            let mut file = fs::File::create(&output)?;
                            file.write_all(&img_bytes)?;
                            return Ok(Some(output));
                        }
                    }
                }
            }

            // If we can't extract image data, export as JSON
            let json = serde_json::to_string_pretty(&data)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let output = path.with_extension("json");
            let mut file = fs::File::create(&output)?;
            file.write_all(json.as_bytes())?;
            return Ok(Some(output));
        }
    };

    let width = texture.m_Width.unwrap_or(0);
    let height = texture.m_Height.unwrap_or(0);

    if width == 0 || height == 0 {
        return Ok(None); // Skip empty textures
    }

    // Set the object_reader so the texture can access resource data
    texture.object_reader = Some(Box::new(obj.clone()));

    // Use save_texture_as_png which handles both embedded and StreamData
    let output = path.with_extension("png");
    crate::export::texture_2d_converter::save_texture_as_png(&texture, &output, true)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    Ok(Some(output))
}

fn export_cubemap(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Cubemap;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let cubemap: Cubemap = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("Cubemap deserialize error: {}", e);
            eprintln!(
                "JSON data: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // Export cubemap data as raw binary (can be processed by other tools)
    if let Some(ref image_data) = cubemap.image_data {
        if !image_data.is_empty() {
            let output = path.with_extension("dds");
            let mut file = fs::File::create(&output)?;
            file.write_all(image_data)?;
            return Ok(Some(output));
        }
    }

    Ok(None)
}

fn export_texture3d(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Texture3D;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let texture3d: Texture3D = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("Texture3D deserialize error: {}", e);
            eprintln!(
                "JSON data: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // Export 3D texture data as raw binary
    if let Some(ref image_data) = texture3d.image_data {
        if !image_data.is_empty() {
            let output = path.with_extension("dds");
            let mut file = fs::File::create(&output)?;
            file.write_all(image_data)?;
            return Ok(Some(output));
        }
    }

    Ok(None)
}

fn export_rendertexture(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::RenderTexture;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let render_texture: RenderTexture = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("RenderTexture deserialize error: {}", e);
            eprintln!(
                "JSON data: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // RenderTexture is a render target, export metadata as JSON
    let json = serde_json::to_string_pretty(&render_texture)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let output = path.with_extension("json");
    let mut file = fs::File::create(&output)?;
    file.write_all(json.as_bytes())?;

    Ok(Some(output))
}

fn export_sprite(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Sprite;

    // Read sprite data
    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    // Try to deserialize to Sprite struct
    let sprite: Sprite = match serde_json::from_value(data.clone()) {
        Ok(s) => s,
        Err(e) => {
            // Deserialization failed - export as JSON instead
            if cfg!(debug_assertions) {
                eprintln!(
                    "Sprite deserialize error (exporting as JSON fallback): {}",
                    e
                );
            }

            let json = serde_json::to_string_pretty(&data)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let output = path.with_extension("json");
            let mut file = fs::File::create(&output)?;
            file.write_all(json.as_bytes())?;
            return Ok(Some(output));
        }
    };

    // Get the SerializedFile from the ObjectReader
    let assets_file = obj
        .assets_file
        .as_ref()
        .and_then(|weak| weak.upgrade())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "Sprite's ObjectReader has no assets_file reference",
            )
        })?;

    // Export sprite to PNG (pass Rc directly so it can be borrowed as needed inside)
    let output = path.with_extension("png");
    crate::export::sprite_helper::save_sprite(
        &sprite,
        &assets_file,
        output
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid output path"))?,
    )
    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    Ok(Some(output))
}

fn export_spriteatlas(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::{SpriteAtlas, Texture2D};

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    // Try to deserialize to SpriteAtlas struct
    let atlas: SpriteAtlas = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("SpriteAtlas deserialize error: {}", e);
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // Get the SerializedFile from the ObjectReader
    let assets_file = obj
        .assets_file
        .as_ref()
        .and_then(|weak| weak.upgrade())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "SpriteAtlas's ObjectReader has no assets_file reference",
            )
        })?;

    // Export the atlas's packed texture(s) from m_RenderDataMap
    if let Some(render_data_map) = &atlas.m_RenderDataMap {
        if !render_data_map.is_empty() {
            // Get the first texture from the render data map (most atlases have one packed texture)
            let first_atlas_data = &render_data_map[0].1;

            if let Some(ref texture_ptr) = first_atlas_data.texture {
                // Read the texture
                let assets_file_ref = assets_file.borrow();
                let mut texture = texture_ptr.read(&assets_file_ref).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to read atlas texture: {}", e),
                    )
                })?;

                // Set the object_reader so the texture can access StreamData
                if let Some(obj_reader) = assets_file_ref.objects.get(&texture_ptr.m_path_id) {
                    texture.object_reader = Some(Box::new(obj_reader.clone()));
                }
                drop(assets_file_ref);

                // Export as PNG with flip=true (standard vertical flip for Unity textures)
                let output = path.with_extension("png");
                crate::export::texture_2d_converter::save_texture_as_png(&texture, &output, true)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

                return Ok(Some(output));
            }
        }
    }

    // No texture to export
    Ok(None)
}

fn export_material_standalone(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Material;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let material: Material = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("Material deserialize error: {}", e);
            eprintln!(
                "JSON data: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // Export material to MTL format
    let mtl_content = crate::export::mesh_renderer_exporter::export_material(&material)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let output = path.with_extension("mtl");
    let mut file = fs::File::create(&output)?;
    file.write_all(mtl_content.as_bytes())?;

    Ok(Some(output))
}

fn export_monoscript(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::MonoScript;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let script: MonoScript = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("MonoScript deserialize error: {}", e);
            eprintln!(
                "JSON data: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // MonoScript is metadata about a C# script, not the actual source code
    // Export as JSON metadata
    let json = serde_json::to_string_pretty(&script)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let output = path.with_extension("json");
    let mut file = fs::File::create(&output)?;
    file.write_all(json.as_bytes())?;

    Ok(Some(output))
}

fn export_animationclip(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::AnimationClip;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let anim: AnimationClip = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("AnimationClip deserialize error: {}", e);
            eprintln!(
                "JSON data: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // Export AnimationClip as JSON with all curve data
    // This can be imported into other tools or processed further
    let json =
        serde_json::to_string_pretty(&anim).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let output = path.with_extension("json");
    let mut file = fs::File::create(&output)?;
    file.write_all(json.as_bytes())?;

    Ok(Some(output))
}

fn export_videoclip(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::VideoClip;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let video: VideoClip = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("VideoClip deserialize error: {}", e);
            eprintln!(
                "JSON data: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // VideoClip data is usually in external resources
    // Try to get the raw video data
    if let Ok(raw_data) = obj_clone.get_raw_data() {
        if !raw_data.is_empty() {
            // Export as .ogv (Ogg Theora) which is Unity's default video format
            // Or could be .mp4, .webm depending on format
            let output = path.with_extension("ogv");
            let mut file = fs::File::create(&output)?;
            file.write_all(&raw_data)?;
            return Ok(Some(output));
        }
    }

    // If no raw data, export metadata as JSON
    let json = serde_json::to_string_pretty(&video)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let output = path.with_extension("json");
    let mut file = fs::File::create(&output)?;
    file.write_all(json.as_bytes())?;

    Ok(Some(output))
}

fn export_movietexture(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::MovieTexture;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let movie: MovieTexture = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("MovieTexture deserialize error: {}", e);
            eprintln!(
                "JSON data: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // MovieTexture has embedded movie data
    if let Some(ref movie_data) = movie.m_MovieData {
        if !movie_data.is_empty() {
            // Export as .ogv (Ogg Theora) which is Unity's MovieTexture format
            let output = path.with_extension("ogv");
            let mut file = fs::File::create(&output)?;
            file.write_all(movie_data)?;
            return Ok(Some(output));
        }
    }

    Ok(None)
}

fn export_mesh(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Mesh;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    // Try to deserialize to Mesh struct
    let mesh: Mesh = match serde_json::from_value(data.clone()) {
        Ok(m) => m,
        Err(e) => {
            // Deserialization failed - export as JSON instead
            if cfg!(debug_assertions) {
                eprintln!("Mesh deserialize error (exporting as JSON fallback): {}", e);
            }

            let json = serde_json::to_string_pretty(&data)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let output = path.with_extension("json");
            let mut file = fs::File::create(&output)?;
            file.write_all(json.as_bytes())?;
            return Ok(Some(output));
        }
    };

    let obj_data = crate::export::mesh_exporter::export_mesh(&mesh, "obj")
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let output = path.with_extension("obj");
    let mut file = fs::File::create(&output)?;
    file.write_all(obj_data.as_bytes())?;

    Ok(Some(output))
}

fn export_audioclip(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::AudioClip;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    // Try to deserialize to AudioClip struct
    let mut audio: AudioClip = match serde_json::from_value(data.clone()) {
        Ok(a) => a,
        Err(e) => {
            // Deserialization failed - try to extract audio data from JSON
            if cfg!(debug_assertions) {
                eprintln!(
                    "AudioClip deserialize error (attempting raw data extraction): {}",
                    e
                );
            }

            // Try to extract m_AudioData from the raw JSON
            if let Some(audio_data_val) = data.get("m_AudioData") {
                if let Some(bytes) = audio_data_val.as_array() {
                    let audio_bytes: Result<Vec<u8>, _> = bytes
                        .iter()
                        .map(|v| v.as_u64().and_then(|n| u8::try_from(n).ok()))
                        .collect::<Option<Vec<u8>>>()
                        .ok_or_else(|| {
                            io::Error::new(io::ErrorKind::InvalidData, "Invalid audio data")
                        });

                    if let Ok(aud_bytes) = audio_bytes {
                        if !aud_bytes.is_empty() {
                            let output = path.with_extension("fsb");
                            let mut file = fs::File::create(&output)?;
                            file.write_all(&aud_bytes)?;
                            return Ok(Some(output));
                        }
                    }
                }
            }

            // If we can't extract audio data, export as JSON
            let json = serde_json::to_string_pretty(&data)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let output = path.with_extension("json");
            let mut file = fs::File::create(&output)?;
            file.write_all(json.as_bytes())?;
            return Ok(Some(output));
        }
    };

    // Set the object_reader so the audio clip can access resource data
    audio.object_reader = Some(Box::new(obj.clone()));

    let samples = crate::export::audio_clip_converter::extract_audioclip_samples(&mut audio, true)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    if samples.is_empty() {
        return Ok(None);
    }

    if samples.len() == 1 {
        let output = path.with_extension("wav");
        let mut file = fs::File::create(&output)?;
        file.write_all(samples.values().next().unwrap())?;
        Ok(Some(output))
    } else {
        fs::create_dir_all(path)?;
        for (name, data) in &samples {
            let output = path.join(format!("{}.wav", name));
            let mut file = fs::File::create(&output)?;
            file.write_all(data)?;
        }
        Ok(Some(path.to_path_buf()))
    }
}

fn export_shader(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Shader;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
    let shader: Shader =
        serde_json::from_value(data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let shader_text = crate::export::shader_converter::export_shader(&shader)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let output = path.with_extension("txt");
    let mut file = fs::File::create(&output)?;
    file.write_all(shader_text.as_bytes())?;

    Ok(Some(output))
}

fn export_textasset(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::TextAsset;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
    let text: TextAsset =
        serde_json::from_value(data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let script = text.m_Script.as_deref().unwrap_or("");
    if script.is_empty() {
        return Ok(None);
    }

    let output = path.with_extension("txt");
    let mut file = fs::File::create(&output)?;
    file.write_all(script.as_bytes())?;

    Ok(Some(output))
}

fn export_font(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Font;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
    let font: Font =
        serde_json::from_value(data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    if let Some(ref font_data) = font.m_FontData {
        // Convert Vec<i8> to &[u8] for comparison
        let font_data_bytes: Vec<u8> = font_data.iter().map(|&b| b as u8).collect();
        let ext = if font_data_bytes.len() >= 4 && &font_data_bytes[0..4] == b"OTTO" {
            "otf"
        } else {
            "ttf"
        };

        let output = path.with_extension(ext);
        let mut file = fs::File::create(&output)?;
        file.write_all(&font_data_bytes)?;
        return Ok(Some(output));
    }

    Ok(None)
}

fn export_monobehaviour(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    // Try to export as JSON using TypeTree
    let mut obj_clone = obj.clone();
    if let Ok(data) = obj_clone.read_typetree(None, true, false) {
        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let output = path.with_extension("json");
        let mut file = fs::File::create(&output)?;
        file.write_all(json.as_bytes())?;
        return Ok(Some(output));
    }

    // Fall back to raw binary
    if let Ok(raw_data) = obj_clone.get_raw_data() {
        let output = path.with_extension("bin");
        let mut file = fs::File::create(&output)?;
        file.write_all(&raw_data)?;
        return Ok(Some(output));
    }

    Ok(None)
}

fn get_object_name(obj: &crate::files::object_reader::ObjectReader<()>) -> String {
    let mut obj_clone = obj.clone();
    if let Ok(data) = obj_clone.read(false) {
        if let Some(name) = data.get("m_Name").and_then(|v| v.as_str()) {
            if !name.is_empty() {
                return name.to_string();
            }
        }
    }
    format!("{:?}", obj.obj_type)
}

// Re-export for compatibility
pub use extract_assets as export_obj;
