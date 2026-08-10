//! Diagnostic: for EVERY GameObject carrying a ParticleSystem, print its `m_Layer` together
//! with the name of its top-level ancestor (the entrance / idle / main root it belongs to).
//!
//! Motivation: Virtuosa's ten `wing_p` mesh rigs render a pink membrane the game shows nowhere,
//! and every authored input has been verified faithful — texture, UV cell, `_TintColor` at the
//! 0.502 half-neutral, the `Torappu/Particles-L2D/AlphaBlend` fragment program, `maxParticles`,
//! mesh geometry, renderer `m_Enabled`/`m_RenderMode`, director activation, and the rendered
//! alpha itself (1.07x the authored texture alpha, i.e. correct). The one field never read is
//! the GameObject's LAYER, which is what a Unity camera's culling mask selects on. If the
//! systems the game draws sit on one layer and the invisible ones on another, the culling mask
//! — not any particle property — is the discriminator.
//!
//! Usage: cargo run --release --example probe_pslayer -- <bundle.ab>

use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn main() {
    let path = std::env::args().nth(1).expect("usage: probe_pslayer <bundle.ab>");
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");

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
        // Transform pathID -> GameObject pathID, and GameObject -> its Transform.
        let mut tf_go: HashMap<i64, i64> = HashMap::new();
        for (p, (cid, v)) in &all {
            if *cid == 4 && let Some(g) = v.get("m_GameObject").and_then(pid) {
                tf_go.insert(*p, g);
            }
        }
        let name = |g: i64| -> String {
            all.get(&g)
                .and_then(|(_, v)| v.get("m_Name").and_then(Value::as_str))
                .unwrap_or("?")
                .to_string()
        };

        // Every GameObject that owns a ParticleSystem (classID 198).
        let mut ps_gos: Vec<i64> = all
            .values()
            .filter(|(cid, _)| *cid == 198)
            .filter_map(|(_, v)| v.get("m_GameObject").and_then(pid))
            .collect();
        ps_gos.sort_unstable();
        ps_gos.dedup();

        for g in ps_gos {
            let Some((_, gv)) = all.get(&g) else { continue };
            let layer = gv.get("m_Layer").and_then(Value::as_i64).unwrap_or(-1);
            // Walk to the top-level ancestor.
            let mut tf = gv.get("m_Component").and_then(Value::as_array).and_then(|cs| {
                cs.iter().find_map(|c| {
                    let q = c.get("component").unwrap_or(c);
                    let id = pid(q)?;
                    matches!(all.get(&id), Some((4, _))).then_some(id)
                })
            });
            let mut root = g;
            while let Some(t) = tf {
                let Some((_, tv)) = all.get(&t) else { break };
                let Some(pt) = tv.get("m_Father").and_then(pid).filter(|&x| x != 0) else { break };
                let Some(&pg) = tf_go.get(&pt) else { break };
                root = pg;
                tf = Some(pt);
            }
            println!("layer={layer:<4} root={:<44} go={}", name(root), name(g));
        }
    }
}
