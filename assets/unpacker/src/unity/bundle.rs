use std::io;

use crate::unity::{compression::decompress, endian_reader::EndianReader};

pub struct BundleFile {
    pub files: Vec<BundleEntry>,
}

pub struct BundleEntry {
    pub path: String,
    pub data: Vec<u8>,
}

struct BlockInfo {
    uncompressed_size: u32,
    compressed_size: u32,
    flags: u16,
}

#[allow(dead_code)]
struct DirectoryEntry {
    offset: u64,
    size: u64,
    flags: u32,
    path: String,
}

impl BundleFile {
    pub fn parse(data: Vec<u8>) -> Result<Self, io::Error> {
        let mut r = EndianReader::new(&data, true); // Header is always big-endian

        let sig = r.read_cstring()?;
        if sig != "UnityFS" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "not a UnityFS bundle",
            ));
        }

        let version = r.read_u32()?;
        let _version_player = r.read_cstring()?;
        let _version_engine = r.read_cstring()?;
        let file_size = r.read_i64()?;
        let compressed_block_info_size = r.read_u32()? as usize;
        let uncompressed_block_info_size = r.read_u32()? as usize;
        let data_flags = r.read_u32()?;

        if version >= 7 {
            r.align(16);
        }

        let data_start = r.position();

        if data_flags & 0x80 != 0 {
            r.set_position((file_size as usize) - compressed_block_info_size);
        }

        let comp_type = data_flags & 0x3F;
        let compressed_bytes = r.read_bytes(compressed_block_info_size)?;
        let block_info = decompress(&compressed_bytes, uncompressed_block_info_size, comp_type)?;

        let mut bi = EndianReader::new(&block_info, true);
        let _hash = bi.read_bytes(16)?;

        let block_count = bi.read_i32()? as usize;
        let mut blocks = Vec::with_capacity(block_count);
        for _ in 0..block_count {
            blocks.push(BlockInfo {
                uncompressed_size: bi.read_u32()?,
                compressed_size: bi.read_u32()?,
                flags: bi.read_u16()?,
            });
        }

        let dir_count = bi.read_i32()? as usize;
        let mut dirs = Vec::with_capacity(dir_count);
        for _ in 0..dir_count {
            dirs.push(DirectoryEntry {
                offset: bi.read_i64()? as u64,
                size: bi.read_i64()? as u64,
                flags: bi.read_u32()?,
                path: bi.read_cstring()?,
            });
        }

        if data_flags & 0x80 != 0 {
            r.set_position(data_start);
        }
        if data_flags & 0x200 != 0 {
            r.align(16);
        }

        let total_uncompressed: usize = blocks.iter().map(|b| b.uncompressed_size as usize).sum();
        let mut data_buf = Vec::with_capacity(total_uncompressed);
        for block in &blocks {
            let comp_data = r.read_bytes(block.compressed_size as usize)?;
            let block_comp_type = (block.flags & 0x3F) as u32;
            let decompressed = decompress(
                &comp_data,
                block.uncompressed_size as usize,
                block_comp_type,
            )?;
            data_buf.extend_from_slice(&decompressed);
        }

        let mut files = Vec::with_capacity(dirs.len());
        for dir in &dirs {
            let start = dir.offset as usize;
            let end = start + dir.size as usize;
            files.push(BundleEntry {
                path: dir.path.clone(),
                data: data_buf[start..end].to_vec(),
            });
        }

        Ok(BundleFile { files })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bundle() {
        let data = match std::fs::read("../downloader/ArkAssets/ui/skin_groups.ab") {
            Ok(d) => d,
            Err(e) => {
                eprintln!("skip: {e}");
                return;
            }
        };
        let bundle = BundleFile::parse(data).unwrap();
        assert!(!bundle.files.is_empty());
        for entry in &bundle.files {
            println!("  {} ({} bytes)", entry.path, entry.data.len());
        }
    }
}
