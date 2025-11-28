use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

use crate::generated_fbs::resource_manifest_generated::*;

/// Resource manifest for mapping extracted file names to proper output paths
pub struct ResourceManifest {
    /// Maps filename (with hash suffix) to proper output path
    /// e.g., "character_tabled88efb" -> "gamedata/excel/character_table"
    pub filename_to_path: HashMap<String, String>,
}

impl ResourceManifest {
    /// Load resource manifest from .idx file
    pub fn load(idx_path: &Path) -> Result<Self> {
        let data = std::fs::read(idx_path)
            .with_context(|| format!("Failed to read manifest file: {:?}", idx_path))?;

        // Skip first 128 bytes (RSA signature)
        if data.len() < 128 {
            anyhow::bail!("Manifest file too small");
        }
        let fb_data = &data[128..];

        // Decode FlatBuffer
        let manifest = unsafe { root_as_clz_torappu_resource_resource_manifest_unchecked(fb_data) };

        let mut filename_to_path = HashMap::new();

        // Regex to strip hash suffix from filename
        // Matches 6 hex chars at the end of the filename (before extension)
        let hash_suffix_re = Regex::new(r"[a-f0-9]{6}$").unwrap();

        if let Some(assets) = manifest.assetToBundleList() {
            for i in 0..assets.len() {
                let asset = assets.get(i);

                let name = match asset.name() {
                    Some(n) => n,
                    None => continue,
                };

                let path = match asset.path() {
                    Some(p) => p,
                    None => continue,
                };

                // Only process paths that start with "dyn/" - these are the game data files
                if !path.starts_with("dyn/") {
                    continue;
                }

                // Remove "dyn/" prefix from path
                let path_without_prefix = &path[4..];

                // Remove .bytes extension if present
                let path_clean = path_without_prefix
                    .strip_suffix(".bytes")
                    .unwrap_or(path_without_prefix);

                // Strip hash suffix from path
                let path_no_hash = if let Some(slash_pos) = path_clean.rfind('/') {
                    let (dir, filename) = path_clean.split_at(slash_pos + 1);
                    let filename_clean = hash_suffix_re.replace(filename, "").to_string();
                    format!("{}{}", dir, filename_clean)
                } else {
                    hash_suffix_re.replace(path_clean, "").to_string()
                };

                filename_to_path.insert(name.to_string(), path_no_hash);
            }
        }

        log::info!(
            "Loaded resource manifest with {} entries",
            filename_to_path.len()
        );

        Ok(ResourceManifest { filename_to_path })
    }

    /// Get the proper output path for a given extracted filename
    ///
    /// Input: filename like "character_tabled88efb" or "enemy_database3cc1f2"
    /// Output: path like "gamedata/excel/character_table" or "gamedata/levels/enemydata/enemy_database"
    pub fn get_output_path(&self, filename: &str) -> Option<&String> {
        self.filename_to_path.get(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_manifest() {
        let manifest = ResourceManifest::load(Path::new(
            "/Users/eltik/Documents/Coding/myrtle.moe/assets/ArkAssets/1dbfe7e0ce159cc1fb800b0a0aa1a565.idx"
        ));

        if let Ok(m) = manifest {
            // Check some known mappings
            if let Some(path) = m.get_output_path("character_tabled88efb") {
                println!("character_tabled88efb -> {}", path);
                assert!(path.contains("character_table"));
            }

            if let Some(path) = m.get_output_path("enemy_database3cc1f2") {
                println!("enemy_database3cc1f2 -> {}", path);
                assert!(path.contains("enemy_database"));
            }
        }
    }
}
