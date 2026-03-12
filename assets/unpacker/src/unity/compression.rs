use crate::unity::lz4ak::fix_lz4ak;
use std::io;

pub fn decompress(
    data: &[u8],
    uncompressed_size: usize,
    compression_type: u32,
) -> Result<Vec<u8>, io::Error> {
    match compression_type {
        0 => Ok(data.to_vec()),                           // None
        1 => decompress_lzma(data, uncompressed_size),    // LZMA
        2 | 3 => decompress_lz4(data, uncompressed_size), // LZ4/LZ4HC
        4 => decompress_lz4ak(data, uncompressed_size),   // Arknights custom
        other => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown compression type: {other}"),
        )),
    }
}

fn decompress_lz4(data: &[u8], uncompressed_size: usize) -> Result<Vec<u8>, io::Error> {
    lz4::block::decompress(data, Some(uncompressed_size as i32))
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn decompress_lz4ak(data: &[u8], uncompressed_size: usize) -> Result<Vec<u8>, io::Error> {
    let fixed = fix_lz4ak(data, uncompressed_size);
    decompress_lz4(&fixed, uncompressed_size)
}

fn decompress_lzma(data: &[u8], uncompressed_size: usize) -> Result<Vec<u8>, io::Error> {
    // Unity LZMA: 5-byte header (1 byte props + 4 byte dict_size LE)
    // then compressed stream. lzma-rs needs the uncompressed size prepended
    // as 8-byte LE after the 5-byte header.
    if data.len() < 5 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "LZMA data too short",
        ));
    }
    let mut input = Vec::with_capacity(13 + data.len() - 5);
    input.extend_from_slice(&data[..5]); // props + dict_size
    input.extend_from_slice(&(uncompressed_size as u64).to_le_bytes()); // uncompressed size
    input.extend_from_slice(&data[5..]); // compressed stream

    let mut output = Vec::with_capacity(uncompressed_size);
    lzma_rs::lzma_decompress(&mut input.as_slice(), &mut output)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(output)
}
