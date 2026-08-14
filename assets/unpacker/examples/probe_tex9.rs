//! THROWAWAY: identify which source `Texture2D` became a given exported scene texture, by
//! DECODING each candidate and matching its pixels — not by assuming export ordering — then
//! report every Material that binds it, with its tint and shader.
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::export::texture::decode_texture_object;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn stats(rgba: &[u8]) -> (f64, f64, [f64; 3]) {
    let n = rgba.len() / 4;
    let mut amin = 255.0f64;
    let mut asum = 0.0;
    let mut lit = [0.0f64; 3];
    let mut litn = 0.0;
    for i in 0..n {
        let a = f64::from(rgba[i * 4 + 3]);
        amin = amin.min(a);
        asum += a;
        if a > 13.0 {
            for c in 0..3 {
                lit[c] += f64::from(rgba[i * 4 + c]);
            }
            litn += 1.0;
        }
    }
    let l = if litn > 0.0 {
        [lit[0] / litn, lit[1] / litn, lit[2] / litn]
    } else {
        [0.0; 3]
    };
    (amin, asum / n as f64, l)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (path, tw, th) = (
        &args[0],
        args[1].parse::<i64>().unwrap(),
        args[2].parse::<i64>().unwrap(),
    );
    let (want_amin, want_amean) = (
        args[3].parse::<f64>().unwrap(),
        args[4].parse::<f64>().unwrap(),
    );
    let Ok(data) = std::fs::read(path) else {
        return;
    };
    let Ok(bundle) = BundleFile::parse(data) else {
        return;
    };
    // Texture pixels live in the .resS stream companion; decode_texture_object needs it.
    let mut res: HashMap<String, Vec<u8>> = HashMap::new();
    for e in &bundle.files {
        let l = e.path.to_ascii_lowercase();
        if l.ends_with(".ress") || l.ends_with(".resource") {
            let base = e.path.rsplit('/').next().unwrap_or(&e.path).to_string();
            res.insert(e.path.clone(), e.data.clone());
            res.insert(base, e.data.clone());
        }
    }
    for entry in &bundle.files {
        let l = entry.path.to_ascii_lowercase();
        if l.ends_with(".ress") || l.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for o in &sf.objects {
            if (o.class_id == 21 || o.class_id == 28)
                && let Ok(v) = read_object(&sf, o)
            {
                all.insert(o.path_id, (o.class_id, v));
            }
        }
        let mut hit: Option<(i64, String)> = None;
        for (pid, (cid, v)) in &all {
            if *cid != 28 {
                continue;
            }
            if v.get("m_Width").and_then(Value::as_i64) != Some(tw)
                || v.get("m_Height").and_then(Value::as_i64) != Some(th)
            {
                continue;
            }
            let name = v
                .get("m_Name")
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_string();
            let Ok(Some(d)) = decode_texture_object(v, &res) else {
                println!("  {name:<28} (decode failed)");
                continue;
            };
            let (amin, amean, lit) = stats(&d.rgba);
            let m = (amin - want_amin).abs() <= 2.0 && (amean - want_amean).abs() <= 3.0;
            println!(
                "  {name:<28} alphaMin {amin:5.1} alphaMean {amean:6.1} litRGB [{:5.1},{:5.1},{:5.1}] {}",
                lit[0],
                lit[1],
                lit[2],
                if m { "<== MATCH" } else { "" }
            );
            if m {
                hit = Some((*pid, name));
            }
        }
        let Some((tpid, tname)) = hit else {
            println!("no match");
            return;
        };
        println!("\nMaterials binding '{tname}':");
        for (cid, v) in all.values() {
            if *cid != 21 {
                continue;
            }
            let Some(te) = v
                .get("m_SavedProperties")
                .and_then(|s| s.get("m_TexEnvs"))
                .and_then(Value::as_object)
            else {
                continue;
            };
            let binds: Vec<&String> = te
                .iter()
                .filter(|(_, x)| {
                    x.get("m_Texture")
                        .and_then(|p| p.get("m_PathID"))
                        .and_then(Value::as_i64)
                        == Some(tpid)
                })
                .map(|(k, _)| k)
                .collect();
            if binds.is_empty() {
                continue;
            }
            let name = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
            let col = |k: &str| {
                v.get("m_SavedProperties")
                    .and_then(|s| s.get("m_Colors"))
                    .and_then(|c| c.get(k))
                    .map_or_else(
                        || "-".into(),
                        |c| {
                            let g = |n: &str| c.get(n).and_then(Value::as_f64).unwrap_or(f64::NAN);
                            format!("[{:.3},{:.3},{:.3},{:.3}]", g("r"), g("g"), g("b"), g("a"))
                        },
                    )
            };
            println!("  MAT '{name}'  binds as {binds:?}");
            println!(
                "      _TintColor={}  _MainColor={}  _Color={}",
                col("_TintColor"),
                col("_MainColor"),
                col("_Color")
            );
            if let Some(o) = v
                .get("m_SavedProperties")
                .and_then(|s| s.get("m_Colors"))
                .and_then(Value::as_object)
            {
                let mut ks: Vec<&String> = o.keys().collect();
                ks.sort();
                println!("      ALL m_Colors keys: {ks:?}");
            }
            println!(
                "      shaderName={:?}  shaderRef={:?}",
                v.get("_shaderName"),
                v.get("m_Shader")
            );
        }
    }
}
