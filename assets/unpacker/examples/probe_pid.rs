//! THROWAWAY diagnostic: dump the full JSON of specific objects by pathID (any class).
//!
//! Usage: cargo run --release --example `probe_pid` -- <bundle.ab> <pathID> [<pathID>...]
#![allow(clippy::case_sensitive_file_extension_comparisons)]

use serde_json::Value;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("bundle path");
    let want: Vec<i64> = args.map(|s| s.parse().expect("pathID int")).collect();
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        for obj in &sf.objects {
            if !want.contains(&obj.path_id) {
                continue;
            }
            match read_object(&sf, obj) {
                Ok(v) => println!(
                    "pid={} class={} :: {}",
                    obj.path_id,
                    obj.class_id,
                    serde_json::to_string_pretty(&v).unwrap_or_default()
                ),
                Err(e) => println!("pid={} class={} read error: {e}", obj.path_id, obj.class_id),
            }
        }
    }
    let _ = Value::Null;
}
