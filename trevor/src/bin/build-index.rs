//! Build the BM25 index from `chunks.jsonl`.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use trevor::search::{bm25, store::ChunkStore};

#[derive(Parser)]
#[command(about = "Build the Trevor BM25 index from chunks.jsonl")]
struct Args {
    /// Directory holding chunks.jsonl. The index is written to <corpus>/bm25.
    #[arg(long, default_value = "artifacts")]
    corpus: PathBuf,
}

fn main() -> Result<()> {
    let a = Args::parse();
    let started = std::time::Instant::now();
    let store = ChunkStore::load(&a.corpus.join("chunks.jsonl"))?;
    bm25::build(&store, &a.corpus.join("bm25"))?;
    eprintln!(
        "indexed {} chunks in {:.1}s (chunks {:.16})",
        store.len(),
        started.elapsed().as_secs_f64(),
        store.chunks_sha
    );
    Ok(())
}
