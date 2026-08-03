//! THROWAWAY diagnostic: corpus census of the two ParticleSystem main-module fields the
//! exporter has never read — `simulationSpeed` and `prewarm`.
//!
//! Motivation: Virtuosa's nested diamond frames (`sys55`) sit at `simulationSpeed 0.3`, i.e.
//! Unity runs that system's whole clock at 30% — emission interval, particle age, sizeOverLife
//! and the 45 deg/s rotation all scale by it. `prewarm` additionally pre-simulates one full
//! duration at t=0 so a looping system starts in steady state instead of building up. Neither
//! is exported, which is enough on its own to put every ring at the wrong radius and angle.
//! Before changing the exporter, measure how much of the corpus this touches.
//!
//! Usage: cargo run --release --example probe_simspeed -- <dir-of-bundles>

use serde_json::Value;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn main() {
    let dir = std::env::args().nth(1).expect("usage: probe_simspeed <dir-or-bundle>");
    let dirp = std::path::PathBuf::from(&dir);
    // A single .ab dumps PER-SYSTEM detail (the startDelay question); a directory censuses.
    let detail = dirp.is_file();
    let mut files: Vec<std::path::PathBuf> = if detail {
        vec![dirp]
    } else {
        std::fs::read_dir(&dir)
            .expect("read dir")
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "ab"))
            .collect()
    };
    files.sort();
    if detail {
        let data = std::fs::read(&files[0]).expect("read bundle");
        let bundle = BundleFile::parse(data).expect("parse");
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
            for o in sf.objects.iter().filter(|o| o.class_id == 198) {
                let Ok(ps) = read_object(&sf, o) else { continue };
                let sp = ps.get("simulationSpeed").and_then(Value::as_f64).unwrap_or(1.0);
                let pw = ps.get("prewarm").and_then(Value::as_bool).unwrap_or(false);
                if (sp - 1.0).abs() <= 1e-3 && !pw {
                    continue;
                }
                // `startDelay` is a MinMaxCurve; the constant lives in `scalar`.
                let sd = ps
                    .get("startDelay")
                    .and_then(|v| v.get("scalar").and_then(Value::as_f64).or_else(|| v.as_f64()))
                    .unwrap_or(0.0);
                println!(
                    "  pathID {:>20}  simSpeed {sp:6.3}  prewarm {:<5}  startDelay {sd:7.3}  looping {}",
                    o.path_id,
                    pw,
                    ps.get("looping").and_then(Value::as_bool).unwrap_or(false)
                );
            }
        }
        return;
    }

    let (mut total, mut slow, mut fast, mut prewarm, mut slow_and_prewarm) = (0usize, 0usize, 0usize, 0usize, 0usize);
    let mut skins_touched: Vec<(String, usize, usize)> = Vec::new();

    for f in &files {
        let Ok(data) = std::fs::read(f) else { continue };
        let Ok(bundle) = BundleFile::parse(data) else { continue };
        let (mut n_skin_slow, mut n_skin_pre) = (0usize, 0usize);
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
            for o in sf.objects.iter().filter(|o| o.class_id == 198) {
                let Ok(ps) = read_object(&sf, o) else { continue };
                total += 1;
                let sp = ps.get("simulationSpeed").and_then(Value::as_f64).unwrap_or(1.0);
                let pw = ps.get("prewarm").and_then(Value::as_bool).unwrap_or(false);
                if (sp - 1.0).abs() > 1e-3 {
                    if sp < 1.0 {
                        slow += 1;
                    } else {
                        fast += 1;
                    }
                    n_skin_slow += 1;
                }
                if pw {
                    prewarm += 1;
                    n_skin_pre += 1;
                }
                if (sp - 1.0).abs() > 1e-3 && pw {
                    slow_and_prewarm += 1;
                }
            }
        }
        if n_skin_slow > 0 || n_skin_pre > 0 {
            let name = f.file_stem().and_then(|s| s.to_str()).unwrap_or("?").to_string();
            skins_touched.push((name, n_skin_slow, n_skin_pre));
        }
    }

    println!("\nparticle systems scanned: {total}   bundles: {}", files.len());
    println!("  simulationSpeed < 1 : {slow}");
    println!("  simulationSpeed > 1 : {fast}");
    println!("  prewarm = true      : {prewarm}");
    println!("  both                : {slow_and_prewarm}");
    println!("\nskins touched: {}", skins_touched.len());
    for (n, s, p) in &skins_touched {
        println!("  {n:<48} simSpeed!=1 {s:<4} prewarm {p}");
    }
}
