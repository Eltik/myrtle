use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use myrtle_gamedata_raw::tables::enemy::EnemyHandbook;

use crate::model::chibi::{ChibiCharacter, ChibiData, ChibiSkin, SpineFiles};

const ANIM_DIRS: &[(&str, &str)] = &[
    ("BattleFront", "front"),
    ("BattleBack", "back"),
    ("BattleDown", "down"),
    ("Building", "dorm"),
    ("DynIllust", "dynamic"),
];

const KNOWN_PREFIXES: &[&str] = &["dyn_illust_", "dyn_portrait_", "build_"];

fn spine_folders<'a>(asset_paths: &'a [String], anim_dir: &str) -> BTreeMap<&'a str, Vec<&'a str>> {
    let prefix = format!("spine/{anim_dir}/");
    let mut folders: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for rel in asset_paths {
        let Some((dir, file)) = rel
            .strip_prefix(&prefix)
            .and_then(|rest| rest.split_once('/'))
            .filter(|(_, file)| !file.contains('/'))
        else {
            continue;
        };
        folders.entry(dir).or_default().push(file);
    }

    folders
}

pub fn init_chibi_data(asset_paths: &[String]) -> ChibiData {
    let mut characters: HashMap<String, ChibiCharacter> = HashMap::new();

    for &(anim_dir_name, anim_key) in ANIM_DIRS {
        let is_dyn_illust = anim_dir_name == "DynIllust";

        for (dir_name, files) in spine_folders(asset_paths, anim_dir_name) {
            let base_url = format!("/spine/{anim_dir_name}/{dir_name}");
            let all_sets = collect_all_spine_sets(&files, &base_url);

            if all_sets.is_empty() {
                continue;
            }

            let (char_id, _) = parse_skin_identity(dir_name);

            let character = characters
                .entry(char_id.clone())
                .or_insert_with(|| ChibiCharacter {
                    operator_code: char_id.clone(),
                    name: char_id.clone(),
                    path: char_id.clone(),
                    skins: Vec::new(),
                });

            let assignments = if is_dyn_illust {
                resolve_dyn_illust_skins(all_sets, dir_name, &char_id)
            } else {
                resolve_dir_skins(all_sets, &char_id)
            };

            for (skin_name, spine) in assignments {
                let skin = get_or_create_skin(character, &skin_name, &base_url);
                skin.animation_types.insert(anim_key.to_owned(), spine);
                skin.has_spine_data = true;
            }
        }
    }

    let mut characters: Vec<ChibiCharacter> = characters.into_values().collect();
    sort_characters(&mut characters);
    let characters_arc: Vec<Arc<ChibiCharacter>> = characters.into_iter().map(Arc::new).collect();
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

pub fn init_enemy_chibi_data(asset_paths: &[String], enemies: &EnemyHandbook) -> ChibiData {
    let mut characters: HashMap<String, ChibiCharacter> = HashMap::new();

    for (dir_name, files) in spine_folders(asset_paths, "Enemy") {
        let base_url = format!("/spine/Enemy/{dir_name}");
        for (stem, spine) in collect_all_spine_sets(&files, &base_url) {
            let name = enemies
                .enemy_data
                .get(&stem)
                .map_or_else(|| stem.clone(), |e| e.name.clone());

            let mut animation_types = HashMap::new();
            animation_types.insert("front".to_owned(), spine);

            characters.insert(
                stem.clone(),
                ChibiCharacter {
                    operator_code: stem.clone(),
                    name,
                    path: stem,
                    skins: vec![ChibiSkin {
                        name: "default".to_owned(),
                        path: base_url.clone(),
                        has_spine_data: true,
                        animation_types,
                    }],
                },
            );
        }
    }

    let mut characters: Vec<ChibiCharacter> = characters.into_values().collect();
    sort_characters(&mut characters);
    let characters_arc: Vec<Arc<ChibiCharacter>> = characters.into_iter().map(Arc::new).collect();
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

#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn collect_all_spine_sets(files: &[&str], base_url: &str) -> Vec<(String, SpineFiles)> {
    let mut groups: BTreeMap<String, SpineFiles> = BTreeMap::new();

    for &name in files {
        let lower = name.to_lowercase();
        if lower.contains("[alpha]") || lower.contains("[mask]") {
            continue;
        }

        let ext = if lower.ends_with(".atlas") {
            "atlas"
        } else if lower.ends_with(".skel") {
            "skel"
        } else if lower.ends_with(".png") {
            "png"
        } else {
            continue;
        };

        let stem = &name[..name.len() - ext.len() - 1];

        let url = format!("{base_url}/{name}");
        let group = groups.entry(stem.to_owned()).or_default();

        match ext {
            "atlas" => group.atlas = Some(url),
            "skel" => group.skel = Some(url),
            "png" => group.png = Some(url),
            _ => {}
        }
    }

    groups
        .into_iter()
        .filter(|(_, files)| files.atlas.is_some() && files.skel.is_some())
        .collect()
}

fn parse_skin_identity(dir_name: &str) -> (String, String) {
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

fn resolve_dyn_illust_skins(
    sets: Vec<(String, SpineFiles)>,
    dir_name: &str,
    char_id: &str,
) -> Vec<(String, SpineFiles)> {
    let (_, dir_skin) = parse_skin_identity(dir_name);
    let char_lower = char_id.to_lowercase();

    let dir_skin_is_numeric = !dir_skin.is_empty() && dir_skin.chars().all(|c| c.is_ascii_digit());

    let mut by_skin: BTreeMap<String, (i32, SpineFiles)> = BTreeMap::new();

    for (stem, files) in sets {
        let cleaned = strip_known_prefix(&stem);
        if is_start_companion(cleaned) {
            continue;
        }

        let skin_name = if cleaned.eq_ignore_ascii_case(&char_lower) || dir_skin_is_numeric {
            "default".to_owned()
        } else {
            dir_skin.clone()
        };

        let priority = if stem.starts_with("dyn_illust_") {
            2
        } else {
            i32::from(stem.starts_with("dyn_portrait_"))
        };

        match by_skin.get(&skin_name) {
            Some((existing, _)) if *existing >= priority => {}
            _ => {
                by_skin.insert(skin_name, (priority, files));
            }
        }
    }

    by_skin.into_iter().map(|(s, (_, f))| (s, f)).collect()
}

fn resolve_dir_skins(sets: Vec<(String, SpineFiles)>, char_id: &str) -> Vec<(String, SpineFiles)> {
    let mut tagged: Vec<(String, bool, SpineFiles)> = sets
        .into_iter()
        .map(|(stem, files)| {
            let (skin, conventional) = derive_skin_name_for_dir(&stem, char_id);
            (skin, conventional, files)
        })
        .collect();

    let has_conventional_default = tagged.iter().any(|(s, c, _)| *c && s == "default");
    let non_conventional_indices: Vec<usize> = tagged
        .iter()
        .enumerate()
        .filter(|(_, (_, c, _))| !*c)
        .map(|(i, _)| i)
        .collect();

    if !has_conventional_default && non_conventional_indices.len() == 1 {
        let i = non_conventional_indices[0];
        tagged[i].0 = "default".to_owned();
        tagged[i].1 = true;
    }

    tagged.into_iter().map(|(s, _, f)| (s, f)).collect()
}

fn derive_skin_name_for_dir(stem: &str, char_id: &str) -> (String, bool) {
    let cleaned = strip_known_prefix(stem);

    let char_lower = char_id.to_lowercase();
    let cleaned_lower = cleaned.to_lowercase();

    if cleaned_lower == char_lower {
        ("default".to_owned(), true)
    } else if let Some(rest_lower) = cleaned_lower.strip_prefix(&format!("{char_lower}_")) {
        // Preserve the original case from `cleaned` rather than the lowercased copy.
        let suffix = &cleaned[cleaned.len() - rest_lower.len()..];
        (suffix.to_owned(), true)
    } else {
        (cleaned.to_owned(), false)
    }
}

fn get_or_create_skin<'a>(
    character: &'a mut ChibiCharacter,
    skin_name: &str,
    base_path: &str,
) -> &'a mut ChibiSkin {
    let idx = character
        .skins
        .iter()
        .position(|s| s.name.eq_ignore_ascii_case(skin_name));
    let i = idx.unwrap_or_else(|| {
        character.skins.push(ChibiSkin {
            name: skin_name.to_owned(),
            path: base_path.to_owned(),
            has_spine_data: true,
            animation_types: HashMap::new(),
        });
        character.skins.len() - 1
    });
    &mut character.skins[i]
}

fn is_start_companion(cleaned: &str) -> bool {
    let lower = cleaned.to_ascii_lowercase();
    if lower.ends_with("_start") {
        return true;
    }
    match lower.rfind("_start#") {
        Some(i) => {
            let suffix = &lower[i + "_start#".len()..];
            !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit())
        }
        None => false,
    }
}

fn strip_known_prefix(stem: &str) -> &str {
    for prefix in KNOWN_PREFIXES {
        if let Some(rest) = stem.strip_prefix(prefix) {
            return rest;
        }
    }
    stem
}

fn sort_characters(characters: &mut [ChibiCharacter]) {
    characters.sort_by(|a, b| a.operator_code.cmp(&b.operator_code));
    for character in characters {
        character
            .skins
            .sort_by(|a, b| (a.name != "default", &a.name).cmp(&(b.name != "default", &b.name)));
    }
}

#[cfg(test)]
mod start_companion_tests {
    use super::is_start_companion;

    #[test]
    fn start_marker_with_and_without_skin_suffix() {
        assert!(is_start_companion("char_003_kalts_boc#6_Start"));
        assert!(is_start_companion("char_2023_ling_nian_Start#12"));
        assert!(is_start_companion("char_2023_ling_nian_start#12"));
        assert!(!is_start_companion("char_2023_ling_nian#12"));
        assert!(!is_start_companion("char_2023_ling_nian_Start#"));
        assert!(!is_start_companion("char_2023_ling_nian_Starter#12"));
    }
}
