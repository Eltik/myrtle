//! Crash-safe file replacement for writers whose output is trusted once it
//! exists: a later pass skips it, or it is the only copy of its data.

use std::io::Write;
use std::path::Path;

/// Replace `path` with `bytes` so that a reader, or the next run after a crash,
/// sees either the old file or the whole new one, never a prefix.
///
/// Writes `<path>.tmp`, fsyncs it, renames it over `path`, then fsyncs the parent
/// directory so the rename itself survives a power loss. Without the two fsyncs a
/// crash can persist the rename before the data, leaving a zero-length file at
/// `path`. On any failure before the rename the temp file is removed and the
/// error returned, and `path` is untouched. The parent directory must exist.
pub fn write_durable(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut tmp_name = path.as_os_str().to_os_string();
    tmp_name.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp_name);
    let written = std::fs::File::create(&tmp).and_then(|mut f| {
        f.write_all(bytes)?;
        f.sync_all()
    });
    if let Err(e) = written.and_then(|()| std::fs::rename(&tmp, path)) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    // The new file is in place from here on; a failed directory fsync is
    // reported to the caller but there is nothing left to clean up.
    #[cfg(unix)]
    {
        let dir = match path.parent() {
            Some(p) if !p.as_os_str().is_empty() => p,
            _ => Path::new("."),
        };
        std::fs::File::open(dir)?.sync_all()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::write_durable;

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "myrtle_write_durable_{}_{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn tmp_of(path: &std::path::Path) -> std::path::PathBuf {
        let mut name = path.as_os_str().to_os_string();
        name.push(".tmp");
        name.into()
    }

    #[test]
    fn writes_a_new_file_and_leaves_no_temp() {
        let dir = scratch("new");
        let path = dir.join("a.json");
        write_durable(&path, b"{\"a\":1}").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"{\"a\":1}");
        assert!(!tmp_of(&path).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn replaces_an_existing_file() {
        let dir = scratch("replace");
        let path = dir.join("b.png");
        std::fs::write(&path, b"old bytes, longer than the new ones").unwrap();
        write_durable(&path, b"new").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"new");
        assert!(!tmp_of(&path).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_failed_rename_removes_the_temp_and_keeps_the_target() {
        let dir = scratch("fail");
        // A non-empty directory at the target path: the temp file writes, the
        // rename over a directory fails.
        let path = dir.join("c.json");
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("keep"), b"k").unwrap();
        assert!(write_durable(&path, b"x").is_err());
        assert!(!tmp_of(&path).exists());
        assert_eq!(std::fs::read(path.join("keep")).unwrap(), b"k");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
