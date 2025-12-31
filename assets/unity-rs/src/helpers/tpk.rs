use bitflags::bitflags;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::io::Read;
use std::sync::Mutex;

use crate::helpers::type_tree_node::TypeTreeNode;

// Embedded TPK data
const TPK_BYTES: &[u8] = include_bytes!("../resources/uncompressed.tpk");

// Global state - initialized lazily on first access
static TPK_TYPE_TREE: Lazy<Mutex<Option<TpkTypeTreeBlob>>> = Lazy::new(|| {
    // Automatically initialize the TPK on first access
    let mut cursor = std::io::Cursor::new(TPK_BYTES);
    let result = TpkFile::from_reader(&mut cursor)
        .and_then(|tpk_file| tpk_file.get_data_blob())
        .ok()
        .and_then(|blob| {
            if let TpkDataBlob::TypeTree(type_tree_blob) = blob {
                Some(type_tree_blob)
            } else {
                None
            }
        });

    Mutex::new(result)
});
static CLASSES_CACHE: Lazy<Mutex<HashMap<(i32, (u16, u16, u16, u16)), TypeTreeNode>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static NODES_CACHE: Lazy<Mutex<HashMap<TpkUnityClass, TypeTreeNode>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TpkCompressionType {
    None = 0,
    Lz4 = 1,
    Lzma = 2,
    Brotli = 3,
}

impl TpkCompressionType {
    pub fn from_u8(value: u8) -> Result<Self, String> {
        match value {
            0 => Ok(TpkCompressionType::None),
            1 => Ok(TpkCompressionType::Lz4),
            2 => Ok(TpkCompressionType::Lzma),
            3 => Ok(TpkCompressionType::Brotli),
            _ => Err(format!("Invalid TpkCompressionType: {}", value)),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnityVersionType {
    Alpha = 0,
    Beta = 1,
    China = 2,
    Final = 3,
    Patch = 4,
    Experimental = 5,
}

impl UnityVersionType {
    pub fn from_u8(value: u8) -> Result<Self, String> {
        match value {
            0 => Ok(UnityVersionType::Alpha),
            1 => Ok(UnityVersionType::Beta),
            2 => Ok(UnityVersionType::China),
            3 => Ok(UnityVersionType::Final),
            4 => Ok(UnityVersionType::Patch),
            5 => Ok(UnityVersionType::Experimental),
            _ => Err(format!("Invalid UnityVersionType: {}", value)),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TpkDataType {
    TypeTreeInformation = 0,
    Collection = 1,
    FileSystem = 2,
    Json = 3,
    ReferenceAssemblies = 4,
    EngineAssets = 5,
}

impl TpkDataType {
    pub fn from_u8(value: u8) -> std::io::Result<Self> {
        match value {
            0 => Ok(TpkDataType::TypeTreeInformation),
            1 => Ok(TpkDataType::Collection),
            2 => Ok(TpkDataType::FileSystem),
            3 => Ok(TpkDataType::Json),
            4 => Ok(TpkDataType::ReferenceAssemblies),
            5 => Ok(TpkDataType::EngineAssets),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid TpkDataType: {}", value),
            )),
        }
    }

    pub fn to_blob<R: Read>(&self, reader: &mut R) -> std::io::Result<TpkDataBlob> {
        match self {
            TpkDataType::TypeTreeInformation => {
                Ok(TpkDataBlob::TypeTree(TpkTypeTreeBlob::from_reader(reader)?))
            }
            TpkDataType::Collection => Ok(TpkDataBlob::Collection(TpkCollectionBlob::from_reader(
                reader,
            )?)),
            TpkDataType::FileSystem => Ok(TpkDataBlob::FileSystem(TpkFileSystemBlob::from_reader(
                reader,
            )?)),
            TpkDataType::Json => Ok(TpkDataBlob::Json(TpkJsonBlob::from_reader(reader)?)),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unimplemented TpkDataType -> Blob conversion",
            )),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TpkUnityClassFlags: u8 {
        const NONE = 0;
        const IS_ABSTRACT = 1;
        const IS_SEALED = 2;
        const IS_EDITOR_ONLY = 4;
        const IS_RELEASE_ONLY = 8;
        const IS_STRIPPED = 16;
        const RESERVED = 32;
        const HAS_EDITOR_ROOT_NODE = 64;
        const HAS_RELEASE_ROOT_NODE = 128;
    }
}

/// UnityVersion stores version information as a packed u64
/// Format: [major: 16 bits][minor: 16 bits][build: 16 bits][type: 8 bits][type_number: 8 bits]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnityVersion(u64);

impl UnityVersion {
    /// Creates a UnityVersion from a binary stream (reads 8 bytes as little-endian u64)
    pub fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf)?;
        let value = u64::from_le_bytes(buf); // Little-endian conversion
        Ok(UnityVersion(value))
    }

    /// Creates a UnityVersion from a string like "2019.4.1.123"
    pub fn from_string(version: &str) -> Result<Self, String> {
        let parts: Vec<&str> = version.split(".").collect();
        if parts.len() != 4 {
            return Err(format!("Invalid version string: {}", version));
        }

        let major = parts[0].parse::<u64>().map_err(|e| e.to_string())?;
        let minor = parts[1].parse::<u64>().map_err(|e| e.to_string())?;
        let patch = parts[2].parse::<u64>().map_err(|e| e.to_string())?;
        let build = parts[3].parse::<u64>().map_err(|e| e.to_string())?;

        Ok(Self::from_parts(major, minor, patch, build))
    }

    /// Creates a UnityVersion from individual parts
    pub fn from_parts(major: u64, minor: u64, patch: u64, build: u64) -> Self {
        // Pack the values into a u64 using bit shifting
        // major occupies bits 48-63 (top 16 bits)
        // minor occupies bits 32-47
        // patch occupies bits 16-31
        // build occupies bits 0-15 (bottom 16 bits)
        let value = (major << 48) | (minor << 32) | (patch << 16) | build;
        UnityVersion(value)
    }

    /// Extracts the major version (top 16 bits)
    pub fn major(&self) -> u16 {
        ((self.0 >> 48) & 0xFFFF) as u16
    }

    /// Extracts the minor version (bits 32-47)
    pub fn minor(&self) -> u16 {
        ((self.0 >> 32) & 0xFFFF) as u16
    }

    /// Extracts the build/patch version (bits 16-31)
    pub fn build(&self) -> u16 {
        ((self.0 >> 16) & 0xFFFF) as u16
    }

    /// Extracts the version type (bits 8-15)
    pub fn version_type(&self) -> Result<UnityVersionType, String> {
        let type_value = ((self.0 >> 8) & 0xFF) as u8;
        UnityVersionType::from_u8(type_value)
    }

    /// Extracts the type number (bits 0-7)
    pub fn type_number(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    /// Returns the raw u64 value
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

// Implement Display for nice printing
impl std::fmt::Display for UnityVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UnityVersion {}.{}.{}.{}",
            self.major(),
            self.minor(),
            self.build(),
            self.type_number()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////////
//
// Helper Functions
//
////////////////////////////////////////////////////////////////////////////////////

/// Reads a variable-length integer (varint) encoded length, then reads that many bytes as a UTF-8 string
pub fn read_string<R: Read>(reader: &mut R) -> std::io::Result<String> {
    // Read varint length
    let mut shift = 0;
    let mut length: usize = 0;

    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        let b = byte[0];

        // Take the bottom 7 bits and shift into position
        length |= ((b & 0x7F) as usize) << shift;
        shift += 7;

        // If the top bit (0x80) is NOT set, we're done
        if (b & 0x80) == 0 {
            break;
        }
    }

    // Read the string bytes
    let mut string_bytes = vec![0u8; length];
    reader.read_exact(&mut string_bytes)?;

    // Convert to UTF-8 string
    String::from_utf8(string_bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Reads an i32 length, then reads that many bytes
pub fn read_data<R: Read>(reader: &mut R) -> std::io::Result<Vec<u8>> {
    // Read i32 length (little-endian)
    let mut length_buf = [0u8; 4];
    reader.read_exact(&mut length_buf)?;
    let length = i32::from_le_bytes(length_buf);

    if length < 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid data length: {}", length),
        ));
    }

    // Read the data
    let mut data = vec![0u8; length as usize];
    reader.read_exact(&mut data)?;

    Ok(data)
}

/// Finds the appropriate item for a given Unity version from a sorted list
/// The list should be sorted by version ascending
/// Returns the item with the highest version that is <= exactVersion
pub fn get_item_for_version<'a, T>(
    exact_version: &UnityVersion,
    items: &'a [(UnityVersion, Option<T>)],
) -> Result<&'a T, String>
where
    T: Clone,
{
    let mut result: Option<&T> = None;

    for (version, item) in items {
        if exact_version >= version {
            // Update result if item exists
            if let Some(ref i) = item {
                result = Some(i);
            }
        } else {
            // Versions are sorted, so we can break early
            break;
        }
    }

    result.ok_or_else(|| "Could not find item for version".to_string())
}

/// Represents a node in Unity's type tree
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TpkUnityNode {
    pub type_name: u16,      // Index into string buffer
    pub name: u16,           // Index into string buffer
    pub byte_size: i32,      // Size in bytes (-1 if variable)
    pub version: i16,        // Version number
    pub type_flags: i8,      // Type flags
    pub meta_flag: u32,      // Metadata flags
    pub sub_nodes: Vec<u16>, // Indices of child nodes
}

impl TpkUnityNode {
    /// Reads a TpkUnityNode from a binary stream
    pub fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut header_buf = [0u8; 17];
        reader.read_exact(&mut header_buf)?;

        let type_name = u16::from_le_bytes([header_buf[0], header_buf[1]]);
        let name = u16::from_le_bytes([header_buf[2], header_buf[3]]);
        let byte_size =
            i32::from_le_bytes([header_buf[4], header_buf[5], header_buf[6], header_buf[7]]);
        let version = i16::from_le_bytes([header_buf[8], header_buf[9]]);
        let type_flags = header_buf[10] as i8;
        let meta_flag = u32::from_le_bytes([
            header_buf[11],
            header_buf[12],
            header_buf[13],
            header_buf[14],
        ]);
        let count = u16::from_le_bytes([header_buf[15], header_buf[16]]);

        // Read sub-nodes (count * u16)
        let mut sub_nodes = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let mut node_buf = [0u8; 2];
            reader.read_exact(&mut node_buf)?;
            sub_nodes.push(u16::from_le_bytes(node_buf));
        }

        Ok(TpkUnityNode {
            type_name,
            name,
            byte_size,
            version,
            type_flags,
            meta_flag,
            sub_nodes,
        })
    }
}

// Implement Hash for use in HashMaps
impl std::hash::Hash for TpkUnityNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.type_name.hash(state);
        self.name.hash(state);
        self.byte_size.hash(state);
        self.version.hash(state);
        self.type_flags.hash(state);
        self.meta_flag.hash(state);
        self.sub_nodes.hash(state);
    }
}

////////////////////////////////////////////////////////////////////////////////////
//
// Tpk Structs
//
////////////////////////////////////////////////////////////////////////////////////

/// Buffer containing string data
#[derive(Debug, Clone)]
pub struct TpkStringBuffer {
    pub strings: Vec<String>,
}

impl TpkStringBuffer {
    /// Reads a TpkStringBuffer from a binary stream
    pub fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        // Read count (i32)
        let mut count_buf = [0u8; 4];
        reader.read_exact(&mut count_buf)?;
        let count = i32::from_le_bytes(count_buf);

        // Read strings
        let mut strings = Vec::with_capacity(count as usize);
        for _ in 0..count {
            strings.push(read_string(reader)?);
        }

        Ok(TpkStringBuffer { strings })
    }

    /// Returns the number of strings
    pub fn count(&self) -> usize {
        self.strings.len()
    }
}

/// Buffer containing Unity type tree nodes
#[derive(Debug, Clone)]
pub struct TpkUnityNodeBuffer {
    pub nodes: Vec<TpkUnityNode>,
}

impl TpkUnityNodeBuffer {
    /// Reads a TpkUnityNodeBuffer from a binary stream
    pub fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        // Read count (i32)
        let mut count_buf = [0u8; 4];
        reader.read_exact(&mut count_buf)?;
        let count = i32::from_le_bytes(count_buf);

        // Read nodes
        let mut nodes = Vec::with_capacity(count as usize);
        for _ in 0..count {
            nodes.push(TpkUnityNode::from_reader(reader)?);
        }

        Ok(TpkUnityNodeBuffer { nodes })
    }
}

// Implement Index trait for array-like access
impl std::ops::Index<usize> for TpkUnityNodeBuffer {
    type Output = TpkUnityNode;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

/// Represents a Unity class definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TpkUnityClass {
    pub name: u16,                      // Index into string buffer
    pub base: u16,                      // Base class index
    pub flags: TpkUnityClassFlags,      // Class flags
    pub editor_root_node: Option<u16>,  // Editor node index (if HasEditorRootNode flag set)
    pub release_root_node: Option<u16>, // Release node index (if HasReleaseRootNode flag set)
}

impl TpkUnityClass {
    /// Reads a TpkUnityClass from a binary stream
    pub fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        // Read fixed header: u16 + u16 + i8 = 5 bytes
        let mut header_buf = [0u8; 5];
        reader.read_exact(&mut header_buf)?;

        let name = u16::from_le_bytes([header_buf[0], header_buf[1]]);
        let base = u16::from_le_bytes([header_buf[2], header_buf[3]]);
        let flags_value = header_buf[4];

        let flags = TpkUnityClassFlags::from_bits(flags_value).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid TpkUnityClassFlags: {}", flags_value),
            )
        })?;

        // Read optional fields based on flags
        let editor_root_node = if flags.contains(TpkUnityClassFlags::HAS_EDITOR_ROOT_NODE) {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf)?;
            Some(u16::from_le_bytes(buf))
        } else {
            None
        };

        let release_root_node = if flags.contains(TpkUnityClassFlags::HAS_RELEASE_ROOT_NODE) {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf)?;
            Some(u16::from_le_bytes(buf))
        } else {
            None
        };

        Ok(TpkUnityClass {
            name,
            base,
            flags,
            editor_root_node,
            release_root_node,
        })
    }
}

// Implement Hash for use in HashMaps
impl std::hash::Hash for TpkUnityClass {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.base.hash(state);
        self.flags.hash(state);
        self.editor_root_node.hash(state);
        self.release_root_node.hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct TpkClassInformation {
    pub id: i32,
    pub classes: Vec<(UnityVersion, Option<TpkUnityClass>)>,
}

impl TpkClassInformation {
    pub fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        // Read class ID
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        let id = i32::from_le_bytes(buf);

        // Read count
        reader.read_exact(&mut buf)?;
        let count = i32::from_le_bytes(buf);

        // Read classes
        let mut classes = Vec::new();
        for _ in 0..count {
            let version = UnityVersion::from_reader(reader)?;

            // Read boolean flag (1 byte)
            let mut flag = [0u8; 1];
            reader.read_exact(&mut flag)?;

            let class = if flag[0] != 0 {
                Some(TpkUnityClass::from_reader(reader)?)
            } else {
                None
            };

            classes.push((version, class));
        }

        Ok(TpkClassInformation { id, classes })
    }

    pub fn get_versioned_class(&self, version: &UnityVersion) -> Result<&TpkUnityClass, String> {
        get_item_for_version(version, &self.classes)
    }
}

#[derive(Debug, Clone)]
pub struct TpkCommonString {
    pub version_information: Vec<(UnityVersion, u8)>,
    pub string_buffer_indices: Vec<u16>,
}

impl TpkCommonString {
    pub fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        // Read version count
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        let version_count = i32::from_le_bytes(buf);

        // Read version information
        let mut version_information = Vec::new();
        for _ in 0..version_count {
            let version = UnityVersion::from_reader(reader)?;

            // Read single byte
            let mut byte = [0u8; 1];
            reader.read_exact(&mut byte)?;

            version_information.push((version, byte[0]));
        }

        // Read indices count
        reader.read_exact(&mut buf)?;
        let indices_count = i32::from_le_bytes(buf);

        // Read indices (u16 values)
        let mut string_buffer_indices = Vec::new();
        for _ in 0..indices_count {
            let mut u16_buf = [0u8; 2];
            reader.read_exact(&mut u16_buf)?;
            string_buffer_indices.push(u16::from_le_bytes(u16_buf));
        }

        Ok(TpkCommonString {
            version_information,
            string_buffer_indices,
        })
    }

    /// Extracts all common strings from the string buffer
    pub fn get_strings(&self, string_buffer: &TpkStringBuffer) -> Vec<String> {
        self.string_buffer_indices
            .iter()
            .filter_map(|&index| {
                if (index as usize) < string_buffer.strings.len() {
                    Some(string_buffer.strings[index as usize].clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_count(&self, exact_version: &UnityVersion) -> Result<u8, String> {
        let mut result = None;
        for (version, count) in &self.version_information {
            if exact_version >= version {
                result = Some(*count);
            } else {
                break;
            }
        }
        result.ok_or_else(|| "Could not find exact version".to_string())
    }
}

#[derive(Debug)]
pub enum TpkDataBlob {
    TypeTree(TpkTypeTreeBlob),
    Collection(TpkCollectionBlob),
    FileSystem(TpkFileSystemBlob),
    Json(TpkJsonBlob),
}

#[derive(Debug, Clone)]
pub struct TpkTypeTreeBlob {
    pub creation_time: i64,
    pub versions: Vec<UnityVersion>,
    pub class_information: std::collections::HashMap<i32, TpkClassInformation>,
    pub common_string: TpkCommonString,
    pub node_buffer: TpkUnityNodeBuffer,
    pub string_buffer: TpkStringBuffer,
}

impl TpkTypeTreeBlob {
    pub fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        // Read creation time
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf)?;
        let creation_time = i64::from_le_bytes(buf);

        // Read versions
        let mut buf4 = [0u8; 4];
        reader.read_exact(&mut buf4)?;
        let version_count = i32::from_le_bytes(buf4);

        let mut versions = Vec::new();
        for _ in 0..version_count {
            versions.push(UnityVersion::from_reader(reader)?);
        }

        // Read class information
        reader.read_exact(&mut buf4)?;
        let class_count = i32::from_le_bytes(buf4);

        let mut class_information = std::collections::HashMap::new();
        for _ in 0..class_count {
            let class_info = TpkClassInformation::from_reader(reader)?;
            class_information.insert(class_info.id, class_info);
        }

        // Read remaining components
        let common_string = TpkCommonString::from_reader(reader)?;
        let node_buffer = TpkUnityNodeBuffer::from_reader(reader)?;
        let string_buffer = TpkStringBuffer::from_reader(reader)?;

        Ok(TpkTypeTreeBlob {
            creation_time,
            versions,
            class_information,
            common_string,
            node_buffer,
            string_buffer,
        })
    }
}

#[derive(Debug)]
pub struct TpkCollectionBlob {
    pub blobs: Vec<(String, TpkDataBlob)>,
}

impl TpkCollectionBlob {
    pub fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        let count = i32::from_le_bytes(buf);

        let mut blobs = Vec::new();
        for _ in 0..count {
            let path = read_string(reader)?;

            // Read data type as single byte
            let mut type_buf = [0u8; 1];
            reader.read_exact(&mut type_buf)?;
            let data_type = TpkDataType::from_u8(type_buf[0])?;

            // Convert to blob
            let blob = data_type.to_blob(reader)?;
            blobs.push((path, blob));
        }

        Ok(TpkCollectionBlob { blobs })
    }
}

#[derive(Debug)]
pub struct TpkFileSystemBlob {
    pub files: Vec<(String, Vec<u8>)>,
}

impl TpkFileSystemBlob {
    pub fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        let count = i32::from_le_bytes(buf);

        let mut files = Vec::new();
        for _ in 0..count {
            let path = read_string(reader)?;
            let data = read_data(reader)?;
            files.push((path, data));
        }

        Ok(TpkFileSystemBlob { files })
    }
}

#[derive(Debug)]
pub struct TpkJsonBlob {
    pub text: String,
}

impl TpkJsonBlob {
    pub fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let text = read_string(reader)?;
        Ok(TpkJsonBlob { text })
    }
}

/// Main TPK file structure
#[derive(Debug)]
pub struct TpkFile {
    pub compression_type: TpkCompressionType,
    pub data_type: TpkDataType,
    pub compressed_size: i32,
    pub uncompressed_size: i32,
    pub compressed_bytes: Vec<u8>,
}

impl TpkFile {
    pub const TPK_MAGIC_BYTES: u32 = 0x2A4B5054; // "TPK*" in little-endian
    pub const TPK_VERSION_NUMBER: i8 = 1;

    /// Reads a TPK file from a binary stream
    pub fn from_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        // Read header: u32 + 4 bytes + u32 (reserved) + 2 u32s = 20 bytes total
        let mut header = [0u8; 20];
        reader.read_exact(&mut header)?;

        // Parse header
        let magic = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
        let version_number = header[4] as i8;
        let compression_type_byte = header[5];
        let data_type_byte = header[6];
        // header[7] is reserved (1 byte)
        // header[8-11] is reserved (4 bytes) - second reserved field
        let compressed_size = i32::from_le_bytes([header[12], header[13], header[14], header[15]]);
        let uncompressed_size =
            i32::from_le_bytes([header[16], header[17], header[18], header[19]]);

        // Validate magic bytes
        if magic != Self::TPK_MAGIC_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid TPK magic bytes: 0x{:08X}", magic),
            ));
        }

        // Validate version number
        if version_number != Self::TPK_VERSION_NUMBER {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid TPK version number: {}", version_number),
            ));
        }

        // Parse enums
        let compression_type = TpkCompressionType::from_u8(compression_type_byte)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let data_type = TpkDataType::from_u8(data_type_byte)?;

        // Read compressed data
        // Special case: If compression_type is None and compressed_size is 0,
        // the actual data size is uncompressed_size (file is already uncompressed)
        let data_size = if compression_type == TpkCompressionType::None && compressed_size == 0 {
            uncompressed_size
        } else {
            compressed_size
        };

        if data_size < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid data size: {}", data_size),
            ));
        }

        let mut compressed_bytes = vec![0u8; data_size as usize];
        reader.read_exact(&mut compressed_bytes)?;

        Ok(TpkFile {
            compression_type,
            data_type,
            compressed_size,
            uncompressed_size,
            compressed_bytes,
        })
    }

    /// Decompresses the data and returns the appropriate blob
    pub fn get_data_blob(&self) -> std::io::Result<TpkDataBlob> {
        // Decompress based on compression type
        let decompressed = match self.compression_type {
            TpkCompressionType::None => {
                // No compression - use data as-is
                self.compressed_bytes.clone()
            }
            TpkCompressionType::Lz4 => {
                lz4::block::decompress(&self.compressed_bytes, Some(self.uncompressed_size))
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
            }
            TpkCompressionType::Lzma => {
                let mut output = Vec::new();
                lzma_rs::lzma_decompress(
                    &mut std::io::Cursor::new(&self.compressed_bytes),
                    &mut output,
                )
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                output
            }
            TpkCompressionType::Brotli => {
                let mut output = Vec::new();
                brotli::BrotliDecompress(
                    &mut std::io::Cursor::new(&self.compressed_bytes),
                    &mut output,
                )
                .map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Brotli error: {}", e),
                    )
                })?;
                output
            }
        };

        // Parse the decompressed data as a blob
        let mut cursor = std::io::Cursor::new(decompressed);
        self.data_type.to_blob(&mut cursor)
    }
}

/// Initializes the TPK module by loading the uncompressed.tpk resource file
///
/// Note: In Python this loads from embedded resources. In Rust you'll need to
/// provide the path to the uncompressed.tpk file or embed it using include_bytes!
pub fn init_from_bytes(tpk_data: &[u8]) -> std::io::Result<()> {
    let mut cursor = std::io::Cursor::new(tpk_data);
    let tpk_file = TpkFile::from_reader(&mut cursor)?;
    let blob = tpk_file.get_data_blob()?;

    // Ensure it's a TypeTree blob
    match blob {
        TpkDataBlob::TypeTree(type_tree_blob) => {
            let mut tree = TPK_TYPE_TREE.lock().unwrap();
            *tree = Some(type_tree_blob);
            Ok(())
        }
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Expected TypeTree blob in TPK file",
        )),
    }
}

/// Gets the loaded TpkTypeTreeBlob
///
/// Returns the TypeTree blob that was loaded via init_from_bytes.
pub fn get_tpk_typetree() -> Result<TpkTypeTreeBlob, String> {
    let tree_guard = TPK_TYPE_TREE.lock().unwrap();
    match &*tree_guard {
        Some(blob) => Ok(blob.clone()),
        None => Err("TPK TypeTree not initialized".to_string()),
    }
}

/// Gets a TypeTreeNode for a specific Unity class ID and version
///
/// This function uses caching to avoid regenerating the same nodes
pub fn get_typetree_node(
    class_id: i32,
    version: (u16, u16, u16, u16),
) -> Result<TypeTreeNode, String> {
    // Check cache first
    {
        let cache = CLASSES_CACHE.lock().unwrap();
        if let Some(node) = cache.get(&(class_id, version)) {
            return Ok(node.clone());
        }
    }

    // Not in cache - need to generate it
    let tree_guard = TPK_TYPE_TREE.lock().unwrap();
    let tree = tree_guard
        .as_ref()
        .ok_or("TPK module not initialized - call init_from_bytes() first")?;

    // Get class information
    let class_info = tree
        .class_information
        .get(&class_id)
        .ok_or(format!("Class ID {} not found", class_id))?;

    // Get versioned class
    let unity_version = UnityVersion::from_parts(
        version.0 as u64,
        version.1 as u64,
        version.2 as u64,
        version.3 as u64,
    );
    let class = class_info.get_versioned_class(&unity_version)?;

    // Generate the node
    let node = generate_node(class, tree)?;

    // Cache it
    {
        let mut cache = CLASSES_CACHE.lock().unwrap();
        cache.insert((class_id, version), node.clone());
    }

    Ok(node)
}

/// Generates a TypeTreeNode from a TpkUnityClass
///
/// This recursively builds the tree by following node references
fn generate_node(class: &TpkUnityClass, tree: &TpkTypeTreeBlob) -> Result<TypeTreeNode, String> {
    // Check cache first
    {
        let cache = NODES_CACHE.lock().unwrap();
        if let Some(node) = cache.get(class) {
            return Ok(node.clone());
        }
    }

    // Get the root node index
    let root_node_idx = class
        .release_root_node
        .ok_or("Class has no release root node")?;

    // Build flat list of nodes using BFS
    let mut nodes_list = Vec::new();
    let mut queue: Vec<(u16, i32)> = vec![(root_node_idx, 0)]; // (node_id, level)

    while let Some((node_id, level)) = queue.first().copied() {
        queue.remove(0); // Pop front (BFS)

        let tpk_node = &tree.node_buffer.nodes[node_id as usize];

        // Create TypeTreeNode
        let type_name = tree.string_buffer.strings[tpk_node.type_name as usize].clone();
        let name = tree.string_buffer.strings[tpk_node.name as usize].clone();

        let tree_node = TypeTreeNode::new(
            level,
            type_name,
            name,
            tpk_node.byte_size,
            tpk_node.version as i32,
        );

        // Set optional fields (would need to modify TypeTreeNode::new or add setters)
        // For now, we'll use the basic constructor
        nodes_list.push(tree_node);

        // Add children to queue
        for &sub_node_id in &tpk_node.sub_nodes {
            queue.push((sub_node_id, level + 1));
        }
    }

    // Build tree from flat list
    let result = TypeTreeNode::from_list(nodes_list)?;

    // Cache it
    {
        let mut cache = NODES_CACHE.lock().unwrap();
        cache.insert(class.clone(), result.clone());
    }

    Ok(result)
}

// Returns the list of common strings from the TPK tree
///
/// # Arguments
///
/// * `version` - Optional Unity version to filter strings
///
/// # Errors
///
/// Returns an error if the TPK tree hasn't been initialized
pub fn get_common_string_list(version: Option<UnityVersion>) -> Result<Vec<String>, String> {
    let tree_guard = TPK_TYPE_TREE.lock().unwrap();
    let tree = tree_guard
        .as_ref()
        .ok_or("TPK tree not initialized - call init_from_bytes() first")?;

    // Get strings from common string buffer
    let mut strings = tree.common_string.get_strings(&tree.string_buffer);

    // Filter by version if specified
    if let Some(v) = &version {
        let count = tree.common_string.get_count(v)?;
        strings = strings.into_iter().take(count as usize).collect();
    }

    Ok(strings)
}
