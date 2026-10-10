//! The two banner facts the release planner reads off the data rather than the
//! rule type, on the real trees: the published 6★ rate-up share per operator
//! (pool-detail sidecar) and the repeating collab handover (`LinkageParam`).
//!
//! Measured 2026-10-10: on CN exactly `LINKAGE_74_0_1`, `LINKAGE_74_0_3` and
//! `LINKAGE_77_0_1` ship `loopTarget6Count: 120`; no EN pool does. Skips a server
//! whose tree or sidecar is absent, which on CI is every server but EN, and EN
//! has no sidecar there.

mod common;

use std::path::Path;

use backend::core::gamedata::{
    enrich::gacha::enrich_banners,
    tables::load_table,
    types::{
        gacha::{GachaPoolClient, GachaTableFile},
        gacha_detail::PoolDetailFile,
    },
};

const LOOPING_CN: [&str; 3] = ["LINKAGE_74_0_1", "LINKAGE_74_0_3", "LINKAGE_77_0_1"];

/// The server's pools enriched with its sidecar, and the sidecar as raw JSON.
fn load(server: &str) -> Option<(Vec<GachaPoolClient>, serde_json::Value)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../assets/output/{server}"));
    let dir = root.join("gamedata/excel");
    let sidecar = root.join("derived/gacha_pool_details.json");
    if !dir.join("gacha_table.json").exists() || !sidecar.exists() {
        eprintln!("skipping {server}: not on disk");
        return None;
    }
    let bytes = std::fs::read(&sidecar).expect("read sidecar");
    let file: PoolDetailFile = serde_json::from_slice(&bytes).expect("sidecar parses");
    let raw: serde_json::Value = serde_json::from_slice(&bytes).expect("sidecar is JSON");
    let gacha: GachaTableFile = load_table(&dir, "gacha_table").expect("gacha_table");
    let mut pools = gacha.gacha_pool_client;
    enrich_banners(&mut pools, Some(&file));
    Some((pools, raw))
}

#[test]
fn only_the_three_new_cn_collabs_loop() {
    for server in ["cn", "en"] {
        let Some((pools, _)) = load(server) else {
            continue;
        };
        let looping: Vec<&str> = pools
            .iter()
            .filter(|p| p.linkage_loop_at.is_some())
            .map(|p| p.gacha_pool_id.as_str())
            .collect();
        eprintln!("{server}: looping {looping:?}");
        if server == "cn" {
            assert_eq!(looping, LOOPING_CN);
            for p in pools.iter().filter(|p| p.linkage_loop_at.is_some()) {
                assert_eq!(p.linkage_loop_at, Some(120), "{}", p.gacha_pool_id);
            }
            // An older collab: the once-only `guaranteeTarget6Count` rule.
            let old = pools.iter().find(|p| p.gacha_pool_id == "LINKAGE_17_0_1");
            assert_eq!(old.expect("LINKAGE_17_0_1").linkage_loop_at, None);
        } else {
            assert!(looping.is_empty(), "{server}: {looping:?}");
        }
    }
}

#[test]
fn declared_share_is_the_sidecar_percent() {
    for server in ["cn", "en"] {
        let Some((pools, raw)) = load(server) else {
            continue;
        };
        let mut declared = 0;
        for p in &pools {
            let expected =
                raw["pools"][&p.gacha_pool_id]["detailInfo"]["upCharInfo"]["perCharList"]
                    .as_array()
                    .and_then(|l| l.iter().find(|e| e["rarityRank"] == 5))
                    .and_then(|e| e["percent"].as_f64());
            assert_eq!(p.declared_share6, expected, "{server} {}", p.gacha_pool_id);
            declared += usize::from(expected.is_some());
        }
        eprintln!(
            "{server}: {declared} of {} pools declare a share",
            pools.len()
        );
        assert!(declared > 0, "{server}: no declared shares");
    }

    if let Some((pools, _)) = load("cn") {
        let share = |id: &str| {
            pools
                .iter()
                .find(|p| p.gacha_pool_id == id)
                .and_then(|p| p.declared_share6)
        };
        // Limited splits 0.35 per operator, a collab gives its one 6★ half.
        assert_eq!(share("LIMITED_76_0_1"), Some(0.35));
        assert_eq!(share("LINKAGE_74_0_1"), Some(0.5));
    }
}
