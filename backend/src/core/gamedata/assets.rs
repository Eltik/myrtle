use std::{collections::HashMap, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetKind {
    Avatar, Portrait, SkinPortrait, SkinCharart,
    SkillIcon, ModuleIcon, ModuleBig,
    EnemyIcon, ItemIcon, BrandLogo,
}

const ALL_KINDS: &[AssetKind] = &[
    AssetKind::Avatar,
    AssetKind::Portrait,
    AssetKind::SkinPortrait,
    AssetKind::SkinCharart,
    AssetKind::SkillIcon,
    AssetKind::ModuleIcon,
    AssetKind::ModuleBig,
    AssetKind::EnemyIcon,
    AssetKind::ItemIcon,
    AssetKind::BrandLogo,
];

#[derive(Debug, Clone, Default)]
pub struct AssetIndex {
    /// category -> name (without .png) -> full relative path from upk root
    map: HashMap<AssetKind, HashMap<String, String>>,
    /// char_id -> (has_e0, has_e2) for chararts
    chararts: HashMap<String, (bool, bool)>,
}

impl AssetIndex {
    /// Single recursive walk of the upk directory.
    /// Categorizes every .png by its parent directory prefix.
    pub fn build(upk_dir: &Path) -> Self {
        let mut idx = Self::default();
        for kind in ALL_KINDS {
            idx.map.insert(*kind, HashMap::new());
        }

        let Ok(walker) = walkdir::WalkDir::new(upk_dir)
            .min_depth(1)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
        else {
            return idx;
        };

        for entry in &walker {
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) != Some("png") {
                continue;
            }

            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };

            let Some(rel) = path.strip_prefix(upk_dir).ok().and_then(|p| p.to_str()) else {
                continue;
            };

            let rel_path = format!("/upk/{rel}");

            let Some(parent) = path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str())
            else {
                continue;
            };
        }

        Self {

        }
    }
    pub fn path(&self, kind: AssetKind, name: &str) -> Option<&str> {

    }
}
