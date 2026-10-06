//! P3a: operator archives as a lore source.
//!
//! The operator files (profile, clinical analysis, archive files, promotion
//! record) hold about 888k tokens of lore that the story corpus never sees,
//! including 58 of the 197 lines in the game text that state a year. This
//! writes a corpus directory whose `chunks.jsonl` is the P0 file, byte for
//! byte, followed by one row per archive chunk. Each operator's sections are
//! packed in order into chunks of at most `--max-tokens`, every section keeping
//! its title; a section longer than that is split at line breaks. The P0
//! directory is never touched, and `search`/`eval` pick this one with
//! `--corpus`.
//!
//! Archive rows use story id `archive_<charId>` and group id `archive`; their
//! line span is the section index range, which never overlaps a gold anchor
//! (anchors name story ids), so the gold set measures only whether the extra
//! rows push story passages out of the top 10.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use tokenizers::Tokenizer;
use trevor::corpus::chunk::Chunk;
use trevor::corpus::derived;
use trevor::util::sha_hex;

#[derive(Parser)]
#[command(about = "Append operator archive chunks to a copy of the P0 corpus")]
struct Args {
    #[arg(long, default_value = "artifacts")]
    corpus: PathBuf,
    #[arg(long, default_value = "../assets/output/en/gamedata")]
    gamedata: PathBuf,
    #[arg(long, default_value = "models/gte-modernbert-base/tokenizer.json")]
    tokenizer: PathBuf,
    #[arg(long, default_value = "artifacts/p3a")]
    out: PathBuf,
    /// Largest chunk, in model tokens including the sequence's special tokens.
    #[arg(long, default_value_t = 600)]
    max_tokens: u32,
}

#[derive(Deserialize)]
struct Handbook {
    #[serde(rename = "HandbookDict")]
    dict: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    value: EntryValue,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct EntryValue {
    #[serde(rename = "CharID")]
    char_id: String,
    #[serde(default)]
    story_text_audio: Vec<Section>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Section {
    story_title: Option<String>,
    #[serde(default)]
    stories: Vec<SectionText>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SectionText {
    story_text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Meta {
    source_chunks_sha: String,
    handbook_sha: String,
    tokenizer_sha: String,
    operators: usize,
    archive_chunks: usize,
    archive_tokens: u64,
    max_chunk_tokens: u32,
    oversize_single_lines: usize,
}

struct Packer<'a> {
    count: &'a dyn Fn(&str) -> u32,
    name: String,
    char_id: String,
    ordinal: u32,
    rows: Vec<Chunk>,
}

impl Packer<'_> {
    fn render(&self, units: &[(u32, String)]) -> String {
        let body: Vec<&str> = units.iter().map(|(_, t)| t.as_str()).collect();
        format!("Operator archive: {}\n{}", self.name, body.join("\n"))
    }

    fn flush(&mut self, cur: &mut Vec<(u32, String)>) {
        if cur.is_empty() {
            return;
        }
        let text = self.render(cur);
        let n = (self.count)(&text);
        self.rows.push(Chunk {
            scene_ordinal: cur[0].0,
            line_start: cur[0].0,
            line_end: cur[cur.len() - 1].0,
            ..derived::generated_chunk(format!("archive_{}#{:04}", self.char_id, self.ordinal), format!("archive_{}", self.char_id),
                                       "archive".to_owned(), self.ordinal, vec![self.name.clone()], text, n)
        });
        self.ordinal += 1;
        cur.clear();
    }
}

fn main() -> Result<()> {
    // SAFETY: set before any other thread exists.
    unsafe { std::env::set_var("TOKENIZERS_PARALLELISM", "false") };
    let a = Args::parse();
    derived::ensure_distinct_out(&a.out, &a.corpus)?;
    let tok_bytes = std::fs::read(&a.tokenizer).with_context(|| format!("reading {}", a.tokenizer.display()))?;
    let tok = Tokenizer::from_bytes(&tok_bytes).map_err(|e| anyhow::anyhow!("{e}"))?;
    let count = |s: &str| derived::token_count(&tok, s);

    let hb_path = a.gamedata.join("excel/handbook_info_table.json");
    let hb_bytes = std::fs::read(&hb_path).with_context(|| format!("reading {}", hb_path.display()))?;
    let hb: Handbook = serde_json::from_slice(&hb_bytes).context("handbook_info_table.json")?;
    let ct: serde_json::Value = serde_json::from_slice(&std::fs::read(a.gamedata.join("excel/character_table.json"))?)?;
    let mut names: HashMap<String, String> = HashMap::new();
    for c in ct["Characters"].as_array().context("character_table Characters")? {
        if let (Some(k), Some(n)) = (c["key"].as_str(), c["value"]["Name"].as_str()) {
            names.insert(k.to_owned(), n.to_owned());
        }
    }

    let src = std::fs::read(a.corpus.join("chunks.jsonl"))?;
    let (mut ops, mut oversize) = (0usize, 0usize);
    let mut rows: Vec<Chunk> = Vec::new();
    for e in &hb.dict {
        let v = &e.value;
        let name = names.get(&v.char_id).cloned().unwrap_or_else(|| v.char_id.clone());
        // Units in order, each "Title: text"; a section over the cap is split at line breaks.
        let mut units: Vec<(u32, String)> = Vec::new();
        for (i, sec) in v.story_text_audio.iter().enumerate() {
            let title = sec.story_title.clone().unwrap_or_default();
            let body = sec
                .stories
                .iter()
                .map(|s| s.story_text.trim())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            if body.is_empty() {
                continue;
            }
            let i = u32::try_from(i).unwrap_or(u32::MAX);
            let whole = format!("{title}: {body}");
            if count(&whole) <= a.max_tokens {
                units.push((i, whole));
            } else {
                for (j, line) in body.lines().filter(|l| !l.trim().is_empty()).enumerate() {
                    let t = if j == 0 { format!("{title}: {line}") } else { format!("{title} (cont.): {line}") };
                    if count(&t) > a.max_tokens {
                        oversize += 1;
                    }
                    units.push((i, t));
                }
            }
        }
        if units.is_empty() {
            continue;
        }
        ops += 1;
        let mut p = Packer { count: &count, name, char_id: v.char_id.clone(), ordinal: 0, rows: Vec::new() };
        let mut cur: Vec<(u32, String)> = Vec::new();
        for u in units {
            cur.push(u);
            if cur.len() > 1 && count(&p.render(&cur)) > a.max_tokens {
                let last = cur.pop().expect("just pushed");
                p.flush(&mut cur);
                cur.push(last);
            }
        }
        p.flush(&mut cur);
        rows.extend(p.rows);
    }

    derived::write_appended(&a.out, &src, &rows)?;
    derived::copy_sidecars(&a.corpus, &a.out)?;
    let meta = Meta {
        source_chunks_sha: sha_hex(&src),
        handbook_sha: sha_hex(&hb_bytes),
        tokenizer_sha: sha_hex(&tok_bytes),
        operators: ops,
        archive_chunks: rows.len(),
        archive_tokens: rows.iter().map(|c| u64::from(c.token_count)).sum(),
        max_chunk_tokens: rows.iter().map(|c| c.token_count).max().unwrap_or(0),
        oversize_single_lines: oversize,
    };
    std::fs::write(a.out.join("archives.meta.json"), serde_json::to_vec_pretty(&meta)?)?;
    eprintln!("{}", serde_json::to_string_pretty(&meta)?);
    Ok(())
}
