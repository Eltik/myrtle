//! Phase 0 embed: `chunks.jsonl` -> `vectors.bin` + `vectors.meta.json`.
//!
//! Paced and resumable (design/trevor-reembed.md): two ONNX threads, a
//! resource guard, a 0.6 duty cycle and a checkpoint by default.
//! `--threads 0 --no-guard --duty 1 --checkpoint-every 0` turns all four off.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use trevor::corpus::embed::{self, EmbedConfig, Guard};

#[derive(Parser)]
#[command(about = "Embed the Trevor P0 corpus with gte-modernbert-base")]
struct Args {
    /// Directory with the ONNX file and tokenizer.json, config.json,
    /// special_tokens_map.json, tokenizer_config.json.
    #[arg(long)]
    model_dir: PathBuf,
    /// The ONNX export. `model.onnx` (fp32) gives the same vectors on every
    /// CPU; `model_int8.onnx` drifts to cosine 0.94 on x86.
    #[arg(long, visible_alias = "model-file", default_value = "model_int8.onnx")]
    onnx_file: String,
    /// Directory holding chunks.jsonl and manifest.p0.json.
    #[arg(long, default_value = "artifacts")]
    corpus: PathBuf,
    /// ONNX Runtime intra-op threads. 0 = ONNX Runtime's own default.
    #[arg(long, default_value_t = 2)]
    threads: usize,
    /// Embed chunk text without its P1 prefix (BM25 still indexes the prefix).
    #[arg(long)]
    ignore_prefix: bool,
    /// Embed every chunk, ignoring any prior vectors (the behavior before
    /// reuse existed; the output is byte-identical either way). An
    /// interrupted run's checkpoint is still read.
    #[arg(long)]
    no_reuse: bool,
    /// Another corpus directory whose vectors may be reused, after this one's
    /// own. Repeatable. E.g. `--reuse-from artifacts` for artifacts/p3a, whose
    /// first 13,837 rows are the P0 file.
    #[arg(long)]
    reuse_from: Vec<PathBuf>,
    /// Never pause for thermal, memory, battery, CPU pressure or load.
    #[arg(long)]
    no_guard: bool,
    /// Linux guard: pause while MemAvailable is below this many GiB.
    #[arg(long, default_value_t = 1.5)]
    guard_min_mem_gib: f64,
    /// Linux guard: pause while /proc/pressure/cpu some avg10 is above this.
    #[arg(long, default_value_t = 40.0)]
    guard_max_cpu_pressure: f64,
    /// Linux guard: do not pause when the load average exceeds the CPU count.
    #[arg(long)]
    guard_no_load: bool,
    /// macOS guard: keep running on battery power.
    #[arg(long)]
    guard_allow_battery: bool,
    /// Seconds between guard probes while running.
    #[arg(long, default_value_t = 10.0)]
    guard_every: f64,
    /// Seconds between guard probes while paused.
    #[arg(long, default_value_t = 30.0)]
    guard_pause: f64,
    /// Largest share of wall time spent embedding, in (0, 1]; the rest is
    /// sleep, to bound sustained heat. 1 = never sleep.
    #[arg(long, default_value_t = 0.6)]
    duty: f64,
    /// Stop cleanly after this many minutes (exit code 3); rerun to resume.
    #[arg(long)]
    max_minutes: Option<f64>,
    /// fsync the checkpoint (vectors.ckpt) every this many embedded chunks.
    /// 0 = no checkpoint.
    #[arg(long, default_value_t = 100)]
    checkpoint_every: usize,
}

fn main() -> Result<()> {
    let a = Args::parse();
    let started = std::time::Instant::now();
    let guard = (!a.no_guard).then_some(Guard {
        min_available_gib: a.guard_min_mem_gib,
        max_cpu_pressure: a.guard_max_cpu_pressure,
        check_load: !a.guard_no_load,
        pause_on_battery: !a.guard_allow_battery,
        every_secs: a.guard_every,
        pause_secs: a.guard_pause,
    });
    eprintln!(
        "embed-corpus: {} on {}, threads {}, guard {}, duty {}, checkpoint every {}{}",
        a.onnx_file,
        a.corpus.display(),
        if a.threads == 0 {
            "ort-default".to_owned()
        } else {
            a.threads.to_string()
        },
        if guard.is_some() { "on" } else { "off" },
        a.duty,
        a.checkpoint_every,
        a.max_minutes.map(|m| format!(", max {m} min")).unwrap_or_default()
    );
    let r = embed::run(
        &EmbedConfig {
            model_dir: a.model_dir,
            onnx_file: a.onnx_file,
            corpus: a.corpus,
            intra_threads: (a.threads > 0).then_some(a.threads),
            ignore_prefix: a.ignore_prefix,
            reuse: !a.no_reuse,
            reuse_from: a.reuse_from,
            guard,
            duty: a.duty,
            max_minutes: a.max_minutes,
            checkpoint_every: a.checkpoint_every,
        },
        &|done, total| {
            let s = started.elapsed().as_secs_f64();
            #[allow(clippy::cast_precision_loss)]
            let rate = done as f64 / s.max(1e-9);
            eprintln!("embedded {done}/{total}  {rate:.1}/s");
        },
    )?;
    for s in &r.sources {
        match &s.outcome {
            Ok((n, how)) => eprintln!("reuse: {} gave {n} vectors ({how})", s.dir.display()),
            Err(why) => eprintln!("reuse: {} skipped: {why}", s.dir.display()),
        }
    }
    let st = &r.stats;
    eprintln!(
        "pacing: {:.1}s embedding, {:.1}s duty sleep, {:.1}s paused in {} pauses",
        st.busy_secs, st.duty_secs, st.paused_secs, st.pauses
    );
    let Some(meta) = r.meta else {
        eprintln!(
            "stopped at {}/{}: {}",
            st.done,
            st.total,
            st.stopped.as_deref().unwrap_or("?")
        );
        std::process::exit(3);
    };
    eprintln!("reused {} | embedded {}", meta.reused, meta.embedded);
    if meta.resumed > 0 {
        eprintln!("of which {} resumed from the checkpoint", meta.resumed);
    }
    eprintln!(
        "wrote {} x {} vectors in {:.1}s (max |norm-1| {:.2e}, model {:.16}, tokenizer {:.16})",
        meta.count, meta.dim, meta.seconds, meta.max_norm_error, meta.model_sha, meta.tokenizer_sha
    );
    Ok(())
}
