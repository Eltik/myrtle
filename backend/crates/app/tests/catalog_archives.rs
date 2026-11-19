#![allow(clippy::unwrap_used)]

mod common;

use std::sync::Arc;

use myrtle_app::AppError;
use myrtle_app::services::catalog::archives::{self, ArchiveList};
use myrtle_app::services::catalog::static_data::key;
use myrtle_app::snapshots::SnapshotRegistry;
use myrtle_types::Server;
use serde_json::Value;

#[tokio::test]
async fn the_archives_build_once_and_serve_every_part() {
    let Some((mut ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let mut config = common::config();
    config.assets.dir = std::env::var("MYRTLE_TEST_ASSETS").unwrap_or_else(|_| {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../../assets/output").to_owned()
    });
    let registry = SnapshotRegistry::new(&config);
    registry.install_shared(Server::EN, Arc::clone(&snapshot));
    ctx.snapshots = Arc::new(registry);

    let network = archives::list(&ctx, Server::EN, ArchiveList::Network)
        .await
        .unwrap();
    let parsed: Value = serde_json::from_str(&network.body).unwrap();
    assert_eq!(
        parsed["imported"], true,
        "network tables read from the extract"
    );
    assert!(!parsed["groups"].as_array().unwrap().is_empty());
    for list in [
        ArchiveList::Dossiers,
        ArchiveList::Demographics,
        ArchiveList::HiddenFactions,
    ] {
        archives::list(&ctx, Server::EN, list).await.unwrap();
    }
    let again = archives::list(&ctx, Server::EN, ArchiveList::Network)
        .await
        .unwrap();
    assert_eq!(network.etag, again.etag);

    let amiya: Value = serde_json::from_str(
        &archives::operator(&ctx, Server::EN, "char_002_amiya")
            .await
            .unwrap()
            .body,
    )
    .unwrap();
    assert!(amiya.is_object());
    assert!(matches!(
        archives::operator(&ctx, Server::EN, "char_none").await,
        Err(AppError::NotFound(None))
    ));
    assert!(matches!(
        archives::list(&ctx, Server::JP, ArchiveList::Network).await,
        Err(AppError::NotFound(_))
    ));

    for evicted in ["archives_network", "archive_operator:char_002_amiya"] {
        ctx.cache
            .invalidate(&key(&snapshot, Server::EN, evicted, "0"))
            .await;
    }
    assert!(
        ctx.cache
            .get(&key(&snapshot, Server::EN, "archives_built", "0"))
            .await
            .is_some()
    );
    let rebuilt = archives::list(&ctx, Server::EN, ArchiveList::Network)
        .await
        .unwrap();
    assert_eq!(rebuilt.etag, network.etag, "an evicted entry rebuilds");
    ctx.cache
        .invalidate(&key(
            &snapshot,
            Server::EN,
            "archive_operator:char_002_amiya",
            "0",
        ))
        .await;
    archives::operator(&ctx, Server::EN, "char_002_amiya")
        .await
        .unwrap();
    db.close().await;
}
