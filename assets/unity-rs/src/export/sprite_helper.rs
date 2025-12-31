/// Sprite extraction and manipulation utilities
///
/// This module provides functionality to extract sprite images from Unity sprite atlases.
/// It handles:
/// - Sprite atlas lookup (direct PPtr or by name tag)
/// - Texture caching for performance
/// - Alpha texture merging
/// - Sprite cropping from atlas
/// - Sprite rotation/flipping for packed sprites
/// - Tight packing support with mesh-based masking
///
/// # Python Equivalent
/// UnityPy/export/SpriteHelper.py (339 lines)
use std::collections::HashMap;

use image::imageops;
use image::{Luma, Rgba, RgbaImage};
use imageproc::point::Point;

use crate::classes::generated::{Sprite, Texture2D};
use crate::classes::pptr::PPtrData;
use crate::enums::class_id_type::ClassIDType;
use crate::enums::sprite_mesh_type::SpriteMeshType;
use crate::enums::sprite_packing_mode::SpritePackingMode;
use crate::enums::sprite_packing_rotation::SpritePackingRotation;
use crate::errors::{UnityError, UnityResult};
use crate::files::serialized_file::SerializedFile;
use crate::helpers::mesh_helper::{MeshHandler, MeshSource};
use crate::helpers::resource_reader::get_resource_data;
use crate::{BuildTarget, TextureFormat};

/// Sprite settings parsed from raw u64 bit field
///
/// The settingsRaw field packs multiple values into a single u64:
/// - Bit 0: packed (1 bit)
/// - Bit 1: packingMode (1 bit)
/// - Bits 2-5: packingRotation (4 bits)
/// - Bit 6: meshType (1 bit)
/// - Bits 7-63: reserved
///
/// # Python Equivalent
/// Python lines 27-39
#[derive(Debug, Clone, Copy)]
pub struct SpriteSettings {
    pub settings_raw: u64,
    pub packed: bool,
    pub packing_mode: SpritePackingMode,
    pub packing_rotation: SpritePackingRotation,
    pub mesh_type: SpriteMeshType,
}

impl SpriteSettings {
    /// Parse sprite settings from raw u64 bit field
    ///
    /// # Python Equivalent
    /// Python lines 33-38
    ///
    /// # Example
    /// ```
    /// use unity_rs::export::sprite_helper::SpriteSettings;
    /// let settings = SpriteSettings::from_raw(0b01101011);
    /// assert_eq!(settings.packed, true);  // Bit 0
    /// ```
    pub fn from_raw(settings_raw: u64) -> Self {
        // Bit 0: packed (1 bit)
        let packed = (settings_raw & 1) != 0;

        // Bit 1: packingMode (1 bit)
        let packing_mode_val = ((settings_raw >> 1) & 1) as u32;
        let packing_mode = SpritePackingMode::from(packing_mode_val);

        // Bits 2-5: packingRotation (4 bits)
        let packing_rotation_val = ((settings_raw >> 2) & 0xF) as u32;
        let packing_rotation = SpritePackingRotation::from(packing_rotation_val);

        // Bit 6: meshType (1 bit)
        let mesh_type_val = ((settings_raw >> 6) & 1) as u32;
        let mesh_type = SpriteMeshType::from(mesh_type_val);

        SpriteSettings {
            settings_raw,
            packed,
            packing_mode,
            packing_rotation,
            mesh_type,
        }
    }
}

/// Helper function to extract an image from a Texture2D object
///
/// This wraps the texture_2d_converter::parse_image_data function to handle
/// extracting all the necessary metadata from the Texture2D object.
///
/// # Arguments
/// * `texture` - The Texture2D object to extract
/// * `flip` - Whether to flip the image vertically
///
/// # Returns
/// The extracted image as RgbaImage
fn get_image_from_texture2d(texture: &Texture2D, flip: bool) -> UnityResult<RgbaImage> {
    let width = texture.m_Width.unwrap_or(0) as u32;
    let height = texture.m_Height.unwrap_or(0) as u32;
    let texture_format = TextureFormat::from(texture.m_TextureFormat.unwrap_or(0) as u32);

    // Get image data - either embedded or from external StreamData
    let image_data = if let Some(data) = texture.image_data.clone().filter(|d| !d.is_empty()) {
        // Embedded image data (non-empty)
        data
    } else if let Some(ref stream_data) = texture.m_StreamData {
        // External image data in .resS resource file
        let obj_reader = texture.object_reader.as_ref().ok_or_else(|| {
            UnityError::Other("No object_reader to fetch stream data".to_string())
        })?;

        let assets_file_weak = obj_reader
            .as_any()
            .downcast_ref::<crate::files::ObjectReader<()>>()
            .and_then(|r| r.assets_file.as_ref())
            .ok_or_else(|| {
                UnityError::Other("Cannot access assets_file for stream data".to_string())
            })?;

        let assets_file_rc = assets_file_weak
            .upgrade()
            .ok_or_else(|| UnityError::Other("SerializedFile was dropped".to_string()))?;

        let mut assets_file_mut = assets_file_rc.borrow_mut();

        let source_path = stream_data
            .path
            .as_ref()
            .ok_or_else(|| UnityError::Other("StreamData has no path".to_string()))?;

        get_resource_data(
            source_path,
            &mut assets_file_mut,
            stream_data.offset.unwrap_or(0) as usize,
            stream_data.size.unwrap_or(0) as usize,
        )?
    } else {
        return Err(UnityError::Other(
            "Texture2D has no image data (neither embedded nor streamed)".to_string(),
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

    crate::export::texture_2d_converter::parse_image_data(
        image_data,
        width,
        height,
        texture_format,
        version,
        platform,
        platform_blob,
        flip,
    )
}

/// Get the full texture image from an atlas, with optional alpha channel merging
///
/// This function caches results to avoid re-decoding the same texture multiple times.
/// If an alpha_texture is provided, it merges the RGB from the main texture with
/// the alpha channel from the alpha texture.
///
/// # Python Equivalent
/// Python lines 42-59
///
/// # Arguments
/// * `texture` - PPtrData to the main texture (RGB or RGBA)
/// * `alpha_texture` - Optional PPtrData to separate alpha texture
/// * `assets_file` - SerializedFile to resolve texture pointers
/// * `cache` - Mutable HashMap for caching decoded textures
///
/// # Returns
/// The full texture image as RgbaImage
pub fn get_image(
    texture: &PPtrData<Texture2D>,
    alpha_texture: &Option<PPtrData<Texture2D>>,
    assets_file: &std::rc::Rc<std::cell::RefCell<SerializedFile>>,
    cache: &mut HashMap<(i64, i64), RgbaImage>,
) -> UnityResult<RgbaImage> {
    // Check for null texture pointer (PathID 0 means null reference in Unity)
    if texture.m_path_id == 0 {
        return Err(UnityError::Other(
            "Texture has null PathID (no texture assigned to sprite)".to_string(),
        ));
    }

    // Determine cache key based on whether we have an alpha texture
    let cache_id = if let Some(ref alpha_tex) = alpha_texture {
        (texture.m_path_id, alpha_tex.m_path_id)
    } else {
        (texture.m_path_id, 0)
    };

    // Check cache first
    if let Some(cached_image) = cache.get(&cache_id) {
        return Ok(cached_image.clone());
    }

    // Decode the main texture
    let mut texture_obj = {
        let assets_file_ref = assets_file.borrow();
        texture.read(&assets_file_ref).map_err(|e| {
            UnityError::Other(format!(
                "Failed to read texture with path_id {}: {}",
                texture.m_path_id, e
            ))
        })?
    };

    // Set the object_reader so the texture can access StreamData
    if let Some(obj_reader) = assets_file.borrow().objects.get(&texture.m_path_id) {
        texture_obj.object_reader = Some(Box::new(obj_reader.clone()));
    }

    let original_image = get_image_from_texture2d(&texture_obj, false)?;

    let result_image = if let Some(alpha_tex) = alpha_texture.as_ref().filter(|a| a.m_path_id != 0)
    {
        // Decode the alpha texture (non-null)
        let mut alpha_texture_obj = {
            let assets_file_ref = assets_file.borrow();
            alpha_tex.read(&assets_file_ref).map_err(|e| {
                UnityError::Other(format!(
                    "Failed to read alpha texture with path_id {}: {}",
                    alpha_tex.m_path_id, e
                ))
            })?
        };

        // Set the object_reader so the texture can access StreamData
        if let Some(obj_reader) = assets_file.borrow().objects.get(&alpha_tex.m_path_id) {
            alpha_texture_obj.object_reader = Some(Box::new(obj_reader.clone()));
        }

        let alpha_image = get_image_from_texture2d(&alpha_texture_obj, false)?;

        // Merge RGB from original with Alpha from alpha_image
        // Python: Image.merge("RGBA", (*original_image.split()[:3], alpha_image.split()[0]))
        merge_rgb_and_alpha(&original_image, &alpha_image)?
    } else {
        original_image
    };

    // Cache the result
    cache.insert(cache_id, result_image.clone());

    Ok(result_image)
}

/// Merge RGB channels from one image with alpha channel from another
///
/// # Python Equivalent
/// Python lines 50-52 (Image.merge)
fn merge_rgb_and_alpha(rgb_image: &RgbaImage, alpha_image: &RgbaImage) -> UnityResult<RgbaImage> {
    let (width, height) = rgb_image.dimensions();
    let (alpha_width, alpha_height) = alpha_image.dimensions();

    if width != alpha_width || height != alpha_height {
        return Err(UnityError::Other(format!(
            "Image dimensions mismatch: RGB {}x{}, Alpha {}x{}",
            width, height, alpha_width, alpha_height
        )));
    }

    let mut result = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let rgb_pixel = rgb_image.get_pixel(x, y);
            let alpha_pixel = alpha_image.get_pixel(x, y);

            // Take RGB from rgb_image, alpha from alpha_image (red channel)
            result.put_pixel(
                x,
                y,
                Rgba([rgb_pixel[0], rgb_pixel[1], rgb_pixel[2], alpha_pixel[0]]),
            );
        }
    }

    Ok(result)
}

/// Extract a sprite image from its atlas texture
///
/// This is the main entry point for sprite extraction. It:
/// 1. Finds the sprite's atlas (via m_SpriteAtlas or m_AtlasTags)
/// 2. Gets the sprite_atlas_data (texture, rect, settings)
/// 3. Decodes the atlas texture (with caching)
/// 4. Crops to the sprite's rectangle
/// 5. Applies rotation/flipping if packed
/// 6. Handles tight packing with mesh if needed
/// 7. Flips vertically (Unity → standard image coordinates)
///
/// # Python Equivalent
/// Python lines 62-125
///
/// # Arguments
/// * `sprite` - The Sprite object to extract
/// * `assets_file` - Rc<RefCell<SerializedFile>> to resolve pointers (allows borrowing as needed)
/// * `cache` - Mutable HashMap for caching decoded textures
///
/// # Returns
/// The extracted sprite image as RgbaImage
pub fn get_image_from_sprite(
    sprite: &Sprite,
    assets_file: &std::rc::Rc<std::cell::RefCell<SerializedFile>>,
    cache: &mut HashMap<(i64, i64), RgbaImage>,
) -> UnityResult<RgbaImage> {
    // Step 1: Find the atlas
    // Python lines 63-73
    let atlas = if let Some(sprite_atlas_ptr) =
        sprite.m_SpriteAtlas.as_ref().filter(|p| p.m_path_id != 0)
    {
        // Direct atlas reference (only if PathID is not 0/null)
        let assets_file_ref = assets_file.borrow();
        sprite_atlas_ptr
            .try_read(&assets_file_ref)
            .map_err(|e| UnityError::Other(format!("Failed to read sprite atlas: {}", e)))?
    } else if let Some(ref atlas_tags) = sprite.m_AtlasTags {
        // Search for atlas by name
        // Python lines 66-73
        if atlas_tags.is_empty() {
            None
        } else {
            // Iterate through all objects to find matching SpriteAtlas
            // Need to search across all SerializedFiles in the environment
            let mut found_atlas = None;

            // Get list of all SerializedFiles to search through
            let serialized_files: Vec<std::rc::Rc<std::cell::RefCell<SerializedFile>>> = {
                let assets_file_ref = assets_file.borrow();
                if let Some(env_rc) = &assets_file_ref.environment {
                    let env_ref = env_rc.borrow();
                    let mut all_files = Vec::new();
                    for file_rc in env_ref.files.values() {
                        let file_ref = file_rc.borrow();
                        match &*file_ref {
                            crate::files::bundle_file::FileType::SerializedFile(sf_rc) => {
                                all_files.push(std::rc::Rc::clone(sf_rc));
                            }
                            crate::files::bundle_file::FileType::BundleFile(bundle) => {
                                // Collect ALL SerializedFiles inside bundles (not just the first one)
                                for bf_rc in bundle.files.values() {
                                    let bf_ref = bf_rc.borrow();
                                    if let crate::files::bundle_file::FileType::SerializedFile(
                                        sf_rc,
                                    ) = &*bf_ref
                                    {
                                        all_files.push(std::rc::Rc::clone(sf_rc));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    all_files
                } else {
                    // No environment, just search current file
                    vec![std::rc::Rc::clone(assets_file)]
                }
            };

            // Search through all SerializedFiles
            for sf_rc in &serialized_files {
                let sf_ref = sf_rc.borrow();
                for obj in sf_ref.objects.values() {
                    // Check for SpriteAtlas by ClassID numeric value (65007 or 687078895) since Unity uses different IDs for different versions
                    let is_sprite_atlas =
                        obj.obj_type == ClassIDType::SpriteAtlas || obj.class_id as u32 == 65007;
                    if is_sprite_atlas {
                        // Try to read the atlas
                        if let Ok(atlas_value) = obj.clone().parse_as_object(None, true) {
                            if let Ok(atlas) = serde_json::from_value::<
                                crate::classes::generated::SpriteAtlas,
                            >(atlas_value)
                            {
                                // Case-insensitive comparison since Unity sometimes uses different casing
                                let names_match = atlas
                                    .m_Name
                                    .as_ref()
                                    .map(|name| name.eq_ignore_ascii_case(&atlas_tags[0]))
                                    .unwrap_or(false);

                                if names_match {
                                    found_atlas = Some(atlas);
                                    break;
                                }
                            }
                        }
                    }
                }
                if found_atlas.is_some() {
                    break;
                }
            }
            found_atlas
        }
    } else {
        None
    };

    // Step 2: Get sprite render data (either from atlas or sprite itself)
    // Python lines 75-82
    let (texture_clone, alpha_texture_clone, texture_rect_clone, settings_raw) =
        if let Some(atlas) = atlas {
            // Find the data in atlas.m_RenderDataMap matching sprite.m_RenderDataKey
            // Python lines 76-80
            let render_data_key = sprite.m_RenderDataKey.as_ref().ok_or_else(|| {
                UnityError::Other("Sprite has atlas but no render data key".to_string())
            })?;

            // Compare GUID and i64 separately since GUID doesn't implement PartialEq
            let render_data_map = atlas
                .m_RenderDataMap
                .as_ref()
                .ok_or_else(|| UnityError::Other("Atlas has no render data map".to_string()))?;

            let found_data = render_data_map
                .iter()
                .find(|(key, _value)| {
                    key.0.data_0_ == render_data_key.0.data_0_
                        && key.0.data_1_ == render_data_key.0.data_1_
                        && key.0.data_2_ == render_data_key.0.data_2_
                        && key.0.data_3_ == render_data_key.0.data_3_
                        && key.1 == render_data_key.1
                })
                .map(|(_key, value)| value);

            let atlas_data = found_data.ok_or_else(|| {
                UnityError::Other(format!(
                    "Sprite render data not found in atlas for key: {:?}",
                    render_data_key
                ))
            })?;

            // Extract and clone fields from SpriteAtlasData to avoid lifetime issues
            // Note: alphaTexture is Optional in SpriteAtlasData
            (
                atlas_data.texture.clone(),
                atlas_data.alphaTexture.clone(),
                atlas_data.textureRect.clone(),
                atlas_data.settingsRaw.unwrap_or(0) as u64,
            )
        } else {
            // Use sprite's own render data (SpriteRenderData)
            // Python line 82
            let rd = sprite.m_RD.as_ref().ok_or_else(|| {
                crate::errors::UnityError::Other("Sprite has no m_RD".to_string())
            })?;
            (
                rd.texture.clone(),
                rd.alphaTexture.clone(),
                rd.textureRect.clone(),
                rd.settingsRaw.unwrap_or(0) as u64,
            )
        };

    let m_texture2d = texture_clone
        .as_ref()
        .ok_or_else(|| UnityError::Other("Sprite has no texture".to_string()))?;
    let alpha_texture = &alpha_texture_clone;
    let texture_rect = &texture_rect_clone;

    // Step 3: Get the full atlas image (with caching)
    // Python line 89
    let original_image = get_image(m_texture2d, alpha_texture, assets_file, cache)?;

    // Step 4: Crop to sprite's rectangle
    // Python lines 91-96
    let rect = texture_rect
        .as_ref()
        .ok_or_else(|| UnityError::Other("Sprite has no texture rect".to_string()))?;
    let sprite_image = imageops::crop_imm(
        &original_image,
        rect.x.unwrap_or(0.0) as u32,
        rect.y.unwrap_or(0.0) as u32,
        rect.width.unwrap_or(0.0) as u32,
        rect.height.unwrap_or(0.0) as u32,
    )
    .to_image();

    // Step 5: Parse settings and apply rotation if packed
    // Python lines 98-112
    let settings = SpriteSettings::from_raw(settings_raw);
    let mut sprite_image = sprite_image;

    if settings.packed {
        sprite_image = apply_packing_rotation(sprite_image, settings.packing_rotation)?;
    }

    // Step 6: Handle tight packing mode
    // Python lines 114-123
    if settings.packing_mode == SpritePackingMode::kSPMTight {
        // Get version from SerializedFile
        let version = Some(assets_file.borrow().version);

        // Create and process mesh handler
        let sprite_rd = sprite
            .m_RD
            .clone()
            .ok_or_else(|| UnityError::Other("Sprite has no m_RD for mesh handler".to_string()))?;
        let mut mesh = MeshHandler::new(
            MeshSource::Sprite(sprite_rd),
            version,
            None, // endianess
        )
        .map_err(|e| UnityError::Other(format!("Failed to create mesh handler: {}", e)))?;

        mesh.process()
            .map_err(|e| UnityError::Other(format!("Failed to process mesh: {}", e)))?;

        // Check if we have UV data to decide between mask_sprite and render_sprite_mesh
        // Python line 118: if any(u or v for u, v in mesh.m_UV0)
        let has_uv_data = if let Some(ref uv0) = mesh.m_uv0 {
            uv0.iter().any(|(u, v)| *u != 0.0 || *v != 0.0)
        } else {
            false
        };

        sprite_image = if has_uv_data {
            // Copy triangles from mesh (Python line 120)
            render_sprite_mesh(sprite, &mesh, &original_image)?
        } else {
            // Create mask to keep only the polygon (Python line 123)
            mask_sprite(sprite, &mesh, sprite_image)?
        };
    }

    // Step 7: Flip vertically (Unity uses bottom-left origin, images use top-left)
    // Python line 125
    let sprite_image = imageops::flip_vertical(&sprite_image);

    Ok(sprite_image)
}

/// Apply sprite packing rotation/flipping
///
/// # Python Equivalent
/// Python lines 100-112
fn apply_packing_rotation(
    image: RgbaImage,
    rotation: SpritePackingRotation,
) -> UnityResult<RgbaImage> {
    let result = match rotation {
        SpritePackingRotation::kSPRNone => image,
        SpritePackingRotation::kSPRFlipHorizontal => {
            // Python line 102: Image.FLIP_LEFT_RIGHT
            imageops::flip_horizontal(&image)
        }
        SpritePackingRotation::kSPRFlipVertical => {
            // Python line 105: Image.FLIP_TOP_BOTTOM
            imageops::flip_vertical(&image)
        }
        SpritePackingRotation::kSPRRotate180 => {
            // Python line 108: Image.ROTATE_180
            imageops::rotate180(&image)
        }
        SpritePackingRotation::kSPRRotate90 => {
            // Python line 111: Image.ROTATE_270 (note: Unity's 90 = PIL's 270)
            imageops::rotate270(&image)
        }
        SpritePackingRotation::Unknown(_) => {
            return Err(UnityError::Other(format!(
                "Unsupported sprite packing rotation: {:?}",
                rotation
            )))
        }
    };

    Ok(result)
}

/// Convenience function to save a sprite directly to a PNG file
///
/// # Arguments
/// * `sprite` - The Sprite object to extract
/// * `assets_file` - Rc<RefCell<SerializedFile>> to resolve pointers (allows borrowing as needed)
/// * `output_path` - Path to save the PNG file
///
/// # Returns
/// Ok(()) on success, error otherwise
pub fn save_sprite(
    sprite: &Sprite,
    assets_file: &std::rc::Rc<std::cell::RefCell<SerializedFile>>,
    output_path: &str,
) -> UnityResult<()> {
    let mut cache = HashMap::new();
    let sprite_image = get_image_from_sprite(sprite, assets_file, &mut cache)?;

    sprite_image
        .save(output_path)
        .map_err(|e| UnityError::Other(format!("Failed to save sprite PNG: {}", e)))?;

    Ok(())
}

/// Create a polygon mask for a sprite using mesh triangles
///
/// # Python Equivalent
/// Python lines 128-169
///
/// # Arguments
/// * `sprite` - The Sprite object
/// * `mesh` - The processed MeshHandler with vertex and triangle data
/// * `sprite_image` - The cropped sprite image to mask
///
/// # Returns
/// The masked sprite image
fn mask_sprite(
    sprite: &Sprite,
    mesh: &MeshHandler,
    sprite_image: RgbaImage,
) -> UnityResult<RgbaImage> {
    let (width, height) = sprite_image.dimensions();
    let mut mask_img = image::GrayImage::new(width, height);

    // Get vertex positions from mesh
    let positions = mesh
        .m_vertices
        .as_ref()
        .ok_or_else(|| UnityError::Other("Mesh has no vertex positions".to_string()))?;

    // Normalize the points (Python lines 137-143)
    // Shift the whole point matrix into positive space
    // Multiply them with a factor to scale them to the image
    let min_x = positions.iter().map(|v| v.0).fold(f32::INFINITY, f32::min);
    let min_y = positions.iter().map(|v| v.1).fold(f32::INFINITY, f32::min);
    let factor = sprite.m_PixelsToUnits.unwrap_or(100.0);

    let positions_2d: Vec<(f32, f32)> = positions
        .iter()
        .map(|(x, y, _z)| ((x - min_x) * factor, (y - min_y) * factor))
        .collect();

    // Generate triangles from the given points (Python lines 146-154)
    let triangles_result = mesh.get_triangles();
    if let Ok(submeshes) = triangles_result {
        for submesh in submeshes {
            for triangle_indices in submesh {
                if triangle_indices.len() >= 3 {
                    let tri_points: Vec<Point<i32>> = triangle_indices[0..3]
                        .iter()
                        .map(|&idx| {
                            let (x, y) = positions_2d[idx as usize];
                            Point::new(x as i32, y as i32)
                        })
                        .collect();

                    // Draw filled triangle on mask (Python line 157)
                    draw_filled_triangle(&mut mask_img, &tri_points, Luma([255]));
                }
            }
        }
    }

    // Apply the mask (Python lines 160-167)
    let mut result = RgbaImage::new(width, height);

    // RgbaImage always has alpha channel, so apply as composite
    for y in 0..height {
        for x in 0..width {
            if mask_img.get_pixel(x, y)[0] > 0 {
                result.put_pixel(x, y, *sprite_image.get_pixel(x, y));
            } else {
                result.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            }
        }
    }

    Ok(result)
}

/// Render a sprite using mesh UV mapping
///
/// # Python Equivalent
/// Python lines 172-220
///
/// # Arguments
/// * `sprite` - The Sprite object
/// * `mesh` - The processed MeshHandler with vertex and UV data
/// * `texture` - The full texture atlas image
///
/// # Returns
/// The rendered sprite image
fn render_sprite_mesh(
    sprite: &Sprite,
    mesh: &MeshHandler,
    texture: &RgbaImage,
) -> UnityResult<RgbaImage> {
    let triangles_result = mesh.get_triangles()?;

    let positions = mesh
        .m_vertices
        .as_ref()
        .ok_or_else(|| UnityError::Other("Mesh has no vertex positions".to_string()))?;

    let uv = mesh
        .m_uv0
        .as_ref()
        .ok_or_else(|| UnityError::Other("Mesh has no UV coordinates".to_string()))?;

    // Patch position data (Python lines 179-193)
    // Make positions 2D - find the axis that has only one value and can be removed
    // Usually the z axis
    let mut axis_values: Vec<Vec<f32>> = vec![
        positions.iter().map(|v| v.0).collect(),
        positions.iter().map(|v| v.1).collect(),
        positions.iter().map(|v| v.2).collect(),
    ];

    // Find constant axis (Python lines 184-188)
    let mut removed_axis = None;
    for i in (0..3).rev() {
        let vals = &axis_values[i];
        let first = vals[0];
        if vals.iter().all(|&v| (v - first).abs() < 0.001) {
            removed_axis = Some(i);
            break;
        }
    }

    if removed_axis.is_none() {
        return Err(UnityError::Other("Can't process 3d sprites!".to_string()));
    }

    let removed_idx = removed_axis.unwrap();
    axis_values.remove(removed_idx);

    let x_min = axis_values[0].iter().copied().fold(f32::INFINITY, f32::min);
    let y_min = axis_values[1].iter().copied().fold(f32::INFINITY, f32::min);
    let x_max = axis_values[0]
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let y_max = axis_values[1]
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);

    // Map positions from middle to top left and convert to absolute (Python lines 198-201)
    let pixels_to_units = sprite.m_PixelsToUnits.unwrap_or(100.0);
    let positions_abs: Vec<(f32, f32)> = (0..positions.len())
        .map(|i| {
            let x = axis_values[0][i];
            let y = axis_values[1][i];
            (
                ((x - x_min) * pixels_to_units).round(),
                ((y - y_min) * pixels_to_units).round(),
            )
        })
        .collect();

    let (tex_width, tex_height) = texture.dimensions();
    let uv_abs: Vec<(f32, f32)> = uv
        .iter()
        .map(|(u, v)| {
            (
                (u * tex_width as f32).round(),
                (v * tex_height as f32).round(),
            )
        })
        .collect();

    // Generate final image size (Python lines 206-210)
    let size = (
        ((x_max - x_min) * pixels_to_units).round() as u32,
        ((y_max - y_min) * pixels_to_units).round() as u32,
    );

    let mut sprite_img = RgbaImage::new(size.0, size.1);

    // Render each triangle (Python lines 212-218)
    for submesh in triangles_result {
        for triangle_indices in submesh {
            if triangle_indices.len() >= 3 {
                let src_tri: [(f32, f32); 3] = [
                    uv_abs[triangle_indices[0] as usize],
                    uv_abs[triangle_indices[1] as usize],
                    uv_abs[triangle_indices[2] as usize],
                ];

                let dst_tri: [(f32, f32); 3] = [
                    positions_abs[triangle_indices[0] as usize],
                    positions_abs[triangle_indices[1] as usize],
                    positions_abs[triangle_indices[2] as usize],
                ];

                copy_triangle(texture, &src_tri, &mut sprite_img, &dst_tri);
            }
        }
    }

    Ok(sprite_img)
}

/// Copy a triangular region from source image to destination image
///
/// Handles both direct copy (when triangles are same size/orientation) and
/// affine transformation (when triangles differ).
///
/// # Python Equivalent
/// Python lines 223-285
///
/// # Arguments
/// * `src_img` - Source image
/// * `src_tri` - Triangle vertices in source image [(x1,y1), (x2,y2), (x3,y3)]
/// * `dst_img` - Destination image (modified in place)
/// * `dst_tri` - Triangle vertices in destination image
fn copy_triangle(
    src_img: &RgbaImage,
    src_tri: &[(f32, f32); 3],
    dst_img: &mut RgbaImage,
    dst_tri: &[(f32, f32); 3],
) {
    // Calculate offsets between triangle points
    let src_off = [
        (src_tri[1].0 - src_tri[0].0, src_tri[1].1 - src_tri[0].1),
        (src_tri[2].0 - src_tri[0].0, src_tri[2].1 - src_tri[0].1),
    ];
    let dst_off = [
        (dst_tri[1].0 - dst_tri[0].0, dst_tri[1].1 - dst_tri[0].1),
        (dst_tri[2].0 - dst_tri[0].0, dst_tri[2].1 - dst_tri[0].1),
    ];

    // Check if transform is necessary by comparing triangle sizes
    // Python lines 239-253
    if (src_off[0].0 - dst_off[0].0).abs() < 0.01
        && (src_off[0].1 - dst_off[0].1).abs() < 0.01
        && (src_off[1].0 - dst_off[1].0).abs() < 0.01
        && (src_off[1].1 - dst_off[1].1).abs() < 0.01
    {
        // No transform necessary, just copy the triangle
        // Make rectangle that contains the triangle
        let mut min_x = src_tri[0].0.min(src_tri[1].0.min(src_tri[2].0));
        let mut min_y = src_tri[0].1.min(src_tri[1].1.min(src_tri[2].1));
        let mut max_x = src_tri[0].0.max(src_tri[1].0.max(src_tri[2].0));
        let mut max_y = src_tri[0].1.max(src_tri[1].1.max(src_tri[2].1));

        // Clamp to image bounds
        min_x = min_x.max(0.0);
        min_y = min_y.max(0.0);
        max_x = max_x.min(src_img.width() as f32 - 1.0);
        max_y = max_y.min(src_img.height() as f32 - 1.0);

        let width = (max_x - min_x + 1.0) as u32;
        let height = (max_y - min_y + 1.0) as u32;

        if width == 0 || height == 0 {
            return;
        }

        // Create a sub-image and mask
        let mut src_part = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let src_x = (min_x as u32 + x).min(src_img.width() - 1);
                let src_y = (min_y as u32 + y).min(src_img.height() - 1);
                src_part.put_pixel(x, y, *src_img.get_pixel(src_x, src_y));
            }
        }

        // Create mask for triangle using imageproc
        let mut mask = image::GrayImage::new(width, height);
        let mask_points: Vec<Point<i32>> = src_tri
            .iter()
            .map(|(x, y)| Point::new((x - min_x) as i32, (y - min_y) as i32))
            .collect();

        // Fill polygon - we need to do this manually as imageproc doesn't have filled polygon
        draw_filled_triangle(&mut mask, &mask_points, Luma([255]));

        // Paste triangle into destination image using mask
        let dst_min_x = dst_tri[0].0.min(dst_tri[1].0.min(dst_tri[2].0)) as u32;
        let dst_min_y = dst_tri[0].1.min(dst_tri[1].1.min(dst_tri[2].1)) as u32;

        for y in 0..height {
            for x in 0..width {
                if mask.get_pixel(x, y)[0] > 0 {
                    let dst_x = dst_min_x + x;
                    let dst_y = dst_min_y + y;
                    if dst_x < dst_img.width() && dst_y < dst_img.height() {
                        dst_img.put_pixel(dst_x, dst_y, *src_part.get_pixel(x, y));
                    }
                }
            }
        }
    } else {
        // Transform is necessary, use affine transformation
        // Python lines 255-285
        let ((x11, x12), (x21, x22), (x31, x32)) = (src_tri[0], src_tri[1], src_tri[2]);
        let ((y11, y12), (y21, y22), (y31, y32)) = (dst_tri[0], dst_tri[1], dst_tri[2]);

        // Construct matrix M manually (Python lines 261-268)
        let m = vec![
            vec![y11 as f64, y12 as f64, 1.0, 0.0, 0.0, 0.0],
            vec![y21 as f64, y22 as f64, 1.0, 0.0, 0.0, 0.0],
            vec![y31 as f64, y32 as f64, 1.0, 0.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0, y11 as f64, y12 as f64, 1.0],
            vec![0.0, 0.0, 0.0, y21 as f64, y22 as f64, 1.0],
            vec![0.0, 0.0, 0.0, y31 as f64, y32 as f64, 1.0],
        ];

        // Vector y corresponds to x coordinates in source triangle (Python line 271)
        let y = vec![
            x11 as f64, x21 as f64, x31 as f64, x12 as f64, x22 as f64, x32 as f64,
        ];

        // Solve for affine transform coefficients
        let a = linalg_solve(m, y);

        // Apply affine transformation to each pixel in destination triangle
        // Create mask for destination triangle
        let mut mask = image::GrayImage::new(dst_img.width(), dst_img.height());
        let dst_points: Vec<Point<i32>> = dst_tri
            .iter()
            .map(|(x, y)| Point::new(*x as i32, *y as i32))
            .collect();
        draw_filled_triangle(&mut mask, &dst_points, Luma([255]));

        // For each pixel in the mask, apply inverse affine transform and sample from source
        for dy in 0..dst_img.height() {
            for dx in 0..dst_img.width() {
                if mask.get_pixel(dx, dy)[0] > 0 {
                    // Apply affine transform: src_x = a[0]*dst_x + a[1]*dst_y + a[2]
                    let src_x = a[0] * dx as f64 + a[1] * dy as f64 + a[2];
                    let src_y = a[3] * dx as f64 + a[4] * dy as f64 + a[5];

                    // Bounds check and sample
                    if src_x >= 0.0
                        && src_x < src_img.width() as f64
                        && src_y >= 0.0
                        && src_y < src_img.height() as f64
                    {
                        let pixel = src_img.get_pixel(src_x as u32, src_y as u32);
                        dst_img.put_pixel(dx, dy, *pixel);
                    }
                }
            }
        }
    }
}

/// Helper function to draw a filled triangle
fn draw_filled_triangle(img: &mut image::GrayImage, points: &[Point<i32>], color: Luma<u8>) {
    if points.len() != 3 {
        return;
    }

    // Simple scanline fill algorithm
    let mut min_y = points[0].y.min(points[1].y).min(points[2].y);
    let mut max_y = points[0].y.max(points[1].y).max(points[2].y);

    // Clamp to image bounds
    min_y = min_y.max(0);
    max_y = max_y.min(img.height() as i32 - 1);

    for y in min_y..=max_y {
        let mut x_intersections = Vec::new();

        // Check each edge
        for i in 0..3 {
            let p1 = points[i];
            let p2 = points[(i + 1) % 3];

            if (p1.y <= y && p2.y > y) || (p2.y <= y && p1.y > y) {
                let t = (y - p1.y) as f32 / (p2.y - p1.y) as f32;
                let x = p1.x as f32 + t * (p2.x - p1.x) as f32;
                x_intersections.push(x as i32);
            }
        }

        x_intersections.sort();

        // Fill between pairs of intersections
        for i in (0..x_intersections.len()).step_by(2) {
            if i + 1 < x_intersections.len() {
                let x_start = x_intersections[i].max(0);
                let x_end = x_intersections[i + 1].min(img.width() as i32 - 1);

                for x in x_start..=x_end {
                    img.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    }
}

/// Solve linear equation Ax = b using matrix inversion
///
/// # Python Equivalent
/// Python lines 288-291
fn linalg_solve(m: Vec<Vec<f64>>, y: Vec<f64>) -> Vec<f64> {
    // M^-1 * y
    let m_inv = get_matrix_inverse(m);
    let len = m_inv.len();
    (0..len)
        .map(|i| (0..y.len()).map(|j| m_inv[i][j] * y[j]).sum())
        .collect()
}

/// Transpose a matrix
///
/// # Python Equivalent
/// Python lines 294-296
fn transpose_matrix(m: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    if m.is_empty() {
        return vec![];
    }
    let rows = m.len();
    let cols = m[0].len();
    (0..cols)
        .map(|col| (0..rows).map(|row| m[row][col]).collect())
        .collect()
}

/// Get the minor of a matrix (submatrix with row i and column j removed)
///
/// # Python Equivalent
/// Python lines 299-301
fn get_matrix_minor(m: &[Vec<f64>], i: usize, j: usize) -> Vec<Vec<f64>> {
    m.iter()
        .enumerate()
        .filter(|(row_idx, _)| *row_idx != i)
        .map(|(_, row)| {
            row.iter()
                .enumerate()
                .filter(|(col_idx, _)| *col_idx != j)
                .map(|(_, &val)| val)
                .collect()
        })
        .collect()
}

/// Calculate the determinant of a matrix using recursive expansion
///
/// # Python Equivalent
/// Python lines 304-313
fn get_matrix_determinant(m: &[Vec<f64>]) -> f64 {
    // Base case for 2x2 matrix
    if m.len() == 2 {
        return m[0][0] * m[1][1] - m[0][1] * m[1][0];
    }

    // Recursive expansion along first row
    (0..m.len())
        .map(|c| {
            let sign = if c % 2 == 0 { 1.0 } else { -1.0 };
            sign * m[0][c] * get_matrix_determinant(&get_matrix_minor(m, 0, c))
        })
        .sum()
}

/// Calculate the inverse of a matrix using cofactor method
///
/// # Python Equivalent
/// Python lines 316-336
fn get_matrix_inverse(m: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let determinant = get_matrix_determinant(&m);

    // Special case for 2x2 matrix
    if m.len() == 2 {
        return vec![
            vec![m[1][1] / determinant, -m[0][1] / determinant],
            vec![-m[1][0] / determinant, m[0][0] / determinant],
        ];
    }

    // Find matrix of cofactors
    let cofactors: Vec<Vec<f64>> = (0..m.len())
        .map(|r| {
            (0..m.len())
                .map(|c| {
                    let sign = if (r + c) % 2 == 0 { 1.0 } else { -1.0 };
                    sign * get_matrix_determinant(&get_matrix_minor(&m, r, c))
                })
                .collect()
        })
        .collect();

    // Transpose cofactors and divide by determinant
    let cofactors_t = transpose_matrix(cofactors);
    cofactors_t
        .into_iter()
        .map(|row| row.into_iter().map(|c| c / determinant).collect())
        .collect()
}
