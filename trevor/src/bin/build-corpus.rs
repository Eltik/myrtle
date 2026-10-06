//! Phase 0 corpus build: ingest from the running backend, chunk with the
//! embedder's tokenizer, write `chunks.jsonl`, `spoiler.jsonl`,
//! `unresolved.jsonl` and `manifest.p0.json`.
//!
//! By default a rerun resumes: stories already in `chunks.jsonl` are skipped
//! by id, so an asset update that edits an existing story is NOT picked up.
//! `--refresh` refetches every story, replaces the chunks of the ones whose
//! lines changed, and writes `changes.json` for the updater.
//!
//! Runs on the Mac next to the backend. Exits nonzero when any story failed
//! to fetch, failed to reconstruct from its chunks, or disagreed with the
//! API's word count.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use clap::Parser;
use tokenizers::Tokenizer;
use trevor::corpus::chunk::ChunkConfig;
use trevor::corpus::ingest::{self, IngestConfig};

#[derive(Parser)]
#[command(about = "Build the Trevor P0 corpus from the running myrtle backend")]
struct Args {
    /// Backend base URL.
    #[arg(long, default_value = "http://127.0.0.1:3060")]
    base: String,
    /// Output directory.
    #[arg(long, default_value = "artifacts")]
    out: PathBuf,
    /// The embedder's tokenizer.json. The SAME file must be used to embed.
    #[arg(long)]
    tokenizer: PathBuf,
    /// Concurrent story fetches.
    #[arg(long, default_value_t = 4)]
    concurrency: usize,
    /// Only the first N scripted stories, for a smoke test.
    #[arg(long)]
    limit: Option<usize>,
    /// Chunk floor and ceiling in model tokens.
    #[arg(long, default_value_t = 400)]
    min_tokens: u32,
    #[arg(long, default_value_t = 600)]
    max_tokens: u32,
    #[arg(long, default_value_t = 3)]
    max_scenes: u32,
    /// Refetch every story and replace the chunks of those whose content
    /// changed, instead of skipping stories already in chunks.jsonl. Writes
    /// changes.json (added, changed, removed, unchanged, failed story ids)
    /// and keeps the previous file as chunks.prev.jsonl. Off: resume, as
    /// before.
    #[arg(long)]
    refresh: bool,
}

/// Derive the sequence and join overhead from the tokenizer itself.
///
/// Measured for gte-modernbert on the real corpus: 2 per sequence, and 1 per
/// `\n` join in all 24,757 real joins. The probes here re-derive both so a
/// different tokenizer cannot silently inherit those numbers, and refuse to
/// guess when the join cost is not constant.
fn calibrate(tok: &Tokenizer) -> Result<(u32, u32)> {
    let n = |s: &str, special: bool| -> Result<i64> {
        Ok(i64::try_from(
            tok.encode(s, special)
                .map_err(|e| anyhow::anyhow!("{e}"))?
                .get_ids()
                .len(),
        )?)
    };
    let probe = "Amiya: Doctor, are you awake?";
    let overhead = n(probe, true)? - n(probe, false)?;

    let pairs = [
        ("Amiya: Doctor, are you awake?", "Kal'tsit: Leave him be."),
        ("W: Heh.", "???: ..."),
        (
            "The rain has not stopped for three days.",
            "Ch'en: Move out.",
        ),
        ("Rhodes Island Operator: Copy that.", "Ifrit-Nian: Fine!"),
    ];
    let mut joins = Vec::new();
    for (a, b) in pairs {
        joins.push(n(&format!("{a}\n{b}"), false)? - n(a, false)? - n(b, false)?);
    }
    joins.sort_unstable();
    joins.dedup();
    if joins.len() != 1 {
        bail!("join cost is not constant across probes: {joins:?}; refusing to guess");
    }
    Ok((u32::try_from(overhead)?, u32::try_from(joins[0])?))
}

#[tokio::main]
async fn main() -> Result<()> {
    // The tokenizers crate parallelises batch encodes with rayon; this build
    // encodes one turn at a time from blocking tasks, and a second thread pool
    // would only fight tokio for the same cores.
    // SAFETY: set before any other thread exists.
    unsafe { std::env::set_var("TOKENIZERS_PARALLELISM", "false") };

    let args = Args::parse();
    let bytes = std::fs::read(&args.tokenizer)
        .with_context(|| format!("reading {}", args.tokenizer.display()))?;
    let tokenizer_sha = trevor::util::sha_hex(&bytes);
    let tok = Tokenizer::from_bytes(&bytes).map_err(|e| anyhow::anyhow!("{e}"))?;
    let (overhead, joiner) = calibrate(&tok)?;
    eprintln!("tokenizer {tokenizer_sha:.16}: sequence overhead {overhead}, join {joiner}");

    let tok = Arc::new(tok);
    let count: Arc<dyn Fn(&str) -> u32 + Send + Sync> = Arc::new(move |s: &str| {
        // Without special tokens: they belong to the sequence and are charged
        // once through `sequence_overhead_tokens`.
        tok.encode(s, false)
            .map_or(0, |e| u32::try_from(e.get_ids().len()).unwrap_or(u32::MAX))
    });

    let cfg = IngestConfig {
        base: args.base,
        out: args.out,
        concurrency: args.concurrency,
        limit: args.limit,
        service_key: std::env::var("TREVOR_SERVICE_KEY").ok(),
        chunk: ChunkConfig {
            target_min_tokens: args.min_tokens,
            target_max_tokens: args.max_tokens,
            max_scenes_per_chunk: args.max_scenes,
            sequence_overhead_tokens: overhead,
            joiner_tokens: joiner,
            ..ChunkConfig::default()
        },
        tokenizer_sha,
        refresh: args.refresh,
    };

    let m = ingest::run(&cfg, count).await?;
    eprintln!(
        "api: {} stories, {} with script | planned {} scripted, {} unscripted",
        m.totals_from_api.stories,
        m.totals_from_api.with_script,
        m.scripted_stories,
        m.unscripted_stories
    );
    eprintln!(
        "fetched {} | skipped as done {} | failed {} | chunks this run {} | chunks in file {}",
        m.stories_fetched,
        m.stories_skipped_as_done,
        m.stories_failed,
        m.chunks_written_this_run,
        m.chunks_in_file
    );
    if let Some(c) = &m.refresh {
        eprintln!(
            "refresh: added {} | changed {} | removed {} | unchanged {} | failed {} | chunks {} -> {} ({} added or changed)",
            c.added,
            c.changed,
            c.removed,
            c.unchanged,
            c.failed,
            c.old_chunks,
            c.new_chunks,
            c.chunks_added_or_changed
        );
    }
    if let Some(s) = &m.recovery.dropped_last_story {
        eprintln!(
            "resume: dropped {} line(s) of {s} for refetch, discarded {} torn byte(s)",
            m.recovery.lines_dropped, m.recovery.torn_bytes_discarded
        );
    }
    eprintln!(
        "checks: lossless failures {} | word-count mismatches {}",
        m.lossless_failures.len(),
        m.word_count_mismatches.len()
    );
    if !m.clean() {
        bail!("corpus build finished with failures; see manifest.p0.json");
    }
    Ok(())
}
