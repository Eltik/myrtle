//! What an extract could not read or write.
//!
//! The asset exporters report how many files they wrote and drop the rest: a
//! bundle that cannot be read is skipped, a write that fails is ignored, and the
//! run still ends `Exported N assets` with exit 0. `run.mjs` reads exit 0 as a
//! clean extract and then sweeps every file in the tree this run did not
//! rewrite, so on a disk that started failing part-way, exit 0 is an
//! instruction to delete the last good copy of each file the run just failed to
//! write. The gamedata step already refuses to report success for this reason
//! (`cmd_extract`); this module is the same rule for everything else.
//!
//! Failures are counted here, process-wide, because the exporters run on the
//! rayon pool and return counts, not results. `cmd_extract` reads [`counts`]
//! after the last exporter and exits non-zero when anything could not be read
//! or written.
//!
//! A bundle that reads but does not PARSE is counted and reported, not fatal:
//! that is a property of the bundle, the same on every run, not a sign that
//! this run lost its disk.

use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

static UNREADABLE: AtomicUsize = AtomicUsize::new(0);
static UNPARSED: AtomicUsize = AtomicUsize::new(0);
static UNWRITTEN: AtomicUsize = AtomicUsize::new(0);

/// How many failures of each kind are printed as they happen. Past it only the
/// totals are reported: a dead disk fails every file the same way, and `run.mjs`
/// keeps the last lines of output, which should be the summary.
const SHOWN: usize = 20;

/// A source bundle could not be read from disk.
pub fn read_failed(path: &Path, err: &io::Error) {
    if UNREADABLE.fetch_add(1, Ordering::Relaxed) < SHOWN {
        eprintln!("  error reading {}: {err}", path.display());
    }
}

/// A source bundle was read but is not a bundle this exporter can parse.
pub fn parse_failed(path: &Path) {
    if UNPARSED.fetch_add(1, Ordering::Relaxed) < SHOWN {
        eprintln!("  skipped (not a readable bundle): {}", path.display());
    }
}

/// An output file could not be written.
pub fn write_failed(path: &Path, err: &io::Error) {
    if UNWRITTEN.fetch_add(1, Ordering::Relaxed) < SHOWN {
        eprintln!("  error writing {}: {err}", path.display());
    }
}

/// `std::fs::write`, with a failure counted. Same arguments and result, so an
/// exporter that ignores the result still leaves the failure on record.
///
/// # Errors
/// Whatever `std::fs::write` returns.
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
    let path = path.as_ref();
    let result = std::fs::write(path, contents);
    if let Err(err) = &result {
        write_failed(path, err);
    }
    result
}

/// The run's failures so far.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    /// Source bundles that could not be read.
    pub unreadable: usize,
    /// Source bundles that were read but did not parse.
    pub unparsed: usize,
    /// Output files that could not be written.
    pub unwritten: usize,
}

impl Counts {
    /// Whether the tree may be missing files this run should have rewritten,
    /// which is when the run must not report success.
    #[must_use]
    pub const fn lost_files(self) -> bool {
        self.unreadable > 0 || self.unwritten > 0
    }
}

/// What has been counted since the process started.
#[must_use]
pub fn counts() -> Counts {
    Counts {
        unreadable: UNREADABLE.load(Ordering::Relaxed),
        unparsed: UNPARSED.load(Ordering::Relaxed),
        unwritten: UNWRITTEN.load(Ordering::Relaxed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The counters are process-wide and other tests may fail writes of their own,
    // so these assert on what one call adds at least, never on absolute totals.

    #[test]
    fn a_write_that_succeeds_is_not_counted_as_lost() {
        let dir = std::env::temp_dir().join(format!("unpacker-failures-ok-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("out.bin");
        write(&path, b"bytes").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"bytes");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_write_that_fails_is_returned_and_counted() {
        let before = counts().unwritten;
        // The parent does not exist, so the write cannot succeed.
        let path = std::env::temp_dir()
            .join(format!("unpacker-failures-missing-{}", std::process::id()))
            .join("out.bin");
        assert!(write(&path, b"bytes").is_err());
        let after = counts();
        assert!(after.unwritten > before);
        assert!(after.lost_files());
    }

    #[test]
    fn an_unparsed_bundle_alone_does_not_fail_the_run() {
        let only_unparsed = Counts {
            unreadable: 0,
            unparsed: 3,
            unwritten: 0,
        };
        assert!(!only_unparsed.lost_files());
        let unreadable = Counts {
            unreadable: 1,
            ..only_unparsed
        };
        assert!(unreadable.lost_files());
    }
}
