use super::resource_manifest_generated::root_as_clz_torappu_resource_resource_manifest_unchecked;
use regex::Regex;
use std::{
    collections::{HashMap, hash_map::Entry},
    io,
    path::Path,
};

pub struct ResourceManifest {
    /// Asset name -> one output path. Ambiguous by construction: an asset name
    /// is NOT unique in the manifest (see `bundle_asset_to_paths`). Kept for
    /// callers that only need a file stem, and as the fallback when a bundle is
    /// not named by the manifest at all.
    pub filename_to_path: HashMap<String, String>,
    /// `(bundle, asset name)` -> every output path that pair addresses, both
    /// keys lowercased. This is the routing key that actually identifies an
    /// object, because the same asset name appears in several bundles.
    bundle_asset_to_paths: HashMap<(String, String), Vec<String>>,
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

        let mut rows: Vec<(String, String, String)> = Vec::new();
        let bundles = manifest.bundles();

        if let Some(assets) = manifest.assetToBundleList() {
            for i in 0..assets.len() {
                let asset = assets.get(i);

                let (Some(name), Some(path)) = (asset.name(), asset.path()) else {
                    continue;
                };

                let index = asset.bundleIndex();
                let bundle = bundles
                    .as_ref()
                    .filter(|b| index >= 0 && (index as usize) < b.len())
                    .and_then(|b| b.get(index as usize).name())
                    .unwrap_or_default();

                rows.push((name.to_string(), path.to_string(), bundle.to_string()));
            }
        }

        Ok(Self::from_rows(
            rows.iter()
                .map(|(n, p, b)| (n.as_str(), p.as_str(), b.as_str())),
        ))
    }

    /// Build the lookup tables from `(asset name, asset path, bundle name)`
    /// rows, which is what `assetToBundleList` carries. Split out from `load`
    /// so the routing rules can be tested without a 31 MB `.idx`.
    #[must_use]
    pub fn from_rows<'a>(rows: impl IntoIterator<Item = (&'a str, &'a str, &'a str)>) -> Self {
        let hash_re = Regex::new(r"[a-f0-9]{6}$").unwrap();
        let mut filename_to_path: HashMap<String, String> = HashMap::new();
        let mut bundle_asset_to_paths: HashMap<(String, String), Vec<String>> = HashMap::new();

        for (name, path, bundle) in rows {
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
            if !result.starts_with("gamedata/") {
                continue;
            }

            // Manifest asset names are lowercase, but TextAsset m_Name keeps
            // the original casing (e.g. `PCHotfixer.lua` vs manifest
            // `pchotfixer.lua`), so key the maps case-insensitively.
            let key = (normalize_bundle(bundle), name.to_lowercase());
            let slot = bundle_asset_to_paths.entry(key).or_default();
            if !slot.iter().any(|p| p == &result) {
                slot.push(result.clone());
            }

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
            match filename_to_path.entry(name.to_lowercase()) {
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

        Self {
            filename_to_path,
            bundle_asset_to_paths,
        }
    }

    #[must_use]
    pub fn get_output_path(&self, filename: &str) -> Option<&str> {
        self.filename_to_path
            .get(&filename.to_lowercase())
            .map(std::string::String::as_str)
    }

    /// Every output path the manifest gives `filename` INSIDE `bundle`.
    ///
    /// An asset name alone does not identify an object. Every story ships
    /// twice under one name: the script at `gamedata/story/<path>.txt` and its
    /// one-line summary at `gamedata/story/[uc]info/<path>.txt`, in two
    /// different bundles. Routing by name sent both objects to whichever path
    /// the map happened to hold, so the summary overwrote the script. The
    /// bundle disambiguates them, and the remaining duplicates are one object
    /// the client addresses under two paths (`avg_segment_report`, the
    /// `act29side`/`act42side` marks), so the caller writes it to each.
    ///
    /// `bundle` is the bundle file's path relative to the server's asset root,
    /// e.g. `anon/0dfb59a7275154d610241fcaef7aef4d.bin`.
    #[must_use]
    pub fn get_output_paths(&self, bundle: &str, filename: &str) -> Option<&[String]> {
        self.bundle_asset_to_paths
            .get(&(normalize_bundle(bundle), filename.to_lowercase()))
            .map(std::vec::Vec::as_slice)
    }
}

/// Bundle keys compare case-insensitively with `/` separators, so a Windows
/// walk and the manifest's own spelling land on the same entry.
fn normalize_bundle(bundle: &str) -> String {
    bundle.replace('\\', "/").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::ResourceManifest;

    /// The two manifest rows of one real story, `1stact_level_a001_01_beg`,
    /// copied verbatim from `a90ddcfdefc21d290ce93eac0fdc1331.idx` rows 57312
    /// and 160708. Both carry the SAME asset name; only the bundle differs.
    const BODY_BUNDLE: &str = "anon/0dfb59a7275154d610241fcaef7aef4d.bin";
    const INFO_BUNDLE: &str = "anon/7aa7486d5afe861f942d1b4d83552da0.bin";

    fn a001() -> ResourceManifest {
        ResourceManifest::from_rows([
            (
                "level_a001_01_beg",
                "dyn/gamedata/story/activities/a001/level_a001_01_beg.txt",
                BODY_BUNDLE,
            ),
            (
                "level_a001_01_beg",
                "dyn/gamedata/story/[uc]info/activities/a001/level_a001_01_beg.txt",
                INFO_BUNDLE,
            ),
        ])
    }

    /// The defect, stated as an assertion: the name alone answers with ONE
    /// path for both objects, so a name-routed export writes the summary over
    /// the script.
    #[test]
    fn asset_name_alone_is_ambiguous() {
        let m = a001();
        assert_eq!(
            m.get_output_path("level_a001_01_beg"),
            Some("gamedata/story/activities/a001/level_a001_01_beg.txt")
        );
    }

    /// The fix: the bundle separates them, each to its own path.
    #[test]
    fn bundle_scoped_lookup_separates_script_from_summary() {
        let m = a001();
        assert_eq!(
            m.get_output_paths(BODY_BUNDLE, "level_a001_01_beg"),
            Some(["gamedata/story/activities/a001/level_a001_01_beg.txt".to_string()].as_slice())
        );
        assert_eq!(
            m.get_output_paths(INFO_BUNDLE, "level_a001_01_beg"),
            Some(
                ["gamedata/story/[uc]info/activities/a001/level_a001_01_beg.txt".to_string()]
                    .as_slice()
            )
        );
    }

    /// `m_Name` keeps the original casing and the bundle may be walked with
    /// either separator; both keys are normalised.
    #[test]
    fn lookup_is_case_and_separator_insensitive() {
        let m = a001();
        assert!(
            m.get_output_paths(
                "ANON\\0DFB59A7275154D610241FCAEF7AEF4D.BIN",
                "Level_A001_01_Beg"
            )
            .is_some()
        );
    }

    /// One object addressed under two paths inside ONE bundle keeps both, in
    /// manifest order, so neither client path is dropped.
    #[test]
    fn duplicate_names_in_one_bundle_keep_every_path() {
        let m = ResourceManifest::from_rows([
            (
                "avg_segment_report",
                "dyn/gamedata/story/activities/avg_segment_report.txt",
                BODY_BUNDLE,
            ),
            (
                "avg_segment_report",
                "dyn/gamedata/story/activities/act49side/avg_segment_report.txt",
                BODY_BUNDLE,
            ),
        ]);
        assert_eq!(
            m.get_output_paths(BODY_BUNDLE, "avg_segment_report")
                .map(<[String]>::len),
            Some(2)
        );
    }

    /// Non-gamedata siblings are still skipped and `gamedata/levels/` still
    /// wins the name-only map, which is what keeps level data from being lost.
    #[test]
    fn levels_still_win_the_name_only_map() {
        let m = ResourceManifest::from_rows([
            (
                "level_main_01-01",
                "dyn/scenes/obt/main/level_main_01-01.unity",
                "scenes/obt.ab",
            ),
            (
                "level_main_01-01",
                "dyn/gamedata/levels/obt/main/level_main_01-01.bytes",
                BODY_BUNDLE,
            ),
        ]);
        assert_eq!(
            m.get_output_path("level_main_01-01"),
            Some("gamedata/levels/obt/main/level_main_01-01")
        );
        assert_eq!(
            m.get_output_paths("scenes/obt.ab", "level_main_01-01"),
            None
        );
    }
}
