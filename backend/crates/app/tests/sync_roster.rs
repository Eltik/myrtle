#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::privacy::{UpdateSettingsRequest, update_settings};
use myrtle_app::services::sync::roster::{
    currency_header_field, inventory, operator, operators, owned_skins, supports,
};
use myrtle_app::services::sync::store;
use myrtle_app::snapshots::Snapshot;
use myrtle_app::{AppContext, AppError, Subject};
use myrtle_db::repos::{gacha, users};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::sandbox_universe::SandboxUniverse;
use myrtle_types::{Role, Server, UserId};
use serde_json::{Value, json};

const UID: &str = "10000001";

async fn owner_and_visitor() -> (AppContext, myrtle_testkit::db::TestDb, Subject, Subject) {
    let (ctx, db) = common::context().await;
    ctx.snapshots.install(
        Server::EN,
        Snapshot::from_model(GameModel::default(), SandboxUniverse::default()),
    );
    store(
        &ctx,
        UID,
        Server::EN,
        include_str!("fixtures/sync/sync_data.json").to_owned(),
    )
    .await
    .unwrap();
    let mut conn = ctx.db.conn().await.unwrap();
    let owner = UserId(
        users::find(&mut conn, UID, Server::EN)
            .await
            .unwrap()
            .unwrap()
            .id,
    );
    gacha::update_flags(&mut conn, owner, None, None)
        .await
        .unwrap();
    drop(conn);
    let visitor = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    let owner = common::user(&ctx, owner, Role::User).await;
    (ctx, db, owner, visitor)
}

async fn settings(ctx: &AppContext, owner: &Subject, body: Value) {
    let request: UpdateSettingsRequest = serde_json::from_value(body).unwrap();
    update_settings(ctx, owner, &request).await.unwrap();
}

#[tokio::test]
async fn a_public_roster_reads_for_anyone_and_a_private_one_for_its_owner() {
    let (ctx, db, owner, visitor) = owner_and_visitor().await;
    for subject in [&owner, &visitor, &Subject::Anonymous] {
        assert_eq!(operators(&ctx, subject, Some(UID)).await.unwrap().len(), 6);
    }
    assert_eq!(operators(&ctx, &owner, None).await.unwrap().len(), 6);
    assert!(matches!(
        operators(&ctx, &Subject::Anonymous, None).await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        operators(&ctx, &visitor, Some("99999999")).await,
        Err(AppError::NotFound(None))
    ));

    settings(&ctx, &owner, json!({ "public_profile": false })).await;
    for subject in [&visitor, &Subject::Anonymous] {
        assert!(matches!(
            operators(&ctx, subject, Some(UID)).await,
            Err(AppError::Forbidden)
        ));
        assert!(matches!(
            supports(&ctx, subject, Some(UID)).await,
            Err(AppError::Forbidden)
        ));
        assert!(matches!(
            inventory(&ctx, subject, Some(UID)).await,
            Err(AppError::Forbidden)
        ));
    }
    assert_eq!(operators(&ctx, &owner, Some(UID)).await.unwrap().len(), 6);
    db.close().await;
}

#[tokio::test]
async fn one_operator_or_not_found() {
    let (ctx, db, owner, visitor) = owner_and_visitor().await;
    let lemuen = operator(&ctx, &visitor, Some(UID), "char_4193_lemuen")
        .await
        .unwrap();
    assert_eq!((lemuen.elite, lemuen.level), (2, 90));
    assert!(matches!(
        operator(&ctx, &owner, None, "char_9999_nobody").await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[tokio::test]
async fn a_hidden_roster_tab_keeps_supports_readable() {
    let (ctx, db, owner, visitor) = owner_and_visitor().await;
    assert_eq!(
        owned_skins(&ctx, &visitor, Some(UID)).await.unwrap().len(),
        3
    );
    settings(
        &ctx,
        &owner,
        json!({ "profile_layout": { "tabs": [{ "id": "roster", "visible": false }] } }),
    )
    .await;
    assert!(matches!(
        operators(&ctx, &visitor, Some(UID)).await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        operator(&ctx, &visitor, Some(UID), "char_4193_lemuen").await,
        Err(AppError::Forbidden)
    ));
    assert!(
        matches!(
            owned_skins(&ctx, &visitor, Some(UID)).await,
            Err(AppError::Forbidden)
        ),
        "skins are roster data"
    );
    let seats = supports(&ctx, &visitor, Some(UID)).await.unwrap();
    assert_eq!(seats.len(), 2, "support units are public in game");
    assert_eq!(owned_skins(&ctx, &owner, Some(UID)).await.unwrap().len(), 3);

    settings(
        &ctx,
        &owner,
        json!({ "profile_layout": { "tabs": [{ "id": "stats", "visible": false }] } }),
    )
    .await;
    assert!(matches!(
        owned_skins(&ctx, &visitor, Some(UID)).await,
        Err(AppError::Forbidden)
    ));
    db.close().await;
}

#[tokio::test]
async fn a_hidden_header_currency_is_no_inventory_row_to_a_visitor() {
    let (ctx, db, owner, visitor) = owner_and_visitor().await;
    let has = |rows: &[myrtle_app::services::sync::roster::ItemEntry], id: &str| {
        rows.iter().any(|r| r.item_id == id)
    };
    let seen = inventory(&ctx, &visitor, Some(UID)).await.unwrap();
    assert!(has(&seen, "4001") && has(&seen, "4003"));

    settings(
        &ctx,
        &owner,
        json!({ "profile_layout": { "header": { "hidden": ["lmd"] } } }),
    )
    .await;
    let seen = inventory(&ctx, &visitor, Some(UID)).await.unwrap();
    assert!(!has(&seen, "4001"), "hidden LMD");
    assert!(has(&seen, "4003") && has(&seen, "30011"));
    for own in [Some(UID), None] {
        assert!(has(&inventory(&ctx, &owner, own).await.unwrap(), "4001"));
    }

    settings(
        &ctx,
        &owner,
        json!({ "profile_layout": { "tabs": [{ "id": "inventory", "visible": false }] } }),
    )
    .await;
    assert!(matches!(
        inventory(&ctx, &visitor, Some(UID)).await,
        Err(AppError::Forbidden)
    ));
    db.close().await;
}

#[test]
fn header_currencies() {
    assert!(currency_header_field("4001").is_some());
    assert!(currency_header_field("7004").is_some());
    assert!(
        currency_header_field("4004").is_none(),
        "shards are not in the header"
    );
    assert!(currency_header_field("30011").is_none());
}
