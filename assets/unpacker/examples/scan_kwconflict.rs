//! THROWAWAY: materials whose shader KEYWORD says a feature is off while still BINDING the
//! texture that feature would sample — i.e. content we would apply and the game would not.
//! Usage: scan_kwconflict <KEYWORD> <_TexProp> <bundle.ab> ...
use serde_json::Value;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn main() {
    let mut a = std::env::args().skip(1);
    let kw = a.next().expect("keyword");
    let tex = a.next().expect("tex prop");
    let (mut kw_on, mut bound, mut both) = (0usize, 0usize, 0usize);
    let mut names: Vec<String> = Vec::new();
    for path in a {
        let (k, b, t, n) = {
            let (kw, tex) = (kw.clone(), tex.clone());
            let p = path.clone();
            std::panic::catch_unwind(move || scan(&p, &kw, &tex)).unwrap_or_default()
        };
        kw_on += k; bound += b; both += t;
        names.extend(n);
    }
    println!("keyword present            : {kw_on}");
    println!("{tex} bound                : {bound}");
    println!("BOTH (conflict)            : {both}");
    names.sort();
    names.dedup();
    for n in names.iter().take(20) { println!("    {n}"); }
}

fn scan(path: &str, kw: &str, tex: &str) -> (usize, usize, usize, Vec<String>) {
    let (mut k, mut b, mut t) = (0, 0, 0);
    let mut names = Vec::new();
    let Ok(data) = std::fs::read(path) else { return (0, 0, 0, names) };
    let Ok(bundle) = BundleFile::parse(data) else { return (0, 0, 0, names) };
    let base = path.rsplit('/').next().unwrap_or(path).trim_end_matches(".ab").to_string();
    for e in &bundle.files {
        let Ok(sf) = SerializedFile::parse(e.data.to_vec()) else { continue };
        for o in &sf.objects {
            if o.class_id != 21 { continue }
            let Ok(v) = read_object(&sf, o) else { continue };
            let has_kw = v.get("m_ValidKeywords").and_then(Value::as_array)
                .is_some_and(|a| a.iter().any(|x| x.as_str() == Some(kw)));
            let has_tex = v.get("m_SavedProperties").and_then(|s| s.get("m_TexEnvs"))
                .and_then(|te| te.get(tex))
                .and_then(|e| e.get("m_Texture"))
                .and_then(|p| p.get("m_PathID")).and_then(Value::as_i64)
                .is_some_and(|p| p != 0);
            if has_kw { k += 1 }
            if has_tex { b += 1 }
            if has_kw && has_tex { t += 1; names.push(base.clone()); }
        }
    }
    (k, b, t, names)
}
