use super::file::File;
use crate::config;
use crate::enums::compression_flags::{ArchiveFlags, ArchiveFlagsOld, CompressionFlags};
use crate::environment::Environment;
use crate::errors::{UnityError, UnityResult};
use crate::files::file::ParentFile;
use crate::helpers::archive_storage_manager::ArchiveStorageDecryptor;
use crate::helpers::compression_helper::{self, chunk_based_compress, compress_lzma};
use crate::helpers::import_helper;
use crate::streams::endian::Endian;
use crate::streams::endian_reader::BinaryReader;
use crate::streams::endian_reader::MemoryReader;
use crate::streams::endian_writer::{BinaryWriter, EndianBinaryWriter};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::rc::{Rc, Weak};

/// Helper struct for block information in UnityFS format
#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub uncompressed_size: u32,
    pub compressed_size: u32,
    pub flags: u16,
}

/// Helper struct for directory information in UnityFS format
/// Note: Different from DirectoryInfo - has flags field
#[derive(Debug, Clone)]
pub struct DirectoryInfoFS {
    pub offset: i64,
    pub size: i64,
    pub flags: u32,
    pub path: String,
}

/// Unified directory entry for both UnityWeb/Raw and UnityFS formats
#[derive(Debug, Clone)]
pub struct BundleDirectoryEntry {
    pub path: String,
    pub offset: i64,
    pub size: i64,
    pub flags: u32, // 0 for UnityWeb/Raw, actual value for UnityFS
}

/// Represents a Unity bundle file (archive container)
#[derive(Debug)]
pub struct BundleFile {
    // Header fields
    pub signature: String,
    pub version: u32,
    pub version_player: String,
    pub version_engine: String,

    // Flags and encryption
    pub dataflags: DataFlags, // enum wrapper for ArchiveFlags/ArchiveFlagsOld
    pub decryptor: Option<ArchiveStorageDecryptor>,
    pub uses_block_alignment: bool,
    pub block_info_flags: u16,

    // Version-specific fields
    pub hash: Option<Vec<u8>>, // 16 bytes, version >= 4
    pub crc: Option<u32>,      // version >= 4

    // File trait fields
    pub name: String,
    pub is_changed: bool,
    pub files: HashMap<String, Rc<RefCell<FileType>>>, // Will need enum for different file types
    pub file_flags: HashMap<String, u32>,
    pub is_dependency: bool,
    pub cab_file: String,
    pub parent: Option<Weak<RefCell<dyn ParentFile>>>,
    pub environment: Option<Rc<RefCell<Environment>>>,
}

/// Wrapper enum for dataflags
#[derive(Debug, Clone, Copy)]
pub enum DataFlags {
    Old(ArchiveFlagsOld),
    New(ArchiveFlags),
}

/// Types that can be stored in BundleFile.files
#[derive(Debug)]
pub enum FileType {
    Raw(MemoryReader),
    SerializedFile(std::rc::Rc<std::cell::RefCell<crate::files::serialized_file::SerializedFile>>),
    BundleFile(Box<BundleFile>),
    WebFile(Box<crate::files::web_file::WebFile>),
    Writer(EndianBinaryWriter),
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = match self {
            FileType::Raw(_) => "Raw",
            FileType::SerializedFile(_) => "SerializedFile",
            FileType::BundleFile(_) => "BundleFile",
            FileType::WebFile(_) => "WebFile",
            FileType::Writer(_) => "Writer",
        };
        write!(f, "<{}>", type_name)
    }
}

impl DataFlags {
    /// Check if a flag is set (bitwise AND)
    pub fn contains(&self, flag: u32) -> bool {
        match self {
            DataFlags::Old(flags) => flags.bits() & flag != 0,
            DataFlags::New(flags) => flags.bits() & flag != 0,
        }
    }

    /// Get the raw bits value
    pub fn bits(&self) -> u32 {
        match self {
            DataFlags::Old(flags) => flags.bits(),
            DataFlags::New(flags) => flags.bits(),
        }
    }

    /// Check if this is the new ArchiveFlags variant
    pub fn is_new(&self) -> bool {
        matches!(self, DataFlags::New(_))
    }

    /// Check if this is the old ArchiveFlagsOld variant
    pub fn is_old(&self) -> bool {
        matches!(self, DataFlags::Old(_))
    }

    /// Get UsesAssetBundleEncryption flag value for this version
    pub fn uses_asset_bundle_encryption_flag(&self) -> u32 {
        match self {
            DataFlags::Old(_) => ArchiveFlagsOld::UsesAssetBundleEncryption.bits(),
            DataFlags::New(_) => ArchiveFlags::UsesAssetBundleEncryption.bits(),
        }
    }
}

/// Parses a string as either decimal (e.g., "64") or hexadecimal (e.g., "0x40")
fn parse_int_or_hex<T>(s: &str) -> Result<T, io::Error>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let trimmed = s.trim();

    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        // Hex format - need to parse manually for generic type
        let without_prefix = &trimmed[2..];
        let value = u64::from_str_radix(without_prefix, 16).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid hex number '{}': {}", trimmed, e),
            )
        })?;

        // Try to convert u64 to target type
        let s = format!("{}", value);
        s.parse::<T>().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Number {} out of range: {}", value, e),
            )
        })
    } else {
        // Decimal format
        trimmed.parse::<T>().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid decimal number '{}': {}", trimmed, e),
            )
        })
    }
}

impl BundleFile {
    /// Constructs a new BundleFile from a binary reader
    ///
    /// # Arguments
    ///
    /// * `reader` - Binary reader positioned at bundle header
    /// * `name` - Optional name for this bundle file
    ///
    /// Python equivalent: BundleFile.__init__ (lines 28-50)
    pub fn new<R: BinaryReader>(
        reader: &mut R,
        name: Option<String>,
    ) -> Result<(Self, MemoryReader, Vec<BundleDirectoryEntry>), io::Error> {
        // Read header (Python lines 36-39)
        let signature = reader.read_string_to_null(1024)?;
        let version = reader.read_u32()?;
        let version_player = reader.read_string_to_null(1024)?;
        let version_engine = reader.read_string_to_null(1024)?;

        // Initialize bundle with defaults
        let mut bundle = Self {
            signature: signature.clone(),
            version,
            version_player,
            version_engine: version_engine.clone(),
            dataflags: DataFlags::Old(ArchiveFlagsOld::empty()), // Will be set by read_fs
            decryptor: None,
            uses_block_alignment: false,
            block_info_flags: 0,
            hash: None,
            crc: None,
            name: name.unwrap_or_else(|| String::from("")),
            is_changed: false,
            files: HashMap::new(),
            file_flags: HashMap::new(),
            is_dependency: false,
            cab_file: String::from("CAB-UnityPy_Mod.resS"),
            parent: None,
            environment: None,
        };

        // Dispatch to format-specific reader (Python lines 41-48)
        let (m_directory_info, blocks_reader) = match signature.as_str() {
            "UnityArchive" => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "BundleFile - UnityArchive not implemented",
                ));
            }
            "UnityWeb" | "UnityRaw" => bundle.read_web_raw(reader)?,
            "UnityFS" => bundle.read_fs(reader)?,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unknown Bundle signature: {}", signature),
                ));
            }
        };

        Ok((bundle, blocks_reader, m_directory_info))
    }

    /// Parses version_engine string into (major, minor, patch) tuple
    ///
    /// # Returns
    ///
    /// Tuple of (major, minor, patch) version numbers
    ///
    /// Python equivalent: get_version_tuple (lines 549-562)
    pub fn get_version_tuple(&self) -> UnityResult<(u32, u32, u32)> {
        use regex::Regex;

        let version = &self.version_engine;

        // Try to match version pattern: (\d+).(\d+).(\d+)\w.+
        let re = Regex::new(r"(\d+)\.(\d+)\.(\d+)\w.+").unwrap();

        if !version.is_empty() && version != "0.0.0" {
            if let Some(captures) = re.captures(version) {
                let major = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
                let minor = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();
                let patch = captures.get(3).unwrap().as_str().parse::<u32>().unwrap();
                return Ok((major, minor, patch));
            }
        }

        // Fall back to config.get_fallback_version()
        let fallback = config::get_fallback_version()?;
        if let Some(captures) = re.captures(&fallback) {
            let major = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
            let minor = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();
            let patch = captures.get(3).unwrap().as_str().parse::<u32>().unwrap();
            return Ok((major, minor, patch));
        }

        Err(UnityError::version_fallback_error(
            "Illegal fallback version format",
        ))
    }

    /// Decompresses block data with optional decryption
    ///
    /// # Arguments
    ///
    /// * `compressed_data` - The compressed data bytes
    /// * `uncompressed_size` - Expected size after decompression
    /// * `flags` - Compression/encryption flags
    /// * `index` - Block index (for decryption)
    ///
    /// Python equivalent: decompress_data (lines 511-547)
    pub fn decompress_data(
        &self,
        compressed_data: Vec<u8>,
        uncompressed_size: u32,
        flags: u32,
        index: usize,
    ) -> Result<Vec<u8>, io::Error> {
        let comp_flag =
            CompressionFlags::from_bits_truncate(flags & ArchiveFlags::CompressionTypeMask.bits());

        // Decrypt if needed (Python lines 534-535)
        let data_to_decompress = if let Some(decryptor) = &self.decryptor {
            if flags & 0x100 != 0 {
                decryptor.decrypt_block(&compressed_data, index)
            } else {
                compressed_data
            }
        } else {
            compressed_data
        };

        // Decompress based on compression flag (Python lines 537-547)
        compression_helper::decompress(
            &data_to_decompress,
            uncompressed_size as usize,
            comp_flag.bits(),
        )
    }

    pub fn read_web_raw<R: BinaryReader>(
        &mut self,
        reader: &mut R,
    ) -> Result<(Vec<BundleDirectoryEntry>, MemoryReader), io::Error> {
        let version = self.version;

        if version >= 4 {
            self.hash = Some(reader.read_bytes(16)?);
            self.crc = Some(reader.read_u32()?);
        }

        let _minimum_streamed_bytes = reader.read_u32()?;
        let header_size = reader.read_u32()?;
        let _number_of_levels_to_download_before_streaming = reader.read_u32()?;
        let level_count = reader.read_i32()?;

        reader.set_position(reader.position() + (4 * 2 * (level_count - 1) as usize));

        let compressed_size = reader.read_u32()?;
        let _uncompressed_size = reader.read_u32()?;

        if version >= 2 {
            let _complete_file_size = reader.read_u32()?;
        }

        if version >= 3 {
            let _file_info_header_size = reader.read_u32()?;
        }

        let mut uncompressed_bytes = reader.read_bytes(compressed_size as usize)?;
        if self.signature == "UnityWeb" {
            uncompressed_bytes = compression_helper::decompress_lzma(&uncompressed_bytes, true)?;
        }

        // Python line 82: EndianBinaryReader defaults to big-endian
        let mut blocks_reader =
            MemoryReader::new(uncompressed_bytes, Endian::Big, header_size as usize);

        // Read directory info (Python lines 83-90)
        let nodes_count = blocks_reader.read_i32()?;
        let m_directory_info: Vec<BundleDirectoryEntry> = (0..nodes_count)
            .map(|_| {
                Ok(BundleDirectoryEntry {
                    path: blocks_reader.read_string_to_null(1024)?,
                    offset: blocks_reader.read_u32()? as i64,
                    size: blocks_reader.read_u32()? as i64,
                    flags: 0, // UnityWeb/Raw format doesn't have flags
                })
            })
            .collect::<Result<Vec<_>, io::Error>>()?;

        Ok((m_directory_info, blocks_reader))
    }

    pub fn read_fs<R: BinaryReader>(
        &mut self,
        reader: &mut R,
    ) -> Result<(Vec<BundleDirectoryEntry>, MemoryReader), io::Error> {
        let _size = reader.read_i64()?;

        // Header
        let compressed_size = reader.read_u32()?;
        let uncompressed_size = reader.read_u32()?;
        let data_flags_value = reader.read_u32()?;

        let version = self
            .get_version_tuple()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        if version.0 < 2020
            || (version.0 == 2020 && version < (2020, 3, 34))
            || (version.0 == 2021 && version < (2021, 3, 2))
            || (version.0 == 2022 && version < (2022, 1, 1))
        {
            self.dataflags = DataFlags::Old(ArchiveFlagsOld::from_bits_truncate(data_flags_value));
        } else {
            self.dataflags = DataFlags::New(ArchiveFlags::from_bits_truncate(data_flags_value));
        }

        if self
            .dataflags
            .contains(self.dataflags.uses_asset_bundle_encryption_flag())
        {
            self.decryptor = Some(
                ArchiveStorageDecryptor::new(reader)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            );
        }

        if self.version >= 7 {
            reader.align_stream(16);
            self.uses_block_alignment = true;
        } else if version.0 >= 2019 && version.1 >= 4 {
            let pre_align = reader.position();
            let align_data = reader.read((16 - pre_align % 16) % 16)?;
            if align_data.iter().any(|&b| b != 0) {
                reader.set_position(pre_align);
            } else {
                self.uses_block_alignment = true;
            }
        }

        let start = reader.position();
        let mut blocks_info_bytes;
        if self
            .dataflags
            .contains(ArchiveFlags::BlocksInfoAtTheEnd.bits())
        {
            reader.set_position(reader.len() - compressed_size as usize);
            blocks_info_bytes = reader.read_bytes(compressed_size as usize)?;
            reader.set_position(start);
        } else {
            blocks_info_bytes = reader.read_bytes(compressed_size as usize)?;
        }

        blocks_info_bytes = self.decompress_data(
            blocks_info_bytes,
            uncompressed_size,
            self.dataflags.bits(),
            0,
        )?;

        // Python line 148: EndianBinaryReader defaults to big-endian
        // offset parameter is for reporting positions, not for where to start reading
        let mut blocks_info_reader = MemoryReader::new(blocks_info_bytes, Endian::Big, 0);
        let _uncompressed_data_hash = blocks_info_reader.read_bytes(16)?;
        let blocks_info_count = blocks_info_reader.read_i32()?;

        let m_blocks_info: Vec<BlockInfo> = (0..blocks_info_count)
            .map(|_| {
                Ok(BlockInfo {
                    uncompressed_size: blocks_info_reader.read_u32()?,
                    compressed_size: blocks_info_reader.read_u32()?,
                    flags: blocks_info_reader.read_u16()?,
                })
            })
            .collect::<Result<Vec<_>, io::Error>>()?;

        let nodes_count = blocks_info_reader.read_i32()?;

        let m_directory_info_fs: Vec<DirectoryInfoFS> = (0..nodes_count)
            .map(|_| {
                Ok(DirectoryInfoFS {
                    offset: blocks_info_reader.read_i64()?,
                    size: blocks_info_reader.read_i64()?,
                    flags: blocks_info_reader.read_u32()?,
                    path: blocks_info_reader.read_string_to_null(1024)?,
                })
            })
            .collect::<Result<Vec<_>, io::Error>>()?;

        if !m_blocks_info.is_empty() {
            self.block_info_flags = m_blocks_info[0].flags;
        }

        if self.dataflags.is_new()
            && self
                .dataflags
                .contains(ArchiveFlags::BlockInfoNeedPaddingAtStart.bits())
        {
            reader.align_stream(16);
        }

        let mut blocks_data = Vec::new();
        for (i, block_info) in m_blocks_info.iter().enumerate() {
            let compressed_block = reader.read_bytes(block_info.compressed_size as usize)?;
            let decompressed_block = self.decompress_data(
                compressed_block,
                block_info.uncompressed_size,
                block_info.flags as u32,
                i,
            )?;
            blocks_data.extend_from_slice(&decompressed_block);
        }

        // Python line 182: EndianBinaryReader defaults to big-endian
        // offset parameter is for reporting positions, not for where to start reading
        let blocks_reader = MemoryReader::new(blocks_data, Endian::Big, 0);

        let m_directory_info: Vec<BundleDirectoryEntry> = m_directory_info_fs
            .into_iter()
            .map(|dir| BundleDirectoryEntry {
                path: dir.path,
                offset: dir.offset,
                size: dir.size,
                flags: dir.flags,
            })
            .collect();

        Ok((m_directory_info, blocks_reader))
    }

    /// Saves the BundleFile and returns it as bytes
    ///
    /// # Arguments
    ///
    /// * `packer` - Compression strategy:
    ///   - None/"none" - no compression (default)
    ///   - "original" - use original compression flags
    ///   - "lz4" - LZ4 compression
    ///   - "lzma" - LZMA compression
    ///   - (block_info_flag, data_flag) tuple for custom flags
    ///
    /// Python equivalent: save() (lines 197-242)
    pub fn save(&self, packer: Option<&str>) -> Result<Vec<u8>, io::Error> {
        let mut writer = EndianBinaryWriter::new(Endian::Big);

        writer
            .write_string_to_null(&self.signature)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        writer
            .write_u32(self.version)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        writer
            .write_string_to_null(&self.version_player)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        writer
            .write_string_to_null(&self.version_engine)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        if self.signature == "UnityArchive" {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "BundleFile - UnityArchive not yet implemented",
            ));
        } else if self.signature == "UnityWeb" || self.signature == "UnityRaw" {
            self.save_web_raw(&mut writer)?;
        } else if self.signature == "UnityFS" {
            if packer.is_none() || packer == Some("none") {
                self.save_fs(&mut writer, 64, 64)?;
            } else if packer == Some("original") {
                self.save_fs(&mut writer, self.dataflags.bits(), self.block_info_flags)?;
            } else if packer == Some("lz4") {
                self.save_fs(&mut writer, 194, 2)?;
            } else if packer == Some("lzma") {
                self.save_fs(&mut writer, 65, 1)?;
            } else if let Some(packer_str) = packer {
                // Check if it's a tuple format: "(num1, num2)"
                if packer_str.starts_with('(') && packer_str.ends_with(')') {
                    // Strip parentheses
                    let inner = &packer_str[1..packer_str.len() - 1];

                    // Split by comma
                    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

                    if parts.len() == 2 {
                        // Parse both numbers (support hex like 0x40 or decimal like 64)
                        let data_flag = parse_int_or_hex::<u32>(parts[0])?;
                        let block_info_flag = parse_int_or_hex::<u16>(parts[1])?;

                        self.save_fs(&mut writer, data_flag, block_info_flag)?;
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("Invalid tuple format: {}", packer_str),
                        ));
                    }
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Unknown packer type: {}", packer_str),
                    ));
                }
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unknown packer type: {:?}", packer),
                ));
            }
        }

        Ok(writer.to_bytes())
    }

    /// Saves UnityFS format
    ///
    /// Python equivalent: save_fs() (lines 244-210)
    fn save_fs(
        &self,
        writer: &mut EndianBinaryWriter,
        mut data_flag: u32,
        mut block_info_flag: u16,
    ) -> Result<(), io::Error> {
        let mut data_writer = EndianBinaryWriter::new(Endian::Big);

        let mut files: Vec<(String, u32, usize)> = Vec::new();
        for (name, f) in &self.files {
            let (flags, file_bytes) = match &*f.borrow() {
                FileType::Raw(reader) => (0u32, reader.bytes().to_vec()),
                FileType::SerializedFile(sf_rc) => {
                    let mut sf = sf_rc.borrow_mut();
                    (0u32, sf.save()?)
                }
                FileType::BundleFile(bf) => (0u32, bf.save(None)?),
                FileType::WebFile(wf) => (0u32, wf.save(None, None, None)?), // Needs arguments!
                FileType::Writer(writer) => {
                    // Get flags from file_flags HashMap, default to 0 if not found
                    let flags = self.file_flags.get(name).copied().unwrap_or(0);
                    let file_bytes = writer.bytes().to_vec();
                    (flags, file_bytes)
                }
            };

            data_writer
                .write_bytes(&file_bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            files.push((name.clone(), flags, file_bytes.len()));
        }

        let mut file_data = data_writer.to_bytes();

        let encryption_flag = self.dataflags.uses_asset_bundle_encryption_flag();

        if (block_info_flag as u32) & encryption_flag != 0 {
            block_info_flag ^= encryption_flag as u16;
        }
        if data_flag & encryption_flag != 0 {
            data_flag ^= self.dataflags.uses_asset_bundle_encryption_flag();
        }

        let (file_data_compressed, block_info) =
            chunk_based_compress(&file_data, block_info_flag as u32)?;
        file_data = file_data_compressed;

        let mut block_writer = EndianBinaryWriter::new(Endian::Big);
        block_writer.write(&vec![0u8; 16])?;

        block_writer.write_i32(block_info.len() as i32)?;
        for (block_uncompressed_size, block_compressed_size, block_flag) in block_info {
            block_writer.write_u32(block_uncompressed_size)?;
            block_writer.write_u32(block_compressed_size)?;
            block_writer.write_u16(block_flag as u16)?;
        }

        if data_flag & 0x40 == 0 {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "UnityRs always writes DirectoryInfo, so data_flag must include 0x40",
            ));
        }

        block_writer.write_i32(files.len() as i32)?;

        let mut offset = 0;
        for (f_name, f_flag, f_len) in files {
            block_writer.write_i64(offset)?;
            block_writer.write_i64(f_len as i64)?;

            offset += f_len as i64;

            block_writer.write_u32(f_flag)?;
            block_writer
                .write_string_to_null(&f_name)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }

        let mut block_data = block_writer.to_bytes();
        let uncompressed_block_data_size = block_data.len();

        let switch = data_flag & 0x3F;
        block_data = compression_helper::compress(&block_data, switch)?;

        let compressed_block_data_size = block_data.len();

        let writer_header_pos = writer.position();
        writer.write_i64(0)?;
        writer.write_u32(compressed_block_data_size as u32)?;
        writer.write_u32(uncompressed_block_data_size as u32)?;
        writer.write_u32(data_flag)?;

        if self.uses_block_alignment {
            writer.align_stream(16)?;
        }

        if data_flag & 0x80 != 0 {
            if data_flag & 0x200 != 0 {
                writer.align_stream(16)?;
            }
            writer.write(&file_data)?;
            writer.write(&block_data)?;
        } else {
            writer.write(&block_data)?;
            if data_flag & 0x200 != 0 {
                writer.align_stream(16)?;
            }
            writer.write(&file_data)?;
        }

        let writer_end_pos = writer.position();
        writer.set_position(writer_header_pos);
        writer.write_i64(writer_end_pos as i64)?;
        writer.set_position(writer_end_pos);

        Ok(())
    }

    /// Saves UnityWeb/UnityRaw format
    ///
    /// Python equivalent: save_web_raw() (lines 211-316)
    fn save_web_raw(&self, writer: &mut EndianBinaryWriter) -> Result<(), io::Error> {
        if self.version > 3 {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Saving Unity Web bundles with version > 3 is not supported",
            ));
        }

        let mut file_info_header_size = 4;
        for file_name in self.files.keys() {
            file_info_header_size += file_name.len() + 1;
            file_info_header_size += 4 * 2;
        }

        let file_info_header_padding_size = if file_info_header_size % 4 != 0 {
            4 - (file_info_header_size % 4)
        } else {
            0
        };

        file_info_header_size += file_info_header_padding_size;

        let mut directory_info_writer = EndianBinaryWriter::new(Endian::Big);
        directory_info_writer.write_i32(self.files.len() as i32)?;

        let mut file_content_writer = EndianBinaryWriter::new(Endian::Big);
        let mut current_offset = file_info_header_size;

        for (file_name, f) in &self.files {
            directory_info_writer.write_string_to_null(file_name)?;
            directory_info_writer.write_u32(current_offset as u32)?;

            let (_flags, file_bytes) = match &*f.borrow() {
                FileType::Raw(reader) => (0u32, reader.bytes().to_vec()),
                FileType::SerializedFile(sf_rc) => {
                    let mut sf = sf_rc.borrow_mut();
                    (0u32, sf.save()?)
                }
                FileType::BundleFile(bf) => (0u32, bf.save(None)?),
                FileType::WebFile(wf) => (0u32, wf.save(None, None, None)?),
                FileType::Writer(writer) => {
                    let flags = self.file_flags.get(file_name).copied().unwrap_or(0);
                    let file_bytes = writer.bytes().to_vec();
                    (flags, file_bytes)
                }
            };

            let file_size = file_bytes.len();
            directory_info_writer.write_u32(file_size as u32)?;

            file_content_writer.write_bytes(&file_bytes)?;
            current_offset += file_size;
        }

        directory_info_writer.write(&vec![0u8; file_info_header_padding_size])?;
        let uncompressed_directory_info = directory_info_writer.to_bytes();
        let uncompressed_file_content = file_content_writer.to_bytes();

        let mut uncompressed_content = uncompressed_directory_info;
        uncompressed_content.extend_from_slice(&uncompressed_file_content);

        let compressed_content = if self.signature == "UnityWeb" {
            compress_lzma(&uncompressed_content, true)?
        } else {
            uncompressed_content.clone()
        };

        let mut header_size = writer.position() + 24;
        if self.version >= 2 {
            header_size += 4;
        }
        if self.version >= 3 {
            header_size += 4;
        }
        if self.version >= 4 {
            header_size += 20;
        }
        header_size = (header_size + 3) & !3;

        if self.version >= 4 {
            let hash = self.hash.as_ref().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Missing hash for version >= 4")
            })?;
            let crc = self.crc.ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Missing crc for version >= 4")
            })?;
            writer.write_bytes(hash)?;
            writer.write_u32(crc)?;
        }

        writer.write_u32((header_size + compressed_content.len()) as u32)?;
        writer.write_u32(header_size as u32)?;
        writer.write_u32(1)?;
        writer.write_i32(1)?;

        writer.write_u32(compressed_content.len() as u32)?;
        writer.write_u32(uncompressed_content.len() as u32)?;

        if self.version >= 2 {
            writer.write_u32((header_size + compressed_content.len()) as u32)?;
        }

        if self.version >= 3 {
            writer.write_u32(file_info_header_size as u32)?;
        }

        writer.align_stream(4)?;
        writer.write(&compressed_content)?;

        Ok(())
    }

    /// Loads child files from the bundle with proper parent reference
    ///
    /// This must be called after the BundleFile is wrapped in Rc<RefCell<>>
    ///
    /// # Arguments
    /// * `self_rc` - Rc reference to this BundleFile
    /// * `blocks_reader` - Reader containing the block data
    /// * `m_directory_info` - Directory entries to load
    pub fn load_files(
        self_rc: &Rc<RefCell<Self>>,
        mut blocks_reader: MemoryReader,
        m_directory_info: Vec<BundleDirectoryEntry>,
    ) -> Result<(), io::Error> {
        let parent_weak = Rc::downgrade(self_rc) as Weak<RefCell<dyn ParentFile>>;

        let mut parsed_files = HashMap::new();

        for node in m_directory_info {
            blocks_reader.set_position(node.offset as usize);
            let file_data = blocks_reader.read(node.size as usize)?;
            let file_reader = MemoryReader::new(
                file_data,
                blocks_reader.endian(),
                blocks_reader.base_offset() + node.offset as usize,
            );

            let is_dependency = self_rc.borrow().is_dependency;

            let parsed_file = import_helper::parse_file(
                file_reader,
                node.path.clone(),
                None,
                is_dependency,
                Some(parent_weak.clone()),
            )?;

            // Wrap the parsed file
            let file_rc = Rc::new(RefCell::new(parsed_file));

            // Register CABs with environment if present
            if let Some(env) = self_rc.borrow().environment.as_ref() {
                // Check if this file type should be registered (Raw or SerializedFile)
                let should_register = matches!(
                    &*file_rc.borrow(),
                    FileType::Raw(_) | FileType::SerializedFile(_)
                );

                if should_register {
                    env.borrow_mut()
                        .register_cab(node.path.clone(), file_rc.clone());
                }
            }

            // Store in parsed_files HashMap
            parsed_files.insert(node.path.clone(), file_rc);
            self_rc
                .borrow_mut()
                .file_flags
                .insert(node.path, node.flags);
        }

        self_rc.borrow_mut().files = parsed_files;
        Ok(())
    }
}

impl File for BundleFile {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_changed(&self) -> bool {
        self.is_changed
    }

    fn mark_changed(&mut self) {
        self.is_changed = true;

        if let Some(parent_weak) = &self.parent {
            if let Some(parent_rc) = parent_weak.upgrade() {
                parent_rc.borrow_mut().mark_changed();
            }
        }
    }

    fn is_dependency(&self) -> bool {
        self.is_dependency
    }

    fn cab_file(&self) -> &str {
        &self.cab_file
    }

    fn files(&self) -> &HashMap<String, Rc<RefCell<FileType>>> {
        &self.files
    }

    fn files_mut(&mut self) -> &mut HashMap<String, Rc<RefCell<FileType>>> {
        &mut self.files
    }

    fn file_flags(&self) -> &HashMap<String, u32> {
        &self.file_flags
    }

    fn file_flags_mut(&mut self) -> &mut HashMap<String, u32> {
        &mut self.file_flags
    }

    fn environment(&self) -> Option<Rc<RefCell<Environment>>> {
        self.environment.clone()
    }
}

impl ParentFile for BundleFile {
    fn find_file(&self, name: &str) -> Option<Rc<RefCell<FileType>>> {
        // Case-insensitive search through files
        let name_lower = name.to_lowercase();
        for (key, file) in &self.files {
            if key.to_lowercase() == name_lower {
                return Some(Rc::clone(file));
            }
        }
        None
    }

    fn mark_changed(&mut self) {
        self.is_changed = true;

        if let Some(parent_weak) = &self.parent {
            if let Some(parent_rc) = parent_weak.upgrade() {
                parent_rc.borrow_mut().mark_changed();
            }
        }
    }

    fn get_writeable_cab(
        &mut self,
        name: Option<&str>,
    ) -> Result<Option<Rc<RefCell<FileType>>>, io::Error> {
        // Delegate to File trait implementation
        <Self as File>::get_writeable_cab(self, name)
    }

    fn get_version_engine(&self) -> Option<String> {
        Some(self.version_engine.clone())
    }
}
