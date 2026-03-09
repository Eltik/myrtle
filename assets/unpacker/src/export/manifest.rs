use super::resource_manifest_generated::*;
use regex::Regex;
use std::{collections::HashMap, io, path::Path};

pub struct ResourceManifest {
    pub filename_to_path: HashMap<String, String>,
}

impl ResourceManifest {
    pub fn load(idx_path: &Path) -> Result<Self, io::Error> {
        let data = std::fs::read(idx_path)?;

        if data.len() < 128 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "manifest too small",
            ));
        }

        // Skip 128-byte RSA signature
        let fb_data = &data[128..];

        let manifest = unsafe { root_as_clz_torappu_resource_resource_manifest_unchecked(fb_data) };

        let hash_re = Regex::new(r"[a-f0-9]{6}$").unwrap();
        let mut filename_to_path = HashMap::new();

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

                // Only process "dyn/" prefixed paths (game data files)
                if !path.starts_with("dyn/") {
                    continue;
                }

                // Clean path: remove "dyn/" prefix and ".bytes" extension
                let clean = path[4..].strip_suffix(".bytes").unwrap_or(&path[4..]);

                // Strip 6-char hex hash suffix from filename portion
                let result = if let Some(slash) = clean.rfind('/') {
                    let (dir, filename) = clean.split_at(slash + 1);
                    format!("{}{}", dir, hash_re.replace(filename, ""))
                } else {
                    hash_re.replace(clean, "").to_string()
                };

                filename_to_path.insert(name.to_string(), result);
            }
        }

        Ok(ResourceManifest { filename_to_path })
    }

    pub fn get_output_path(&self, filename: &str) -> Option<&str> {
        self.filename_to_path.get(filename).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_load_manifest() {
        let idx = PathBuf::from("../downloader/ArkAssets/17e57a849bd61f65f7acd03d65bd137b.idx");
        let manifest = match ResourceManifest::load(&idx) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("skip: {e}");
                return;
            }
        };

        println!("Loaded {} entries", manifest.filename_to_path.len());

        // Print some sample mappings
        for (name, path) in manifest.filename_to_path.iter().take(20) {
            println!("  {name} -> {path}");
        }

        assert!(!manifest.filename_to_path.is_empty());
    }
}
