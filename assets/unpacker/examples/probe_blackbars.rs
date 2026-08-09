//! THROWAWAY diagnostic: where are the `BG_black_*` / `Transition_black_*` planes, in world units?
//!
//! Motivation: Civilight Eterna's entrance renders into a hard 1920x1080 aperture inside a
//! 2340x1080 screen (cold-boot reproduced; simulating it moves her MADC 38.578 -> 33.076 at a
//! sharp 1920/2340 = 0.8205 optimum). Her prefab ships BG_black_01..04 — four black planes, which
//! is exactly the count a letterbox frame needs — and the exporter drops them on the
//! `_meshExtResolved` + no-window gate. If two of them sit left and right of a 16:9 window, the
//! pillarbox is AUTHORED GEOMETRY rather than an unexplained per-skin rule.
//!
//! Usage: cargo run --release --example probe_blackbars -- <bundle.ab>...
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn pid(v: &Value, k: &str) -> Option<i64> { v.get(k)?.get("m_PathID")?.as_i64().filter(|p| *p != 0) }
fn f3(v: &Value, k: &str, d: f64) -> [f64; 3] {
    let g = |a: &str| v.get(k).and_then(|x| x.get(a)).and_then(Value::as_f64).unwrap_or(d);
    [g("x"), g("y"), g("z")]
}

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else { continue };
        let Ok(bundle) = BundleFile::parse(data) else { continue };
        let short = path.rsplit('/').next().unwrap_or(&path).trim_end_matches(".ab").to_string();
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") { continue; }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
            let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
            for o in &sf.objects {
                if let Ok(v) = read_object(&sf, o) { all.insert(o.path_id, (o.class_id, v)); }
            }
            let mut go_to_tf: HashMap<i64, i64> = HashMap::new();
            let mut tf_father: HashMap<i64, i64> = HashMap::new();
            for (p, (cid, v)) in &all {
                if matches!(cid, 4 | 224) {
                    if let Some(g) = pid(v, "m_GameObject") { go_to_tf.insert(g, *p); }
                    if let Some(f) = pid(v, "m_Father") { tf_father.insert(*p, f); }
                }
            }
            let mut rows: Vec<String> = Vec::new();
            for (gpid, (cid, v)) in &all {
                if *cid != 1 { continue; }
                let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("");
                let l = name.to_ascii_lowercase();
                if !(l.contains("black") || l.contains("mask")) { continue; }
                let active = v.get("m_IsActive").and_then(Value::as_bool).unwrap_or(true);
                let Some(&tf) = go_to_tf.get(gpid) else { continue };
                // accumulate world position and scale up the chain
                let (mut wx, mut wy) = (0.0, 0.0);
                let (mut sx, mut sy) = (1.0, 1.0);
                let mut cur = Some(tf);
                let mut lp = [0.0; 3];
                let mut ls = [1.0; 3];
                let mut first = true;
                for _ in 0..64 {
                    let Some(t) = cur else { break };
                    let (_, tv) = &all[&t];
                    let p = f3(tv, "m_LocalPosition", 0.0);
                    let s = f3(tv, "m_LocalScale", 1.0);
                    if first { lp = p; ls = s; first = false; }
                    wx = p[0] + wx * s[0];
                    wy = p[1] + wy * s[1];
                    sx *= s[0];
                    sy *= s[1];
                    cur = tf_father.get(&t).copied().filter(|&f| f != 0);
                }
                // mesh AABB, so the plane's real extent (and therefore the window's inner edge)
                // is known rather than assumed from the scale
                let mut ext = [0.0f64, 0.0f64];
                let mut ctr = [0.0f64, 0.0f64];
                for (_, (c2, v2)) in &all {
                    if *c2 != 33 { continue; }
                    if pid(v2, "m_GameObject") != Some(*gpid) { continue; }
                    if let Some(mp) = pid(v2, "m_Mesh")
                        && let Some((_, mv)) = all.get(&mp)
                        && let Some(ab) = mv.get("m_LocalAABB")
                    {
                        let e = f3(ab, "m_Extent", 0.0);
                        let c = f3(ab, "m_Center", 0.0);
                        ext = [e[0], e[1]];
                        ctr = [c[0], c[1]];
                    }
                }
                // material colour + shader, so "is this an opaque BLACK surround?" is a data
                // question rather than a name match
                let mut matinfo = String::from("(no material)");
                for (_, (c2, v2)) in &all {
                    if *c2 != 23 { continue; }
                    if pid(v2, "m_GameObject") != Some(*gpid) { continue; }
                    if let Some(mp) = v2.get("m_Materials").and_then(Value::as_array).and_then(|a| a.first()).and_then(|m| m.get("m_PathID")).and_then(Value::as_i64)
                        && let Some((_, mv)) = all.get(&mp)
                    {
                        let sh = mv.get("m_Shader").and_then(|x| x.get("m_PathID")).and_then(Value::as_i64).unwrap_or(0);
                        let mut cols = String::new();
                        if let Some(arr) = mv.get("m_SavedProperties").and_then(|x| x.get("m_Colors")).and_then(Value::as_array) {
                            for e in arr {
                                let n = e.get("first").and_then(Value::as_str).unwrap_or("");
                                let g = |k: &str| e.get("second").and_then(|x| x.get(k)).and_then(Value::as_f64).unwrap_or(-1.0);
                                cols.push_str(&format!(" {}=({:.2},{:.2},{:.2},{:.2})", n, g("r"), g("g"), g("b"), g("a")));
                            }
                        }
                        matinfo = format!("shader={} colors:{}", sh, cols);
                    }
                }
                rows.push(format!("   {:26} {}", name, matinfo));
                let (hw, hh) = (ext[0] * sx, ext[1] * sy);
                let (cxw, cyw) = (wx + ctr[0] * sx, wy + ctr[1] * sy);
                rows.push(format!(
                    "   {:26} spans x[{:+9.2},{:+9.2}] y[{:+9.2},{:+9.2}]",
                    name, cxw - hw, cxw + hw, cyw - hh, cyw + hh
                ));
                rows.push(format!(
                    "   {:26} active={:<5} world=({:+8.3},{:+8.3}) worldScale=({:7.3},{:7.3}) local=({:+.3},{:+.3}) localScale=({:.3},{:.3})",
                    name, active, wx, wy, sx, sy, lp[0], lp[1], ls[0], ls[1]
                ));
            }
            if rows.is_empty() { continue; }
            rows.sort();
            println!("== {short}");
            for r in rows { println!("{r}"); }
        }
    }
}
