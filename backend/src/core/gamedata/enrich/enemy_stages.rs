//! Build the enemy -> stages inverted index from per-stage level files.
//!
//! Each stage in `stage_table` carries a `LevelId` (e.g.
//! `Obt/Main/level_main_00-01`) that points to a level file on disk under
//! `gamedata/levels/` (lowercased, `.json`). Those files list the enemies a
//! stage uses in two places:
//!   - `EnemyDbRefs[].Id` - every db enemy declared for the stage.
//!   - `Waves[].Fragments[].Actions[]` - the actual spawn timeline; `SPAWN`
//!     actions name an enemy via `Key` and how many via `Count`.
//!
//! We seed each stage's tally from `EnemyDbRefs` (count 0) so conditionally
//! summoned enemies still register as "appears here", then add `SPAWN` counts.
//! The result is inverted into `enemy_id -> Vec<EnemyStageRef>`.
//!
//! Not every stage has an extracted level file (the dump is a subset); missing
//! files are skipped silently, so the index naturally covers whatever is
//! present and grows as more levels are extracted.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::Path;

use serde::Deserialize;

use crate::core::gamedata::types::enemy_stages::{EnemyStageIndex, EnemyStageRef};
use crate::core::gamedata::types::stage::{Stage, StageDifficulty};

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

/// Resolve a stage's `LevelId` to its on-disk level file path. Paths are
/// stored lowercase under `levels_dir` with a `.json` extension.
fn level_path(levels_dir: &Path, level_id: &str) -> std::path::PathBuf {
    levels_dir.join(format!("{}.json", level_id.to_lowercase()))
}

/// Parse one level file into `enemy_id -> spawn count`. Returns `None` if the
/// file is absent or unparseable (both expected for the partial dump).
fn tally_stage(path: &Path) -> Option<HashMap<String, u32>> {
    let bytes = std::fs::read(path).ok()?;
    let level: LevelFile = serde_json::from_slice(&bytes).ok()?;

    let mut counts: HashMap<String, u32> = HashMap::new();
    // Seed every declared db enemy at 0 so summon-only enemies still count.
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
                // Restrict to handbook-shaped ids; inline non-db actors use
                // other key namespaces we don't want in this index.
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

/// True for hard-mode / Adverse variants: `level_tough_*` / `.../hard/...`
/// files and any non-Normal stage difficulty.
fn is_hard(stage: &Stage) -> bool {
    let path_hard = stage.level_id.as_deref().is_some_and(|l| {
        let l = l.to_lowercase();
        l.contains("tough") || l.contains("/hard/") || l.contains("hard_")
    });
    path_hard || !matches!(stage.difficulty, StageDifficulty::Normal)
}

/// Rank a stage as a representative for its level file: lower is better.
/// Prefer non-Challenge-Mode ids (CM stages carry a `#` and reuse their base
/// stage's level), then Normal difficulty, so a normal stage always wins over
/// its `#f#` CM twin that points at the same file.
fn canonical_rank(stage: &Stage) -> (u8, u8) {
    let cm = u8::from(stage.stage_id.contains('#'));
    let non_normal = u8::from(!matches!(stage.difficulty, StageDifficulty::Normal));
    (cm, non_normal)
}

/// Build the `enemy_id -> stages` index by walking every stage's level file.
pub fn build_enemy_stage_index(
    levels_dir: &Path,
    stages: &HashMap<String, Stage>,
) -> EnemyStageIndex {
    // Collapse stages that reuse the same level file to one representative.
    // Challenge Mode (`main_xx-yy#f#`) and similar variants share their base
    // stage's level, so without this every normal stage would appear twice.
    // Adverse (`level_tough_*`) stages have their own file and are kept.
    let mut by_level: HashMap<String, &Stage> = HashMap::new();
    for stage in stages.values() {
        let Some(level_id) = stage.level_id.as_deref() else {
            continue;
        };
        match by_level.entry(level_id.to_lowercase()) {
            Entry::Vacant(e) => {
                e.insert(stage);
            }
            Entry::Occupied(mut e) => {
                let better = canonical_rank(stage) < canonical_rank(e.get())
                    || (canonical_rank(stage) == canonical_rank(e.get())
                        && stage.stage_id < e.get().stage_id);
                if better {
                    e.insert(stage);
                }
            }
        }
    }

    let mut index: EnemyStageIndex = HashMap::new();

    for stage in by_level.into_values() {
        let Some(level_id) = stage.level_id.as_deref() else {
            continue;
        };
        let Some(counts) = tally_stage(&level_path(levels_dir, level_id)) else {
            continue;
        };

        let hard = is_hard(stage);
        for (enemy_id, count) in counts {
            index.entry(enemy_id).or_default().push(EnemyStageRef {
                stage_id: stage.stage_id.clone(),
                code: stage.code.clone(),
                zone_id: stage.zone_id.clone(),
                stage_name: stage.name.clone(),
                is_hard: hard,
                count,
            });
        }
    }

    // Stable ordering: by code, then stage id, with hard variants after normal.
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
