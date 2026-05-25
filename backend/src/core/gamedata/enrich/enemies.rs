use std::collections::HashMap;

use crate::core::gamedata::{
    assets::{AssetIndex, AssetKind},
    types::enemy::{
        EnemyAttributes, EnemyDatabaseFile, EnemyHandbook, EnemyHandbookTableFile, EnemyLevelStats,
        EnemySkill, EnemyStats, RawEnemyLevelEntry,
    },
};

/// Build a fully-resolved level entry by overlaying `entry`'s defined fields
/// on top of `base`. Hypergryph's `enemy_database.json` only fills in the
/// fields a higher difficulty tier (or boss phase) explicitly overrides;
/// everything else relies on `MaybeValue.defined == false` falling through to
/// the previous level. If we drop that flag we end up serving `ASPD=0`,
/// `weight=0`, empty skill lists, etc. for every non-zero-indexed level -
/// which is wrong. Merging here means the frontend receives a complete,
/// self-contained attribute set per level/phase.
fn merge_level_entry(
    base: Option<&EnemyLevelStats>,
    entry: &RawEnemyLevelEntry,
) -> EnemyLevelStats {
    let data = &entry.enemy_data;
    let attrs = &data.attributes;
    let base_attrs = base.map(|l| &l.attributes);

    let pick_i32 = |mv: &crate::core::gamedata::types::enemy::MaybeValue<i32>,
                    pick_base: fn(&EnemyAttributes) -> i32|
     -> i32 {
        if mv.defined {
            mv.value
        } else {
            base_attrs.map(pick_base).unwrap_or(0)
        }
    };
    let pick_f64 = |mv: &crate::core::gamedata::types::enemy::MaybeValue<f64>,
                    pick_base: fn(&EnemyAttributes) -> f64|
     -> f64 {
        if mv.defined {
            mv.value
        } else {
            base_attrs.map(pick_base).unwrap_or(0.0)
        }
    };
    let pick_bool = |mv: &crate::core::gamedata::types::enemy::MaybeValue<bool>,
                     pick_base: fn(&EnemyAttributes) -> bool|
     -> bool {
        if mv.defined {
            mv.value
        } else {
            base_attrs.map(pick_base).unwrap_or(false)
        }
    };

    let attributes = EnemyAttributes {
        max_hp: pick_i32(&attrs.max_hp, |a| a.max_hp),
        atk: pick_i32(&attrs.atk, |a| a.atk),
        def: pick_i32(&attrs.def, |a| a.def),
        magic_resistance: pick_f64(&attrs.magic_resistance, |a| a.magic_resistance),
        move_speed: pick_f64(&attrs.move_speed, |a| a.move_speed),
        attack_speed: pick_f64(&attrs.attack_speed, |a| a.attack_speed),
        base_attack_time: pick_f64(&attrs.base_attack_time, |a| a.base_attack_time),
        mass_level: pick_i32(&attrs.mass_level, |a| a.mass_level),
        hp_recovery_per_sec: pick_f64(&attrs.hp_recovery_per_sec, |a| a.hp_recovery_per_sec),
        stun_immune: pick_bool(&attrs.stun_immune, |a| a.stun_immune),
        silence_immune: pick_bool(&attrs.silence_immune, |a| a.silence_immune),
        sleep_immune: pick_bool(&attrs.sleep_immune, |a| a.sleep_immune),
        frozen_immune: pick_bool(&attrs.frozen_immune, |a| a.frozen_immune),
        levitate_immune: pick_bool(&attrs.levitate_immune, |a| a.levitate_immune),
    };

    let apply_way = data
        .apply_way
        .get()
        .cloned()
        .or_else(|| base.and_then(|l| l.apply_way.clone()));
    let motion = data
        .motion
        .get()
        .cloned()
        .or_else(|| base.and_then(|l| l.motion.clone()));
    let range_radius = data
        .range_radius
        .get()
        .copied()
        .or_else(|| base.and_then(|l| l.range_radius));
    let life_point_reduce = if data.life_point_reduce.defined {
        data.life_point_reduce.value
    } else {
        base.map(|l| l.life_point_reduce).unwrap_or(0)
    };

    // Skills are authored once at level 0 for nearly every enemy; higher
    // difficulty tiers ship an empty array. Treat an empty raw skills list as
    // "inherit", since that matches in-game behavior (the boss keeps fighting
    // with the same kit, just with scaled stats).
    let skills = if data.skills.is_empty() {
        base.map(|l| l.skills.clone()).unwrap_or_default()
    } else {
        data.skills
            .iter()
            .map(|s| EnemySkill {
                prefab_key: s.prefab_key.clone(),
                priority: s.priority,
                cooldown: s.cooldown,
                init_cooldown: s.init_cooldown,
                sp_cost: s.sp_cost,
                blackboard: s.blackboard.clone(),
            })
            .collect()
    };

    EnemyLevelStats {
        level: entry.level,
        attributes,
        apply_way,
        motion,
        range_radius,
        life_point_reduce,
        skills,
    }
}

fn convert_levels(entries: &[RawEnemyLevelEntry]) -> Vec<EnemyLevelStats> {
    let mut out: Vec<EnemyLevelStats> = Vec::with_capacity(entries.len());
    for entry in entries {
        let merged = merge_level_entry(out.last(), entry);
        out.push(merged);
    }
    out
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
                levels: convert_levels(&entry.value),
            };
            (entry.key.clone(), stats)
        })
        .collect();

    let tags_map: HashMap<String, Vec<String>> = enemy_database
        .enemies
        .iter()
        .filter_map(|entry| {
            entry
                .value
                .iter()
                .find_map(|lvl| lvl.enemy_data.enemy_tags.get().cloned())
                .map(|tags| (entry.key.clone(), tags))
        })
        .collect();

    let enemy_data = handbook
        .enemy_data
        .into_iter()
        .map(|(id, mut enemy)| {
            if let Some(stats) = stats_map.get(&enemy.enemy_id) {
                enemy.stats = Some(stats.clone());
            }
            if enemy.enemy_tags.is_none()
                && let Some(tags) = tags_map.get(&enemy.enemy_id)
            {
                enemy.enemy_tags = Some(tags.clone());
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
