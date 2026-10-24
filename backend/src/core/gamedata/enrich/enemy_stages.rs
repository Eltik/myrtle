//! Enemy -> stages inverted index.
//!
//! Walks every level file under `gamedata/levels/`, tallies referenced enemies
//! (`EnemyDbRefs[].Id` plus `Waves[].Fragments[].Actions[]` SPAWN keys/counts),
//! classifies the level via [`StageClassifier`] and inverts to
//! `enemy_id -> Vec<EnemyStageRef>`. Boss-only declarations from `activity_table`
//! are folded in too (some bosses never appear in a level file). Taxonomy and
//! labelling live in [`super::stage_class`].

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::stage_class::{StageClassifier, StageInfo};
use crate::core::gamedata::types::enemy::MaybeValue;
use crate::core::gamedata::types::enemy_stages::{
    EnemyStageIndex, EnemyStageRef, StageEnemyBattle,
};
use crate::core::startup;

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
    /// Index into the enemy's `enemy_database.json` levels: the stage fields
    /// this level, not level 0 (1,235 of 18,177 EN refs use 1 or 2).
    #[serde(default)]
    level: i32,
    #[serde(default)]
    overwritten_data: Option<Overwritten>,
}

/// The per-stage stat overrides a `EnemyDbRefs` entry may carry. Only the two
/// stats the damage engine reads are kept.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Overwritten {
    #[serde(default)]
    attributes: Option<OverwrittenAttributes>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct OverwrittenAttributes {
    #[serde(default)]
    def: Option<MaybeValue<f64>>,
    #[serde(default)]
    magic_resistance: Option<MaybeValue<f64>>,
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

/// Parse one level file into `enemy_id -> spawn count`, plus the database
/// level and DEF / RES overrides each `EnemyDbRefs` entry declares. `EnemyDbRefs`
/// seeds a count of 0 (declared but maybe summon-only); `SPAWN` actions add
/// their count. An id is declared at most once per file (0 repeats in EN).
type LevelTally = (HashMap<String, u32>, HashMap<String, StageEnemyBattle>);

fn tally_level(path: &Path) -> Option<LevelTally> {
    let bytes = std::fs::read(path).ok()?;
    let level: LevelFile = serde_json::from_slice(&bytes).ok()?;

    let mut counts: HashMap<String, u32> = HashMap::new();
    let mut battle: HashMap<String, StageEnemyBattle> = HashMap::new();
    for r in &level.enemy_db_refs {
        counts.entry(r.id.clone()).or_insert(0);
        let attrs = r
            .overwritten_data
            .as_ref()
            .and_then(|o| o.attributes.as_ref());
        let pick = |v: Option<&MaybeValue<f64>>| v.and_then(|m| m.get().copied());
        battle.insert(
            r.id.clone(),
            StageEnemyBattle {
                level: r.level,
                def: pick(attrs.and_then(|a| a.def.as_ref())),
                res: pick(attrs.and_then(|a| a.magic_resistance.as_ref())),
            },
        );
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
    Some((counts, battle))
}

/// Recursively collect every `*.json` level file, skipping the `enemydata/`
/// stats directory and `levelreplacers/` (IS layout variants that duplicate a
/// base node).
pub(crate) fn collect_level_files(dir: &Path, out: &mut Vec<PathBuf>) {
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
pub(crate) fn relative_level_id(levels_dir: &Path, path: &Path) -> Option<String> {
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
    battle: Option<StageEnemyBattle>,
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
        battle,
    });
}

pub fn build_enemy_stage_index(
    levels_dir: &Path,
    data_dir: &Path,
    classifier: &StageClassifier,
) -> EnemyStageIndex {
    startup::step("enemy index · scan");
    let mut files = Vec::new();
    collect_level_files(levels_dir, &mut files);

    let mut index: EnemyStageIndex = HashMap::new();

    // ~2800 files, a count the boot bar can use directly instead of guessing
    // from elapsed time.
    startup::step("enemy index · parse");
    let total = files.len() as u64;
    for (done, path) in files.iter().enumerate() {
        startup::step_progress(done as u64, total);
        let Some(rel) = relative_level_id(levels_dir, path) else {
            continue;
        };
        let Some((counts, mut battle)) = tally_level(path) else {
            continue;
        };
        let info = classifier.classify_level(&rel);
        let stage_id = rel.rsplit('/').next().unwrap_or(&rel).to_owned();
        for (enemy_id, count) in counts {
            // A SPAWN key with no `EnemyDbRefs` entry (0 in EN) is level 0.
            let b = battle.remove(&enemy_id).unwrap_or_default();
            push_ref(
                &mut index,
                enemy_id,
                stage_id.clone(),
                &info,
                count,
                Some(b),
            );
        }
    }

    // Boss declarations that never appear in a level file.
    startup::step("enemy index · bosses");
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
        push_ref(&mut index, enemy_id, stage_id, &info, 0, None);
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
