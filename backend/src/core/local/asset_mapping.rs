use std::collections::HashMap;
use std::path::Path;

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
    /// Operator portraits: char_xxx_yyy_N.png exists in portraits folder
    pub portraits: std::collections::HashSet<String>,
    /// Skill icons: skill_icon_xxx.png -> skill_icons_N
    pub skill_icons: HashMap<String, String>,
}

impl AssetMappings {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build mappings by scanning the asset directories
    /// assets_dir is the ASSETS_DIR environment variable path (e.g., /assets/Unpacked/)
    /// - Spritepacks at {assets_dir}/upk/spritepack/
    /// - Portraits at {assets_dir}/portraits/
    pub fn build(assets_dir: &Path) -> Self {
        // Spritepack is at {assets_dir}/upk/spritepack
        let spritepack = assets_dir.join("upk/spritepack");

        eprintln!("Scanning spritepack directory: {:?}", spritepack);

        let mut mappings = Self::new();

        // Scan avatar directories
        for i in 0..20 {
            let dir_name = format!("ui_char_avatar_{}", i);
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

        // Scan skin portrait directories
        for i in 0..15 {
            let dir_name = format!("arts_shop_skin_portrait_{}", i);
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

        // Scan module big image directories
        for i in 0..25 {
            let dir_name = format!("ui_equip_big_img_hub_{}", i);
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

        // Scan module small image directories
        for i in 0..25 {
            let dir_name = format!("ui_equip_small_img_hub_{}", i);
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

        // Scan skill icon directories
        for i in 0..10 {
            let dir_name = format!("skill_icons_{}", i);
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

        // Scan portraits from {assets_dir}/upk/chararts/{char_id}/*.png
        let chararts_dir = assets_dir.join("upk/chararts");

        eprintln!("Scanning chararts directory: {:?}", chararts_dir);

        if chararts_dir.exists() {
            if let Ok(char_dirs) = std::fs::read_dir(&chararts_dir) {
                for char_dir in char_dirs.flatten() {
                    if char_dir.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        if let Ok(files) = std::fs::read_dir(char_dir.path()) {
                            for file in files.flatten() {
                                if let Some(file_name) = file.file_name().to_str() {
                                    if file_name.ends_with(".png") {
                                        let base_name = file_name.trim_end_matches(".png");
                                        mappings.portraits.insert(base_name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        eprintln!(
            "Built asset mappings: {} avatars, {} skin portraits, {} module big, {} module small, {} skill icons, {} portraits",
            mappings.avatars.len(),
            mappings.skin_portraits.len(),
            mappings.module_big.len(),
            mappings.module_small.len(),
            mappings.skill_icons.len(),
            mappings.portraits.len()
        );

        mappings
    }

    /// Get avatar path
    pub fn get_avatar_path(&self, avatar_id: &str) -> String {
        if let Some(dir) = self.avatars.get(avatar_id) {
            format!("/upk/spritepack/{}/{}.png", dir, avatar_id)
        } else {
            // Fallback - try first directory
            format!("/upk/spritepack/ui_char_avatar_0/{}.png", avatar_id)
        }
    }

    /// Get skin portrait path (for skins with @ or # in ID)
    pub fn get_skin_portrait_path(&self, portrait_id: &str) -> String {
        if let Some(dir) = self.skin_portraits.get(portrait_id) {
            format!("/upk/spritepack/{}/{}.png", dir, portrait_id)
        } else if let Some(dir) = self.avatars.get(portrait_id) {
            // Fallback to avatar directories for default skins
            format!("/upk/spritepack/{}/{}.png", dir, portrait_id)
        } else {
            format!("/upk/spritepack/ui_char_avatar_0/{}.png", portrait_id)
        }
    }

    /// Get module big image path
    pub fn get_module_big_path(&self, equip_icon: &str) -> String {
        if equip_icon == "original" {
            return "/upk/spritepack/ui_equip_big_img_hub_0/default.png".to_string();
        }
        if let Some(dir) = self.module_big.get(equip_icon) {
            format!("/upk/spritepack/{}/{}.png", dir, equip_icon)
        } else {
            format!("/upk/spritepack/ui_equip_big_img_hub_0/{}.png", equip_icon)
        }
    }

    /// Get module small image path
    pub fn get_module_small_path(&self, equip_icon: &str) -> String {
        if equip_icon == "original" {
            return "/upk/spritepack/ui_equip_small_img_hub_0/default.png".to_string();
        }
        if let Some(dir) = self.module_small.get(equip_icon) {
            format!("/upk/spritepack/{}/{}.png", dir, equip_icon)
        } else {
            format!(
                "/upk/spritepack/ui_equip_small_img_hub_0/{}.png",
                equip_icon
            )
        }
    }

    /// Get operator portrait path (E2 preferred, fallback to E0)
    /// Portrait naming: char_xxx_yyy_2.png (E2), char_xxx_yyy_1.png (E0)
    /// Path format: /upk/chararts/{char_id}/{portrait_name}.png
    pub fn get_portrait_path(&self, char_id: &str) -> Option<String> {
        // Try E2 portrait first (e.g., char_002_amiya_2.png)
        let e2_name = format!("{}_2", char_id);
        if self.portraits.contains(&e2_name) {
            return Some(format!("/upk/chararts/{}/{}.png", char_id, e2_name));
        }

        // Try E0 portrait (e.g., char_002_amiya_1.png)
        let e0_name = format!("{}_1", char_id);
        if self.portraits.contains(&e0_name) {
            return Some(format!("/upk/chararts/{}/{}.png", char_id, e0_name));
        }

        None
    }

    /// Get skill icon path
    /// skill_id: e.g., "skchr_phatm2_1" -> looks up "skill_icon_skchr_phatm2_1" in mappings
    pub fn get_skill_icon_path(&self, skill_id: &str) -> String {
        let icon_name = format!("skill_icon_{}", skill_id);
        if let Some(dir) = self.skill_icons.get(&icon_name) {
            format!("/upk/spritepack/{}/{}.png", dir, icon_name)
        } else {
            // Fallback to skill_icons_0
            format!("/upk/spritepack/skill_icons_0/{}.png", icon_name)
        }
    }
}
