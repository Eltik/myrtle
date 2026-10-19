//! The pruned read of `roguelike_topic_table.json` replaces a full
//! `serde_json::Value` parse (`ROGUELIKE_VALUE_PARSE=1` restores it). Pins that
//! on the real tables both paths give the IDENTICAL derived IS data: every
//! theme's maxima and archive id lists, every tier-list entry in order, and
//! the stage classifier's season names, node triples, node ids and KV banners.
//!
//! Runs on every server tree present under `../assets/output`; a server
//! without the table is skipped, and the test fails if none has it.

use std::path::Path;

use backend::core::gamedata::enrich::stage_class::roguelike_mode_names;
use backend::core::gamedata::tables::load_table;
use backend::core::gamedata::types::roguelike::{RoguelikeGameData, RoguelikeTopicTableFile};
use backend::core::gamedata::types::roguelike_prune::load_pruned;

#[test]
fn pruned_read_matches_the_value_parse() {
    let mut compared = 0;
    for server in ["en", "cn", "jp", "kr"] {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../assets/output/{server}/gamedata/excel"));
        if !dir.join("roguelike_topic_table.json").is_file() {
            eprintln!("{server}: no roguelike_topic_table; skipped");
            continue;
        }

        let whole: RoguelikeTopicTableFile =
            load_table(&dir, "roguelike_topic_table").expect("value parse");
        let old_data = RoguelikeGameData::from_table(&whole);
        let old_names = roguelike_mode_names(&whole);
        let whole_bytes = whole.topics.to_string().len() + whole.details.to_string().len();
        drop(whole);

        let pruned = load_pruned(&dir).expect("pruned read");
        let new_data = RoguelikeGameData::from_table(&pruned);
        let new_names = roguelike_mode_names(&pruned);
        let pruned_bytes = pruned.topics.to_string().len() + pruned.details.to_string().len();

        assert!(!old_data.themes.is_empty(), "{server}: no themes read");
        assert_eq!(old_data.themes, new_data.themes, "{server}: themes differ");
        assert_eq!(
            old_data.entries.entries, new_data.entries.entries,
            "{server}: entries differ"
        );
        assert!(
            !old_names.node_named.is_empty(),
            "{server}: no IS nodes read"
        );
        assert_eq!(old_names, new_names, "{server}: classifier names differ");
        eprintln!(
            "{server}: {} themes, {} entries, {} nodes, {} node ids, {} KV identical; \
             Topics+Details {whole_bytes} -> {pruned_bytes} bytes as JSON",
            new_data.themes.len(),
            new_data.entries.entries.len(),
            new_names.node_named.len(),
            new_names.is_node_ids.len(),
            new_names.is_kv.len(),
        );
        compared += 1;
    }
    assert!(
        compared > 0,
        "no server tree has roguelike_topic_table.json"
    );
}
