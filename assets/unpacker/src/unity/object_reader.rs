use crate::unity::endian_reader::EndianReader;
use crate::unity::serialized_file::{ObjectInfo, SerializedFile};
use crate::unity::type_tree::TypeTreeNode;
use base64::Engine;
use serde_json::{Value, json};
use std::io;

pub fn read_object(file: &SerializedFile, obj: &ObjectInfo) -> Result<Value, io::Error> {
    let mut r = EndianReader::new(&file.data, file.big_endian);
    r.set_position(obj.byte_start as usize);
    let type_tree = file.types[obj.type_index]
        .type_tree
        .as_ref()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "no type tree"))?;
    read_value(&mut r, type_tree)
}

fn read_value(r: &mut EndianReader, node: &TypeTreeNode) -> Result<Value, io::Error> {
    let val = match node.type_name.as_str() {
        "bool" => Value::Bool(r.read_bool()?),
        "SInt8" | "char" => Value::Number(r.read_i8()?.into()),
        "UInt8" => Value::Number(r.read_u8()?.into()),
        "SInt16" | "short" => Value::Number(r.read_i16()?.into()),
        "UInt16" | "unsigned short" => Value::Number(r.read_u16()?.into()),
        "SInt32" | "int" => Value::Number(r.read_i32()?.into()),
        "UInt32" | "unsigned int" | "Type*" => Value::Number(r.read_u32()?.into()),
        "SInt64" | "long long" => Value::Number(r.read_i64()?.into()),
        "UInt64" | "unsigned long long" | "FileSize" => {
            let v = r.read_u64()?;
            if v > i64::MAX as u64 {
                Value::String(v.to_string())
            } else {
                Value::Number((v as i64).into())
            }
        }
        "float" => json!(r.read_f32()?),
        "double" => json!(r.read_f64()?),
        "string" => {
            let len = r.read_i32()?;
            if len < 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("negative string length: {len}"),
                ));
            }
            let bytes = r.read_bytes(len as usize)?;
            r.align(4);
            // If valid UTF-8, store as string; otherwise store as base64
            match String::from_utf8(bytes.clone()) {
                Ok(s) => Value::String(s),
                Err(_) => Value::String(format!(
                    "base64:{}",
                    base64::engine::general_purpose::STANDARD.encode(&bytes)
                )),
            }
        }
        "TypelessData" => {
            let size = r.read_i32()?;
            if size < 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("negative TypelessData size: {size}"),
                ));
            }
            let bytes = r.read_bytes(size as usize)?;
            r.align(4);
            Value::String(base64::engine::general_purpose::STANDARD.encode(&bytes))
        }
        _ if !node.children.is_empty() && node.children[0].type_name == "Array" => {
            // This is a container with an Array child (e.g., vector, set, etc.)
            read_array(r, &node.children[0])?
        }
        _ => {
            // Struct: read each child into a JSON object
            let mut map = serde_json::Map::new();
            for child in &node.children {
                map.insert(child.name.clone(), read_value(r, child)?);
            }
            Value::Object(map)
        }
    };

    // Post-read alignment
    if node.meta_flag & 0x4000 != 0 {
        r.align(4);
    }

    Ok(val)
}

fn read_array(r: &mut EndianReader, array_node: &TypeTreeNode) -> Result<Value, io::Error> {
    // array_node.children[0] = "size" (int), children[1] = element type
    let size = r.read_i32()?;
    if size < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("negative array size: {size}"),
        ));
    }
    let size = size as usize;
    let element_node = &array_node.children[1];

    // Cap initial capacity to avoid huge allocations when size is valid but large;
    // the loop will still read exactly `size` elements (failing on EOF if data is short).
    const MAX_INITIAL_CAP: usize = 1024;

    // Check if this is a map (element type is "pair")
    if element_node.type_name == "pair" {
        let mut map = serde_json::Map::with_capacity(size.min(MAX_INITIAL_CAP));
        for _ in 0..size {
            let key = read_value(r, &element_node.children[0])?;
            let val = read_value(r, &element_node.children[1])?;
            let key_str = match key {
                Value::String(s) => s,
                other => other.to_string(),
            };
            map.insert(key_str, val);
        }
        Ok(Value::Object(map))
    } else {
        let mut arr = Vec::with_capacity(size.min(MAX_INITIAL_CAP));
        for _ in 0..size {
            arr.push(read_value(r, element_node)?);
        }
        Ok(Value::Array(arr))
    }
}

pub fn read_objects_by_class(file: &SerializedFile, class_ids: &[i32]) -> Vec<(i64, Value)> {
    file.objects
        .iter()
        .filter(|obj| class_ids.contains(&obj.class_id))
        .filter_map(|obj| read_object(file, obj).ok().map(|v| (obj.path_id, v)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unity::bundle::BundleFile;
    use crate::unity::serialized_file::SerializedFile;

    /// Bug: read_array with corrupted huge size (negative i32) causes capacity overflow panic
    /// instead of returning an error.
    #[test]
    fn test_read_array_corrupted_size_no_panic() {
        use crate::unity::endian_reader::EndianReader;
        use crate::unity::type_tree::TypeTreeNode;

        // Create a minimal type tree: root struct with one Array child containing int elements
        let element_node = TypeTreeNode {
            type_name: "int".to_string(),
            name: "data".to_string(),
            byte_size: 4,
            version: 0,
            level: 2,
            type_flags: 0,
            meta_flag: 0,
            children: vec![],
        };
        let size_node = TypeTreeNode {
            type_name: "int".to_string(),
            name: "size".to_string(),
            byte_size: 4,
            version: 0,
            level: 2,
            type_flags: 0,
            meta_flag: 0,
            children: vec![],
        };
        let array_node = TypeTreeNode {
            type_name: "Array".to_string(),
            name: "Array".to_string(),
            byte_size: -1,
            version: 0,
            level: 1,
            type_flags: 0,
            meta_flag: 0x4000,
            children: vec![size_node, element_node],
        };
        let root = TypeTreeNode {
            type_name: "vector".to_string(),
            name: "m_Container".to_string(),
            byte_size: -1,
            version: 0,
            level: 0,
            type_flags: 0,
            meta_flag: 0,
            children: vec![array_node],
        };

        // Data contains array size = -1 (0xFFFFFFFF) — wraps to usize::MAX, should not panic
        let mut data = vec![0xFF, 0xFF, 0xFF, 0xFF]; // i32 = -1 in LE
        data.extend_from_slice(&[0u8; 64]); // some padding

        let mut r = EndianReader::new(&data, false);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            super::read_value(&mut r, &root)
        }));
        assert!(
            result.is_ok(),
            "read_value should not panic on huge array size"
        );
        assert!(
            result.unwrap().is_err(),
            "should return Err for corrupted array size"
        );
    }

    #[test]
    fn test_read_object() {
        let data = match std::fs::read("../downloader/ArkAssets/ui/skin_groups.ab") {
            Ok(d) => d,
            Err(e) => {
                eprintln!("skip: {e}");
                return;
            }
        };
        let bundle = BundleFile::parse(data).unwrap();
        let sf = SerializedFile::parse(bundle.files[0].data.clone()).unwrap();

        let targets = [28, 49, 83, 142];

        for obj in sf.objects.iter().filter(|o| targets.contains(&o.class_id)) {
            match read_object(&sf, obj) {
                Ok(val) => {
                    let s = serde_json::to_string(&val).unwrap();
                    let preview = if s.len() > 200 {
                        format!("{}...", &s[..200])
                    } else {
                        s
                    };
                    println!(
                        "OK  path_id={} class={} => {}",
                        obj.path_id, obj.class_id, preview
                    );
                }
                Err(e) => {
                    if obj.class_id != 213 {
                        println!(
                            "ERR path_id={} class={} offset={} size={} => {}",
                            obj.path_id, obj.class_id, obj.byte_start, obj.byte_size, e
                        );
                    }
                }
            }
        }
    }
}
