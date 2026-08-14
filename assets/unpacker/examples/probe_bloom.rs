//! THROWAWAY diagnostic: does a dynchar prefab attach a BLOOM / post-effect `MonoBehaviour` to its
//! camera, and with what parameters?
//!
//! Motivation: the client carries a `MobileBloom` / `PostEffectBase` stack (IL2CPP dump), and the
//! game shows a broad ground GLOW at Mlynar's t=13 that our render is missing 72% of. Our own
//! bloom is effectively inert (threshold 1.0). If the prefab attaches a bloom with authored
//! parameters, that is a real missing mechanism — and it lives in the DATA, not the code.
//!
//! Usage: cargo run --release --example `probe_bloom` -- <bundle.ab>
use serde_json::Value;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn walk(v: &Value, path: &str, out: &mut Vec<(String, String)>) {
    match v {
        Value::Object(m) => {
            for (k, vv) in m {
                let lk = k.to_ascii_lowercase();
                if lk.contains("bloom")
                    || lk.contains("threshold")
                    || lk.contains("intensity")
                    || lk.contains("glow")
                    || lk.contains("blur")
                    || lk.contains("hdr")
                    || lk.contains("exposure")
                    || lk.contains("postprocess")
                    || lk.contains("softknee")
                    || lk.contains("diffusion")
                    || lk.contains("clamp")
                {
                    out.push((
                        format!("{path}/{k}"),
                        format!("{vv}").chars().take(90).collect(),
                    ));
                }
                walk(vv, &format!("{path}/{k}"), out);
            }
        }
        Value::Array(a) => {
            for (i, vv) in a.iter().enumerate().take(64) {
                walk(vv, &format!("{path}[{i}]"), out);
            }
        }
        _ => {}
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle.ab");
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    let mut cameras = 0usize;
    let mut mono = 0usize;
    for entry in &bundle.files {
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        for obj in &sf.objects {
            // 20 = Camera, 114 = MonoBehaviour
            if obj.class_id != 20 && obj.class_id != 114 {
                continue;
            }
            let Some(Ok(v)) =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| read_object(&sf, obj)))
                    .ok()
            else {
                continue;
            };
            if obj.class_id == 20 {
                cameras += 1;
                println!(
                    "  CAMERA pid={} ortho={:?} size={:?} depth={:?} clearFlags={:?} cullingMask={:?}",
                    obj.path_id,
                    v.get("orthographic"),
                    v.get("orthographic size"),
                    v.get("m_Depth"),
                    v.get("m_ClearFlags"),
                    v.get("m_CullingMask"),
                );
                continue;
            }
            mono += 1;
            let mut hits = Vec::new();
            walk(&v, "", &mut hits);
            if !hits.is_empty() {
                println!(
                    "  MONO pid={} enabled={:?}",
                    obj.path_id,
                    v.get("m_Enabled")
                );
                for (k, val) in hits.iter().take(12) {
                    println!("      {k} = {val}");
                }
            }
        }
    }
    println!("  ({cameras} cameras, {mono} MonoBehaviours scanned)");
}
