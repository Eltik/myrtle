//! Build the enemy -> stages inverted index.
//!
//! This walks every level file under `gamedata/levels/`, tallies the enemies it
//! references (`EnemyDbRefs[].Id` plus `Waves[].Fragments[].Actions[]` SPAWN
//! keys/counts), classifies the level via [`StageClassifier`], and inverts the
//! result into `enemy_id -> Vec<EnemyStageRef>`. It also folds in boss-only
//! declarations from `activity_table` (some bosses never appear in a level
//! file). All stage taxonomy/labelling lives in [`stage_class`].

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::stage_class::{StageClassifier, StageInfo};
use crate::core::gamedata::types::activity::ActivityBasicInfo;
use crate::core::gamedata::types::enemy_stages::{EnemyStageIndex, EnemyStageRef};
use crate::core::gamedata::types::stage::Stage;
use crate::core::gamedata::types::zone::Zone;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LevelFile {
    #[serde(default)]
    enemy_db_refs: Vec<DbRef>,
    #[serde(default)]
    waves: Vec<Wave>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DbRef {
    id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Wave {
    #[serde(default)]
    fragments: Vec<Fragment>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Fragment {
    #[serde(default)]
    actions: Vec<Action>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Action {
    #[serde(default)]
    action_type: String,
    #[serde(default)]
    key: Option<String>,
    #[serde(default)]
    count: i64,
}

/// Parse one level file into `enemy_id -> spawn count`. `EnemyDbRefs` seeds a
/// count of 0 (declared but maybe summon-only); `SPAWN` actions add their count.
fn tally_level(path: &Path) -> Option<HashMap<String, u32>> {
    let bytes = std::fs::read(path).ok()?;
    let level: LevelFile = serde_json::from_slice(&bytes).ok()?;

    let mut counts: HashMap<String, u32> = HashMap::new();
    for r in &level.enemy_db_refs {
        counts.entry(r.id.clone()).or_insert(0);
    }
    for wave in &level.waves {
        for frag in &wave.fragments {
            for action in &frag.actions {
                if action.action_type != "SPAWN" {
                    continue;
                }
                let Some(key) = action.key.as_deref() else {
                    continue;
                };
                if !key.starts_with("enemy_") {
                    continue;
                }
                let add = u32::try_from(action.count.max(0)).unwrap_or(0);
                *counts.entry(key.to_owned()).or_insert(0) += add;
            }
        }
    }
    Some(counts)
}

/// Recursively collect every `*.json` level file, skipping the `enemydata/`
/// stats directory and `levelreplacers/` (IS layout variants that duplicate a
/// base node).
fn collect_level_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
            if name == "enemydata" || name == "levelreplacers" {
                continue;
            }
            collect_level_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("json") {
            out.push(path);
        }
    }
}

/// The relative level id of a file under `levels_dir`: lowercased, forward
/// slashes, no extension (matches the keys [`StageClassifier`] expects).
fn relative_level_id(levels_dir: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(levels_dir).ok()?;
    Some(
        rel.with_extension("")
            .to_string_lossy()
            .to_lowercase()
            .replace('\\', "/"),
    )
}

fn push_ref(
    index: &mut EnemyStageIndex,
    enemy_id: String,
    stage_id: String,
    info: &StageInfo,
    count: u32,
) {
    index.entry(enemy_id).or_default().push(EnemyStageRef {
        stage_id,
        code: info.code.clone(),
        zone_id: info.zone_id.clone(),
        zone_name: info.zone_name.clone(),
        category: info.category.to_owned(),
        group: info.group.to_owned(),
        stage_name: info.stage_name.clone(),
        is_hard: info.is_hard,
        count,
    });
}

/// Build the `enemy_id -> stages` index from the level files plus boss-only
/// declarations.
pub fn build_enemy_stage_index(
    levels_dir: &Path,
    data_dir: &Path,
    stages: &HashMap<String, Stage>,
    zones: &HashMap<String, Zone>,
    activities: &HashMap<String, ActivityBasicInfo>,
) -> EnemyStageIndex {
    let classifier = StageClassifier::new(data_dir, stages, zones, activities);

    let mut files = Vec::new();
    collect_level_files(levels_dir, &mut files);

    let mut index: EnemyStageIndex = HashMap::new();

    for path in &files {
        let Some(rel) = relative_level_id(levels_dir, path) else {
            continue;
        };
        let Some(counts) = tally_level(path) else {
            continue;
        };
        let info = classifier.classify_level(&rel);
        let stage_id = rel.rsplit('/').next().unwrap_or(&rel).to_owned();
        for (enemy_id, count) in counts {
            push_ref(&mut index, enemy_id, stage_id.clone(), &info, count);
        }
    }

    // Boss declarations that never appear in a level file.
    for (enemy_id, info) in classifier.boss_appearances(data_dir) {
        let already = index.get(&enemy_id).is_some_and(|refs| {
            refs.iter()
                .any(|r| r.zone_id == info.zone_id && r.code == info.code)
        });
        if already {
            continue;
        }
        // No level file, so key the row by zone+code.
        let stage_id = format!("{}/{}", info.zone_id, info.code);
        push_ref(&mut index, enemy_id, stage_id, &info, 0);
    }

    for refs in index.values_mut() {
        refs.sort_by(|a, b| {
            a.is_hard
                .cmp(&b.is_hard)
                .then_with(|| a.code.cmp(&b.code))
                .then_with(|| a.stage_id.cmp(&b.stage_id))
        });
    }

    index
}
