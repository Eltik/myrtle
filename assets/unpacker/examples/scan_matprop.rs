//! THROWAWAY: how many MATERIALS actually bind/set a named property, and to what?
//! Answers "is this shader property real content or serialized residue" without opening
//! materials one at a time.
//!
//! Usage: scan_matprop <_PropName> <bundle.ab> [bundle.ab ...]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn main() {
    let mut a = std::env::args().skip(1);
    let prop = a.next().expect("property name");
    let mut tex_bound = 0usize;
    let mut tex_null = 0usize;
    let mut floats: HashMap<String, usize> = HashMap::new();
    let mut mats = 0usize;
    for path in a {
        let (p, m, tb, tn, fl) = {
            let prop = prop.clone();
            let r = std::panic::catch_unwind(move || scan(&path, &prop));
            r.unwrap_or_default()
        };
        let _ = p;
        mats += m;
        tex_bound += tb;
        tex_null += tn;
        for (k, v) in fl {
            *floats.entry(k).or_default() += v;
        }
    }
    println!("materials scanned: {mats}");
    println!("  texture slot bound to a real texture : {tex_bound}");
    println!("  texture slot present but NULL        : {tex_null}");
    if !floats.is_empty() {
        let mut rows: Vec<_> = floats.into_iter().collect();
        rows.sort_by_key(|r| std::cmp::Reverse(r.1));
        println!("  scalar values:");
        for (k, v) in rows.into_iter().take(12) {
            println!("    {k:>12}  x{v}");
        }
    }
}

type Out = (usize, usize, usize, usize, HashMap<String, usize>);

fn scan(path: &str, prop: &str) -> Out {
    let mut floats: HashMap<String, usize> = HashMap::new();
    let (mut mats, mut tb, mut tn) = (0usize, 0usize, 0usize);
    let Ok(data) = std::fs::read(path) else {
        return (0, 0, 0, 0, floats);
    };
    let Ok(bundle) = BundleFile::parse(data) else {
        return (0, 0, 0, 0, floats);
    };
    for e in &bundle.files {
        let Ok(sf) = SerializedFile::parse(e.data.to_vec()) else {
            continue;
        };
        for o in &sf.objects {
            if o.class_id != 21 {
                continue;
            }
            let Ok(v) = read_object(&sf, o) else { continue };
            mats += 1;
            let sp = v.get("m_SavedProperties");
            if let Some(env) = sp
                .and_then(|s| s.get("m_TexEnvs"))
                .and_then(|t| t.get(prop))
            {
                let pid = env
                    .get("m_Texture")
                    .and_then(|t| t.get("m_PathID"))
                    .and_then(Value::as_i64)
                    .unwrap_or(0);
                if pid == 0 {
                    tn += 1
                } else {
                    tb += 1;
                    let nm = v.get("m_Name").and_then(Value::as_str).unwrap_or("?");
                    println!("    bound: mat='{nm}' texPathID={pid}");
                }
            }
            if let Some(f) = sp
                .and_then(|s| s.get("m_Floats"))
                .and_then(|f| f.get(prop))
                .and_then(Value::as_f64)
            {
                *floats.entry(format!("{f:.3}")).or_default() += 1;
            }
        }
    }
    (0, mats, tb, tn, floats)
}
