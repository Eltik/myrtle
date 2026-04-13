use std::{collections::HashMap, path::Path, sync::Arc};

use crate::core::gamedata::types::chibi::{ChibiCharacter, ChibiData, ChibiSkin, SpineFiles};

const ANIM_DIRS: &[(&str, &str)] = &[
    ("BattleFront", "front"),
    ("BattleBack", "back"),
    ("Building", "dorm"),
    ("DynIllust", "dynamic"),
];

pub fn init_chibi_data(assets_dir: &Path) -> ChibiData {
    let spine_dir = assets_dir.join("spine");
    let mut characters: HashMap<String, ChibiCharacter> = HashMap::new();

    for &(anim_dir_name, anim_key) in ANIM_DIRS {
        let anim_path = spine_dir.join(anim_dir_name);
        let Ok(entries) = std::fs::read_dir(&anim_path) else {
            continue;
        };

        let is_dyn_illust = anim_dir_name == "DynIllust";

        for entry in entries.flatten() {
            if !entry.file_type().is_ok_and(|t| t.is_dir()) {
                continue;
            }
            let Some(dir_name) = entry.file_name().to_str().map(String::from) else {
                continue;
            };

            let base_url = format!("/spine/{anim_dir_name}/{dir_name}");
            let all_sets = collect_all_spine_sets(&entry.path(), &base_url);

            if all_sets.is_empty() {
                continue;
            }

            let (char_id, skin_name) = parse_skin_identity(&dir_name);

            let character = characters
                .entry(char_id.clone())
                .or_insert_with(|| ChibiCharacter {
                    operator_code: char_id.clone(),
                    name: char_id.clone(),
                    path: char_id.clone(),
                    skins: Vec::new(),
                });

            if is_dyn_illust {
                if let Some(spine) = pick_best_spine_set(all_sets, &dir_name) {
                    let skin = get_or_create_skin(character, &skin_name, &base_url);
                    skin.animation_types.insert(anim_key.to_owned(), spine);
                    skin.has_spine_data = true;
                }
            } else {
                for (stem, spine) in all_sets {
                    let skin_name = derive_skin_name(&stem, &char_id);
                    let skin = get_or_create_skin(character, &skin_name, &base_url);
                    skin.animation_types.insert(anim_key.to_owned(), spine);
                    skin.has_spine_data = true;
                }
            }
        }
    }

    let characters_arc: Vec<Arc<ChibiCharacter>> = characters.into_values().map(Arc::new).collect();
    let by_operator: HashMap<String, Arc<ChibiCharacter>> = characters_arc
        .iter()
        .map(|c| (c.operator_code.clone(), Arc::clone(c)))
        .collect();

    ChibiData {
        raw_items: Vec::new(),
        characters: characters_arc,
        by_operator,
    }
}

/// Helper to find or create a skin entry on a character
fn get_or_create_skin<'a>(
    character: &'a mut ChibiCharacter,
    skin_name: &str,
    base_path: &str,
) -> &'a mut ChibiSkin {
    let idx = character.skins.iter().position(|s| s.name == skin_name);
    match idx {
        Some(i) => &mut character.skins[i],
        None => {
            character.skins.push(ChibiSkin {
                name: skin_name.to_owned(),
                path: base_path.to_owned(),
                has_spine_data: true,
                animation_types: HashMap::new(),
            });
            character.skins.last_mut().unwrap()
        }
    }
}

/// Extract char_id and skin name from a directory name.
/// "char_002_amiya" → ("char_002_amiya", "default")
/// "char_002_amiya_epoque#4" → ("char_002_amiya", "epoque#4")
/// "char_003_kalts_sale#14" → ("char_003_kalts", "sale#14")
fn parse_skin_identity(dir_name: &str) -> (String, String) {
    // Pattern: char_{digits}_{name} optionally followed by _{skin_suffix}
    // Find the third underscore group boundary
    let parts: Vec<&str> = dir_name.splitn(4, '_').collect();
    if parts.len() >= 3 && parts[0] == "char" {
        let char_id = format!("{}_{}_{}", parts[0], parts[1], parts[2]);
        let skin_name = if parts.len() == 4 {
            parts[3].to_owned()
        } else {
            "default".to_owned()
        };
        (char_id, skin_name)
    } else {
        (dir_name.to_owned(), "default".to_owned())
    }
}

/// Collect .atlas, .skel, .png from a directory, returning full paths.
fn collect_all_spine_sets(dir: &Path, base_url: &str) -> Vec<(String, SpineFiles)> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut groups: HashMap<String, SpineFiles> = HashMap::new();

    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(String::from) else {
            continue;
        };
        let lower = name.to_lowercase();
        if lower.contains("[alpha]") || lower.contains("[mask]") {
            continue;
        }

        let (stem, ext) = if let Some(s) = lower.strip_suffix(".atlas") {
            (s, "atlas")
        } else if let Some(s) = lower.strip_suffix(".skel") {
            (s, "skel")
        } else if let Some(s) = lower.strip_suffix(".png") {
            (s, "png")
        } else {
            continue;
        };

        let url = format!("{base_url}/{name}");
        let group = groups.entry(stem.to_owned()).or_default();

        match ext {
            "atlas" => {
                group.atlas = Some(url);
            }
            "skel" => {
                group.skel = Some(url);
            }
            "png" => {
                group.png = Some(url);
            }
            _ => {}
        }
    }

    groups
        .into_iter()
        .filter(|(_, files)| files.atlas.is_some() && files.skel.is_some())
        .collect()
}

/// Given a file stem like "char_4087_ines_boc#8" and char_id "char_4087_ines",
/// returns the skin name "boc#8". If the stem IS the char_id, returns "default".
/// For DynIllust stems like "dyn_illust_char_4087_ines_boc#8", strips the prefix first.
fn derive_skin_name(stem: &str, char_id: &str) -> String {
    // Strip common DynIllust prefixes
    let cleaned = stem
        .strip_prefix("dyn_illust_")
        .or_else(|| stem.strip_prefix("dyn_portrait_"))
        .unwrap_or(stem);

    let char_lower = char_id.to_lowercase();
    let cleaned_lower = cleaned.to_lowercase();

    if cleaned_lower == char_lower {
        "default".to_owned()
    } else if let Some(suffix) = cleaned_lower.strip_prefix(&format!("{char_lower}_")) {
        suffix.to_owned()
    } else {
        // Can't parse — use the stem as-is
        cleaned.to_owned()
    }
}

/// Pick the best spine file set for a given base name.
///
/// Scoring (higher = better match):
///   - Exact stem match with base_name         → 100
///   - Stem ends with base_name (e.g. dyn_illust_{base}) → 50
///   - Stem contains base_name                  → 10
///   - Prefer non-"portrait" stems              → +5 bonus
///
fn pick_best_spine_set(sets: Vec<(String, SpineFiles)>, base_name: &str) -> Option<SpineFiles> {
    if sets.is_empty() {
        return None;
    }

    let base_lower = base_name.to_lowercase();

    sets.into_iter()
        .map(|(stem, files)| {
            let mut score: i32 = 0;

            if stem == base_lower {
                score = 100;
            } else if stem.ends_with(&base_lower) {
                score = 50;
            } else if stem.contains(&base_lower) {
                score = 10;
            }

            // Prefer non-portrait variants (dyn_illust over dyn_portrait)
            if !stem.contains("portrait") {
                score += 5;
            }

            (score, files)
        })
        .filter(|(score, _)| *score > 0)
        .max_by_key(|(score, _)| *score)
        .map(|(_, files)| files)
}
