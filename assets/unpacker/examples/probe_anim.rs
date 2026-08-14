//! THROWAWAY: dump `AnimatorController` state timing (speed, cycleOffset, transition
//! duration/offset/exitTime) plus the `GameObject` hierarchy + `m_IsActive` for the Animator
//! hosts — to see how the game SEQUENCES several clips on one prefab.
//!
//! Usage: cargo run --release --example `probe_anim` -- <bundle.ab> [go-substr]
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::too_many_lines
)]
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};
fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}
fn nf(v: &Value, k: &str) -> f64 {
    v.get(k).and_then(Value::as_f64).unwrap_or(f64::NAN)
}

fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("bundle");
    let want = a.next().unwrap_or_default().to_lowercase();
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
        let skip: HashSet<i32> = [28, 43, 48, 49, 83, 128, 213].into_iter().collect();
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for obj in &sf.objects {
            if skip.contains(&obj.class_id) {
                continue;
            }
            if let Ok(v) = read_object(&sf, obj) {
                all.insert(obj.path_id, (obj.class_id, v));
            }
        }
        let mut gname: HashMap<i64, String> = HashMap::new();
        let mut gactive: HashMap<i64, i64> = HashMap::new();
        for (p, (cid, v)) in &all {
            if *cid == 1 {
                gname.insert(
                    *p,
                    v.get("m_Name").and_then(Value::as_str).unwrap_or("").into(),
                );
                gactive.insert(
                    *p,
                    v.get("m_IsActive").and_then(Value::as_i64).unwrap_or(-1),
                );
            }
        }
        for (cid, v) in all.values() {
            if *cid != 95 {
                continue;
            }
            let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
            let name = gname.get(&go).cloned().unwrap_or_default();
            if !want.is_empty() && !name.to_lowercase().contains(&want) {
                continue;
            }
            println!(
                "\n##### Animator on GO '{name}'  m_IsActive={}",
                gactive.get(&go).copied().unwrap_or(-1)
            );
            for k in [
                "m_Enabled",
                "m_Speed",
                "m_UpdateMode",
                "m_ApplyRootMotion",
                "m_HasTransformHierarchy",
            ] {
                if let Some(x) = v.get(k) {
                    println!("    {k} = {x}");
                }
            }
            let Some(cp) = v.get("m_Controller").and_then(pid) else {
                continue;
            };
            let Some((_, ctrl)) = all.get(&cp) else {
                println!("    controller {cp} EXTERNAL");
                continue;
            };
            let clips: Vec<String> = ctrl
                .get("m_AnimationClips")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(pid)
                        .map(|p| {
                            all.get(&p)
                                .and_then(|(_, cv)| cv.get("m_Name"))
                                .and_then(Value::as_str)
                                .unwrap_or("?")
                                .to_string()
                        })
                        .collect()
                })
                .unwrap_or_default();
            println!("    controller clips: {clips:?}");
            let Some(c) = ctrl.get("m_Controller") else {
                continue;
            };
            if let Some(sms) = c.get("m_StateMachineArray").and_then(Value::as_array) {
                for (si, sm) in sms.iter().enumerate() {
                    if let Some(states) = sm.get("m_StateConstantArray").and_then(Value::as_array) {
                        for (i, st) in states.iter().enumerate() {
                            println!(
                                "    sm{si} state{i}: speed={} cycleOffset={} mirror={:?} iKOnFeet={:?}",
                                nf(st, "m_Speed"),
                                nf(st, "m_CycleOffset"),
                                st.get("m_Mirror").and_then(Value::as_i64),
                                st.get("m_IKOnFeet").and_then(Value::as_i64)
                            );
                            if let Some(trs) = st
                                .get("m_TransitionConstantArray")
                                .and_then(Value::as_array)
                            {
                                for (ti, tr) in trs.iter().enumerate() {
                                    println!(
                                        "        transition{ti}: duration={} offset={} exitTime={} hasExitTime={:?} atomic={:?}",
                                        nf(tr, "m_TransitionDuration"),
                                        nf(tr, "m_TransitionOffset"),
                                        nf(tr, "m_ExitTime"),
                                        tr.get("m_HasExitTime").and_then(Value::as_i64),
                                        tr.get("m_Atomic").and_then(Value::as_i64)
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
