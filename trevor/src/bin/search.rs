//! Ask the corpus a question and print the passages it would retrieve.

use anyhow::Result;
use clap::Parser;
use trevor::search::pipeline::{Mode, RetrievalConfig};
use trevor::search::runtime::{ConfigArgs, Runtime, RuntimeArgs};

#[derive(Parser)]
#[command(about = "Retrieve passages for a question")]
struct Args {
    /// The question.
    query: String,
    #[command(flatten)]
    runtime: RuntimeArgs,
    #[command(flatten)]
    config: ConfigArgs,
    /// Characters of passage text to show per hit.
    #[arg(long, default_value_t = 240)]
    snippet: usize,
    /// Print JSON instead of a table.
    #[arg(long)]
    json: bool,
}

fn main() -> Result<()> {
    let a = Args::parse();
    let cfg = RetrievalConfig::from(&a.config);
    let mut rt = Runtime::load(
        &a.runtime,
        matches!(cfg.mode, Mode::Dense | Mode::Hybrid),
        matches!(cfg.mode, Mode::Bm25 | Mode::Hybrid),
        cfg.rerank_top > 0,
    )?;
    let started = std::time::Instant::now();
    let hits = rt.retrieve(&a.query, &cfg)?;
    let ms = started.elapsed().as_secs_f64() * 1000.0;
    if a.json {
        let rows: Vec<_> = hits
            .iter()
            .map(|h| {
                let c = &rt.store.chunks[h.row];
                serde_json::json!({"chunkId": c.chunk_id, "storyId": c.story_id,
                    "lineStart": c.line_start, "lineEnd": c.line_end, "score": h.score,
                    "denseRank": h.dense_rank, "bm25Rank": h.bm25_rank, "fusedRank": h.fused_rank})
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({"query": a.query, "ms": ms, "hits": rows})
            )?
        );
        return Ok(());
    }
    println!(
        "{} hits in {ms:.0} ms ({:?}, rerank_top {})",
        hits.len(),
        cfg.mode,
        cfg.rerank_top
    );
    let rank = |r: Option<usize>| r.map_or_else(|| "-".to_owned(), |r| r.to_string());
    for (i, h) in hits.iter().enumerate() {
        let c = &rt.store.chunks[h.row];
        let text: String = c
            .text
            .replace('\n', " / ")
            .chars()
            .take(a.snippet)
            .collect();
        println!(
            "\n{:>2}. {:.4}  {}  lines {}-{}  [dense {} | bm25 {} | fused {}]\n    {text}",
            i + 1,
            h.score,
            c.chunk_id,
            c.line_start,
            c.line_end,
            rank(h.dense_rank),
            rank(h.bm25_rank),
            rank(h.fused_rank)
        );
    }
    Ok(())
}
