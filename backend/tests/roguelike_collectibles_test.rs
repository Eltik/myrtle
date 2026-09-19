//! Roguelike collectible counting: archive id lists vs the client's buckets.
//!
//! Background (measured 2026-09-17 on 2,630 local accounts): the in-game
//! collectible archive (`ArchiveComp.Relic.Relic`) lists the tool items, which
//! the client stores in `collect.activeTool` / `challenge.collect.exploreTool`,
//! never `collect.relic`; and `collect.relic` carries `_a/_b/_c` upgrade
//! variants the archive does not list. Counting bucket keys gave a 256/262
//! ceiling on `rogue_2` and 333/294 on `rogue_5`, and `BandRef` is not the
//! band list (0 entries on `rogue_1`/`rogue_3`, 12 of 22 on `rogue_2`).

use std::path::Path;

use backend::core::gamedata::tables::load_table;
use backend::core::gamedata::types::roguelike::{
    CollectedCounts, RoguelikeGameData, RoguelikeThemeGameData, RoguelikeTopicTableFile,
};
use serde_json::json;

fn en_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/output/en/gamedata/excel")
}

fn load_en() -> Option<RoguelikeGameData> {
    let dir = en_dir();
    if !dir.join("roguelike_topic_table.json").exists() {
        return None;
    }
    let file: RoguelikeTopicTableFile = load_table(&dir, "roguelike_topic_table").ok()?;
    Some(RoguelikeGameData::from_table(&file))
}

#[test]
fn archive_id_lists_match_the_en_table() {
    let Some(data) = load_en() else {
        eprintln!("skipping: EN roguelike_topic_table.json not present");
        return;
    };
    let expect = [
        // theme, relics, bands, capsules, an id the archive lists outside `collect.relic`
        ("rogue_1", 245, 10, 26, None),
        ("rogue_2", 262, 22, 0, Some("rogue_2_active_tool_1")),
        ("rogue_3", 263, 13, 0, Some("rogue_3_explore_tool_1")),
        ("rogue_4", 296, 55, 0, Some("rogue_4_active_tool_1")),
        ("rogue_5", 294, 46, 0, Some("rogue_5_active_tool_1")),
    ];
    for (theme_id, relics, bands, capsules, tool_id) in expect {
        let t = &data.themes[theme_id];
        assert_eq!(t.relic_ids.len(), relics, "{theme_id} relic_ids");
        assert_eq!(t.max_relics as usize, relics, "{theme_id} max_relics");
        assert_eq!(t.band_ids.len(), bands, "{theme_id} band_ids");
        assert_eq!(t.max_bands as usize, bands, "{theme_id} max_bands");
        assert_eq!(t.capsule_ids.len(), capsules, "{theme_id} capsule_ids");
        assert_eq!(t.max_capsules as usize, capsules, "{theme_id} max_capsules");
        if let Some(id) = tool_id {
            assert!(t.relic_ids.iter().any(|r| r == id), "{theme_id} lists {id}");
        }
        assert!(
            t.band_ids
                .iter()
                .all(|b| b.starts_with(&format!("{theme_id}_band_"))),
            "{theme_id} band ids are band items"
        );
    }
}

#[test]
fn counts_look_ids_up_across_buckets_and_ignore_non_archive_keys() {
    let theme = RoguelikeThemeGameData {
        relic_ids: vec![
            "rogue_x_relic_legacy_1".into(),
            "rogue_x_relic_legacy_2".into(),
            "rogue_x_active_tool_1".into(),
            "rogue_x_explore_tool_1".into(),
        ],
        capsule_ids: vec!["rogue_x_capsule_1".into()],
        band_ids: vec!["rogue_x_band_1".into(), "rogue_x_band_2".into()],
        ..Default::default()
    };
    let progress = json!({
        "collect": {
            "relic": {
                "rogue_x_relic_legacy_1": {"state": 2, "progress": null},
                // upgrade variants: in the bucket, not in the archive, never counted
                "rogue_x_relic_legacy_1_a": {"state": 1, "progress": null},
                "rogue_x_relic_legacy_1_b": {"state": 1, "progress": null},
                "rogue_x_relic_legacy_2": {"state": 0, "progress": null},
                // stray bucket key for a band id: bands are not read from here
                "rogue_x_band_2": {"state": 1, "progress": null}
            },
            "activeTool": {"rogue_x_active_tool_1": {"state": 1, "progress": null}},
            "capsule": {"rogue_x_capsule_1": {"state": 1, "progress": null}},
            "band": {
                "rogue_x_band_1": {"state": 1, "progress": null},
                "rogue_x_band_99": {"state": 1, "progress": null}
            }
        },
        "challenge": {
            "collect": {"exploreTool": {"rogue_x_explore_tool_1": {"state": 1, "progress": null}}}
        }
    });
    assert_eq!(
        theme.count_collected(&progress),
        CollectedCounts {
            relics: 3,
            capsules: 1,
            bands: 1
        }
    );
    assert_eq!(
        theme.count_collected(&json!({})),
        CollectedCounts::default()
    );
    assert_eq!(
        theme.count_collected(&json!(null)),
        CollectedCounts::default()
    );
}

/// Before/after on the local roster. Needs Postgres with real
/// `user_roguelike_progress` rows, so it is ignored by default:
/// `cargo test --test roguelike_collectibles_test -- --ignored --nocapture`.
///
/// Prints, per theme, the old bucket-key count (capped, as shipped before) and
/// the archive-id count: ceiling, and how many accounts read as complete.
#[tokio::test]
#[ignore = "reads the local Postgres roster"]
async fn real_data_before_after() {
    let Some(data) = load_en() else {
        eprintln!("skipping: EN roguelike_topic_table.json not present");
        return;
    };
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("connect to local postgres");
    let rows: Vec<(String, serde_json::Value)> =
        sqlx::query_as("SELECT theme_id, progress FROM user_roguelike_progress")
            .fetch_all(&pool)
            .await
            .expect("select progress");

    let old_count = |p: &serde_json::Value, bucket: &str| -> usize {
        p.get("collect")
            .and_then(|c| c.get(bucket))
            .and_then(|b| b.as_object())
            .map_or(0, |o| {
                o.values()
                    .filter(|v| v.get("state").and_then(serde_json::Value::as_i64).unwrap_or(0) >= 1)
                    .count()
            })
    };

    let mut themes: Vec<&String> = data.themes.keys().collect();
    themes.sort();
    for theme_id in themes {
        let t = &data.themes[theme_id];
        let (mut old_top, mut old_full, mut new_top, mut new_full, mut n) = (0, 0, 0, 0, 0);
        let (mut old_band_top, mut new_band_top) = (0, 0);
        for (tid, p) in &rows {
            if tid != theme_id {
                continue;
            }
            n += 1;
            let old = old_count(p, "relic").min(t.max_relics as usize);
            let new = t.count_collected(p);
            old_top = old_top.max(old);
            new_top = new_top.max(new.relics);
            old_full += usize::from(old == t.max_relics as usize);
            new_full += usize::from(new.relics == t.max_relics as usize);
            old_band_top = old_band_top.max(old_count(p, "band").min(t.max_bands as usize));
            new_band_top = new_band_top.max(new.bands);
        }
        println!(
            "{theme_id}: users {n}; relics max {} old top {old_top} full {old_full} -> new top {new_top} full {new_full}; bands max {} old top {old_band_top} -> new top {new_band_top}",
            t.max_relics, t.max_bands
        );
        assert!(new_top <= t.max_relics as usize);
        assert!(new_band_top <= t.max_bands as usize);
    }
}
