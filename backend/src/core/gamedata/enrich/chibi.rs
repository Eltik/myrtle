use std::{collections::HashMap, path::Path, sync::Arc};

use crate::core::gamedata::types::chibi::{ChibiCharacter, ChibiData, ChibiSkin, SpineFiles};

const ANIM_DIRS: &[(&str, &str)] = &[
    ("BattleFront", "front"),
    ("BattleBack", "back"),
    ("Building", "dorm"),
    ("DynIllust", "dynamic"),
];

/// Prefixes the game uses on file stems to identify the variant flavor.
/// `dyn_illust_` is the canonical dynamic; `dyn_portrait_` is its portrait
/// counterpart and we prefer `dyn_illust_` whenever both exist for a skin.
/// `build_` is used inside `Building/` directories.
const KNOWN_PREFIXES: &[&str] = &["dyn_illust_", "dyn_portrait_", "build_"];

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

            let (char_id, _) = parse_skin_identity(&dir_name);

            let character = characters
                .entry(char_id.clone())
                .or_insert_with(|| ChibiCharacter {
                    operator_code: char_id.clone(),
                    name: char_id.clone(),
                    path: char_id.clone(),
                    skins: Vec::new(),
                });

            let assignments = if is_dyn_illust {
                resolve_dyn_illust_skins(all_sets, &dir_name, &char_id)
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

/// Helper to find or create a skin entry on a character.
fn get_or_create_skin<'a>(
    character: &'a mut ChibiCharacter,
    skin_name: &str,
    base_path: &str,
) -> &'a mut ChibiSkin {
    let idx = character
        .skins
        .iter()
        .position(|s| s.name.eq_ignore_ascii_case(skin_name));
    if let Some(i) = idx {
        &mut character.skins[i]
    } else {
        character.skins.push(ChibiSkin {
            name: skin_name.to_owned(),
            path: base_path.to_owned(),
            has_spine_data: true,
            animation_types: HashMap::new(),
        });
        character.skins.last_mut().unwrap()
    }
}

/// Extract `char_id` and skin name from a directory name.
/// "`char_002_amiya`" → ("`char_002_amiya`", "default")
/// "`char_002_amiya_epoque#4`" → ("`char_002_amiya`", "epoque#4")
/// "`char_003_kalts_sale#14`" → ("`char_003_kalts`", "sale#14")
/// Names that don't begin with "char_" (e.g. "token_*") are treated as a
/// single opaque `char_id` with a "default" skin.
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

/// Collect .atlas, .skel, .png from a directory, returning each spine set
/// keyed by its file stem. [alpha]/[mask] companion textures are excluded
/// and any stem missing both atlas+skel is dropped (incomplete set).
//
// `name` is lowercased into `lower` before the extension checks below, so those
// `ends_with` comparisons are already case-insensitive (the lint's concern is moot).
#[allow(clippy::case_sensitive_file_extension_comparisons)]
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

fn strip_known_prefix(stem: &str) -> &str {
    for prefix in KNOWN_PREFIXES {
        if let Some(rest) = stem.strip_prefix(prefix) {
            return rest;
        }
    }
    stem
}

/// Derive a skin name for a non-DynIllust spine stem.
/// Returns `(skin_name, is_conventional)`. `is_conventional` is true when
/// the stem matched the directory's `char_id` (either as the default or as a
/// `{char_id}_{suffix}` variant). Non-conventional stems mean the file
/// inside this character's folder has an unrelated name (e.g.
/// `BattleFront/char_008_owl/char_502_nblade.atlas` — owl reuses Blade's
/// chibi); the caller may decide to promote them to "default".
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

/// Resolve all spine sets in a non-DynIllust directory into (skin, files)
/// pairs. When a directory contains exactly one stem that does not follow
/// the `{char_id}` / `{char_id}_{suffix}` convention and no conventional
/// default exists, that stem is promoted to "default" — which covers cases
/// like Owl reusing Blade's chibi or Liskarm-the-typo files living under
/// `char_107_liskam/`.
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

/// Resolve `DynIllust` directory contents.
///
/// One `DynIllust` folder corresponds to exactly one skin variant, so the
/// folder name is the canonical skin identifier (e.g. dir
/// `char_4087_ines_ambiencesynesthesia#5/` is the `ambiencesynesthesia#5`
/// skin's dynamic — even when the file inside drops the `#5` and case).
///
/// The one exception is "duplicate-of-default" folders such as
/// `char_1012_skadi2_2/` whose file is just `dyn_illust_char_1012_skadi2.atlas`
/// (no `_2` suffix in the file name). The trailing `_2` is a disambiguator,
/// not a skin variant, and the spine actually belongs to the default skin —
/// we detect this by checking whether the file's stripped stem matches the
/// dir's `char_id` with no skin suffix at all.
///
/// Across files within one folder we prefer `dyn_illust_` over
/// `dyn_portrait_` and skip `_Start` intro-animation companions.
fn resolve_dyn_illust_skins(
    sets: Vec<(String, SpineFiles)>,
    dir_name: &str,
    char_id: &str,
) -> Vec<(String, SpineFiles)> {
    let (_, dir_skin) = parse_skin_identity(dir_name);
    let char_lower = char_id.to_lowercase();

    // Pure numeric dir suffixes (`_2`, `_3`, …) are duplicate-of-default
    // counters, not real skin variants — Arknights' real skin IDs always
    // use named tags like `sale#X` / `boc#X` / `iteration#N`. Folders such
    // as `char_2014_nian_2/` whose file is `dyn_illust_char_2014_nian2.atlas`
    // (concatenated, not the bare char_id) only fall through to this check.
    let dir_skin_is_numeric = !dir_skin.is_empty() && dir_skin.chars().all(|c| c.is_ascii_digit());

    let mut by_skin: HashMap<String, (i32, SpineFiles)> = HashMap::new();

    for (stem, files) in sets {
        let cleaned = strip_known_prefix(&stem);
        if cleaned.ends_with("_Start") || cleaned.ends_with("_start") {
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
