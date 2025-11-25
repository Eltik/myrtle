use crate::enums::file_type::FileType as FileTypeEnum;
use crate::files::bundle_file::{BundleFile, FileType};
use crate::files::file::ParentFile;
use crate::files::serialized_file::SerializedFile;
use crate::files::web_file::WebFile;
use crate::helpers::compression_helper::{BROTLI_MAGIC, GZIP_MAGIC};
use crate::streams::endian::Endian;
use crate::streams::endian_reader::{BinaryReader, MemoryReader};
use std::cell::RefCell;
use std::fs;
use std::io;
use std::path::Path;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub enum FileSource {
    /// File path as string
    Path(String),
    /// Raw bytes data
    Bytes(Vec<u8>),
}

/// Returns the file name without extension
///
/// # Arguments
///
/// * `file_name` - The file path
///
/// # Returns
///
/// Path with directory and filename without extension
///
/// Python equivalent: lines 15-18
/// Python: os.path.join(os.path.dirname(file_name), os.path.splitext(os.path.basename(file_name))[0])
pub fn file_name_without_extension(file_name: &str) -> String {
    // dirname: everything before the last slash (or empty if no slash)
    let dirname = if let Some(last_slash) = file_name.rfind('/') {
        &file_name[..=last_slash]
    } else {
        ""
    };

    // basename: everything after the last slash (or entire string if no slash)
    let basename = if let Some(last_slash) = file_name.rfind('/') {
        &file_name[last_slash + 1..]
    } else {
        file_name
    };

    // splitext: split at the last '.' in basename
    // Special cases: "." and ".." have no extension
    // ".hidden" has no extension (starts with . and has no other dots)
    let name_without_ext = if basename == "." || basename == ".." {
        basename
    } else if let Some(dot_pos) = basename.rfind('.') {
        if dot_pos == 0 {
            // ".hidden" -> no extension
            basename
        } else {
            &basename[..dot_pos]
        }
    } else {
        basename
    };

    // join: concatenate dirname and name
    format!("{}{}", dirname, name_without_ext)
}

/// Recursively lists all files in a directory (excludes .git)
///
/// # Arguments
///
/// * `directory` - Root directory path
///
/// # Returns
///
/// Vector of all file paths
///
/// Python equivalent: lines 21-30
pub fn list_all_files(directory: &str) -> Result<Vec<String>, io::Error> {
    let mut files = Vec::new();
    fn visit_dirs(dir: &Path, files: &mut Vec<String>) -> io::Result<()> {
        if dir.is_dir() {
            // Skip .git directories
            if dir.file_name().and_then(|n| n.to_str()) == Some(".git") {
                return Ok(());
            }

            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    visit_dirs(&path, files)?;
                } else {
                    files.push(path.to_string_lossy().to_string());
                }
            }
        }
        Ok(())
    }

    visit_dirs(Path::new(directory), &mut files)?;
    Ok(files)
}

/// Finds all files in a directory matching a search string
///
/// # Arguments
///
/// * `directory` - Root directory path
/// * `search_str` - String to search for in filenames
///
/// # Returns
///
/// Vector of matching file paths
///
/// Python equivalent: lines 33-46
pub fn find_all_files(directory: &str, search_str: &str) -> Result<Vec<String>, io::Error> {
    let mut files = Vec::new();

    fn visit_dirs(dir: &Path, search: &str, files: &mut Vec<String>) -> io::Result<()> {
        if dir.is_dir() {
            // Skip .git directories
            if dir.file_name().and_then(|n| n.to_str()) == Some(".git") {
                return Ok(());
            }

            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    visit_dirs(&path, search, files)?;
                } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.contains(search) {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
        Ok(())
    }

    visit_dirs(Path::new(directory), search_str, &mut files)?;
    Ok(files)
}

/// Finds a path with case-insensitive matching
///
/// # Arguments
///
/// * `dir` - Base directory
/// * `insensitive_path` - Path to search for (case-insensitive)
///
/// # Returns
///
/// The actual path with correct casing, or None if not found
///
/// Python equivalent: lines 157-171
pub fn find_sensitive_path(dir: &str, insensitive_path: &str) -> Option<String> {
    let parts: Vec<&str> = insensitive_path
        .trim_matches(std::path::MAIN_SEPARATOR)
        .split(std::path::MAIN_SEPARATOR)
        .collect();

    let mut sensitive_path = Path::new(dir).to_path_buf();

    for part in parts {
        let part_lower = part.to_lowercase();

        // Find matching entry (case-insensitive)
        let found = fs::read_dir(&sensitive_path)
            .ok()?
            .filter_map(|entry| entry.ok())
            .find(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .map(|n| n.to_lowercase() == part_lower)
                    .unwrap_or(false)
            })?;

        sensitive_path = found.path();
    }

    Some(sensitive_path.to_string_lossy().to_string())
}

/// Detects file type by reading signature bytes
///
/// # Arguments
///
/// * `reader` - Binary reader positioned at file start
///
/// # Returns
///
/// Tuple of (FileType, reader position reset to 0)
///
/// Python equivalent: lines 49-128
pub fn check_file_type<R: BinaryReader>(reader: &mut R) -> Result<FileTypeEnum, io::Error> {
    // Need at least 20 bytes to check signature
    if reader.len() < 20 {
        return Ok(FileTypeEnum::ResourceFile);
    }

    // Read signature as bytes (Python's read_string_to_null doesn't enforce UTF-8)
    reader.set_position(0);
    let sig_bytes = reader.read_bytes(20)?;

    // Find null terminator and convert to string (lossy to handle invalid UTF-8)
    let null_pos = sig_bytes.iter().position(|&b| b == 0).unwrap_or(20);
    let signature = String::from_utf8_lossy(&sig_bytes[..null_pos]).to_string();

    // Reset to start
    reader.set_position(0);

    // Check for BundleFile signatures
    if signature == "UnityWeb"
        || signature == "UnityRaw"
        || signature == "UnityFS"
        || signature.starts_with("\u{fa}\u{fa}\u{fa}\u{fa}\u{fa}\u{fa}\u{fa}\u{fa}")
    {
        return Ok(FileTypeEnum::BundleFile);
    }

    // Check for WebFile signatures
    if signature.starts_with("UnityWebData") || signature.starts_with("TuanjieWebData") {
        return Ok(FileTypeEnum::WebFile);
    }

    // Check for ZIP signature
    if signature.starts_with("PK\x03\x04") {
        return Ok(FileTypeEnum::ZIP);
    }

    // Not enough data for further checks
    if reader.len() < 128 {
        return Ok(FileTypeEnum::ResourceFile);
    }

    // Check for GZIP magic
    reader.set_position(0);
    let magic = reader.read_bytes(2)?;
    reader.set_position(0);
    if magic == GZIP_MAGIC {
        return Ok(FileTypeEnum::WebFile);
    }

    // Check for Brotli magic at 0x20
    reader.set_position(0x20);
    let magic = reader.read_bytes(6)?;
    reader.set_position(0);
    if magic == BROTLI_MAGIC {
        return Ok(FileTypeEnum::WebFile);
    }

    // Check if it's an AssetsFile by validating header
    reader.set_position(0);

    // Try with current endianness first, then try opposite if it fails
    let endianness_to_try = [reader.endian(), !reader.endian()];

    for try_endian in &endianness_to_try {
        reader.set_position(0);
        reader.set_endian(*try_endian);

        // Read header fields
        let mut metadata_size = reader.read_u32()? as u64;
        let mut file_size = reader.read_u32()? as u64;
        let version = reader.read_u32()?;
        let mut data_offset = reader.read_u32()? as u64;

        log::debug!(
            "check_file_type: Checking AssetsFile header with {:?} - version={}, metadata_size={}, file_size={}, data_offset={}, reader_len={}",
            try_endian, version, metadata_size, file_size, data_offset, reader.len()
        );

        // For version >= 22, read extended header (Python lines 99-110)
        if version >= 22 {
            // Read endianness byte but DON'T apply it yet (matches SerializedFile::new)
            let raw_endian = reader.read_u8()?;
            let _endian_value = if raw_endian == 0 {
                Endian::Little
            } else {
                Endian::Big
            };

            // Read 3 reserved bytes
            let _reserved = reader.read_bytes(3)?;

            // Read extended header with SAME endianness as initial header
            // (SerializedFile::new doesn't call set_endian until line 624, after reading extended header)
            metadata_size = reader.read_u32()? as u64;
            file_size = reader.read_u64()?;
            data_offset = reader.read_u64()?;
            let _unknown = reader.read_i64()?;

            log::debug!(
                "check_file_type: Extended header (v>=22) - metadata_size={}, file_size={}, data_offset={}",
                metadata_size, file_size, data_offset
            );
        }

        // Validate header sanity (Python lines 115-125)
        // Python checks: version < 0, version > 100, each value in range [0, reader.Length],
        // file_size >= metadata_size, file_size >= data_offset
        //
        // NOTE: For Unity files with external resources (.resS files), file_size and data_offset
        // can be larger than reader_len. Only validate that metadata_size is reasonable and
        // that the fields are internally consistent.
        let reader_len = reader.len() as u64;
        let is_invalid = version > 100
            || metadata_size > reader_len
            || file_size < metadata_size
            || file_size < data_offset;

        log::debug!(
            "check_file_type: Validation result - is_invalid={} (version>100:{}, metadata_size>len:{}, file_size<metadata:{}, file_size<data:{})",
            is_invalid,
            version > 100,
            metadata_size > reader_len,
            file_size < metadata_size,
            file_size < data_offset
        );

        if !is_invalid {
            // Valid AssetsFile found! Reset and return
            reader.set_position(0);
            reader.set_endian(*try_endian);
            return Ok(FileTypeEnum::AssetsFile);
        }

        // Invalid, try next endianness
        log::debug!(
            "check_file_type: Invalid with {:?}, trying next endianness",
            try_endian
        );
    }

    // Neither endianness worked, return ResourceFile
    reader.set_position(0);
    Ok(FileTypeEnum::ResourceFile)
}

/// Parses a file and creates the appropriate file object
///
/// # Arguments
///
/// * `reader` - Binary reader containing file data
/// * `parent` - Parent file (for nested files)
/// * `name` - File name
/// * `typ` - Optional file type (will auto-detect if None)
/// * `is_dependency` - Whether this is a dependency file
///
/// # Returns
///
/// FileType enum (either parsed file or raw reader)
///
/// Python equivalent: lines 131-154
pub fn parse_file<R: BinaryReader + 'static>(
    mut reader: R,
    name: String,
    typ: Option<FileTypeEnum>,
    is_dependency: bool,
    parent: Option<Weak<RefCell<dyn ParentFile>>>,
) -> Result<FileType, io::Error> {
    // Auto-detect type if not provided
    let file_type = if let Some(t) = typ {
        t
    } else {
        check_file_type(&mut reader)?
    };

    log::debug!(
        "parse_file: '{}' detected as {:?} (len: {})",
        name,
        file_type,
        reader.len()
    );

    // Skip parsing for certain file extensions (even if detected as AssetsFile)
    let skip_extensions = [".resS", ".resource", ".config", ".xml", ".dat"];
    let should_skip = skip_extensions.iter().any(|ext| name.ends_with(ext));

    match file_type {
        FileTypeEnum::AssetsFile if !should_skip => {
            // Create SerializedFile
            let bytes = reader.bytes().to_vec();
            let endian = reader.endian();
            let memory_reader = MemoryReader::new(bytes, endian, 0);
            let serialized_file = SerializedFile::new(memory_reader, Some(name), is_dependency)?;
            let serialized_rc = Rc::new(RefCell::new(serialized_file));
            SerializedFile::set_object_parents(&serialized_rc)?;

            // Set parent and is_dependency while keeping the Rc alive
            {
                let mut sf_mut = serialized_rc.borrow_mut();
                sf_mut.parent = parent;
                sf_mut.is_dependency = is_dependency;
            }

            Ok(FileType::SerializedFile(serialized_rc))
        }
        FileTypeEnum::BundleFile => {
            // BundleFile format uses big-endian (Python's EndianBinaryReader defaults to ">")
            reader.set_endian(Endian::Big);

            let (bundle_file, blocks_reader, m_directory_info) =
                BundleFile::new(&mut reader, Some(name))?;
            let bundle_rc = Rc::new(RefCell::new(bundle_file));
            BundleFile::load_files(&bundle_rc, blocks_reader, m_directory_info)?;

            let mut bundle_final = Rc::try_unwrap(bundle_rc)
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to unwrap BundleFile"))?
                .into_inner();

            bundle_final.parent = parent;
            bundle_final.is_dependency = is_dependency;

            Ok(FileType::BundleFile(Box::new(bundle_final)))
        }
        FileTypeEnum::WebFile => {
            let mut web_file = WebFile::new(&mut reader, Some(name))?;
            web_file.parent = parent;
            web_file.is_dependency = is_dependency;
            Ok(FileType::WebFile(Box::new(web_file)))
        }
        _ => {
            // Return raw reader for ResourceFile, ZIP, etc.
            let bytes = reader.bytes().to_vec();
            let mem_reader = MemoryReader::new(bytes, reader.endian(), 0);
            Ok(FileType::Raw(mem_reader))
        }
    }
}
