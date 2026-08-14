//! THROWAWAY diagnostic: dump a `PostProcessProfile`'s settings by pathID, and report which clips
//! animate the volume's weight.
//!
//! Usage: cargo run --release --example `probe_ppprofile` -- <bundle.ab> <profile-pathID>
use serde_json::Value;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn dump(v: &Value, depth: usize, path: &str) {
    if depth > 5 {
        return;
    }
    match v {
        Value::Object(m) => {
            for (k, vv) in m {
                if k.starts_with("m_") && k != "m_Name" && k != "m_Enabled" {
                    continue;
                }
                match vv {
                    Value::Object(_) | Value::Array(_) => {
                        dump(vv, depth + 1, &format!("{path}.{k}"));
                    }
                    _ => println!("   {path}.{k} = {vv}"),
                }
            }
        }
        Value::Array(a) => {
            for (i, vv) in a.iter().enumerate().take(20) {
                dump(vv, depth + 1, &format!("{path}[{i}]"));
            }
        }
        _ => {}
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle");
    let want: i64 = std::env::args()
        .nth(2)
        .expect("pathID")
        .parse()
        .expect("int");
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    for entry in &bundle.files {
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        for obj in &sf.objects {
            if obj.path_id != want {
                continue;
            }
            let Ok(v) = read_object(&sf, obj) else {
                continue;
            };
            println!("profile object (class {}):", obj.class_id);
            println!("RAW: {}", {
                let t = format!("{v}");
                if t.len() > 600 {
                    format!("{}…", &t[..600])
                } else {
                    t
                }
            });
            // Follow the settings PPtr array and dump each settings MonoBehaviour.
            if let Some(arr) = v.get("settings").and_then(Value::as_array) {
                for s in arr {
                    let Some(pid) = s.get("m_PathID").and_then(Value::as_i64) else {
                        continue;
                    };
                    for o2 in &sf.objects {
                        if o2.path_id != pid {
                            continue;
                        }
                        let Ok(sv) = read_object(&sf, o2) else {
                            continue;
                        };
                        println!("\n  --- settings pathID {pid}:");
                        dump(&sv, 0, "  ");
                    }
                }
            }
            dump(&v, 0, "");
        }
    }
}
