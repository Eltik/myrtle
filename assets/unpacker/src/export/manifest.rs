use super::resource_manifest_generated::root_as_clz_torappu_resource_resource_manifest_unchecked;
use regex::Regex;
use std::{
    collections::{HashMap, hash_map::Entry},
    io,
    path::Path,
};

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

                // The only consumer (export_gamedata) extracts gamedata files.
                // Many assets share an asset `name` across domains — e.g. a
                // stage's `gamedata/levels/.../level_main_xx.bytes` and its Unity
                // `scenes/.../level_main_xx.unity`. A plain insert is
                // last-write-wins, so whenever the non-gamedata sibling happened
                // to come later in the manifest it clobbered the real level path
                // and the level was silently dropped (this is why only an
                // arbitrary subset of levels — those without a colliding sibling
                // ordered after them — ever extracted). Skip non-gamedata paths
                // entirely, and on a gamedata-vs-gamedata collision prefer
                // `gamedata/levels/` so level data is never lost.
                if !result.starts_with("gamedata/") {
                    continue;
                }
                match filename_to_path.entry(name.to_string()) {
                    Entry::Vacant(e) => {
                        e.insert(result);
                    }
                    Entry::Occupied(mut e) => {
                        if result.starts_with("gamedata/levels/")
                            && !e.get().starts_with("gamedata/levels/")
                        {
                            e.insert(result);
                        }
                    }
                }
            }
        }

        Ok(Self { filename_to_path })
    }

    #[must_use]
    pub fn get_output_path(&self, filename: &str) -> Option<&str> {
        self.filename_to_path
            .get(filename)
            .map(std::string::String::as_str)
    }
}
