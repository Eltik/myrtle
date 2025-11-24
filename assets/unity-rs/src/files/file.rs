use super::bundle_file::{BundleDirectoryEntry, FileType};
use crate::enums::class_id_type::ClassIDType;
use crate::environment::Environment;
use crate::files::ObjectReader;
use crate::helpers::import_helper;
use crate::streams::endian::Endian;
use crate::streams::endian_reader::BinaryReader;
use crate::streams::endian_reader::MemoryReader;
use crate::streams::endian_writer::BinaryWriter;
use crate::streams::endian_writer::EndianBinaryWriter;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::rc::Rc;

pub trait ParentFile: fmt::Debug {
    /// Find a file by name (case-insensitive) in this parent's files collection
    ///
    /// # Arguments
    /// * `name` - File name to search for (will be compared case-insensitively)
    ///
    /// # Returns
    /// Reference-counted pointer to the file if found, None otherwise
    fn find_file(&self, name: &str) -> Option<Rc<RefCell<FileType>>>;

    /// Mark this file as changed
    fn mark_changed(&mut self);

    /// Creates or retrieves a writeable CAB file for resource data
    ///
    /// Python equivalent: File.get_writeable_cab() (lines 96-127)
    fn get_writeable_cab(
        &mut self,
        name: Option<&str>,
    ) -> Result<Option<Rc<RefCell<FileType>>>, io::Error>;

    /// Get the version_engine from the parent file (if applicable)
    ///
    /// Returns None for files that don't have a version_engine field.
    /// BundleFile overrides this to return its version_engine.
    fn get_version_engine(&self) -> Option<String> {
        None
    }
}

pub trait File: fmt::Debug {
    fn name(&self) -> &str;
    fn is_changed(&self) -> bool;
    fn mark_changed(&mut self);
    fn is_dependency(&self) -> bool;
    fn cab_file(&self) -> &str;

    fn files_mut(&mut self) -> &mut HashMap<String, Rc<RefCell<FileType>>> {
        panic!("files_mut() not implemented for this File type")
    }

    fn files(&self) -> &HashMap<String, Rc<RefCell<FileType>>> {
        panic!("files() not implemented for this File type")
    }

    fn file_flags_mut(&mut self) -> &mut HashMap<String, u32> {
        panic!("file_flags_mut() not implemented for this File type")
    }

    fn file_flags(&self) -> &HashMap<String, u32> {
        panic!("file_flags() not implemented for this File type")
    }

    fn environment(&self) -> Option<Rc<RefCell<Environment>>> {
        None
    }

    // Dictionary-like access methods (Python File.py lines 138-148)

    /// Returns the file attribute by key, with optional default value
    /// Python equivalent: getattr(self, key, default)
    fn get(&self, key: &str, default: Option<String>) -> Option<String> {
        match key {
            "name" => Some(self.name().to_string()),
            "is_changed" => Some(self.is_changed().to_string()),
            "is_dependency" => Some(self.is_dependency().to_string()),
            "cab_file" => Some(self.cab_file().to_string()),
            _ => default,
        }
    }

    /// Returns an iterator over file names (HashMap keys)
    /// Python equivalent: self.files.keys()
    fn keys(&self) -> std::collections::hash_map::Keys<String, Rc<RefCell<FileType>>> {
        self.files().keys()
    }

    /// Returns an iterator over all ObjectReaders in nested files
    /// Python equivalent: File.py lines 65-74
    fn get_objects(&self) -> Vec<ObjectReader<()>> {
        let mut result = Vec::new();

        for file_rc in self.files().values() {
            let file_ref = file_rc.borrow();
            match &*file_ref {
                FileType::SerializedFile(sf_rc) => {
                    // SerializedFile: iterate objects HashMap
                    let serialized_file = sf_rc.borrow();
                    for obj_reader in serialized_file.objects.values() {
                        result.push(obj_reader.clone());
                    }
                }
                FileType::BundleFile(bundle_file) => {
                    // BundleFile: recursively get objects
                    result.extend(bundle_file.get_objects());
                }
                FileType::WebFile(web_file) => {
                    // WebFile: recursively get objects
                    result.extend(web_file.get_objects());
                }
                FileType::Writer(_) => {
                    // Writers have no objects
                }
                FileType::Raw(_) => {
                    // Raw files have no objects
                }
            }
        }

        result
    }

    /// Returns ObjectReaders filtered by ClassIDType
    /// Python equivalent: File.py lines 52-63
    fn get_filtered_objects(&self, obj_types: &[ClassIDType]) -> Vec<ObjectReader<()>> {
        // If no types specified, return all objects
        if obj_types.is_empty() {
            return self.get_objects();
        }

        let mut result = Vec::new();

        for file_rc in self.files().values() {
            let file_ref = file_rc.borrow();
            match &*file_ref {
                FileType::SerializedFile(sf_rc) => {
                    // SerializedFile: filter objects by type
                    let serialized_file = sf_rc.borrow();
                    for obj_reader in serialized_file.objects.values() {
                        if obj_types.contains(&obj_reader.obj_type) {
                            result.push(obj_reader.clone());
                        }
                    }
                }
                FileType::BundleFile(bundle_file) => {
                    // BundleFile: recursively get filtered objects
                    result.extend(bundle_file.get_filtered_objects(obj_types));
                }
                FileType::WebFile(web_file) => {
                    // WebFile: recursively get filtered objects
                    result.extend(web_file.get_filtered_objects(obj_types));
                }
                FileType::Writer(_) => {
                    // Writers have no objects
                }
                FileType::Raw(_) => {
                    // Raw files have no objects
                }
            }
        }

        result
    }

    /// Aggregates all container paths from nested SerializedFiles
    /// Python equivalent: File.py lines 130-136
    fn container(&self) -> HashMap<String, i64> {
        let mut result = HashMap::new();

        for file_rc in self.files().values() {
            let file_ref = file_rc.borrow();
            match &*file_ref {
                FileType::SerializedFile(sf_rc) => {
                    // SerializedFile has a container - merge its items
                    let serialized_file = sf_rc.borrow();
                    for (path, path_id) in serialized_file.container.items() {
                        result.insert(path.clone(), *path_id);
                    }
                }
                FileType::BundleFile(bundle_file) => {
                    // BundleFile: recursively get container from nested files
                    result.extend(bundle_file.container());
                }
                FileType::WebFile(web_file) => {
                    // WebFile: recursively get container from nested files
                    result.extend(web_file.container());
                }
                FileType::Writer(_) => {
                    // Writers have no objects
                }
                FileType::Raw(_) => {
                    // Raw files have no container
                }
            }
        }

        result
    }

    /// Returns all SerializedFile instances from nested file structure
    /// Python equivalent: File.py lines 41-50
    fn get_assets(&self) -> Vec<Rc<RefCell<FileType>>> {
        let mut result = Vec::new();

        for file_rc in self.files().values() {
            match &*file_rc.borrow() {
                FileType::SerializedFile(_) => {
                    // Add the SerializedFile to results
                    result.push(Rc::clone(file_rc));
                }
                FileType::BundleFile(bundle_file) => {
                    // Recursively get assets from BundleFile
                    result.extend(bundle_file.get_assets());
                }
                FileType::WebFile(web_file) => {
                    // Recursively get assets from WebFile
                    result.extend(web_file.get_assets());
                }
                FileType::Writer(_) | FileType::Raw(_) => {
                    // No assets in these types
                }
            }
        }

        result
    }

    /// Returns an iterator over (name, file) pairs
    /// Python equivalent: self.files.items()
    fn items(&self) -> std::collections::hash_map::Iter<String, Rc<RefCell<FileType>>> {
        self.files().iter()
    }

    /// Returns an iterator over file values
    /// Python equivalent: self.files.values()
    fn values(&self) -> std::collections::hash_map::Values<String, Rc<RefCell<FileType>>> {
        self.files().values()
    }

    // Default implementation of read_files (Python File.py lines 76-94)
    fn read_files(
        &mut self,
        mut blocks_reader: MemoryReader,
        m_directory_info: Vec<BundleDirectoryEntry>,
    ) -> Result<(), io::Error> {
        for node in m_directory_info {
            blocks_reader.set_position(node.offset as usize);
            let file_data = blocks_reader.read(node.size as usize)?;
            let file_reader = MemoryReader::new(
                file_data,
                blocks_reader.endian(),
                blocks_reader.base_offset() + node.offset as usize,
            );

            let parsed_file = import_helper::parse_file(
                file_reader,
                node.path.clone(),
                None,
                self.is_dependency(),
                None,
            )?;

            let path = node.path.clone();
            self.files_mut()
                .insert(path.clone(), Rc::new(RefCell::new(parsed_file)));
            self.file_flags_mut().insert(path, node.flags);
        }

        Ok(())
    }

    // Python File.py lines 96-127: get_writeable_cab
    fn get_writeable_cab(
        &mut self,
        name: Option<&str>,
    ) -> Result<Option<Rc<RefCell<FileType>>>, io::Error> {
        // Determine the name to use
        let name_str = match name {
            Some(n) => n.to_string(),
            None => {
                let cab_name = self.cab_file();
                if cab_name.is_empty() {
                    return Ok(None);
                }
                cab_name.to_string()
            }
        };

        // Check if file already exists
        if let Some(existing_file) = self.files().get(name_str.as_str()) {
            match &*existing_file.borrow() {
                FileType::Writer(_) => {
                    return Ok(Some(Rc::clone(existing_file)));
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "This cab already exists and isn't an EndianBinaryWriter",
                    ));
                }
            }
        }

        // Create new writer with default endian
        let mut writer = EndianBinaryWriter::new(Endian::Little);

        // Try to find a .resS file to copy settings from
        let mut flags_to_copy = None;
        let mut endian_to_copy = None;

        for (fname, file_rc) in self.files().iter() {
            if fname.ends_with(".resS") {
                // Get flags from file_flags HashMap
                if let Some(&flags) = self.file_flags().get(fname) {
                    flags_to_copy = Some(flags);
                }

                // Get endian from the file
                match &*file_rc.borrow() {
                    FileType::Raw(reader) => {
                        endian_to_copy = Some(reader.endian());
                    }
                    _ => {}
                }

                break;
            }
        }

        // Apply copied values or defaults
        writer.flags = flags_to_copy.unwrap_or(0);

        if let Some(endian) = endian_to_copy {
            writer.set_endian(endian);
        }

        writer.name = name_str.clone();

        // Insert into files
        let writer_rc = Rc::new(RefCell::new(FileType::Writer(writer)));
        self.files_mut().insert(name_str, Rc::clone(&writer_rc));

        Ok(Some(writer_rc))
    }
}

#[derive(Debug, Clone)]
pub struct DirectoryInfo {
    pub path: String,
    pub offset: u64,
    pub size: u64,
}
