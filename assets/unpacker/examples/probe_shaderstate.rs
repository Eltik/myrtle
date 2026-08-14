//! THROWAWAY diagnostic: dump a Shader's per-pass BLEND STATE.
//!
//! Motivation: `Torappu/Particles-L2D/Additive`'s fragment sets
//! `SV_Target0.xyz = 2 * color.rgb * tint.rgb * tex.rgb` — it never multiplies RGB by alpha. Our
//! renderer premultiplies, so on a particle authored at `startColor.a = 0.11` we draw ~9x too
//! dim... but ONLY if the pass blends `One One`. If it blends `SrcAlpha One` the GPU applies the
//! alpha instead and our result is right. The GLSL cannot answer that; the pass state can.
//!
//! Usage: cargo run --release --example `probe_shaderstate` -- <bundle.ab> <pathID>
#![allow(clippy::or_fun_call)]

use serde_json::Value;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn walk(v: &Value, path: &str, out: &mut Vec<(String, String)>) {
    match v {
        Value::Object(m) => {
            for (k, vv) in m {
                let p = format!("{path}.{k}");
                let kl = k.to_ascii_lowercase();
                if kl.contains("blend")
                    || kl.contains("zwrite")
                    || kl.contains("colormask")
                    || kl.contains("srcfactor")
                    || kl.contains("dstfactor")
                {
                    out.push((p.clone(), format!("{vv}")));
                }
                walk(vv, &p, out);
            }
        }
        Value::Array(a) => {
            for (i, vv) in a.iter().enumerate().take(24) {
                walk(vv, &format!("{path}[{i}]"), out);
            }
        }
        _ => {}
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("bundle");
    let want: i64 = args
        .next()
        .expect("pathID")
        .parse()
        .expect("pathID must be an integer");
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    for entry in &bundle.files {
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        for obj in &sf.objects {
            if obj.path_id != want {
                continue;
            }
            let Ok(v) = read_object(&sf, obj) else {
                continue;
            };
            println!(
                "shader '{}'",
                v.get("m_ParsedForm")
                    .and_then(|p| p.get("m_Name"))
                    .and_then(Value::as_str)
                    .unwrap_or(v["m_Name"].as_str().unwrap_or("?"))
            );
            let mut out = Vec::new();
            walk(&v, "", &mut out);
            if out.is_empty() {
                println!("  (no blend-ish fields; dumping top-level keys)");
                if let Value::Object(m) = &v {
                    for k in m.keys() {
                        println!("    {k}");
                    }
                }
            }
            for (p, val) in out.iter().take(60) {
                let short = if val.len() > 200 {
                    format!("{}…", &val[..200])
                } else {
                    val.clone()
                };
                println!("  {p} = {short}");
            }
        }
    }
}
