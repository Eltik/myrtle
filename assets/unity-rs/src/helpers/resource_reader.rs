//! ResourceReader - Helper for reading external Unity resource files (.resS)
//!
//! Unity stores large assets (textures, audio) in separate .resS files.
//! This module handles finding and reading data from these resource files.

use crate::files::bundle_file::FileType;
use crate::files::serialized_file::SerializedFile;
use std::io;
use std::path::Path;

/// Gets resource data from external .resS files
///
/// # Arguments
///
/// * `res_path` - Path to the resource file
/// * `assets_file` - Mutable reference to the SerializedFile that references this resource
/// * `offset` - Byte offset in the resource file
/// * `size` - Number of bytes to read
///
/// # Returns
///
/// The resource data as a Vec<u8>
///
/// # Errors
///
/// Returns error if resource file cannot be found or read fails
///
/// # Python equivalent
/// ResourceReader.py: get_resource_data() (lines 10-35)
pub fn get_resource_data(
    res_path: &str,
    assets_file: &mut SerializedFile,
    offset: usize,
    size: usize,
) -> Result<Vec<u8>, io::Error> {
    // Extract basename from path (Python: ntpath.basename)
    let path = Path::new(res_path);
    let basename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(res_path);

    // Split name and extension (Python: ntpath.splitext)
    let (name, _ext) = match basename.rfind('.') {
        Some(pos) => (&basename[..pos], &basename[pos..]),
        None => (basename, ""),
    };

    // Generate possible filenames (Python lines 15-20)
    let possible_names = vec![
        basename.to_string(),
        format!("{}.resource", name),
        format!("{}.assets.resS", name),
        format!("{}.resS", name),
    ];

    // Get environment reference and clone the Rc (to avoid borrow conflicts)
    let environment = assets_file
        .environment
        .as_ref()
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "SerializedFile has no environment")
        })?
        .clone(); // Clone the Rc, not the Environment itself

    // Try to find resource in CAB registry (Python lines 22-26)
    let mut reader = None;
    for possible_name in &possible_names {
        if let Some(cab_reader) = environment.borrow().get_cab(possible_name) {
            reader = Some(cab_reader);
            break;
        }
    }

    // If not found, try loading dependencies (Python lines 27-32)
    if reader.is_none() {
        // Load dependencies with possible names
        let _ = assets_file
            .load_dependencies(possible_names.clone())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Try again to find resource
        for possible_name in &possible_names {
            if let Some(cab_reader) = environment.borrow().get_cab(possible_name) {
                reader = Some(cab_reader);
                break;
            }
        }
    }

    // If still not found, return error (Python lines 33-34)
    let reader = reader.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Resource file {} not found", basename),
        )
    })?;

    // Read the data at offset (Python line 35)
    get_resource_data_from_reader(&reader, offset, size)
}

/// Reads bytes from a reader at a specific offset
///
/// # Arguments
///
/// * `reader_rc` - Rc<RefCell<FileType>> containing the resource data
/// * `offset` - Byte offset to start reading from
/// * `size` - Number of bytes to read
///
/// # Returns
///
/// The data as a Vec<u8>
///
/// # Python equivalent
/// ResourceReader.py: _get_resource_data() (lines 38-40)
fn get_resource_data_from_reader(
    reader_rc: &std::rc::Rc<std::cell::RefCell<FileType>>,
    offset: usize,
    size: usize,
) -> Result<Vec<u8>, io::Error> {
    use crate::streams::endian_reader::BinaryReader;

    // Borrow the FileType and extract the MemoryReader
    let file_type = reader_rc.borrow();

    match &*file_type {
        FileType::Raw(memory_reader) => {
            // Python line 39: reader.Position = offset
            // Python line 40: return reader.read_bytes(size)

            // Since MemoryReader doesn't implement Clone, we get the bytes directly
            let all_bytes = memory_reader.bytes();

            // Validate offset and size
            if offset + size > all_bytes.len() {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    format!(
                        "Attempted to read {} bytes at offset {}, but only {} bytes available",
                        size,
                        offset,
                        all_bytes.len()
                    ),
                ));
            }

            // Return the slice
            Ok(all_bytes[offset..offset + size].to_vec())
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Expected Raw file type for resource file",
        )),
    }
}
