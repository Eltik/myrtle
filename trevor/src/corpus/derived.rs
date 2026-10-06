//! What the corpus builders share when they write a derived corpus directory next to their input
//! (`build-prefixes` P1a, `build-archives` P3a, `build-profiles` P3b, `build-units` P4): the guard against
//! writing over the input, the stored token count, the generated-chunk shape, and the copy of the input's
//! `chunks.jsonl` with rows appended plus its sidecar files.

use std::io::Write as _;
use std::path::Path;

use anyhow::{Context, Result, bail};
use tokenizers::Tokenizer;

use crate::corpus::chunk::{Chunk, sha16};

/// The files of a corpus directory every derived corpus copies unchanged.
pub const SIDECARS: [&str; 3] = ["manifest.p0.json", "spoiler.jsonl", "unresolved.jsonl"];

/// Fail when `out` is the input corpus directory itself.
///
/// # Errors
/// `corpus` cannot be resolved, or `out` is the same directory.
pub fn ensure_distinct_out(out: &Path, corpus: &Path) -> Result<()> {
    if out.canonicalize().ok() == Some(corpus.canonicalize()?) {
        bail!("--out must not be the input corpus directory");
    }
    Ok(())
}

/// The stored token count of a generated chunk: the whole-text encode with the special tokens, as for story
/// chunks; `u32::MAX` when the text does not encode.
#[must_use]
pub fn token_count(tok: &Tokenizer, s: &str) -> u32 {
    tok.encode(s, true).map_or(u32::MAX, |e| u32::try_from(e.get_ids().len()).unwrap_or(u32::MAX))
}

/// A generated chunk (one not cut from a story script): no scene, line range, sprites or background, and
/// `content_sha` from its text.
#[must_use]
pub fn generated_chunk(chunk_id: String, story_id: String, group_id: String, ordinal: u32, speakers: Vec<String>,
                       text: String, token_count: u32) -> Chunk {
    Chunk {
        chunk_id,
        story_id,
        group_id,
        ordinal,
        scene_ordinal: 0,
        line_start: 0,
        line_end: 0,
        content_sha: sha16(&text),
        token_count,
        speakers,
        on_screen: Vec::new(),
        background: None,
        text,
        prefix: None,
    }
}

/// Write `<out>/chunks.jsonl` as the input's bytes `src` followed by `rows`, one JSON line each.
///
/// # Errors
/// I/O or serialization failures.
pub fn write_appended(out: &Path, src: &[u8], rows: &[Chunk]) -> Result<()> {
    std::fs::create_dir_all(out)?;
    let mut w = std::io::BufWriter::new(std::fs::File::create(out.join("chunks.jsonl"))?);
    w.write_all(src)?;
    for c in rows {
        serde_json::to_writer(&mut w, c)?;
        w.write_all(b"\n")?;
    }
    w.flush()?;
    Ok(())
}

/// Copy the [`SIDECARS`] from `corpus` to `out`.
///
/// # Errors
/// A sidecar that cannot be copied ("copying <name>").
pub fn copy_sidecars(corpus: &Path, out: &Path) -> Result<()> {
    for f in SIDECARS {
        std::fs::copy(corpus.join(f), out.join(f)).with_context(|| format!("copying {f}"))?;
    }
    Ok(())
}
