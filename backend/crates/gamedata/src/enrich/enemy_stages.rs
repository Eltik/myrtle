use std::collections::HashMap;

use myrtle_gamedata_raw::tables::{
    enemy::MaybeValue,
    level::{LevelFile, LevelTables},
};

use crate::{
    enrich::stage_class::{StageClassifier, StageInfo},
    model::enemy_stages::{EnemyStageIndex, EnemyStageRef, StageEnemyBattle},
};

type LevelTally = (HashMap<String, u32>, HashMap<String, StageEnemyBattle>);

fn tally_level(level: &LevelFile) -> LevelTally {
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
    (counts, battle)
}

pub(crate) fn relative_level_id(rel_path: &str) -> String {
    let rel = rel_path.replace('\\', "/");
    let (dir, file) = rel
        .rsplit_once('/')
        .map_or(("", rel.as_str()), |(d, f)| (d, f));
    let stem = match file.rsplit_once('.') {
        Some((stem, _)) if !stem.is_empty() => stem,
        _ => file,
    };
    let id = if dir.is_empty() {
        stem.to_owned()
    } else {
        format!("{dir}/{stem}")
    };
    id.to_lowercase()
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

#[must_use]
pub fn build_enemy_stage_index(
    levels: &LevelTables,
    bosses: &[(String, String)],
    classifier: &StageClassifier<'_>,
) -> EnemyStageIndex {
    let mut index: EnemyStageIndex = HashMap::new();

    for (path, level) in &levels.levels {
        let Some(level) = level else {
            continue;
        };
        let rel = relative_level_id(path);
        let (counts, mut battle) = tally_level(level);
        let info = classifier.classify_level(&rel);
        let stage_id = rel.rsplit('/').next().unwrap_or(&rel).to_owned();
        for (enemy_id, count) in counts {
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

    for (enemy_id, info) in classifier.boss_appearances(bosses) {
        let already = index.get(&enemy_id).is_some_and(|refs| {
            refs.iter()
                .any(|r| r.zone_id == info.zone_id && r.code == info.code)
        });
        if already {
            continue;
        }
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
