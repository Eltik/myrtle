//! THROWAWAY measurement: what the exporter's own PNG encoder would save at a higher
//! deflate level, on the pages it actually writes.
//!
//! Motivation: every page in the tree is written by `image::save_buffer`, whose PNG default
//! is `CompressionType::Fast` (zlib header 78 01, FLEVEL 0). `PngEncoder::new_with_quality`
//! exposes `Default` (balanced) and `Best` (high) with adaptive filters; this decodes each
//! listed file and re-encodes it at all three settings, printing the bytes and the encode
//! wall time, so the delivery lever can be priced in the pipeline's own terms rather than
//! by an external optimizer's.
//!
//! Usage: cargo run --release --example `probe_pngbest` -- <file.png>...
use image::ImageEncoder;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use std::time::Instant;

fn encode(rgba: &[u8], w: u32, h: u32, c: CompressionType) -> (usize, f64) {
    let t0 = Instant::now();
    let mut out = Vec::with_capacity(rgba.len() / 4);
    PngEncoder::new_with_quality(&mut out, c, FilterType::Adaptive)
        .write_image(rgba, w, h, image::ExtendedColorType::Rgba8)
        .ok();
    (out.len(), t0.elapsed().as_secs_f64())
}

fn main() {
    let mut tot = [0usize; 4];
    let mut time = [0f64; 3];
    for path in std::env::args().skip(1) {
        let Ok(img) = image::open(&path) else {
            eprintln!("skip {path}");
            continue;
        };
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let orig = std::fs::metadata(&path)
            .map(|m| m.len() as usize)
            .unwrap_or(0);
        let (fast, tf) = encode(&rgba, w, h, CompressionType::Fast);
        let (def, td) = encode(&rgba, w, h, CompressionType::Default);
        let (best, tb) = encode(&rgba, w, h, CompressionType::Best);
        tot[0] += orig;
        tot[1] += fast;
        tot[2] += def;
        tot[3] += best;
        time[0] += tf;
        time[1] += td;
        time[2] += tb;
        let name = std::path::Path::new(&path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?");
        println!(
            "{name}\t{w}x{h}\torig {orig}\tfast {fast} ({tf:.2}s)\tdefault {def} ({td:.2}s)\tbest {best} ({tb:.2}s)"
        );
    }
    println!(
        "TOTAL orig {} fast {} default {} best {} | time fast {:.1}s default {:.1}s best {:.1}s",
        tot[0], tot[1], tot[2], tot[3], time[0], time[1], time[2]
    );
}
