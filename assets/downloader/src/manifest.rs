use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::types::HotFile;

pub struct Manifest {
    path: PathBuf,
    entries: HashMap<String, String>,
}

impl Manifest {
    pub fn load(dir: &Path) -> anyhow::Result<Self> {
        let path = dir.join("persistent_res_list.json");
        let entries = match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => HashMap::new(),
        };
        Ok(Self { path, entries })
    }

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

    pub fn save(&self) -> anyhow::Result<()> {
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(&self.entries)?)?;
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}
