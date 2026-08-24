//! THROWAWAY: does any staged bundle define <path_id>, and with what class? Detects
//! cross-bundle path_id COLLISIONS, which a flat `all_objects` map silently resolves
//! to whichever bundle was read last.
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};
fn main() {
    let mut a = std::env::args().skip(1);
    let want: i64 = a.next().expect("path_id").parse().expect("i64");
    for path in a {
        let p2 = path.clone();
        let _ = std::panic::catch_unwind(move || {
            scan(&p2, want);
        });
    }
}
fn scan(path: &str, want: i64) {
    {
        let Ok(data) = std::fs::read(path) else {
            return;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            return;
        };
        for e in &bundle.files {
            let Ok(sf) = SerializedFile::parse(e.data.to_vec()) else {
                continue;
            };
            for o in &sf.objects {
                if o.path_id == want {
                    let name = read_object(&sf, o)
                        .ok()
                        .and_then(|v| v.get("m_Name").and_then(|n| n.as_str().map(str::to_string)))
                        .unwrap_or_default();
                    println!("  class={:<5} name={:<28} in {}", o.class_id, name, path);
                }
            }
        }
    }
}
