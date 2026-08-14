//! THROWAWAY diagnostic: every `ParticleSystem` on a dynchar prefab with the facts the
//! exporter's skip gates use, so a MISSING effect can be matched to the rule that dropped it.
//!
//! Usage: cargo run --release --example `probe_skipped` -- <bundle.ab> [root-substr]
#![allow(
    clippy::assigning_clones,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::or_fun_call,
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
fn col(v: &Value, k: &str) -> String {
    v.get(k)
        .and_then(|c| c.get("maxColor"))
        .map_or("-".into(), |m| {
            let f = |n: &str| m.get(n).and_then(Value::as_f64).unwrap_or(f64::NAN);
            format!("({:.2},{:.2},{:.2},{:.2})", f("r"), f("g"), f("b"), f("a"))
        })
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
        let mut gtf: HashMap<i64, i64> = HashMap::new();
        let mut grend: HashMap<i64, i64> = HashMap::new();
        for (p, (cid, v)) in &all {
            match cid {
                1 => {
                    gname.insert(
                        *p,
                        v.get("m_Name").and_then(Value::as_str).unwrap_or("").into(),
                    );
                    gactive.insert(
                        *p,
                        v.get("m_IsActive").and_then(Value::as_i64).unwrap_or(-1),
                    );
                    if let Some(cs) = v.get("m_Component").and_then(Value::as_array) {
                        for c in cs {
                            if let Some(cp) = c
                                .get("component")
                                .and_then(pid)
                                .or_else(|| c.get("second").and_then(pid))
                                && let Some((4, _)) = all.get(&cp)
                            {
                                gtf.insert(*p, cp);
                            }
                        }
                    }
                }
                199 => {
                    if let Some(go) = v.get("m_GameObject").and_then(pid) {
                        grend.insert(go, v.get("m_Enabled").and_then(Value::as_i64).unwrap_or(1));
                    }
                }
                _ => {}
            }
        }
        for (cid, v) in all.values() {
            if *cid != 198 {
                continue;
            }
            let Some(go) = v.get("m_GameObject").and_then(pid) else {
                continue;
            };
            let name = gname.get(&go).cloned().unwrap_or_default();
            // ancestor chain + any inactive ancestor
            let mut chain = Vec::new();
            let mut inactive_anc = String::new();
            let mut t = gtf.get(&go).copied();
            let mut depth = 0;
            while let Some(tt) = t {
                let Some(f) = all
                    .get(&tt)
                    .and_then(|(_, x)| x.get("m_Father").and_then(pid))
                    .filter(|p| *p != 0)
                else {
                    break;
                };
                if let Some(g) = all
                    .get(&f)
                    .and_then(|(_, x)| x.get("m_GameObject").and_then(pid))
                {
                    let n = gname.get(&g).cloned().unwrap_or_default();
                    if gactive.get(&g).copied().unwrap_or(1) == 0 && inactive_anc.is_empty() {
                        inactive_anc = n.clone();
                    }
                    chain.push(n);
                }
                t = Some(f);
                depth += 1;
                if depth > 8 {
                    break;
                }
            }
            if !want.is_empty()
                && !chain.iter().any(|c| c.to_lowercase().contains(&want))
                && !name.to_lowercase().contains(&want)
            {
                continue;
            }
            let init = v.get("InitialModule");
            let sc = init.map_or("-".into(), |i| col(i, "startColor"));
            let em_on = v
                .get("EmissionModule")
                .and_then(|e| e.get("enabled"))
                .and_then(Value::as_i64)
                .unwrap_or(-1);
            let rend_on = grend.get(&go).copied().unwrap_or(-1);
            let go_active = gactive.get(&go).copied().unwrap_or(-1);
            let rm = v
                .get("ParticleSystemRenderer")
                .and_then(|r| r.get("m_RenderMode"))
                .and_then(Value::as_i64)
                .unwrap_or(-1);
            println!(
                "{name:<28} active={go_active} rendEnabled={rend_on} emission={em_on} rm={rm} startColorMax={sc} inactiveAncestor={}",
                if inactive_anc.is_empty() {
                    "-".into()
                } else {
                    inactive_anc
                }
            );
        }
    }
}
