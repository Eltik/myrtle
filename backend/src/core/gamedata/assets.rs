use std::{collections::HashMap, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetKind {
    Avatar,       // textures/spritepack/ui_char_avatar_N/
    Portrait,     // portraits/{char_id}_{suffix}.png  (flat dir)
    SkinPortrait, // textures/spritepack/arts_shop_skin_portrait_N/
    SkillIcon,    // textures/spritepack/skill_icons_N/
    ModuleIcon,   // textures/spritepack/ui_equip_small_img_hub_N/
    ModuleBig,    // textures/spritepack/ui_equip_big_img_hub_N/
    EnemyIcon,    // textures/spritepack/icon_enemies_N/
    ItemIcon,     // textures/arts/ui_item_icons_N/ + arts/items/*_hub/
}

const ALL_KINDS: &[AssetKind] = &[
    AssetKind::Avatar,
    AssetKind::Portrait,
    AssetKind::SkinPortrait,
    AssetKind::SkillIcon,
    AssetKind::ModuleIcon,
    AssetKind::ModuleBig,
    AssetKind::EnemyIcon,
    AssetKind::ItemIcon,
];

#[derive(Debug, Clone, Default)]
pub struct AssetIndex {
    /// category -> name (stem without .png) -> relative path from assets root
    map: HashMap<AssetKind, HashMap<String, String>>,
    /// char_id -> list of available suffixes (e.g. ["_1", "_1+", "_2", "_2b"])
    chararts: HashMap<String, Vec<String>>,
    /// char_id -> list of skin art paths in skinpack/
    skinpacks: HashMap<String, Vec<String>>,
}

impl AssetIndex {
    /// Build the index by scanning the assets directory.
    /// Expects `assets_dir` to be the `output/` root containing `textures/` and `portraits/`.
    pub fn build(assets_dir: &Path) -> Self {
        let mut idx = Self::default();
        for kind in ALL_KINDS {
            idx.map.insert(*kind, HashMap::new());
        }

        let textures_dir = assets_dir.join("textures");
        let portraits_dir = assets_dir.join("portraits");

        for entry in walkdir::WalkDir::new(&textures_dir).min_depth(1) {
            let Ok(entry) = entry else { continue };
            let path = entry.path();

            if path.extension().is_none_or(|e| e != "png") {
                continue;
            }

            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let Some(rel) = path.strip_prefix(assets_dir).ok().and_then(|p| p.to_str()) else {
                continue;
            };
            let rel_path = format!("/{rel}");

            let Some(parent) = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
            else {
                continue;
            };

            // Classify into AssetKind by parent directory name
            if let Some(kind) = classify_dir(parent) {
                idx.map
                    .get_mut(&kind)
                    .unwrap()
                    .insert(stem.to_owned(), rel_path.clone());
            }

            // chararts/{char_id}/{char_id}_{suffix}.png
            if parent.starts_with("char_")
                && let Some(gp) = grandparent_name(path)
                && gp == "chararts"
            {
                let suffixes = idx.chararts.entry(parent.to_owned()).or_default();
                if let Some(suffix) = stem.strip_prefix(parent) {
                    suffixes.push(suffix.to_owned());
                }
            }

            // skinpack/{char_id}/*.png
            if let Some(gp) = grandparent_name(path)
                && gp == "skinpack"
                && parent.starts_with("char_")
            {
                idx.skinpacks
                    .entry(parent.to_owned())
                    .or_default()
                    .push(rel_path);
            }
        }

        if let Ok(entries) = std::fs::read_dir(&portraits_dir) {
            let portraits = idx.map.get_mut(&AssetKind::Portrait).unwrap();
            for entry in entries.flatten() {
                if let Some(stem) = png_stem(&entry) {
                    portraits.insert(stem.clone(), format!("/portraits/{}.png", stem));
                }
            }
        }

        idx
    }

    pub fn path(&self, kind: AssetKind, name: &str) -> Option<&str> {
        self.map.get(&kind)?.get(name).map(|s| s.as_str())
    }

    pub fn portrait_path(&self, char_id: &str) -> Option<&str> {
        let mut buf = String::with_capacity(char_id.len() + 2);
        buf.push_str(char_id);
        buf.push_str("_2");
        if let Some(p) = self.path(AssetKind::Portrait, &buf) {
            return Some(p);
        }
        buf.truncate(char_id.len());
        buf.push_str("_1");
        self.path(AssetKind::Portrait, &buf)
    }

    pub fn portrait_variants(&self, char_id: &str) -> Vec<&str> {
        let Some(portraits) = self.map.get(&AssetKind::Portrait) else {
            return vec![];
        };
        let prefix = format!("{char_id}_");
        portraits
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(_, v)| v.as_str())
            .collect()
    }

    pub fn charart_path(&self, char_id: &str) -> Option<String> {
        let suffixes = self.chararts.get(char_id)?;
        for preferred in &["_2", "_1"] {
            if suffixes.contains(&"_2".to_owned()) {
                return Some(format!(
                    "/textures/chararts/{char_id}/{char_id}{preferred}.png"
                ));
            }
        }
        None
    }

    pub fn charart_variants(&self, char_id: &str) -> Option<&[String]> {
        self.chararts.get(char_id).map(|v| v.as_slice())
    }

    pub fn has_charart(&self, char_id: &str) -> bool {
        self.chararts.contains_key(char_id)
    }

    pub fn skill_icon_path(&self, skill_id: &str) -> Option<&str> {
        self.path(AssetKind::SkillIcon, &format!("skill_icon_{skill_id}"))
    }

    pub fn module_big_path(&self, equip_icon: &str) -> Option<&str> {
        if equip_icon == "original" || equip_icon.starts_with("uniequip_001_") {
            return None;
        }
        self.path(AssetKind::ModuleBig, equip_icon)
    }

    pub fn module_icon_path(&self, equip_icon: &str) -> Option<&str> {
        if equip_icon == "original" || equip_icon.starts_with("uniequip_001_") {
            return None;
        }
        self.path(AssetKind::ModuleIcon, equip_icon)
    }

    pub fn skinpack_paths(&self, char_id: &str) -> Option<&[String]> {
        self.skinpacks.get(char_id).map(|v| v.as_slice())
    }
}

fn classify_dir(dir_name: &str) -> Option<AssetKind> {
    if dir_name.starts_with("ui_char_avatar_") {
        Some(AssetKind::Avatar)
    } else if dir_name.starts_with("arts_shop_skin_portrait_") {
        Some(AssetKind::SkinPortrait)
    } else if dir_name.starts_with("ui_equip_big_img_hub_") {
        Some(AssetKind::ModuleBig)
    } else if dir_name.starts_with("ui_equip_small_img_hub_") {
        Some(AssetKind::ModuleIcon)
    } else if dir_name.starts_with("skill_icons_") {
        Some(AssetKind::SkillIcon)
    } else if dir_name.starts_with("icon_enemies_") {
        Some(AssetKind::EnemyIcon)
    } else if dir_name.starts_with("ui_item_icons_") {
        Some(AssetKind::ItemIcon)
    } else if dir_name == "item_icons_no_tiny_hub" || dir_name == "item_icons_stack_hub" {
        Some(AssetKind::ItemIcon)
    } else {
        None
    }
}

fn grandparent_name(path: &Path) -> Option<&str> {
    path.parent()?.parent()?.file_name()?.to_str()
}

fn png_stem(entry: &std::fs::DirEntry) -> Option<String> {
    let name = entry.file_name();
    let s = name.to_str()?;
    s.strip_suffix(".png").map(String::from)
}
