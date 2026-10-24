//! Phase 4 of the DPS refactor (docs/dps-refactor.md) on real EN game data:
//! the roster retention score, its enemy set, and the deduplication of
//! (DEF, RES) pairs.
//!
//! `RETENTION_DUMP=<path>` writes every row as
//! `operator\tskill\tmodule\tbaseDps\tretention\tretentionBySpawns` at full
//! `{:?}` precision.
#![allow(
    clippy::float_cmp,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]

use std::path::Path;
use std::sync::{Arc, OnceLock};

use backend::app::services::dps::calculate_in;
use backend::app::services::dps_retention::{
    RetentionTable, compute_retention, enemy_roster, reference_context,
};
use backend::core::gamedata::{self, types::GameData};

fn game_data() -> &'static Arc<GameData> {
    static GD: OnceLock<Arc<GameData>> = OnceLock::new();
    GD.get_or_init(|| {
        // No `dotenv`: under the local backend/.env the loader sees 7 enemies
        // and 10 appearances (boss declarations only, no level files), so the
        // roster would be empty. The paths below hold the EN level files.
        let data_dir = std::env::var("GAME_DATA_DIR")
            .unwrap_or_else(|_| "../assets/output/en/gamedata/excel".into());
        let assets_dir =
            std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output/en".into());
        Arc::new(
            gamedata::init_game_data(Path::new(&data_dir), Path::new(&assets_dir))
                .expect("Failed to load game data")
                .0,
        )
    })
}

fn table() -> &'static RetentionTable {
    static T: OnceLock<RetentionTable> = OnceLock::new();
    T.get_or_init(|| compute_retention(game_data()))
}

#[test]
fn roster_is_the_stage_appearances() {
    let gd = game_data();
    let (roster, pairs) = enemy_roster(gd);
    println!("{}", serde_json::to_string_pretty(&roster).unwrap());
    let total: u64 = pairs.iter().map(|p| p.stages).sum();
    assert_eq!(total as usize, roster.appearances);
    assert_eq!(pairs.len(), roster.distinct_pairs);
    assert_eq!(
        pairs.iter().map(|p| p.spawns).sum::<u64>(),
        roster.total_spawns
    );
    // Every appearance of every kept enemy is counted once.
    let kept: usize = gd
        .enemy_stage_index
        .iter()
        .filter(|(id, _)| {
            gd.enemies
                .enemy_data
                .get(*id)
                .and_then(|e| e.stats.as_ref())
                .is_some_and(|s| !s.levels.is_empty())
        })
        .map(|(_, refs)| refs.len())
        .sum();
    assert_eq!(kept, roster.appearances);
    // Pairs are distinct and sorted.
    for w in pairs.windows(2) {
        assert!((w[0].def, w[0].res) < (w[1].def, w[1].res));
    }
    // Level 1 / 2 and overrides are read from the level files.
    assert!(
        roster.appearances > roster.no_level_file,
        "no level files under ASSETS_DIR"
    );
    assert!(roster.nonzero_level > 0);
    assert!(roster.overridden > 0);
    let mut by_weight = pairs.clone();
    by_weight.sort_by_key(|p| std::cmp::Reverse(p.stages));
    for p in by_weight.iter().take(10) {
        println!(
            "DEF {} RES {}: {} appearances, {} spawns",
            p.def, p.res, p.stages, p.spawns
        );
    }
}

#[test]
fn stage_index_body_is_unchanged() {
    // The level / override data rides on the index server side only.
    let gd = game_data();
    let refs = gd.enemy_stage_index.values().next().expect("an enemy");
    let v = serde_json::to_value(&refs[0]).unwrap();
    assert!(v.get("battle").is_none(), "{v}");
    assert_eq!(v.as_object().unwrap().len(), 9);
}

#[test]
fn retention_scores() {
    let gd = game_data();
    let t = table();
    println!(
        "rows {} pairs {} computeMs {:?} (debug build)",
        t.rows.len(),
        t.roster.distinct_pairs,
        t.compute_ms
    );
    if let Ok(path) = std::env::var("RETENTION_DUMP") {
        let mut lines: Vec<String> = t
            .rows
            .iter()
            .map(|r| {
                format!(
                    "{}\t{}\t{}\t{:?}\t{:?}\t{:?}",
                    r.operator_id,
                    r.skill_index,
                    r.module_index,
                    r.base_dps,
                    r.retention,
                    r.retention_by_spawns
                )
            })
            .collect();
        lines.sort();
        std::fs::write(path, lines.join("\n") + "\n").unwrap();
    }

    let null = t.rows.iter().filter(|r| r.retention.is_none()).count();
    let zero_base = t.rows.iter().filter(|r| r.base_dps == 0.0).count();
    println!("null {null} (base dps 0: {zero_base})");
    assert_eq!(null, zero_base, "a non-zero row failed a pair");

    let mut scored: Vec<_> = t.rows.iter().filter(|r| r.retention.is_some()).collect();
    scored.sort_by(|a, b| {
        b.retention
            .unwrap()
            .total_cmp(&a.retention.unwrap())
            .then_with(|| a.operator_id.cmp(&b.operator_id))
            .then_with(|| a.skill_index.cmp(&b.skill_index))
            .then_with(|| a.module_index.cmp(&b.module_index))
    });
    let name = |id: &str| {
        gd.operators
            .get(id)
            .map_or(String::new(), |o| o.name.clone())
    };
    let show = |r: &&backend::app::services::dps_retention::RetentionRow| {
        format!(
            "{} ({}) S{} M{}: {:?} (by spawns {:?}, base {:?})",
            name(&r.operator_id),
            r.operator_id,
            r.skill_index,
            r.module_index,
            r.retention.unwrap(),
            r.retention_by_spawns,
            r.base_dps
        )
    };
    println!("TOP 10");
    for r in scored.iter().take(10) {
        println!("  {}", show(r));
    }
    println!("BOTTOM 10");
    for r in scored
        .iter()
        .rev()
        .take(10)
        .collect::<Vec<_>>()
        .iter()
        .rev()
    {
        println!("  {}", show(r));
    }
    let above_one = scored.iter().filter(|r| r.retention.unwrap() > 1.0).count();
    println!("retention > 1: {above_one}");

    // Alternative weight: does the top 20 change?
    let key = |r: &&backend::app::services::dps_retention::RetentionRow| {
        (r.operator_id.clone(), r.skill_index, r.module_index)
    };
    let top_stages: Vec<_> = scored.iter().take(20).map(key).collect();
    let mut by_spawns = scored.clone();
    by_spawns.sort_by(|a, b| {
        b.retention_by_spawns
            .unwrap_or(f64::NEG_INFINITY)
            .total_cmp(&a.retention_by_spawns.unwrap_or(f64::NEG_INFINITY))
            .then_with(|| a.operator_id.cmp(&b.operator_id))
            .then_with(|| a.skill_index.cmp(&b.skill_index))
            .then_with(|| a.module_index.cmp(&b.module_index))
    });
    let top_spawns: Vec<_> = by_spawns.iter().take(20).map(key).collect();
    let same_set = top_stages.iter().filter(|k| top_spawns.contains(k)).count();
    let same_pos = top_stages
        .iter()
        .zip(&top_spawns)
        .filter(|(a, b)| a == b)
        .count();
    println!("top 20 by spawns: {same_set} of 20 shared, {same_pos} at the same rank");
    for (i, (a, b)) in top_stages.iter().zip(&top_spawns).enumerate() {
        println!("  {:>2} {:?} | {:?}", i + 1, a, b);
    }
    let max_shift = scored
        .iter()
        .filter_map(|r| {
            r.retention_by_spawns
                .map(|s| (s - r.retention.unwrap()).abs())
        })
        .fold(0.0_f64, f64::max);
    println!("max |retention - retentionBySpawns| {max_shift:?}");

    // Sanity: an arts operator against a low-ATK physical multi-hitter.
    for (id, s, m) in [
        ("char_2013_cerber", 3, 0), // Ceobe S3, arts
        ("char_180_amgoat", 3, 0),  // Eyjafjalla S3, arts
        ("char_124_kroos", 1, 0),   // Kroos S1, low-ATK physical double shot
        ("char_010_chen", 3, 0),    // Ch'en S3, physical multi-hit
    ] {
        if let Some(r) = t
            .rows
            .iter()
            .find(|r| r.operator_id == id && r.skill_index == s && r.module_index == m)
        {
            println!("SANITY {}", show(&r));
        }
    }
}

#[test]
fn deduplicated_pairs_equal_the_appearance_sum() {
    // Evaluating every appearance on its own gives the deduplicated score to
    // rounding, on rows of both damage types.
    let gd = game_data();
    let t = table();
    let mut apps: Vec<(f64, f64)> = Vec::new();
    for (id, refs) in &gd.enemy_stage_index {
        let Some(levels) = gd
            .enemies
            .enemy_data
            .get(id)
            .and_then(|e| e.stats.as_ref())
            .filter(|s| !s.levels.is_empty())
            .map(|s| &s.levels)
        else {
            continue;
        };
        for r in refs {
            let b = r.battle.unwrap_or_default();
            let at = levels
                .iter()
                .find(|l| l.level == b.level)
                .unwrap_or(&levels[0]);
            apps.push((
                b.def.unwrap_or(f64::from(at.attributes.def)),
                b.res.unwrap_or(at.attributes.magic_resistance),
            ));
        }
    }
    for (id, s) in [("char_2013_cerber", 3), ("char_010_chen", 3)] {
        let row = t
            .rows
            .iter()
            .find(|r| r.operator_id == id && r.skill_index == s && r.module_index == 0)
            .expect("row");
        let mut sum = 0.0;
        let mut memo: std::collections::HashMap<(u64, u64), f64> = Default::default();
        for &(d, r) in &apps {
            let v = *memo.entry((d.to_bits(), r.to_bits())).or_insert_with(|| {
                calculate_in(gd, reference_context(d, r).row_request(id, s, 0))
                    .unwrap()
                    .skill_dps
            });
            sum += v;
        }
        let brute = sum / (apps.len() as f64 * row.base_dps);
        let rel = (brute - row.retention.unwrap()).abs() / brute;
        println!(
            "{id} S{s}: dedup {:?} per-appearance {brute:?} rel {rel:e}",
            row.retention.unwrap()
        );
        assert!(rel < 1e-12);
    }
}
