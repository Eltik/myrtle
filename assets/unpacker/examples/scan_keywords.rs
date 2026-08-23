//! THROWAWAY: census of `m_ValidKeywords` across every Material in a bundle, so a shader
//! VARIANT (e.g. `_HG_UV_ROTATION`) can be told apart from a shader feature nothing enables.
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn main() {
    let mut tally: HashMap<String, usize> = HashMap::new();
    let mut mats = 0usize;
    for path in std::env::args().skip(1) {
        let p2 = path.clone();
        let r = std::panic::catch_unwind(move || {
            let mut local: HashMap<String, usize> = HashMap::new();
            let mut n = 0usize;
            let Ok(data) = std::fs::read(&p2) else { return (local, n) };
            let Ok(bundle) = BundleFile::parse(data) else { return (local, n) };
            for e in &bundle.files {
                let Ok(sf) = SerializedFile::parse(e.data.to_vec()) else { continue };
                for o in &sf.objects {
                    if o.class_id != 21 { continue }
                    let Ok(v) = read_object(&sf, o) else { continue };
                    n += 1;
                    if let Some(kws) = v.get("m_ValidKeywords").and_then(Value::as_array) {
                        for k in kws {
                            if let Some(s) = k.as_str() { *local.entry(s.to_string()).or_default() += 1; }
                        }
                    }
                }
            }
            (local, n)
        });
        if let Ok((local, n)) = r {
            mats += n;
            for (k, v) in local { *tally.entry(k).or_default() += v; }
        }
    }
    let mut rows: Vec<_> = tally.into_iter().collect();
    rows.sort_by_key(|r| std::cmp::Reverse(r.1));
    println!("materials scanned: {mats}");
    for (k, v) in rows { println!("  {v:>6}  {k}"); }
}
