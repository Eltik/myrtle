#![allow(clippy::unwrap_used)]

mod common;

use std::path::PathBuf;

use myrtle_app::snapshots::{Snapshot, SnapshotRegistry};
use myrtle_gamedata_raw::tables::obtain::ObtainChannel;
use myrtle_types::Server;

fn unclassified(snapshot: &Snapshot) -> usize {
    let other = Some(ObtainChannel::Other);
    let ops = snapshot.model.operators.values();
    ops.filter(|op| op.obtain_channel == other).count()
}

fn registry(root: &str) -> SnapshotRegistry {
    let mut config = common::config();
    config.assets.dir = root.to_owned();
    config.server.servers = vec![Server::EN, Server::KR];
    config.switches.profile_align.enabled = true;
    config.switches.obtain_channel_align.enabled = true;
    config.switches.art_fallback.enabled = true;
    SnapshotRegistry::new(&config)
}

#[tokio::test]
async fn the_registry_aligns_a_text_only_server_on_the_servers_loaded_before_it() {
    let root = std::env::var_os("MYRTLE_TEST_ASSETS").map_or_else(
        || {
            PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../assets/output"
            ))
        },
        PathBuf::from,
    );
    let extracted = |server: &str| {
        root.join(server)
            .join("gamedata/excel/character_table.json")
            .is_file()
    };
    if !extracted("en") || !extracted("kr") {
        return;
    }
    let root = root.to_str().unwrap();

    let booted = registry(root);
    for server in booted.servers() {
        booted.activate_at(server, 0, None).await.unwrap();
    }
    let aligned = booted.require(Server::KR).unwrap();

    let without_en = registry(root);
    without_en.activate_at(Server::KR, 0, None).await.unwrap();
    let alone = without_en.require(Server::KR).unwrap();

    assert!(unclassified(&alone) > 0, "KR wording classified everything");
    assert!(unclassified(&aligned) < unclassified(&alone));
    assert_ne!(
        aligned.key("k"),
        alone.key("k"),
        "one tree and generation, other references"
    );
}
