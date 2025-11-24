//! Compression Helper for Unity AssetBundles
//!
//! Provides compression/decompression functions for various formats used by Unity:
//! - LZMA (Unity-specific variant)
//! - LZ4/LZ4HC (high-speed compression)
//! - Brotli (Google's compression)
//! - GZIP (standard compression)
use brotli::{BrotliCompress, BrotliDecompress};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use lz4::block::{compress as lz4_compress, decompress as lz4_decompress};
use lzma_rs::lzma_compress;
use lzma_rs::lzma_decompress;
use std::io::Cursor;
use std::io::{self, Read, Write};

/// Type alias for byte data
pub type ByteString = Vec<u8>;

/// GZIP magic bytes (0x14, 0x8b)
pub const GZIP_MAGIC: &[u8] = b"\x1f\x8b";

/// Brotli magic bytes
pub const BROTLI_MAGIC: &[u8] = b"brotli";

/// Decompresses GZIP-compressed data
///
/// # Arguments
///
/// * `data` - Compressed data
///
/// # Returns
///
/// Decompressed data
///
/// # Errors
///
/// Returns an error if decompression fails
pub fn decompress_gzip(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

/// Compresses data using GZIP
///
/// # Arguments
///
/// * `data` - Uncompressed data
///
/// # Returns
///
/// Compressed data
///
/// # Errors
///
/// Returns an error if compression fails
pub fn compress_gzip(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

/// Decompresses Brotli-compressed data
///
/// # Arguments
///
/// * `data` - Compressed data
///
/// # Returns
///
/// Decompressed data
///
/// # Errors
///
/// Returns an error if decompression fails
pub fn decompress_brotli(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut decompressed = Vec::new();
    let mut cursor = io::Cursor::new(data);

    BrotliDecompress(&mut cursor, &mut decompressed)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(decompressed)
}

/// Compresses data using Brotli
///
/// # Arguments
///
/// * `data` - Uncompressed data
///
/// # Returns
///
/// Compressed data
///
/// # Errors
///
/// Returns an error if compression fails
pub fn compress_brotli(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut compressed = Vec::new();
    let mut cursor = io::Cursor::new(data);

    BrotliCompress(&mut cursor, &mut compressed, &Default::default())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(compressed)
}

/// Decompresses LZ4-compressed data (LZ4/LZ4HC)
///
/// # Arguments
///
/// * `data` - Compressed data
/// * `uncompressed_size` - Size of the uncompressed data (must be known in advance)
///
/// # Returns
///
/// Decompressed data
///
/// # Errors
///
/// Returns an error if decompression fails or output buffer is insufficient
pub fn decompress_lz4(data: &[u8], uncompressed_size: usize) -> io::Result<Vec<u8>> {
    lz4_decompress(data, Some(uncompressed_size as i32))
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Compresses data using LZ4 high compression
///
/// Uses high compression mode with level 9 (matching Python settings)
///
/// # Arguments
///
/// * `data` - Uncompressed data
///
/// # Returns
///
/// Compressed data
///
/// # Errors
///
/// Returns an error if compression fails
pub fn compress_lz4(data: &[u8]) -> io::Result<Vec<u8>> {
    // Python uses: mode="high_compression", compression=9, store_size=False
    let compressed = lz4_compress(
        data,
        Some(lz4::block::CompressionMode::HIGHCOMPRESSION(9)),
        false,
    )
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(compressed)
}

/// Decompresses LZMA-compressed data (Unity-specific format)
///
/// Unity uses raw LZMA1 with a custom 5 or 13-byte header
///
/// # Arguments
///
/// * `data` - Compressed data with Unity LZMA header
/// * `read_decompressed_size` - Whether header includes 8-byte size field
///
/// # Returns
///
/// Decompressed data
///
/// # Errors
///
/// Returns an error if decompression fails or header is invalid
pub fn decompress_lzma(data: &[u8], read_decompressed_size: bool) -> io::Result<Vec<u8>> {
    if data.len() < 5 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "LZMA data too short for header",
        ));
    }

    // Parse Unity LZMA header (first 5 bytes)
    let props = data[0];
    let dict_size = u32::from_le_bytes([data[1], data[2], data[3], data[4]]);

    // Decode props byte into lc, lp, pb
    let _lc = props % 9;
    let remainder = props / 9;
    let _pb = remainder / 5;
    let _lp = remainder % 5;

    // Determine where compressed data starts
    let data_offset = if read_decompressed_size { 13 } else { 5 };

    if data.len() < data_offset {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "LZMA data too short (need {}, got {})",
                data_offset,
                data.len()
            ),
        ));
    }

    // Build LZMA properties for lzma-rs
    // lzma-rs expects a 13-byte header: [props: 5 bytes][size: 8 bytes]
    let mut lzma_header = Vec::with_capacity(13);
    lzma_header.push(props);
    lzma_header.extend_from_slice(&dict_size.to_le_bytes());

    // Add size (0xFFFFFFFFFFFFFFFF means unknown size)
    if read_decompressed_size && data.len() >= 13 {
        // Read the 8-byte size from Unity header
        lzma_header.extend_from_slice(&data[5..13]);
    } else {
        // Unknown size
        lzma_header.extend_from_slice(&[0xFF; 8]);
    }

    // Combine header with compressed data
    lzma_header.extend_from_slice(&data[data_offset..]);

    // Decompress using lzma-rs
    let mut cursor = Cursor::new(lzma_header);
    let mut decompressed = Vec::new();

    lzma_decompress(&mut cursor, &mut decompressed).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("LZMA decompression failed: {}", e),
        )
    })?;

    Ok(decompressed)
}

/// Compresses data using LZMA (Unity-specific format)
///
/// Uses Unity's standard LZMA settings:
/// - dict_size: 8 MB (0x800000)
/// - lc: 3, lp: 0, pb: 2
///
/// # Arguments
///
/// * `data` - Uncompressed data
/// * `write_decompressed_size` - Whether to include 8-byte size in header
///
/// # Returns
///
/// Compressed data with Unity LZMA header
///
/// # Errors
///
/// Returns an error if compression fails
pub fn compress_lzma(data: &[u8], write_decompressed_size: bool) -> io::Result<Vec<u8>> {
    // Unity's standard LZMA parameters
    let dict_size: u32 = 0x800000; // 8 MB
    let lc: u8 = 3;
    let lp: u8 = 0;
    let pb: u8 = 2;

    // Encode lc, lp, pb into props byte
    // Formula: props = (pb * 5 + lp) * 9 + lc
    let props: u8 = (pb * 5 + lp) * 9 + lc; // Should be 0x5D

    // Compress data using lzma-rs
    let mut compressed_data = Vec::new();
    let mut cursor = Cursor::new(data);

    lzma_compress(&mut cursor, &mut compressed_data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("LZMA compression failed: {}", e),
        )
    })?;

    // Skip the 13-byte LZMA header that lzma-rs adds (we'll build our own)
    // lzma-rs header: [props: 1][dict_size: 4][size: 8]
    let compressed_without_header = if compressed_data.len() > 13 {
        &compressed_data[13..]
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Compressed data too short",
        ));
    };

    // Build Unity LZMA header
    let mut result = Vec::new();
    result.push(props); // 1 byte
    result.extend_from_slice(&dict_size.to_le_bytes()); // 4 bytes

    if write_decompressed_size {
        // Add 8-byte size
        result.extend_from_slice(&(data.len() as u64).to_le_bytes());
    }

    // Add compressed data
    result.extend_from_slice(compressed_without_header);

    Ok(result)
}

fn compress_lzma_wrapper(data: &[u8]) -> io::Result<Vec<u8>> {
    compress_lzma(data, false)
}

/// Compresses data using chunk-based compression (for LZ4/LZ4HC)
///
/// Splits data into chunks and compresses each separately. Falls back to
/// uncompressed storage if compression increases size.
///
/// # Arguments
///
/// * `data` - Uncompressed data
/// * `block_info_flag` - Block info flags (includes compression type)
///
/// # Returns
///
/// Tuple of (compressed_data, block_info_list)
/// Each block_info entry is (uncompressed_size, compressed_size, flags)
///
/// # Errors
///
/// Returns an error if compression fails
pub fn chunk_based_compress(
    data: &[u8],
    block_info_flag: u32,
) -> io::Result<(Vec<u8>, Vec<(u32, u32, u32)>)> {
    // Extract compression type from flags
    let compression_type = block_info_flag & 0x3F;

    // Handle NONE compression
    if compression_type == 0 {
        let block_info = vec![(data.len() as u32, data.len() as u32, block_info_flag)];
        return Ok((data.to_vec(), block_info));
    }

    // Determine compression function based on type
    let compress_func: fn(&[u8]) -> io::Result<Vec<u8>> = match compression_type {
        1 => compress_lzma_wrapper, // LZMA
        2 | 3 => compress_lz4,      // LZ4 or LZ4HC (same function)
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("Unsupported compression type: {}", compression_type),
            ));
        }
    };

    // Determine chunk size based on compression type
    let chunk_size: usize = match compression_type {
        1 => 0xFFFFFFFF,     // LZMA - no chunking
        2 | 3 => 0x00020000, // LZ4/LZ4HC - 128KB chunks
        _ => 0xFFFFFFFF,
    };

    let mut block_info = Vec::new();
    let mut compressed_file_data = Vec::new();
    let mut pos = 0;
    let mut remaining_size = data.len();

    while remaining_size > 0 {
        // Determine current chunk size
        let current_chunk_size = if remaining_size > chunk_size {
            chunk_size
        } else {
            remaining_size
        };

        // Get chunk data
        let chunk = &data[pos..pos + current_chunk_size];

        // Compress the chunk
        let compressed_chunk = compress_func(chunk)?;

        // Check if compression helped or made it worse
        if compressed_chunk.len() >= current_chunk_size {
            // Compression didn't help - use uncompressed
            compressed_file_data.extend_from_slice(chunk);
            block_info.push((
                current_chunk_size as u32,
                current_chunk_size as u32,
                block_info_flag ^ compression_type, // XOR to remove compression flag
            ));
        } else {
            // Compression helped - use compressed data
            compressed_file_data.extend_from_slice(&compressed_chunk);
            block_info.push((
                current_chunk_size as u32,
                compressed_chunk.len() as u32,
                block_info_flag,
            ));
        }

        pos += current_chunk_size;
        remaining_size -= current_chunk_size;
    }

    Ok((compressed_file_data, block_info))
}

/// Decompresses data based on compression type
///
/// # Arguments
///
/// * `data` - Compressed data
/// * `uncompressed_size` - Expected uncompressed size (used for LZ4)
/// * `compression_type` - CompressionFlags value
///
/// # Returns
///
/// Decompressed data
///
/// # Errors
///
/// Returns an error if decompression fails or type is unsupported
pub fn decompress(
    data: &[u8],
    uncompressed_size: usize,
    compression_type: u32,
) -> io::Result<Vec<u8>> {
    match compression_type {
        0 => Ok(data.to_vec()),                           // NONE
        1 => decompress_lzma(data, false),                // LZMA
        2 | 3 => decompress_lz4(data, uncompressed_size), // LZ4/LZ4HC
        4 => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "LZHAM compression not supported (removed by Unity)",
        )),
        _ => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("Unknown compression type: {}", compression_type),
        )),
    }
}

/// Compresses data based on compression type
///
/// # Arguments
///
/// * `data` - Uncompressed data
/// * `compression_type` - CompressionFlags value
///
/// # Returns
///
/// Compressed data
///
/// # Errors
///
/// Returns an error if compression fails or type is unsupported
pub fn compress(data: &[u8], compression_type: u32) -> io::Result<Vec<u8>> {
    match compression_type {
        0 => Ok(data.to_vec()),          // NONE
        1 => compress_lzma(data, false), // LZMA
        2 | 3 => compress_lz4(data),     // LZ4/LZ4HC
        _ => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("Unknown compression type: {}", compression_type),
        )),
    }
}

/// Returns the chunk size for a given compression type
///
/// # Arguments
///
/// * `compression_type` - CompressionFlags value
///
/// # Returns
///
/// Chunk size in bytes
pub fn get_chunk_size(compression_type: u32) -> usize {
    match compression_type {
        0 => 0xFFFFFFFF,     // NONE - no chunking
        1 => 0xFFFFFFFF,     // LZMA - no chunking
        2 | 3 => 0x00020000, // LZ4/LZ4HC - 128KB chunks
        _ => 0xFFFFFFFF,
    }
}

/// LZHAM decompression (not supported)
///
/// LZHAM was removed by Unity and is no longer supported.
///
/// # Errors
///
/// Always returns an error indicating LZHAM is not supported
pub fn decompress_lzham(_data: &[u8], _uncompressed_size: usize) -> io::Result<Vec<u8>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Custom compression or unimplemented LZHAM (removed by Unity) encountered!",
    ))
}
