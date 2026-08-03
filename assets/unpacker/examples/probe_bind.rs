use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};
fn main() {
    let path = std::env::args().nth(1).unwrap();
    let data = std::fs::read(&path).unwrap();
    let bundle = BundleFile::parse(data).unwrap();
    for entry in &bundle.files {
        let l = entry.path.to_ascii_lowercase();
        if l.ends_with(".ress") || l.ends_with(".resource") { continue; }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        let skip: HashSet<i32> = [28,43,48,49,83,128,213].into_iter().collect();
        for obj in &sf.objects {
            if obj.class_id != 74 || skip.contains(&obj.class_id) { continue; }
            let Ok(v) = read_object(&sf, obj) else { continue };
            let n = v.get("m_Name").and_then(Value::as_str).unwrap_or("");
            if !n.ends_with("epoque#28_Start") { continue; }
            println!("clip '{n}'");
            if let Some(bs) = v.get("m_ClipBindingConstant").and_then(|b| b.get("genericBindings")).and_then(Value::as_array) {
                println!("  genericBindings: {} entries; first 3 raw:", bs.len());
                for b in bs.iter().take(3) { println!("    {}", serde_json::to_string(b).unwrap_or_default()); }
                let mut by_ty: HashMap<i64, usize> = HashMap::new();
                for b in bs { *by_ty.entry(b.get("typeID").and_then(Value::as_i64).unwrap_or(-1)).or_default() += 1; }
                println!("  typeID histogram: {by_ty:?}");
            } else { println!("  NO genericBindings"); }
            return;
        }
    }
}
