//! THROWAWAY: dump every MonoBehaviour with a given script pathID, with all of its scalar
//! fields, so an UNREAD authored subsystem can be sized before anything is implemented.
//!
//! Usage: probe_script <script_path_id> <bundle.ab> [bundle.ab ...]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn main() {
    let mut a = std::env::args().skip(1);
    let want: i64 = a.next().expect("script path id").parse().expect("i64");

    for path in a {
        let p2 = path.clone();
        let _ = std::panic::catch_unwind(move || scan(&p2, want));
    }
}

fn scan(path: &str, want: i64) {
    let Ok(data) = std::fs::read(path) else {
        return;
    };
    let Ok(bundle) = BundleFile::parse(data) else {
        return;
    };
    let mut objs: HashMap<i64, (i32, Value)> = HashMap::new();
    for e in &bundle.files {
        let Ok(sf) = SerializedFile::parse(e.data.to_vec()) else {
            continue;
        };
        for o in &sf.objects {
            if let Ok(v) = read_object(&sf, o) {
                objs.insert(o.path_id, (o.class_id, v));
            }
        }
    }
    let name_of = |go: i64| -> String {
        objs.get(&go)
            .and_then(|(_, v)| v.get("m_Name"))
            .and_then(Value::as_str)
            .unwrap_or("?")
            .to_string()
    };
    let mut rows: Vec<String> = Vec::new();
    for (cid, v) in objs.values() {
        if *cid != 114 {
            continue;
        }
        if v.get("m_Script").and_then(pid) != Some(want) {
            continue;
        }
        let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
        if std::env::var("RAW").is_ok() {
            println!(
                "--- {} ---\n{}",
                name_of(go),
                serde_json::to_string_pretty(v).unwrap_or_default()
            );
            continue;
        }
        let mut fields: Vec<String> = Vec::new();
        if let Some(o) = v.as_object() {
            for (k, val) in o {
                if k.starts_with("m_") {
                    continue;
                }
                match val {
                    Value::Number(n) => fields.push(format!("{k}={n}")),
                    Value::Bool(b) => fields.push(format!("{k}={b}")),
                    Value::String(s) if !s.is_empty() => fields.push(format!("{k}=\"{s}\"")),
                    Value::Array(arr) => fields.push(format!("{k}=[{} items]", arr.len())),
                    _ => {}
                }
            }
        }
        fields.sort();
        rows.push(format!("  {:<34} {}", name_of(go), fields.join(" ")));
    }
    if rows.is_empty() {
        return;
    }
    rows.sort();
    println!(
        "=== {} ({} instances)",
        path.rsplit('/').next().unwrap_or(path),
        rows.len()
    );
    for r in rows {
        println!("{r}");
    }
}
