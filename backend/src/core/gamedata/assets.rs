use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

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
    MedalIcon,    // textures/spritepack/ui_medal_icons_N/
    /// `textures/spritepack/ui_home_act_banner_gacha`_{h2,en}_`N/{picLimited_76_0_1}.png`
    /// Keyed by the NORMALISED stem (lowercase, alphanumerics only): the packs
    /// spell the same pool `picClassicAttain_68_0_2` and `picClassic_Attain_57_0_2`.
    GachaBanner,
    /// textures/arts/ui/stage/[uc]`homeentry/{act_id}.png`: the home-screen
    /// entry card, the event's official key visual. Only that directory: the
    /// `ui_zone_home_theme_{act_id}` spritepacks unpack to the whole atlas
    /// page (1000 or 1024 px square, 33 on CN / 45 on EN), not the card, and
    /// walking them after `homeentry` used to overwrite the card with the
    /// sheet.
    EventBanner,
    /// `textures/spritepack/story_review_chapter_bg_h1_N/storyEntryPic_{id}.png`
    /// and `story_review_mini_activity_bg_h1_storyentrypic_N/`: the Archives
    /// entry art, kept for every story event (70 of 75 story reviews on CN,
    /// 67 on EN). Keyed by the story review id, which is the activity id.
    StoryEntryPic,
    /// `textures/arts/loadingillusts_N/{loading_pic_id}.png`: the loading
    /// illustrations, one official piece per event, named by event code and
    /// reached from an activity through its stages' `LoadingPicId`. Kept by
    /// the client for every event (184 on EN, 201 on CN).
    LoadingIllust,
    /// `textures/spritepack/ui_kv_img_N/{kv_id}.png` (skin brand key visuals)
    BrandKv,
    /// `textures/spritepack/ui_brand_image_hub_N/brand`_{`brand_id}.png`
    BrandLogo,
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
    AssetKind::MedalIcon,
    AssetKind::GachaBanner,
    AssetKind::EventBanner,
    AssetKind::StoryEntryPic,
    AssetKind::LoadingIllust,
    AssetKind::BrandKv,
    AssetKind::BrandLogo,
];

#[derive(Debug, Clone, Default)]
pub struct AssetIndex {
    /// category -> name (stem without .png) -> relative path from assets root
    map: HashMap<AssetKind, HashMap<String, String>>,
    /// `char_id` -> list of available suffixes (e.g. ["_1", "_1+", "_2", "_2b"])
    chararts: HashMap<String, Vec<String>>,
    /// `char_id` -> list of skin art paths in skinpack/
    skinpacks: HashMap<String, Vec<String>>,
    /// Audio file stem -> relative paths under `audio/audio/sound_beta_2/`.
    /// Battle SFX assets reference a logical path whose directory is flattened
    /// into arbitrary numbered buckets on disk, so they resolve by basename.
    audio_by_name: HashMap<String, Vec<String>>,
    /// As above but keyed by the stem with a trailing `_<digits>` stripped, so a
    /// base asset (`b_char_kong`) also collects its weighted variants (`_1`,`_2`).
    audio_by_base: HashMap<String, Vec<String>>,
    /// Every relative audio path under `sound_beta_2/` (verbatim, unencoded).
    /// Used to confirm voice-bark assets, which resolve by direct path.
    audio_paths: HashSet<String>,
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

        // No cheap count for these walks (~90k textures, ~73k audio per server),
        // so the bar interpolates across three steps.
        crate::core::startup::step("textures");
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
                let key = match kind {
                    AssetKind::GachaBanner => normalize_stem(stem),
                    AssetKind::StoryEntryPic => stem
                        .strip_prefix("storyEntryPic_")
                        .unwrap_or(stem)
                        .to_owned(),
                    _ => stem.to_owned(),
                };
                idx.map
                    .get_mut(&kind)
                    .unwrap()
                    .insert(key, rel_path.clone());
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

        // Release-art archive (see `core::release::art`): images kept after the
        // client pack dropped them. Read AFTER textures so a live file wins.
        for (sub, kind) in [
            ("gacha", AssetKind::GachaBanner),
            ("event", AssetKind::EventBanner),
        ] {
            let dir = assets_dir.join("derived/release-art").join(sub);
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            let map = idx.map.get_mut(&kind).unwrap();
            for entry in entries.flatten() {
                if let Some(stem) = png_stem(&entry) {
                    let key = if kind == AssetKind::GachaBanner {
                        normalize_stem(&format!("pic{stem}"))
                    } else {
                        stem.clone()
                    };
                    map.entry(key)
                        .or_insert_with(|| format!("/derived/release-art/{sub}/{stem}.png"));
                }
            }
        }

        crate::core::startup::step("portraits");
        if let Ok(entries) = std::fs::read_dir(&portraits_dir) {
            let portraits = idx.map.get_mut(&AssetKind::Portrait).unwrap();
            for entry in entries.flatten() {
                if let Some(stem) = png_stem(&entry) {
                    portraits.insert(stem.clone(), format!("/portraits/{stem}.png"));
                }
            }
        }

        crate::core::startup::step("audio");
        let audio_root = assets_dir.join("audio/audio/sound_beta_2");
        for entry in walkdir::WalkDir::new(&audio_root).min_depth(1) {
            let Ok(entry) = entry else { continue };
            let path = entry.path();

            if path.extension().is_none_or(|e| e != "ogg") {
                continue;
            }

            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let Some(rel) = path.strip_prefix(&audio_root).ok().and_then(|p| p.to_str()) else {
                continue;
            };
            let rel = rel.to_owned();

            idx.audio_paths.insert(rel.clone());
            idx.audio_by_name
                .entry(stem.to_owned())
                .or_default()
                .push(rel.clone());
            if let Some(base) = strip_numeric_suffix(stem) {
                idx.audio_by_base
                    .entry(base.to_owned())
                    .or_default()
                    .push(rel);
            }
        }

        idx
    }

    /// Resolve a logical audio asset (`Audio/Sound_Beta_2/...`) to playable URLs
    /// (`/audio/sound_beta_2/...`). Voice-bark assets (`Voice*`/`Vox`) resolve by
    /// their verbatim disk path with the first segment lowercased; everything
    /// else (battle SFX) resolves by basename, returning the base file plus any
    /// weighted variants. Returns empty when nothing matches.
    pub fn resolve_audio(&self, asset: &str) -> Vec<String> {
        const PREFIX: &str = "audio/sound_beta_2/";
        const VOICE_DIRS: &[&str] = &["voice", "voice_cn", "voice_en", "voice_kr", "vox"];

        let logical =
            if asset.len() >= PREFIX.len() && asset[..PREFIX.len()].eq_ignore_ascii_case(PREFIX) {
                &asset[PREFIX.len()..]
            } else {
                asset
            };

        let (seg0, rest) = match logical.split_once('/') {
            Some((a, b)) => (a, Some(b)),
            None => (logical, None),
        };
        let seg0_lower = seg0.to_ascii_lowercase();

        if VOICE_DIRS.contains(&seg0_lower.as_str()) {
            if let Some(rest) = rest {
                let rel = format!("{seg0_lower}/{rest}.ogg");
                if self.audio_paths.contains(&rel) {
                    return vec![audio_url(&rel)];
                }
            }
            return Vec::new();
        }

        let base = logical.rsplit('/').next().unwrap_or(logical);
        let mut urls: Vec<String> = self
            .audio_by_name
            .get(base)
            .into_iter()
            .chain(self.audio_by_base.get(base))
            .flatten()
            .map(|rel| audio_url(rel))
            .collect();
        urls.sort();
        urls.dedup();
        urls
    }

    pub fn path(&self, kind: AssetKind, name: &str) -> Option<&str> {
        self.map
            .get(&kind)?
            .get(name)
            .map(std::string::String::as_str)
    }

    /// Banner art for a gacha pool id (`LIMITED_76_0_1`, `DOUBLE_EN_41_0_2`):
    /// the pack names the file `pic` + the pool id with its own spelling, so
    /// both sides are normalised before the lookup.
    pub fn gacha_banner_path(&self, pool_id: &str) -> Option<&str> {
        let key = normalize_stem(&format!("pic{pool_id}"));
        self.path(AssetKind::GachaBanner, &key)
    }

    /// Event art by activity id: the home entry (the official key visual)
    /// while the client or the archive has it, else the Archives entry
    /// picture. The loading illustration is indexed but not used here: it is
    /// a scene, not the event's poster.
    pub fn event_banner_path(&self, act_id: &str) -> Option<&str> {
        self.path(AssetKind::EventBanner, act_id)
            .or_else(|| self.path(AssetKind::StoryEntryPic, act_id))
    }

    pub fn loading_illust_path(&self, pic: &str) -> Option<&str> {
        self.path(AssetKind::LoadingIllust, pic)
    }

    pub fn brand_kv_path(&self, kv_id: &str) -> Option<&str> {
        self.path(AssetKind::BrandKv, kv_id)
    }

    pub fn brand_logo_path(&self, brand_id: &str) -> Option<&str> {
        self.path(AssetKind::BrandLogo, &format!("brand_{brand_id}"))
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
            if suffixes.iter().any(|s| s == preferred) {
                return Some(format!(
                    "/textures/chararts/{char_id}/{char_id}{preferred}.png"
                ));
            }
        }
        None
    }

    pub fn charart_variants(&self, char_id: &str) -> Option<&[String]> {
        self.chararts.get(char_id).map(std::vec::Vec::as_slice)
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
        self.skinpacks.get(char_id).map(std::vec::Vec::as_slice)
    }
}

fn classify_dir(dir_name: &str) -> Option<AssetKind> {
    if dir_name.starts_with("ui_char_avatar_") || dir_name.starts_with("ui_player_avatar_list_") {
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
    } else if dir_name.starts_with("ui_item_icons_")
        || dir_name == "item_icons_no_tiny_hub"
        || dir_name == "item_icons_stack_hub"
    {
        Some(AssetKind::ItemIcon)
    } else if dir_name.starts_with("ui_medal_icons_") {
        Some(AssetKind::MedalIcon)
    } else if dir_name.starts_with("ui_home_act_banner_gacha_")
        || dir_name == "home_banner_gacha_hub"
    {
        Some(AssetKind::GachaBanner)
    } else if dir_name == "[uc]homeentry" {
        Some(AssetKind::EventBanner)
    } else if dir_name.starts_with("loadingillusts_") {
        Some(AssetKind::LoadingIllust)
    } else if dir_name.starts_with("story_review_chapter_bg_")
        || dir_name.starts_with("story_review_mini_activity_bg_")
    {
        Some(AssetKind::StoryEntryPic)
    } else if dir_name.starts_with("ui_kv_img_") {
        Some(AssetKind::BrandKv)
    } else if dir_name.starts_with("ui_brand_image_hub_") {
        Some(AssetKind::BrandLogo)
    } else {
        None
    }
}

/// Lowercase alphanumerics only: `picClassic_Attain_57_0_2` and
/// `picClassicAttain_57_0_2` both become `picclassicattain5702`.
pub fn normalize_stem(stem: &str) -> String {
    stem.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Returns the stem with a trailing `_<digits>` removed, if present
/// (`b_char_kong_2` -> `b_char_kong`). `None` for stems like `b_char_kong` or
/// `p_atk_arrow_n` where the suffix after the last `_` is not all digits.
fn strip_numeric_suffix(stem: &str) -> Option<&str> {
    let idx = stem.rfind('_')?;
    let base = &stem[..idx];
    let suffix = &stem[idx + 1..];
    if !base.is_empty() && !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit()) {
        Some(base)
    } else {
        None
    }
}

/// Build the served URL for a disk-relative audio path, percent-encoding `#`
/// (present in skin folder names like `char_1012_skadi2_iteration#2`) so it is
/// not treated as a URL fragment.
fn audio_url(rel: &str) -> String {
    format!("/audio/sound_beta_2/{}", rel.replace('#', "%23"))
}

fn grandparent_name(path: &Path) -> Option<&str> {
    path.parent()?.parent()?.file_name()?.to_str()
}

fn png_stem(entry: &std::fs::DirEntry) -> Option<String> {
    let name = entry.file_name();
    let s = name.to_str()?;
    s.strip_suffix(".png").map(String::from)
}
