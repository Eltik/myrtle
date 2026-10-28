use std::collections::{HashMap, HashSet};

use myrtle_gamedata_raw::tables::story::StorySpriteFiles;

use crate::story_sprites::StorySprites;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum AssetKind {
    Avatar,
    Portrait,
    SkinPortrait,
    SkillIcon,
    ModuleIcon,
    ModuleBig,
    EnemyIcon,
    ItemIcon,
    MedalIcon,
    GachaBanner,
    EventBanner,
    StoryEntryPic,
    LoadingIllust,
    BrandKv,
    BrandLogo,
    FurnitureIcon,
    StorylineArt,
    ProfessionIcon,
    SubProfessionIcon,
    CampLogo,
    TeamIcon,
    AutoChessIcon,
    RoguelikeItem,
    RoguelikeTheme,
    ModuleType,
    HomeBackground,
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
    AssetKind::FurnitureIcon,
    AssetKind::StorylineArt,
    AssetKind::ProfessionIcon,
    AssetKind::SubProfessionIcon,
    AssetKind::CampLogo,
    AssetKind::TeamIcon,
    AssetKind::AutoChessIcon,
    AssetKind::RoguelikeItem,
    AssetKind::RoguelikeTheme,
    AssetKind::ModuleType,
    AssetKind::HomeBackground,
];

const AUDIO_ROOT: &str = "audio/audio/sound_beta_2/";

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct AssetIndex {
    map: HashMap<AssetKind, HashMap<String, String>>,
    chararts: HashMap<String, Vec<String>>,
    skinpacks: HashMap<String, Vec<String>>,
    audio_by_name: HashMap<String, Vec<String>>,
    audio_by_base: HashMap<String, Vec<String>>,
    audio_paths: HashSet<String>,
    story_sprites: StorySprites,
}

impl AssetIndex {
    pub fn from_paths(paths: &[String]) -> Self {
        let mut idx = Self::default();
        for kind in ALL_KINDS {
            idx.map.insert(*kind, HashMap::new());
        }

        for rel in paths {
            if !rel.starts_with("textures/") {
                continue;
            }

            let Some(stem) = stem_with_ext(rel, "png") else {
                continue;
            };

            let rel_path = format!("/{rel}");

            let Some(parent) = ancestor(rel, 1) else {
                continue;
            };

            let grandparent = ancestor(rel, 2);

            if let Some(kind) = classify_dir(parent) {
                let key = match kind {
                    AssetKind::GachaBanner => normalize_stem(stem),
                    AssetKind::StoryEntryPic => stem
                        .strip_prefix("storyEntryPic_")
                        .unwrap_or(stem)
                        .to_owned(),
                    AssetKind::CampLogo => stem
                        .to_ascii_lowercase(),
                    _ => stem.to_owned(),
                };
                idx.kind_mut(kind).insert(key, rel_path.clone());
            } else if grandparent == Some("rglktopic") {
                idx.kind_mut(AssetKind::RoguelikeItem)
                    .entry(stem.to_owned())
                    .or_insert_with(|| rel_path.clone());
            } else if parent == "commonassets" && grandparent == Some("activity") {
                idx.kind_mut(AssetKind::ItemIcon)
                    .entry(stem.to_owned())
                    .or_insert_with(|| rel_path.clone());
            }

            if parent.starts_with("char_") && grandparent == Some("chararts") {
                let suffixes = idx.chararts.entry(parent.to_owned()).or_default();
                if let Some(suffix) = stem.strip_prefix(parent) {
                    suffixes.push(suffix.to_owned());
                }
            }

            if grandparent == Some("skinpack") && parent.starts_with("char_") {
                idx.skinpacks
                    .entry(parent.to_owned())
                    .or_default()
                    .push(rel_path);
            }
        }

        for (sub, kind) in [
            ("gacha", AssetKind::GachaBanner),
            ("event", AssetKind::EventBanner)
        ] {
            let prefix = format!("derived/release-art/{sub}");
            let map = idx.kind_mut(kind);
            for rel in paths {
                let Some(stem) = flat_png_stem(rel, &prefix) else {
                    continue;
                };
                let key = if kind == AssetKind::GachaBanner {
                    normalize_stem(&format!("pic{stem}"))
                } else {
                    stem.to_owned()
                };
                map.entry(key)
                    .or_insert_with(|| format!("/derived/release-art/{sub}/{stem}.png"));
            }
        }

        let portraits = idx.kind_mut(AssetKind::Portrait);
        for rel in paths {
            if let Some(stem) = flat_png_stem(rel, "portraits/") {
                portraits.insert(stem.to_owned(), format!("/portraits/{stem}.png"));
            }
        }

        for path in paths {
            let Some(rel) = path.strip_prefix(AUDIO_ROOT) else {
                continue;
            };
            let Some(stem) = stem_with_ext(rel, "ogg") else {
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

    pub fn build(paths: &[String], sprites: &StorySpriteFiles, scripts: &[String]) -> Self {
        let mut idx = Self::from_paths(paths);
        idx.story_sprites = StorySprites::build(sprites, scripts, &story_folders(paths));
        idx
    }

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
        self.map.get(&kind)?.get(name).map(String::as_str)
    }

    pub fn gacha_banner_path(&self, pool_id: &str) -> Option<&str> {
        let key = normalize_stem(&format!("pic{pool_id}"));
        self.path(AssetKind::GachaBanner, &key)
    }

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
        self.chararts.get(char_id).map(Vec::as_slice)
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
        self.skinpacks.get(char_id).map(Vec::as_slice)
    }

    pub const fn story_sprites(&self) -> &StorySprites {
        &self.story_sprites
    }

    fn kind_mut(&mut self, kind: AssetKind) -> &mut HashMap<String, String> {
        self.map.entry(kind).or_default()
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
    } else if dir_name.starts_with("fur_icon_") {
        Some(AssetKind::FurnitureIcon)
    } else if dir_name.starts_with("mixstory_") {
        Some(AssetKind::StorylineArt)
    } else if dir_name == "[uc]charcommon" {
        Some(AssetKind::ProfessionIcon)
    } else if dir_name.starts_with("ui_sub_profession_icon_hub_") {
        Some(AssetKind::SubProfessionIcon)
    } else if dir_name.starts_with("ui_camp_logo_") {
        Some(AssetKind::CampLogo)
    } else if dir_name.starts_with("ui_team_icon_h2_") {
        Some(AssetKind::TeamIcon)
    } else if dir_name == "[uc]autochesscommon" {
        Some(AssetKind::AutoChessIcon)
    } else if dir_name.starts_with("ui_roguelike_topic_item_") {
        Some(AssetKind::RoguelikeItem)
    } else if dir_name.starts_with("ui_zone_home_theme_rogue_") {
        Some(AssetKind::RoguelikeTheme)
    } else if dir_name.starts_with("ui_equip_type_direction_hub_") {
        Some(AssetKind::ModuleType)
    } else if dir_name.starts_with("ui_homebackground_preview_") {
        Some(AssetKind::HomeBackground)
    } else {
        None
    }
}

pub fn normalize_stem(stem: &str) -> String {
    stem.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

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

fn story_folders(paths: &[String]) -> HashMap<String, Vec<String>> {
    let mut folders: HashMap<String, Vec<String>> = HashMap::new();
    for rel in paths {
        if !rel.starts_with("textures/") {
            continue;
        }
        let Some(stem) = stem_with_ext(rel, "png") else {
            continue;
        };
        let Some(parent) = ancestor(rel, 1) else {
            continue;
        };
        let grandparent = ancestor(rel, 2);
        if classify_dir(parent).is_some()
            || grandparent == Some("rglktopic")
            || (parent == "commonassets" && grandparent == Some("activity"))
        {
            continue;
        }
        if grandparent == Some("characters") && ancestor(rel, 3) == Some("avg") {
            folders
                .entry(parent.to_owned())
                .or_default()
                .push(stem.to_owned());
        }
    }
    folders
}

fn stem_with_ext<'a>(rel: &'a str, ext: &str) -> Option<&'a str> {
    let name = rel.rsplit('/').next()?;
    let (stem, found) = name.rsplit_once('.')?;
    (found == ext && !stem.is_empty()).then_some(stem)
}

fn ancestor(rel: &str, n: usize) -> Option<&str> {
    rel.rsplit('/').nth(n)
}

fn flat_png_stem<'a>(rel: &'a str, prefix: &str) -> Option<&'a str> {
    let name = rel.strip_prefix(prefix)?;
    if name.contains('/') {
        return None;
    }
    name.strip_suffix(".png")
}

fn audio_url(rel: &str) -> String {
    format!("/audio/sound_beta_2/{}", rel.replace('#', "%23"))
}
