use super::bundle_file::FileType;
use super::file::File;
use super::object_reader::ObjectReader;
use crate::config;
use crate::enums::build_target::BuildTarget;
use crate::enums::class_id_type::ClassIDType;
use crate::environment::Environment;
use crate::files::file::ParentFile;
use crate::helpers::import_helper::FileSource;
use crate::helpers::type_tree_node::TypeTreeNode;
use crate::streams::endian::Endian;
use crate::streams::endian_reader::BinaryReader;
use crate::streams::endian_reader::MemoryReader;
use crate::streams::endian_writer::BinaryWriter;
use crate::streams::endian_writer::EndianBinaryWriter;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::rc::{Rc, Weak};

#[derive(Debug, Clone)]
pub struct SerializedFileHeader {
    pub metadata_size: u32,
    pub file_size: u64,
    pub version: u32,
    pub data_offset: u64,

    pub endian: Endian,
    pub reserved: Vec<u8>,
}

impl SerializedFileHeader {
    pub fn new<R: BinaryReader>(reader: &mut R) -> Result<Self, io::Error> {
        let values = reader.read_u32_array(Some(4))?;
        Ok(Self {
            metadata_size: values[0],
            file_size: values[1] as u64,
            version: values[2],
            data_offset: values[3] as u64,
            endian: Endian::Little,
            reserved: Vec::new(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct LocalSerializedObjectIdentifier {
    pub local_serialized_file_index: i32,
    pub local_identifier_in_file: i64,
}

impl LocalSerializedObjectIdentifier {
    pub fn new<R: BinaryReader>(
        header: &SerializedFileHeader,
        reader: &mut R,
    ) -> Result<Self, io::Error> {
        let local_serialized_file_index = reader.read_i32()?;
        let local_identifier_in_file = if header.version < 14 {
            reader.read_i32()? as i64
        } else {
            reader.align_stream(4);
            reader.read_i64()?
        };

        Ok(Self {
            local_serialized_file_index,
            local_identifier_in_file,
        })
    }

    pub fn write<W: BinaryWriter>(
        &self,
        header: &SerializedFileHeader,
        writer: &mut W,
    ) -> Result<(), io::Error> {
        writer.write_i32(self.local_serialized_file_index)?;

        if header.version < 14 {
            writer.write_i32(self.local_identifier_in_file as i32)?;
        } else {
            writer.align_stream(4)?;
            writer.write_i64(self.local_identifier_in_file)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileIdentifier {
    pub path: String,
    pub temp_empty: Option<String>,
    pub guid: Option<Vec<u8>>, // 16 bytes
    pub r#type: Option<i32>,   // Note: "type" is a keyword, use r#type
}

impl FileIdentifier {
    pub fn new<R: BinaryReader>(
        header: &SerializedFileHeader,
        reader: &mut R,
    ) -> Result<Self, io::Error> {
        let temp_empty = if header.version >= 6 {
            Some(reader.read_string_to_null(1024)?)
        } else {
            None
        };

        let (guid, r#type) = if header.version >= 5 {
            (Some(reader.read_bytes(16)?), Some(reader.read_i32()?))
        } else {
            (None, None)
        };

        let path = reader.read_string_to_null(1024)?;

        Ok(Self {
            path,
            temp_empty,
            guid,
            r#type,
        })
    }

    pub fn name(&self) -> String {
        std::path::Path::new(&self.path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string()
    }

    pub fn write<W: BinaryWriter>(
        &self,
        header: &SerializedFileHeader,
        writer: &mut W,
    ) -> Result<(), io::Error> {
        if header.version >= 6 {
            writer.write_string_to_null(
                self.temp_empty
                    .as_ref()
                    .expect("temp_empty required for version >= 6"),
            )?;
        }

        if header.version >= 5 {
            writer.write_bytes(self.guid.as_ref().expect("guid required for version >= 5"))?;
            writer.write_i32(self.r#type.expect("type required for version >= 5"))?;
        }

        writer.write_string_to_null(&self.path)?;
        Ok(())
    }
}

impl fmt::Display for FileIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<FileIdentifier({})>", self.path)
    }
}

#[derive(Debug, Clone)]
pub struct BuildType {
    pub build_type: String,
}

impl BuildType {
    pub fn new(build_type: String) -> Self {
        Self { build_type }
    }

    pub fn is_alpha(&self) -> bool {
        self.build_type == "a"
    }

    pub fn is_patch(&self) -> bool {
        self.build_type == "p"
    }
}

#[derive(Debug, Clone)]
pub struct SerializedType {
    pub class_id: i32,
    pub is_stripped_type: Option<bool>,
    pub script_type_index: i16,
    pub script_id: Option<Vec<u8>>,     // Hash128 (16 bytes)
    pub old_type_hash: Option<Vec<u8>>, // Hash128 (16 bytes)
    pub node: Option<TypeTreeNode>,
    // ref type fields
    pub m_class_name: Option<String>,
    pub m_name_space: Option<String>,
    pub m_assembly_name: Option<String>,
    // version 21+
    pub type_dependencies: Option<Vec<i32>>,
}

impl SerializedType {
    pub fn new<R: BinaryReader>(
        reader: &mut R,
        header: &SerializedFileHeader,
        enable_type_tree: bool,
        is_ref_type: bool,
    ) -> Result<Self, io::Error> {
        let version = header.version;

        let class_id = reader.read_i32()?;

        let is_stripped_type = if version >= 16 {
            Some(reader.read_bool()?)
        } else {
            None
        };

        let script_type_index = if version >= 17 {
            reader.read_i16()?
        } else {
            -1
        };

        let script_id = if version >= 13
            && ((is_ref_type && script_type_index >= 0)
                || (version < 16 && class_id < 0)
                || (version >= 16 && class_id == 114))
        {
            Some(reader.read_bytes(16)?)
        } else {
            None
        };

        let old_type_hash = if version >= 13 {
            Some(reader.read_bytes(16)?)
        } else {
            None
        };

        let node = if enable_type_tree {
            if version >= 12 || version == 10 {
                Some(TypeTreeNode::parse_blob(reader, version as i32)?)
            } else {
                Some(TypeTreeNode::parse(reader, version as i32)?)
            }
        } else {
            None
        };

        let m_class_name = if enable_type_tree && version >= 21 {
            if is_ref_type {
                Some(reader.read_string_to_null(1024)?)
            } else {
                None
            }
        } else {
            None
        };

        let m_name_space = if enable_type_tree && version >= 21 {
            if is_ref_type {
                Some(reader.read_string_to_null(1024)?)
            } else {
                None
            }
        } else {
            None
        };

        let m_assembly_name = if enable_type_tree && version >= 21 {
            if is_ref_type {
                Some(reader.read_string_to_null(1024)?)
            } else {
                None
            }
        } else {
            None
        };

        let type_dependencies = if enable_type_tree && version >= 21 {
            if !is_ref_type {
                Some(reader.read_i32_array(None)?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            class_id,
            is_stripped_type,
            script_type_index,
            script_id,
            old_type_hash,
            node,
            m_class_name,
            m_name_space,
            m_assembly_name,
            type_dependencies,
        })
    }

    pub fn write<W: BinaryWriter>(
        &self,
        header: &SerializedFileHeader,
        writer: &mut W,
        enable_type_tree: bool,
        is_ref_type: bool,
    ) -> Result<(), io::Error> {
        let version = header.version;
        writer.write_i32(self.class_id)?;

        if version >= 16 {
            writer.write_bool(
                self.is_stripped_type
                    .expect("is_stripped_type required for version >= 16"),
            )?;
        }

        if version >= 17 {
            writer.write_i16(self.script_type_index)?;
        }

        if version >= 13 {
            if (is_ref_type && self.script_type_index >= 0)
                || (version < 16 && self.class_id < 0)
                || (version >= 16 && self.class_id == 114)
            {
                writer.write_bytes(self.script_id.as_ref().expect("script_id required"))?;
            }
            writer.write_bytes(
                self.old_type_hash
                    .as_ref()
                    .expect("old_type_hash required for version >= 13"),
            )?;
        }

        if enable_type_tree {
            let node = self
                .node
                .as_ref()
                .expect("node required when enable_type_tree is true");
            if version >= 12 || version == 10 {
                node.dump_blob(writer, version as i32)?;
            } else {
                node.dump(writer, version as i32)?;
            }

            if version >= 21 {
                if is_ref_type {
                    writer.write_string_to_null(
                        self.m_class_name.as_ref().expect("m_class_name required"),
                    )?;
                    writer.write_string_to_null(
                        self.m_name_space.as_ref().expect("m_name_space required"),
                    )?;
                    writer.write_string_to_null(
                        self.m_assembly_name
                            .as_ref()
                            .expect("m_assembly_name required"),
                    )?;
                } else {
                    // Write i32 array with length prefix
                    let deps = self
                        .type_dependencies
                        .as_ref()
                        .expect("type_dependencies required");
                    writer.write_i32(deps.len() as i32)?;
                    for &dep in deps {
                        writer.write_i32(dep)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn nodes(&self) -> Option<&TypeTreeNode> {
        self.node.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub path_id: i64,
    pub byte_start: u64,
    pub byte_size: u32,
    pub type_id: i32,
    pub class_id: i32,
    pub data: Vec<u8>,
}

impl ObjectInfo {
    pub fn new<R: BinaryReader>(
        reader: &mut R,
        header: &SerializedFileHeader,
        big_id_enabled: bool,
    ) -> Result<Self, io::Error> {
        let version = header.version;

        let path_id = if big_id_enabled {
            reader.read_i64()?
        } else if version < 14 {
            reader.read_i32()? as i64
        } else {
            reader.align_stream(4);
            reader.read_i64()?
        };

        let byte_start = if version >= 22 {
            reader.read_i64()? as u64
        } else {
            reader.read_u32()? as u64
        };

        let byte_start = byte_start + header.data_offset;

        let byte_size = reader.read_u32()?;
        let type_id = reader.read_i32()?;

        let class_id = if version < 16 {
            reader.read_i16()? as i32
        } else {
            let type_index = reader.read_i32()?;
            // type_id is actually the index into types array
            type_index // In Python this looks up the class_id from types
        };

        let current_pos = reader.position();
        reader.set_position(byte_start as usize);
        let data = reader.read_bytes(byte_size as usize)?;
        reader.set_position(current_pos);

        Ok(Self {
            path_id,
            byte_start,
            byte_size,
            type_id,
            class_id,
            data,
        })
    }

    pub fn write<W: BinaryWriter>(
        &self,
        header: &SerializedFileHeader,
        big_id_enabled: bool,
        meta_writer: &mut W,
        data_writer: &mut W,
    ) -> Result<(), io::Error> {
        let version = header.version;

        // Write path_id (matches Python lines 66-72)
        if big_id_enabled {
            meta_writer.write_i64(self.path_id)?;
        } else if version < 14 {
            meta_writer.write_i32(self.path_id as i32)?;
        } else {
            meta_writer.write_align()?;
            meta_writer.write_i64(self.path_id)?;
        }

        // Write byte_start position in data_writer (matches Python lines 89-92)
        if version >= 22 {
            meta_writer.write_i64(data_writer.position() as i64)?;
        } else {
            meta_writer.write_u32(data_writer.position() as u32)?;
        }

        // Write byte_size (matches Python line 94)
        meta_writer.write_u32(self.data.len() as u32)?;

        // Write actual object data (matches Python line 95)
        data_writer.write(&self.data)?;

        // Write type_id (matches Python line 97)
        meta_writer.write_i32(self.type_id)?;

        // Write class_id for old versions (matches Python lines 99-100)
        if version < 16 {
            meta_writer.write_i16(self.class_id as i16)?;
        }

        // Note: Python also writes is_destroyed, script_type_index, and stripped
        // for specific versions, but ObjectInfo doesn't store these fields.
        // This is fine - those fields are only needed for versions < 17,
        // and most modern Unity files use version >= 17.

        Ok(())
    }
}

#[derive(Debug)]
pub struct SerializedFile {
    // Core metadata
    pub header: SerializedFileHeader,
    pub version: (u32, u32, u32, u32),
    pub unity_version: String,
    pub build_type: BuildType,
    pub target_platform: BuildTarget,

    // Type information
    pub enable_type_tree: bool,
    pub types: Vec<SerializedType>,
    pub ref_types: Option<Vec<SerializedType>>,

    // Shared reader for all ObjectReaders
    pub reader: Rc<RefCell<MemoryReader>>,

    // Object data (store metadata, create ObjectReaders on-demand)
    pub objects: HashMap<i64, ObjectReader<()>>,
    pub script_types: Vec<LocalSerializedObjectIdentifier>,

    // External references
    pub externals: Vec<FileIdentifier>,

    // Additional metadata
    pub big_id_enabled: i32,
    pub user_information: Option<String>,
    pub unknown: i64,

    // File trait fields
    pub name: String,
    pub is_changed: bool,

    // Internal state
    pub m_target_platform: i32, // Raw platform value

    // Parent reference (BundleFile or WebFile that contains this file)
    pub parent: Option<Weak<RefCell<dyn ParentFile>>>,
    pub environment: Option<Rc<RefCell<Environment>>>,
    // pub assetbundle: Option<AssetBundle>,
    pub is_dependency: bool,

    // Container system - maps Unity asset paths to PathIDs
    pub container: ContainerHelper, // path -> path_id mapping

    // Files HashMap - empty for SerializedFile (inherited from File base class in Python)
    pub files: HashMap<String, Rc<RefCell<FileType>>>,
    pub file_flags: HashMap<String, u32>,
}

impl File for SerializedFile {
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
        // Return default CAB file name
        "CAB-UnityPy_Mod.resS"
    }

    fn files(&self) -> &HashMap<String, Rc<RefCell<FileType>>> {
        &self.files
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

impl SerializedFile {
    pub fn new(
        reader: MemoryReader,
        name: Option<String>,
        is_dependency: bool,
    ) -> Result<Self, io::Error> {
        let reader_rc = Rc::new(RefCell::new(reader));

        let mut header = SerializedFileHeader::new(&mut *reader_rc.borrow_mut())?;
        let mut unknown = 0i64;

        if header.version >= 9 {
            header.endian = if reader_rc.borrow_mut().read_bool()? {
                Endian::Big
            } else {
                Endian::Little
            };

            header.reserved = reader_rc.borrow_mut().read_bytes(3)?;

            if header.version >= 22 {
                header.metadata_size = reader_rc.borrow_mut().read_u32()?;
                header.file_size = reader_rc.borrow_mut().read_u64()?;
                header.data_offset = reader_rc.borrow_mut().read_u64()?;

                unknown = reader_rc.borrow_mut().read_i64()?;
            }
        } else {
            reader_rc
                .borrow_mut()
                .set_position((header.file_size - header.metadata_size as u64) as usize);
            header.endian = if reader_rc.borrow_mut().read_bool()? {
                Endian::Big
            } else {
                Endian::Little
            };
        }

        reader_rc.borrow_mut().set_endian(header.endian);

        let mut unity_version = String::from("2.5.0f5");

        if header.version >= 7 {
            unity_version = reader_rc.borrow_mut().read_string_to_null(1024)?;
        }

        let _m_target_platform = if header.version >= 8 {
            reader_rc.borrow_mut().read_i32()?
        } else {
            -1
        };

        let target_platform = BuildTarget::from_i32(_m_target_platform);

        let mut _enable_type_tree = if header.version >= 13 {
            reader_rc.borrow_mut().read_bool()?
        } else {
            true
        };

        let type_count = reader_rc.borrow_mut().read_i32()?;
        let mut types = Vec::new();
        for _ in 0..type_count {
            types.push(SerializedType::new(
                &mut *reader_rc.borrow_mut(),
                &header,
                _enable_type_tree,
                false,
            )?);
        }

        let big_id_enabled = if header.version >= 7 && header.version < 14 {
            reader_rc.borrow_mut().read_i32()?
        } else {
            0
        };

        let object_count = reader_rc.borrow_mut().read_i32()?;
        let mut objects = HashMap::new();
        for _ in 0..object_count {
            let obj = ObjectReader::new(
                None,
                Rc::clone(&reader_rc),
                None, // ref_types not available yet, will be set later if needed
                big_id_enabled,
                (0, 0, 0, 0), // version - default per Python line 255
                header.version,
                target_platform,
                BuildType::new("".to_string()), // build_type - default per Python line 256
                &header,
                &types,
            )
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            objects.insert(obj.path_id, obj);
        }

        let script_types = if header.version >= 11 {
            let script_count = reader_rc.borrow_mut().read_i32()?;
            let mut scripts = Vec::new();
            for _ in 0..script_count {
                scripts.push(LocalSerializedObjectIdentifier::new(
                    &header,
                    &mut *reader_rc.borrow_mut(),
                )?);
            }
            scripts
        } else {
            Vec::new()
        };

        let externals_count = reader_rc.borrow_mut().read_i32()?;
        let mut externals = Vec::new();
        for _ in 0..externals_count {
            externals.push(FileIdentifier::new(&header, &mut *reader_rc.borrow_mut())?);
        }

        let ref_types = if header.version >= 20 {
            let ref_type_count = reader_rc.borrow_mut().read_i32()?;
            let mut ref_types_vec = Vec::new();
            for _ in 0..ref_type_count {
                ref_types_vec.push(SerializedType::new(
                    &mut *reader_rc.borrow_mut(),
                    &header,
                    _enable_type_tree,
                    true,
                )?);
            }

            Some(ref_types_vec)
        } else {
            None
        };

        if !config::should_parse_typetree() {
            _enable_type_tree = false;
        }

        let user_information = if header.version >= 5 {
            Some(reader_rc.borrow_mut().read_string_to_null(1024)?)
        } else {
            None
        };

        // Read the asset_bundles to get the containers (Python lines 340-348)
        let container = {
            let mut found_container = Vec::new();

            // Search for AssetBundle object
            for obj_reader in objects.values() {
                if obj_reader.obj_type == ClassIDType::AssetBundle {
                    // Parse AssetBundle using read() to get wrapped Object
                    // Use check_read=false during construction since reader positioning isn't fully set up yet
                    match obj_reader.clone().read(false) {
                        Ok(assetbundle_json) => {
                            // Extract m_Container array from the Object
                            // NOTE: Due to type tree parsing bug with numeric type IDs,
                            // m_Container comes as a nested object structure instead of an array
                            // This is a known limitation - see type_tree_helper.rs line 485
                            if let Some(m_container) = assetbundle_json.get("m_Container") {
                                if let Some(items) = m_container.as_array() {
                                    // Parse each container entry: [path, AssetInfo]
                                    for item in items {
                                        if let Some(entry) = item.as_array() {
                                            // entry[0] = path (string)
                                            // entry[1] = AssetInfo object with "asset" field
                                            if entry.len() >= 2 {
                                                if let (Some(path), Some(asset_info)) =
                                                    (entry[0].as_str(), entry[1].as_object())
                                                {
                                                    // Extract path_id from asset_info.asset.m_PathID
                                                    if let Some(asset) = asset_info.get("asset") {
                                                        if let Some(asset_obj) = asset.as_object() {
                                                            if let Some(path_id) =
                                                                asset_obj.get("m_PathID")
                                                            {
                                                                if let Some(path_id_num) =
                                                                    path_id.as_i64()
                                                                {
                                                                    found_container.push((
                                                                        path.to_string(),
                                                                        path_id_num,
                                                                    ));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse AssetBundle: {}", e);
                        }
                    }
                    break; // Found AssetBundle, stop searching
                }
            }

            // Create ContainerHelper from found data
            if found_container.is_empty() {
                ContainerHelper::new()
            } else {
                ContainerHelper::from_container(found_container)
            }
        };

        let mut serialized_file = Self {
            header,
            reader: reader_rc,
            unity_version: unity_version.clone(),
            big_id_enabled,
            build_type: BuildType::new("".to_string()),
            enable_type_tree: _enable_type_tree,
            externals,
            is_changed: false,
            version: (0, 0, 0, 0),
            m_target_platform: _m_target_platform,
            name: name.unwrap_or_else(|| String::from("")),
            objects,
            target_platform,
            types,
            ref_types,
            script_types,
            unknown,
            user_information,
            parent: None,
            is_dependency,
            container,
            files: HashMap::new(),
            file_flags: HashMap::new(),
            environment: None,
        };

        serialized_file.set_version(unity_version);

        Ok(serialized_file)
    }

    /// Sets parent references for all ObjectReaders
    ///
    /// This must be called after the SerializedFile is wrapped in Rc<RefCell<>>
    ///
    /// # Arguments
    /// * `self_rc` - Rc reference to this SerializedFile
    pub fn set_object_parents(self_rc: &Rc<RefCell<Self>>) -> Result<(), io::Error> {
        let parent_weak: Weak<RefCell<SerializedFile>> = Rc::downgrade(self_rc);

        let mut self_mut = self_rc.borrow_mut();

        // Clone ref_types before the loop to avoid borrowing issues
        let ref_types_clone = self_mut.ref_types.clone();

        for obj_reader in self_mut.objects.values_mut() {
            obj_reader.assets_file = Some(parent_weak.clone());

            // Set ref_types if available (Python line 311 context)
            if let Some(ref ref_types) = ref_types_clone {
                obj_reader.ref_types = Some(ref_types.clone());
            }
        }

        Ok(())
    }

    pub fn save(&mut self) -> Result<Vec<u8>, io::Error> {
        let header = &self.header;

        // Create writers with correct endian
        let mut meta_writer = EndianBinaryWriter::new(header.endian);
        let mut data_writer = EndianBinaryWriter::new(header.endian);

        // Write metadata section
        if header.version >= 7 {
            meta_writer.write_string_to_null(&self.unity_version)?;
        }

        if header.version >= 8 {
            meta_writer.write_i32(self.m_target_platform)?;
        }

        if header.version >= 13 {
            meta_writer.write_bool(self.enable_type_tree)?;
        }

        // Write types
        meta_writer.write_i32(self.types.len() as i32)?;
        for typ in &self.types {
            typ.write(header, &mut meta_writer, self.enable_type_tree, false)?;
        }

        if header.version >= 7 && header.version < 14 {
            meta_writer.write_i32(self.big_id_enabled)?;
        }

        // Write objects
        meta_writer.write_i32(self.objects.len() as i32)?;
        for obj in self.objects.values_mut() {
            obj.write(&header, &mut meta_writer, &mut data_writer)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            data_writer.align_stream(8)?;
        }

        // Write script_types
        if header.version >= 11 {
            meta_writer.write_i32(self.script_types.len() as i32)?;
            for script_type in &self.script_types {
                script_type.write(header, &mut meta_writer)?;
            }
        }

        // Write externals
        meta_writer.write_i32(self.externals.len() as i32)?;
        for external in &self.externals {
            external.write(header, &mut meta_writer)?;
        }

        // Write ref_types
        if header.version >= 20 {
            if let Some(ref_types) = &self.ref_types {
                meta_writer.write_i32(ref_types.len() as i32)?;
                for ref_type in ref_types {
                    ref_type.write(header, &mut meta_writer, self.enable_type_tree, true)?;
                }
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "ref_types required for version >= 20",
                ));
            }
        }

        // Write user_information
        if header.version >= 5 {
            if let Some(user_info) = &self.user_information {
                meta_writer.write_string_to_null(user_info)?;
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "user_information required for version >= 5",
                ));
            }
        }

        // Calculate sizes
        let mut writer = EndianBinaryWriter::new(Endian::Little);
        let header_size: usize = 16; // 4*4
        let metadata_size = meta_writer.len();
        let data_size = data_writer.len();

        if header.version >= 9 {
            // Calculate header size
            let extra_header_size = if header.version < 22 { 4 } else { 4 + 28 };
            let total_header_size = header_size + extra_header_size;

            // Calculate data offset with 16-byte alignment
            let mut data_offset = total_header_size + metadata_size;
            data_offset += (16 - data_offset % 16) % 16;

            let file_size = data_offset + data_size;

            if header.version < 22 {
                writer.write_u32(metadata_size as u32)?;
                writer.write_u32(file_size as u32)?;
                writer.write_u32(header.version)?;
                writer.write_u32(data_offset as u32)?;
                writer.write_bool(header.endian == Endian::Big)?;
                writer.write_bytes(&header.reserved)?;
            } else {
                // Version >= 22: old header values are 0
                writer.write_u32(0)?;
                writer.write_u32(0)?;
                writer.write_u32(header.version)?;
                writer.write_u32(0)?;
                writer.write_bool(header.endian == Endian::Big)?;
                writer.write_bytes(&header.reserved)?;
                writer.write_u32(metadata_size as u32)?;
                writer.write_u64(file_size as u64)?;
                writer.write_u64(data_offset as u64)?;
                writer.write_i64(self.unknown)?;
            }

            // Write metadata
            writer.write_bytes(&meta_writer.to_bytes())?;

            // Align to 16 bytes
            let current_pos = writer.position();
            let padding = (16 - (current_pos % 16)) % 16;
            for _ in 0..padding {
                writer.write_u8(0)?;
            }

            // Write data
            writer.write_bytes(&data_writer.to_bytes())?;
        } else {
            // Version < 9: simpler structure
            let metadata_size_with_endian = metadata_size + 1; // +1 for endian boolean
            let file_size = header_size + metadata_size_with_endian + data_size;

            writer.write_u32(metadata_size_with_endian as u32)?;
            writer.write_u32(file_size as u32)?;
            writer.write_u32(header.version)?;
            writer.write_u32(32)?; // data_offset

            // Write data first
            writer.write_bytes(&data_writer.to_bytes())?;

            // Then endian and metadata
            writer.write_bool(header.endian == Endian::Big)?;
            writer.write_bytes(&meta_writer.to_bytes())?;
        }

        Ok(writer.to_bytes())
    }

    /// Parses unity version string into version tuple and build_type
    ///
    /// # Arguments
    ///
    /// * `string_version` - Unity version string (e.g. "2019.4.3f1")
    ///
    /// Python equivalent: lines 371-383
    pub fn set_version(&mut self, mut string_version: String) {
        self.unity_version = string_version.clone();

        // Handle edge cases (empty or "0.0.0")
        // Python equivalent: lines 373-379
        if string_version.is_empty() || string_version == "0.0.0" {
            // Try to get version from parent (Python lines 376-377)
            if let Some(parent_weak) = &self.parent {
                if let Some(parent_rc) = parent_weak.upgrade() {
                    let parent_ref = parent_rc.borrow();
                    if let Some(parent_version) = parent_ref.get_version_engine() {
                        if !parent_version.is_empty() && parent_version != "0.0.0" {
                            string_version = parent_version;
                        }
                    }
                }
            }

            // Fall back to config (Python line 379)
            if string_version.is_empty() || string_version == "0.0.0" {
                string_version = config::get_fallback_version()
                    .map_err(|e| format!("Failed to get fallback version: {}", e))
                    .unwrap_or_else(|_| String::from("2017.4.0f1"));
            }
        }

        // Extract build_type: find first non-digit, non-period character
        // Python: re.findall(r"([^\d.])", string_version)
        let build_type_str = string_version
            .chars()
            .find(|c| !c.is_ascii_digit() && *c != '.')
            .map(|c| c.to_string())
            .unwrap_or_else(|| String::from(""));

        self.build_type = BuildType::new(build_type_str);

        // Split version into components: split on non-digit characters
        // Python: re.split(r"\D", string_version)
        let version_parts: Vec<u32> = string_version
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse::<u32>().ok())
            .take(4)
            .collect();

        // Build version tuple, padding with 0s if needed
        self.version = (
            version_parts.get(0).copied().unwrap_or(0),
            version_parts.get(1).copied().unwrap_or(0),
            version_parts.get(2).copied().unwrap_or(0),
            version_parts.get(3).copied().unwrap_or(0),
        );
    }

    pub fn load_dependencies(&mut self, possible_dependencies: Vec<String>) -> Result<(), String> {
        // Clone the Rc to avoid borrow conflicts
        let environment = self.environment.as_ref()
            .ok_or("No environment available for loading depedencies")?
            .clone();

        // Collect all files to load
        let mut files_to_load = Vec::new();

        // PART 1: Collect externals
        for file_id in &self.externals {
            files_to_load.push(file_id.path.clone());
        }

        // PART 2: Collect possible_dependencies
        files_to_load.extend(possible_dependencies);

        // Try to load all files
        // Use try_borrow_mut to avoid panic if environment is already borrowed
        match environment.try_borrow_mut() {
            Ok(mut env) => {
                for path in files_to_load {
                    let file_source = FileSource::Path(path);
                    env.load_file(file_source, None, true);
                }
            }
            Err(_) => {
                // Environment is already borrowed, skip loading dependencies
                // This can happen when processing objects that reference external resources
                log::debug!("Skipping dependency loading - environment already borrowed");
            }
        }

        Ok(())
    }

    pub fn get_writeable_cab(
        &mut self,
        name: Option<&str>,
    ) -> Result<Option<Rc<RefCell<FileType>>>, io::Error> {
        let name = name.unwrap_or("CAB-UnityPy_Mod.resS");
        let parent = match &self.parent {
            Some(weak_parent) => match weak_parent.upgrade() {
                Some(parent_rc) => parent_rc,
                None => return Ok(None), // Parent was dropped
            },
            None => return Ok(None), // No parent
        };
        let cab = {
            let mut parent_mut = parent.borrow_mut();
            parent_mut.get_writeable_cab(Some(name))?
        };
        let cab = match cab {
            Some(c) => c,
            None => return Ok(None),
        };

        let cab_path = format!("archive:/{}/{}", self.name, name);

        {
            let mut cab_mut = cab.borrow_mut();
            if let FileType::Writer(ref mut writer) = &mut *cab_mut {
                writer.name = cab_path.clone();
            }
        }

        let already_exists = self
            .externals
            .iter()
            .any(|external| external.path == cab_path);

        if !already_exists {
            use std::time::{SystemTime, UNIX_EPOCH};

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            let guid_str = format!("{:016x}", timestamp as u64);
            let guid_bytes = guid_str.as_bytes().to_vec();

            let file_identifier = FileIdentifier {
                path: cab_path,
                temp_empty: Some(String::new()),
                guid: Some(guid_bytes),
                r#type: Some(0),
            };

            self.externals.push(file_identifier);
        }

        Ok(Some(cab))
    }
}

/// Helper struct for Unity container access
///
/// Provides dictionary-like access to assets by path.
/// Maps Unity asset paths to ObjectReaders.
///
/// Python equivalent: ContainerHelper (lines 532-580)
#[derive(Debug, Clone)]
pub struct ContainerHelper {
    /// Maps asset path to path_id
    /// Python equivalent: container_dict (but stores path_id instead of PPtr)
    path_to_id: HashMap<String, i64>,

    /// Maps path_id back to asset path
    /// Python equivalent: path_dict
    id_to_path: HashMap<i64, String>,
}

impl ContainerHelper {
    /// Creates an empty ContainerHelper
    ///
    /// Python equivalent: ContainerHelper.__init__(assetbundle=None) - line 541-542
    pub fn new() -> Self {
        Self {
            path_to_id: HashMap::new(),
            id_to_path: HashMap::new(),
        }
    }

    /// Creates ContainerHelper from container data
    ///
    /// Python equivalent: ContainerHelper.__init__(assetbundle) - line 544-547
    pub fn from_container(container: Vec<(String, i64)>) -> Self {
        let mut path_to_id = HashMap::new();
        let mut id_to_path = HashMap::new();

        for (path, path_id) in container {
            path_to_id.insert(path.clone(), path_id);
            id_to_path.insert(path_id, path);
        }

        Self {
            path_to_id,
            id_to_path,
        }
    }

    /// Gets path_id by asset path
    ///
    /// Python equivalent: container_dict[key] - line 559
    pub fn get(&self, path: &str) -> Option<i64> {
        self.path_to_id.get(path).copied()
    }

    /// Gets asset path by path_id
    ///
    /// Python equivalent: path_dict[path_id]
    pub fn get_path(&self, path_id: i64) -> Option<&String> {
        self.id_to_path.get(&path_id)
    }

    /// Returns all asset paths
    ///
    /// Python equivalent: keys() - line 552-553
    pub fn keys(&self) -> Vec<&String> {
        self.path_to_id.keys().collect()
    }

    /// Returns all path_ids
    ///
    /// Python equivalent: values() - line 555-556 (returns PPtrs)
    pub fn values(&self) -> Vec<i64> {
        self.path_to_id.values().copied().collect()
    }

    /// Returns iterator over (path, path_id) pairs
    ///
    /// Python equivalent: items() - line 549-550
    pub fn items(&self) -> impl Iterator<Item = (&String, &i64)> {
        self.path_to_id.iter()
    }

    /// Returns number of containers
    ///
    /// Python equivalent: __len__() - line 570-571
    pub fn len(&self) -> usize {
        self.path_to_id.len()
    }

    /// Returns true if container is empty
    pub fn is_empty(&self) -> bool {
        self.path_to_id.is_empty()
    }
}

impl Default for ContainerHelper {
    fn default() -> Self {
        Self::new()
    }
}
