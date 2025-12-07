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
}

impl AssetMappings {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build mappings by scanning the spritepack directories
    /// assets_dir is expected to be the "decoded/gamedata" directory
    /// spritepack is at "../upk/spritepack" relative to that
    pub fn build(assets_dir: &Path) -> Self {
        // Navigate from decoded/gamedata to Unpacked/upk/spritepack
        let spritepack = assets_dir
            .parent() // decoded
            .and_then(|p| p.parent()) // Unpacked
            .map(|p| p.join("upk/spritepack"))
            .unwrap_or_else(|| assets_dir.join("upk/spritepack"));

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

        eprintln!(
            "Built asset mappings: {} avatars, {} skin portraits, {} module big, {} module small",
            mappings.avatars.len(),
            mappings.skin_portraits.len(),
            mappings.module_big.len(),
            mappings.module_small.len()
        );

        mappings
    }

    /// Get avatar path
    pub fn get_avatar_path(&self, avatar_id: &str) -> String {
        if let Some(dir) = self.avatars.get(avatar_id) {
            format!("/spritepack/{}/{}.png", dir, avatar_id)
        } else {
            // Fallback - try first directory
            format!("/spritepack/ui_char_avatar_0/{}.png", avatar_id)
        }
    }

    /// Get skin portrait path (for skins with @ or # in ID)
    pub fn get_skin_portrait_path(&self, portrait_id: &str) -> String {
        if let Some(dir) = self.skin_portraits.get(portrait_id) {
            format!("/spritepack/{}/{}.png", dir, portrait_id)
        } else if let Some(dir) = self.avatars.get(portrait_id) {
            // Fallback to avatar directories for default skins
            format!("/spritepack/{}/{}.png", dir, portrait_id)
        } else {
            format!("/spritepack/ui_char_avatar_0/{}.png", portrait_id)
        }
    }

    /// Get module big image path
    pub fn get_module_big_path(&self, equip_icon: &str) -> String {
        if equip_icon == "original" {
            return "/spritepack/ui_equip_big_img_hub_0/default.png".to_string();
        }
        if let Some(dir) = self.module_big.get(equip_icon) {
            format!("/spritepack/{}/{}.png", dir, equip_icon)
        } else {
            format!("/spritepack/ui_equip_big_img_hub_0/{}.png", equip_icon)
        }
    }

    /// Get module small image path
    pub fn get_module_small_path(&self, equip_icon: &str) -> String {
        if equip_icon == "original" {
            return "/spritepack/ui_equip_small_img_hub_0/default.png".to_string();
        }
        if let Some(dir) = self.module_small.get(equip_icon) {
            format!("/spritepack/{}/{}.png", dir, equip_icon)
        } else {
            format!("/spritepack/ui_equip_small_img_hub_0/{}.png", equip_icon)
        }
    }
}
