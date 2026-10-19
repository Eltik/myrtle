//! The resumable checkpoint: `vectors.ckpt` (one record per embedded row) and its `vectors.ckpt.json` head.

use std::collections::HashMap;
use std::io::{BufWriter, Read as _, Write as _};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const CKPT_FILE: &str = "vectors.ckpt";
const CKPT_HEAD: &str = "vectors.ckpt.json";
const KEY_LEN: usize = 16;

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct CkptHead {
    pub(super) model_sha: String,
    pub(super) tokenizer_sha: String,
    pub(super) dtype: String,
    pub(super) pooling: String,
    pub(super) batch_size: usize,
}

/// Parse checkpoint records from `bytes`: the rows by key, the length of the
/// valid prefix, and the dim. The first record that is cut short, has a key
/// that is not 16 lowercase hex characters, a dim other than the first
/// record's, or a vector that is not unit length ends the prefix.
pub(super) fn parse_checkpoint(bytes: &[u8]) -> (HashMap<String, Vec<u8>>, usize, Option<usize>) {
    let mut rows = HashMap::new();
    let mut at = 0usize;
    let mut dim: Option<usize> = None;
    while bytes.len() >= at + KEY_LEN + 4 {
        let key = &bytes[at..at + KEY_LEN];
        if !key.iter().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b)) {
            break;
        }
        let d = u32::from_le_bytes([
            bytes[at + KEY_LEN],
            bytes[at + KEY_LEN + 1],
            bytes[at + KEY_LEN + 2],
            bytes[at + KEY_LEN + 3],
        ]) as usize;
        if d == 0 || dim.is_some_and(|x| x != d) {
            break;
        }
        let start = at + KEY_LEN + 4;
        let Some(row) = bytes.get(start..start + d * 4) else {
            break;
        };
        let norm = row
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .map(|x| x * x)
            .sum::<f32>()
            .sqrt();
        if !norm.is_finite() || (norm - 1.0).abs() > 1e-3 {
            break;
        }
        dim = Some(d);
        rows.insert(String::from_utf8_lossy(key).into_owned(), row.to_vec());
        at = start + d * 4;
    }
    (rows, at, dim)
}

/// The append side of the checkpoint.
pub(super) struct Checkpoint {
    dir: PathBuf,
    file: BufWriter<std::fs::File>,
    every: usize,
    since_sync: usize,
}

impl Checkpoint {
    /// Open `dir`'s checkpoint for this model: the rows a previous run left
    /// (a checkpoint of another model or tokenizer is discarded), with the
    /// file cut back to its valid prefix and opened for appending.
    pub(super) fn open(dir: &Path, head: &CkptHead, every: usize) -> Result<(Self, HashMap<String, Vec<u8>>)> {
        let head_path = dir.join(CKPT_HEAD);
        let ckpt_path = dir.join(CKPT_FILE);
        let same = crate::util::read_json_opt::<CkptHead>(&head_path).is_some_and(|h| h == *head);
        let mut rows = HashMap::new();
        if same {
            let mut bytes = Vec::new();
            match std::fs::File::open(&ckpt_path) {
                Ok(mut f) => {
                    f.read_to_end(&mut bytes)?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e).context("reading vectors.ckpt"),
            }
            let (r, valid, _) = parse_checkpoint(&bytes);
            if valid < bytes.len() {
                eprintln!(
                    "checkpoint: kept {} rows, cut {} trailing bytes",
                    r.len(),
                    bytes.len() - valid
                );
                std::fs::OpenOptions::new()
                    .write(true)
                    .open(&ckpt_path)?
                    .set_len(valid as u64)?;
            }
            rows = r;
        } else {
            if head_path.exists() {
                eprintln!("checkpoint: discarding one built by another model or tokenizer");
            }
            match std::fs::remove_file(&ckpt_path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e).context("removing vectors.ckpt"),
            }
            let tmp = dir.join("vectors.ckpt.json.tmp");
            std::fs::write(&tmp, serde_json::to_vec_pretty(head)?)?;
            std::fs::rename(&tmp, &head_path)?;
        }
        let f = std::fs::OpenOptions::new().create(true).append(true).open(&ckpt_path)?;
        Ok((
            Self {
                dir: dir.to_path_buf(),
                file: BufWriter::new(f),
                every: every.max(1),
                since_sync: 0,
            },
            rows,
        ))
    }

    /// Append one row; it reaches the OS at once (a killed process loses
    /// nothing) and the disk every `every` rows.
    pub(super) fn append(&mut self, key: &str, v: &[f32]) -> Result<()> {
        debug_assert_eq!(key.len(), KEY_LEN);
        let dim = u32::try_from(v.len()).context("dim over u32")?;
        self.file.write_all(key.as_bytes())?;
        self.file.write_all(&dim.to_le_bytes())?;
        for x in v {
            self.file.write_all(&x.to_le_bytes())?;
        }
        self.file.flush()?;
        self.since_sync += 1;
        if self.since_sync >= self.every {
            self.sync()?;
        }
        Ok(())
    }

    pub(super) fn sync(&mut self) -> Result<()> {
        self.file.flush()?;
        self.file.get_ref().sync_data()?;
        self.since_sync = 0;
        Ok(())
    }

    /// The run is complete: the checkpoint is no longer needed.
    pub(super) fn finish(self) -> Result<()> {
        drop(self.file);
        for name in [CKPT_FILE, CKPT_HEAD] {
            match std::fs::remove_file(self.dir.join(name)) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e).with_context(|| format!("removing {name}")),
            }
        }
        Ok(())
    }
}
