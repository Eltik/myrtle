//! Local eviction: freeing the bytes of a file once the bucket holds it.
//!
//! An evicted file is a STUB: the same path and inode, truncated to 0 bytes,
//! with its mtime PRESERVED, never unlinked. The backend reads the names of
//! files under the served roots to build its indexes and uses local existence
//! as the redirect oracle, the pipeline compares file counts between
//! extracts, and the unpacker skips expensive work when an output exists and
//! is newer than its source. A 0-byte file keeps every one of those working;
//! a deleted file breaks all of them. The mtime is the one the extract wrote,
//! already newer than the source, so the skip checks still pass; stamping it
//! with now instead would make a subtree look freshly written to the orphan
//! sweep (`orphans.mjs` `subtreeWasWritten`) during a filtered extract, which
//! then sweeps every older file under it. The manifest marks which
//! 0-byte files are stubs (`ManifestRecord::evicted`).
//!
//! Some files are read for their BYTES by the backend or the unpacker; those
//! are on the keep list and never evicted.

use std::path::Path;

use crate::config::{ConfigError, get as get_var};

/// Rel-path prefixes (under `<root>/<server>/`, forward slashes) that are
/// never evicted, because a local reader needs the bytes. `S3_EVICT_KEEP`
/// adds to this list and cannot remove from it.
///
/// One built-in rule is not a plain prefix and lives in
/// [`is_server_home_banner`]: `textures/[[<server>]]/arts/ui/homebanners/`
/// (the `home_banner_gacha_hub` gacha banner source, which
/// `core/release/art.rs` copies into `derived/release-art`).
pub const BUILTIN_KEEP: &[&str] = &[
    // Story sprites: hub.json, sprites.json, PNG headers, thumbnail renders,
    // gallery variants.
    "textures/avg/",
    // Story art variants.
    "textures/spritepack/cutin_char_",
    // `core/release/art.rs` copies these into `derived/release-art`.
    "textures/spritepack/ui_home_act_banner_gacha_",
    "textures/spritepack/ui_zone_home_theme_",
    "textures/arts/ui/stage/[uc]homeentry/",
    // `event_banner.rs` composes the generated event banners from the Story
    // Collection key visuals and titles.
    "textures/spritepack/mixstory_",
    // The unpacker reads it (`music_m4a.rs`).
    "audio/m4a_sources.json",
];

/// Whether `sync` evicts, and what it keeps on top of [`BUILTIN_KEEP`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EvictPolicy {
    /// `S3_EVICT_LOCAL`; off by default.
    pub enabled: bool,
    /// `S3_EVICT_KEEP`: extra rel-path prefixes to keep.
    pub extra_keep: Vec<String>,
}

impl EvictPolicy {
    /// `S3_EVICT_LOCAL` and `S3_EVICT_KEEP` through `lookup`; empty values
    /// count as unset. `S3_EVICT_LOCAL` is on only for `1`, `true`, `yes` or
    /// `on` (any case); every other value is off, so a typo never truncates.
    ///
    /// # Errors
    /// An `S3_EVICT_KEEP` entry containing a backslash.
    pub fn from_lookup(lookup: &dyn Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let enabled = get_var(lookup, "S3_EVICT_LOCAL").is_some_and(|v| {
            matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
        });
        let extra_keep = match get_var(lookup, "S3_EVICT_KEEP") {
            None => Vec::new(),
            Some(raw) => parse_keep(&raw)?,
        };
        Ok(Self {
            enabled,
            extra_keep,
        })
    }

    /// True when `rel` lies under a built-in or an extra keep prefix, or is a
    /// server-tagged home banner ([`is_server_home_banner`]).
    #[must_use]
    pub fn keeps(&self, rel: &str) -> bool {
        BUILTIN_KEEP.iter().any(|p| rel.starts_with(p))
            || is_server_home_banner(rel)
            || self.extra_keep.iter().any(|p| rel.starts_with(p.as_str()))
    }
}

/// `textures/[[<anything>]]/arts/ui/homebanners/...`: the gacha banners
/// under a server-tagged texture tree (only `[[en]]` exists today), read as
/// bytes by `core/release/art.rs`.
#[must_use]
pub fn is_server_home_banner(rel: &str) -> bool {
    rel.strip_prefix("textures/[[")
        .and_then(|rest| rest.split_once("]]/"))
        .is_some_and(|(_, tail)| tail.starts_with("arts/ui/homebanners/"))
}

/// Splits `S3_EVICT_KEEP`: comma separated, entries trimmed, a leading `/`
/// dropped, empty entries ignored.
///
/// # Errors
/// An entry with a backslash: keys and rels use forward slashes only.
pub fn parse_keep(raw: &str) -> Result<Vec<String>, ConfigError> {
    let mut out = Vec::new();
    for entry in raw.split(',') {
        let entry = entry.trim().trim_start_matches('/');
        if entry.is_empty() {
            continue;
        }
        if entry.contains('\\') {
            return Err(ConfigError::Invalid {
                var: "S3_EVICT_KEEP",
                value: raw.to_owned(),
                reason: "prefixes use forward slashes",
            });
        }
        out.push(entry.to_owned());
    }
    Ok(out)
}

/// Truncates the file at `path` to 0 bytes in place and puts its mtime back
/// to what it was.
///
/// Truncation would otherwise stamp it with now; see the module doc for why
/// that matters. The file is opened for writing with
/// truncation, never removed or recreated, so its inode and any open handle
/// stay valid. Blocking: call it from `spawn_blocking`.
///
/// # Errors
/// The file's metadata cannot be read, it cannot be opened for writing, or
/// its mtime cannot be set.
pub fn truncate_in_place(path: &Path) -> std::io::Result<()> {
    let modified = std::fs::metadata(path)?.modified()?;
    let file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)?;
    file.set_modified(modified)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn policy(pairs: &[(&str, &str)]) -> Result<EvictPolicy, ConfigError> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        EvictPolicy::from_lookup(&|k| map.get(k).cloned())
    }

    #[test]
    fn eviction_is_off_unless_switched_on() {
        assert_eq!(policy(&[]), Ok(EvictPolicy::default()));
        for off in ["", "  ", "0", "false", "no", "off", "enable", "2"] {
            assert!(
                !policy(&[("S3_EVICT_LOCAL", off)]).unwrap().enabled,
                "{off:?}"
            );
        }
        for on in ["1", "true", "TRUE", "yes", "On", " on "] {
            assert!(policy(&[("S3_EVICT_LOCAL", on)]).unwrap().enabled, "{on:?}");
        }
    }

    #[test]
    fn extra_keep_prefixes_parse() {
        let p = policy(&[("S3_EVICT_KEEP", " /textures/ui/ , ,audio/bgm/x.m4a,")]).unwrap();
        assert_eq!(p.extra_keep, ["textures/ui/", "audio/bgm/x.m4a"]);
        assert_eq!(
            policy(&[("S3_EVICT_KEEP", "")]).unwrap().extra_keep,
            Vec::<String>::new()
        );
        assert!(matches!(
            policy(&[("S3_EVICT_KEEP", "textures\\avg")]),
            Err(ConfigError::Invalid {
                var: "S3_EVICT_KEEP",
                ..
            })
        ));
    }

    #[test]
    fn the_keep_list_matches_by_prefix() {
        let p = EvictPolicy::default();
        for kept in [
            "textures/avg/characters/x.png",
            "textures/avg/hub.json",
            "textures/spritepack/cutin_char_amiya_1.png",
            "textures/spritepack/ui_home_act_banner_gacha_1.png",
            "textures/spritepack/ui_zone_home_theme_2.png",
            "textures/arts/ui/stage/[uc]homeentry/a.png",
            "textures/spritepack/mixstory_kv_0/kv_act17side.png",
            "audio/m4a_sources.json",
            "textures/[[en]]/arts/ui/homebanners/a.png",
            "textures/[[cn]]/arts/ui/homebanners/sub/b.png",
        ] {
            assert!(p.keeps(kept), "{kept}");
        }
        for evicted in [
            "textures/avgx/a.png",
            "textures/spritepack/ui_other.png",
            "textures/arts/ui/stage/[uc]mappreviewsspecial/a.png",
            "spine/DynIllust/x/[scene].json",
            "audio/sound_beta_2/m.m4a",
            "portraits/a.png",
            "textures/[[en]]/arts/ui/other/a.png",
            "textures/[[en]]/x/arts/ui/homebanners/a.png",
            "textures/arts/ui/homebanners/a.png",
            "textures/[[en]arts/ui/homebanners/a.png",
            "spine/[[en]]/arts/ui/homebanners/a.png",
        ] {
            assert!(!p.keeps(evicted), "{evicted}");
        }
        let extra = EvictPolicy {
            enabled: true,
            extra_keep: vec!["portraits/".into()],
        };
        assert!(extra.keeps("portraits/a.png"));
        assert!(extra.keeps("textures/avg/a.png"));
    }

    #[test]
    fn truncation_preserves_the_mtime() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.png");
        std::fs::write(&path, b"bytes").unwrap();
        let old = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
        std::fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(old)
            .unwrap();
        let before = std::fs::metadata(&path).unwrap().modified().unwrap();
        truncate_in_place(&path).unwrap();
        let after = std::fs::metadata(&path).unwrap();
        assert_eq!(after.len(), 0);
        assert_eq!(after.modified().unwrap(), before);
        assert_eq!(before, old);
    }

    #[test]
    fn truncation_keeps_the_inode() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.png");
        std::fs::write(&path, b"bytes").unwrap();
        #[cfg(unix)]
        let ino = {
            use std::os::unix::fs::MetadataExt;
            std::fs::metadata(&path).unwrap().ino()
        };
        truncate_in_place(&path).unwrap();
        let meta = std::fs::metadata(&path).unwrap();
        assert_eq!(meta.len(), 0);
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            assert_eq!(meta.ino(), ino);
        }
        assert!(truncate_in_place(&dir.path().join("missing.png")).is_err());
        assert!(!dir.path().join("missing.png").exists());
    }
}
