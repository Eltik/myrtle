use std::io;

use crate::unity::{
    endian_reader::EndianReader,
    type_tree::{TypeTreeNode, read_type_tree_blob},
};

#[allow(dead_code)]
pub struct SerializedFile {
    pub objects: Vec<ObjectInfo>,
    pub types: Vec<SerializedType>,
    pub unity_version: String,
    pub target_platform: i32,
    pub enable_type_tree: bool,
    pub data: Vec<u8>,
    pub data_offset: u64,
    pub big_endian: bool,
}

pub struct ObjectInfo {
    pub path_id: i64,
    pub byte_start: u64,
    pub byte_size: u32,
    pub type_index: usize,
    pub class_id: i32,
}

pub struct SerializedType {
    pub class_id: i32,
    pub type_tree: Option<TypeTreeNode>,
}

impl SerializedFile {
    pub fn parse(data: Vec<u8>) -> Result<Self, io::Error> {
        let mut r = EndianReader::new(&data, true);

        let _metadata_size = r.read_u32()?;
        let _file_size = r.read_u32()?;
        let version = r.read_u32()?;

        let mut data_offset = r.read_u32()? as u64;
        let endian = r.read_u8()?;
        let _ = r.read_bytes(3)?;

        let big_endian = endian != 0;

        if version >= 22 {
            let _metadata_size = r.read_u32()?;
            let _file_size = r.read_u64()?;
            data_offset = r.read_u64()?;
            let _ = r.read_u64()?;
        }

        r.set_big_endian(big_endian);

        let unity_version = r.read_cstring()?;
        let target_platform = r.read_i32()?;
        let enable_type_tree = r.read_bool()?;
        let type_count = r.read_i32()?;

        let mut types = Vec::with_capacity(type_count as usize);
        for _ in 0..type_count {
            let class_id = r.read_i32()?;

            let is_stripped = if version >= 16 { r.read_bool()? } else { false };
            let script_type_index = if version >= 17 { r.read_i16()? } else { -1 };

            if is_stripped || script_type_index >= 0 {
                r.read_bytes(16)?;
            }
            r.read_bytes(16)?;

            let type_tree = if enable_type_tree {
                let tree = read_type_tree_blob(&mut r)?;
                if version >= 21 {
                    // Type dependencies array: count (i32) + count * i32
                    let dep_count = r.read_i32()?;
                    if dep_count > 0 {
                        r.read_bytes((dep_count as usize) * 4)?;
                    }
                }
                Some(tree)
            } else {
                None
            };

            types.push(SerializedType {
                class_id,
                type_tree,
            });
        }

        let object_count = r.read_i32()?;
        let mut objects = Vec::with_capacity(object_count as usize);
        for _ in 0..object_count {
            if version >= 14 {
                r.align(4);
            }
            let path_id = r.read_i64()?;
            let mut byte_start = if version >= 22 {
                r.read_u64()?
            } else {
                r.read_u32()? as u64
            };
            byte_start += data_offset; // absolute offset into data
            let byte_size = r.read_u32()?;
            let type_index = r.read_i32()? as usize;
            let class_id = types[type_index].class_id;

            objects.push(ObjectInfo {
                path_id,
                byte_start,
                byte_size,
                type_index,
                class_id,
            });
        }

        Ok(SerializedFile {
            objects,
            types,
            unity_version,
            target_platform,
            enable_type_tree,
            data,
            data_offset,
            big_endian,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unity::bundle::BundleFile;

    #[test]
    fn test_parse_serialized() {
        let data = match std::fs::read("../downloader/ArkAssets/ui/skin_groups.ab") {
            Ok(d) => d,
            Err(e) => {
                eprintln!("skip: {e}");
                return;
            }
        };
        let bundle = BundleFile::parse(data).unwrap();
        let entry = &bundle.files[0];
        println!("Parsing: {} ({} bytes)", entry.path, entry.data.len());

        let sf = SerializedFile::parse(entry.data.clone()).unwrap();
        println!("Unity: {}", sf.unity_version);
        println!("Platform: {}", sf.target_platform);
        println!("Type trees: {}", sf.enable_type_tree);
        println!("Types ({}):", sf.types.len());
        for (i, t) in sf.types.iter().enumerate() {
            let tree_info = match &t.type_tree {
                Some(n) => format!("root={}/{}", n.type_name, n.name),
                None => "no tree".to_string(),
            };
            println!("  [{i}] class_id={} {tree_info}", t.class_id);
        }
        println!("Objects ({}):", sf.objects.len());
        for obj in &sf.objects {
            println!(
                "  path_id={} class={} offset={} size={}",
                obj.path_id, obj.class_id, obj.byte_start, obj.byte_size
            );
        }
    }
}
