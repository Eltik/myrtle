use super::serialized_file::{BuildType, SerializedFile, SerializedFileHeader, SerializedType};
use crate::classes::class_id_type_to_class_map::{get_class_for_type, has_mapping};
use crate::classes::object::ObjectReaderTrait;
use crate::enums::build_target::BuildTarget;
use crate::enums::class_id_type::ClassIDType;
use crate::errors::{UnityError, UnityResult};
use crate::helpers::tpk::get_typetree_node;
use crate::helpers::type_tree_helper::{read_typetree, write_typetree, UnityValue};
use crate::helpers::type_tree_node::TypeTreeNode;
use crate::streams::endian_reader::{BinaryReader, MemoryReader};
use crate::streams::endian_writer::{BinaryWriter, EndianBinaryWriter};
use serde_json;
use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

/// ObjectReader - reads and deserializes Unity objects from binary data
///
/// Generic over:
/// - 'a: Lifetime tying this reader to its parent SerializedFile
/// - T: The target Unity object type (Texture2D, Mesh, etc.)
#[derive(Debug)]
pub struct ObjectReader<T> {
    pub reader: Rc<RefCell<MemoryReader>>,

    // Reference to parent SerializedFile
    pub assets_file: Option<Weak<RefCell<SerializedFile>>>,

    pub ref_types: Option<Vec<SerializedType>>,
    pub big_id_enabled: i32,

    // Owned data
    pub data: RefCell<Vec<u8>>,
    pub version: (u32, u32, u32, u32),
    pub version2: u32,
    pub platform: BuildTarget,
    pub build_type: BuildType,
    pub path_id: i64,
    pub byte_start_offset: (usize, usize),
    pub byte_start: u64,
    pub byte_size_offset: (usize, usize),
    pub byte_size: u32,
    pub type_id: i32,
    pub serialized_type: Option<SerializedType>,
    pub class_id: u16,
    pub obj_type: ClassIDType,
    pub is_destroyed: Option<u16>,
    pub is_stripped: Option<u8>,

    // Field added dynamically in Python (line 204)
    read_until: Option<usize>,

    // Additional metadata fields (Python lines 86-87)
    pub byte_header_offset: u64,
    pub byte_base_offset: u64,

    // PhantomData for generic type T
    _phantom: PhantomData<T>,
}

impl<T> ObjectReader<T> {
    pub fn new(
        assets_file: Option<Weak<RefCell<SerializedFile>>>,
        reader: Rc<RefCell<MemoryReader>>,
        ref_types: Option<Vec<SerializedType>>,
        big_id_enabled: i32,
        version: (u32, u32, u32, u32),
        version2: u32,
        platform: BuildTarget,
        build_type: BuildType,
        header: &SerializedFileHeader,
        types: &Vec<SerializedType>,
    ) -> Result<Self, String> {
        let mut reader_ref = reader.borrow_mut();

        // 2. Read path_id (version-dependent format)
        let path_id = if big_id_enabled != 0 {
            reader_ref.read_i64().map_err(|e| e.to_string())? as i64
        } else if header.version < 14 {
            reader_ref.read_i32().map_err(|e| e.to_string())? as i64
        } else {
            reader_ref.align_stream(4);
            reader_ref.read_i64().map_err(|e| e.to_string())? as i64
        };

        // 3. Read byte_start (version-dependent size)
        let (byte_start_offset, byte_start) = if header.version >= 22 {
            let offset = reader_ref.real_offset();
            let value = reader_ref.read_i64().map_err(|e| e.to_string())? as u64;
            ((offset, 8), value)
        } else {
            let offset = reader_ref.real_offset();
            let value = reader_ref.read_u32().map_err(|e| e.to_string())? as u64;
            ((offset, 4), value)
        };

        // 4. Adjust byte_start and store metadata
        let byte_start = byte_start + header.data_offset;
        let byte_header_offset = header.data_offset;
        let byte_base_offset = reader_ref.base_offset();

        // 5. Read byte_size
        let byte_size_offset = (reader_ref.real_offset(), 4);
        let byte_size = reader_ref.read_u32().map_err(|e| e.to_string())?;

        // 6. Read type_id
        let type_id = reader_ref.read_i32().map_err(|e| e.to_string())?;

        // 7. Read class_id and find serialized_type (version-dependent)
        let (class_id, mut serialized_type) = if header.version < 16 {
            let class_id = reader_ref.read_u16().map_err(|e| e.to_string())?;
            let mut found_type = None;
            for typ in types {
                if typ.class_id == type_id {
                    found_type = Some(typ.clone());
                    break;
                }
            }
            (class_id, found_type)
        } else {
            let typ = types
                .get(type_id as usize)
                .ok_or(format!("Invalid type_id: {}", type_id))?;
            (typ.class_id as u16, Some(typ.clone()))
        };

        // 8. Convert to ClassIDType enum
        let obj_type = ClassIDType::from(class_id as i32);

        // 9. Read optional is_destroyed
        let is_destroyed = if header.version < 11 {
            Some(reader_ref.read_u16().map_err(|e| e.to_string())?)
        } else {
            None
        };

        // 10. Update serialized_type with script_type_index
        if (11..17).contains(&header.version) {
            let script_type_index = reader_ref.read_i16().map_err(|e| e.to_string())?;
            if let Some(ref mut typ) = serialized_type {
                typ.script_type_index = script_type_index;
            }
        }

        // 11. Read optional is_stripped
        let is_stripped = if header.version == 15 || header.version == 16 {
            Some(reader_ref.read_u8().map_err(|e| e.to_string())?)
        } else {
            None
        };

        drop(reader_ref);

        // 12. Construct and return
        Ok(ObjectReader {
            assets_file,
            reader,
            ref_types,
            big_id_enabled,
            data: RefCell::new(Vec::new()),
            version,
            version2,
            platform,
            build_type,
            path_id,
            byte_start_offset,
            byte_start,
            byte_size_offset,
            byte_size,
            type_id,
            serialized_type,
            class_id,
            obj_type,
            is_destroyed,
            is_stripped,
            read_until: None,
            byte_header_offset,
            byte_base_offset: byte_base_offset as u64,
            _phantom: PhantomData,
        })
    }

    // === Position Management ===

    /// Resets the reader position to the start of this object's data
    pub fn reset(&mut self) {
        self.reader
            .borrow_mut()
            .set_position(self.byte_start as usize);
    }

    /// Gets the current position in the reader
    pub fn position(&self) -> usize {
        self.reader.borrow().position()
    }

    /// Sets the reader position
    pub fn set_position(&mut self, pos: usize) {
        self.reader.borrow_mut().set_position(pos);
    }

    // === Data Management ===

    /// Sets the raw binary data for this object
    ///
    /// Note: In Python this calls assets_file.mark_changed(), but we can't
    /// do that here because assets_file is an immutable reference.
    /// The caller will need to mark the file as changed separately.
    pub fn set_raw_data(&mut self, data: Vec<u8>) {
        *self.data.borrow_mut() = data;
    }

    /// Gets the raw binary data for this object
    ///
    /// Saves and restores the current position.
    pub fn get_raw_data(&mut self) -> Result<Vec<u8>, String> {
        let pos = self.position();
        self.reset();
        let ret = self
            .reader
            .borrow_mut()
            .read_bytes(self.byte_size as usize)
            .map_err(|e| e.to_string())?;
        self.set_position(pos);
        Ok(ret)
    }

    /// Gets a field value by name, with optional default
    ///
    /// This provides dictionary-like access to ObjectReader fields.
    /// Returns the field value as JSON, or the default value if the field doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `key` - The field name to retrieve
    /// * `default` - Optional default value if field not found
    ///
    /// # Returns
    ///
    /// The field value as `serde_json::Value`, or default (or null) if not found
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use serde_json::json;
    ///
    /// let path_id = obj.get("path_id", None);
    /// let unknown = obj.get("unknown_field", Some(json!(42)));
    /// ```
    ///
    /// Python equivalent: lines 207-208
    pub fn get(&self, key: &str, default: Option<serde_json::Value>) -> serde_json::Value {
        use serde_json::json;

        match key {
            // Primary fields (commonly accessed)
            "path_id" => json!(self.path_id),
            "byte_start" => json!(self.byte_start),
            "byte_size" => json!(self.byte_size),
            "type_id" => json!(self.type_id),
            "class_id" => json!(self.class_id),
            "type" => json!(format!("{:?}", self.obj_type)), // Python calls this "type"

            // Version fields
            "version" => json!([
                self.version.0,
                self.version.1,
                self.version.2,
                self.version.3
            ]),
            "version2" => json!(self.version2),

            // Offset tracking
            "byte_start_offset" => json!([self.byte_start_offset.0, self.byte_start_offset.1]),
            "byte_size_offset" => json!([self.byte_size_offset.0, self.byte_size_offset.1]),
            "byte_header_offset" => json!(self.byte_header_offset),
            "byte_base_offset" => json!(self.byte_base_offset),

            // Platform/build info
            "platform" => json!(format!("{:?}", self.platform)),
            "build_type" => json!(self.build_type.build_type),

            // Optional fields
            "is_destroyed" => self.is_destroyed.map(|v| json!(v)).unwrap_or(json!(null)),
            "is_stripped" => self.is_stripped.map(|v| json!(v)).unwrap_or(json!(null)),

            // Data field (return length, not actual bytes for practicality)
            "data" => json!(self.data.borrow().len()),

            // Not found - return default or null
            _ => default.unwrap_or(json!(null)),
        }
    }

    // === TypeTree Operations ===

    /// Dumps the type tree structure for debugging
    ///
    /// Returns a formatted string showing the type hierarchy.
    pub fn dump_typetree_structure(
        &mut self,
        nodes: Option<TypeTreeNode>,
        indent: &str,
    ) -> Result<String, String>
    where
        T: fmt::Debug + 'static,
    {
        let node = self.get_typetree_node(nodes).map_err(|e| e.to_string())?;
        Ok(node.dump_structure(indent))
    }

    /// Reads and deserializes this object using its type tree
    ///
    ///
    /// # Arguments
    ///
    /// * `nodes` - Optional type tree node (uses default if None)
    /// * `wrap` - If true, returns Object; if false, returns dict
    /// * `check_read` - If true, validates all data was read
    pub fn read_typetree(
        &mut self,
        nodes: Option<TypeTreeNode>,
        wrap: bool,
        check_read: bool,
    ) -> Result<serde_json::Value, String>
    where
        T: fmt::Debug + 'static,
    {
        let node = self.get_typetree_node(nodes).map_err(|e| e.to_string())?;
        self.reset();

        let ref_types = &self.ref_types;
        let mut ret = read_typetree(
            &node,
            &mut *self.reader.borrow_mut(),
            !wrap,
            Some(self.byte_size as usize),
            check_read,
            ref_types,
        )?;

        // If wrap=true and we got an Object, set the object_reader back-reference
        if wrap {
            if let UnityValue::Object(ref mut obj) = ret {
                obj.set_object_reader(Box::new(self.clone()));
            }
        }

        Ok(ret.to_json())
    }

    /// Reads this object as a Unity Object (wrapped)
    ///
    pub fn read(&mut self, check_read: bool) -> Result<serde_json::Value, String>
    where
        T: fmt::Debug + 'static,
    {
        let result = self.read_typetree(None, true, check_read);
        if result.is_ok() {
            self.read_until = Some(self.position());
        }
        result
    }

    /// Parses this object as a Unity Object (UnityPy 2 syntax)
    pub fn parse_as_object(
        &mut self,
        node: Option<TypeTreeNode>,
        check_read: bool,
    ) -> Result<serde_json::Value, String>
    where
        T: fmt::Debug + 'static,
    {
        self.read_typetree(node, true, check_read)
    }

    /// Parses this object as a dictionary (UnityPy 2 syntax)
    pub fn parse_as_dict(
        &mut self,
        node: Option<TypeTreeNode>,
        check_read: bool,
    ) -> Result<serde_json::Value, String>
    where
        T: fmt::Debug + 'static,
    {
        self.read_typetree(node, false, check_read)
    }

    /// Saves an object back to binary using its type tree
    ///
    /// # Arguments
    ///
    /// * `tree` - The Object to serialize
    /// * `nodes` - Optional type tree node (uses default if None)
    /// * `writer` - Optional writer (creates new if None)
    pub fn save_typetree(
        &mut self,
        tree: &serde_json::Value,
        nodes: Option<TypeTreeNode>,
    ) -> Result<Vec<u8>, String>
    where
        T: fmt::Debug + 'static,
    {
        let node = self.get_typetree_node(nodes).map_err(|e| e.to_string())?;
        let ref_types = &self.ref_types;

        let mut w = EndianBinaryWriter::new(self.reader.borrow().endian());
        write_typetree(&UnityValue::Json(tree.clone()), &node, &mut w, ref_types)?;
        let data = w.to_bytes();
        self.set_raw_data(data.clone());
        Ok(data)
    }

    /// Peeks the name of the object without reading/parsing the whole object
    pub fn peek_name(&mut self) -> Result<Option<String>, String>
    where
        T: fmt::Debug + 'static,
    {
        let node = self.get_typetree_node(None).map_err(|e| e.to_string())?;

        if let Some((peek_node, key)) = node.get_name_peek_node() {
            let dict = self.parse_as_dict(Some(peek_node), false)?;
            let name_value = dict
                .get(&key)
                .ok_or_else(|| format!("Key '{}' not found in parsed dict", key))?;

            // Convert JSON value to string
            let name = name_value
                .as_str()
                .ok_or("Name value is not a string")?
                .to_string();

            return Ok(Some(name));
        }

        Ok(None)
    }

    // === Helper Methods ===

    /// Gets the type tree node for this object
    ///
    /// Tries multiple sources in order:
    /// 1. Provided node parameter
    /// 2. serialized_type.node from parent file
    /// 3. TPK database lookup by class_id and version
    ///
    /// Special handling for MonoBehaviour (requires custom script parsing).
    fn get_typetree_node(&mut self, node: Option<TypeTreeNode>) -> UnityResult<TypeTreeNode>
    where
        T: fmt::Debug + 'static,
    {
        // If node provided directly, use it
        if let Some(n) = node {
            return Ok(n);
        }

        // Try to get from serialized_type
        if let Some(ref st) = self.serialized_type {
            if let Some(ref n) = st.node {
                return Ok(n.clone());
            }
        }

        let version = (
            self.version.0 as u16,
            self.version.1 as u16,
            self.version.2 as u16,
            self.version.3 as u16,
        );
        match get_typetree_node(self.class_id as i32, version) {
            Ok(n) => {
                // Special case: MonoBehaviour needs custom script parsing
                if n.m_type == "MonoBehaviour" {
                    // Try to get custom MonoBehaviour node
                    // This requires parsing the MonoBehaviour to get the script reference,
                    // then looking up the custom type tree from the generator
                    match self.try_monobehaviour_node(&n) {
                        Ok(custom_node) => return Ok(custom_node),
                        Err(_) => {
                            // Fall back to base MonoBehaviour node if custom lookup fails
                            return Ok(n);
                        }
                    }
                }
                Ok(n)
            }
            Err(_) => Err(UnityError::type_tree_error(
                "There are no TypeTree nodes for this object.",
                None,
            )),
        }
    }

    /// Attempts to get a custom type tree node for MonoBehaviour scripts
    ///
    /// Python equivalent: ObjectReader.py lines 307-322
    fn try_monobehaviour_node(&mut self, base_node: &TypeTreeNode) -> Result<TypeTreeNode, String>
    where
        T: fmt::Debug + 'static,
    {
        // Python equivalent: ObjectReader.py lines 307-322

        // Step 1: Check if environment has a typetree generator (Python lines 308-311)
        let assets_file_rc = self
            .assets_file
            .as_ref()
            .ok_or("ObjectReader has no assets_file reference")?
            .upgrade()
            .ok_or("SerializedFile reference no longer valid")?;

        let assets_file_ref = assets_file_rc.borrow();
        let env_rc = assets_file_ref
            .environment
            .as_ref()
            .ok_or("SerializedFile has no environment")?;

        let env_ref = env_rc.borrow();
        let generator_refcell = env_ref
            .typetree_generator
            .as_ref()
            .ok_or("No typetree generator set!")?;

        // Step 2: Parse this ObjectReader as MonoBehaviour (without read check)
        // Python line 313: monobehaviour = cast(MonoBehaviour, self.parse_as_object(base_node, check_read=False))
        let mb_json = self.parse_as_object(Some(base_node.clone()), false)?;

        // Step 3: Extract m_Script and dereference it
        // Python line 315: script = monobehaviour.m_Script.deref_parse_as_object()
        let script_data = mb_json
            .get("m_Script")
            .ok_or("MonoBehaviour missing m_Script field")?;

        let m_file_id = script_data
            .get("m_FileID")
            .and_then(|v| v.as_i64())
            .ok_or("m_Script missing m_FileID")? as i32;

        let m_path_id = script_data
            .get("m_PathID")
            .and_then(|v| v.as_i64())
            .ok_or("m_Script missing m_PathID")?;

        // Convert to PPtr and dereference
        use crate::classes::pptr::PPtr;
        let script_pptr = PPtr::<()>::with_assetsfile(m_file_id, m_path_id, &assets_file_ref);
        let script_json = script_pptr.deref_parse_as_object(None)?;

        // Step 4: Extract MonoScript metadata
        // Python lines 316-317: generator.get_nodes_up(script.m_AssemblyName, f"{script.m_Namespace}.{script.m_ClassName}")
        let assembly_name = script_json
            .get("m_AssemblyName")
            .and_then(|v| v.as_str())
            .ok_or("MonoScript missing m_AssemblyName")?;

        let namespace = script_json
            .get("m_Namespace")
            .and_then(|v| v.as_str())
            .unwrap_or(""); // Empty namespace is allowed

        let class_name = script_json
            .get("m_ClassName")
            .and_then(|v| v.as_str())
            .ok_or("MonoScript missing m_ClassName")?;

        // Step 5: Build fullname and get custom type tree
        // Python line 317: f"{script.m_Namespace}.{script.m_ClassName}"
        let fullname = if namespace.is_empty() {
            class_name.to_string()
        } else {
            format!("{}.{}", namespace, class_name)
        };

        // Step 6: Get type tree from generator (with interior mutability via RefCell)
        // Python lines 316-322
        let node = generator_refcell
            .borrow_mut()
            .get_nodes_up(assembly_name, &fullname)
            .map_err(|e| format!("Failed to get custom MonoBehaviour node: {}", e))?;

        Ok(node)
    }

    /// Writes this object back to binary format
    ///
    /// This is the serialization counterpart to new().
    /// Writes object metadata in version-specific format.
    ///
    /// # Arguments
    ///
    /// * `header` - The file header (contains version info)
    /// * `writer` - Writer for metadata (object table)
    /// * `data_writer` - Writer for actual object data
    pub fn write(
        &mut self,
        header: &SerializedFileHeader,
        writer: &mut EndianBinaryWriter,
        data_writer: &mut EndianBinaryWriter,
    ) -> Result<(), String> {
        // 1. Write path_id (version-dependent format)
        if self.big_id_enabled != 0 {
            writer.write_i64(self.path_id).map_err(|e| e.to_string())?;
        } else if header.version < 14 {
            writer
                .write_i32(self.path_id as i32)
                .map_err(|e| e.to_string())?;
        } else {
            writer.align_stream(4).map_err(|e| e.to_string())?;
            writer.write_i64(self.path_id).map_err(|e| e.to_string())?;
        }

        // 2. Get data to write
        let data = if !self.data.borrow().is_empty() {
            // Use modified data
            self.data.borrow().clone()
        } else {
            // Read original data from file
            self.get_raw_data()?
        };

        // 3. Write byte_start offset (points to where data will be written)
        if header.version >= 22 {
            writer
                .write_i64(data_writer.position() as i64)
                .map_err(|e| e.to_string())?;
        } else {
            writer
                .write_u32(data_writer.position() as u32)
                .map_err(|e| e.to_string())?;
        }

        // 4. Write data size
        writer
            .write_u32(data.len() as u32)
            .map_err(|e| e.to_string())?;

        // 5. Write actual data
        data_writer.write(&data).map_err(|e| e.to_string())?;

        // 6. Write type_id
        writer.write_i32(self.type_id).map_err(|e| e.to_string())?;

        // 7. Write class_id (old format only)
        if header.version < 16 {
            writer.write_u16(self.class_id).map_err(|e| e.to_string())?;
        }

        // 8. Write is_destroyed (very old versions)
        if header.version < 11 {
            let is_destroyed = self
                .is_destroyed
                .ok_or("is_destroyed must be set for version < 11")?;
            writer.write_u16(is_destroyed).map_err(|e| e.to_string())?;
        }

        // 9. Write script_type_index (versions 11-16)
        if (11..17).contains(&header.version) {
            let serialized_type = self
                .serialized_type
                .as_ref()
                .ok_or("serialized_type required for versions 11-16")?;
            writer
                .write_i16(serialized_type.script_type_index)
                .map_err(|e| e.to_string())?;
        }

        // 10. Write is_stripped (versions 15-16)
        if header.version == 15 || header.version == 16 {
            let is_stripped = self
                .is_stripped
                .ok_or("is_stripped must be set for versions 15-16")?;
            writer.write_u8(is_stripped).map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Gets the container path for this object
    ///
    /// Returns the asset path from the AssetBundle container if it exists.
    ///
    /// Python equivalent: ObjectReader.container property (line 188-189)
    pub fn container(&self) -> Result<Option<String>, String> {
        // Check if we have a reference to the parent SerializedFile
        let assets_file_weak = match &self.assets_file {
            Some(weak_ref) => weak_ref,
            None => return Ok(None), // No parent reference, return None
        };

        // Upgrade Weak to Rc (fails if SerializedFile was dropped)
        let assets_file_rc = assets_file_weak
            .upgrade()
            .ok_or("Parent SerializedFile no longer exists")?;

        // Borrow the SerializedFile and access its container
        let assets_file = assets_file_rc.borrow();

        // Get the path from the container using this object's path_id
        // Python: self.assets_file._container.path_dict.get(self.path_id)
        Ok(assets_file
            .container
            .get_path(self.path_id)
            .map(|s| s.clone()))
    }

    /// Gets the Rust class type for this object from ClassIDTypeToClassMap
    ///
    /// Returns the class name as a string, or None if unknown.
    ///
    /// # Python equivalent
    ///
    /// `ObjectReader.get_class()` in ObjectReader.py (line 175)
    ///
    /// # Parity Notes
    ///
    /// ⚠️ **Partial parity**: Python returns the actual class type (Type[T]) which can be
    /// instantiated, while Rust returns the class name as a string. This is due to Rust's
    /// type system limitations - Rust cannot return "types" like Python does.
    ///
    /// Python: `return ClassIDTypeToClassMap.get(self.type)  # Returns GameObject class`
    /// Rust: `return Ok(Some("GameObject".to_string()))      # Returns string name`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let class_name = object_reader.get_class()?;
    /// match class_name {
    ///     Some(name) => println!("Object is a {}", name),
    ///     None => println!("Unknown object type"),
    /// }
    /// ```
    pub fn get_class(&self) -> Result<Option<String>, String> {
        // Convert i32 type_id to ClassIDType enum
        let class_id_type = ClassIDType::from(self.type_id);

        // Check if this ClassIDType has a mapping
        if has_mapping(class_id_type) {
            // Get the class instance and return its type name
            let class_instance = get_class_for_type(class_id_type);
            Ok(Some(class_instance.type_name().to_string()))
        } else {
            // No mapping found - return None
            Ok(None)
        }
    }
}

// === Display Trait ===

impl<T> fmt::Display for ObjectReader<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<ObjectReader {:?}>", self.obj_type)
    }
}

// === Clone Implementation ===
// Manual implementation because T doesn't need to be Clone
// (T is just a PhantomData marker)

impl<T> Clone for ObjectReader<T> {
    fn clone(&self) -> Self {
        Self {
            reader: self.reader.clone(),
            assets_file: self.assets_file.clone(),
            ref_types: self.ref_types.clone(),
            big_id_enabled: self.big_id_enabled,
            data: RefCell::new(self.data.borrow().clone()),
            version: self.version,
            version2: self.version2,
            platform: self.platform,
            build_type: self.build_type.clone(),
            path_id: self.path_id,
            byte_start_offset: self.byte_start_offset,
            byte_start: self.byte_start,
            byte_size_offset: self.byte_size_offset,
            byte_size: self.byte_size,
            type_id: self.type_id,
            serialized_type: self.serialized_type.clone(),
            class_id: self.class_id,
            obj_type: self.obj_type,
            is_destroyed: self.is_destroyed,
            is_stripped: self.is_stripped,
            read_until: self.read_until,
            byte_header_offset: self.byte_header_offset,
            byte_base_offset: self.byte_base_offset,
            _phantom: PhantomData,
        }
    }
}

// === ObjectReaderTrait Implementation ===
impl<T: fmt::Debug + 'static> ObjectReaderTrait for ObjectReader<T> {
    fn assets_file(&self) -> Option<&SerializedFile> {
        self.assets_file
            .as_ref()
            .and_then(|weak| weak.upgrade())
            .map(|rc| unsafe { &*rc.as_ptr() })
    }

    fn save_typetree(&self, obj: &dyn crate::classes::object::Object) -> Result<(), String> {
        // Get the JSON representation of the object
        let json = obj.to_json_value();

        // Create a writer with the correct endianness
        let mut writer = EndianBinaryWriter::new(self.reader.borrow().endian());

        // Get the typetree node for this object type
        let node = match &self.serialized_type {
            Some(st) => st.nodes().ok_or("No typetree nodes available")?.clone(),
            None => return Err("No serialized type available".to_string()),
        };

        // Write the typetree
        write_typetree(&UnityValue::Json(json), &node, &mut writer, &self.ref_types)?;

        // Get the written bytes and update self.data using interior mutability
        let data = writer.to_bytes();
        *self.data.borrow_mut() = data;

        // Note: Python also calls self.assets_file.mark_changed() here,
        // but we can't do that with &self. The SerializedFile.mark_changed()
        // requires &mut self. This is an architectural difference:
        // - Python can modify through references
        // - Rust requires &mut for modification
        // The data is updated in ObjectReader, but the parent file's
        // is_changed flag won't be set until the file is explicitly saved.

        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
