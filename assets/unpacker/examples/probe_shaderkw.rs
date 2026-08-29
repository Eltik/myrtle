//! THROWAWAY: print the KEYWORD SET of every subprogram of a Shader, so a compiled variant
//! can be matched to the keywords a material actually sets.
//!
//! Motivation: `Torappu/Particles-L2D/Ram/Disturb(CustomData)` ships four vertex variants,
//! the 2x2 of {custom data on/off} x {UV rotation on/off}. whitw2's `sx (1)` sets
//! `_HGCUSTOMVERTEXSTREAM_ON` but NOT `_CUSTOMDATA_ON`, and which of those two gates the
//! custom-data variant decides whether our per-particle `vCustom` is right or is a fabrication.
//! Guessing it is not acceptable, so this reads the mapping.
//!
//! Usage: cargo run --release --example probe_shaderkw -- <shaders.ab> <pathID>
use serde_json::Value;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn strs(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn main() {
    let path = std::env::args().nth(1).expect("shaders.ab");
    let want: i64 = std::env::args()
        .nth(2)
        .expect("pathID")
        .parse()
        .expect("pathID must be an integer");
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
        for obj in &sf.objects {
            if obj.class_id != 48 || obj.path_id != want {
                continue;
            }
            let Ok(v) = read_object(&sf, obj) else {
                continue;
            };
            // Unity 2021+ keeps one shared keyword-name table on the Shader and indexes into
            // it per subprogram; older builds inline the names. Print whichever is present.
            let table = strs(v.get("m_ParsedForm").and_then(|p| p.get("m_KeywordNames")));
            if !table.is_empty() {
                println!("keyword table ({}): {table:?}", table.len());
            }
            let pf = v.get("m_ParsedForm");
            let subs = pf
                .and_then(|p| p.get("m_SubShaders"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for (si, sub) in subs.iter().enumerate() {
                let passes = sub
                    .get("m_Passes")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                for (pi, pass) in passes.iter().enumerate() {
                    for stage in ["progVertex", "progFragment"] {
                        let Some(prog) = pass.get(stage) else {
                            continue;
                        };
                        let sp = prog
                            .get("m_SubPrograms")
                            .and_then(Value::as_array)
                            .cloned()
                            .unwrap_or_default();
                        for (k, s) in sp.iter().enumerate() {
                            let idx: Vec<i64> = s
                                .get("m_KeywordIndices")
                                .and_then(Value::as_array)
                                .map(|a| a.iter().filter_map(Value::as_i64).collect())
                                .unwrap_or_default();
                            let named: Vec<&str> = idx
                                .iter()
                                .filter_map(|i| {
                                    table.get(usize::try_from(*i).ok()?).map(String::as_str)
                                })
                                .collect();
                            let inline = strs(s.get("m_GlobalKeywords"));
                            let local = strs(s.get("m_LocalKeywords"));
                            println!(
                                "  sub[{si}] pass[{pi}] {stage}[{k}] indices={idx:?} named={named:?} global={inline:?} local={local:?}"
                            );
                        }
                    }
                }
            }
        }
    }
}
