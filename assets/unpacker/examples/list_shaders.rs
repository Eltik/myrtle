//! THROWAWAY: list every Shader object in a bundle with its pathID and name, so a shader can be
//! fed to `probe_shadersrc`, which takes a pathID rather than a name.
use serde_json::Value;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn main() {
    let path = std::env::args().nth(1).expect("shaders.ab");
    let filter = std::env::args().nth(2).unwrap_or_default();
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        for obj in &sf.objects {
            if obj.class_id != 48 {
                continue;
            }
            let Ok(v) = read_object(&sf, obj) else { continue };
            let name = v
                .get("m_ParsedForm")
                .and_then(|p| p.get("m_Name"))
                .and_then(Value::as_str)
                .unwrap_or("?");
            if filter.is_empty() || name.contains(&filter) {
                println!("  pathID {:>22}  {}", obj.path_id, name);
            }
        }
    }
}
