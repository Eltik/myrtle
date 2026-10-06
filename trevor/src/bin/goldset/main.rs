//! Build the retrieval gold set with local models and no manual review.
//!
//! Stages, each resumable and each writing one file under `--work`:
//!
//! ```text
//! sample        backend index + chunks        -> sample.jsonl, names.json
//! generate      generator server (Gemma)      -> generated.jsonl
//! filter        BM25 + embedder, no LLM       -> filtered.jsonl
//! closed-book   generator server              -> closed_book.jsonl
//! judge         judge server (Qwen)           -> judged.jsonl
//! finish        judge server + indexes        -> eval/goldset_v1.jsonl, .meta.json, .spotcheck.md
//! ```

mod files;
mod prompt;
mod sampling;
mod generate;
mod filter;
mod judge;
mod calibrate;
mod finish;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use trevor::search::runtime::RuntimeArgs;

use crate::calibrate::cmd_calibrate;
use crate::filter::cmd_filter;
use crate::finish::cmd_finish;
use crate::generate::{GenArgs, cmd_generate};
use crate::judge::{cmd_closed_book, cmd_judge};
use crate::sampling::{cmd_probe, cmd_sample};

#[derive(Parser)]
#[command(about = "Generate the Trevor retrieval gold set with local models")]
struct Args {
    /// Directory for the intermediate files.
    #[arg(long, default_value = "artifacts/goldgen", global = true)]
    work: PathBuf,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Draw source passages. Appends a new round if sample.jsonl exists.
    Sample {
        #[arg(long, default_value = "artifacts")]
        corpus: PathBuf,
        /// Backend, for story and group names.
        #[arg(long, default_value = "http://127.0.0.1:3060")]
        base: String,
        #[arg(long, default_value_t = 1.0)]
        scale: f64,
        #[arg(long, default_value_t = 20_260_925)]
        seed: u64,
    },
    /// Render one prompt and run it, to check a server and its template.
    Probe {
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
        #[arg(long, default_value = "artifacts")]
        corpus: PathBuf,
    },
    Generate {
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
        #[arg(long, default_value = "artifacts")]
        corpus: PathBuf,
        #[command(flatten)]
        knobs: GenArgs,
    },
    Filter {
        #[command(flatten)]
        runtime: RuntimeArgs,
        /// Accept an evidence quote that is not verbatim in its passage when
        /// its verbatim prefix has at least this many characters, and carry
        /// the prefix forward as the evidence. Absent means v1: whole quote
        /// or reject.
        #[arg(long)]
        evidence_min_prefix: Option<usize>,
    },
    ClosedBook {
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
    },
    Judge {
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
        #[arg(long, default_value = "artifacts")]
        corpus: PathBuf,
    },
    /// Measure the judge itself: its verdicts on passages known NOT to
    /// answer a question, and on questions known to be bad or good.
    Calibrate {
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
        #[arg(long, default_value = "artifacts")]
        corpus: PathBuf,
        #[arg(long, default_value_t = 60)]
        n: usize,
    },
    Finish {
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
        #[command(flatten)]
        runtime: RuntimeArgs,
        #[arg(long, default_value = "eval/goldset_v1")]
        out: PathBuf,
        /// Passages taken from each retriever for anchor completion.
        #[arg(long, default_value_t = 5)]
        pool_depth: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let a = Args::parse();
    std::fs::create_dir_all(&a.work)?;
    let work = a.work.clone();
    match a.cmd {
        Cmd::Sample {
            corpus,
            base,
            scale,
            seed,
        } => cmd_sample(&work, &corpus, &base, scale, seed).await,
        Cmd::Probe { server, corpus } => cmd_probe(&work, &server, &corpus).await,
        Cmd::Generate {
            server,
            corpus,
            knobs,
        } => cmd_generate(&work, &server, &corpus, &knobs).await,
        Cmd::Filter {
            runtime,
            evidence_min_prefix,
        } => cmd_filter(&work, &runtime, evidence_min_prefix),
        Cmd::ClosedBook { server } => cmd_closed_book(&work, &server).await,
        Cmd::Judge { server, corpus } => cmd_judge(&work, &server, &corpus).await,
        Cmd::Calibrate { server, corpus, n } => cmd_calibrate(&work, &server, &corpus, n).await,
        Cmd::Finish {
            server,
            runtime,
            out,
            pool_depth,
        } => cmd_finish(&work, &server, &runtime, &out, pool_depth).await,
    }
}
