use std::collections::HashMap;

use crate::core::gamedata::{
    assets::{AssetIndex, AssetKind},
    types::enemy::{
        EnemyAttributes, EnemyDatabaseFile, EnemyHandbook, EnemyHandbookTableFile, EnemyLevelStats,
        EnemySkill, EnemyStats, RawEnemyLevelEntry,
    },
};

fn convert_level_entry(entry: &RawEnemyLevelEntry) -> EnemyLevelStats {
    let data = &entry.enemy_data;
    let attrs = &data.attributes;

    EnemyLevelStats {
        level: entry.level,
        attributes: EnemyAttributes {
            max_hp: attrs.max_hp.value,
            atk: attrs.atk.value,
            def: attrs.def.value,
            magic_resistance: attrs.magic_resistance.value,
            move_speed: attrs.move_speed.value,
            attack_speed: attrs.attack_speed.value,
            base_attack_time: attrs.base_attack_time.value,
            mass_level: attrs.mass_level.value,
            hp_recovery_per_sec: attrs.hp_recovery_per_sec.value,
            stun_immune: attrs.stun_immune.value,
            silence_immune: attrs.silence_immune.value,
            sleep_immune: attrs.sleep_immune.value,
            frozen_immune: attrs.frozen_immune.value,
            levitate_immune: attrs.levitate_immune.value,
        },
        apply_way: data.apply_way.get().cloned(),
        motion: data.motion.get().cloned(),
        range_radius: data.range_radius.get().copied(),
        life_point_reduce: data.life_point_reduce.value,
        skills: data
            .skills
            .iter()
            .map(|s| EnemySkill {
                prefab_key: s.prefab_key.clone(),
                priority: s.priority,
                cooldown: s.cooldown,
                init_cooldown: s.init_cooldown,
                sp_cost: s.sp_cost,
                blackboard: s.blackboard.clone(),
            })
            .collect(),
    }
}

pub fn enrich_enemies(
    handbook: EnemyHandbookTableFile,
    enemy_database: &EnemyDatabaseFile,
    assets: &AssetIndex,
) -> EnemyHandbook {
    let stats_map: HashMap<String, EnemyStats> = enemy_database
        .enemies
        .iter()
        .map(|entry| {
            let stats = EnemyStats {
                levels: entry.value.iter().map(convert_level_entry).collect(),
            };
            (entry.key.clone(), stats)
        })
        .collect();

    let enemy_data = handbook
        .enemy_data
        .into_iter()
        .map(|(id, mut enemy)| {
            if let Some(stats) = stats_map.get(&enemy.enemy_id) {
                enemy.stats = Some(stats.clone());
            }
            enemy.portrait = assets
                .path(AssetKind::EnemyIcon, &enemy.enemy_id)
                .map(str::to_owned);
            (id, enemy)
        })
        .collect();

    EnemyHandbook {
        level_info_list: handbook.level_info_list,
        enemy_data,
        race_data: handbook.race_data,
    }
}
