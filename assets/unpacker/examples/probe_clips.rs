//! THROWAWAY diagnostic: every AnimationClip / Animator / Animation on a dynchar prefab —
//! name, sample rate, stop time, wrap mode, speed — to see how the game advances the camera
//! rig relative to the (shorter) spine clip.
//!
//! Usage: cargo run --release --example probe_clips -- <bundle.ab> [name-substr]
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn f(v: &Value, k: &str) -> Option<f64> { v.get(k).and_then(Value::as_f64) }
fn pid(v: &Value) -> Option<i64> { v.get("m_PathID").and_then(Value::as_i64) }

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
        let mut gname: HashMap<i64, String> = HashMap::new();
        for (p, (cid, v)) in &all { if *cid == 1 { gname.insert(*p, v.get("m_Name").and_then(Value::as_str).unwrap_or("").into()); } }

        println!("\n--- AnimationClips (class 74)");
        for (p, (cid, v)) in &all {
            if *cid != 74 { continue; }
            let n = v.get("m_Name").and_then(Value::as_str).unwrap_or("");
            if !want.is_empty() && !n.to_lowercase().contains(&want) { continue; }
            let sr = f(v, "m_SampleRate").unwrap_or(f64::NAN);
            let mc = v.get("m_MuscleClip");
            let start = mc.and_then(|s| f(s, "m_StartTime")).unwrap_or(f64::NAN);
            let stop = mc.and_then(|s| f(s, "m_StopTime")).unwrap_or(f64::NAN);
            let loopt = mc.and_then(|s| s.get("m_LoopTime")).and_then(Value::as_i64).unwrap_or(-1);
            let legacy = v.get("m_Legacy").and_then(Value::as_i64).unwrap_or(-1);
            let wrap = v.get("m_WrapMode").and_then(Value::as_i64).unwrap_or(-1);
            println!("  pid={p} '{n}'  sampleRate={sr}  start={start:.4} stop={stop:.4} (frames={:.1})  loopTime={loopt} legacy={legacy} wrapMode={wrap}", (stop - start) * sr);
        }
        println!("\n--- Animator (95) / Animation (111) components");
        for (cid, v) in all.values() {
            if *cid != 95 && *cid != 111 { continue; }
            let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
            let n = gname.get(&go).cloned().unwrap_or_default();
            if !want.is_empty() && !n.to_lowercase().contains(&want) { continue; }
            let speed = f(v, "m_Speed").unwrap_or(f64::NAN);
            let ctrl = v.get("m_Controller").and_then(pid).unwrap_or(0);
            let playauto = v.get("m_PlayAutomatically").and_then(Value::as_i64).unwrap_or(-1);
            let wrap = v.get("m_WrapMode").and_then(Value::as_i64).unwrap_or(-1);
            let clip = v.get("m_Animation").and_then(pid).unwrap_or(0);
            println!("  class {cid} on GO '{n}'  speed={speed}  controller={ctrl}  playAuto={playauto} wrapMode={wrap} defaultClip={clip}");
            if let Some(arr) = v.get("m_Animations").and_then(Value::as_array) {
                for c in arr {
                    if let Some(cp) = pid(c) {
                        let nm = all.get(&cp).and_then(|(_, cv)| cv.get("m_Name")).and_then(Value::as_str).unwrap_or("?");
                        println!("      clip pid={cp} '{nm}'");
                    }
                }
            }
        }
    }
}
