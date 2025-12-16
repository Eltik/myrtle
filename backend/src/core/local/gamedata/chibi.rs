use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::core::local::types::chibi::{
    AnimationType, ChibiCharacter, ChibiData, ChibiSkin, SpineFiles,
};

// Subdirectories for different sprite types
const ANIM_SUBDIRS: [(&str, AnimationType); 3] = [
    ("BattleFront", AnimationType::Front),
    ("BattleBack", AnimationType::Back),
    ("Building", AnimationType::Dorm),
];

/// Crawl local chibi asset directories and build chibi data
pub fn init_chibi_data(assets_dir: &Path) -> ChibiData {
    // Assets are in upk/ subdirectory
    let upk_dir = assets_dir.join("upk");

    let chararts_path = upk_dir.join("chararts");
    let skinpack_path = upk_dir.join("skinpack");
    let dynchars_path = upk_dir.join("arts").join("dynchars");

    let mut characters: HashMap<String, ChibiCharacter> = HashMap::new();

    // Process chararts (base skins)
    if chararts_path.exists() {
        process_chararts(&chararts_path, &mut characters);
    }

    // Process skinpack (operator skins)
    if skinpack_path.exists() {
        process_skinpack(&skinpack_path, &mut characters);
    }

    // Process dynchars (dynamic illustrations)
    if dynchars_path.exists() {
        process_dynchars(&dynchars_path, &mut characters);
    }

    let characters_vec: Vec<ChibiCharacter> = characters.into_values().collect();

    // Build lookup map
    let by_operator: HashMap<String, ChibiCharacter> = characters_vec
        .iter()
        .map(|c| (c.operator_code.clone(), c.clone()))
        .collect();

    println!(
        "Chibi data loaded: {} operators with spine data",
        characters_vec.len()
    );

    ChibiData {
        raw_items: Vec::new(),
        characters: characters_vec,
        by_operator,
    }
}

/// Process chararts directory (base skins)
/// Structure: chararts/{char_id}/{AnimType}/{char_id}/*.atlas, *.skel, *.png
fn process_chararts(chararts_path: &Path, characters: &mut HashMap<String, ChibiCharacter>) {
    if let Ok(entries) = fs::read_dir(chararts_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let char_id = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) if name.starts_with("char_") => name.to_string(),
                _ => continue,
            };

            // Initialize character if not exists
            let character = characters
                .entry(char_id.clone())
                .or_insert_with(|| ChibiCharacter {
                    operator_code: char_id.clone(),
                    name: char_id.clone(),
                    path: char_id.clone(),
                    skins: Vec::new(),
                });

            // Create or find default skin
            let default_skin = character
                .skins
                .iter_mut()
                .find(|s| s.name == "default")
                .map(|s| s as *mut ChibiSkin);

            let default_skin = if let Some(ptr) = default_skin {
                unsafe { &mut *ptr }
            } else {
                character.skins.push(ChibiSkin {
                    name: "default".to_string(),
                    path: format!("/upk/chararts/{}", char_id),
                    has_spine_data: false,
                    animation_types: HashMap::new(),
                });
                character.skins.last_mut().unwrap()
            };

            // Process each animation type subdirectory
            for (anim_dir, anim_type) in &ANIM_SUBDIRS {
                let anim_path = path.join(anim_dir);
                if !anim_path.exists() {
                    continue;
                }

                // Look for the char_id subdirectory inside the animation type
                let spine_dir = anim_path.join(&char_id);
                if spine_dir.exists() && spine_dir.is_dir() {
                    if let Some(spine_files) = collect_spine_files(&spine_dir) {
                        let anim_key = anim_type.as_str().to_string();
                        // Format paths for the API
                        let formatted = SpineFiles {
                            atlas: spine_files.atlas.map(|f| {
                                format!("/upk/chararts/{}/{}/{}/{}", char_id, anim_dir, char_id, f)
                            }),
                            skel: spine_files.skel.map(|f| {
                                format!("/upk/chararts/{}/{}/{}/{}", char_id, anim_dir, char_id, f)
                            }),
                            png: spine_files.png.map(|f| {
                                format!("/upk/chararts/{}/{}/{}/{}", char_id, anim_dir, char_id, f)
                            }),
                        };
                        default_skin.animation_types.insert(anim_key, formatted);
                        default_skin.has_spine_data = true;
                    }
                }
            }
        }
    }
}

/// Process skinpack directory (operator skins)
/// Structure: skinpack/{char_id}/{AnimType}/{skin_id}/*.atlas, *.skel, *.png
fn process_skinpack(skinpack_path: &Path, characters: &mut HashMap<String, ChibiCharacter>) {
    if let Ok(entries) = fs::read_dir(skinpack_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let char_id = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) if name.starts_with("char_") => name.to_string(),
                _ => continue,
            };

            // Initialize character if not exists
            let character = characters
                .entry(char_id.clone())
                .or_insert_with(|| ChibiCharacter {
                    operator_code: char_id.clone(),
                    name: char_id.clone(),
                    path: char_id.clone(),
                    skins: Vec::new(),
                });

            // Collect all skin IDs from all animation type directories
            let mut skin_ids: HashMap<String, HashMap<AnimationType, SpineFiles>> = HashMap::new();

            for (anim_dir, anim_type) in &ANIM_SUBDIRS {
                let anim_path = path.join(anim_dir);
                if !anim_path.exists() {
                    continue;
                }

                // Each subdirectory in the anim_path is a skin
                if let Ok(skin_entries) = fs::read_dir(&anim_path) {
                    for skin_entry in skin_entries.flatten() {
                        let skin_path = skin_entry.path();
                        if !skin_path.is_dir() {
                            continue;
                        }

                        let dir_name = match skin_path.file_name().and_then(|n| n.to_str()) {
                            Some(name) => name.to_string(),
                            None => continue,
                        };

                        // Normalize skin_id for grouping: Building directories use "build_" prefix
                        // e.g., "build_char_002_amiya_epoque#4" -> "char_002_amiya_epoque#4"
                        let skin_id = if dir_name.starts_with("build_") {
                            dir_name.strip_prefix("build_").unwrap().to_string()
                        } else {
                            dir_name.clone()
                        };

                        if let Some(spine_files) = collect_spine_files(&skin_path) {
                            // Use dir_name (original directory) for paths, skin_id (normalized) for grouping
                            let formatted = SpineFiles {
                                atlas: spine_files.atlas.map(|f| {
                                    format!(
                                        "/upk/skinpack/{}/{}/{}/{}",
                                        char_id, anim_dir, dir_name, f
                                    )
                                }),
                                skel: spine_files.skel.map(|f| {
                                    format!(
                                        "/upk/skinpack/{}/{}/{}/{}",
                                        char_id, anim_dir, dir_name, f
                                    )
                                }),
                                png: spine_files.png.map(|f| {
                                    format!(
                                        "/upk/skinpack/{}/{}/{}/{}",
                                        char_id, anim_dir, dir_name, f
                                    )
                                }),
                            };

                            skin_ids
                                .entry(skin_id)
                                .or_insert_with(HashMap::new)
                                .insert(*anim_type, formatted);
                        }
                    }
                }
            }

            // Add collected skins to character
            for (skin_id, anim_types) in skin_ids {
                // Extract skin name from skin_id (e.g., "char_002_amiya_epoque#4" -> "epoque#4")
                let skin_name = extract_skin_name(&skin_id, &char_id);

                let skin = ChibiSkin {
                    name: skin_name,
                    path: format!("/upk/skinpack/{}", char_id),
                    has_spine_data: true,
                    animation_types: anim_types
                        .into_iter()
                        .map(|(k, v)| (k.as_str().to_string(), v))
                        .collect(),
                };

                character.skins.push(skin);
            }
        }
    }
}

/// Process dynchars directory (dynamic illustrations)
/// Structure: dynchars/{skin_id}/*.atlas, *.skel, *.png
fn process_dynchars(dynchars_path: &Path, characters: &mut HashMap<String, ChibiCharacter>) {
    if let Ok(entries) = fs::read_dir(dynchars_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let skin_id = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            // Extract char_id from skin_id (e.g., "char_003_kalts_boc#6" -> "char_003_kalts")
            let char_id = match extract_char_id_from_skin(&skin_id) {
                Some(id) => id,
                None => continue,
            };

            // Initialize character if not exists
            let character = characters
                .entry(char_id.clone())
                .or_insert_with(|| ChibiCharacter {
                    operator_code: char_id.clone(),
                    name: char_id.clone(),
                    path: char_id.clone(),
                    skins: Vec::new(),
                });

            // Collect spine files from this directory
            if let Some(spine_files) = collect_spine_files(&path) {
                let formatted = SpineFiles {
                    atlas: spine_files
                        .atlas
                        .map(|f| format!("/upk/arts/dynchars/{}/{}", skin_id, f)),
                    skel: spine_files
                        .skel
                        .map(|f| format!("/upk/arts/dynchars/{}/{}", skin_id, f)),
                    png: spine_files
                        .png
                        .map(|f| format!("/upk/arts/dynchars/{}/{}", skin_id, f)),
                };

                // Extract skin name
                let skin_name = extract_skin_name(&skin_id, &char_id);
                let dyn_skin_name = format!("dyn_{}", skin_name);

                let mut anim_types = HashMap::new();
                anim_types.insert("dynamic".to_string(), formatted);

                let skin = ChibiSkin {
                    name: dyn_skin_name,
                    path: format!("/upk/arts/dynchars/{}", skin_id),
                    has_spine_data: true,
                    animation_types: anim_types,
                };

                character.skins.push(skin);
            }
        }
    }
}

/// Spine files found in a directory (just filenames, not full paths)
struct RawSpineFiles {
    atlas: Option<String>,
    skel: Option<String>,
    png: Option<String>,
}

/// Collect spine files (.atlas, .skel, .png) from a directory
fn collect_spine_files(dir: &Path) -> Option<RawSpineFiles> {
    let mut atlas = None;
    let mut skel = None;
    let mut png = None;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                let lower = filename.to_lowercase();
                if lower.ends_with(".atlas") {
                    atlas = Some(filename.to_string());
                } else if lower.ends_with(".skel") {
                    skel = Some(filename.to_string());
                } else if lower.ends_with(".png") && png.is_none() {
                    // Only take the first PNG (avoid multiple PNGs)
                    png = Some(filename.to_string());
                }
            }
        }
    }

    if atlas.is_some() || skel.is_some() || png.is_some() {
        Some(RawSpineFiles { atlas, skel, png })
    } else {
        None
    }
}

/// Extract skin name from a skin_id
/// e.g., "char_002_amiya_epoque#4" with char_id "char_002_amiya" -> "epoque#4"
fn extract_skin_name(skin_id: &str, char_id: &str) -> String {
    if skin_id.starts_with(char_id) && skin_id.len() > char_id.len() + 1 {
        // Remove char_id prefix and the underscore
        skin_id[char_id.len() + 1..].to_string()
    } else {
        skin_id.to_string()
    }
}

/// Extract char_id from a skin_id
/// e.g., "char_003_kalts_boc#6" -> "char_003_kalts"
/// e.g., "char_1012_skadi2_2" -> "char_1012_skadi2"
fn extract_char_id_from_skin(skin_id: &str) -> Option<String> {
    // Pattern: char_{number}_{name} optionally followed by _{something}
    let re = regex::Regex::new(r"^(char_\d+_[a-zA-Z0-9]+)").unwrap();
    re.captures(skin_id)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

/// Extract list of operator codes
pub fn extract_operator_list(characters: &[ChibiCharacter]) -> Vec<String> {
    characters.iter().map(|c| c.operator_code.clone()).collect()
}
