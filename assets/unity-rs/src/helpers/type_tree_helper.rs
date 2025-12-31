use crate::classes::class_id_type_to_class_map::{
    deserialize_typed_object, get_class_id_from_name,
};
use crate::classes::{Object, PPtr, UnknownObject};
use crate::files::serialized_file::SerializedType;
use crate::helpers::tpk;
use crate::helpers::type_tree_node::TypeTreeNode;
use crate::streams::endian_reader::BinaryReader;
use crate::streams::endian_writer::BinaryWriter;
use serde_json::{json, Map, Value};

const K_ALIGN_BYTES: u32 = 0x4000;

pub enum UnityValue<'a> {
    Json(Value),             // Dict mode
    Object(Box<dyn Object>), // UnknownObject or typed classes
    PPtr(PPtr<'a, ()>),      // Type-erased PPtr (PPtr[Any] in Python)
}

impl<'a> UnityValue<'a> {
    /// Converts this value to JSON, regardless of mode
    pub fn to_json(&self) -> Value {
        match self {
            UnityValue::Json(v) => v.clone(),
            UnityValue::Object(obj) => {
                if let Some(unknown) = (**obj).as_any().downcast_ref::<UnknownObject>() {
                    unknown.to_json()
                } else {
                    // Typed object - use serde serialization
                    obj.to_json_value()
                }
            }
            UnityValue::PPtr(pptr) => {
                json!({
                    "m_FileID": pptr.m_file_id,
                    "m_PathID": pptr.m_path_id
                })
            }
        }
    }

    /// Returns true if this is a JSON value
    pub fn is_json(&self) -> bool {
        matches!(self, UnityValue::Json(_))
    }

    /// Unwraps as JSON, panics if not JSON
    pub fn as_json(&self) -> &Value {
        match self {
            UnityValue::Json(v) => v,
            _ => panic!("Not a JSON value"),
        }
    }

    /// Tries to get as JSON, returns None if not JSON
    pub fn try_as_json(&self) -> Option<&Value> {
        match self {
            UnityValue::Json(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct TypeTreeConfig {
    pub as_dict: bool,
    pub assetsfile: Option<*const ()>, // Will store SerializedFile reference
    pub has_registry: bool,
}

impl TypeTreeConfig {
    pub fn new(as_dict: bool, assetsfile: Option<*const ()>) -> Self {
        Self {
            as_dict,
            assetsfile,
            has_registry: false,
        }
    }

    pub fn copy(&self) -> Self {
        Self {
            as_dict: self.as_dict,
            assetsfile: self.assetsfile,
            has_registry: self.has_registry,
        }
    }
}

/// Checks if a metaflag indicates alignment is needed
///
/// Python equivalent: lines 390-391
///
/// # Arguments
///
/// * `meta_flag` - The metaflag value from TypeTreeNode
///
/// # Returns
///
/// `true` if the kAlignBytes flag is set, `false` otherwise
pub fn metaflag_is_aligned(meta_flag: Option<u32>) -> bool {
    ((meta_flag.unwrap_or(0)) & K_ALIGN_BYTES) != 0
}

/// Resolves a type string that may be a numeric type ID to its actual type name
///
/// When Unity uses numeric type IDs (e.g., "49") instead of type names (e.g., "Array"),
/// this function looks up the type name from the TPK common strings buffer.
///
/// # Arguments
///
/// * `type_str` - The type string to resolve (may be numeric or already a type name)
///
/// # Returns
///
/// The resolved type name, or the original string if it's not a numeric ID or resolution fails
pub fn resolve_type_id(type_str: &str) -> String {
    // Check if the type string is purely numeric
    if type_str.chars().all(|c| c.is_ascii_digit()) {
        // Try to parse as index
        if let Ok(index) = type_str.parse::<usize>() {
            // Try to get common strings from TPK
            if let Ok(common_strings) = tpk::get_common_string_list(None) {
                // Return the string at this index if it exists
                if index < common_strings.len() {
                    return common_strings[index].clone();
                }
            }
        }
    }

    // If resolution fails or type is not numeric, return original
    type_str.to_string()
}

/// Resolves referenced type nodes for ReferencedObject types
///
/// Python equivalent: lines 87-114
///
/// # Arguments
///
/// * `ref_object` - The partially-read ReferencedObject data containing type info
/// * `ref_types` - The ref_types array from SerializedFile
///
/// # Returns
///
/// The TypeTreeNode for the referenced type, or None if class is empty
///
/// # Errors
///
/// Returns error if ref_types is empty or referenced type not found
pub fn get_ref_type_node(
    ref_object: &Value,
    ref_types: &Option<Vec<SerializedType>>,
) -> Result<Option<TypeTreeNode>, String> {
    let typ = ref_object
        .get("type")
        .ok_or("ReferencedObject missing 'type' field")?;

    let (cls, ns, asm) = if typ.is_object() {
        let cls = typ
            .get("class")
            .and_then(|v| v.as_str())
            .ok_or("type.class missing or not string")?;
        let ns = typ
            .get("ns")
            .and_then(|v| v.as_str())
            .ok_or("type.ns missing or not string")?;
        let asm = typ
            .get("asm")
            .and_then(|v| v.as_str())
            .ok_or("type.asm missing or not string")?;
        (cls, ns, asm)
    } else {
        return Err("Object format for type not supported in Rust".to_string());
    };

    let ref_types = ref_types
        .as_ref()
        .ok_or("SerializedFile has no ref_types")?;

    if cls.is_empty() {
        return Ok(None);
    }

    for ref_type in ref_types {
        if ref_type.m_class_name.as_deref() == Some(cls)
            && ref_type.m_name_space.as_deref() == Some(ns)
            && ref_type.m_assembly_name.as_deref() == Some(asm)
        {
            return Ok(ref_type.node.clone());
        }
    }

    Err(format!("Referenced type not found: {} {} {}", cls, ns, asm))
}

/// Reads a Unity object from binary using its type tree
///
/// Python equivalent: lines 117-156
///
/// # Arguments
///
/// * `root_node` - The root TypeTreeNode describing the object structure
/// * `reader` - Binary reader positioned at object data
/// * `as_dict` - If true, returns dict; if false, constructs Object instances
/// * `byte_size` - Expected number of bytes to read (for validation)
/// * `check_read` - If true, validates that exactly byte_size bytes were read
/// * `ref_types` - Optional ref_types from SerializedFile (for ReferencedObject)
///
/// # Returns
///
/// The deserialized object as a serde_json::Value
///
/// # Errors
///
/// Returns error if reading fails or byte count mismatch (when check_read=true)
pub fn read_typetree<'a, R: BinaryReader>(
    root_node: &TypeTreeNode,
    reader: &mut R,
    as_dict: bool,
    byte_size: Option<usize>,
    check_read: bool,
    ref_types: &Option<Vec<SerializedType>>,
) -> Result<UnityValue<'a>, String> {
    let pos = reader.position();
    let mut config = TypeTreeConfig::new(as_dict, None);
    let obj = read_value(root_node, reader, &mut config, ref_types)?;
    let bytes_read = reader.position() - pos;

    if check_read {
        if let Some(expected) = byte_size {
            if bytes_read != expected {
                return Err(format!(
                    "Expected to read {} bytes, but only read {} bytes",
                    expected, bytes_read
                ));
            }
        }
    }

    Ok(obj)
}

/// Writes a Unity object to binary using its type tree
///
/// Python equivalent: lines 159-177
///
/// # Arguments
///
/// * `value` - The object to serialize (as dict or Object)
/// * `root_node` - The root TypeTreeNode describing the object structure
/// * `writer` - Binary writer to write data to
/// * `ref_types` - Optional ref_types from SerializedFile (for ReferencedObject)
///
/// # Errors
///
/// Returns error if writing fails
pub fn write_typetree<W: BinaryWriter>(
    value: &UnityValue,
    root_node: &TypeTreeNode,
    writer: &mut W,
    ref_types: &Option<Vec<SerializedType>>,
) -> Result<(), String> {
    let json_value = match value {
        UnityValue::Json(v) => v.clone(),
        UnityValue::Object(obj) => {
            if let Some(unknown) = (**obj).as_any().downcast_ref::<UnknownObject>() {
                unknown.to_json()
            } else {
                // Typed object - use serde serialization
                obj.to_json_value()
            }
        }
        UnityValue::PPtr(pptr) => {
            json!({
                "m_FileID": pptr.m_file_id,
                "m_PathID": pptr.m_path_id
            })
        }
    };

    let is_dict = json_value.is_object();
    let mut config = TypeTreeConfig::new(is_dict, None);
    write_value(&json_value, root_node, writer, &mut config, ref_types)
}

fn read_primitive_type<R: BinaryReader>(
    type_name: &str,
    reader: &mut R,
) -> Option<Result<Value, String>> {
    let result = match type_name {
        // Integers
        "SInt8" => reader.read_i8().map(|v| json!(v)),
        "UInt8" | "char" => reader.read_u8().map(|v| json!(v)),
        "short" | "SInt16" => reader.read_i16().map(|v| json!(v)),
        "unsigned short" | "UInt16" => reader.read_u16().map(|v| json!(v)),
        "int" | "SInt32" => reader.read_i32().map(|v| json!(v)),
        "unsigned int" | "UInt32" | "Type*" => reader.read_u32().map(|v| json!(v)),
        "long long" | "SInt64" => reader.read_i64().map(|v| json!(v)),
        "unsigned long long" | "UInt64" | "FileSize" => reader.read_u64().map(|v| json!(v)),

        // Floats
        "float" => reader.read_f32().map(|v| json!(v)),
        "double" => reader.read_f64().map(|v| json!(v)),

        // Boolean
        "bool" => reader.read_bool().map(|v| json!(v)),

        // String
        "string" => reader.read_aligned_string().map(|v| json!(v)),

        // Byte array
        "TypelessData" => reader.read_byte_array().map(|v| json!(v)),

        // Not a primitive type
        _ => return None,
    };

    Some(result.map_err(|e| e.to_string()))
}

fn read_primitive_type_array<R: BinaryReader>(
    type_name: &str,
    reader: &mut R,
    size: usize,
) -> Option<Result<Value, String>> {
    let result = match type_name {
        "bool" => {
            let mut arr = Vec::new();
            for _ in 0..size {
                match reader.read_bool() {
                    Ok(v) => arr.push(v),
                    Err(e) => return Some(Err(e.to_string())),
                }
            }
            Ok(json!(arr))
        }
        "SInt8" => {
            let mut arr = Vec::new();
            for _ in 0..size {
                match reader.read_i8() {
                    Ok(v) => arr.push(v),
                    Err(e) => return Some(Err(e.to_string())),
                }
            }
            Ok(json!(arr))
        }
        "UInt8" | "char" => {
            let mut arr = Vec::new();
            for _ in 0..size {
                match reader.read_u8() {
                    Ok(v) => arr.push(v),
                    Err(e) => return Some(Err(e.to_string())),
                }
            }
            Ok(json!(arr))
        }
        "short" | "SInt16" => {
            let mut arr = Vec::new();
            for _ in 0..size {
                match reader.read_i16() {
                    Ok(v) => arr.push(v),
                    Err(e) => return Some(Err(e.to_string())),
                }
            }
            Ok(json!(arr))
        }
        "unsigned short" | "UInt16" => {
            let mut arr = Vec::new();
            for _ in 0..size {
                match reader.read_u16() {
                    Ok(v) => arr.push(v),
                    Err(e) => return Some(Err(e.to_string())),
                }
            }
            Ok(json!(arr))
        }
        "int" | "SInt32" => {
            let mut arr = Vec::new();
            for _ in 0..size {
                match reader.read_i32() {
                    Ok(v) => arr.push(v),
                    Err(e) => return Some(Err(e.to_string())),
                }
            }
            Ok(json!(arr))
        }
        "unsigned int" | "UInt32" | "Type*" => {
            let mut arr = Vec::new();
            for _ in 0..size {
                match reader.read_u32() {
                    Ok(v) => arr.push(v),
                    Err(e) => return Some(Err(e.to_string())),
                }
            }
            Ok(json!(arr))
        }
        "long long" | "SInt64" => {
            let mut arr = Vec::new();
            for _ in 0..size {
                match reader.read_i64() {
                    Ok(v) => arr.push(v),
                    Err(e) => return Some(Err(e.to_string())),
                }
            }
            Ok(json!(arr))
        }
        "unsigned long long" | "UInt64" | "FileSize" => {
            let mut arr = Vec::new();
            for _ in 0..size {
                match reader.read_u64() {
                    Ok(v) => arr.push(v),
                    Err(e) => return Some(Err(e.to_string())),
                }
            }
            Ok(json!(arr))
        }
        "float" => {
            let mut arr = Vec::new();
            for _ in 0..size {
                match reader.read_f32() {
                    Ok(v) => arr.push(v),
                    Err(e) => return Some(Err(e.to_string())),
                }
            }
            Ok(json!(arr))
        }
        "double" => {
            let mut arr = Vec::new();
            for _ in 0..size {
                match reader.read_f64() {
                    Ok(v) => arr.push(v),
                    Err(e) => return Some(Err(e.to_string())),
                }
            }
            Ok(json!(arr))
        }

        _ => return None,
    };

    Some(result)
}

/// Recursively reads a value based on its TypeTreeNode
///
/// Python equivalent: lines 180-263
///
/// This is the heart of the deserialization system. It dispatches to the
/// appropriate reader based on the node's m_Type field.
///
/// # Arguments
///
/// * `node` - The TypeTreeNode describing this value
/// * `reader` - Binary reader positioned at value data
/// * `config` - Configuration for dict vs Object mode
/// * `ref_types` - Optional ref_types from SerializedFile
///
/// # Returns
///
/// The deserialized value as serde_json::Value
pub fn read_value<'a, R: BinaryReader>(
    node: &TypeTreeNode,
    reader: &mut R,
    config: &mut TypeTreeConfig,
    ref_types: &Option<Vec<SerializedType>>,
) -> Result<UnityValue<'a>, String> {
    let mut align = metaflag_is_aligned(node.m_meta_flag);

    // We'll build the value in this big match statement
    let value = match node.m_type.as_str() {
        // Try primitive types first (Python lines 188-190)
        type_name => {
            if let Some(result) = read_primitive_type(type_name, reader) {
                UnityValue::Json(result?)
            } else if type_name == "pair" {
                let first = read_value(&node.m_children[0], reader, config, ref_types)?;
                let second = read_value(&node.m_children[1], reader, config, ref_types)?;
                UnityValue::Json(json!([first.to_json(), second.to_json()]))
            } else if type_name == "ReferencedObject" {
                let mut value = Map::new();
                for child in &node.m_children {
                    if child.m_type == "ReferencedObjectData" {
                        let ref_type_node = get_ref_type_node(&json!(value), ref_types)?;
                        if let Some(ref_node) = ref_type_node {
                            value.insert(
                                child.m_name.clone(),
                                read_value(&ref_node, reader, config, ref_types)?.to_json(),
                            );
                        }
                    } else {
                        value.insert(
                            child.m_name.clone(),
                            read_value(child, reader, config, ref_types)?.to_json(),
                        );
                    }
                }

                UnityValue::Json(Value::Object(value))
            } else if !node.m_children.is_empty() {
                // Resolve numeric type IDs to actual type names before checking for Array
                let first_child_type = resolve_type_id(&node.m_children[0].m_type);

                if first_child_type == "Array" {
                    if metaflag_is_aligned(node.m_children[0].m_meta_flag) {
                        align = true;
                    }

                    let size = reader.read_i32().map_err(|e| e.to_string())? as usize;
                    let subtype = &node.m_children[0].m_children[1];
                    if metaflag_is_aligned(subtype.m_meta_flag) {
                        read_value_array(subtype, reader, config, size, ref_types)?
                    } else {
                        let mut arr = Vec::new();
                        for _ in 0..size {
                            arr.push(read_value(subtype, reader, config, ref_types)?.to_json());
                        }
                        UnityValue::Json(json!(arr))
                    }
                } else {
                    // Non-array complex types
                    let mut value = Map::new();
                    for child in &node.m_children {
                        if child.m_type == "ManagedReferencesRegistry" {
                            if config.has_registry {
                                continue;
                            } else {
                                *config = config.copy();
                                config.has_registry = true;
                            }
                        }
                        let field_name = if config.as_dict {
                            child.m_name.clone()
                        } else {
                            child.clean_name.clone()
                        };

                        value.insert(
                            field_name,
                            read_value(child, reader, config, ref_types)?.to_json(),
                        );
                    }

                    if !config.as_dict {
                        if node.m_type.starts_with("PPtr<") {
                            let m_file_id =
                                value.get("m_FileID").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                            let m_path_id =
                                value.get("m_PathID").and_then(|v| v.as_i64()).unwrap_or(0);

                            UnityValue::PPtr(PPtr::new(m_file_id, m_path_id))
                        } else {
                            // Try to create typed object (Python line 239-258)
                            if let Some(class_id) = get_class_id_from_name(&node.m_type) {
                                // We have a mapping! Try to deserialize into typed object
                                match deserialize_typed_object(
                                    class_id,
                                    Value::Object(value.clone()),
                                ) {
                                    Ok(typed_obj) => {
                                        // Success! Return the typed object
                                        UnityValue::Object(typed_obj)
                                    }
                                    Err(_) => {
                                        // Deserialization failed, fall back to UnknownObject
                                        let unknown = UnknownObject::new(Some(node.clone()), value);
                                        UnityValue::Object(Box::new(unknown))
                                    }
                                }
                            } else {
                                // No mapping exists, use UnknownObject
                                let unknown = UnknownObject::new(Some(node.clone()), value);
                                UnityValue::Object(Box::new(unknown))
                            }
                        }
                    } else {
                        UnityValue::Json(Value::Object(value))
                    }
                }
            } else {
                // Handle nodes with no children - should not normally happen
                UnityValue::Json(Value::Null)
            }
        }
    };

    // Align if needed (Python lines 260-261)
    if align {
        reader.align_stream(4);
    }

    Ok(value)
}

/// Optimized array reader for aligned primitive types
///
/// Python equivalent: lines 266-387
///
/// When reading arrays of primitive types that have alignment requirements,
/// this function reads them more efficiently than calling read_value() per element.
pub fn read_value_array<'a, R: BinaryReader>(
    node: &TypeTreeNode,
    reader: &mut R,
    config: &mut TypeTreeConfig,
    size: usize,
    ref_types: &Option<Vec<SerializedType>>,
) -> Result<UnityValue<'a>, String> {
    let mut align = metaflag_is_aligned(node.m_meta_flag);

    let value = match node.m_type.as_str() {
        // Try primitive types first (Python lines 188-190)
        type_name => {
            if let Some(result) = read_primitive_type_array(type_name, reader, size) {
                UnityValue::Json(result?)
            } else if type_name == "string" {
                let mut arr = Vec::new();
                for _ in 0..size {
                    arr.push(reader.read_aligned_string().map_err(|e| e.to_string())?);
                }
                UnityValue::Json(json!(arr))
            } else if type_name == "TypelessData" {
                let mut arr = Vec::new();
                for _ in 0..size {
                    arr.push(reader.read_byte_array().map_err(|e| e.to_string())?);
                }
                UnityValue::Json(json!(arr))
            } else if type_name == "pair" {
                let key_node = &node.m_children[0];
                let value_node = &node.m_children[1];

                let mut arr = Vec::new();
                for _ in 0..size {
                    let key = read_value(key_node, reader, config, ref_types)?.to_json();
                    let val = read_value(value_node, reader, config, ref_types)?.to_json();
                    arr.push(json!([key, val]));
                }
                UnityValue::Json(json!(arr))
            } else if type_name == "ReferencedObject" {
                let mut arr = Vec::new();
                for _ in 0..size {
                    let mut item = Map::new();
                    for child in &node.m_children {
                        if child.m_type == "ReferencedObjectData" {
                            let ref_type_nodes = get_ref_type_node(&json!(item), ref_types)?;
                            if let Some(ref_node) = ref_type_nodes {
                                item.insert(
                                    child.m_name.clone(),
                                    read_value(&ref_node, reader, config, ref_types)?.to_json(),
                                );
                            }
                        } else {
                            item.insert(
                                child.m_name.clone(),
                                read_value(child, reader, config, ref_types)?.to_json(),
                            );
                        }
                    }
                    arr.push(Value::Object(item));
                }

                UnityValue::Json(json!(arr))
            } else if !node.m_children.is_empty() && node.m_children[0].m_type == "Array" {
                if metaflag_is_aligned(node.m_children[0].m_meta_flag) {
                    align = true;
                }
                let subtype = &node.m_children[0].m_children[1];
                if metaflag_is_aligned(subtype.m_meta_flag) {
                    let mut arr = Vec::new();
                    for _ in 0..size {
                        let inner_size = reader.read_i32().map_err(|e| e.to_string())? as usize;
                        let item =
                            read_value_array(subtype, reader, config, inner_size, ref_types)?
                                .to_json();
                        arr.push(item)
                    }
                    UnityValue::Json(json!(arr))
                } else {
                    let mut arr = Vec::new();
                    for _ in 0..size {
                        let item = read_value(subtype, reader, config, ref_types)?.to_json();
                        arr.push(item)
                    }
                    UnityValue::Json(json!(arr))
                }
            } else if config.as_dict {
                let mut arr = Vec::new();
                for _ in 0..size {
                    let mut item = Map::new();
                    for child in &node.m_children {
                        item.insert(
                            child.m_name.clone(),
                            read_value(child, reader, config, ref_types)?.to_json(),
                        );
                    }
                    arr.push(Value::Object(item));
                }
                UnityValue::Json(json!(arr))
            } else if node.m_type.starts_with("PPtr<") {
                // Note: node.m_type not type_name
                let mut arr = Vec::new();
                for _ in 0..size {
                    let mut item = Map::new();
                    for child in &node.m_children {
                        item.insert(
                            child.m_name.clone(),
                            read_value(child, reader, config, ref_types)?.to_json(),
                        );
                    }
                    // Construct PPtr representation
                    let m_file_id =
                        item.get("m_FileID").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    let m_path_id = item.get("m_PathID").and_then(|v| v.as_i64()).unwrap_or(0);
                    arr.push(json!({
                        "_type": "PPtr",
                        "m_FileID": m_file_id,
                        "m_PathID": m_path_id
                    }));
                }
                UnityValue::Json(json!(arr))
            } else {
                // For now, return dicts like as_dict mode
                // Python lines 341-383 have complex class construction
                let mut arr = Vec::new();
                for _ in 0..size {
                    let mut item = Map::new();
                    for child in &node.m_children {
                        item.insert(
                            child.clean_name.clone(),
                            read_value(child, reader, config, ref_types)?.to_json(),
                        );
                    }
                    arr.push(Value::Object(item));
                }
                UnityValue::Json(json!(arr))
            }
        }
    };

    if align {
        reader.align_stream(4);
    }

    Ok(value)
}

fn write_primitive_type<W: BinaryWriter>(
    type_name: &str,
    value: &Value,
    writer: &mut W,
) -> Option<Result<(), String>> {
    let result: Result<(), std::io::Error> = match type_name {
        // Integers
        "SInt8" => {
            let v = match value.as_i64() {
                Some(v) => v as i8,
                None => return Some(Err("Expected i8 value".to_string())),
            };
            writer.write_i8(v)
        }
        "UInt8" | "char" => {
            let v = match value.as_u64() {
                Some(v) => v as u8,
                None => return Some(Err("Expected u8 value".to_string())),
            };
            writer.write_u8(v)
        }
        "short" | "SInt16" => {
            let v = match value.as_i64() {
                Some(v) => v as i16,
                None => return Some(Err("Expected i16 value".to_string())),
            };
            writer.write_i16(v)
        }
        "unsigned short" | "UInt16" => {
            let v = match value.as_u64() {
                Some(v) => v as u16,
                None => return Some(Err("Expected u16 value".to_string())),
            };
            writer.write_u16(v)
        }
        "int" | "SInt32" => {
            let v = match value.as_i64() {
                Some(v) => v as i32,
                None => return Some(Err("Expected i32 value".to_string())),
            };
            writer.write_i32(v)
        }
        "unsigned int" | "UInt32" | "Type*" => {
            let v = match value.as_u64() {
                Some(v) => v as u32,
                None => return Some(Err("Expected u32 value".to_string())),
            };
            writer.write_u32(v)
        }
        "long long" | "SInt64" => {
            let v = match value.as_i64() {
                Some(v) => v,
                None => return Some(Err("Expected i64 value".to_string())),
            };
            writer.write_i64(v)
        }
        "unsigned long long" | "UInt64" | "FileSize" => {
            let v = match value.as_u64() {
                Some(v) => v,
                None => return Some(Err("Expected u64 value".to_string())),
            };
            writer.write_u64(v)
        }

        // Floats
        "float" => {
            let v = match value.as_f64() {
                Some(v) => v as f32,
                None => return Some(Err("Expected f32 value".to_string())),
            };
            writer.write_f32(v)
        }
        "double" => {
            let v = match value.as_f64() {
                Some(v) => v,
                None => return Some(Err("Expected f64 value".to_string())),
            };
            writer.write_f64(v)
        }

        // Boolean
        "bool" => {
            let v = match value.as_bool() {
                Some(v) => v,
                None => return Some(Err("Expected bool value".to_string())),
            };
            writer.write_bool(v)
        }

        // String
        "string" => {
            let v = match value.as_str() {
                Some(v) => v,
                None => return Some(Err("Expected string value".to_string())),
            };
            // Manually implement write_aligned_string:
            // 1. Write string length
            let bytes = v.as_bytes();
            writer.write_i32(bytes.len() as i32).ok()?;
            // 2. Write string bytes
            writer.write(bytes).ok()?;
            // 3. Align to 4 bytes
            writer.write_align()
        }

        // Byte array
        "TypelessData" => {
            let arr = match value.as_array() {
                Some(v) => v,
                None => return Some(Err("Expected byte array".to_string())),
            };
            let bytes: Vec<u8> = match arr
                .iter()
                .map(|b| b.as_u64().map(|n| n as u8).ok_or("Invalid byte"))
                .collect::<Result<Vec<u8>, &str>>()
            {
                Ok(b) => b,
                Err(e) => return Some(Err(e.to_string())),
            };
            writer.write_bytes(&bytes)
        }

        // Not a primitive type
        _ => return None,
    };

    Some(result.map_err(|e| e.to_string()))
}

/// Recursively writes a value based on its TypeTreeNode
///
/// Python equivalent: lines 140-191
///
/// This is the counterpart to read_value - it serializes a Value back to binary.
///
/// # Arguments
///
/// * `value` - The serde_json::Value to serialize
/// * `node` - The TypeTreeNode describing the value structure
/// * `writer` - Binary writer to write data to
/// * `config` - Configuration for dict vs Object mode
/// * `ref_types` - Optional ref_types from SerializedFile
///
/// # Errors
///
/// Returns error if writing fails
pub fn write_value<W: BinaryWriter>(
    value: &Value,
    node: &TypeTreeNode,
    writer: &mut W,
    config: &mut TypeTreeConfig,
    ref_types: &Option<Vec<SerializedType>>,
) -> Result<(), String> {
    let mut align = metaflag_is_aligned(node.m_meta_flag);

    let type_name = node.m_type.as_str();

    if let Some(result) = write_primitive_type(type_name, value, writer) {
        result?;
    } else if type_name == "pair" {
        let arr = value.as_array().ok_or("Expected array for pair")?;
        write_value(&arr[0], &node.m_children[0], writer, config, ref_types)?;
        write_value(&arr[1], &node.m_children[1], writer, config, ref_types)?;
    } else if type_name == "ReferencedObject" {
        for child in &node.m_children {
            if child.m_type == "ReferencedObjectData" {
                let ref_type_nodes = get_ref_type_node(value, ref_types)?
                    .ok_or("ReferencedObjectData has empty class")?;
                let field = value
                    .get(&child.m_name)
                    .ok_or_else(|| format!("Missing field: {}", child.m_name))?;
                write_value(field, &ref_type_nodes, writer, config, ref_types)?;
            } else {
                let field = value
                    .get(&child.m_name)
                    .ok_or_else(|| format!("Missing field: {}", child.m_name))?;
                write_value(field, child, writer, config, ref_types)?;
            }
        }
    } else if !node.m_children.is_empty() && node.m_children[0].m_type == "Array" {
        if metaflag_is_aligned(node.m_children[0].m_meta_flag) {
            align = true;
        }

        let arr = value.as_array().ok_or("Expected array")?;
        writer
            .write_i32(arr.len() as i32)
            .map_err(|e| e.to_string())?;

        let subtype = &node.m_children[0].m_children[1];
        for sub_value in arr {
            write_value(sub_value, subtype, writer, config, ref_types)?;
        }
    } else if config.as_dict {
        for child in &node.m_children {
            if child.m_type == "ManagedReferencesRegistry" {
                if config.has_registry {
                    continue;
                } else {
                    *config = config.copy();
                    config.has_registry = true;
                }
            }
            let field = value
                .get(&child.m_name)
                .ok_or_else(|| format!("Missing field: {}", child.m_name))?;
            write_value(field, child, writer, config, ref_types)?;
        }
    } else {
        for child in &node.m_children {
            if child.m_type == "ManagedReferencesRegistry" {
                if config.has_registry {
                    continue;
                } else {
                    *config = config.copy();
                    config.has_registry = true;
                }
            }
            let field = value
                .get(&child.clean_name)
                .ok_or_else(|| format!("Missing field: {}", child.clean_name))?;
            write_value(field, child, writer, config, ref_types)?;
        }
    }

    if align {
        writer.align_stream(4).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Gets common strings from the TPK type tree
///
/// This is a wrapper around tpk::get_common_string_list for API compatibility
///
/// # Arguments
/// * `version` - Optional Unity version tuple (major, minor, patch, build)
///
/// # Returns
/// Result containing vector of common strings
///
/// # Errors
/// Returns error if TPK tree hasn't been initialized
pub fn get_common_strings(version: Option<(u16, u16, u16, u16)>) -> Result<Vec<String>, String> {
    use crate::helpers::tpk::{get_common_string_list, UnityVersion};

    let unity_version = version.map(|(major, minor, patch, build)| {
        UnityVersion::from_parts(major as u64, minor as u64, patch as u64, build as u64)
    });

    get_common_string_list(unity_version)
}
