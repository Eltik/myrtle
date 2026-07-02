use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::types::HotFile;

pub struct Manifest {
    path: PathBuf,
    entries: HashMap<String, String>,
}

impl Manifest {
    /// # Errors
    ///
    /// Currently infallible: a missing or unreadable manifest file yields an
    /// empty entry set and malformed JSON falls back to defaults, so `Ok` is
    /// always returned. The `Result` return type leaves room for future
    /// fallible initialization.
    pub fn load(dir: &Path) -> anyhow::Result<Self> {
        let path = dir.join("persistent_res_list.json");
        let entries = std::fs::read_to_string(&path).map_or_else(
            |_| HashMap::new(),
            |content| serde_json::from_str(&content).unwrap_or_default(),
        );
        Ok(Self { path, entries })
    }

    #[must_use]
    pub fn filter_needed(&self, files: &[HotFile]) -> Vec<HotFile> {
        files
            .iter()
            .filter(|f| self.entries.get(&f.name) != Some(&f.md5))
            .cloned()
            .collect()
    }

    pub fn update(&mut self, filename: &str, md5: &str) {
        self.entries.insert(filename.to_string(), md5.to_string());
    }

    /// # Errors
    ///
    /// Returns an error if the entries cannot be serialized to JSON, or if the
    /// temp file cannot be written or renamed into place.
    pub fn save(&self) -> anyhow::Result<()> {
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(&self.entries)?)?;
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}
