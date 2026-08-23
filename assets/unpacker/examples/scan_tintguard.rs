//! THROWAWAY: joint distribution of the legacy-x2 tint gate inputs.
//! For every dynchar material: resolved shader name, whether _TintColor rgb is
//! EXACTLY 0.5 (the documented inert-residue signature), and whether _MainColor
//! is also present (i.e. the shader probably modulates by _MainColor instead).
//!
//! Usage: scan_tintguard <bundle.ab> [bundle.ab ...]
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
use unpacker::export::shader_map;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn col(mat: &Value, name: &str) -> Option<[f64; 4]> {
    let c = mat.get("m_SavedProperties")?.get("m_Colors")?.get(name)?;
    let g = |k: &str| c.get(k).and_then(Value::as_f64).unwrap_or(0.0);
    Some([g("r"), g("g"), g("b"), g("a")])
}

fn main() {
    let paths: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();
    // the shader bundles must be in scope for the map
    let mut all = paths.clone();
    for extra in ["[uc]shaders.ab", "[uc]uishaders.ab"] {
        if let Some(p) = paths.first().and_then(|p| p.parent()) {
            let c = p.join("..").join("..").join(extra);
            if c.exists() {
                all.push(c);
            }
            let c2 = p.join(extra);
            if c2.exists() {
                all.push(c2);
            }
        }
    }
    let smap = shader_map::build_shader_map(&all);
    eprintln!("shader map: {} entries", smap.len());

    // shader -> (total, tint05_and_maincolor, tint05_only, maincolor_only)
    let mut rows: BTreeMap<String, [usize; 4]> = BTreeMap::new();
    for path in &paths {
        let smap2 = smap.clone();
        let path2 = path.clone();
        let got = std::panic::catch_unwind(move || {
            let mut local: BTreeMap<String, [usize; 4]> = BTreeMap::new();
            scan_one(&path2, &smap2, &mut local);
            local
        });
        if let Ok(local) = got {
            for (k, v) in local {
                let e = rows.entry(k).or_default();
                for i in 0..4 {
                    e[i] += v[i];
                }
            }
        } else {
            eprintln!("PANIC on {}", path.display());
        }
    }
    println!(
        "{:<58} {:>6} {:>10} {:>10} {:>10}",
        "shader", "mats", "0.5+MC", "0.5only", "other+MC"
    );
    for (k, v) in rows {
        if v[1] == 0 && v[3] == 0 {
            continue;
        }
        println!("{k:<58} {:>6} {:>10} {:>10} {:>10}", v[0], v[1], v[2], v[3]);
    }
}

fn scan_one(path: &PathBuf, smap: &shader_map::ShaderMap, rows: &mut BTreeMap<String, [usize; 4]>) {
    {
        let Ok(data) = std::fs::read(path) else {
            return;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            return;
        };
        for e in &bundle.files {
            let Ok(sf) = SerializedFile::parse(e.data.to_vec()) else {
                continue;
            };
            for obj in &sf.objects {
                if obj.class_id != 21 {
                    continue;
                }
                let Ok(v) = read_object(&sf, obj) else {
                    continue;
                };
                let Some((fid, pid)) = v.get("m_Shader").and_then(|s| {
                    Some((s.get("m_FileID")?.as_i64()?, s.get("m_PathID")?.as_i64()?))
                }) else {
                    continue;
                };
                let Some(name) = shader_map::resolve_shader(&sf.externals, fid, pid, smap) else {
                    continue;
                };
                if !name.contains("Particles") {
                    continue;
                }
                if std::env::var("PERMAT").is_ok() {
                    let mn = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
                    if std::env::var("PERMAT")
                        .unwrap()
                        .split(',')
                        .any(|f| mn.contains(f))
                    {
                        println!(
                            "MAT {mn:<28} {name}\n    _TintColor={:?}\n    _MainColor={:?}",
                            col(&v, "_TintColor"),
                            col(&v, "_MainColor")
                        );
                    }
                }
                let t = col(&v, "_TintColor");
                let has_mc = col(&v, "_MainColor").is_some();
                let tint05 = t.is_some_and(|c| c[0] == 0.5 && c[1] == 0.5 && c[2] == 0.5);
                let e = rows.entry(name.to_string()).or_default();
                e[0] += 1;
                if t.is_some() {
                    match (tint05, has_mc) {
                        (true, true) => e[1] += 1,
                        (true, false) => e[2] += 1,
                        (false, true) => e[3] += 1,
                        _ => {}
                    }
                }
            }
        }
    }
}
