//! Texture format conversion and image extraction
//!
//! This module handles conversion of Unity Texture2D objects to standard image formats.
//! It supports 60+ texture formats including compressed formats (DXT/BCn, ASTC, ETC, PVRTC)
//! and raw formats (RGBA32, RGB24, etc.).
//!
//! # Architecture
//!
//! 1. `get_image_from_texture2d()` - Main entry point, extracts data from Texture2D
//! 2. `parse_image_data()` - Core conversion logic with format detection
//! 3. Format-specific decoders (decode_rgba32, decode_dxt1, etc.)
//! 4. Platform-specific handlers (Switch deswizzle, Xbox byte swap)
//!
//! # Python Equivalent
//!
//! Python: UnityPy/export/Texture2DConverter.py (626 lines)
//! Rust: This file

use crate::errors::UnityResult;
use crate::generated::Texture2D; // Re-exported from crate::classes::generated
use crate::helpers::resource_reader::get_resource_data;
use crate::helpers::texture_swizzler;
use crate::{BuildTarget, TextureFormat, UnityError}; // Re-exported from crate::enums::*
use bcdec_rs;
use half::f16;
use image::imageops;
use image::{ImageBuffer, RgbaImage};
use std::path::Path;
use texpresso::Format;

/// Main entry point: Convert a Texture2D to an image and save as PNG
///
/// # Arguments
///
/// * `texture` - The Texture2D object to convert
/// * `output_path` - Where to save the PNG file
/// * `flip` - Whether to flip the image vertically (Unity textures are upside down by default)
///
/// # Python Equivalent
///
/// ```python
/// def get_image_from_texture2d(texture_2d: Texture2D, flip: bool = True) -> Image.Image
/// ```
///
/// Python lines 268-290
pub fn save_texture_as_png(texture: &Texture2D, output_path: &Path, flip: bool) -> UnityResult<()> {
    // Extract metadata
    let width = texture.m_Width.unwrap_or(0) as u32;
    let height = texture.m_Height.unwrap_or(0) as u32;

    // Convert m_TextureFormat (i32) to TextureFormat enum
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
            &mut *assets_file_mut,
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
            // Default if no assets_file
            ((0, 0, 0, 0), BuildTarget::UnknownPlatform)
        }
    } else {
        // Default if no object_reader
        ((0, 0, 0, 0), BuildTarget::UnknownPlatform)
    };

    // Parse image data
    let rgba_image = parse_image_data(
        image_data,
        width,
        height,
        texture_format,
        version,
        platform,
        platform_blob,
        flip,
    )?;

    // Save as PNG
    rgba_image
        .save(output_path)
        .map_err(|e| UnityError::Other(format!("Failed to save PNG: {}", e)))?;

    Ok(())
}

/// Core conversion logic: Parse raw texture data into an RgbaImage
///
/// # Arguments
///
/// * `image_data` - Raw texture bytes
/// * `width` - Texture width in pixels
/// * `height` - Texture height in pixels
/// * `texture_format` - The TextureFormat enum value
/// * `version` - Unity version tuple (major, minor, patch, build)
/// * `platform` - Build target platform (for platform-specific handling)
/// * `flip` - Whether to flip the image vertically
///
/// # Python Equivalent
///
/// ```python
/// def parse_image_data(
/// image_data: bytes,
/// width: int,
/// height: int,
/// texture_format: Union[int, TextureFormat],
/// version: Tuple[int, int, int, int],
/// platform: int,
/// platform_blob: Optional[bytes] = None,
/// flip: bool = True,
/// ) -> Image.Image
/// ```
///
/// Python lines 293-363
pub fn parse_image_data(
    image_data: Vec<u8>,
    width: u32,
    height: u32,
    texture_format: TextureFormat,
    version: (u32, u32, u32, u32),
    platform: BuildTarget,
    platform_blob: Option<Vec<u8>>,
    flip: bool,
) -> UnityResult<RgbaImage> {
    // Validation (Python lines 303-308)
    if width == 0 || height == 0 {
        return Ok(ImageBuffer::new(0, 0));
    }

    if image_data.is_empty() {
        return Err(crate::errors::UnityError::Other(
            "Texture2D has no image data".to_string(),
        ));
    }

    let mut working_data = image_data;
    let mut working_width = width;
    let mut working_height = height;

    let mut texture_format = texture_format;

    // Xbox 360 byte swapping (Python lines 313-314)
    if platform == BuildTarget::XBOX360 {
        // Formats that need byte swapping on Xbox360
        const XBOX_SWAP_FORMATS: &[TextureFormat] = &[
            TextureFormat::RGB565,
            TextureFormat::DXT1,
            TextureFormat::DXT5,
            TextureFormat::DXT1Crunched,
            TextureFormat::DXT5Crunched,
        ];

        if XBOX_SWAP_FORMATS.contains(&texture_format) {
            swap_bytes_for_xbox(&mut working_data);
        }
    }

    // Handle Crunched formats (Python lines 337-347)
    // Crunched formats decode directly to RGBA using decode_crunch()
    if matches!(
        texture_format,
        TextureFormat::DXT1Crunched
            | TextureFormat::DXT5Crunched
            | TextureFormat::ETC_RGB4Crunched
            | TextureFormat::ETC2_RGBA8Crunched
    ) {
        let decoded = decode_crunch(&working_data, width, height, version)?;
        // Apply flip if requested
        return Ok(if flip {
            image::imageops::flip_vertical(&decoded)
        } else {
            decoded
        });
    }

    // Store original dimensions for Switch cropping (Python line 316)
    let original_width = width;
    let original_height = height;

    // Nintendo Switch deswizzling preprocessing (Python lines 318-328)
    let mut switch_swizzle_params: Option<((usize, usize), usize)> = None;
    if platform == BuildTarget::Switch {
        if let Some(ref blob) = platform_blob {
            // Get GOBs per block from platform blob
            let gobs_per_block = texture_swizzler::get_switch_gobs_per_block(blob);

            // Python lines 320-323: Handle RGB24/BGR24 format conversion
            if texture_format == TextureFormat::RGB24 {
                texture_format = TextureFormat::RGBA32;
            } else if texture_format == TextureFormat::BGR24 {
                texture_format = TextureFormat::BGRA32;
            }

            // Get block size for the current format
            if let Some((block_width, block_height)) =
                texture_swizzler::get_texture_format_block_size(texture_format)
            {
                // Calculate padded dimensions for Switch
                let (padded_width, padded_height) = texture_swizzler::get_padded_texture_size(
                    width as usize,
                    height as usize,
                    block_width,
                    block_height,
                    gobs_per_block,
                );

                // Update working dimensions
                working_width = padded_width as u32;
                working_height = padded_height as u32;

                // Store parameters for post-processing
                switch_swizzle_params = Some(((block_width, block_height), gobs_per_block));
            }
        }
    }

    // Note: Width/height padding for compressed formats is handled
    // by individual decoder functions (e.g., decode_dxt1, decode_bc4).
    // Each decoder calls get_padded_dimensions() internally.
    // See Python's get_compressed_image_size() - Python lines 36-50

    // Format conversion dispatch (Python line 349)
    let decoded_image = match texture_format {
        // Simple raw formats
        TextureFormat::RGBA32 => decode_rgba32(&working_data, working_width, working_height)?,
        TextureFormat::RGB24 => decode_rgb24(&working_data, working_width, working_height)?,
        TextureFormat::ARGB32 => decode_argb32(&working_data, working_width, working_height)?,
        TextureFormat::BGRA32 => decode_bgra32(&working_data, working_width, working_height)?,
        TextureFormat::BGR24 => decode_bgr24(&working_data, working_width, working_height)?,
        TextureFormat::Alpha8 => decode_alpha8(&working_data, working_width, working_height)?,
        TextureFormat::R8 => decode_r8(&working_data, working_width, working_height)?,
        TextureFormat::R16 => decode_r16(&working_data, working_width, working_height)?,
        TextureFormat::RHalf => decode_rhalf(&working_data, working_width, working_height)?,
        TextureFormat::RGHalf => decode_rghalf(&working_data, working_width, working_height)?,
        TextureFormat::RGBAHalf => decode_rgbahalf(&working_data, working_width, working_height)?,
        TextureFormat::RFloat => decode_rfloat(&working_data, working_width, working_height)?,
        TextureFormat::RGFloat => decode_rgfloat(&working_data, working_width, working_height)?,
        TextureFormat::RGBAFloat => decode_rgbafloat(&working_data, working_width, working_height)?,
        TextureFormat::ARGBFloat => decode_argbfloat(&working_data, working_width, working_height)?,
        TextureFormat::RGB9e5Float => {
            decode_rgb9e5_float(&working_data, working_width, working_height)?
        }
        TextureFormat::RG16 => decode_rg16(&working_data, working_width, working_height)?,
        TextureFormat::RG32 => decode_rg32(&working_data, working_width, working_height)?,
        TextureFormat::RGB48 => decode_rgb48(&working_data, working_width, working_height)?,
        TextureFormat::RGBA64 => decode_rgba64(&working_data, working_width, working_height)?,
        TextureFormat::R8_SIGNED => decode_r8_signed(&working_data, working_width, working_height)?,
        TextureFormat::RG16_SIGNED => {
            decode_rg16_signed(&working_data, working_width, working_height)?
        }
        TextureFormat::RGB24_SIGNED => {
            decode_rgb24_signed(&working_data, working_width, working_height)?
        }
        TextureFormat::RGBA32_SIGNED => {
            decode_rgba32_signed(&working_data, working_width, working_height)?
        }
        TextureFormat::R16_SIGNED => {
            decode_r16_signed(&working_data, working_width, working_height)?
        }
        TextureFormat::RG32_SIGNED => {
            decode_rg32_signed(&working_data, working_width, working_height)?
        }
        TextureFormat::RGB48_SIGNED => {
            decode_rgb48_signed(&working_data, working_width, working_height)?
        }
        TextureFormat::RGBA64_SIGNED => {
            decode_rgba64_signed(&working_data, working_width, working_height)?
        }
        TextureFormat::RGB565 => decode_rgb565(&working_data, working_width, working_height)?,
        TextureFormat::ARGB4444 => decode_argb4444(&working_data, working_width, working_height)?,
        TextureFormat::RGBA4444 => decode_rgba4444(&working_data, working_width, working_height)?,

        // DXT/BCn formats (use texpresso crate)
        TextureFormat::DXT1 => decode_dxt1(&working_data, working_width, working_height)?,
        TextureFormat::DXT3 => decode_dxt3(&working_data, working_width, working_height)?,
        TextureFormat::DXT5 => decode_dxt5(&working_data, working_width, working_height)?,
        TextureFormat::BC4 => decode_bc4(&working_data, working_width, working_height)?,
        TextureFormat::BC5 => decode_bc5(&working_data, working_width, working_height)?,
        TextureFormat::BC6H => decode_bc6h(&working_data, working_width, working_height)?,
        TextureFormat::BC7 => decode_bc7(&working_data, working_width, working_height)?,

        // Compressed texture formats (from Python CONV_TABLE lines 540-620)
        // All formats implemented using texture2ddecoder crate
        TextureFormat::ETC_RGB4 => decode_etc_rgb4(&working_data, working_width, working_height)?,
        TextureFormat::ETC2_RGB => decode_etc2_rgb(&working_data, working_width, working_height)?,
        TextureFormat::ETC2_RGBA1 => {
            decode_etc2_rgba1(&working_data, working_width, working_height)?
        }
        TextureFormat::ETC2_RGBA8 => {
            decode_etc2_rgba8(&working_data, working_width, working_height)?
        }
        TextureFormat::EAC_R => decode_eacr(&working_data, working_width, working_height)?,
        TextureFormat::EAC_R_SIGNED => {
            decode_eacr_signed(&working_data, working_width, working_height)?
        }
        TextureFormat::EAC_RG => decode_eacrg(&working_data, working_width, working_height)?,
        TextureFormat::EAC_RG_SIGNED => {
            decode_eacrg_signed(&working_data, working_width, working_height)?
        }
        TextureFormat::PVRTC_RGB2 => {
            decode_pvrtc_rgb2(&working_data, working_width, working_height)?
        }
        TextureFormat::PVRTC_RGBA2 => {
            decode_pvrtc_rgba2(&working_data, working_width, working_height)?
        }
        TextureFormat::PVRTC_RGB4 => {
            decode_pvrtc_rgb4(&working_data, working_width, working_height)?
        }
        TextureFormat::PVRTC_RGBA4 => {
            decode_pvrtc_rgba4(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_RGB_4x4 => {
            decode_astc_rgb_4x4(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_RGB_5x5 => {
            decode_astc_rgb_5x5(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_RGB_6x6 => {
            decode_astc_rgb_6x6(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_RGB_8x8 => {
            decode_astc_rgb_8x8(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_RGB_10x10 => {
            decode_astc_rgb_10x10(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_RGB_12x12 => {
            decode_astc_rgb_12x12(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_RGBA_4x4 => {
            decode_astc_rgba_4x4(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_RGBA_5x5 => {
            decode_astc_rgba_5x5(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_RGBA_6x6 => {
            decode_astc_rgba_6x6(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_RGBA_8x8 => {
            decode_astc_rgba_8x8(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_RGBA_10x10 => {
            decode_astc_rgba_10x10(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_RGBA_12x12 => {
            decode_astc_rgba_12x12(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_HDR_4x4 => {
            decode_astc_hdr_4x4(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_HDR_5x5 => {
            decode_astc_hdr_5x5(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_HDR_6x6 => {
            decode_astc_hdr_6x6(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_HDR_8x8 => {
            decode_astc_hdr_8x8(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_HDR_10x10 => {
            decode_astc_hdr_10x10(&working_data, working_width, working_height)?
        }
        TextureFormat::ASTC_HDR_12x12 => {
            decode_astc_hdr_12x12(&working_data, working_width, working_height)?
        }
        // Crunch formats are handled before dispatch (lines 163-170) and return early
        // These cases should never be reached
        TextureFormat::DXT1Crunched
        | TextureFormat::DXT5Crunched
        | TextureFormat::ETC_RGB4Crunched
        | TextureFormat::ETC2_RGBA8Crunched => {
            unreachable!("Crunch formats should be handled before dispatch")
        }
        TextureFormat::YUY2 => decode_yuy2(&working_data, working_width, working_height)?,
        TextureFormat::ETC_RGB4_3DS => {
            decode_etc_rgb4_3ds(&working_data, working_width, working_height)?
        }
        TextureFormat::ETC_RGBA8_3DS => {
            decode_etc_rgba8_3ds(&working_data, working_width, working_height)?
        }
        TextureFormat::ATC_RGB4 => decode_atc_rgb4(&working_data, working_width, working_height)?,
        TextureFormat::ATC_RGBA8 => decode_atc_rgba8(&working_data, working_width, working_height)?,
        _ => {
            return Err(crate::errors::UnityError::Other(format!(
                "Unsupported texture format: {:?}",
                texture_format
            )))
        }
    };

    // Post-processing: Nintendo Switch deswizzling, cropping, and flipping
    // Nintendo Switch deswizzling post-processing (Python lines 351-358)
    let mut final_image = decoded_image;
    if let Some(((block_width, block_height), gobs_per_block)) = switch_swizzle_params {
        // Get raw RGBA bytes from the decoded image
        let rgba_bytes = final_image.into_raw();

        // Deswizzle the texture data
        let deswizzled_data = texture_swizzler::deswizzle(
            &rgba_bytes,
            working_width as usize,
            working_height as usize,
            block_width,
            block_height,
            gobs_per_block,
        );

        // Create new image from deswizzled data
        final_image = RgbaImage::from_raw(working_width, working_height, deswizzled_data)
            .ok_or_else(|| UnityError::Other("Failed to create deswizzled image".to_string()))?;

        // Crop back to original dimensions (Python lines 357-358)
        if original_width != working_width || original_height != working_height {
            final_image =
                imageops::crop(&mut final_image, 0, 0, original_width, original_height).to_image();
        }
    }

    // Flip vertically if requested (Unity stores textures upside-down)
    let final_image = if flip {
        image::imageops::flip_vertical(&final_image)
    } else {
        final_image
    };

    Ok(final_image)
}

// ============================================================================
// Simple Raw Format Decoders
// ============================================================================

/// Decode RGBA32 format (8 bits per channel, 4 channels)
///
/// This is the simplest format - just copy bytes directly into RgbaImage
///
/// # Python Equivalent
/// Python line 546: (TF.RGBA32, pillow, "RGBA", "raw", "RGBA")
fn decode_rgba32(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 4) as usize;
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid RGBA32 data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let buffer = RgbaImage::from_raw(width, height, data.to_vec())
        .ok_or_else(|| UnityError::Other("Failed to create RGBA image".to_string()))?;

    Ok(buffer)
}

/// Decode RGB24 format (8 bits per channel, 3 channels, no alpha)
///
/// Need to add alpha channel (set to 255 = opaque)
///
/// # Python Equivalent
/// Python line 545: (TF.RGB24, pillow, "RGB", "raw", "RGB")
fn decode_rgb24(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 3) as usize;
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid RGB24 data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    // Create output buffer
    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    // Convert RGB -> RGBA
    for chunk in data.chunks_exact(3) {
        rgba_data.push(chunk[0]); // R
        rgba_data.push(chunk[1]); // G
        rgba_data.push(chunk[2]); // B
        rgba_data.push(255); // A (opaque)
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create RGB image".to_string()))?;

    Ok(buffer)
}

/// Decode ARGB32 format (8 bits per channel, channels in ARGB order)
///
/// Need to swap from ARGB to RGBA
///
/// # Python Equivalent
/// Python line 547: (TF.ARGB32, pillow, "RGBA", "raw", "ARGB")
fn decode_argb32(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 4) as usize;
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid ARGB32 data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    for chunk in data.chunks_exact(4) {
        // chunk[0] = A, chunk[1] = R, chunk[2] = G, chunk[3] = B
        // Reorder to R, G, B, A
        rgba_data.push(chunk[1]); // R
        rgba_data.push(chunk[2]); // G
        rgba_data.push(chunk[3]); // B
        rgba_data.push(chunk[0]); // A
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create ARGB image".to_string()))?;

    Ok(buffer)
}

/// Decode BGRA32 format (8 bits per channel, channels in BGRA order)
///
/// Need to swap from BGRA to RGBA
///
/// # Python Equivalent
/// Python line 558: (TF.BGRA32, pillow, "RGBA", "raw", "BGRA")
fn decode_bgra32(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 4) as usize;
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid BGRA32 data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    for chunk in data.chunks_exact(4) {
        // chunk[0] = B, chunk[1] = G, chunk[2] = R, chunk[3] = A
        // Reorder to R, G, B, A
        rgba_data.push(chunk[2]); // R
        rgba_data.push(chunk[1]); // G
        rgba_data.push(chunk[0]); // B
        rgba_data.push(chunk[3]); // A
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create BGRA image".to_string()))?;

    Ok(buffer)
}

/// Decode BGR24 format (8 bits per channel, 3 channels, BGR order)
///
/// Need to reverse to RGB and add alpha channel (set to 255 = opaque)
///
/// # Python Equivalent
/// Python line 550: (TF.BGR24, pillow, "RGB", "raw", "BGR")
fn decode_bgr24(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 3) as usize;
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid BGR24 data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    // Convert BGR -> RGBA
    for chunk in data.chunks_exact(3) {
        // chunk[0] = B, chunk[1] = G, chunk[2] = R
        // Reorder to R, G, B, A
        rgba_data.push(chunk[2]); // R (was at index 2)
        rgba_data.push(chunk[1]); // G (stays at index 1)
        rgba_data.push(chunk[0]); // B (was at index 0)
        rgba_data.push(255); // A (opaque)
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create BGR24 image".to_string()))?;

    Ok(buffer)
}

/// Decode Alpha8 format (8-bit alpha channel only)
///
/// Converts to RGBA with white color (255, 255, 255) and the alpha from input.
/// Used for UI masks, text rendering, and particle effects.
///
/// # Python Equivalent
/// Python line 543: (TF.Alpha8, pillow, "RGBA", "raw", "A")
fn decode_alpha8(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 1) as usize; // 1 byte per pixel
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid Alpha8 data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    // Convert A -> RGBA (white with varying alpha)
    for &alpha in data {
        rgba_data.push(255); // R (white)
        rgba_data.push(255); // G (white)
        rgba_data.push(255); // B (white)
        rgba_data.push(alpha); // A (from input)
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create Alpha8 image".to_string()))?;

    Ok(buffer)
}

/// Decode R8 format (8-bit red channel only)
///
/// Converts to RGBA with only the red channel populated, green/blue = 0.
/// Used for heightmaps, masks, and single-channel data.
///
/// # Python Equivalent
/// Python line 551: (TF.R8, pillow, "RGB", "raw", "R")
fn decode_r8(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 1) as usize; // 1 byte per pixel
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid R8 data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    // Convert R -> RGBA (red only, green/blue = 0)
    for &red in data {
        rgba_data.push(red); // R (from input)
        rgba_data.push(0); // G (zero)
        rgba_data.push(0); // B (zero)
        rgba_data.push(255); // A (opaque)
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create R8 image".to_string()))?;

    Ok(buffer)
}

/// Decode R16 format (16-bit red channel only)
///
/// Converts 16-bit red values to 8-bit by taking the high byte (>> 8).
/// Green and blue channels are set to 0, alpha to 255.
///
/// # Python Equivalent
/// Python line 552: (TF.R16, pillow, "RGB", "raw", "R;16")
fn decode_r16(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 2) as usize; // 2 bytes per pixel
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid R16 data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    // Convert R16 -> RGBA (red only, green/blue = 0)
    for chunk in data.chunks_exact(2) {
        // Read 16-bit value (little-endian)
        let r16 = u16::from_le_bytes([chunk[0], chunk[1]]);

        // Scale from 16-bit to 8-bit by taking high byte
        let r8 = (r16 >> 8) as u8;

        rgba_data.push(r8); // R (from input, scaled)
        rgba_data.push(0); // G (zero)
        rgba_data.push(0); // B (zero)
        rgba_data.push(255); // A (opaque)
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create R16 image".to_string()))?;

    Ok(buffer)
}

/// Decode RHalf format (16-bit half-precision float red channel)
///
/// Converts half-floats to 8-bit by scaling [0.0, 1.0] → [0, 255]
/// Uses the same algorithm as Python: f16 * 256, then clamp to u8
///
/// # Python Equivalent
/// Python line 559: (TF.RHalf, half, "R", "raw", "R")
/// Python lines 480-495: half() function with struct.unpack("e", ...) * 256
fn decode_rhalf(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 2) as usize; // 2 bytes per pixel (half-float)
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid RHalf data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    // Convert RHalf -> RGBA (red only, green/blue = 0)
    for chunk in data.chunks_exact(2) {
        // Read 16-bit half-float (little-endian)
        let half_bits = u16::from_le_bytes([chunk[0], chunk[1]]);
        let half_float = half::f16::from_bits(half_bits);

        // Convert to f32, then scale by 256 (Python uses * 256, not * 255!)
        let f32_val = half_float.to_f32();
        let scaled = f32_val * 256.0;

        // Clamp to [0, 255] and convert to u8
        let r8 = scaled.clamp(0.0, 255.0) as u8;

        rgba_data.push(r8); // R (from input, scaled)
        rgba_data.push(0); // G (zero)
        rgba_data.push(0); // B (zero)
        rgba_data.push(255); // A (opaque)
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create RHalf image".to_string()))?;

    Ok(buffer)
}

/// Decode RGHalf format (2-channel 16-bit half-precision floats)
///
/// Converts RG half-floats to RGBA, with B=0 and A=255.
/// Python uses a special decoder that pads RG data to RGB by adding zeros,
/// then converts f16 → u8 by scaling * 256.
///
/// # Python Equivalent
/// Python line 560: (TF.RGHalf, rg, "RGB", "raw", "RGE")
/// Python lines 507-521: rg() function that pads RG to RGB
/// Python lines 480-495: half() function for f16 → u8 conversion
fn decode_rghalf(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 4) as usize; // 2 channels × 2 bytes each = 4 bytes per pixel
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid RGHalf data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    // Convert RGHalf -> RGBA (RG channels from input, B=0, A=255)
    for chunk in data.chunks_exact(4) {
        // Read two 16-bit half-floats (little-endian): R and G
        let r_half_bits = u16::from_le_bytes([chunk[0], chunk[1]]);
        let g_half_bits = u16::from_le_bytes([chunk[2], chunk[3]]);

        let r_half = half::f16::from_bits(r_half_bits);
        let g_half = half::f16::from_bits(g_half_bits);

        // Convert to f32, then scale by 256 (Python: struct.unpack("e", ...) * 256)
        let r_scaled = r_half.to_f32() * 256.0;
        let g_scaled = g_half.to_f32() * 256.0;

        // Clamp to [0, 255] and convert to u8
        let r8 = r_scaled.clamp(0.0, 255.0) as u8;
        let g8 = g_scaled.clamp(0.0, 255.0) as u8;

        rgba_data.push(r8); // R (from input, scaled)
        rgba_data.push(g8); // G (from input, scaled)
        rgba_data.push(0); // B (zero - Python pads with zeros)
        rgba_data.push(255); // A (opaque)
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create RGHalf image".to_string()))?;

    Ok(buffer)
}

/// Decode RGBAHalf format (3-channel 16-bit half-precision floats)
///
/// NOTE: Despite the "RGBA" name, Python's implementation only uses 3 channels (RGB).
/// This matches Python's exact behavior for parity.
/// Converts RGB half-floats to RGBA, with A=255.
///
/// # Python Equivalent
/// Python line 561: (TF.RGBAHalf, half, "RGB", "raw", "RGB")
/// Python lines 480-495: half() function with len(codec)=3 for RGB
fn decode_rgbahalf(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 6) as usize; // 3 channels × 2 bytes each = 6 bytes per pixel
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid RGBAHalf data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    // Convert RGBAHalf -> RGBA (RGB channels from input, A=255)
    // NOTE: Python only reads 3 channels despite "RGBA" in the name
    for chunk in data.chunks_exact(6) {
        // Read three 16-bit half-floats (little-endian): R, G, and B
        let r_half_bits = u16::from_le_bytes([chunk[0], chunk[1]]);
        let g_half_bits = u16::from_le_bytes([chunk[2], chunk[3]]);
        let b_half_bits = u16::from_le_bytes([chunk[4], chunk[5]]);

        let r_half = half::f16::from_bits(r_half_bits);
        let g_half = half::f16::from_bits(g_half_bits);
        let b_half = half::f16::from_bits(b_half_bits);

        // Convert to f32, then scale by 256
        let r_scaled = r_half.to_f32() * 256.0;
        let g_scaled = g_half.to_f32() * 256.0;
        let b_scaled = b_half.to_f32() * 256.0;

        // Clamp to [0, 255] and convert to u8
        let r8 = r_scaled.clamp(0.0, 255.0) as u8;
        let g8 = g_scaled.clamp(0.0, 255.0) as u8;
        let b8 = b_scaled.clamp(0.0, 255.0) as u8;

        rgba_data.push(r8); // R (from input, scaled)
        rgba_data.push(g8); // G (from input, scaled)
        rgba_data.push(b8); // B (from input, scaled)
        rgba_data.push(255); // A (opaque - not in input data)
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create RGBAHalf image".to_string()))?;

    Ok(buffer)
}

/// Decode RFloat format (32-bit single-precision float red channel)
///
/// Converts floats to 8-bit by scaling [0.0, 1.0] → [0, 255]
/// Uses the same algorithm as Python: f32 * 256, then clamp to u8
///
/// # Python Equivalent
/// Python line 562: (TF.RFloat, pillow, "RGB", "raw", "RF")
fn decode_rfloat(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 4) as usize; // 4 bytes per pixel (f32)
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid RFloat data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    for chunk in data.chunks_exact(4) {
        // Read 32-bit float (little-endian)
        let float_bits = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let f32_val = f32::from_bits(float_bits);

        // Scale by 256 (same as RHalf)
        let scaled = f32_val * 256.0;

        // Clamp to [0, 255] and convert to u8
        let r8 = scaled.clamp(0.0, 255.0) as u8;

        rgba_data.push(r8); // R
        rgba_data.push(0); // G
        rgba_data.push(0); // B
        rgba_data.push(255); // A
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create RFloat image".to_string()))?;

    Ok(buffer)
}

/// Decode RGFloat format (2-channel 32-bit floats)
///
/// Converts RG floats to RGBA, with B=0 and A=255.
/// Python uses a special decoder that pads RG data to RGB by adding zeros,
/// then converts f32 → u8 by scaling * 256.
///
/// # Python Equivalent
/// Python line 563: (TF.RGFloat, rg, "RGB", "raw", "RGF")
/// Python lines 507-521: rg() function that pads RG to RGB
fn decode_rgfloat(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 8) as usize; // 2 channels × 4 bytes each = 8 bytes per pixel
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid RGFloat data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    // Convert RGFloat -> RGBA (RG channels from input, B=0, A=255)
    for chunk in data.chunks_exact(8) {
        // Read two 32-bit floats (little-endian): R and G
        let r_bits = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let g_bits = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);

        let r_f32 = f32::from_bits(r_bits);
        let g_f32 = f32::from_bits(g_bits);

        // Convert floats, scale by 256 (same as RGHalf but with f32)
        let r_scaled = r_f32 * 256.0;
        let g_scaled = g_f32 * 256.0;

        // Clamp to [0, 255] and convert to u8
        let r8 = r_scaled.clamp(0.0, 255.0) as u8;
        let g8 = g_scaled.clamp(0.0, 255.0) as u8;

        rgba_data.push(r8); // R (from input, scaled)
        rgba_data.push(g8); // G (from input, scaled)
        rgba_data.push(0); // B (zero - Python pads with zeros)
        rgba_data.push(255); // A (opaque)
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create RGFloat image".to_string()))?;

    Ok(buffer)
}

/// Decode RGBAFloat format (32-bit float RGBA, 4 channels)
///
/// Converts 4 float channels to 8-bit by scaling [0.0, 1.0] → [0, 255]
/// Each pixel is 16 bytes: RGBA as f32 values
///
/// # Python Equivalent
/// Python line 564: (TF.RGBAFloat, pillow, "RGBA", "raw", "RGBAF")
fn decode_rgbafloat(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 16) as usize; // 16 bytes per pixel (4 × f32)
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid RGBAFloat data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    for chunk in data.chunks_exact(16) {
        // Read 4 floats (R, G, B, A) - all little-endian
        let r_bits = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let g_bits = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
        let b_bits = u32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]);
        let a_bits = u32::from_le_bytes([chunk[12], chunk[13], chunk[14], chunk[15]]);

        let r_f32 = f32::from_bits(r_bits);
        let g_f32 = f32::from_bits(g_bits);
        let b_f32 = f32::from_bits(b_bits);
        let a_f32 = f32::from_bits(a_bits);

        // Scale all channels by 256
        let r8 = (r_f32 * 256.0).clamp(0.0, 255.0) as u8;
        let g8 = (g_f32 * 256.0).clamp(0.0, 255.0) as u8;
        let b8 = (b_f32 * 256.0).clamp(0.0, 255.0) as u8;
        let a8 = (a_f32 * 256.0).clamp(0.0, 255.0) as u8;

        rgba_data.push(r8);
        rgba_data.push(g8);
        rgba_data.push(b8);
        rgba_data.push(a8);
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create RGBAFloat image".to_string()))?;

    Ok(buffer)
}

/// Decode ARGBFloat format (32-bit float ARGB, 4 channels with swapped order)
///
/// Same as RGBAFloat but channels are in ARGB order in the source data
/// Converts 4 float channels to 8-bit by scaling [0.0, 1.0] → [0, 255]
/// Each pixel is 16 bytes: ARGB as f32 values
///
/// # Python Equivalent
/// Python line 548: (TF.ARGBFloat, pillow, "RGBA", "raw", "RGBAF", (2, 1, 0, 3))
fn decode_argbfloat(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 16) as usize; // 16 bytes per pixel (4 × f32)
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid ARGBFloat data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    for chunk in data.chunks_exact(16) {
        // Read 4 floats (A, R, G, B) - all little-endian
        let a_bits = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let r_bits = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
        let g_bits = u32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]);
        let b_bits = u32::from_le_bytes([chunk[12], chunk[13], chunk[14], chunk[15]]);

        let a_f32 = f32::from_bits(a_bits);
        let r_f32 = f32::from_bits(r_bits);
        let g_f32 = f32::from_bits(g_bits);
        let b_f32 = f32::from_bits(b_bits);

        // Scale all channels by 256
        let r8 = (r_f32 * 256.0).clamp(0.0, 255.0) as u8;
        let g8 = (g_f32 * 256.0).clamp(0.0, 255.0) as u8;
        let b8 = (b_f32 * 256.0).clamp(0.0, 255.0) as u8;
        let a8 = (a_f32 * 256.0).clamp(0.0, 255.0) as u8;

        // Output in RGBA order
        rgba_data.push(r8);
        rgba_data.push(g8);
        rgba_data.push(b8);
        rgba_data.push(a8);
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create ARGBFloat image".to_string()))?;

    Ok(buffer)
}

/// Decode RG16 format (16-bit red and green channels)
///
/// Converts 16-bit RG values to 8-bit by taking the high byte (>> 8).
/// Blue channel is set to 0, alpha to 255.
///
/// # Python Equivalent
/// Python line 553: (TF.RG16, rg, "RGB", "raw", "RG")
fn decode_rg16(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 4) as usize; // 4 bytes per pixel (2 for R, 2 for G)
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid RG16 data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    for chunk in data.chunks_exact(4) {
        // Read two 16-bit values (both little-endian)
        let r16 = u16::from_le_bytes([chunk[0], chunk[1]]);
        let g16 = u16::from_le_bytes([chunk[2], chunk[3]]);

        // Scale from 16-bit to 8-bit
        let r8 = (r16 >> 8) as u8;
        let g8 = (g16 >> 8) as u8;

        rgba_data.push(r8); // R
        rgba_data.push(g8); // G
        rgba_data.push(0); // B (zero)
        rgba_data.push(255); // A (opaque)
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create RG16 image".to_string()))?;

    Ok(buffer)
}

/// Decode RG32 format (2-channel 16-bit unsigned integers)
///
/// NOTE: Despite the name "RG32", this format uses 16-bit values per channel (32 bits total per pixel).
/// Converts 2 × 16-bit unsigned integers to RGBA with B=0 and A=255.
/// Python uses rg decoder with codec="RG;16".
///
/// # Python Equivalent
/// Python line 609: (TF.RG32, rg, "RGB", "raw", "RG;16")
/// Python lines 507-521: rg() function that pads RG to RGB
fn decode_rg32(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 4) as usize; // 4 bytes per pixel (2 × u16)
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid RG32 data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    // Convert RG32 -> RGBA (RG from input, B=0, A=255)
    for chunk in data.chunks_exact(4) {
        // Read two 16-bit unsigned integers (little-endian)
        let r16 = u16::from_le_bytes([chunk[0], chunk[1]]);
        let g16 = u16::from_le_bytes([chunk[2], chunk[3]]);

        // Scale from 16-bit to 8-bit (take high byte)
        let r8 = (r16 >> 8) as u8;
        let g8 = (g16 >> 8) as u8;

        rgba_data.push(r8); // R (from input)
        rgba_data.push(g8); // G (from input)
        rgba_data.push(0); // B (zero - Python pads with zeros)
        rgba_data.push(255); // A (opaque)
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create RG32 image".to_string()))?;

    Ok(buffer)
}

/// Decode RGB48 format (3-channel 16-bit unsigned integers)
///
/// Each pixel is 6 bytes: 3 channels × 16 bits (RGB).
/// Converts 3 × 16-bit unsigned integers to RGBA with A=255.
///
/// # Python Equivalent
/// Python line 610: (TF.RGB48, pillow, "RGB", "raw", "RGB;16")
fn decode_rgb48(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 6) as usize; // 6 bytes per pixel (3 × u16)
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid RGB48 data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    // Convert RGB48 -> RGBA (RGB from input, A=255)
    for chunk in data.chunks_exact(6) {
        // Read three 16-bit unsigned integers (little-endian)
        let r16 = u16::from_le_bytes([chunk[0], chunk[1]]);
        let g16 = u16::from_le_bytes([chunk[2], chunk[3]]);
        let b16 = u16::from_le_bytes([chunk[4], chunk[5]]);

        // Scale from 16-bit to 8-bit (take high byte)
        let r8 = (r16 >> 8) as u8;
        let g8 = (g16 >> 8) as u8;
        let b8 = (b16 >> 8) as u8;

        rgba_data.push(r8); // R (from input)
        rgba_data.push(g8); // G (from input)
        rgba_data.push(b8); // B (from input)
        rgba_data.push(255); // A (opaque)
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create RGB48 image".to_string()))?;

    Ok(buffer)
}

/// Decode RGBA64 format (4-channel 16-bit unsigned integers)
///
/// Each pixel is 8 bytes: 4 channels × 16 bits (RGBA).
/// Converts 4 × 16-bit unsigned integers to RGBA.
///
/// # Python Equivalent
/// Python line 611: (TF.RGBA64, pillow, "RGBA", "raw", "RGBA;16")
fn decode_rgba64(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 8) as usize; // 8 bytes per pixel (4 × u16)
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid RGBA64 data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    // Convert RGBA64 -> RGBA (all 4 channels from input)
    for chunk in data.chunks_exact(8) {
        // Read four 16-bit unsigned integers (little-endian)
        let r16 = u16::from_le_bytes([chunk[0], chunk[1]]);
        let g16 = u16::from_le_bytes([chunk[2], chunk[3]]);
        let b16 = u16::from_le_bytes([chunk[4], chunk[5]]);
        let a16 = u16::from_le_bytes([chunk[6], chunk[7]]);

        // Scale from 16-bit to 8-bit (take high byte)
        let r8 = (r16 >> 8) as u8;
        let g8 = (g16 >> 8) as u8;
        let b8 = (b16 >> 8) as u8;
        let a8 = (a16 >> 8) as u8;

        rgba_data.push(r8); // R (from input)
        rgba_data.push(g8); // G (from input)
        rgba_data.push(b8); // B (from input)
        rgba_data.push(a8); // A (from input)
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create RGBA64 image".to_string()))?;

    Ok(buffer)
}

/// Decode RGB9e5Float format (shared exponent HDR format)
///
/// Each pixel is 4 bytes (32 bits) with:
/// - Bits 0-8: Red (9 bits)
/// - Bits 9-17: Green (9 bits)
/// - Bits 18-26: Blue (9 bits)
/// - Bits 27-31: Shared exponent (5 bits)
///
/// This is a compact HDR format where all three color channels share a single
/// 5-bit exponent, allowing representation of high dynamic range values.
///
/// # Python Equivalent
/// Python lines 524-537: rgb9e5float function
fn decode_rgb9e5_float(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 4) as usize; // 4 bytes per pixel
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid RGB9e5Float data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    // Convert RGB9e5Float -> RGBA
    for chunk in data.chunks_exact(4) {
        // Read 32-bit value (little-endian)
        let n = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);

        // Extract 5-bit shared exponent (bits 27-31)
        let exponent = ((n >> 27) & 0x1F) as i32;

        // Calculate scale factor: 2^(exponent - 24)
        let scale_f = 2f32.powi(exponent - 24);

        // Scale by 255.0 to convert to 8-bit range
        let scale_b = scale_f * 255.0;

        // Extract 9-bit channel values and apply scale
        let r9 = (n & 0x1FF) as f32; // Bits 0-8: Red
        let g9 = ((n >> 9) & 0x1FF) as f32; // Bits 9-17: Green
        let b9 = ((n >> 18) & 0x1FF) as f32; // Bits 18-26: Blue

        let r = (r9 * scale_b).min(255.0) as u8;
        let g = (g9 * scale_b).min(255.0) as u8;
        let b = (b9 * scale_b).min(255.0) as u8;

        rgba_data.push(r); // R
        rgba_data.push(g); // G
        rgba_data.push(b); // B
        rgba_data.push(255); // A (opaque)
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create RGB9e5Float image".to_string()))?;

    Ok(buffer)
}

/// Decode RGB565 format (16-bit packed color: 5-6-5 bits per channel)
///
/// Bit layout in little-endian u16:
/// - Bits 11-15: Red (5 bits)
/// - Bits 5-10: Green (6 bits)
/// - Bits 0-4: Blue (5 bits)
///
/// # Python Equivalent
/// Python line 549: (TF.RGB565, pillow, "RGB", "raw", "BGR;16")

/// Generic decoder for signed integer formats
///
/// Converts signed integers to unsigned by shifting the range:
/// - i8:  [-128, 127]  → u8:  [0, 255]   (add 128)
/// - i16: [-32768, 32767] → u16: [0, 65535] (add 32768, then scale to u8)
///
/// Python equivalent: Uses PIL's signed format specifiers like "R;8s", "RG;8s", etc.
fn decode_signed_integer(
    data: &[u8],
    width: u32,
    height: u32,
    channels: usize,
    bytes_per_channel: usize,
    format_name: &str,
) -> UnityResult<RgbaImage> {
    let bytes_per_pixel = channels * bytes_per_channel;
    let expected_size = (width * height) as usize * bytes_per_pixel;

    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid {} data size: expected {} bytes, got {}",
            format_name,
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    for chunk in data.chunks_exact(bytes_per_pixel) {
        // Read and convert each channel from signed to unsigned
        for i in 0..channels {
            let offset = i * bytes_per_channel;

            let unsigned = if bytes_per_channel == 1 {
                // i8 → u8: shift range by adding 128
                let signed = chunk[offset] as i8;
                (signed as i16 + 128) as u8
            } else {
                // i16 → u8: shift range by adding 32768, then scale down
                let signed = i16::from_le_bytes([chunk[offset], chunk[offset + 1]]);
                ((signed as i32 + 32768) >> 8) as u8
            };

            rgba_data.push(unsigned);
        }

        // Pad remaining channels to make RGBA
        match channels {
            1 => {
                rgba_data.push(0); // G = 0
                rgba_data.push(0); // B = 0
                rgba_data.push(255); // A = 255
            }
            2 => {
                rgba_data.push(0); // B = 0
                rgba_data.push(255); // A = 255
            }
            3 => {
                rgba_data.push(255); // A = 255
            }
            4 => {} // Already RGBA
            _ => {}
        }
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other(format!("Failed to create {} image", format_name)))?;

    Ok(buffer)
}

fn decode_rgb565(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 2) as usize; // 2 bytes per pixel
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid RGB565 data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    for chunk in data.chunks_exact(2) {
        // Read 16-bit value (little-endian)
        let rgb565 = u16::from_le_bytes([chunk[0], chunk[1]]);

        // Extract 5-bit red (bits 11-15)
        let r5 = ((rgb565 >> 11) & 0b11111) as u8;

        // Extract 6-bit green (bits 5-10)
        let g6 = ((rgb565 >> 5) & 0b111111) as u8;

        // Extract 5-bit blue (bits 0-4)
        let b5 = (rgb565 & 0b11111) as u8;

        // Scale to 8-bit: multiply by 255 and divide by max value
        // For 5-bit: max = 31, so scale = val * 255 / 31 ≈ val * 8.226
        // For 6-bit: max = 63, so scale = val * 255 / 63 ≈ val * 4.047
        // Common approach: left-shift and OR with high bits
        let r8 = (r5 << 3) | (r5 >> 2); // 5-bit → 8-bit
        let g8 = (g6 << 2) | (g6 >> 4); // 6-bit → 8-bit
        let b8 = (b5 << 3) | (b5 >> 2); // 5-bit → 8-bit

        rgba_data.push(r8); // R
        rgba_data.push(g8); // G
        rgba_data.push(b8); // B
        rgba_data.push(255); // A (opaque)
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create RGB565 image".to_string()))?;

    Ok(buffer)
}

/// Decode ARGB4444 format (16-bit packed color: 4 bits per channel, ARGB order)
///
/// Bit layout in little-endian u16:
/// - Bits 12-15: Alpha (4 bits)
/// - Bits 8-11: Red (4 bits)
/// - Bits 4-7: Green (4 bits)
/// - Bits 0-3: Blue (4 bits)
///
/// # Python Equivalent
/// Python line 544: (TF.ARGB4444, pillow, "RGBA", "raw", "RGBA;4B", (2, 1, 0, 3))
fn decode_argb4444(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 2) as usize; // 2 bytes per pixel
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid ARGB4444 data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    for chunk in data.chunks_exact(2) {
        // Read 16-bit value (little-endian)
        let argb4444 = u16::from_le_bytes([chunk[0], chunk[1]]);

        // Extract 4-bit channels
        let a4 = ((argb4444 >> 12) & 0xF) as u8; // Alpha (bits 12-15)
        let r4 = ((argb4444 >> 8) & 0xF) as u8; // Red (bits 8-11)
        let g4 = ((argb4444 >> 4) & 0xF) as u8; // Green (bits 4-7)
        let b4 = (argb4444 & 0xF) as u8; // Blue (bits 0-3)

        // Scale from 4-bit to 8-bit by duplicating bits
        // (val << 4) | val: e.g., 0xF (15) -> 0xFF (255)
        let r8 = (r4 << 4) | r4;
        let g8 = (g4 << 4) | g4;
        let b8 = (b4 << 4) | b4;
        let a8 = (a4 << 4) | a4;

        rgba_data.push(r8); // R
        rgba_data.push(g8); // G
        rgba_data.push(b8); // B
        rgba_data.push(a8); // A
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create ARGB4444 image".to_string()))?;

    Ok(buffer)
}

/// Decode RGBA4444 format (16-bit packed color: 4 bits per channel, RGBA order)
///
/// Bit layout in little-endian u16:
/// - Bits 12-15: Red (4 bits)
/// - Bits 8-11: Green (4 bits)
/// - Bits 4-7: Blue (4 bits)
/// - Bits 0-3: Alpha (4 bits)
///
/// # Python Equivalent
/// Python line 557: (TF.RGBA4444, pillow, "RGBA", "raw", "RGBA;4B", (3, 2, 1, 0))
fn decode_rgba4444(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let expected_size = (width * height * 2) as usize; // 2 bytes per pixel
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid RGBA4444 data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    for chunk in data.chunks_exact(2) {
        // Read 16-bit value (little-endian)
        let rgba4444 = u16::from_le_bytes([chunk[0], chunk[1]]);

        // Extract 4-bit channels (RGBA order - different from ARGB4444!)
        let r4 = ((rgba4444 >> 12) & 0xF) as u8; // Red (bits 12-15)
        let g4 = ((rgba4444 >> 8) & 0xF) as u8; // Green (bits 8-11)
        let b4 = ((rgba4444 >> 4) & 0xF) as u8; // Blue (bits 4-7)
        let a4 = (rgba4444 & 0xF) as u8; // Alpha (bits 0-3)

        // Scale from 4-bit to 8-bit
        let r8 = (r4 << 4) | r4;
        let g8 = (g4 << 4) | g4;
        let b8 = (b4 << 4) | b4;
        let a8 = (a4 << 4) | a4;

        rgba_data.push(r8); // R
        rgba_data.push(g8); // G
        rgba_data.push(b8); // B
        rgba_data.push(a8); // A
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create RGBA4444 image".to_string()))?;

    Ok(buffer)
}

/// Decode R8_SIGNED format (1-channel signed integers, 1 byte per channel)
///
/// # Python Equivalent
/// Python line 612: (TF.R8_SIGNED, pillow, "R", "raw", "R;8s")
fn decode_r8_signed(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_signed_integer(data, width, height, 1, 1, "R8_SIGNED")
}

/// Decode RG16_SIGNED format (2-channel signed integers, 1 byte per channel)
///
/// # Python Equivalent
/// Python line 613: (TF.RG16_SIGNED, rg, "RGB", "raw", "RG;8s")
fn decode_rg16_signed(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_signed_integer(data, width, height, 2, 1, "RG16_SIGNED")
}

/// Decode RGB24_SIGNED format (3-channel signed integers, 1 byte per channel)
///
/// # Python Equivalent
/// Python line 614: (TF.RGB24_SIGNED, pillow, "RGB", "raw", "RGB;8s")
fn decode_rgb24_signed(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_signed_integer(data, width, height, 3, 1, "RGB24_SIGNED")
}

/// Decode RGBA32_SIGNED format (4-channel signed integers, 1 byte per channel)
///
/// # Python Equivalent
/// Python line 615: (TF.RGBA32_SIGNED, pillow, "RGBA", "raw", "RGBA;8s")
fn decode_rgba32_signed(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_signed_integer(data, width, height, 4, 1, "RGBA32_SIGNED")
}

/// Decode R16_SIGNED format (1-channel signed integers, 2 bytes per channel)
///
/// # Python Equivalent
/// Python line 616: (TF.R16_SIGNED, pillow, "R", "raw", "R;16s")
fn decode_r16_signed(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_signed_integer(data, width, height, 1, 2, "R16_SIGNED")
}

/// Decode RG32_SIGNED format (2-channel signed integers, 2 bytes per channel)
///
/// # Python Equivalent
/// Python line 617: (TF.RG32_SIGNED, rg, "RGB", "raw", "RG;16s")
fn decode_rg32_signed(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_signed_integer(data, width, height, 2, 2, "RG32_SIGNED")
}

/// Decode RGB48_SIGNED format (3-channel signed integers, 2 bytes per channel)
///
/// # Python Equivalent
/// Python line 618: (TF.RGB48_SIGNED, pillow, "RGB", "raw", "RGB;16s")
fn decode_rgb48_signed(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_signed_integer(data, width, height, 3, 2, "RGB48_SIGNED")
}

/// Decode RGBA64_SIGNED format (4-channel signed integers, 2 bytes per channel)
///
/// # Python Equivalent
/// Python line 619: (TF.RGBA64_SIGNED, pillow, "RGBA", "raw", "RGBA;16s")
fn decode_rgba64_signed(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_signed_integer(data, width, height, 4, 2, "RGBA64_SIGNED")
}

// ============================================================================
// DXT/BCn Format Decoders (using texpresso crate)
// ============================================================================

/// Decode DXT1/BC1 format (4x4 blocks, 64 bits per block, RGB with 1-bit alpha)
///
/// Uses the texpresso crate for fast decompression
///
/// # Python Equivalent
/// Python line 554: (TF.DXT1, pillow, "RGBA", "bcn", 1)
fn decode_dxt1(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    // Get padded dimensions for block compression (4x4 blocks)
    let block_size = get_block_size(TextureFormat::DXT1).unwrap_or((4, 4));
    let (padded_width, padded_height) =
        get_padded_dimensions(width, height, block_size.0, block_size.1);

    // Create output buffer for RGBA data
    let mut output = vec![0u8; (padded_width * padded_height * 4) as usize];

    // Decompress using texpresso
    Format::Bc1.decompress(
        data,
        padded_width as usize,
        padded_height as usize,
        &mut output,
    );

    // If image was padded, crop to actual size
    if padded_width != width || padded_height != height {
        let mut cropped = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            let src_offset = (y * padded_width * 4) as usize;
            let dst_offset = (y * width * 4) as usize;
            let row_bytes = (width * 4) as usize;
            cropped[dst_offset..dst_offset + row_bytes]
                .copy_from_slice(&output[src_offset..src_offset + row_bytes]);
        }
        output = cropped;
    }

    let buffer = RgbaImage::from_raw(width, height, output)
        .ok_or_else(|| UnityError::Other("Failed to create DXT1 image".to_string()))?;

    Ok(buffer)
}

/// Decode DXT3/BC2 format (4x4 blocks, 128 bits per block, RGBA with explicit alpha)
///
/// Uses the texpresso crate for fast decompression. Unlike DXT5's interpolated alpha,
/// DXT3 stores 4 bits of alpha per pixel (16 alpha values per 4x4 block).
///
/// # Python Equivalent
/// Python line 555: (TF.DXT3, pillow, "RGBA", "bcn", 2)
fn decode_dxt3(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    // Get padded dimensions for block compression (4x4 blocks)
    let block_size = get_block_size(TextureFormat::DXT3).unwrap_or((4, 4));
    let (padded_width, padded_height) =
        get_padded_dimensions(width, height, block_size.0, block_size.1);

    // Create output buffer for RGBA data
    let mut output = vec![0u8; (padded_width * padded_height * 4) as usize];

    // Decompress using texpresso (BC2 is DXT3)
    Format::Bc2.decompress(
        data,
        padded_width as usize,
        padded_height as usize,
        &mut output,
    );

    // If image was padded, crop to actual size
    if padded_width != width || padded_height != height {
        let mut cropped = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            let src_offset = (y * padded_width * 4) as usize;
            let dst_offset = (y * width * 4) as usize;
            let row_bytes = (width * 4) as usize;
            cropped[dst_offset..dst_offset + row_bytes]
                .copy_from_slice(&output[src_offset..src_offset + row_bytes]);
        }
        output = cropped;
    }

    let buffer = RgbaImage::from_raw(width, height, output)
        .ok_or_else(|| UnityError::Other("Failed to create DXT3 image".to_string()))?;

    Ok(buffer)
}

/// Decode DXT5/BC3 format (4x4 blocks, 128 bits per block, RGBA with interpolated alpha)
///
/// # Python Equivalent
/// Python line 556: (TF.DXT5, pillow, "RGBA", "bcn", 3)
fn decode_dxt5(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    // Get padded dimensions for block compression (4x4 blocks)
    let block_size = get_block_size(TextureFormat::DXT5).unwrap_or((4, 4));
    let (padded_width, padded_height) =
        get_padded_dimensions(width, height, block_size.0, block_size.1);

    // Create output buffer for RGBA data
    let mut output = vec![0u8; (padded_width * padded_height * 4) as usize];

    // Decompress using texpresso
    Format::Bc3.decompress(
        data,
        padded_width as usize,
        padded_height as usize,
        &mut output,
    );

    // If image was padded, crop to actual size
    if padded_width != width || padded_height != height {
        let mut cropped = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            let src_offset = (y * padded_width * 4) as usize;
            let dst_offset = (y * width * 4) as usize;
            let row_bytes = (width * 4) as usize;
            cropped[dst_offset..dst_offset + row_bytes]
                .copy_from_slice(&output[src_offset..src_offset + row_bytes]);
        }
        output = cropped;
    }

    let buffer = RgbaImage::from_raw(width, height, output)
        .ok_or_else(|| UnityError::Other("Failed to create DXT5 image".to_string()))?;

    Ok(buffer)
}

/// Decode BC4 format (4x4 blocks, 64 bits per block, single channel)
///
/// # Python Equivalent
/// Python line 567: (TF.BC4, pillow, "L", "bcn", 4)
fn decode_bc4(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let block_size = get_block_size(TextureFormat::BC4).unwrap_or((4, 4));
    let (padded_width, padded_height) =
        get_padded_dimensions(width, height, block_size.0, block_size.1);

    // BC4 decompresses to 1 byte per pixel (grayscale)
    let mut grayscale = vec![0u8; (padded_width * padded_height) as usize];

    Format::Bc4.decompress(
        data,
        padded_width as usize,
        padded_height as usize,
        &mut grayscale,
    );

    // Convert grayscale to RGBA: (L, L, L, 255)
    let mut rgba_data = Vec::with_capacity((padded_width * padded_height * 4) as usize);
    for &luminance in &grayscale {
        rgba_data.push(luminance); // R
        rgba_data.push(luminance); // G
        rgba_data.push(luminance); // B
        rgba_data.push(255); // A
    }

    // Crop if needed
    let output = if padded_width != width || padded_height != height {
        let mut cropped = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            let src_offset = (y * padded_width * 4) as usize;
            let dst_offset = (y * width * 4) as usize;
            let row_bytes = (width * 4) as usize;
            cropped[dst_offset..dst_offset + row_bytes]
                .copy_from_slice(&rgba_data[src_offset..src_offset + row_bytes]);
        }
        cropped
    } else {
        rgba_data
    };

    let buffer = RgbaImage::from_raw(width, height, output)
        .ok_or_else(|| UnityError::Other("Failed to create BC4 image".to_string()))?;

    Ok(buffer)
}

/// Decode BC5 format (4x4 blocks, 128 bits per block, two channels RG)
///
/// # Python Equivalent
/// Python line 568: (TF.BC5, pillow, "RGB", "bcn", 5)
fn decode_bc5(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    let block_size = get_block_size(TextureFormat::BC5).unwrap_or((4, 4));
    let (padded_width, padded_height) =
        get_padded_dimensions(width, height, block_size.0, block_size.1);

    // BC5 decompresses to 2 bytes per pixel (RG)
    let mut rg_data = vec![0u8; (padded_width * padded_height * 2) as usize];

    Format::Bc5.decompress(
        data,
        padded_width as usize,
        padded_height as usize,
        &mut rg_data,
    );

    // Convert RG to RGBA: (R, G, 0, 255)
    let mut rgba_data = Vec::with_capacity((padded_width * padded_height * 4) as usize);
    for chunk in rg_data.chunks_exact(2) {
        rgba_data.push(chunk[0]); // R
        rgba_data.push(chunk[1]); // G
        rgba_data.push(0); // B (set to 0, could reconstruct for normal maps)
        rgba_data.push(255); // A
    }

    // Crop if needed
    let output = if padded_width != width || padded_height != height {
        let mut cropped = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            let src_offset = (y * padded_width * 4) as usize;
            let dst_offset = (y * width * 4) as usize;
            let row_bytes = (width * 4) as usize;
            cropped[dst_offset..dst_offset + row_bytes]
                .copy_from_slice(&rgba_data[src_offset..src_offset + row_bytes]);
        }
        cropped
    } else {
        rgba_data
    };

    let buffer = RgbaImage::from_raw(width, height, output)
        .ok_or_else(|| UnityError::Other("Failed to create BC5 image".to_string()))?;

    Ok(buffer)
}

/// Decode BC6H format (HDR compression)
///
/// BC6H uses 4x4 pixel blocks (16 bytes per block) and outputs RGB float32.
/// This is designed for HDR textures and has no alpha channel.
/// We convert the float HDR values to LDR u8 for standard image output.
///
/// # Python Equivalent
/// Python line 569: (TF.BC6H, pillow, "RGBA", "bcn", 6)
fn decode_bc6h(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    // Get padded dimensions for block compression (4x4 blocks)
    let block_size = get_block_size(TextureFormat::BC6H).unwrap_or((4, 4));
    let (padded_width, padded_height) =
        get_padded_dimensions(width, height, block_size.0, block_size.1);

    // BC6H: 4x4 blocks, 16 bytes per block
    let blocks_x = (padded_width + 3) / 4;
    let blocks_y = (padded_height + 3) / 4;
    let total_blocks = (blocks_x * blocks_y) as usize;

    // Verify we have enough data
    let expected_size = total_blocks * 16;
    if data.len() < expected_size {
        return Err(UnityError::Other(format!(
            "BC6H data too small: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    // Create output buffer for RGBA data (4 bytes per pixel)
    let mut rgba_data = vec![0u8; (padded_width * padded_height * 4) as usize];

    // Decompress each 4x4 block
    for block_y in 0..blocks_y {
        for block_x in 0..blocks_x {
            let block_index = (block_y * blocks_x + block_x) as usize;
            let block_data = &data[block_index * 16..(block_index + 1) * 16];

            // Decompress this block (outputs 16 pixels = 48 floats RGB)
            let mut decompressed_floats = [0f32; 48]; // 16 pixels * 3 floats (RGB)
            bcdec_rs::bc6h_float(block_data, &mut decompressed_floats, 12, false); // pitch = 4 pixels * 3 floats = 12 floats per row

            // Convert float RGB to u8 RGBA
            for pixel_y in 0..4 {
                for pixel_x in 0..4 {
                    let out_x = block_x * 4 + pixel_x;
                    let out_y = block_y * 4 + pixel_y;

                    // Skip if outside padded dimensions
                    if out_x >= padded_width || out_y >= padded_height {
                        continue;
                    }

                    let pixel_index = (pixel_y * 4 + pixel_x) as usize;
                    let src_offset = pixel_index * 3; // 3 floats per pixel (RGB)
                    let dst_offset = ((out_y * padded_width + out_x) * 4) as usize;

                    // Convert HDR float to LDR u8: multiply by 256, clamp to [0, 255]
                    let r = (decompressed_floats[src_offset] * 256.0).clamp(0.0, 255.0) as u8;
                    let g = (decompressed_floats[src_offset + 1] * 256.0).clamp(0.0, 255.0) as u8;
                    let b = (decompressed_floats[src_offset + 2] * 256.0).clamp(0.0, 255.0) as u8;

                    rgba_data[dst_offset] = r;
                    rgba_data[dst_offset + 1] = g;
                    rgba_data[dst_offset + 2] = b;
                    rgba_data[dst_offset + 3] = 255; // Alpha (BC6H has no alpha)
                }
            }
        }
    }

    // Crop if needed
    let output = if padded_width != width || padded_height != height {
        let mut cropped = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            let src_offset = (y * padded_width * 4) as usize;
            let dst_offset = (y * width * 4) as usize;
            let row_bytes = (width * 4) as usize;
            cropped[dst_offset..dst_offset + row_bytes]
                .copy_from_slice(&rgba_data[src_offset..src_offset + row_bytes]);
        }
        cropped
    } else {
        rgba_data
    };

    let buffer = RgbaImage::from_raw(width, height, output)
        .ok_or_else(|| UnityError::Other("Failed to create BC6H image".to_string()))?;

    Ok(buffer)
}

/// Decode BC7 format (highest quality block compression)
///
/// BC7 uses 4x4 pixel blocks (16 bytes per block) and outputs RGBA8.
/// It's the highest quality block compression format, replacing DXT5 in modern games.
///
/// # Python Equivalent
/// Python line 570: (TF.BC7, pillow, "RGBA", "bcn", 7)
fn decode_bc7(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    // Get padded dimensions for block compression (4x4 blocks)
    let block_size = get_block_size(TextureFormat::BC7).unwrap_or((4, 4));
    let (padded_width, padded_height) =
        get_padded_dimensions(width, height, block_size.0, block_size.1);

    // BC7: 4x4 blocks, 16 bytes per block
    let blocks_x = (padded_width + 3) / 4;
    let blocks_y = (padded_height + 3) / 4;
    let total_blocks = (blocks_x * blocks_y) as usize;

    // Verify we have enough data
    let expected_size = total_blocks * 16;
    if data.len() < expected_size {
        return Err(UnityError::Other(format!(
            "BC7 data too small: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    // Create output buffer for RGBA data (4 bytes per pixel)
    let mut rgba_data = vec![0u8; (padded_width * padded_height * 4) as usize];

    // Decompress each 4x4 block
    for block_y in 0..blocks_y {
        for block_x in 0..blocks_x {
            let block_index = (block_y * blocks_x + block_x) as usize;
            let block_data = &data[block_index * 16..(block_index + 1) * 16];

            // Decompress this block (outputs 16 pixels = 64 bytes RGBA)
            let mut decompressed_block = [0u8; 64];
            bcdec_rs::bc7(block_data, &mut decompressed_block, 16); // pitch = 4 pixels * 4 bytes = 16 bytes per row

            // Copy decompressed block to output buffer
            for pixel_y in 0..4 {
                for pixel_x in 0..4 {
                    let out_x = block_x * 4 + pixel_x;
                    let out_y = block_y * 4 + pixel_y;

                    // Skip if outside padded dimensions
                    if out_x >= padded_width || out_y >= padded_height {
                        continue;
                    }

                    let src_offset = ((pixel_y * 4 + pixel_x) * 4) as usize;
                    let dst_offset = ((out_y * padded_width + out_x) * 4) as usize;

                    rgba_data[dst_offset..dst_offset + 4]
                        .copy_from_slice(&decompressed_block[src_offset..src_offset + 4]);
                }
            }
        }
    }

    // Crop if needed
    let output = if padded_width != width || padded_height != height {
        let mut cropped = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            let src_offset = (y * padded_width * 4) as usize;
            let dst_offset = (y * width * 4) as usize;
            let row_bytes = (width * 4) as usize;
            cropped[dst_offset..dst_offset + row_bytes]
                .copy_from_slice(&rgba_data[src_offset..src_offset + row_bytes]);
        }
        cropped
    } else {
        rgba_data
    };

    let buffer = RgbaImage::from_raw(width, height, output)
        .ok_or_else(|| UnityError::Other("Failed to create BC7 image".to_string()))?;

    Ok(buffer)
}

// ============================================================================
// Advanced Format Decoders (ASTC, ETC, PVRTC)
// ============================================================================

/// Decode ASTC format (Adaptive Scalable Texture Compression)
///
/// ASTC supports many block sizes: 4x4, 5x5, 6x6, 8x8, 10x10, 12x12
///
/// # Python Equivalent
/// Python lines 404-416 (astc function)
/// Python lines 587-608 (CONV_TABLE entries)
///
// ============================================================================
// Helper Functions
// ============================================================================

/// Swap bytes in pairs for Xbox 360 textures
///
/// Xbox 360 uses big-endian byte order for certain texture formats
///
/// # Python Equivalent
/// Python lines 366-372
fn swap_bytes_for_xbox(data: &mut [u8]) {
    for chunk in data.chunks_exact_mut(2) {
        chunk.swap(0, 1);
    }
}

/// Convert BGRA32 (u32) format from texture2ddecoder to RGBA (Vec<u8>)
///
/// texture2ddecoder outputs colors as u32 in BGRA format.
/// We need to convert to RGBA byte array for RgbaImage.
fn bgra32_to_rgba(bgra_data: &[u32], width: u32, height: u32) -> Vec<u8> {
    let pixel_count = (width * height) as usize;
    let mut rgba_data = Vec::with_capacity(pixel_count * 4);

    for &bgra in bgra_data.iter().take(pixel_count) {
        let b = (bgra & 0xFF) as u8;
        let g = ((bgra >> 8) & 0xFF) as u8;
        let r = ((bgra >> 16) & 0xFF) as u8;
        let a = ((bgra >> 24) & 0xFF) as u8;

        rgba_data.push(r);
        rgba_data.push(g);
        rgba_data.push(b);
        rgba_data.push(a);
    }

    rgba_data
}

/// Generic decoder for block-compressed formats using texture2ddecoder
///
/// Handles padding, decoding, color conversion, and cropping in one function.
///
/// # Arguments
/// * `data` - Compressed texture data
/// * `width` - Original texture width
/// * `height` - Original texture height
/// * `block_width` - Block width (e.g., 4 for 4x4 blocks)
/// * `block_height` - Block height (e.g., 4 for 4x4 blocks)
/// * `decode_fn` - Function that decodes the compressed data (takes mutable buffer)
fn decode_with_blocks<F>(
    data: &[u8],
    width: u32,
    height: u32,
    block_width: u32,
    block_height: u32,
    decode_fn: F,
) -> UnityResult<RgbaImage>
where
    F: FnOnce(&[u8], usize, usize, &mut [u32]) -> Result<(), &'static str>,
{
    // Calculate padded dimensions
    let (padded_width, padded_height) =
        get_padded_dimensions(width, height, block_width, block_height);

    // Allocate BGRA32 output buffer
    let pixel_count = (padded_width * padded_height) as usize;
    let mut bgra_data = vec![0u32; pixel_count];

    // Decode using texture2ddecoder (writes to mutable buffer)
    decode_fn(
        data,
        padded_width as usize,
        padded_height as usize,
        &mut bgra_data,
    )
    .map_err(|e| UnityError::Other(format!("Texture decode error: {}", e)))?;

    // Convert BGRA32 to RGBA
    let rgba_data = bgra32_to_rgba(&bgra_data, padded_width, padded_height);

    // Create image from padded data
    let mut img = RgbaImage::from_raw(padded_width, padded_height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create image from decoded data".to_string()))?;

    // Crop to original dimensions if needed
    if padded_width != width || padded_height != height {
        img = imageops::crop(&mut img, 0, 0, width, height).to_image();
    }

    Ok(img)
}

/// Decode Unity crunch-compressed texture data
///
/// Unity uses crunch compression as an additional layer over DXT/ETC formats.
/// This function handles both old crunch and Unity's custom crunch format.
/// Unlike other decoders, crunch decoders directly output BGRA32.
///
/// # Arguments
/// * `data` - Crunch-compressed data
/// * `width` - Texture width
/// * `height` - Texture height
/// * `version` - Unity version (major, minor, patch, type)
///
/// # Python Equivalent
/// Python lines 337-347
fn decode_crunch(
    data: &[u8],
    width: u32,
    height: u32,
    version: (u32, u32, u32, u32),
) -> UnityResult<RgbaImage> {
    // Unity 2017.3+ uses unity_crunch format
    let use_unity_crunch = version.0 > 2017 || (version.0 == 2017 && version.1 >= 3);

    // Allocate BGRA32 output buffer
    let pixel_count = (width * height) as usize;
    let mut bgra_data = vec![0u32; pixel_count];

    // Decode based on version
    let result = if use_unity_crunch {
        texture2ddecoder::decode_unity_crunch(data, width as usize, height as usize, &mut bgra_data)
    } else {
        texture2ddecoder::decode_crunch(data, width as usize, height as usize, &mut bgra_data)
    };

    result.map_err(|e| UnityError::Other(format!("Crunch decompression failed: {}", e)))?;

    // Convert BGRA32 to RGBA
    let rgba_data = bgra32_to_rgba(&bgra_data, width, height);

    // Create image
    RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create crunch image".to_string()))
}

/// Calculate padded dimensions for compressed texture formats
///
/// Compressed formats work on blocks (e.g., 4x4 pixels)
/// Dimensions must be padded to block boundaries
///
/// # Python Equivalent
/// Python lines 36-50 (get_compressed_image_size)
fn get_padded_dimensions(
    width: u32,
    height: u32,
    block_width: u32,
    block_height: u32,
) -> (u32, u32) {
    let padded_width = ((width + block_width - 1) / block_width) * block_width;
    let padded_height = ((height + block_height - 1) / block_height) * block_height;
    (padded_width, padded_height)
}

/// Get the block size for a given texture format
///
/// Returns (block_width, block_height) or None for non-compressed formats
///
/// # Python Equivalent
/// Python lines 22-33 (TEXTURE_FORMAT_BLOCK_SIZE_TABLE initialization)
fn get_block_size(format: TextureFormat) -> Option<(u32, u32)> {
    let format_name = format!("{:?}", format);
    if format_name.starts_with("ASTC") {
        // Python lines 24-26: Parse block size from name
        // "ASTC_RGB_4x4" → split by "_" → last part "4x4" → split by "x" → (4, 4)
        if let Some(last_part) = format_name.rsplit('_').next() {
            if let Some((width_str, height_str)) = last_part.split_once('x') {
                if let (Ok(w), Ok(h)) = (width_str.parse::<u32>(), height_str.parse::<u32>()) {
                    return Some((w, h));
                }
            }
        }
        // Fallback for ASTC formats without proper name
        return Some((4, 4));
    } else if format_name.starts_with("DXT")
        || format_name.starts_with("BC")
        || format_name.starts_with("ETC")
        || format_name.starts_with("EAC")
        || format_name.starts_with("ATC")
    {
        // Python line 27-28: DXT/BC/ETC/EAC/ATC all use 4x4 blocks
        return Some((4, 4));
    } else if format_name.starts_with("PVRTC") {
        // Python lines 29-30: PVRTC is 8x4 if ends with "2", else 4x4
        let block_width = if format_name.ends_with('2') { 8 } else { 4 };
        return Some((block_width, 4));
    }

    // Python line 32: Everything else has no blocks
    None
}

// ============================================================================
// Additional Format Decoders (Generated)
// ============================================================================

/// Decode ETC_RGB4 format (ETC/EAC (Ericsson Texture Compression))
///
/// # Complexity: HARD
///
/// # Python Equivalent
/// Python line 571: (TF.ETC_RGB4, etc, "RGB", ...)
///
/// # Implementation Notes
/// ETC is a block-based compression format commonly used on mobile.
///
/// Options for implementation:
/// 1. Use etc-rs crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. Port algorithm from reference implementation
fn decode_etc_rgb4(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 4, 4, |d, w, h, buf| {
        texture2ddecoder::decode_etc1(d, w, h, buf)
    })
}

/// Decode ETC2_RGB format (ETC/EAC (Ericsson Texture Compression))
///
/// # Complexity: HARD
///
/// # Python Equivalent
/// Python line 580: (TF.ETC2_RGB, etc, "RGB", ...)
///
/// # Implementation Notes
/// ETC is a block-based compression format commonly used on mobile.
///
/// Options for implementation:
/// 1. Use etc-rs crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. Port algorithm from reference implementation
fn decode_etc2_rgb(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 4, 4, |d, w, h, buf| {
        texture2ddecoder::decode_etc2_rgb(d, w, h, buf)
    })
}

/// Decode ETC2_RGBA1 format (ETC/EAC (Ericsson Texture Compression))
///
/// # Complexity: HARD
///
/// # Python Equivalent
/// Python line 581: (TF.ETC2_RGBA1, etc, "RGBA", ...)
///
/// # Implementation Notes
/// ETC is a block-based compression format commonly used on mobile.
///
/// Options for implementation:
/// 1. Use etc-rs crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. Port algorithm from reference implementation
fn decode_etc2_rgba1(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 4, 4, |d, w, h, buf| {
        texture2ddecoder::decode_etc2_rgba1(d, w, h, buf)
    })
}

/// Decode ETC2_RGBA8 format (ETC/EAC (Ericsson Texture Compression))
///
/// # Complexity: HARD
///
/// # Python Equivalent
/// Python line 582: (TF.ETC2_RGBA8, etc, "RGBA", ...)
///
/// # Implementation Notes
/// ETC is a block-based compression format commonly used on mobile.
///
/// Options for implementation:
/// 1. Use etc-rs crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. Port algorithm from reference implementation
fn decode_etc2_rgba8(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 4, 4, |d, w, h, buf| {
        texture2ddecoder::decode_etc2_rgba8(d, w, h, buf)
    })
}

/// Decode EAC_R format (ETC/EAC (Ericsson Texture Compression))
///
/// # Complexity: HARD
///
/// # Python Equivalent
/// Python line 595: (TF.EAC_R, etc, "R", ...)
///
/// # Implementation Notes
/// ETC is a block-based compression format commonly used on mobile.
///
/// Options for implementation:
/// 1. Use etc-rs crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. Port algorithm from reference implementation
fn decode_eacr(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 4, 4, |d, w, h, buf| {
        texture2ddecoder::decode_eacr(d, w, h, buf)
    })
}

/// Decode EAC_R_SIGNED format (ETC/EAC (Ericsson Texture Compression))
///
/// # Complexity: HARD
///
/// # Python Equivalent
/// Python line 596: (TF.EAC_R_SIGNED, etc, "R", ...)
///
/// # Implementation Notes
/// ETC is a block-based compression format commonly used on mobile.
///
/// Options for implementation:
/// 1. Use etc-rs crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. Port algorithm from reference implementation
fn decode_eacr_signed(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 4, 4, |d, w, h, buf| {
        texture2ddecoder::decode_eacr_signed(d, w, h, buf)
    })
}

/// Decode EAC_RG format (ETC/EAC (Ericsson Texture Compression))
///
/// # Complexity: HARD
///
/// # Python Equivalent
/// Python line 597: (TF.EAC_RG, etc, "RGB", ...)
///
/// # Implementation Notes
/// ETC is a block-based compression format commonly used on mobile.
///
/// Options for implementation:
/// 1. Use etc-rs crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. Port algorithm from reference implementation
fn decode_eacrg(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 4, 4, |d, w, h, buf| {
        texture2ddecoder::decode_eacrg(d, w, h, buf)
    })
}

/// Decode EAC_RG_SIGNED format (ETC/EAC (Ericsson Texture Compression))
///
/// # Complexity: HARD
///
/// # Python Equivalent
/// Python line 598: (TF.EAC_RG_SIGNED, etc, "RGB", ...)
///
/// # Implementation Notes
/// ETC is a block-based compression format commonly used on mobile.
///
/// Options for implementation:
/// 1. Use etc-rs crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. Port algorithm from reference implementation
fn decode_eacrg_signed(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 4, 4, |d, w, h, buf| {
        texture2ddecoder::decode_eacrg_signed(d, w, h, buf)
    })
}

/// Decode PVRTC_RGB2 format (PVRTC (PowerVR Texture Compression))
///
/// # Complexity: HARD
///
/// # Python Equivalent
/// Python line 572: (TF.PVRTC_RGB2, pvrtc, "RGB", ...)
///
/// # Implementation Notes
/// PVRTC is iOS/PowerVR specific texture compression.
/// It is quite complex and proprietary.
///
/// Options for implementation:
/// 1. PyO3 bridge to Python PIL
/// 2. FFI to PVRTexLib
/// 3. Skip (low priority - iOS specific)
fn decode_pvrtc_rgb2(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    // PVRTC requires full texture decode (no block-level)
    let pixel_count = (width * height) as usize;
    let mut bgra_data = vec![0u32; pixel_count];

    texture2ddecoder::decode_pvrtc_2bpp(data, width as usize, height as usize, &mut bgra_data)
        .map_err(|e| UnityError::Other(format!("PVRTC decode error: {}", e)))?;

    let rgba_data = bgra32_to_rgba(&bgra_data, width, height);
    RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create PVRTC image".to_string()))
}

/// Decode PVRTC_RGBA2 format (PVRTC (PowerVR Texture Compression))
///
/// # Complexity: HARD
///
/// # Python Equivalent
/// Python line 573: (TF.PVRTC_RGBA2, pvrtc, "RGBA", ...)
///
/// # Implementation Notes
/// PVRTC is iOS/PowerVR specific texture compression.
/// It is quite complex and proprietary.
///
/// Options for implementation:
/// 1. PyO3 bridge to Python PIL
/// 2. FFI to PVRTexLib
/// 3. Skip (low priority - iOS specific)
fn decode_pvrtc_rgba2(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    // PVRTC requires full texture decode (no block-level)
    let pixel_count = (width * height) as usize;
    let mut bgra_data = vec![0u32; pixel_count];

    texture2ddecoder::decode_pvrtc_2bpp(data, width as usize, height as usize, &mut bgra_data)
        .map_err(|e| UnityError::Other(format!("PVRTC decode error: {}", e)))?;

    let rgba_data = bgra32_to_rgba(&bgra_data, width, height);
    RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create PVRTC image".to_string()))
}

/// Decode PVRTC_RGB4 format (PVRTC (PowerVR Texture Compression))
///
/// # Complexity: HARD
///
/// # Python Equivalent
/// Python line 574: (TF.PVRTC_RGB4, pvrtc, "RGB", ...)
///
/// # Implementation Notes
/// PVRTC is iOS/PowerVR specific texture compression.
/// It is quite complex and proprietary.
///
/// Options for implementation:
/// 1. PyO3 bridge to Python PIL
/// 2. FFI to PVRTexLib
/// 3. Skip (low priority - iOS specific)
fn decode_pvrtc_rgb4(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    // PVRTC requires full texture decode (no block-level)
    let pixel_count = (width * height) as usize;
    let mut bgra_data = vec![0u32; pixel_count];

    texture2ddecoder::decode_pvrtc_4bpp(data, width as usize, height as usize, &mut bgra_data)
        .map_err(|e| UnityError::Other(format!("PVRTC decode error: {}", e)))?;

    let rgba_data = bgra32_to_rgba(&bgra_data, width, height);
    RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create PVRTC image".to_string()))
}

/// Decode PVRTC_RGBA4 format (PVRTC (PowerVR Texture Compression))
///
/// # Complexity: HARD
///
/// # Python Equivalent
/// Python line 575: (TF.PVRTC_RGBA4, pvrtc, "RGBA", ...)
///
/// # Implementation Notes
/// PVRTC is iOS/PowerVR specific texture compression.
/// It is quite complex and proprietary.
///
/// Options for implementation:
/// 1. PyO3 bridge to Python PIL
/// 2. FFI to PVRTexLib
/// 3. Skip (low priority - iOS specific)
fn decode_pvrtc_rgba4(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    // PVRTC requires full texture decode (no block-level)
    let pixel_count = (width * height) as usize;
    let mut bgra_data = vec![0u32; pixel_count];

    texture2ddecoder::decode_pvrtc_4bpp(data, width as usize, height as usize, &mut bgra_data)
        .map_err(|e| UnityError::Other(format!("PVRTC decode error: {}", e)))?;

    let rgba_data = bgra32_to_rgba(&bgra_data, width, height);
    RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create PVRTC image".to_string()))
}

/// Decode ASTC_RGB_4x4 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 576: (TF.ASTC_RGB_4x4, astc, "RGB", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_rgb_4x4(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 4, 4, |d, w, h, buf| {
        texture2ddecoder::decode_astc_4_4(d, w, h, buf)
    })
}

/// Decode ASTC_RGB_5x5 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 577: (TF.ASTC_RGB_5x5, astc, "RGB", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_rgb_5x5(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 5, 5, |d, w, h, buf| {
        texture2ddecoder::decode_astc_5_5(d, w, h, buf)
    })
}

/// Decode ASTC_RGB_6x6 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 578: (TF.ASTC_RGB_6x6, astc, "RGB", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_rgb_6x6(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 6, 6, |d, w, h, buf| {
        texture2ddecoder::decode_astc_6_6(d, w, h, buf)
    })
}

/// Decode ASTC_RGB_8x8 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 579: (TF.ASTC_RGB_8x8, astc, "RGB", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_rgb_8x8(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 8, 8, |d, w, h, buf| {
        texture2ddecoder::decode_astc_8_8(d, w, h, buf)
    })
}

/// Decode ASTC_RGB_10x10 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 583: (TF.ASTC_RGB_10x10, astc, "RGB", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_rgb_10x10(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 10, 10, |d, w, h, buf| {
        texture2ddecoder::decode_astc_10_10(d, w, h, buf)
    })
}

/// Decode ASTC_RGB_12x12 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 584: (TF.ASTC_RGB_12x12, astc, "RGB", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_rgb_12x12(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 12, 12, |d, w, h, buf| {
        texture2ddecoder::decode_astc_12_12(d, w, h, buf)
    })
}

/// Decode ASTC_RGBA_4x4 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 585: (TF.ASTC_RGBA_4x4, astc, "RGBA", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_rgba_4x4(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 4, 4, |d, w, h, buf| {
        texture2ddecoder::decode_astc_4_4(d, w, h, buf)
    })
}

/// Decode ASTC_RGBA_5x5 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 586: (TF.ASTC_RGBA_5x5, astc, "RGBA", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_rgba_5x5(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 5, 5, |d, w, h, buf| {
        texture2ddecoder::decode_astc_5_5(d, w, h, buf)
    })
}

/// Decode ASTC_RGBA_6x6 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 587: (TF.ASTC_RGBA_6x6, astc, "RGBA", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_rgba_6x6(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 6, 6, |d, w, h, buf| {
        texture2ddecoder::decode_astc_6_6(d, w, h, buf)
    })
}

/// Decode ASTC_RGBA_8x8 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 588: (TF.ASTC_RGBA_8x8, astc, "RGBA", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_rgba_8x8(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 8, 8, |d, w, h, buf| {
        texture2ddecoder::decode_astc_8_8(d, w, h, buf)
    })
}

/// Decode ASTC_RGBA_10x10 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 589: (TF.ASTC_RGBA_10x10, astc, "RGBA", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_rgba_10x10(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 10, 10, |d, w, h, buf| {
        texture2ddecoder::decode_astc_10_10(d, w, h, buf)
    })
}

/// Decode ASTC_RGBA_12x12 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 590: (TF.ASTC_RGBA_12x12, astc, "RGBA", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_rgba_12x12(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 12, 12, |d, w, h, buf| {
        texture2ddecoder::decode_astc_12_12(d, w, h, buf)
    })
}

/// Decode ASTC_HDR_4x4 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 620: (TF.ASTC_HDR_4x4, astc, "RGB", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_hdr_4x4(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 4, 4, |d, w, h, buf| {
        texture2ddecoder::decode_astc_4_4(d, w, h, buf)
    })
}

/// Decode ASTC_HDR_5x5 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 621: (TF.ASTC_HDR_5x5, astc, "RGB", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_hdr_5x5(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 5, 5, |d, w, h, buf| {
        texture2ddecoder::decode_astc_5_5(d, w, h, buf)
    })
}

/// Decode ASTC_HDR_6x6 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 622: (TF.ASTC_HDR_6x6, astc, "RGB", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_hdr_6x6(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 6, 6, |d, w, h, buf| {
        texture2ddecoder::decode_astc_6_6(d, w, h, buf)
    })
}

/// Decode ASTC_HDR_8x8 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 623: (TF.ASTC_HDR_8x8, astc, "RGB", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_hdr_8x8(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 8, 8, |d, w, h, buf| {
        texture2ddecoder::decode_astc_8_8(d, w, h, buf)
    })
}

/// Decode ASTC_HDR_10x10 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 624: (TF.ASTC_HDR_10x10, astc, "RGB", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_hdr_10x10(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 10, 10, |d, w, h, buf| {
        texture2ddecoder::decode_astc_10_10(d, w, h, buf)
    })
}

/// Decode ASTC_HDR_12x12 format (ASTC (Adaptive Scalable Texture Compression))
///
/// # Complexity: VERY HARD
///
/// # Python Equivalent
/// Python line 625: (TF.ASTC_HDR_12x12, astc, "RGB", ...)
///
/// # Implementation Notes
/// This is one of the most complex texture compression algorithms.
/// It supports variable block sizes and multiple compression modes.
///
/// Options for implementation:
/// 1. Use astc-encoder crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. FFI to C++ astc-encoder library
/// 4. Port decompression algorithm from reference implementation
fn decode_astc_hdr_12x12(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 12, 12, |d, w, h, buf| {
        texture2ddecoder::decode_astc_12_12(d, w, h, buf)
    })
}

/// Decode YUY2 format (YUY2 (YUV 4:2:2))
///
/// # Complexity: MEDIUM
///
/// # Python Equivalent
/// Python line 599: (TF.YUY2, yuy2, "RGB", ...)
///
/// # Implementation Notes
/// YUY2 is a YUV color space format with chroma subsampling.
/// Each 4 bytes encodes 2 pixels: Y0 U Y1 V
///
/// Conversion: YUV -> RGB
/// R = Y + 1.402 * (V - 128)
/// G = Y - 0.344 * (U - 128) - 0.714 * (V - 128)
/// B = Y + 1.772 * (U - 128)
fn decode_yuy2(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    // YUY2 is packed YUV 4:2:2 format where each 4 bytes encodes 2 pixels
    // Layout: Y0 U Y1 V (shares U and V between 2 adjacent pixels)
    let expected_size = (width * height * 2) as usize; // 2 bytes per pixel
    if data.len() != expected_size {
        return Err(UnityError::Other(format!(
            "Invalid YUY2 data size: expected {} bytes, got {}",
            expected_size,
            data.len()
        )));
    }

    let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

    // Process 2 pixels at a time (4 bytes: Y0 U Y1 V)
    for chunk in data.chunks_exact(4) {
        let y0 = chunk[0] as f32;
        let u = chunk[1] as f32;
        let y1 = chunk[2] as f32;
        let v = chunk[3] as f32;

        // Convert YUV to RGB for both pixels
        // Standard BT.601 conversion:
        // R = Y + 1.402 * (V - 128)
        // G = Y - 0.344 * (U - 128) - 0.714 * (V - 128)
        // B = Y + 1.772 * (U - 128)

        let u_offset = u - 128.0;
        let v_offset = v - 128.0;

        // First pixel
        let r0 = (y0 + 1.402 * v_offset).clamp(0.0, 255.0) as u8;
        let g0 = (y0 - 0.344 * u_offset - 0.714 * v_offset).clamp(0.0, 255.0) as u8;
        let b0 = (y0 + 1.772 * u_offset).clamp(0.0, 255.0) as u8;

        rgba_data.push(r0);
        rgba_data.push(g0);
        rgba_data.push(b0);
        rgba_data.push(255); // Alpha

        // Second pixel
        let r1 = (y1 + 1.402 * v_offset).clamp(0.0, 255.0) as u8;
        let g1 = (y1 - 0.344 * u_offset - 0.714 * v_offset).clamp(0.0, 255.0) as u8;
        let b1 = (y1 + 1.772 * u_offset).clamp(0.0, 255.0) as u8;

        rgba_data.push(r1);
        rgba_data.push(g1);
        rgba_data.push(b1);
        rgba_data.push(255); // Alpha
    }

    let buffer = RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| UnityError::Other("Failed to create YUY2 image".to_string()))?;

    Ok(buffer)
}

/// Decode ETC_RGB4_3DS format (ETC/EAC (Ericsson Texture Compression))
///
/// # Complexity: HARD
///
/// # Python Equivalent
/// Python line 604: (TF.ETC_RGB4_3DS, etc, "RGB", ...)
///
/// # Implementation Notes
/// ETC is a block-based compression format commonly used on mobile.
///
/// Options for implementation:
/// 1. Use etc-rs crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. Port algorithm from reference implementation
fn decode_etc_rgb4_3ds(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 4, 4, |d, w, h, buf| {
        texture2ddecoder::decode_etc1(d, w, h, buf)
    })
}

/// Decode ETC_RGBA8_3DS format (ETC/EAC (Ericsson Texture Compression))
///
/// # Complexity: HARD
///
/// # Python Equivalent
/// Python line 605: (TF.ETC_RGBA8_3DS, etc, "RGBA", ...)
///
/// # Implementation Notes
/// ETC is a block-based compression format commonly used on mobile.
///
/// Options for implementation:
/// 1. Use etc-rs crate (if exists)
/// 2. PyO3 bridge to Python PIL
/// 3. Port algorithm from reference implementation
fn decode_etc_rgba8_3ds(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    // ETC1 with alpha - use ETC1 for RGB, keep alpha separate
    decode_with_blocks(data, width, height, 4, 4, |d, w, h, buf| {
        texture2ddecoder::decode_etc1(d, w, h, buf)
    })
}

/// Decode ATC_RGB4 format (ATC (AMD/Qualcomm Texture Compression))
///
/// # Complexity: MEDIUM
///
/// # Python Equivalent
/// Python line 606: (TF.ATC_RGB4, atc, "RGB", ...)
///
/// # Implementation Notes
/// ATC is Qualcomm Adreno GPU texture compression.
/// Similar to DXT but with different algorithms.
///
/// Rarely used - low priority.
fn decode_atc_rgb4(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 4, 4, |d, w, h, buf| {
        texture2ddecoder::decode_atc_rgb4(d, w, h, buf)
    })
}

/// Decode ATC_RGBA8 format (ATC (AMD/Qualcomm Texture Compression))
///
/// # Complexity: MEDIUM
///
/// # Python Equivalent
/// Python line 607: (TF.ATC_RGBA8, atc, "RGBA", ...)
///
/// # Implementation Notes
/// ATC is Qualcomm Adreno GPU texture compression.
/// Similar to DXT but with different algorithms.
///
/// Rarely used - low priority.
fn decode_atc_rgba8(data: &[u8], width: u32, height: u32) -> UnityResult<RgbaImage> {
    decode_with_blocks(data, width, height, 4, 4, |d, w, h, buf| {
        texture2ddecoder::decode_atc_rgba8(d, w, h, buf)
    })
}
