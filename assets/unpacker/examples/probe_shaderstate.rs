//! THROWAWAY diagnostic: dump a Shader's SERIALIZED PASS STATE (name, tags, blend factors,
//! ZWrite/Cull, and the declared properties) straight out of `[uc]shaders.ab`.
//!
//! Motivation: Virtuosa's ten wing particle rigs draw a saturated PINK wisp where the game shows
//! essentially nothing. Every authored input has been verified correct — pink source texture,
//! neutral `_TintColor`, `maxParticles` respected, director-activated, right UV cell, no vertex
//! colour on the mesh. The only component still taken on trust is the shader itself,
//! `Torappu/Particles-L2D/AlphaBlend`, whose name we assume means `tex * color` over the frame.
//! Unity's SerializedShader keeps per-pass fixed-function state, so the real blend equation and
//! render queue can be read without decompiling any bytecode.
//!
//! Usage: cargo run --release --example probe_shaderstate -- <shaders.ab> [pathID|name-substr]

use serde_json::Value;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

/// Walk an arbitrary JSON subtree and print any key that looks like render state.
fn dump_state(v: &Value, depth: usize, out: &mut Vec<String>) {
    if depth > 8 {
        return;
    }
    match v {
        Value::Object(m) => {
            for (k, val) in m {
                let kl = k.to_ascii_lowercase();
                if kl.contains("blend")
                    || kl.contains("zwrite")
                    || kl.contains("ztest")
                    || kl.contains("cull")
                    || kl.contains("queue")
                    || kl.contains("rendertype")
                    || kl == "m_tags"
                    || kl == "m_name"
                    || kl == "m_lod"
                {
                    let s = serde_json::to_string(val).unwrap_or_default();
                    if s.len() < 400 {
                        out.push(format!("{:indent$}{k} = {s}", "", indent = depth * 2));
                    }
                }
                dump_state(val, depth + 1, out);
            }
        }
        Value::Array(a) => {
            for x in a.iter().take(16) {
                dump_state(x, depth + 1, out);
            }
        }
        _ => {}
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("shaders.ab");
    let want = std::env::args().nth(2).unwrap_or_default();
    let want_pid: Option<i64> = want.parse().ok();
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");

    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        for obj in &sf.objects {
            if obj.class_id != 48 {
                continue;
            }
            let Ok(v) = read_object(&sf, obj) else { continue };
            let name = v
                .get("m_ParsedForm")
                .and_then(|p| p.get("m_Name"))
                .and_then(Value::as_str)
                .or_else(|| v.get("m_Name").and_then(Value::as_str))
                .unwrap_or("?");
            let hit = match want_pid {
                Some(p) => obj.path_id == p,
                None => want.is_empty() || name.to_ascii_lowercase().contains(&want.to_ascii_lowercase()),
            };
            if !hit {
                continue;
            }
            println!("\n===== SHADER '{name}'  pathID={}", obj.path_id);
            println!("  top-level keys: {:?}", v.as_object().map(|m| m.keys().collect::<Vec<_>>()));
            let mut out = Vec::new();
            dump_state(&v, 0, &mut out);
            for line in out.iter().take(120) {
                println!("  {line}");
            }
        }
    }
}
