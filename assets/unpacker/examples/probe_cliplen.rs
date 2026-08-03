//! THROWAWAY: dump an AnimationClip's top-level keys and any length-bearing fields.
//! Usage: cargo run --release --example probe_cliplen -- <bundle.ab> <name-substr>
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};
fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("bundle");
    let want = a.next().unwrap_or_default().to_lowercase();
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") { continue; }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        let skip: HashSet<i32> = [28, 43, 48, 49, 83, 128, 213].into_iter().collect();
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for obj in &sf.objects {
            if skip.contains(&obj.class_id) { continue; }
            if let Ok(v) = read_object(&sf, obj) { all.insert(obj.path_id, (obj.class_id, v)); }
        }
        for (cid, v) in all.values() {
            if *cid != 74 { continue; }
            let n = v.get("m_Name").and_then(Value::as_str).unwrap_or("");
            if !n.to_lowercase().contains(&want) { continue; }
            println!("=== clip '{n}'");
            if let Some(o) = v.as_object() {
                for (k, vv) in o {
                    let d = match vv {
                        Value::Array(a) => format!("[{} items]", a.len()),
                        Value::Object(oo) => format!("{{{}}}", oo.keys().cloned().collect::<Vec<_>>().join(",")),
                        other => other.to_string(),
                    };
                    println!("   {k} = {}", &d[..d.len().min(220)]);
                }
            }
            // max keyframe time across float/position curves
            let mut maxt = f64::NAN;
            for key in ["m_FloatCurves", "m_PositionCurves", "m_RotationCurves", "m_ScaleCurves", "m_EulerCurves"] {
                if let Some(arr) = v.get(key).and_then(Value::as_array) {
                    for c in arr {
                        if let Some(ks) = c.get("curve").and_then(|cc| cc.get("m_Curve")).and_then(Value::as_array) {
                            for k in ks {
                                if let Some(t) = k.get("time").and_then(Value::as_f64) { if !(maxt >= t) { maxt = t; } }
                            }
                        }
                    }
                }
            }
            println!("   >>> max keyframe time across curves = {maxt:.4}");
            return;
        }
    }
}
