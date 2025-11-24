use super::bundle_file::{BundleDirectoryEntry, FileType};
use super::file::File;
use super::file::ParentFile;
use crate::helpers::compression_helper::{
    compress_brotli, compress_gzip, decompress_brotli, decompress_gzip, BROTLI_MAGIC, GZIP_MAGIC,
};
use crate::streams::endian::Endian;
use crate::streams::endian_reader::BinaryReader;
use crate::streams::endian_reader::MemoryReader;
use crate::streams::endian_writer::{BinaryWriter, EndianBinaryWriter};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::rc::Rc;
use std::rc::Weak;

/// WebFile - Unity web data package handler
///
/// Handles UnityWebData and TuanjieWebData packages with optional compression (gzip/brotli)
#[derive(Debug)]
pub struct WebFile {
    pub signature: String, // "UnityWebData1.0" or "TuanjieWebData1.0"
    pub packer: String,    // "gzip", "brotli", or "none"

    // File trait fields
    pub name: String,
    pub is_changed: bool,
    pub files: HashMap<String, Rc<RefCell<FileType>>>,

    pub parent: Option<Weak<RefCell<dyn ParentFile>>>,
    pub is_dependency: bool,

    pub file_flags: HashMap<String, u32>,
}

impl WebFile {
    /// Creates a new WebFile from a binary reader
    ///
    /// # Arguments
    ///
    /// * `reader` - Binary reader containing the WebFile data
    /// * `name` - Optional file name
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Compression decompression fails
    /// - Signature is invalid
    /// - Header parsing fails
    pub fn new<R: BinaryReader>(reader: &mut R, name: Option<String>) -> Result<Self, io::Error> {
        reader.set_position(0);

        let magic_0 = reader.read_bytes(2)?;
        reader.set_position(0);

        let (packer, data) = if magic_0 == GZIP_MAGIC {
            // GZIP detected
            let all_data = reader.bytes().to_vec();
            let decompressed = decompress_gzip(&all_data)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            ("gzip".to_string(), decompressed)
        } else {
            // Check for Brotli at position 0x20
            reader.set_position(0x20);
            let magic_20 = reader.read_bytes(6)?;
            reader.set_position(0);

            if magic_20 == BROTLI_MAGIC {
                let all_data = reader.bytes().to_vec();
                let decompressed = decompress_brotli(&all_data)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                ("brotli".to_string(), decompressed)
            } else {
                // No compression
                let all_data = reader.bytes().to_vec();
                ("none".to_string(), all_data)
            }
        };

        let mut data_reader = MemoryReader::new(data, Endian::Little, 0);

        let signature = data_reader.read_string_to_null(1024)?;
        if !signature.starts_with("UnityWebData") && !signature.starts_with("TuanjieWebData") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid WebFile signature: {}", signature),
            ));
        }

        let head_length = data_reader.read_i32()? as usize;

        let mut directory_info = Vec::new();
        while data_reader.position() < head_length {
            let offset = data_reader.read_i32()? as i64;
            let length = data_reader.read_i32()? as i64;
            let path_length = data_reader.read_i32()? as usize;
            let path_bytes = data_reader.read_bytes(path_length)?;
            let path = String::from_utf8(path_bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            directory_info.push(BundleDirectoryEntry {
                path,
                offset,
                size: length,
                flags: 0,
            });
        }

        let mut web_file = WebFile {
            signature,
            packer,
            name: name.unwrap_or_else(|| String::from("")),
            is_changed: false,
            files: HashMap::new(),
            parent: None,
            is_dependency: false,
            file_flags: HashMap::new(),
        };

        web_file.read_files(data_reader, directory_info)?;

        Ok(web_file)
    }

    /// Saves the WebFile to bytes
    ///
    /// # Arguments
    ///
    /// * `files` - Optional files to save (uses self.files if None)
    /// * `packer` - Compression type: "none", "gzip", or "brotli" (uses self.packer if None)
    /// * `signature` - Signature string (default: "UnityWebData1.0")
    ///
    /// # Returns
    ///
    /// The serialized WebFile as bytes
    pub fn save(
        &self,
        files: Option<&HashMap<String, Rc<RefCell<FileType>>>>,
        packer: Option<&str>,
        signature: Option<&str>,
    ) -> Result<Vec<u8>, io::Error> {
        let files_to_save = files.unwrap_or(&self.files);
        let packer = packer.unwrap_or(&self.packer);
        let signature = signature.unwrap_or("UnityWebData1.0");

        let mut file_data: HashMap<String, Vec<u8>> = HashMap::new();
        for (name, file_type) in files_to_save {
            match &*file_type.borrow() {
                FileType::Raw(reader) => {
                    file_data.insert(name.clone(), reader.bytes().to_vec());
                }
                FileType::SerializedFile(sf_rc) => {
                    let mut serialized_file = sf_rc.borrow_mut();
                    let saved_bytes = serialized_file.save().map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("Failed to save SerializedFile: {}", e),
                        )
                    })?;
                    file_data.insert(name.clone(), saved_bytes);
                }

                FileType::BundleFile(bundle) => {
                    let saved_bytes = bundle.save(None)?;
                    file_data.insert(name.clone(), saved_bytes);
                }
                FileType::WebFile(web) => {
                    let saved_bytes = web.save(None, None, None)?;
                    file_data.insert(name.clone(), saved_bytes);
                }
                FileType::Writer(writer) => {
                    file_data.insert(name.clone(), writer.bytes().to_vec());
                }
            }
        }

        let mut writer = EndianBinaryWriter::new(Endian::Little);
        writer
            .write_string_to_null(signature)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let writer_position = writer.position();
        let path_bytes_total: usize = files_to_save.keys().map(|path| path.as_bytes().len()).sum();
        let per_file_overhead = 12 * files_to_save.len();
        let fixed_offset = 4;

        let mut offset = writer_position + path_bytes_total + per_file_overhead + fixed_offset;

        writer.write_i32(offset as i32)?;

        for (name, data) in &file_data {
            writer.write_i32(offset as i32)?;

            let length = data.len();
            writer.write_i32(length as i32)?;

            offset += length;
            writer.write_i32(name.len() as i32)?;
            writer.write(name.as_bytes())?;
        }

        for data in file_data.values() {
            writer.write(data)?;
        }

        let final_bytes = writer.to_bytes();

        match packer {
            "gzip" => {
                compress_gzip(&final_bytes).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            }
            "brotli" => {
                compress_brotli(&final_bytes).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            }
            _ => Ok(final_bytes),
        }
    }
}

impl File for WebFile {
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
        "CAB-UnityPy_Mod.resS"
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
}

impl ParentFile for WebFile {
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
}
