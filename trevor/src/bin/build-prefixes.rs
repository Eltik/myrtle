//! P1a: deterministic contextual prefixes, no model.
//!
//! Writes a new corpus directory whose `chunks.jsonl` is the P0 file with a
//! `prefix` on every chunk. `embed-corpus` and `build-index` then run on that
//! directory unchanged, and `search`/`eval` pick it with `--corpus`. The P0
//! directory is never touched.
//!
//! The prefix follows corpus spec section 8 in its order, minus two parts:
//! group and story (name, `StoryCode`, `AvgTag`), position in the arc (story
//! N of M in the group), speakers present, and the game's own synopsis of the
//! story (`StoryInfo`, the archive blurb), which stands in for "one clause of
//! what is happening" at story rather than chunk granularity. Location is left
//! out: the background plates are mostly generic (`bg_black` on 536 chunks,
//! `bg_corridor` 302, `bg_room_2` 227). Referent resolution needs a model and
//! is P1b.

use std::collections::HashMap;
use std::io::{BufRead as _, Write as _};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use trevor::corpus::chunk::Chunk;

#[derive(Parser)]
#[command(about = "Write a corpus directory whose chunks carry deterministic P1a prefixes")]
struct Args {
    /// The P0 corpus directory (chunks.jsonl, manifest.p0.json, spoiler.jsonl).
    #[arg(long, default_value = "artifacts")]
    corpus: PathBuf,
    /// The unpacked EN gamedata root (excel/story_review_table.json and story/).
    #[arg(long, default_value = "../assets/output/en/gamedata")]
    gamedata: PathBuf,
    /// Output corpus directory. Must not be the input.
    #[arg(long, default_value = "artifacts/p1a")]
    out: PathBuf,
    /// Leave out the group, story name and code line.
    #[arg(long)]
    no_story: bool,
    /// Leave out "story N of M".
    #[arg(long)]
    no_position: bool,
    /// Leave out the speakers line.
    #[arg(long)]
    no_speakers: bool,
    /// Leave out the game's story synopsis.
    #[arg(long)]
    no_synopsis: bool,
}

/// At most this many speakers are listed, in first-appearance order.
const MAX_SPEAKERS: usize = 10;

#[derive(Deserialize)]
struct ReviewTable {
    #[serde(rename = "Story_reviews")]
    reviews: Vec<ReviewEntry>,
}

#[derive(Deserialize)]
struct ReviewEntry {
    value: ReviewGroup,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ReviewGroup {
    name: Option<String>,
    info_unlock_datas: Vec<ReviewStory>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ReviewStory {
    story_id: String,
    story_name: Option<String>,
    story_code: Option<String>,
    avg_tag: Option<String>,
    story_sort: Option<i64>,
    story_info: Option<String>,
}

struct StoryFacts {
    group: Option<String>,
    name: Option<String>,
    code: Option<String>,
    tag: Option<String>,
    position: (usize, usize),
    synopsis: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PrefixMeta {
    source_chunks_sha: String,
    story_review_sha: String,
    parts: Vec<&'static str>,
    chunks: usize,
    chunks_without_story_facts: usize,
    chunks_without_synopsis: usize,
    prefix_chars_p50: usize,
    prefix_chars_max: usize,
}

fn sha_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn one_line(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The unpacker writes `name.txt` as `name.txt.txt`; accept either.
fn read_synopsis(gamedata: &Path, info: &str) -> Option<String> {
    let base = gamedata.join("story").join(format!("[uc]{info}.txt"));
    let alt = PathBuf::from(format!("{}.txt", base.display()));
    [base, alt]
        .iter()
        .find_map(|p| std::fs::read_to_string(p).ok())
        .map(|s| one_line(&s))
        .filter(|s| !s.is_empty())
}

fn story_facts(gamedata: &Path) -> Result<(HashMap<String, StoryFacts>, String)> {
    let path = gamedata.join("excel/story_review_table.json");
    let bytes = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
    let table: ReviewTable = serde_json::from_slice(&bytes).context("story_review_table.json")?;
    let mut out = HashMap::new();
    for e in table.reviews {
        let mut stories = e.value.info_unlock_datas;
        stories.sort_by_key(|s| s.story_sort.unwrap_or(i64::MAX));
        let m = stories.len();
        for (i, s) in stories.into_iter().enumerate() {
            let synopsis = s.story_info.as_deref().and_then(|p| read_synopsis(gamedata, p));
            out.insert(
                s.story_id,
                StoryFacts {
                    group: e.value.name.clone(),
                    name: s.story_name,
                    code: s.story_code,
                    tag: s.avg_tag,
                    position: (i + 1, m),
                    synopsis,
                },
            );
        }
    }
    Ok((out, sha_hex(&bytes)))
}

fn prefix(a: &Args, c: &Chunk, f: Option<&StoryFacts>) -> Option<String> {
    let mut lines = Vec::new();
    if let Some(f) = f {
        if !a.no_story {
            let title = [f.code.as_deref(), f.name.as_deref()]
                .into_iter()
                .flatten()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            let mut l = [f.group.as_deref().map(str::trim), Some(title.as_str())]
                .into_iter()
                .flatten()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(", ");
            if let Some(t) = f.tag.as_deref().filter(|t| !t.is_empty()) {
                l.push_str(&format!(" ({t})"));
            }
            if !l.is_empty() {
                lines.push(l);
            }
        }
        if !a.no_position {
            lines.push(format!("Story {} of {}", f.position.0, f.position.1));
        }
    }
    if !a.no_speakers && !c.speakers.is_empty() {
        let names: Vec<&str> = c.speakers.iter().take(MAX_SPEAKERS).map(String::as_str).collect();
        lines.push(format!("Speakers: {}", names.join(", ")));
    }
    if let Some(s) = f.and_then(|f| f.synopsis.as_deref()).filter(|_| !a.no_synopsis) {
        lines.push(format!("Synopsis: {s}"));
    }
    (!lines.is_empty()).then(|| lines.join("\n"))
}

fn main() -> Result<()> {
    let a = Args::parse();
    if a.out.canonicalize().ok() == Some(a.corpus.canonicalize()?) {
        bail!("--out must not be the input corpus directory");
    }
    let chunks_path = a.corpus.join("chunks.jsonl");
    let bytes = std::fs::read(&chunks_path).with_context(|| format!("reading {}", chunks_path.display()))?;
    let source_chunks_sha = sha_hex(&bytes);
    let (facts, story_review_sha) = story_facts(&a.gamedata)?;

    std::fs::create_dir_all(&a.out)?;
    let mut out = std::io::BufWriter::new(std::fs::File::create(a.out.join("chunks.jsonl"))?);
    let (mut n, mut no_facts, mut no_synopsis) = (0usize, 0usize, 0usize);
    let mut lens = Vec::new();
    for (i, line) in bytes.lines().enumerate() {
        let mut c: Chunk = serde_json::from_str(&line?).with_context(|| format!("chunks.jsonl line {}", i + 1))?;
        if c.prefix.is_some() {
            bail!("{} already has a prefix; build from the P0 corpus", c.chunk_id);
        }
        let f = facts.get(&c.story_id);
        no_facts += usize::from(f.is_none());
        no_synopsis += usize::from(f.and_then(|f| f.synopsis.as_ref()).is_none());
        c.prefix = prefix(&a, &c, f);
        lens.push(c.prefix.as_ref().map_or(0, String::len));
        serde_json::to_writer(&mut out, &c)?;
        out.write_all(b"\n")?;
        n += 1;
    }
    out.flush()?;
    for f in ["manifest.p0.json", "spoiler.jsonl", "unresolved.jsonl"] {
        std::fs::copy(a.corpus.join(f), a.out.join(f)).with_context(|| format!("copying {f}"))?;
    }
    lens.sort_unstable();
    let parts = [
        (!a.no_story, "story"),
        (!a.no_position, "position"),
        (!a.no_speakers, "speakers"),
        (!a.no_synopsis, "synopsis"),
    ]
    .into_iter()
    .filter_map(|(on, p)| on.then_some(p))
    .collect();
    let meta = PrefixMeta {
        source_chunks_sha,
        story_review_sha,
        parts,
        chunks: n,
        chunks_without_story_facts: no_facts,
        chunks_without_synopsis: no_synopsis,
        prefix_chars_p50: lens.get(lens.len() / 2).copied().unwrap_or(0),
        prefix_chars_max: lens.last().copied().unwrap_or(0),
    };
    std::fs::write(a.out.join("prefixes.meta.json"), serde_json::to_vec_pretty(&meta)?)?;
    eprintln!("{}", serde_json::to_string_pretty(&meta)?);
    Ok(())
}
