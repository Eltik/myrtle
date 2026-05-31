//! Regression test for the Reclamation Algorithm node-count inflation: the old
//! code summed nodes across all ~59 randomized map variants (~1032), which both
//! showed a bogus ">1,000" total and made the exploration sub-score unfillable.

use backend::core::gamedata::types::sandbox_universe::SandboxUniverse;
use std::path::Path;

#[test]
fn sandbox_node_count_reflects_one_playthrough() {
    let data_dir_str =
        std::env::var("GAME_DATA_DIR").unwrap_or_else(|_| "../assets/output/gamedata/excel".into());
    let path = Path::new(&data_dir_str).join("sandbox_perm_table.json");
    let raw: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("read sandbox_perm_table"))
            .expect("parse sandbox_perm_table");

    let universe = SandboxUniverse::build(&raw);

    // The persistent overworld (sandbox_1_main) has ~127 nodes — "over 100",
    // matching what a player actually explores. The old bug produced 1032.
    assert!(
        universe.max_nodes > 100 && universe.max_nodes < 400,
        "max_nodes should reflect a single playthrough (~127), got {}",
        universe.max_nodes
    );
    assert!(
        universe.max_nodes < 1000,
        "max_nodes regressed to the old all-variants sum: {}",
        universe.max_nodes
    );
}
