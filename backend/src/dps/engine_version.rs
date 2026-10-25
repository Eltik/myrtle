//! The engine's identity, for the DPS cache keys (`dps:calc:*`, `dps:list:*`,
//! `dps:rank:*`): a hash over every source file the engine is built from,
//! computed by `build.rs` at compile time and read as [`super::ENGINE_VERSION`].
//!
//! The keys already carry the game-data identity, so a data reload cannot serve
//! an old result. An engine change could: the rank key had no code slot at all
//! (24 h TTL), and the calculate keys relied on a hand-bumped `kind` (`dps3`,
//! `dpsbatch1`), which only works when someone remembers. Redis survives a
//! deploy and the only `dps:` flush is the asset watcher's, on a data reload.
//! With this hash in the key, any edit to an input below is a new key space on
//! the next build, with nothing to remember.
//!
//! Shared with `build.rs` through `#[path]`, so it uses `std` only.

use std::io;
use std::path::{Path, PathBuf};

/// What the engine is built from, relative to the crate root: the engine
/// (formulas, its JSON config tables, zones, supporters, the operator cache)
/// and the service that maps a request onto it and builds the rank rows. The
/// engine reads nothing else from the crate but the game-data types, whose
/// content the game-data identity already covers.
pub const INPUTS: &[&str] = &["src/dps", "src/app/services/dps.rs"];

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv(mut h: u64, bytes: &[u8]) -> u64 {
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

fn collect(path: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            collect(&entry?.path(), out)?;
        }
    } else {
        out.push(path.to_path_buf());
    }
    Ok(())
}

/// FNV-1a 64 over every file under [`INPUTS`] in path order: its path relative
/// to `root` and its bytes, so an edit, a new file and a rename each change it.
/// 16 hex digits. Not a security hash: it only has to change when the engine
/// does.
///
/// # Errors
/// When an input cannot be read.
pub fn hash_inputs(root: &Path) -> io::Result<String> {
    let mut files = Vec::new();
    for input in INPUTS {
        collect(&root.join(input), &mut files)?;
    }
    files.sort();
    let mut h = FNV_OFFSET;
    for file in &files {
        let rel = file.strip_prefix(root).unwrap_or(file);
        h = fnv(h, rel.to_string_lossy().replace('\\', "/").as_bytes());
        h = fnv(h, &[0]);
        h = fnv(h, &std::fs::read(file)?);
        h = fnv(h, &[0xff]);
    }
    Ok(format!("{h:016x}"))
}

#[cfg(test)]
mod tests {
    use super::{INPUTS, hash_inputs};
    use std::fs;
    use std::path::Path;

    fn tree() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("src/dps/config")).expect("mkdir");
        fs::create_dir_all(dir.path().join("src/app/services")).expect("mkdir");
        fs::write(dir.path().join("src/dps/engine.rs"), "fn a() {}").expect("write");
        fs::write(dir.path().join("src/dps/config/f.json"), "{}").expect("write");
        fs::write(dir.path().join("src/app/services/dps.rs"), "fn b() {}").expect("write");
        dir
    }

    #[test]
    fn the_compiled_version_is_the_hash_of_the_sources_on_disk() {
        // `build.rs` reruns when an input changes (`rerun-if-changed`); a stale
        // build script output would leave old keys after an engine edit.
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        assert_eq!(
            crate::dps::ENGINE_VERSION,
            hash_inputs(root).expect("engine sources")
        );
        assert_eq!(crate::dps::ENGINE_VERSION.len(), 16);
    }

    #[test]
    fn an_edit_a_new_file_or_a_rename_changes_the_version() {
        let dir = tree();
        let root = dir.path();
        let base = hash_inputs(root).expect("hash");
        assert_eq!(base, hash_inputs(root).expect("hash"), "deterministic");

        fs::write(root.join("src/dps/config/f.json"), "{\"x\":1}").expect("write");
        let edited = hash_inputs(root).expect("hash");
        assert_ne!(edited, base, "a config table edit");

        fs::write(root.join("src/app/services/dps.rs"), "fn b() { 1; }").expect("write");
        let service = hash_inputs(root).expect("hash");
        assert_ne!(service, edited, "a service edit");

        fs::write(root.join("src/dps/zones.rs"), "").expect("write");
        let added = hash_inputs(root).expect("hash");
        assert_ne!(added, service, "a new, even empty, file");

        fs::rename(root.join("src/dps/zones.rs"), root.join("src/dps/zone.rs")).expect("rename");
        assert_ne!(hash_inputs(root).expect("hash"), added, "a rename");
    }

    #[test]
    fn a_file_outside_the_inputs_does_not_change_it() {
        let dir = tree();
        let root = dir.path();
        let base = hash_inputs(root).expect("hash");
        fs::write(root.join("src/app/services/base.rs"), "fn c() {}").expect("write");
        assert_eq!(hash_inputs(root).expect("hash"), base);
        assert_eq!(INPUTS.len(), 2);
    }
}
