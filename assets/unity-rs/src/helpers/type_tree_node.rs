use crate::helpers::tpk::UnityVersion;
use crate::streams::endian::Endian;
use crate::streams::endian_reader::BinaryReader;
use crate::streams::endian_writer::BinaryWriter;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;

static NAME_PEEK_NODE_CACHE: Lazy<
    Mutex<HashMap<(String, String, i32), Option<(TypeTreeNode, String)>>>,
> = Lazy::new(|| Mutex::new(HashMap::new()));
static COMMONSTRING_CACHE: Lazy<Mutex<HashMap<Option<UnityVersion>, HashMap<usize, String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Represents a node in Unity's type tree
#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TypeTreeNode {
    pub m_level: i32,
    pub m_type: String,
    pub m_name: String,
    pub m_byte_size: i32,
    pub m_version: i32,
    pub m_children: Vec<TypeTreeNode>,
    pub m_type_flags: Option<i32>,
    pub m_variable_count: Option<i32>,
    pub m_index: Option<i32>,
    pub m_meta_flag: Option<u32>,
    pub m_ref_type_hash: Option<u64>,

    // Computed field
    #[serde(skip)]
    pub clean_name: String,
}

/// Manual Clone implementation using iterative approach to avoid stack overflow
/// on deeply nested type trees (common in Unity 2021+ files)
impl Clone for TypeTreeNode {
    fn clone(&self) -> Self {
        // Clone scalar fields for root
        let mut root = TypeTreeNode {
            m_level: self.m_level,
            m_type: self.m_type.clone(),
            m_name: self.m_name.clone(),
            m_byte_size: self.m_byte_size,
            m_version: self.m_version,
            m_children: Vec::with_capacity(self.m_children.len()),
            m_type_flags: self.m_type_flags,
            m_variable_count: self.m_variable_count,
            m_index: self.m_index,
            m_meta_flag: self.m_meta_flag,
            m_ref_type_hash: self.m_ref_type_hash,
            clean_name: self.clean_name.clone(),
        };

        if self.m_children.is_empty() {
            return root;
        }

        // Work stack: (source_children, target_parent_ptr, child_index)
        // We process children iteratively to avoid deep recursion
        let mut work_stack: Vec<(&[TypeTreeNode], *mut TypeTreeNode, usize)> = vec![];
        work_stack.push((&self.m_children, &mut root as *mut TypeTreeNode, 0));

        while let Some((src_children, target_parent, idx)) = work_stack.pop() {
            if idx >= src_children.len() {
                continue;
            }

            // Push next sibling back onto stack
            if idx + 1 < src_children.len() {
                work_stack.push((src_children, target_parent, idx + 1));
            }

            let src_node = &src_children[idx];

            // Clone scalar fields for this node
            let new_node = TypeTreeNode {
                m_level: src_node.m_level,
                m_type: src_node.m_type.clone(),
                m_name: src_node.m_name.clone(),
                m_byte_size: src_node.m_byte_size,
                m_version: src_node.m_version,
                m_children: Vec::with_capacity(src_node.m_children.len()),
                m_type_flags: src_node.m_type_flags,
                m_variable_count: src_node.m_variable_count,
                m_index: src_node.m_index,
                m_meta_flag: src_node.m_meta_flag,
                m_ref_type_hash: src_node.m_ref_type_hash,
                clean_name: src_node.clean_name.clone(),
            };

            // Add to parent's children
            // SAFETY: We maintain exclusive access to target_parent through our stack discipline
            unsafe {
                (*target_parent).m_children.push(new_node);

                // If this node has children, push them onto the work stack
                if !src_node.m_children.is_empty() {
                    let new_node_ptr =
                        (*target_parent).m_children.last_mut().unwrap() as *mut TypeTreeNode;
                    work_stack.push((&src_node.m_children, new_node_ptr, 0));
                }
            }
        }

        root
    }
}

/// Manual Debug implementation to avoid stack overflow on deeply nested trees
impl std::fmt::Debug for TypeTreeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Count total nodes iteratively to avoid recursion
        let mut total_nodes = 1;
        let mut stack: Vec<&TypeTreeNode> = self.m_children.iter().collect();
        while let Some(node) = stack.pop() {
            total_nodes += 1;
            stack.extend(node.m_children.iter());
        }

        f.debug_struct("TypeTreeNode")
            .field("m_level", &self.m_level)
            .field("m_type", &self.m_type)
            .field("m_name", &self.m_name)
            .field("m_byte_size", &self.m_byte_size)
            .field("m_version", &self.m_version)
            .field(
                "m_children",
                &format!(
                    "[{} children, {} total nodes]",
                    self.m_children.len(),
                    total_nodes
                ),
            )
            .field("m_type_flags", &self.m_type_flags)
            .field("m_meta_flag", &self.m_meta_flag)
            .finish()
    }
}

impl TypeTreeNode {
    /// Creates a new TypeTreeNode
    pub fn new(
        m_level: i32,
        m_type: String,
        m_name: String,
        m_byte_size: i32,
        m_version: i32,
    ) -> Self {
        let clean_name = clean_name_str(&m_name);

        TypeTreeNode {
            m_level,
            m_type,
            m_name,
            m_byte_size,
            m_version,
            m_children: Vec::new(),
            m_type_flags: None,
            m_variable_count: None,
            m_index: None,
            m_meta_flag: None,
            m_ref_type_hash: None,
            clean_name,
        }
    }

    /// Gets the cleaned name (sanitized for Rust identifiers)
    pub fn clean_name(&self) -> &str {
        &self.clean_name
    }

    /// Builds a tree structure from a flat list of nodes
    /// Nodes must be ordered by depth-first traversal
    pub fn from_list(nodes: Vec<TypeTreeNode>) -> Result<TypeTreeNode, String> {
        if nodes.is_empty() {
            return Err("Node list is empty".to_string());
        }

        // Create a fake root node at level -1
        let mut fake_root = TypeTreeNode::new(-1, String::new(), String::new(), 0, 0);
        let mut stack: Vec<*mut TypeTreeNode> = vec![&mut fake_root as *mut TypeTreeNode];
        let mut parent: *mut TypeTreeNode = &mut fake_root;
        let mut prev: *mut TypeTreeNode = &mut fake_root;

        for node in nodes {
            unsafe {
                // Determine parent based on level changes
                if node.m_level > (*prev).m_level {
                    // Going deeper - previous node becomes parent
                    stack.push(parent);
                    parent = prev;
                } else if node.m_level < (*prev).m_level {
                    // Going up - pop stack until we find the right parent
                    while node.m_level <= (*parent).m_level {
                        parent = stack.pop().ok_or("Stack underflow")?;
                    }
                }

                // Add node as child of current parent
                (*parent).m_children.push(node);

                // Update prev to point to the last child we just added
                let children_len = (*parent).m_children.len();
                prev = &mut (&mut (*parent).m_children)[children_len - 1] as *mut TypeTreeNode;
            }
        }

        // Return the first (and only) child of fake_root
        fake_root
            .m_children
            .into_iter()
            .next()
            .ok_or("No root node found".to_string())
    }

    /// Returns an iterator over all nodes in depth-first order
    pub fn traverse(&self) -> TypeTreeTraverseIter<'_> {
        TypeTreeTraverseIter { stack: vec![self] }
    }

    // Finds the first child with name "m_Name" or "name" and returns a peek node
    ///
    /// This is an optimization for quickly reading object names. It creates a
    /// truncated node containing only fields up to the name field, which can be
    /// parsed much faster than the full object.
    ///
    /// Results are cached globally for performance.
    ///
    /// # Returns
    ///
    /// `Some((peek_node, name_field))` if a name field is found, `None` otherwise
    pub fn get_name_peek_node(&self) -> Option<(TypeTreeNode, String)> {
        let key = (self.m_name.clone(), self.m_type.clone(), self.m_version);

        // Check cache first
        {
            let cache = NAME_PEEK_NODE_CACHE.lock().unwrap();
            if let Some(cached) = cache.get(&key) {
                return cached.clone();
            }
        } // Lock released here

        // Search for name field
        let mut result = None;
        for (i, child) in self.m_children.iter().enumerate() {
            if child.m_name == "m_Name" || child.m_name == "name" {
                // Create peek node with only children up to (and including) this one
                let peek_node = TypeTreeNode {
                    m_level: self.m_level,
                    m_type: self.m_type.clone(),
                    m_name: self.m_name.clone(),
                    m_byte_size: self.m_byte_size,
                    m_version: self.m_version,
                    m_children: self.m_children[..=i].to_vec(),
                    m_type_flags: self.m_type_flags,
                    m_variable_count: self.m_variable_count,
                    m_index: self.m_index,
                    m_meta_flag: self.m_meta_flag,
                    m_ref_type_hash: self.m_ref_type_hash,
                    clean_name: self.clean_name.clone(),
                };

                result = Some((peek_node, child.m_name.clone()));
                break;
            }
        }

        // Cache the result
        {
            let mut cache = NAME_PEEK_NODE_CACHE.lock().unwrap();
            cache.insert(key, result.clone());
        }

        result
    }

    /// Converts the TypeTreeNode to a JSON-compatible dictionary
    ///
    /// Includes all non-None fields, including the m_Children array.
    ///
    /// # Returns
    ///
    /// A `serde_json::Value` object containing the node's fields
    pub fn to_dict(&self) -> Value {
        let mut map = serde_json::Map::new();

        map.insert("m_Level".to_string(), json!(self.m_level));
        map.insert("m_Type".to_string(), json!(self.m_type));
        map.insert("m_Name".to_string(), json!(self.m_name));
        map.insert("m_ByteSize".to_string(), json!(self.m_byte_size));
        map.insert("m_Version".to_string(), json!(self.m_version));

        // Include children as array of dicts (recursive)
        let children: Vec<Value> = self
            .m_children
            .iter()
            .map(|child| child.to_dict())
            .collect();
        map.insert("m_Children".to_string(), json!(children));

        if let Some(v) = self.m_type_flags {
            map.insert("m_TypeFlags".to_string(), json!(v));
        }
        if let Some(v) = self.m_variable_count {
            map.insert("m_VariableCount".to_string(), json!(v));
        }
        if let Some(v) = self.m_index {
            map.insert("m_Index".to_string(), json!(v));
        }
        if let Some(v) = self.m_meta_flag {
            map.insert("m_MetaFlag".to_string(), json!(v));
        }
        if let Some(v) = self.m_ref_type_hash {
            map.insert("m_RefTypeHash".to_string(), json!(v));
        }

        Value::Object(map)
    }

    /// Converts the entire tree to a flat list of dictionaries
    ///
    /// Uses depth-first traversal to flatten the tree structure.
    /// Each node is represented as a dictionary (via `to_dict()`).
    ///
    /// # Returns
    ///
    /// A vector of `serde_json::Value` objects representing all nodes
    pub fn to_dict_list(&self) -> Vec<Value> {
        self.traverse().map(|node| node.to_dict()).collect()
    }

    /// Dumps the tree structure in a human-readable format
    ///
    /// Similar to AssetRipper's TypeTreeDumps format. Shows the type hierarchy
    /// with indentation and metadata for each node.
    ///
    /// # Arguments
    ///
    /// * `indent` - The indentation string to use (default "  ")
    ///
    /// # Returns
    ///
    /// A formatted string showing the tree structure
    pub fn dump_structure(&self, indent: &str) -> String {
        let mut result = format!(
            "{}{} {} // ByteSize{{0x{:X}}}, Index{{{:?}}}, Version{{{}}}, TypeFlags{{{:?}}}, MetaFlag{{{:?}}}",
            indent,
            self.m_type,
            self.m_name,
            self.m_byte_size,
            self.m_index,
            self.m_version,
            self.m_type_flags,
            self.m_meta_flag
        );

        for child in &self.m_children {
            result.push('\n');
            result.push_str(&child.dump_structure(&format!("{}  ", indent)));
        }

        result
    }

    /// Parses a TypeTreeNode tree from a binary stream
    ///
    /// Uses a stack-based approach for efficiency (much faster than recursion).
    /// The format varies slightly based on the Unity serialization version.
    ///
    /// # Arguments
    ///
    /// * `reader` - Binary reader positioned at the start of the type tree data
    /// * `version` - Unity serialization format version (2, 3, or other)
    ///
    /// # Returns
    ///
    /// The root TypeTreeNode with all children parsed
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails or data is malformed
    pub fn parse<R: BinaryReader>(reader: &mut R, version: i32) -> std::io::Result<TypeTreeNode> {
        // Using a fake root node to avoid special case for root node
        let dummy_node = TypeTreeNode::new(-1, String::new(), String::new(), 0, 0);
        let mut dummy_root = TypeTreeNode::new(-1, String::new(), String::new(), 0, 0);
        dummy_root.m_children = vec![dummy_node];

        let mut stack: Vec<(*mut TypeTreeNode, usize)> =
            vec![(&mut dummy_root as *mut TypeTreeNode, 1)];

        while !stack.is_empty() {
            let (parent_ptr, count) = *stack.last().unwrap();

            if count == 1 {
                stack.pop();
            } else {
                let last_idx = stack.len() - 1;
                stack[last_idx] = (parent_ptr, count - 1);
            }

            unsafe {
                let parent = &mut *parent_ptr;

                // Read node data
                let m_level = parent.m_level + 1;
                let m_type = reader.read_string(None)?;
                let m_name = reader.read_string(None)?;
                let m_byte_size = reader.read_i32()?;
                let m_variable_count = if version == 2 {
                    Some(reader.read_i32()?)
                } else {
                    None
                };
                let m_index = if version != 3 {
                    Some(reader.read_i32()?)
                } else {
                    None
                };
                let m_type_flags = Some(reader.read_i32()?);
                let m_version = reader.read_i32()?;
                let m_meta_flag = if version != 3 {
                    Some(reader.read_u32()?)
                } else {
                    None
                };

                let mut node = TypeTreeNode::new(m_level, m_type, m_name, m_byte_size, m_version);
                node.m_variable_count = m_variable_count;
                node.m_index = m_index;
                node.m_type_flags = m_type_flags;
                node.m_meta_flag = m_meta_flag;

                // Set node as child at correct position
                let child_idx = parent.m_children.len() - count;
                parent.m_children[child_idx] = node;

                // Read children count and prepare placeholders
                let children_count = reader.read_i32()? as usize;
                if children_count > 0 {
                    let node_ref = &mut parent.m_children[child_idx];
                    node_ref.m_children =
                        vec![
                            TypeTreeNode::new(-1, String::new(), String::new(), 0, 0);
                            children_count
                        ];
                    stack.push((node_ref as *mut TypeTreeNode, children_count));
                }
            }
        }

        dummy_root.m_children.into_iter().next().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "No root node found")
        })
    }

    /// Parses a TypeTreeNode tree from blob format
    ///
    /// Blob format is an optimized binary representation where strings are stored
    /// in a separate string buffer to avoid duplication. Nodes reference strings
    /// by offset.
    ///
    /// # Arguments
    ///
    /// * `reader` - Binary reader positioned at the start of blob data
    /// * `version` - Unity serialization format version
    ///
    /// # Returns
    ///
    /// The root TypeTreeNode with all children parsed
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails or data is malformed
    pub fn parse_blob<R: BinaryReader>(
        reader: &mut R,
        version: i32,
    ) -> std::io::Result<TypeTreeNode> {
        let node_count = reader.read_i32()? as usize;
        let stringbuffer_size = reader.read_i32()? as usize;

        // Determine struct layout
        let (_, struct_size) = get_blob_node_struct(reader.endian(), version);

        // Read all node data
        let nodes_data = reader.read(struct_size * node_count)?;

        // Read string buffer
        let stringbuffer_data = reader.read(stringbuffer_size)?;
        let mut string_reader =
            crate::streams::endian_reader::MemoryReader::new(stringbuffer_data, reader.endian(), 0);

        // Get common strings for string resolution
        let common_strings = get_common_strings(None).unwrap_or_else(|_| HashMap::new());

        // Helper function to read string from buffer
        let read_string = |value: u32,
                           string_reader: &mut crate::streams::endian_reader::MemoryReader|
         -> std::io::Result<String> {
            let is_offset = (value & 0x80000000) == 0;
            if is_offset {
                string_reader.set_position(value as usize);
                string_reader.read_string(None)
            } else {
                let offset = (value & 0x7FFFFFFF) as usize;
                Ok(common_strings
                    .get(&offset)
                    .cloned()
                    .unwrap_or_else(|| offset.to_string()))
            }
        };

        // Parse nodes from binary data
        let mut node_cursor = std::io::Cursor::new(nodes_data);
        let mut nodes = Vec::with_capacity(node_count);

        for _ in 0..node_count {
            use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

            // Read struct fields based on endianness
            let m_version = match reader.endian() {
                Endian::Little => node_cursor.read_i16::<LittleEndian>()?,
                Endian::Big => node_cursor.read_i16::<BigEndian>()?,
            } as i32;

            let m_level = node_cursor.read_u8()? as i32;
            let m_type_flags = Some(node_cursor.read_u8()? as i32);

            let type_str_offset = match reader.endian() {
                Endian::Little => node_cursor.read_u32::<LittleEndian>()?,
                Endian::Big => node_cursor.read_u32::<BigEndian>()?,
            };
            let name_str_offset = match reader.endian() {
                Endian::Little => node_cursor.read_u32::<LittleEndian>()?,
                Endian::Big => node_cursor.read_u32::<BigEndian>()?,
            };

            let m_byte_size = match reader.endian() {
                Endian::Little => node_cursor.read_i32::<LittleEndian>()?,
                Endian::Big => node_cursor.read_i32::<BigEndian>()?,
            };

            let m_index = Some(match reader.endian() {
                Endian::Little => node_cursor.read_i32::<LittleEndian>()?,
                Endian::Big => node_cursor.read_i32::<BigEndian>()?,
            });

            let m_meta_flag = Some(match reader.endian() {
                Endian::Little => node_cursor.read_u32::<LittleEndian>()?,
                Endian::Big => node_cursor.read_u32::<BigEndian>()?,
            });

            let m_ref_type_hash = if version >= 19 {
                Some(match reader.endian() {
                    Endian::Little => node_cursor.read_u64::<LittleEndian>()?,
                    Endian::Big => node_cursor.read_u64::<BigEndian>()?,
                })
            } else {
                None
            };

            // Read strings from string buffer
            let m_type = read_string(type_str_offset, &mut string_reader)?;
            let m_name = read_string(name_str_offset, &mut string_reader)?;

            let mut node = TypeTreeNode::new(m_level, m_type, m_name, m_byte_size, m_version);
            node.m_type_flags = m_type_flags;
            node.m_variable_count = None;
            node.m_index = m_index;
            node.m_meta_flag = m_meta_flag;
            node.m_ref_type_hash = m_ref_type_hash;

            nodes.push(node);
        }

        // Build tree from flat list using from_list
        TypeTreeNode::from_list(nodes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Writes the TypeTreeNode tree to a binary stream
    ///
    /// The format varies based on Unity serialization version.
    /// Uses depth-first traversal to write all nodes.
    ///
    /// # Arguments
    ///
    /// * `writer` - Binary writer to write the tree data
    /// * `version` - Unity serialization format version (2, 3, or other)
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails or required fields are None
    pub fn dump<W: BinaryWriter>(&self, writer: &mut W, version: i32) -> std::io::Result<()> {
        let mut stack: Vec<&TypeTreeNode> = vec![self];

        while let Some(node) = stack.pop() {
            writer.write_string_to_null(&node.m_type)?;
            writer.write_string_to_null(&node.m_name)?;
            writer.write_i32(node.m_byte_size)?;

            if version == 2 {
                let variable_count = node.m_variable_count.ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "m_VariableCount required for version 2",
                    )
                })?;
                writer.write_i32(variable_count)?;
            }

            if version != 3 {
                let index = node.m_index.ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "m_Index required for version != 3",
                    )
                })?;
                writer.write_i32(index)?;
            }

            writer.write_i32(node.m_type_flags.unwrap_or(0))?;
            writer.write_i32(node.m_version)?;

            if version != 3 {
                let meta_flag = node.m_meta_flag.ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "m_MetaFlag required for version != 3",
                    )
                })?;
                writer.write_u32(meta_flag)?;
            }

            writer.write_i32(node.m_children.len() as i32)?;

            // Add children in reverse order for correct traversal
            for child in node.m_children.iter().rev() {
                stack.push(child);
            }
        }

        Ok(())
    }

    /// Writes the TypeTreeNode tree to blob format
    ///
    /// Blob format deduplicates strings by storing them in a separate buffer
    /// and referencing them by offset. Common Unity strings are marked with
    /// a special flag.
    ///
    /// # Arguments
    ///
    /// * `writer` - Binary writer to write the blob data
    /// * `version` - Unity serialization format version
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails
    pub fn dump_blob<W: BinaryWriter>(&self, writer: &mut W, version: i32) -> std::io::Result<()> {
        use crate::streams::endian_writer::EndianBinaryWriter;

        let mut node_writer = EndianBinaryWriter::new(writer.endian());
        let mut string_writer = EndianBinaryWriter::new(Endian::Little);

        // Get common string mappings (string → offset with flag)
        let common_strings = get_common_strings(None).unwrap_or_else(|_| HashMap::new());
        let common_string_offset_map: HashMap<String, usize> = common_strings
            .into_iter()
            .map(|(offset, string)| (string, offset))
            .collect();

        let mut string_offsets: HashMap<String, u32> = HashMap::new();

        // Helper to write a string and return its offset
        let write_string = |s: &str,
                            string_offsets: &mut HashMap<String, u32>,
                            string_writer: &mut EndianBinaryWriter|
         -> u32 {
            if let Some(&offset) = string_offsets.get(s) {
                return offset;
            }

            // Check if it's a common string
            if let Some(&common_offset) = common_string_offset_map.get(s) {
                let offset = (common_offset as u32) | 0x80000000;
                string_offsets.insert(s.to_string(), offset);
                offset
            } else {
                let offset = string_writer.position() as u32;
                string_writer.write_string_to_null(s).unwrap();
                string_offsets.insert(s.to_string(), offset);
                offset
            }
        };

        // Traverse tree and write all nodes
        let mut node_count = 0;
        for node in self.traverse() {
            use byteorder::{BigEndian, LittleEndian, WriteBytesExt};

            let type_offset = write_string(&node.m_type, &mut string_offsets, &mut string_writer);
            let name_offset = write_string(&node.m_name, &mut string_offsets, &mut string_writer);

            // Write node struct
            let mut temp_buffer = Vec::new();

            match writer.endian() {
                Endian::Little => {
                    temp_buffer
                        .write_i16::<LittleEndian>(node.m_version as i16)
                        .unwrap();
                }
                Endian::Big => {
                    temp_buffer
                        .write_i16::<BigEndian>(node.m_version as i16)
                        .unwrap();
                }
            }

            temp_buffer.push(node.m_level as u8);
            temp_buffer.push(node.m_type_flags.unwrap_or(0) as u8);

            match writer.endian() {
                Endian::Little => {
                    temp_buffer.write_u32::<LittleEndian>(type_offset).unwrap();
                    temp_buffer.write_u32::<LittleEndian>(name_offset).unwrap();
                    temp_buffer
                        .write_i32::<LittleEndian>(node.m_byte_size)
                        .unwrap();
                    temp_buffer
                        .write_i32::<LittleEndian>(node.m_index.unwrap_or(0))
                        .unwrap();
                    temp_buffer
                        .write_u32::<LittleEndian>(node.m_meta_flag.unwrap_or(0))
                        .unwrap();
                }
                Endian::Big => {
                    temp_buffer.write_u32::<BigEndian>(type_offset).unwrap();
                    temp_buffer.write_u32::<BigEndian>(name_offset).unwrap();
                    temp_buffer
                        .write_i32::<BigEndian>(node.m_byte_size)
                        .unwrap();
                    temp_buffer
                        .write_i32::<BigEndian>(node.m_index.unwrap_or(0))
                        .unwrap();
                    temp_buffer
                        .write_u32::<BigEndian>(node.m_meta_flag.unwrap_or(0))
                        .unwrap();
                }
            }

            if version >= 19 {
                match writer.endian() {
                    Endian::Little => {
                        temp_buffer
                            .write_u64::<LittleEndian>(node.m_ref_type_hash.unwrap_or(0))
                            .unwrap();
                    }
                    Endian::Big => {
                        temp_buffer
                            .write_u64::<BigEndian>(node.m_ref_type_hash.unwrap_or(0))
                            .unwrap();
                    }
                }
            }

            node_writer.write(&temp_buffer)?;
            node_count += 1;
        }

        // Write blob header and data
        writer.write_i32(node_count)?;
        writer.write_i32(string_writer.position() as i32)?;
        writer.write(&node_writer.to_bytes())?;
        writer.write(&string_writer.to_bytes())?;

        Ok(())
    }
}

/// Retrieves common Unity string constants from the TPK tree
///
/// These are frequently-used strings like "m_GameObject", "m_Name", etc.
/// that are stored in a compressed format in the TPK file. The function
/// builds a mapping from byte offset to string value.
///
/// # Arguments
///
/// * `version` - Optional Unity version to filter strings (Some versions use fewer common strings)
///
/// # Returns
///
/// A HashMap mapping byte offsets to string values
///
/// # Errors
///
/// Returns an error if the TPK tree hasn't been initialized or if there's a version mismatch
pub fn get_common_strings(version: Option<UnityVersion>) -> Result<HashMap<usize, String>, String> {
    // Check cache first
    {
        let cache = COMMONSTRING_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(&version) {
            return Ok(cached.clone());
        }
    }

    // Get strings from TPK tree (hold lock only briefly)
    let strings = crate::helpers::tpk::get_common_string_list(version)?;

    // Build offset → string mapping
    let mut result = HashMap::new();
    let mut offset: usize = 0;
    for string in strings {
        result.insert(offset, string.clone());
        offset += string.len() + 1; // +1 for null terminator
    }

    // Cache the result
    {
        let mut cache = COMMONSTRING_CACHE.lock().unwrap();
        cache.insert(version, result.clone());
    }

    Ok(result)
}

/// Helper function that defines the binary structure layout for blob nodes
///
/// Returns the field names and their binary layout for parsing TypeTreeNode
/// from blob format. The structure varies based on Unity version.
///
/// # Arguments
///
/// * `endian` - The byte order (Little or Big endian)
/// * `version` - The Unity serialization version
///
/// # Returns
///
/// A tuple of (field_names, total_size_bytes)
pub fn get_blob_node_struct(_endian: Endian, version: i32) -> (Vec<&'static str>, usize) {
    let base_fields = vec![
        "m_Version",       // i16 (2 bytes)
        "m_Level",         // u8  (1 byte)
        "m_TypeFlags",     // u8  (1 byte)
        "m_TypeStrOffset", // u32 (4 bytes)
        "m_NameStrOffset", // u32 (4 bytes)
        "m_ByteSize",      // i32 (4 bytes)
        "m_Index",         // i32 (4 bytes)
        "m_MetaFlag",      // i32 (4 bytes)
    ];

    let mut fields = base_fields;
    let mut size = 2 + 1 + 1 + 4 + 4 + 4 + 4 + 4; // 24 bytes

    if version >= 19 {
        fields.push("m_RefTypeHash"); // u64 (8 bytes)
        size += 8; // 32 bytes total
    }

    (fields, size)
}

/// Iterator for traversing TypeTreeNode
pub struct TypeTreeTraverseIter<'a> {
    stack: Vec<&'a TypeTreeNode>,
}

impl<'a> Iterator for TypeTreeTraverseIter<'a> {
    type Item = &'a TypeTreeNode;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            // Add children in reverse order (so left-most is processed first)
            for child in node.m_children.iter().rev() {
                self.stack.push(child);
            }
            Some(node)
        } else {
            None
        }
    }
}

/// Sanitizes a field name for use as a Rust identifier
pub fn clean_name_str(name: &str) -> String {
    if name.is_empty() {
        return String::new();
    }

    let mut name = name.to_string();

    // Remove prefix
    if name.starts_with("(int&)") {
        name = name[6..].to_string();
    }

    // Remove suffix
    if name.ends_with('?') {
        name = name[..name.len() - 1].to_string();
    }

    // Replace invalid characters with underscore
    let re = Regex::new(r"[ \.:\-\[\]]").unwrap();
    name = re.replace_all(&name, "_").to_string();

    // Handle Rust keywords
    if name == "pass" || name == "from" {
        name.push('_');
    }

    // Prepend 'x' if starts with digit
    if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        name = format!("x{}", name);
    }

    name
}
