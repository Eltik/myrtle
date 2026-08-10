//! THROWAWAY diagnostic: what decides a dynchar entrance's RENDER APERTURE?
//!
//! Motivation: Iris' (`char_4134_cetsyr_epoque#50`) captured entrance renders into exactly
//! 1920x1080 centred in the 2340x1080 screen — symmetric 210px pillarbox, pure limited-range
//! black, snapping on at the instant the cinematic starts and never moving. Eyjafjalla the
//! Hvit Aska (`char_1016_agoat2_epoque#34`) is her twin on every camera field we export
//! (`aspect` 1.0, `cameraViewPx` 2048.0, `cameraSizePx` 1050) and captures FULL-FRAME on the
//! same emulator, same route, same encoder. So the aperture is decided by something the
//! exporter never reads.
//!
//! 1920/2340 = 0.8205, and 2048 x (1080/1152) = 1920 exactly -- i.e. a 16:9 (2048x1152)
//! target height-fit onto a 19.5:9 screen. Two candidates carry that:
//!   * a `Camera` (class 20) with a non-full `m_NormalizedViewPortRect` or a forced aspect,
//!   * a `RenderTexture` (class 84) asset with a fixed 16:9 size the entrance composites into.
//! This dumps both, plus any MonoBehaviour field whose name smells of aspect/viewport/rect,
//! so cet and eyja can be diffed directly.
//!
//! Usage: cargo run --release --example probe_viewport -- <bundle.ab> [<bundle.ab> ...]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

/// Walk a serde value and yield `(dotted.path, scalar)` for every leaf whose path mentions one
/// of `needles`. Keeps the probe generic -- we do not know the field names up front.
fn collect_matching(v: &Value, path: &str, needles: &[&str], out: &mut Vec<(String, String)>) {
    match v {
        Value::Object(m) => {
            for (k, sub) in m {
                let p = if path.is_empty() { k.clone() } else { format!("{path}.{k}") };
                collect_matching(sub, &p, needles, out);
            }
        }
        Value::Array(a) => {
            for (i, sub) in a.iter().enumerate() {
                collect_matching(sub, &format!("{path}[{i}]"), needles, out);
            }
        }
        scalar => {
            let lower = path.to_ascii_lowercase();
            if needles.iter().any(|n| lower.contains(n)) {
                out.push((path.to_string(), scalar.to_string()));
            }
        }
    }
}

fn go_name<'a>(all: &'a HashMap<i64, (i32, Value)>, v: &Value) -> &'a str {
    v.get("m_GameObject")
        .and_then(|g| g.get("m_PathID"))
        .and_then(Value::as_i64)
        .and_then(|go| all.get(&go))
        .and_then(|(_, gv)| gv.get("m_Name"))
        .and_then(Value::as_str)
        .unwrap_or("?")
}

fn main() {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    assert!(!paths.is_empty(), "usage: probe_viewport <bundle.ab> [...]");
    // Field-name needles. `aspect`/`viewport`/`rect` cover Unity's own camera fields; the rest
    // cover whatever HG named a letterbox helper, if one exists at all.
    let needles = ["aspect", "viewport", "letterbox", "pillar", "safearea", "rendertexture", "targettexture"];

    for path in &paths {
        println!("\n######## {path}");
        let Ok(data) = std::fs::read(path) else {
            println!("  (unreadable)");
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            println!("  (not a bundle)");
            continue;
        };
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
            let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
            for o in &sf.objects {
                if let Ok(v) = read_object(&sf, o) {
                    all.insert(o.path_id, (o.class_id, v));
                }
            }

            // --- Cameras (class 20) -------------------------------------------------------
            let mut cams: Vec<(i64, &Value)> =
                all.iter().filter(|(_, (cid, _))| *cid == 20).map(|(p, (_, v))| (*p, v)).collect();
            cams.sort_unstable_by_key(|(p, _)| *p);
            for (pid, v) in &cams {
                println!("  [Camera pid={pid}] go={:?} — ALL FIELDS:", go_name(&all, v));
                let mut fields = Vec::new();
                collect_matching(v, "", &[""], &mut fields); // empty needle matches every leaf
                fields.sort();
                for (k, val) in &fields {
                    println!("       {k} = {val}");
                }
            }

            // --- RenderTextures (class 84) ------------------------------------------------
            let mut rts: Vec<(i64, &Value)> =
                all.iter().filter(|(_, (cid, _))| *cid == 84).map(|(p, (_, v))| (*p, v)).collect();
            rts.sort_unstable_by_key(|(p, _)| *p);
            for (pid, v) in &rts {
                let w = v.get("m_Width").and_then(Value::as_i64).unwrap_or(-1);
                let h = v.get("m_Height").and_then(Value::as_i64).unwrap_or(-1);
                println!(
                    "  [RenderTexture pid={pid}] name={:?} {w}x{h} aspect={:.4}",
                    v.get("m_Name").and_then(Value::as_str).unwrap_or("?"),
                    w as f64 / h.max(1) as f64
                );
            }

            // --- Anything else named like an aperture control ------------------------------
            let mut hits: Vec<(i64, String, String, String)> = Vec::new();
            for (pid, (cid, v)) in &all {
                if *cid == 20 || *cid == 84 {
                    continue; // already dumped in full above
                }
                let mut found = Vec::new();
                collect_matching(v, "", &needles, &mut found);
                for (fp, val) in found {
                    hits.push((*pid, go_name(&all, v).to_string(), fp, val));
                }
            }
            hits.sort();
            for (pid, go, fp, val) in &hits {
                println!("  [pid={pid} go={go}] {fp} = {val}");
            }
        }
    }
}
