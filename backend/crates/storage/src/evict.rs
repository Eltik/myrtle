//! Local eviction: clears files once uploaded to bucket

use std::fs::{OpenOptions, metadata};
use std::io;
use std::path::Path;

use crate::config::{ConfigError, get as get_var};

/// Rel-path prefixes (under `<root>/<server>/`) that are
/// never evicted. `STORAGE_EVICT_KEEP`
/// adds to this list and cannot remove from it.
pub const BUILTIN_KEEP: &[&str] = &[
    // Story sprites: hub.json, sprites.json, PNG headers, thumbnail renders,
    // gallery variants
    "textures/avg/",
    // Story art variants
    "textures/spritepack/cutin_char_",
    // `core/release/art.rs` copies these into `derived/release-art`
    "textures/spritepack/ui_home_act_banner_gacha_",
    "textures/spritepack/ui_zone_home_theme_",
    "textures/arts/ui/stage/[uc]homeentry/",
    // `event_banner.rs` composes generated event banners from Story
    // Collection key visuals and titles
    "textures/spritepack/mixstory_",
    // Unpacker reads this
    "audio/m4a_sources.json",
];

/// Whether `sync` evicts, and what it keeps on top of [`BUILTIN_KEEP`]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EvictPolicy {
    /// `STORAGE_EVICT_LOCAL`; off by default
    pub enabled: bool,
    /// `STORAGE_EVICT_KEEP`: extra rel-path prefixes to keep
    pub extra_keep: Vec<String>,
}

impl EvictPolicy {
    /// # Errors
    /// An `STORAGE_EVICT_KEEP` entry containing a backslash.
    pub fn from_lookup(lookup: &dyn Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let enabled = get_var(lookup, "STORAGE_EVICT_LOCAL").is_some_and(|v| {
            matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
        });
        let extra_keep = match get_var(lookup, "STORAGE_EVICT_KEEP") {
            None => Vec::new(),
            Some(raw) => parse_keep(&raw)?,
        };
        Ok(Self {
            enabled,
            extra_keep,
        })
    }

    /// True when `rel` lies under a built-in or an extra keep prefix, or is a
    /// server-tagged home banner ([`is_server_home_banner`])
    #[must_use]
    pub fn keeps(&self, rel: &str) -> bool {
        BUILTIN_KEEP.iter().any(|p| rel.starts_with(p))
            || is_server_home_banner(rel)
            || self.extra_keep.iter().any(|p| rel.starts_with(p.as_str()))
    }
}

/// `textures/[[<anything>]]/arts/ui/homebanners/...`: gacha banners
/// under a server-tagged texture tree
#[must_use]
pub fn is_server_home_banner(rel: &str) -> bool {
    rel.strip_prefix("textures/[[")
        .and_then(|rest| rest.split_once("]]/"))
        .is_some_and(|(_, tail)| tail.starts_with("arts/ui/homebanners/"))
}

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
                var: "STORAGE_EVICT_KEEP",
                value: raw.to_owned(),
                reason: "prefixes use forward slashes",
            });
        }
        out.push(entry.to_owned());
    }
    Ok(out)
}

/// # Errors
/// The file's metadata cannot be read, it cannot be opened for writing, or
/// its mtime cannot be set.
pub fn truncate_in_place(path: &Path) -> io::Result<()> {
    let modified = metadata(path)?.modified()?;
    let file = OpenOptions::new().write(true).truncate(true).open(path)?;
    file.set_modified(modified)
}
