//! The returning-player pool flag, on the real trees.
//!
//! Measured 2026-10-10: every server has exactly one pool a `PlayerReturn`
//! tier grants, and it is the server's only BACKFLOW pool. Skips a server
//! whose `assets/output/<server>` is absent, which on CI is every server but EN.

mod common;

use std::path::Path;

use backend::core::gamedata::{
    enrich::gacha::mark_returning_pools,
    tables::load_table,
    types::{gacha::GachaTableFile, open_server::OpenServerTableFile},
};

const EXPECTED: [(&str, &str); 4] = [
    ("en", "RETURN_EN_41_0_1"),
    ("cn", "RETURN_71_0_1"),
    ("jp", "RETURN_JP_41_0_1"),
    ("kr", "RETURN_KR_41_0_1"),
];

#[test]
fn one_returning_pool_per_server_and_it_is_the_backflow_one() {
    for (server, expected) in EXPECTED {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../assets/output/{server}/gamedata/excel"));
        if !dir.join("gacha_table.json").exists() {
            eprintln!("skipping {server}: not on disk");
            continue;
        }
        let gacha: GachaTableFile = load_table(&dir, "gacha_table").expect("gacha_table");
        let open_server: OpenServerTableFile =
            load_table(&dir, "open_server_table").expect("open_server_table");
        let mut pools = gacha.gacha_pool_client;
        mark_returning_pools(&mut pools, &open_server.returning_pool_ids());

        let returning: Vec<&str> = pools
            .iter()
            .filter(|p| p.returning)
            .map(|p| p.gacha_pool_id.as_str())
            .collect();
        let backflow: Vec<&str> = pools
            .iter()
            .filter(|p| p.gacha_rule_type == "BACKFLOW")
            .map(|p| p.gacha_pool_id.as_str())
            .collect();
        eprintln!(
            "{server}: {} pools, returning {returning:?}, backflow {backflow:?}",
            pools.len()
        );
        assert_eq!(returning, [expected], "{server}");
        assert_eq!(
            backflow, returning,
            "{server}: BACKFLOW and returning differ"
        );
    }
}

/// The flag survives the full boot load, which is what the API serves.
#[test]
fn the_boot_load_carries_the_flag() {
    let data = common::load_game_data();
    let returning: Vec<&str> = data
        .gacha
        .gacha_pool_client
        .iter()
        .filter(|p| p.returning)
        .map(|p| p.gacha_pool_id.as_str())
        .collect();
    assert_eq!(returning, ["RETURN_EN_41_0_1"]);
}
