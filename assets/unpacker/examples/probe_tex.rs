//! THROWAWAY diagnostic: decode a Texture2D by path ID out of a bundle and report its
//! channel statistics (plus an optional PNG dump).
//!
//! Motivation: a scene layer's material can reference maps the scene exporter never
//! writes (`_DissolveTex`, `_DisturbTex`). Whether ignoring them matters depends
//! entirely on what those maps CONTAIN — a dissolve whose red channel sits near zero
//! masks almost the whole sheet away, one near white does nothing.
//!
//! Usage: cargo run --release --example probe_tex -- <bundle.ab> <pathID> [outdir]

use unpacker::export::texture::{decode_texture_object, save_decoded_texture};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("bundle path");
    let want: i64 = args.next().expect("pathID").parse().expect("pathID int");
    let outdir = args.next();

    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    let mut resources = std::collections::HashMap::new();
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            resources.insert(entry.path.clone(), entry.data.clone());
        }
    }

    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        for obj in &sf.objects {
            if obj.class_id != 28 || obj.path_id != want {
                continue;
            }
            let Ok(v) = read_object(&sf, obj) else {
                continue;
            };
            println!(
                "name={} {}x{} fmt={}",
                v["m_Name"].as_str().unwrap_or("?"),
                v["m_Width"].as_u64().unwrap_or(0),
                v["m_Height"].as_u64().unwrap_or(0),
                v["m_TextureFormat"].as_i64().unwrap_or(-1)
            );
            let Ok(Some(d)) = decode_texture_object(&v, &resources) else {
                println!("decode failed");
                return;
            };
            for (ci, cn) in ["R", "G", "B", "A"].iter().enumerate() {
                let mut vals: Vec<u8> = d.rgba.iter().skip(ci).step_by(4).copied().collect();
                vals.sort_unstable();
                let n = vals.len();
                let mean = vals.iter().map(|&x| f64::from(x)).sum::<f64>() / n as f64;
                println!(
                    "  {cn}: mean {:.1}  p10 {}  p50 {}  p90 {}",
                    mean,
                    vals[n / 10],
                    vals[n / 2],
                    vals[n * 9 / 10]
                );
            }
            if let Some(dir) = outdir {
                let p = std::path::Path::new(&dir);
                std::fs::create_dir_all(p).expect("mkdir");
                save_decoded_texture(&d, p).expect("save");
                println!("wrote {}/{}.png", dir, d.name);
            }
            return;
        }
    }
    println!("pathID {want} not found as a Texture2D");
}
