//! P4: typed source units (module stories, voice lines, Integrated Strategies, enemy entries, outfits,
//! items; `scripts/sources.py`) appended to a copy of a corpus.
//!
//! Source-specific questions ("what does Kal'tsit's module say") were answered 0.14 of the time on real
//! questions, from story passages. This writes a corpus directory whose `chunks.jsonl` is the input corpus
//! byte for byte, followed by each unit packed into chunks of at most `--max-tokens`: whole lines are packed
//! in order, and every chunk after a unit's first repeats the unit's title line, so a voice-line or IS chunk
//! still says whose lines or which run it is. `groupId` is the source kind. The input directory is never
//! touched.

use std::io::Write as _;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;
use sha2::{Digest, Sha256};
use tokenizers::Tokenizer;
use trevor::corpus::chunk::Chunk;

#[derive(Parser)]
#[command(about = "Append typed source units to a copy of a corpus")]
struct Args {
    #[arg(long, default_value = "artifacts/p3b")]
    corpus: PathBuf,
    #[arg(long, default_value = "artifacts/sources/units.jsonl")]
    units: PathBuf,
    #[arg(long, default_value = "models/gte-modernbert-base/tokenizer.json")]
    tokenizer: PathBuf,
    /// Largest chunk, in model tokens including the sequence's special tokens (P0 packs 400 to 600).
    #[arg(long, default_value_t = 480)]
    max_tokens: u32,
    #[arg(long, default_value = "artifacts/p4")]
    out: PathBuf,
}

fn sha_hex(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}

fn main() -> Result<()> {
    // SAFETY: set before any other thread exists.
    unsafe { std::env::set_var("TOKENIZERS_PARALLELISM", "false") };
    let a = Args::parse();
    if a.out.canonicalize().ok() == Some(a.corpus.canonicalize()?) {
        bail!("--out must not be the input corpus directory");
    }
    let tok = Tokenizer::from_bytes(std::fs::read(&a.tokenizer)?).map_err(|e| anyhow::anyhow!("{e}"))?;
    let count = |s: &str| -> u32 {
        tok.encode(s, true).map_or(u32::MAX, |e| u32::try_from(e.get_ids().len()).unwrap_or(u32::MAX))
    };
    let mut rows = Vec::new();
    let mut split_lines = 0usize;
    for l in std::fs::read_to_string(&a.units).with_context(|| format!("reading {}", a.units.display()))?.lines() {
        let u: serde_json::Value = serde_json::from_str(l)?;
        let sid = u["storyId"].as_str().context("storyId")?.to_owned();
        let group = u["groupId"].as_str().context("groupId")?.to_owned();
        let speakers: Vec<String> = u["speakers"].as_array().map(|v| v.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect()).unwrap_or_default();
        let text = u["text"].as_str().unwrap_or_default();
        let mut it = text.lines();
        let title = it.next().unwrap_or_default().to_owned();
        let mut chunks: Vec<String> = Vec::new();
        let mut cur = title.clone();
        for line in it {
            // A single line longer than the budget is cut by words so no chunk exceeds it.
            let mut pieces = vec![line.to_owned()];
            if count(&format!("{title}\n{line}")) > a.max_tokens {
                split_lines += 1;
                pieces.clear();
                let mut p = String::new();
                for w in line.split_whitespace() {
                    let next = if p.is_empty() { w.to_owned() } else { format!("{p} {w}") };
                    if count(&format!("{title}\n{next}")) > a.max_tokens && !p.is_empty() {
                        pieces.push(std::mem::take(&mut p));
                        p = w.to_owned();
                    } else {
                        p = next;
                    }
                }
                if !p.is_empty() {
                    pieces.push(p);
                }
            }
            for piece in pieces {
                let next = format!("{cur}\n{piece}");
                if count(&next) > a.max_tokens && cur != title {
                    chunks.push(std::mem::replace(&mut cur, format!("{title}\n{piece}")));
                } else {
                    cur = next;
                }
            }
        }
        if cur != title || chunks.is_empty() {
            chunks.push(cur);
        }
        for (i, t) in chunks.into_iter().enumerate() {
            rows.push(Chunk {
                chunk_id: format!("{sid}#{i:04}"),
                story_id: sid.clone(),
                group_id: group.clone(),
                ordinal: u32::try_from(i)?,
                scene_ordinal: 0,
                line_start: 0,
                line_end: 0,
                content_sha: sha_hex(t.as_bytes())[..16].to_owned(),
                token_count: count(&t),
                speakers: speakers.clone(),
                on_screen: Vec::new(),
                background: None,
                text: t,
                prefix: None,
            });
        }
    }
    let src = std::fs::read(a.corpus.join("chunks.jsonl"))?;
    std::fs::create_dir_all(&a.out)?;
    let mut out = std::io::BufWriter::new(std::fs::File::create(a.out.join("chunks.jsonl"))?);
    out.write_all(&src)?;
    for c in &rows {
        serde_json::to_writer(&mut out, c)?;
        out.write_all(b"\n")?;
    }
    out.flush()?;
    for f in ["manifest.p0.json", "spoiler.jsonl", "unresolved.jsonl"] {
        std::fs::copy(a.corpus.join(f), a.out.join(f)).with_context(|| format!("copying {f}"))?;
    }
    let mut by = std::collections::BTreeMap::new();
    for c in &rows {
        *by.entry(c.group_id.clone()).or_insert(0usize) += 1;
    }
    eprintln!(
        "units: {} chunks appended to {} ({by:?}; {split_lines} over-long lines cut by words; max {} tokens)",
        rows.len(),
        a.corpus.display(),
        rows.iter().map(|c| c.token_count).max().unwrap_or(0)
    );
    Ok(())
}
