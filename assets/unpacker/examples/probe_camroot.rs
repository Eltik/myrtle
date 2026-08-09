//! THROWAWAY diagnostic: does the ENTRANCE camera's parent chain stop at a DIFFERENT spine root
//! than the one the scene quads are anchored to?
//!
//! Motivation: Muelsyse's entrance frame is off by a constant +300 authored px in Y and exactly 0
//! in X (measured: MADC 87.168 -> 23.699 at `?camdy=300`, sharp optimum, walls ±10px ≈ +10 MADC).
//! `entrance_camera_track` walks camera→up and BREAKS at the first GameObject holding a
//! SkeletonAnimation, claiming that "matches the quad exporter's stop set so camera + scene share
//! one frame". But an entrance bundle ships TWO skeleton prefab roots (`_Start` and idle), and the
//! quads are anchored to `own_root` specifically. If the camera hangs under the other one, the two
//! frames differ by exactly the offset between the roots.
//!
//! Usage: cargo run --release --example probe_camroot -- <bundle.ab>
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn pid(v: &Value, k: &str) -> Option<i64> {
    v.get(k)?.get("m_PathID")?.as_i64().filter(|p| *p != 0)
}
fn f3(v: &Value, k: &str) -> [f64; 3] {
    let g = |a: &str| v.get(k).and_then(|x| x.get(a)).and_then(Value::as_f64).unwrap_or(0.0);
    [g("x"), g("y"), g("z")]
}

fn main() {
    let path = std::env::args().nth(1).expect("usage: probe_camroot <bundle.ab>");
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") { continue; }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for o in &sf.objects {
            if let Ok(v) = read_object(&sf, o) { all.insert(o.path_id, (o.class_id, v)); }
        }
        let mut go_to_tf: HashMap<i64, i64> = HashMap::new();
        let mut tf_go: HashMap<i64, i64> = HashMap::new();
        let mut tf_father: HashMap<i64, i64> = HashMap::new();
        let mut cams: Vec<(i64, i64)> = Vec::new();
        for (p, (cid, v)) in &all {
            match cid {
                4 | 224 => {
                    if let Some(go) = pid(v, "m_GameObject") { go_to_tf.insert(go, *p); tf_go.insert(*p, go); }
                    if let Some(f) = pid(v, "m_Father") { tf_father.insert(*p, f); }
                }
                20 => { if let Some(g) = pid(v, "m_GameObject") { cams.push((*p, g)); } }
                _ => {}
            }
        }
        let name = |go: i64| -> String {
            all.get(&go).and_then(|(_, v)| v.get("m_Name")).and_then(Value::as_str).unwrap_or("?").to_string()
        };
        // Every GameObject holding a SkeletonAnimation (the walk's stop set).
        let spine_gos: HashSet<i64> = all.values()
            .filter(|(cid, v)| *cid == 114 && v.get("skeletonDataAsset").is_some())
            .filter_map(|(_, v)| pid(v, "m_GameObject")).collect();
        if spine_gos.is_empty() || cams.is_empty() { continue; }
        cams.sort();
        println!("== {}", entry.path);
        println!("   CAMERAS ({}): {}", cams.len(), cams.iter().map(|(cp,g)| format!("{} (cam pid {}, go {})", name(*g), cp, g)).collect::<Vec<_>>().join(", "));
        println!("   skeleton GOs: {}", spine_gos.iter().map(|g| format!("{} ({})", name(*g), g)).collect::<Vec<_>>().join(", "));
        for (campid, camgo) in &cams {
        println!("   --- camera pid {} ({}) ---", campid, name(*camgo));
        // Walk camera -> up, exactly as entrance_camera_track does.
        let mut cur = go_to_tf.get(camgo).copied();
        let mut chain: Vec<i64> = Vec::new();
        let mut stopped_at: Option<i64> = None;
        for _ in 0..64 {
            let Some(tf) = cur else { break };
            if let Some(go) = tf_go.get(&tf) && spine_gos.contains(go) { stopped_at = Some(*go); break; }
            chain.push(tf);
            cur = tf_father.get(&tf).copied().filter(|&f| f != 0);
        }
        for t in &chain {
            let go = tf_go.get(t).copied().unwrap_or(0);
            let (_, v) = &all[t];
            let lp = f3(v, "m_LocalPosition");
            let ls = f3(v, "m_LocalScale");
            let q = {
                let g = |k: &str, d: f64| v.get("m_LocalRotation").and_then(|x| x.get(k)).and_then(Value::as_f64).unwrap_or(d);
                [g("x", 0.0), g("y", 0.0), g("z", 0.0), g("w", 1.0)]
            };
            println!("     {:34} pos=({:+.4},{:+.4},{:+.4}) rot=({:+.4},{:+.4},{:+.4},{:+.4}) scale=({:+.4},{:+.4},{:+.4})",
                name(go), lp[0], lp[1], lp[2], q[0], q[1], q[2], q[3], ls[0], ls[1], ls[2]);
        }
        match stopped_at {
            Some(go) => println!("   STOPPED AT skeleton root: {} ({})", name(go), go),
            None => println!("   STOPPED AT: (ran off the top — no skeleton ancestor)"),
        }
        // Accumulated camera BASIS at the static pose. The exporter projects the camera position
        // onto these axes to get the frame centre; that is only equivalent to reading the world
        // XY when the basis is axis-aligned. A rolled rig folds the roll into the centre.
        {
            let mut m = [[1.0f64, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
            for tf in chain.iter().rev() {
                let (_, v) = &all[tf];
                let g = |k: &str, d: f64| v.get("m_LocalRotation").and_then(|x| x.get(k)).and_then(Value::as_f64).unwrap_or(d);
                let (x, y, z, w) = (g("x", 0.0), g("y", 0.0), g("z", 0.0), g("w", 1.0));
                let r = [[1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y - z * w), 2.0 * (x * z + y * w)],
                         [2.0 * (x * y + z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z - x * w)],
                         [2.0 * (x * z - y * w), 2.0 * (y * z + x * w), 1.0 - 2.0 * (x * x + y * y)]];
                let mut o = [[0.0f64; 3]; 3];
                for i in 0..3 { for j in 0..3 { for k in 0..3 { o[i][j] += m[i][k] * r[k][j]; } } }
                m = o;
            }
            let nrm = |c: usize| {
                let v = [m[0][c], m[1][c], m[2][c]];
                let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-9);
                [v[0] / n, v[1] / n, v[2] / n]
            };
            let r = nrm(0); let u = nrm(1);
            let roll = r[1].atan2(r[0]).to_degrees();
            println!("   BASIS right=({:+.4},{:+.4},{:+.4}) up=({:+.4},{:+.4},{:+.4})  ROLL={:+.3} deg  axis-aligned={}",
                r[0], r[1], r[2], u[0], u[1], u[2], roll, if roll.abs() < 0.5 { "YES" } else { "** NO **" });
        }
        }
        // World Y of each skeleton GO, by accumulating local positions to the top.
        for go in &spine_gos {
            let mut t = go_to_tf.get(go).copied();
            let (mut wx, mut wy) = (0.0f64, 0.0f64);
            let mut depth = 0;
            while let Some(tf) = t {
                if depth > 64 { break }
                let lp = all.get(&tf).map(|(_, v)| f3(v, "m_LocalPosition")).unwrap_or([0.0; 3]);
                wx += lp[0]; wy += lp[1];
                t = tf_father.get(&tf).copied().filter(|&f| f != 0);
                depth += 1;
            }
            // accumulate the SCALE chain too — a skeleton root scaled != 1 means the spine and the
            // scene quads are NOT in the same units, which `spine.scale.set(1)` assumes.
            let mut t2 = go_to_tf.get(go).copied();
            let (mut sx, mut sy) = (1.0f64, 1.0f64);
            let mut d2 = 0;
            while let Some(tf) = t2 {
                if d2 > 64 { break }
                let ls = all.get(&tf).map(|(_, v)| f3(v, "m_LocalScale")).unwrap_or([1.0; 3]);
                sx *= ls[0]; sy *= ls[1];
                t2 = tf_father.get(&tf).copied().filter(|&f| f != 0);
                d2 += 1;
            }
            println!("   root {:28} world=({:+.4},{:+.4})  [x100 px = ({:+.1},{:+.1})]  SCALE=({:.5},{:.5})", name(*go), wx, wy, wx * 100.0, wy * 100.0, sx, sy);
        }
    }
}
