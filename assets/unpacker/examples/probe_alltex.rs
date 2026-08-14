//! THROWAWAY diagnostic: enumerate EVERY `Texture2D` in a bundle with name, size and
//! (optionally) a PNG dump, so the raw asset inventory can be diffed against what the
//! exporter actually wrote.
//!
//! Motivation: Civilight Eterna's game capture shows rocky terrain, ruined architecture
//! and an ember field at t=8/11 that appear in none of our three draw groups, and none of
//! the 26 scene textures or 59 particle textures we export contain that art. Either the
//! exporter is dropping a texture, or the art is bound somewhere the scene/particle
//! exporters never look. This answers which by listing the ground truth.
//!
//! Usage: cargo run --release --example `probe_alltex` -- <bundle.ab> [outdir]
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_precision_loss,
    clippy::many_single_char_names
)]

use unpacker::export::texture::decode_texture_object;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("bundle path");
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

    if let Some(d) = &outdir {
        std::fs::create_dir_all(d).ok();
    }

    let mut n = 0usize;
    let mut total_px = 0u64;
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        for obj in &sf.objects {
            if obj.class_id != 28 {
                continue;
            }
            let Ok(v) = read_object(&sf, obj) else {
                continue;
            };
            let name = v["m_Name"].as_str().unwrap_or("?").to_string();
            let w = v["m_Width"].as_u64().unwrap_or(0);
            let h = v["m_Height"].as_u64().unwrap_or(0);
            let fmt = v["m_TextureFormat"].as_i64().unwrap_or(-1);
            n += 1;
            total_px += w * h;
            print!(
                "{:>4} pathID={:<22} {:<44} {:>5}x{:<5} fmt={:<3}",
                n, obj.path_id, name, w, h, fmt
            );
            // Mean luminance is the cheapest "is this art or a mask?" signal; a terrain
            // painting and a flow map are trivially separable by eye once dumped.
            match decode_texture_object(&v, &resources) {
                Ok(Some(d)) => {
                    let px = d.rgba.len() / 4;
                    let mut sum = 0u64;
                    for i in 0..px {
                        let r = u64::from(d.rgba[i * 4]);
                        let g = u64::from(d.rgba[i * 4 + 1]);
                        let b = u64::from(d.rgba[i * 4 + 2]);
                        sum += (r * 299 + g * 587 + b * 114) / 1000;
                    }
                    print!(" lum={:>5.1}", sum as f64 / px.max(1) as f64);
                    if let Some(dir) = &outdir {
                        // Key the file by pathID, NOT by name: duplicate m_Names are common
                        // and `save_decoded_texture` would silently overwrite them.
                        let safe: String = name
                            .chars()
                            .map(|c| {
                                if c.is_alphanumeric() || c == '_' {
                                    c
                                } else {
                                    '_'
                                }
                            })
                            .collect();
                        let out = format!("{dir}/{}__{}.png", obj.path_id, safe);
                        if image::save_buffer(
                            &out,
                            &d.rgba,
                            d.width,
                            d.height,
                            image::ColorType::Rgba8,
                        )
                        .is_err()
                        {
                            print!(" (save failed)");
                        }
                    }
                }
                _ => print!(" lum=  n/a"),
            }
            println!();
        }
    }
    println!("\n{n} Texture2D objects, {total_px} total pixels");
}
