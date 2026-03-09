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

#[cfg(test)]
mod tests {
    use crate::unity::compression::decompress;
    use crate::unity::endian_reader::EndianReader;

    #[test]
    fn test_decompress_block_info() {
        // Pick a small AB file from your assets
        let data = match std::fs::read("../downloader/ArkAssets/ui/skin_groups.ab") {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to read file: {}", e);
                return;
            }
        };
        let mut r = EndianReader::new(&data, true);

        // Parse header
        let sig = r.read_cstring().unwrap();
        assert_eq!(sig, "UnityFS");
        let version = r.read_u32().unwrap();
        let _version_player = r.read_cstring().unwrap();
        let _version_engine = r.read_cstring().unwrap();
        let _file_size = r.read_i64().unwrap();
        let compressed_size = r.read_u32().unwrap() as usize;
        let uncompressed_size = r.read_u32().unwrap() as usize;
        let flags = r.read_u32().unwrap();

        // Align for version >= 7
        if version >= 7 {
            r.align(16);
        }

        // Decompress block info (compression type from flags & 0x3F)
        let comp_type = flags & 0x3F;
        let compressed_data = r.read_bytes(compressed_size).unwrap();
        let block_info = decompress(&compressed_data, uncompressed_size, comp_type).unwrap();

        assert_eq!(block_info.len(), uncompressed_size);

        // Parse block info to verify structure
        let mut bi = EndianReader::new(&block_info, true);
        let _hash = bi.read_bytes(16).unwrap();
        let block_count = bi.read_i32().unwrap();
        assert!(block_count > 0, "should have at least one block");

        println!("Blocks: {block_count}");
        for i in 0..block_count {
            let uncomp = bi.read_u32().unwrap();
            let comp = bi.read_u32().unwrap();
            let bflags = bi.read_u16().unwrap();
            println!(
                "  block {i}: uncomp={uncomp} comp={comp} type={}",
                bflags & 0x3F
            );
        }

        // Read directory entries
        let dir_count = bi.read_i32().unwrap();
        assert!(dir_count > 0, "should have at least one file entry");
        println!("Directory entries: {dir_count}");
        for i in 0..dir_count {
            let offset = bi.read_i64().unwrap();
            let size = bi.read_i64().unwrap();
            let dflags = bi.read_u32().unwrap();
            let path = bi.read_cstring().unwrap();
            println!("  entry {i}: offset={offset} size={size} flags={dflags} path={path}");
        }
    }

    #[test]
    fn test_decompress_first_data_block() {
        let data = match std::fs::read("../downloader/ArkAssets/ui/skin_groups.ab") {
            Ok(d) => d,
            Err(e) => {
                eprintln!("skip: {e}");
                return;
            }
        };
        let mut r = EndianReader::new(&data, true);

        // Parse header
        let sig = r.read_cstring().unwrap();
        assert_eq!(sig, "UnityFS");
        let version = r.read_u32().unwrap();
        let _vp = r.read_cstring().unwrap();
        let _ve = r.read_cstring().unwrap();
        let _file_size = r.read_i64().unwrap();
        let comp_bi_size = r.read_u32().unwrap() as usize;
        let uncomp_bi_size = r.read_u32().unwrap() as usize;
        let flags = r.read_u32().unwrap();
        if version >= 7 {
            r.align(16);
        }

        // Read block info
        let comp_type = flags & 0x3F;
        let comp_bytes = r.read_bytes(comp_bi_size).unwrap();
        let bi_data = decompress(&comp_bytes, uncomp_bi_size, comp_type).unwrap();

        // Parse first block
        let mut bi = EndianReader::new(&bi_data, true);
        bi.read_bytes(16).unwrap(); // hash
        let _block_count = bi.read_i32().unwrap();
        let uncomp = bi.read_u32().unwrap();
        let comp = bi.read_u32().unwrap();
        let bflags = bi.read_u16().unwrap();

        println!(
            "First data block: comp={comp} uncomp={uncomp} type={}",
            bflags & 0x3F
        );
        println!("Reader position before data block: {}", r.position());

        r.align(16);
        println!("Reader position after align: {}", r.position());

        // Read and decompress first data block
        let block_data = r.read_bytes(comp as usize).unwrap();
        println!(
            "First 16 bytes of compressed data: {:02x?}",
            &block_data[..16]
        );

        let result = decompress(&block_data, uncomp as usize, (bflags & 0x3F) as u32);
        match &result {
            Ok(d) => println!("Success! Decompressed {} bytes", d.len()),
            Err(e) => println!("Failed: {e}"),
        }
        result.unwrap();
    }
}
