//! Cross-machine embedding check (read-only): re-embed N chunks of a corpus on
//! this machine and compare each fresh vector with the stored one.
//!
//! The gate for a new host is this check, not a CPU flag (design/trevor-p0-status.md: the INT8 export scores cosine
//! 0.94 on an AVX-512 VNNI Xeon, 0.997 on the Mac). A host may add vectors to a corpus only when its fresh vectors
//! match the stored ones: the fp32 export should give a minimum near 1.0 everywhere. Writes nothing; exit code 1 when
//! the minimum cosine is below `--min-cos`, or when the fresh model is not the one the vectors were built with (a
//! mismatch is still measured, for the INT8-vs-fp32 comparison).
//!
//! ```sh
//! target/release/embed-check --corpus artifacts/p3b-fp32 --n 200
//! ```

#[cfg(feature = "embed-core")]
fn main() -> anyhow::Result<()> {
    imp::main()
}

#[cfg(not(feature = "embed-core"))]
fn main() {
    eprintln!("embed-check needs ONNX Runtime: build with --features embed");
    std::process::exit(2);
}

#[cfg(feature = "embed-core")]
mod imp {
    use std::io::BufRead as _;
    use std::path::PathBuf;

    use anyhow::{Context, Result, bail};
    use clap::Parser;
    use serde::Deserialize;
    use trevor::corpus::chunk::indexed_text;
    use trevor::corpus::embed::load_model;
    use trevor::util::{sha_hex, splitmix64};

    #[derive(Parser)]
    #[command(about = "Re-embed N stored chunks on this machine and compare with the stored vectors (read-only)")]
    struct Args {
        #[arg(long, default_value = "models/gte-modernbert-base")]
        model_dir: PathBuf,
        /// The ONNX file to embed with now; usually the one the vectors were built with.
        #[arg(long, visible_alias = "model-file", default_value = "model.onnx")]
        onnx_file: String,
        /// Directory holding chunks.jsonl, vectors.bin and vectors.meta.json.
        #[arg(long, default_value = "artifacts/p3b")]
        corpus: PathBuf,
        /// Chunks to sample (seeded, without replacement).
        #[arg(long, default_value_t = 200)]
        n: usize,
        #[arg(long, default_value_t = 20_261_016)]
        seed: u64,
        /// Comma-separated chunk ids to check instead of a sample.
        #[arg(long)]
        ids: Option<String>,
        /// ONNX Runtime intra-op threads (0 = its default).
        #[arg(long, default_value_t = 2)]
        threads: usize,
        /// Exit 1 when the minimum cosine is below this.
        #[arg(long, default_value_t = 0.999)]
        min_cos: f32,
        /// Also write one JSON line per chunk (chunkId, cos) here.
        #[arg(long)]
        out: Option<PathBuf>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Meta {
        model_file: String,
        model_sha: String,
        tokenizer_sha: String,
        chunks_sha: String,
        dim: usize,
        count: usize,
        chunk_ids: Vec<String>,
        #[serde(default)]
        prefix_ignored: bool,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Line {
        chunk_id: String,
        text: String,
        #[serde(default)]
        prefix: Option<String>,
    }

    fn pct(sorted: &[f32], q: f64) -> f32 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
        let i = ((sorted.len() - 1) as f64 * q).round() as usize;
        sorted[i]
    }

    #[allow(clippy::too_many_lines)]
    pub fn main() -> Result<()> {
        let a = Args::parse();
        let meta: Meta = serde_json::from_slice(
            &std::fs::read(a.corpus.join("vectors.meta.json")).context("reading vectors.meta.json")?,
        )?;
        let chunks = std::fs::read(a.corpus.join("chunks.jsonl")).context("reading chunks.jsonl")?;
        if sha_hex(&chunks) != meta.chunks_sha {
            bail!("vectors.meta.json was built from other chunks than chunks.jsonl; the stored vectors are stale");
        }
        let lines: Vec<Line> = chunks
            .lines()
            .map(|l| Ok(serde_json::from_str(&l?)?))
            .collect::<Result<_>>()?;
        if lines.len() != meta.count || meta.chunk_ids.len() != meta.count {
            bail!("{} chunks, {} vectors", lines.len(), meta.count);
        }
        let bin = std::fs::read(a.corpus.join("vectors.bin")).context("reading vectors.bin")?;
        if bin.len() != meta.count * meta.dim * 4 {
            bail!("vectors.bin is {} bytes, expected {}", bin.len(), meta.count * meta.dim * 4);
        }

        let rows: Vec<usize> = if let Some(ids) = &a.ids {
            ids.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|id| {
                    meta.chunk_ids.iter().position(|x| x == id).with_context(|| format!("no chunk {id}"))
                })
                .collect::<Result<_>>()?
        } else {
            let mut idx: Vec<usize> = (0..meta.count).collect();
            let mut st = a.seed;
            let n = a.n.min(meta.count);
            for i in 0..n {
                #[allow(clippy::cast_possible_truncation)]
                let j = i + (splitmix64(&mut st) % (meta.count - i) as u64) as usize;
                idx.swap(i, j);
            }
            let mut pick = idx[..n].to_vec();
            pick.sort_unstable();
            pick
        };
        if rows.is_empty() {
            bail!("no chunks to check");
        }

        let m = load_model(&a.model_dir, &a.onnx_file, (a.threads > 0).then_some(a.threads))?;
        if m.tokenizer_sha != meta.tokenizer_sha {
            bail!("tokenizer {:.16} differs from the vectors' {:.16}", m.tokenizer_sha, meta.tokenizer_sha);
        }
        let same_model = m.model_sha == meta.model_sha;
        eprintln!(
            "stored: {} ({:.16}) in {}; fresh: {} ({:.16}){}",
            meta.model_file,
            meta.model_sha,
            a.corpus.display(),
            a.onnx_file,
            m.model_sha,
            if same_model { "" } else { "  MODEL DIFFERS" }
        );
        let mut emb = m.embedding;
        let mut out = match &a.out {
            Some(p) => Some(std::io::BufWriter::new(std::fs::File::create(p)?)),
            None => None,
        };
        let started = std::time::Instant::now();
        let mut cos: Vec<(f32, usize)> = Vec::with_capacity(rows.len());
        for &r in &rows {
            let l = &lines[r];
            let text = indexed_text(l.prefix.as_deref().filter(|_| !meta.prefix_ignored), &l.text);
            let v = emb
                .embed([text.as_ref()], None)
                .map_err(|e| anyhow::anyhow!("embedding {}: {e}", l.chunk_id))?
                .pop()
                .context("no vector")?;
            if v.len() != meta.dim {
                bail!("fresh dim {} vs stored {}", v.len(), meta.dim);
            }
            let stored = &bin[r * meta.dim * 4..(r + 1) * meta.dim * 4];
            let c: f32 = stored
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .zip(&v)
                .map(|(s, f)| s * f)
                .sum();
            if let Some(w) = out.as_mut() {
                use std::io::Write as _;
                writeln!(w, "{}", serde_json::json!({"chunkId": l.chunk_id, "cos": c}))?;
            }
            cos.push((c, r));
        }
        // Flushed here: a FAIL exits through process::exit, which skips drops.
        if let Some(mut w) = out.take() {
            use std::io::Write as _;
            w.flush()?;
        }
        let secs = started.elapsed().as_secs_f64();
        let mut sorted: Vec<f32> = cos.iter().map(|x| x.0).collect();
        sorted.sort_by(f32::total_cmp);
        #[allow(clippy::cast_precision_loss)]
        let mean = sorted.iter().map(|&x| f64::from(x)).sum::<f64>() / sorted.len() as f64;
        #[allow(clippy::cast_precision_loss)]
        let rate = rows.len() as f64 / secs.max(1e-9);
        println!(
            "n {}  min {:.6}  p01 {:.6}  p05 {:.6}  median {:.6}  mean {:.6}  max {:.6}  ({:.1} chunks/s)",
            sorted.len(),
            sorted[0],
            pct(&sorted, 0.01),
            pct(&sorted, 0.05),
            pct(&sorted, 0.5),
            mean,
            sorted[sorted.len() - 1],
            rate
        );
        cos.sort_by(|x, y| x.0.total_cmp(&y.0));
        for (c, r) in cos.iter().take(5) {
            println!("  worst {:.6}  {}", c, meta.chunk_ids[*r]);
        }
        let pass = same_model && sorted[0] >= a.min_cos;
        println!(
            "{}: {}",
            if pass { "PASS" } else { "FAIL" },
            if same_model {
                format!("min cosine {:.6} vs bar {}", sorted[0], a.min_cos)
            } else {
                format!("fresh model is not the vectors' model ({})", meta.model_file)
            }
        );
        if !pass {
            std::process::exit(1);
        }
        Ok(())
    }
}
