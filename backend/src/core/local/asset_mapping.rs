use std::collections::HashMap;
use std::path::Path;

use crate::events::AssetStats;

/// Maps asset file names to their full directory paths
#[derive(Debug, Clone, Default)]
pub struct AssetMappings {
    /// Avatar images: char_xxx_yyy.png -> ui_char_avatar_N
    pub avatars: HashMap<String, String>,
    /// Skin portraits: char_xxx_yyy_skin#N.png -> arts_shop_skin_portrait_N
    pub skin_portraits: HashMap<String, String>,
    /// Module big images: uniequip_xxx_yyy.png -> ui_equip_big_img_hub_N
    pub module_big: HashMap<String, String>,
    /// Module small images: uniequip_xxx_yyy.png -> ui_equip_small_img_hub_N
    pub module_small: HashMap<String, String>,
    /// Operator portraits: char_xxx_yyy_N.png -> pack folder (e.g., "pack11")
    pub portraits: HashMap<String, String>,
    /// Skill icons: skill_icon_xxx.png -> skill_icons_N
    pub skill_icons: HashMap<String, String>,
    /// Full character art: char_xxx_yyy -> (has_e0, has_e2)
    pub chararts: HashMap<String, (bool, bool)>,
    /// Item icons: icon_id.png -> directory path (spritepack or arts/items/icons)
    pub item_icons: HashMap<String, String>,
}

/// Result of building asset mappings, includes stats for logging
pub struct AssetMappingResult {
    pub mappings: AssetMappings,
    pub stats: AssetStats,
}

impl AssetMappings {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build mappings by scanning the asset directories
    /// Returns both the mappings and statistics for logging
    pub fn build(assets_dir: &Path) -> AssetMappingResult {
        let spritepack = assets_dir.join("upk/spritepack");

        let mut mappings = Self::new();
        let mut stats = AssetStats::default();

        // Scan avatar directories (20 dirs)
        for i in 0..20 {
            let dir_name = format!("ui_char_avatar_{i}");
            let dir_path = spritepack.join(&dir_name);
            if dir_path.exists() {
                if let Ok(entries) = std::fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if file_name.ends_with(".png") {
                                let base_name = file_name.trim_end_matches(".png");
                                mappings
                                    .avatars
                                    .insert(base_name.to_string(), dir_name.clone());
                            }
                        }
                    }
                }
            }
        }
        stats.avatars = mappings.avatars.len();

        // Scan skin portrait directories (15 dirs)
        for i in 0..15 {
            let dir_name = format!("arts_shop_skin_portrait_{i}");
            let dir_path = spritepack.join(&dir_name);
            if dir_path.exists() {
                if let Ok(entries) = std::fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if file_name.ends_with(".png") {
                                let base_name = file_name.trim_end_matches(".png");
                                mappings
                                    .skin_portraits
                                    .insert(base_name.to_string(), dir_name.clone());
                            }
                        }
                    }
                }
            }
        }
        stats.skin_portraits = mappings.skin_portraits.len();

        // Scan module big image directories (25 dirs)
        for i in 0..25 {
            let dir_name = format!("ui_equip_big_img_hub_{i}");
            let dir_path = spritepack.join(&dir_name);
            if dir_path.exists() {
                if let Ok(entries) = std::fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if file_name.ends_with(".png") {
                                let base_name = file_name.trim_end_matches(".png");
                                mappings
                                    .module_big
                                    .insert(base_name.to_string(), dir_name.clone());
                            }
                        }
                    }
                }
            }
        }
        stats.module_big = mappings.module_big.len();

        // Scan module small image directories (25 dirs)
        for i in 0..25 {
            let dir_name = format!("ui_equip_small_img_hub_{i}");
            let dir_path = spritepack.join(&dir_name);
            if dir_path.exists() {
                if let Ok(entries) = std::fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if file_name.ends_with(".png") {
                                let base_name = file_name.trim_end_matches(".png");
                                mappings
                                    .module_small
                                    .insert(base_name.to_string(), dir_name.clone());
                            }
                        }
                    }
                }
            }
        }
        stats.module_small = mappings.module_small.len();

        // Scan skill icon directories (10 dirs)
        let skill_count_before = mappings.skill_icons.len();
        for i in 0..10 {
            let dir_name = format!("skill_icons_{i}");
            let dir_path = spritepack.join(&dir_name);
            if dir_path.exists() {
                if let Ok(entries) = std::fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if file_name.ends_with(".png") {
                                let base_name = file_name.trim_end_matches(".png");
                                mappings
                                    .skill_icons
                                    .insert(base_name.to_string(), dir_name.clone());
                            }
                        }
                    }
                }
            }
        }
        stats.skill_icons = mappings.skill_icons.len() - skill_count_before;

        // Scan alternate skill icons location at {assets_dir}/upk/arts/skills/
        let arts_skills_dir = assets_dir.join("upk/arts/skills");
        let skill_alt_count_before = mappings.skill_icons.len();
        if arts_skills_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&arts_skills_dir) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        if let Some(dir_name) = entry.file_name().to_str() {
                            let png_path = entry.path().join(format!("{dir_name}.png"));
                            if png_path.exists() {
                                mappings.skill_icons.insert(
                                    dir_name.to_string(),
                                    format!("arts_skills:{dir_name}"),
                                );
                            }
                        }
                    }
                }
            }
        }
        stats.skill_icons_alt = mappings.skill_icons.len() - skill_alt_count_before;

        // Scan portraits from {assets_dir}/upk/arts/charportraits/pack{N}/*.png
        let charportraits_dir = assets_dir.join("upk/arts/charportraits");
        if charportraits_dir.exists() {
            if let Ok(pack_dirs) = std::fs::read_dir(&charportraits_dir) {
                for pack_dir in pack_dirs.flatten() {
                    if pack_dir.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        if let Some(pack_name) = pack_dir.file_name().to_str() {
                            if let Ok(files) = std::fs::read_dir(pack_dir.path()) {
                                for file in files.flatten() {
                                    if let Some(file_name) = file.file_name().to_str() {
                                        if file_name.ends_with(".png") {
                                            let base_name = file_name.trim_end_matches(".png");
                                            mappings.portraits.insert(
                                                base_name.to_string(),
                                                pack_name.to_string(),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        stats.portraits = mappings.portraits.len();

        // Scan full character art from {assets_dir}/upk/chararts/{char_id}/
        let chararts_dir = assets_dir.join("upk/chararts");
        if chararts_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&chararts_dir) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        if let Some(dir_name) = entry.file_name().to_str() {
                            if dir_name.starts_with("char_") {
                                let e0_path = entry.path().join(format!("{dir_name}_1.png"));
                                let e2_path = entry.path().join(format!("{dir_name}_2.png"));
                                let has_e0 = e0_path.exists();
                                let has_e2 = e2_path.exists();
                                if has_e0 || has_e2 {
                                    mappings
                                        .chararts
                                        .insert(dir_name.to_string(), (has_e0, has_e2));
                                }
                            }
                        }
                    }
                }
            }
        }
        stats.chararts = mappings.chararts.len();

        // Scan item icon directories in spritepack
        let item_icon_dirs = [
            "ui_item_icons_h1_0",
            "ui_item_icons_h1_acticon_0",
            "ui_item_icons_h1_apsupply_0",
            "ui_item_icons_h1_classpotential_0",
            "ui_item_icons_h1_potential_0",
        ];
        let item_count_before = mappings.item_icons.len();
        for dir_name in &item_icon_dirs {
            let dir_path = spritepack.join(dir_name);
            if dir_path.exists() {
                if let Ok(entries) = std::fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if file_name.ends_with(".png") {
                                let base_name = file_name.trim_end_matches(".png");
                                mappings
                                    .item_icons
                                    .entry(base_name.to_string())
                                    .or_insert_with(|| format!("spritepack:{dir_name}"));
                            }
                        }
                    }
                }
            }
        }
        stats.item_icons = mappings.item_icons.len() - item_count_before;

        // Also scan the standard arts/items/icons directory
        let arts_items_dir = assets_dir.join("upk/arts/items/icons");
        let item_arts_count_before = mappings.item_icons.len();
        if arts_items_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&arts_items_dir) {
                for entry in entries.flatten() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if file_name.ends_with(".png") {
                            let base_name = file_name.trim_end_matches(".png");
                            mappings
                                .item_icons
                                .entry(base_name.to_string())
                                .or_insert_with(|| "arts_items".to_string());
                        }
                    }
                }
            }
        }
        stats.item_icons_arts = mappings.item_icons.len() - item_arts_count_before;

        AssetMappingResult { mappings, stats }
    }

    /// Get avatar path
    pub fn get_avatar_path(&self, avatar_id: &str) -> String {
        if let Some(dir) = self.avatars.get(avatar_id) {
            format!("/upk/spritepack/{dir}/{avatar_id}.png")
        } else {
            // Fallback - try first directory
            format!("/upk/spritepack/ui_char_avatar_0/{avatar_id}.png")
        }
    }

    /// Get skin portrait path (for skins with @ or # in ID)
    pub fn get_skin_portrait_path(&self, portrait_id: &str) -> String {
        if let Some(dir) = self.skin_portraits.get(portrait_id) {
            format!("/upk/spritepack/{dir}/{portrait_id}.png")
        } else if let Some(dir) = self.avatars.get(portrait_id) {
            // Fallback to avatar directories for default skins
            format!("/upk/spritepack/{dir}/{portrait_id}.png")
        } else {
            format!("/upk/spritepack/ui_char_avatar_0/{portrait_id}.png")
        }
    }

    /// Get module big image path
    /// uniequip_001_xxx modules are "original" placeholder modules with no image
    pub fn get_module_big_path(&self, equip_icon: &str) -> Option<String> {
        // "original" and uniequip_001_xxx are placeholder modules with no actual image
        if equip_icon == "original" || equip_icon.starts_with("uniequip_001_") {
            return None;
        }
        if let Some(dir) = self.module_big.get(equip_icon) {
            Some(format!("/upk/spritepack/{dir}/{equip_icon}.png"))
        } else {
            Some(format!(
                "/upk/spritepack/ui_equip_big_img_hub_0/{equip_icon}.png"
            ))
        }
    }

    /// Get module small image path
    /// uniequip_001_xxx modules are "original" placeholder modules with no image
    pub fn get_module_small_path(&self, equip_icon: &str) -> Option<String> {
        // "original" and uniequip_001_xxx are placeholder modules with no actual image
        if equip_icon == "original" || equip_icon.starts_with("uniequip_001_") {
            return None;
        }
        if let Some(dir) = self.module_small.get(equip_icon) {
            Some(format!("/upk/spritepack/{dir}/{equip_icon}.png"))
        } else {
            Some(format!(
                "/upk/spritepack/ui_equip_small_img_hub_0/{equip_icon}.png"
            ))
        }
    }

    /// Get operator portrait path (E2 preferred, fallback to E0)
    /// Portrait naming: char_xxx_yyy_2.png (E2), char_xxx_yyy_1.png (E0)
    /// Path format: /upk/arts/charportraits/{pack}/{portrait_name}.png
    pub fn get_portrait_path(&self, char_id: &str) -> Option<String> {
        // Try E2 portrait first (e.g., char_002_amiya_2.png)
        let e2_name = format!("{char_id}_2");
        if let Some(pack) = self.portraits.get(&e2_name) {
            return Some(format!("/upk/arts/charportraits/{pack}/{e2_name}.png"));
        }

        // Try E0 portrait (e.g., char_002_amiya_1.png)
        let e0_name = format!("{char_id}_1");
        if let Some(pack) = self.portraits.get(&e0_name) {
            return Some(format!("/upk/arts/charportraits/{pack}/{e0_name}.png"));
        }

        None
    }

    /// Get full character art path (E2 preferred, fallback to E0)
    /// Path format: /upk/chararts/{char_id}/{char_id}_{1|2}.png
    /// Returns None if chararts not available
    pub fn get_charart_path(&self, char_id: &str) -> Option<String> {
        match self.chararts.get(char_id) {
            Some((has_e0, has_e2)) => {
                // Prefer E2, fallback to E0
                if *has_e2 {
                    Some(format!("/upk/chararts/{char_id}/{char_id}_2.png"))
                } else if *has_e0 {
                    Some(format!("/upk/chararts/{char_id}/{char_id}_1.png"))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Check if full character art exists for an operator
    pub fn has_charart(&self, char_id: &str) -> bool {
        self.chararts.contains_key(char_id)
    }

    /// Get skill icon path
    /// skill_id: e.g., "skchr_phatm2_1" -> looks up "skill_icon_skchr_phatm2_1" in mappings
    pub fn get_skill_icon_path(&self, skill_id: &str) -> String {
        let icon_name = format!("skill_icon_{skill_id}");
        if let Some(dir) = self.skill_icons.get(&icon_name) {
            // Check for arts/skills alternate location (prefixed with "arts_skills:")
            if let Some(subdir) = dir.strip_prefix("arts_skills:") {
                format!("/upk/arts/skills/{subdir}/{icon_name}.png")
            } else {
                format!("/upk/spritepack/{dir}/{icon_name}.png")
            }
        } else {
            // Fallback to skill_icons_0
            format!("/upk/spritepack/skill_icons_0/{icon_name}.png")
        }
    }

    /// Get item icon path by icon_id
    /// Returns the correct path based on where the icon was found during scanning
    pub fn get_item_icon_path(&self, icon_id: &str) -> String {
        if let Some(location) = self.item_icons.get(icon_id) {
            if let Some(dir) = location.strip_prefix("spritepack:") {
                format!("/upk/spritepack/{dir}/{icon_id}.png")
            } else if location == "arts_items" {
                format!("/upk/arts/items/icons/{icon_id}.png")
            } else {
                // Fallback for any other format
                format!("/upk/arts/items/icons/{icon_id}.png")
            }
        } else {
            // Fallback to default items directory
            format!("/upk/arts/items/icons/{icon_id}.png")
        }
    }
}
