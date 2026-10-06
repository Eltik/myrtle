//! Small helpers shared across the crate and its binaries: content hashes and JSON/JSONL file reading
//! and writing. Each helper keeps the exact semantics of the copies it replaced (which lines count, what a
//! missing file means, the error text), so a caller's behavior does not change by using it.

use std::io::Write as _;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};

/// sha256 of `bytes` as 64 lowercase hex characters.
#[must_use]
pub fn sha_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// sha256 of `s`, first 16 hex chars: the `content_sha` rule, shared with the embedder's reuse key and the
/// gold-set prompt hashes so they agree byte for byte.
#[must_use]
pub fn sha16(s: &str) -> String {
    sha_hex(s.as_bytes())[..16].to_owned()
}

/// One step of SplitMix64: advance `state` and return the next 64 random bits (the seeded generator of the gold-set
/// sampler and the bootstrap statistics).
pub fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A JSON file, or `None` when it cannot be read or does not parse.
#[must_use]
pub fn read_json_opt<T: DeserializeOwned>(path: &Path) -> Option<T> {
    serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

/// Every line of a JSONL file that parses (blank and malformed lines are skipped), or `None` when the file
/// cannot be read.
#[must_use]
pub fn read_jsonl_opt<T: DeserializeOwned>(path: &Path) -> Option<Vec<T>> {
    Some(std::fs::read_to_string(path).ok()?.lines().filter_map(|l| serde_json::from_str(l).ok()).collect())
}

/// Every line of a JSONL file, strictly: the file must be readable ("reading <path>") and every line, blank
/// ones included, must parse.
///
/// # Errors
/// An unreadable file or a line that does not parse.
pub fn read_jsonl_strict<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    Ok(std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?
        .lines()
        .map(serde_json::from_str)
        .collect::<std::result::Result<_, _>>()?)
}

/// Write `rows` as JSON lines to `path` through a temporary file next to it (`path` with extension
/// `tmp_ext`), renamed into place so a reader never sees a half-written file.
///
/// # Errors
/// I/O or serialization failures.
pub fn write_jsonl_atomic<T: Serialize>(path: &Path, tmp_ext: &str, rows: impl IntoIterator<Item = T>) -> Result<()> {
    let tmp = path.with_extension(tmp_ext);
    {
        let mut w = std::io::BufWriter::new(std::fs::File::create(&tmp)?);
        for r in rows {
            serde_json::to_writer(&mut w, &r)?;
            w.write_all(b"\n")?;
        }
        w.flush()?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_match_the_rules_they_replaced() {
        assert_eq!(sha_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(sha16("abc"), "ba7816bf8f01cfea");
        // The old byte-by-byte `content_sha` fold.
        let fold = Sha256::digest("Ch'en".as_bytes()).iter().take(8).map(|b| format!("{b:02x}")).collect::<String>();
        assert_eq!(sha16("Ch'en"), fold);
    }

    #[test]
    fn splitmix64_matches_the_reference_sequence() {
        // SplitMix64 from seed 0: the reference outputs (Vigna's splitmix64.c).
        let mut s = 0u64;
        assert_eq!(splitmix64(&mut s), 0xE220_A839_7B1D_CDAF);
        assert_eq!(splitmix64(&mut s), 0x6E78_9E6A_A1B9_65F4);
    }

    #[test]
    fn jsonl_readers_keep_their_line_rules() {
        let dir = std::env::temp_dir().join(format!("trevor-util-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("a.jsonl");
        std::fs::write(&p, "{\"a\": 1}\n\nnot json\n{\"a\": 2}\n").unwrap();
        let lossy: Vec<serde_json::Value> = read_jsonl_opt(&p).unwrap();
        assert_eq!(lossy.len(), 2);
        assert!(read_jsonl_strict::<serde_json::Value>(&p).is_err());
        assert!(read_jsonl_opt::<serde_json::Value>(&dir.join("missing.jsonl")).is_none());
        let e = read_jsonl_strict::<serde_json::Value>(&dir.join("missing.jsonl")).unwrap_err();
        assert!(e.to_string().starts_with("reading "));
        write_jsonl_atomic(&p, "jsonl.tmp", [serde_json::json!({"b": 1}), serde_json::json!({"b": 2})]).unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "{\"b\":1}\n{\"b\":2}\n");
        assert!(!p.with_extension("jsonl.tmp").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
