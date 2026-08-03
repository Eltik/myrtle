//! THROWAWAY: corpus impact of preferring `_TintColor` over `_Color` in
//! `material_color_props`. Counts materials carrying BOTH, split by whether the swap would
//! actually change the exported tint.
use serde_json::Value;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};
fn rgba(c: &Value) -> [f64; 4] {
    let g = |k: &str| c.get(k).and_then(Value::as_f64).unwrap_or(1.0);
    [g("r"), g("g"), g("b"), g("a")]
}
fn main() {
    let (mut mats, mut both, mut changes, mut white_tint, mut nonwhite_color) = (0u32, 0u32, 0u32, 0u32, 0u32);
    let mut worst: Vec<(String, String, [f64; 4])> = Vec::new();
    for path in std::env::args().skip(1) {
        let Ok(d) = std::fs::read(&path) else { continue };
        let Ok(b) = BundleFile::parse(d) else { continue };
        let skin = path.rsplit('/').next().unwrap_or("?").to_string();
        for e in &b.files {
            let l = e.path.to_ascii_lowercase();
            if l.ends_with(".ress") || l.ends_with(".resource") { continue; }
            let Ok(sf) = SerializedFile::parse(e.data.clone()) else { continue };
            for o in sf.objects.iter().filter(|o| o.class_id == 21) {
                let Ok(v) = read_object(&sf, o) else { continue };
                let Some(cols) = v.get("m_SavedProperties").and_then(|s| s.get("m_Colors")).and_then(Value::as_object) else { continue };
                mats += 1;
                let (c, t) = (cols.get("_Color"), cols.get("_TintColor"));
                let (Some(c), Some(t)) = (c, t) else { continue };
                both += 1;
                let (cv, tv) = (rgba(c), rgba(t));
                let cw = cv.iter().all(|x| (x - 1.0).abs() < 1e-3);
                let tw = tv.iter().all(|x| (x - 1.0).abs() < 1e-3);
                if !cw { nonwhite_color += 1; }
                if tw { white_tint += 1; }
                if cv.iter().zip(tv.iter()).any(|(a, b)| (a - b).abs() > 1.5 / 255.0) {
                    changes += 1;
                    if worst.len() < 10 {
                        let n = v.get("m_Name").and_then(Value::as_str).unwrap_or("?").to_string();
                        worst.push((skin.clone(), n, tv));
                    }
                }
            }
        }
    }
    println!("materials with any m_Colors : {mats}");
    println!("  carrying BOTH _Color and _TintColor : {both}");
    println!("  ...where the two DIFFER (swap changes the exported tint) : {changes}");
    println!("  ...of those, _Color is non-white already : {nonwhite_color}");
    println!("  ...where _TintColor is pure white (swap would LOSE a real _Color) : {white_tint}");
    println!("\nexamples of materials whose tint would change:");
    for (s, n, t) in worst { println!("  {s:<38} {n:<26} _TintColor [{:.3},{:.3},{:.3},{:.3}]", t[0], t[1], t[2], t[3]); }
}
