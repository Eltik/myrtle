//! Nintendo Switch texture deswizzling
//!
//! Based on: https://github.com/nesrak1/UABEA/blob/master/TexturePlugin/Texture2DSwitchDeswizzler.cs
//!
//! Handles conversion between swizzled (GOB-based GPU layout) and linear texture formats
//! for Nintendo Switch platform textures.

use crate::enums::texture_format::TextureFormat;

// GOB (Group of Blocks) layout constants
const GOB_X_TEXEL_COUNT: usize = 4;
const GOB_Y_TEXEL_COUNT: usize = 8;
const TEXEL_BYTE_SIZE: usize = 16;
#[allow(dead_code)] // Reference constant
const TEXELS_IN_GOB: usize = GOB_X_TEXEL_COUNT * GOB_Y_TEXEL_COUNT; // 32

// Pre-calculated GOB coordinate mapping for 32 texels
// Python: [(((l >> 3) & 0b10) | ((l >> 1) & 0b1), ((l >> 1) & 0b110) | (l & 0b1)) for l in range(32)]
const GOB_MAP: [(usize, usize); 32] = [
    (0, 0),
    (0, 1),
    (1, 0),
    (1, 1),
    (0, 2),
    (0, 3),
    (1, 2),
    (1, 3),
    (0, 4),
    (0, 5),
    (1, 4),
    (1, 5),
    (0, 6),
    (0, 7),
    (1, 6),
    (1, 7),
    (2, 0),
    (2, 1),
    (3, 0),
    (3, 1),
    (2, 2),
    (2, 3),
    (3, 2),
    (3, 3),
    (2, 4),
    (2, 5),
    (3, 4),
    (3, 5),
    (2, 6),
    (2, 7),
    (3, 6),
    (3, 7),
];

/// Ceiling division - divides a by b and rounds up
/// Python: return (a + b - 1) // b
pub fn ceil_divide(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

/// Texture format to block size mapping
/// Maps TextureFormat to (block_width, block_height) tuples
/// This represents the amount of pixels that can fit in 16 bytes
pub fn get_texture_format_block_size(format: TextureFormat) -> Option<(usize, usize)> {
    match format {
        TextureFormat::Alpha8 => Some((16, 1)),   // 1 byte per pixel
        TextureFormat::ARGB4444 => Some((8, 1)),  // 2 bytes per pixel
        TextureFormat::RGBA32 => Some((4, 1)),    // 4 bytes per pixel
        TextureFormat::ARGB32 => Some((4, 1)),    // 4 bytes per pixel
        TextureFormat::ARGBFloat => Some((1, 1)), // 16 bytes per pixel
        TextureFormat::RGB565 => Some((8, 1)),    // 2 bytes per pixel
        TextureFormat::R16 => Some((8, 1)),       // 2 bytes per pixel
        TextureFormat::DXT1 => Some((8, 4)),      // 8 bytes per 4x4=16 pixels
        TextureFormat::DXT5 => Some((4, 4)),      // 16 bytes per 4x4=16 pixels
        TextureFormat::RGBA4444 => Some((8, 1)),  // 2 bytes per pixel
        TextureFormat::BGRA32 => Some((4, 1)),    // 4 bytes per pixel
        TextureFormat::BC6H => Some((4, 4)),      // 16 bytes per 4x4=16 pixels
        TextureFormat::BC7 => Some((4, 4)),       // 16 bytes per 4x4=16 pixels
        TextureFormat::BC4 => Some((8, 4)),       // 8 bytes per 4x4=16 pixels
        TextureFormat::BC5 => Some((4, 4)),       // 16 bytes per 4x4=16 pixels
        TextureFormat::ASTC_RGB_4x4 => Some((4, 4)), // 16 bytes per 4x4=16 pixels
        TextureFormat::ASTC_RGB_5x5 => Some((5, 5)), // 16 bytes per 5x5=25 pixels
        TextureFormat::ASTC_RGB_6x6 => Some((6, 6)), // 16 bytes per 6x6=36 pixels
        TextureFormat::ASTC_RGB_8x8 => Some((8, 8)), // 16 bytes per 8x8=64 pixels
        TextureFormat::ASTC_RGB_10x10 => Some((10, 10)), // 16 bytes per 10x10=100 pixels
        TextureFormat::ASTC_RGB_12x12 => Some((12, 12)), // 16 bytes per 12x12=144 pixels
        TextureFormat::ASTC_RGBA_4x4 => Some((4, 4)), // 16 bytes per 4x4=16 pixels
        TextureFormat::ASTC_RGBA_5x5 => Some((5, 5)), // 16 bytes per 5x5=25 pixels
        TextureFormat::ASTC_RGBA_6x6 => Some((6, 6)), // 16 bytes per 6x6=36 pixels
        TextureFormat::ASTC_RGBA_8x8 => Some((8, 8)), // 16 bytes per 8x8=64 pixels
        TextureFormat::ASTC_RGBA_10x10 => Some((10, 10)), // 16 bytes per 10x10=100 pixels
        TextureFormat::ASTC_RGBA_12x12 => Some((12, 12)), // 16 bytes per 12x12=144 pixels
        TextureFormat::RG16 => Some((8, 1)),      // 2 bytes per pixel
        TextureFormat::R8 => Some((16, 1)),       // 1 byte per pixel
        _ => None,                                // Unsupported format
    }
}

pub fn deswizzle(
    data: &[u8],
    width: usize,
    height: usize,
    block_width: usize,
    block_height: usize,
    texels_per_block: usize,
) -> Vec<u8> {
    let block_count_x = ceil_divide(width, block_width);
    let block_count_y = ceil_divide(height, block_height);
    let gob_count_x = block_count_x / GOB_X_TEXEL_COUNT;
    let gob_count_y = block_count_y / GOB_Y_TEXEL_COUNT;
    let mut new_data = vec![0u8; data.len()];
    let mut src_index = 0;

    for i in 0..(gob_count_y / texels_per_block) {
        for j in 0..gob_count_x {
            let base_gob_dst_x = j * 4;
            for k in 0..texels_per_block {
                let base_gob_dst_y = (i * texels_per_block + k) * GOB_Y_TEXEL_COUNT;
                for (gob_x, gob_y) in GOB_MAP.iter() {
                    let dst_offset = ((base_gob_dst_y + gob_y) * block_count_x
                        + (base_gob_dst_x + gob_x))
                        * TEXEL_BYTE_SIZE;
                    new_data[dst_offset..dst_offset + TEXEL_BYTE_SIZE]
                        .copy_from_slice(&data[src_index..src_index + TEXEL_BYTE_SIZE]);
                    src_index += TEXEL_BYTE_SIZE;
                }
            }
        }
    }
    new_data
}

pub fn swizzle(
    data: &[u8],
    width: usize,
    height: usize,
    block_width: usize,
    block_height: usize,
    texels_per_block: usize,
) -> Vec<u8> {
    let block_count_x = ceil_divide(width, block_width);
    let block_count_y = ceil_divide(height, block_height);
    let gob_count_x = block_count_x / GOB_X_TEXEL_COUNT;
    let gob_count_y = block_count_y / GOB_Y_TEXEL_COUNT;
    let mut new_data = vec![0u8; data.len()];
    let mut dst_index = 0;

    for i in 0..(gob_count_y / texels_per_block) {
        for j in 0..gob_count_x {
            let base_gob_dst_x = j * 4;
            for k in 0..texels_per_block {
                let base_gob_dst_y = (i * texels_per_block + k) * GOB_Y_TEXEL_COUNT;
                for (gob_x, gob_y) in GOB_MAP.iter() {
                    let src_offset = ((base_gob_dst_y + gob_y) * block_count_x
                        + (base_gob_dst_x + gob_x))
                        * TEXEL_BYTE_SIZE;
                    new_data[dst_index..dst_index + TEXEL_BYTE_SIZE]
                        .copy_from_slice(&data[src_offset..src_offset + TEXEL_BYTE_SIZE]);
                    dst_index += TEXEL_BYTE_SIZE;
                }
            }
        }
    }
    new_data
}

pub fn get_padded_texture_size(
    width: usize,
    height: usize,
    block_width: usize,
    block_height: usize,
    texels_per_block: usize,
) -> (usize, usize) {
    let width =
        ceil_divide(width, block_width * GOB_X_TEXEL_COUNT) * block_width * GOB_X_TEXEL_COUNT;
    let height = ceil_divide(height, block_height * GOB_Y_TEXEL_COUNT * texels_per_block)
        * block_height
        * GOB_Y_TEXEL_COUNT
        * texels_per_block;
    (width, height)
}

pub fn get_switch_gobs_per_block(platform_blob: &[u8]) -> usize {
    let value = u32::from_le_bytes([
        platform_blob[8],
        platform_blob[9],
        platform_blob[10],
        platform_blob[11],
    ]);
    1 << value
}
