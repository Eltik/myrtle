//! P3b: character dossiers and event summaries as search units.
//!
//! Comparison and overview questions ("rank the Sui by power") need material no single passage
//! holds. This writes a corpus directory whose `chunks.jsonl` is the input corpus byte for byte,
//! followed by one row per character dossier (`scripts/dossiers.py`) and per method-C event
//! summary (`scripts/p2-summaries.py`). Profile rows use story id `profile_<slug>` or
//! `summary_<groupId>` and never overlap a gold anchor. The input directory is never touched.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use tokenizers::Tokenizer;
use trevor::corpus::derived;

#[derive(Parser)]
#[command(about = "Append dossiers and event summaries to a copy of a corpus")]
struct Args {
    #[arg(long, default_value = "artifacts/p3a")]
    corpus: PathBuf,
    #[arg(long, default_value = "artifacts/dossiers/dossiers.jsonl")]
    dossiers: PathBuf,
    #[arg(long, default_value = "artifacts/p2/groups.jsonl")]
    groups: PathBuf,
    #[arg(long, default_value = "artifacts/p2/stories.jsonl")]
    stories: PathBuf,
    #[arg(long, default_value = "models/gte-modernbert-base/tokenizer.json")]
    tokenizer: PathBuf,
    #[arg(long, default_value = "artifacts/p3b")]
    out: PathBuf,
}

fn slug(s: &str) -> String {
    s.chars().map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' }).collect()
}

fn lines(p: &PathBuf) -> Result<Vec<serde_json::Value>> {
    std::fs::read_to_string(p)
        .with_context(|| format!("reading {}", p.display()))?
        .lines()
        .map(|l| Ok(serde_json::from_str(l)?))
        .collect()
}

fn main() -> Result<()> {
    // SAFETY: set before any other thread exists.
    unsafe { std::env::set_var("TOKENIZERS_PARALLELISM", "false") };
    let a = Args::parse();
    derived::ensure_distinct_out(&a.out, &a.corpus)?;
    let tok = Tokenizer::from_bytes(std::fs::read(&a.tokenizer)?).map_err(|e| anyhow::anyhow!("{e}"))?;
    let count = |s: &str| derived::token_count(&tok, s);
    let mut group_name = std::collections::HashMap::new();
    for s in lines(&a.stories)? {
        if let (Some(g), Some(h)) = (s["groupId"].as_str(), s["header"].as_str()) {
            group_name.entry(g.to_owned()).or_insert_with(|| h.split(',').next().unwrap_or(h).trim().to_owned());
        }
    }
    let row = |id: String, group: &str, speakers: Vec<String>, text: String| {
        let n = count(&text);
        derived::generated_chunk(format!("{id}#0000"), id, group.to_owned(), 0, speakers, text, n)
    };
    let mut rows = Vec::new();
    for d in lines(&a.dossiers)? {
        let name = d["name"].as_str().unwrap_or_default().to_owned();
        let aka: Vec<&str> = d["names"]
            .as_array()
            .map(|v| v.iter().filter_map(|x| x.as_str()).filter(|x| *x != name).collect())
            .unwrap_or_default();
        let head = if aka.is_empty() {
            format!("Character profile: {name}")
        } else {
            format!("Character profile: {name} (also known as {})", aka.join(", "))
        };
        let text = format!("{head}\n{}", d["dossier"].as_str().unwrap_or_default());
        rows.push(row(format!("profile_{}", slug(&name)), "profile", vec![name], text));
    }
    for g in lines(&a.groups)? {
        if g["method"].as_str() != Some("C") {
            continue;
        }
        let gid = g["groupId"].as_str().unwrap_or_default();
        let name = group_name.get(gid).cloned().unwrap_or_else(|| gid.to_owned());
        let text = format!("Event summary: {name}\n{}", g["summary"].as_str().unwrap_or_default());
        rows.push(row(format!("summary_{gid}"), "summary", Vec::new(), text));
    }
    let src = std::fs::read(a.corpus.join("chunks.jsonl"))?;
    derived::write_appended(&a.out, &src, &rows)?;
    derived::copy_sidecars(&a.corpus, &a.out)?;
    eprintln!(
        "profiles: {} rows appended to {} ({} source bytes, max {} tokens)",
        rows.len(),
        a.corpus.display(),
        src.len(),
        rows.iter().map(|c| c.token_count).max().unwrap_or(0)
    );
    Ok(())
}
