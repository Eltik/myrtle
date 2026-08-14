#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::many_single_char_names
)]
use std::collections::HashMap;
use unpacker::unity::{bundle::BundleFile, serialized_file::SerializedFile};
fn main() {
    for path in std::env::args().skip(1) {
        let Ok(d) = std::fs::read(&path) else {
            continue;
        };
        let Ok(b) = BundleFile::parse(d) else {
            continue;
        };
        println!("== {}", path.rsplit('/').next().unwrap());
        for e in &b.files {
            let l = e.path.to_ascii_lowercase();
            if l.ends_with(".ress") || l.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(e.data.clone()) else {
                continue;
            };
            let mut h: HashMap<i32, usize> = HashMap::new();
            for o in &sf.objects {
                *h.entry(o.class_id).or_default() += 1;
            }
            let mut v: Vec<_> = h.into_iter().collect();
            v.sort_unstable();
            println!("  file {} : {} objects  {:?}", e.path, sf.objects.len(), v);
        }
    }
}
