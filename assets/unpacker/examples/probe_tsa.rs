//! THROWAWAY diagnostic: dump the raw Texture Sheet Animation (`UVModule`) of every
//! ParticleSystem in a bundle.
//!
//! Motivation: Civilight Eterna's background planes sit on ONE tile of a 1x2 sheet for the
//! whole 5.5-12.0 s life of the planes in the capture, while we flip every ~0.5 s. The
//! exporter writes `frameOverTime` only when `minMaxState == 1` (curve mode) and drops it
//! otherwise, and never reads `startFrame` at all — so a CONSTANT authored frame would reach
//! the renderer as `null` and be re-interpreted as "animate by life fraction". This prints
//! the fields that decide it.
//!
//! Usage: cargo run --release --example probe_tsa -- <bundle.ab> [name-substring]

use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn f(v: &serde_json::Value, k: &str) -> String {
    v.get(k)
        .map(|x| {
            x.as_f64()
                .map(|n| format!("{n:.4}"))
                .unwrap_or_else(|| x.to_string())
        })
        .unwrap_or_else(|| "-".into())
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("bundle path");
    let want = args.next().unwrap_or_default().to_ascii_lowercase();

    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");

    let mut n = 0usize;
    for entry in &bundle.files {
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        for obj in &sf.objects {
            // 198 = ParticleSystem
            if obj.class_id != 198 {
                continue;
            }
            let Ok(ps) = read_object(&sf, obj) else {
                continue;
            };
            let name = ps["m_Name"].as_str().unwrap_or("").to_string();
            let Some(uv) = ps.get("UVModule") else {
                continue;
            };
            let enabled = uv.get("enabled").and_then(|x| x.as_bool()).unwrap_or(false)
                || uv.get("enabled").and_then(|x| x.as_i64()).unwrap_or(0) != 0;
            if !enabled {
                continue;
            }
            if !want.is_empty() && !name.to_ascii_lowercase().contains(&want) {
                continue;
            }
            n += 1;
            println!("=== {} (pathID {}) ===", if name.is_empty() { "<unnamed>" } else { &name }, obj.path_id);
            println!(
                "   tilesX={} tilesY={} cycles={} animationType={} rowMode={} rowIndex={} randomRow={} uvChannelMask={}",
                f(uv, "tilesX"),
                f(uv, "tilesY"),
                f(uv, "cycles"),
                f(uv, "animationType"),
                f(uv, "rowMode"),
                f(uv, "rowIndex"),
                f(uv, "randomRow"),
                f(uv, "uvChannelMask"),
            );
            // timeMode decides whether frameOverTime is used at all: 0 = Lifetime (curve-driven),
            // 1 = Speed, 2 = FPS (advance at `fps` frames/sec, curve IGNORED).
            println!(
                "   mode={} timeMode={} fps={} speedRange={} flipU={} flipV={}",
                f(uv, "mode"),
                f(uv, "timeMode"),
                f(uv, "fps"),
                uv.get("speedRange").map(|x| x.to_string()).unwrap_or("-".into()),
                f(uv, "flipU"),
                f(uv, "flipV"),
            );
            // startFrame and frameOverTime are both MinMaxCurves. minMaxState: 0 = constant,
            // 1 = curve, 2 = two curves, 3 = two constants. The exporter reads ONLY state 1.
            for key in ["frameOverTime", "startFrame"] {
                match uv.get(key) {
                    Some(c) => {
                        let state = c.get("minMaxState").and_then(|x| x.as_i64()).unwrap_or(-1);
                        let label = match state {
                            0 => "CONSTANT",
                            1 => "curve",
                            2 => "two curves",
                            3 => "two constants",
                            _ => "?",
                        };
                        println!(
                            "   {key}: minMaxState={state} ({label})  scalar={}  minScalar={}",
                            f(c, "scalar"),
                            f(c, "minScalar"),
                        );
                        // For a constant, Unity stores the value in the curve's single key.
                        for ck in ["maxCurve", "minCurve"] {
                            if let Some(curve) = c.get(ck)
                                && let Some(keys) = curve.get("m_Curve").and_then(|x| x.as_array())
                            {
                                let pts: Vec<String> = keys
                                    .iter()
                                    .take(6)
                                    .map(|k| format!("({},{})", f(k, "time"), f(k, "value")))
                                    .collect();
                                println!("      {ck}: {} keys {}", keys.len(), pts.join(" "));
                            }
                        }
                    }
                    None => println!("   {key}: ABSENT"),
                }
            }
        }
    }
    println!("\n{n} ParticleSystems with an ENABLED UVModule");
}
