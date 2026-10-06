//! Phase 0 embed: `chunks.jsonl` -> `vectors.bin` + `vectors.meta.json`.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use trevor::corpus::embed::{self, EmbedConfig};

#[derive(Parser)]
#[command(about = "Embed the Trevor P0 corpus with gte-modernbert-base")]
struct Args {
    /// Directory with the ONNX file and tokenizer.json, config.json,
    /// special_tokens_map.json, tokenizer_config.json.
    #[arg(long)]
    model_dir: PathBuf,
    #[arg(long, default_value = "model_int8.onnx")]
    onnx_file: String,
    /// Directory holding chunks.jsonl and manifest.p0.json.
    #[arg(long, default_value = "artifacts")]
    corpus: PathBuf,
    #[arg(long)]
    threads: Option<usize>,
    /// Embed chunk text without its P1 prefix (BM25 still indexes the prefix).
    #[arg(long)]
    ignore_prefix: bool,
    /// Embed every chunk, ignoring any prior vectors (the behavior before
    /// reuse existed; the output is byte-identical either way).
    #[arg(long)]
    no_reuse: bool,
    /// Another corpus directory whose vectors may be reused, after this one's
    /// own. Repeatable. E.g. `--reuse-from artifacts` for artifacts/p3a, whose
    /// first 13,837 rows are the P0 file.
    #[arg(long)]
    reuse_from: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let a = Args::parse();
    let started = std::time::Instant::now();
    let (meta, sources) = embed::run(
        &EmbedConfig {
            model_dir: a.model_dir,
            onnx_file: a.onnx_file,
            corpus: a.corpus,
            intra_threads: a.threads,
            ignore_prefix: a.ignore_prefix,
            reuse: !a.no_reuse,
            reuse_from: a.reuse_from,
        },
        &|done, total| {
            let s = started.elapsed().as_secs_f64();
            #[allow(clippy::cast_precision_loss)]
            let rate = done as f64 / s.max(1e-9);
            eprintln!("embedded {done}/{total}  {rate:.1}/s");
        },
    )?;
    for s in &sources {
        match &s.outcome {
            Ok((n, how)) => eprintln!("reuse: {} gave {n} vectors ({how})", s.dir.display()),
            Err(why) => eprintln!("reuse: {} skipped: {why}", s.dir.display()),
        }
    }
    eprintln!("reused {} | embedded {}", meta.reused, meta.embedded);
    eprintln!(
        "wrote {} x {} vectors in {:.1}s (max |norm-1| {:.2e}, model {:.16}, tokenizer {:.16})",
        meta.count, meta.dim, meta.seconds, meta.max_norm_error, meta.model_sha, meta.tokenizer_sha
    );
    Ok(())
}
