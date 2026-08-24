//! THROWAWAY: dump the UV-SCROLL MonoBehaviour (script pathID 1369540917083035942) — the
//! component that scrolls a NAMED material map at runtime and that the exporter reads nowhere.
//! Prints the host GameObject, its prefab root, and every scroll field.
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

const UVSCROLL: i64 = 1_369_540_917_083_035_942;

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}
fn f(v: &Value, k: &str) -> f64 {
    v.get(k).and_then(Value::as_f64).unwrap_or(0.0)
}

fn main() {
    for path in std::env::args().skip(1) {
        let p2 = path.clone();
        let _ = std::panic::catch_unwind(move || scan(&p2));
    }
}
fn scan(path: &str) {
    {
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
        println!("=== {path}");
        let mut rows: Vec<String> = Vec::new();
        for (cid, v) in objs.values() {
            if *cid != 114 {
                continue;
            }
            let Some(script) = v.get("m_Script").and_then(pid) else {
                continue;
            };
            if script != UVSCROLL {
                continue;
            }
            let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
            let (xs, ys) = (f(v, "xspeed"), f(v, "yspeed"));
            let use2 = v
                .get("useSecondMap")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                || f(v, "useSecondMap") != 0.0;
            let map2 = v.get("secondMapName").and_then(Value::as_str).unwrap_or("");
            let (sx, sy) = (f(v, "secondXSpeed"), f(v, "secondYSpeed"));
            let keep = v
                .get("keepInitOffset")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if xs == 0.0 && ys == 0.0 && sx == 0.0 && sy == 0.0 {
                continue;
            }
            rows.push(format!(
                "  {:<30} main=({xs:+.4},{ys:+.4})  use2={use2:<5} map2={:<18} second=({sx:+.4},{sy:+.4}) keepInit={keep}",
                name_of(go), map2));
        }
        rows.sort();
        println!("  {} components with a NON-ZERO speed", rows.len());
        for r in rows {
            println!("{r}");
        }
    }
}
