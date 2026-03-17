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

        for entry in entries.flatten() {
            if !entry.file_type().is_ok_and(|t| t.is_dir()) {
                continue;
            }
            let Some(dir_name) = entry.file_name().to_str().map(String::from) else {
                continue;
            };

            let Some(spine) =
                collect_spine_files(&entry.path(), &format!("/spine/{anim_dir_name}/{dir_name}"))
            else {
                continue;
            };

            let (char_id, skin_name) = parse_skin_identity(&dir_name);

            let character = characters
                .entry(char_id.clone())
                .or_insert_with(|| ChibiCharacter {
                    operator_code: char_id.clone(),
                    name: char_id.clone(),
                    path: char_id.clone(),
                    skins: Vec::new(),
                });

            let skin = match character.skins.iter_mut().find(|s| s.name == skin_name) {
                Some(s) => s,
                None => {
                    character.skins.push(ChibiSkin {
                        name: skin_name.clone(),
                        path: format!("/spine/{anim_dir_name}/{dir_name}"),
                        has_spine_data: true,
                        animation_types: HashMap::new(),
                    });
                    character.skins.last_mut().unwrap()
                }
            };

            skin.animation_types.insert(anim_key.to_owned(), spine);
            skin.has_spine_data = true;
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
fn collect_spine_files(dir: &Path, base_url: &str) -> Option<SpineFiles> {
    let mut atlas = None;
    let mut skel = None;
    let mut png = None;

    let Ok(entries) = std::fs::read_dir(dir) else {
        return None;
    };
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(String::from) else {
            continue;
        };
        let lower = name.to_lowercase();
        if lower.ends_with(".atlas") {
            atlas = Some(format!("{base_url}/{name}"));
        } else if lower.ends_with(".skel") {
            skel = Some(format!("{base_url}/{name}"));
        } else if lower.ends_with(".png") && png.is_none() {
            png = Some(format!("{base_url}/{name}"));
        }
    }

    if atlas.is_some() || skel.is_some() || png.is_some() {
        Some(SpineFiles { atlas, skel, png })
    } else {
        None
    }
}
