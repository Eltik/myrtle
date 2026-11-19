#![allow(clippy::unwrap_used)]

mod common;

use std::collections::HashSet;
use std::time::Duration;

use myrtle_app::policy::Subject;
use myrtle_app::privacy::layout::{ProfileLayout, ProfileTabId};
use myrtle_app::privacy::{
    CREDENTIALS_TABLE, USER_TABLES, UpdateSettingsRequest, ensure_tab_visible, mask_identifier,
    resolve_public_profile, resolve_uid, resolve_user_id, resolve_user_id_as_viewer,
    update_settings,
};
use myrtle_app::services::accounts::{self, jwt::Claims};
use myrtle_app::{AppContext, AppError};
use myrtle_db::repos::{gacha, users};
use myrtle_db::tools::{self, Scope};
use myrtle_types::{Role, Server, UserId};
use serde_json::{Value, json};

async fn with_settings(ctx: &AppContext, user: UserId) {
    let mut conn = ctx.db.conn().await.unwrap();
    gacha::update_flags(&mut conn, user, None, None)
        .await
        .unwrap();
}

fn request(body: Value) -> UpdateSettingsRequest {
    serde_json::from_value(body).unwrap()
}

async fn save(ctx: &AppContext, owner: UserId, body: Value) {
    update_settings(
        ctx,
        &common::user(ctx, owner, Role::User).await,
        &request(body),
    )
    .await
    .unwrap();
}

async fn stored_layout(ctx: &AppContext, user: UserId) -> Option<Value> {
    let mut conn = ctx.db.conn().await.unwrap();
    users::profile(&mut conn, user)
        .await
        .unwrap()
        .unwrap()
        .profile_layout
}

#[tokio::test]
async fn profile_gate_outcomes() {
    let (ctx, db) = common::context().await;
    let owner = db.user(1, Server::EN).await;
    let visitor = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    let admin = common::user(&ctx, db.user(3, Server::EN).await, Role::SuperAdmin).await;
    with_settings(&ctx, owner).await;
    let me = common::user(&ctx, owner, Role::User).await;

    assert!(matches!(
        resolve_public_profile(&ctx, &visitor, "999").await,
        Err(AppError::NotFound(None))
    ));

    let (row, own) = resolve_public_profile(&ctx, &Subject::Anonymous, "1")
        .await
        .unwrap();
    assert_eq!((row.id, own), (owner.0, false));
    assert!(resolve_public_profile(&ctx, &me, "1").await.unwrap().1);

    save(&ctx, owner, json!({ "public_profile": false })).await;
    for subject in [&visitor, &admin, &Subject::Anonymous] {
        assert!(matches!(
            resolve_public_profile(&ctx, subject, "1").await,
            Err(AppError::Forbidden)
        ));
    }
    assert!(resolve_public_profile(&ctx, &me, "1").await.unwrap().1);
    db.close().await;
}

#[tokio::test]
async fn a_user_without_a_settings_row_is_private() {
    let (ctx, db) = common::context().await;
    let owner = db.user(1, Server::EN).await;
    let visitor = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    assert!(matches!(
        resolve_public_profile(&ctx, &visitor, "1").await,
        Err(AppError::Forbidden)
    ));
    let me = common::user(&ctx, owner, Role::User).await;
    assert!(resolve_public_profile(&ctx, &me, "1").await.unwrap().1);
    db.close().await;
}

#[tokio::test]
async fn no_uid_is_the_subjects_own() {
    let (ctx, db) = common::context().await;
    let owner = db.user(1, Server::EN).await;
    let me = common::user(&ctx, owner, Role::User).await;
    assert!(matches!(
        resolve_user_id(&ctx, &Subject::Anonymous, None, &[]).await,
        Err(AppError::Unauthorized)
    ));
    assert_eq!(
        resolve_user_id(&ctx, &me, None, &[ProfileTabId::Roster])
            .await
            .unwrap(),
        owner
    );
    assert_eq!(
        resolve_uid(&ctx, &me, None, ProfileTabId::Roster)
            .await
            .unwrap(),
        "1"
    );
    assert!(matches!(
        resolve_uid(&ctx, &Subject::Anonymous, None, ProfileTabId::Roster).await,
        Err(AppError::Unauthorized)
    ));
    let gone = Claims {
        sub: uuid::Uuid::from_u128(7).to_string(),
        uid: "7".into(),
        server: "en".into(),
        role: "user".into(),
        exp: u64::MAX,
        iat: 0,
    };
    let stale = accounts::subject(&ctx, &gone).await.unwrap();
    assert!(
        matches!(
            resolve_uid(&ctx, &stale, None, ProfileTabId::Roster).await,
            Err(AppError::Unauthorized)
        ),
        "the own uid comes from the row, not the token"
    );
    assert_eq!(
        resolve_user_id_as_viewer(&ctx, &me, None, ProfileTabId::Roster)
            .await
            .unwrap(),
        (owner, None)
    );
    db.close().await;
}

#[tokio::test]
async fn a_hidden_tab_refuses_visitors_and_passes_the_owner() {
    let (ctx, db) = common::context().await;
    let owner = db.user(1, Server::EN).await;
    let visitor = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    let me = common::user(&ctx, owner, Role::User).await;
    with_settings(&ctx, owner).await;

    let roster = &[ProfileTabId::Roster];
    assert_eq!(
        resolve_user_id(&ctx, &visitor, Some("1"), roster)
            .await
            .unwrap(),
        owner
    );
    assert_eq!(
        resolve_user_id_as_viewer(&ctx, &visitor, Some("1"), ProfileTabId::Roster)
            .await
            .unwrap(),
        (owner, None)
    );

    save(
        &ctx,
        owner,
        json!({ "profile_layout": { "tabs": [
            { "id": "roster", "visible": false },
            { "id": "lookback", "visible": false },
        ] } }),
    )
    .await;
    for tabs in [
        &[ProfileTabId::Roster][..],
        &[ProfileTabId::Lookback],
        &[ProfileTabId::Stats, ProfileTabId::Roster],
    ] {
        for subject in [&visitor, &Subject::Anonymous] {
            assert!(matches!(
                resolve_user_id(&ctx, subject, Some("1"), tabs).await,
                Err(AppError::Forbidden)
            ));
        }
        assert_eq!(
            resolve_user_id(&ctx, &me, Some("1"), tabs).await.unwrap(),
            owner
        );
    }
    assert_eq!(
        resolve_user_id(&ctx, &visitor, Some("1"), &[ProfileTabId::Stats])
            .await
            .unwrap(),
        owner
    );
    assert_eq!(
        resolve_uid(&ctx, &visitor, Some("1"), ProfileTabId::Stats)
            .await
            .unwrap(),
        "1"
    );
    assert!(matches!(
        resolve_uid(&ctx, &visitor, Some("1"), ProfileTabId::Roster).await,
        Err(AppError::Forbidden)
    ));

    let (_, layout) = resolve_user_id_as_viewer(&ctx, &visitor, Some("1"), ProfileTabId::Stats)
        .await
        .unwrap();
    assert!(!layout.unwrap().is_visible(ProfileTabId::Roster));
    let (_, layout) = resolve_user_id_as_viewer(&ctx, &me, Some("1"), ProfileTabId::Roster)
        .await
        .unwrap();
    assert_eq!(layout, None);

    let (row, _) = resolve_public_profile(&ctx, &visitor, "1").await.unwrap();
    assert!(ensure_tab_visible(&ctx, &visitor, &row, None, ProfileTabId::Roster).is_ok());
    let hidden =
        ProfileLayout::normalize(&json!({ "tabs": [{ "id": "plans", "visible": false }] }));
    assert!(matches!(
        ensure_tab_visible(&ctx, &visitor, &row, Some(&hidden), ProfileTabId::Plans),
        Err(AppError::Forbidden)
    ));
    db.close().await;
}

#[tokio::test]
async fn a_private_profile_answers_forbidden_before_any_tab() {
    let (ctx, db) = common::context().await;
    let owner = db.user(1, Server::EN).await;
    let visitor = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    with_settings(&ctx, owner).await;
    save(&ctx, owner, json!({ "public_profile": false })).await;
    assert!(matches!(
        resolve_user_id_as_viewer(&ctx, &visitor, Some("1"), ProfileTabId::Stats).await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        resolve_user_id(&ctx, &visitor, Some("404"), &[]).await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[test]
fn the_request_tells_absent_from_null() {
    assert_eq!(request(json!({})).profile_layout, None);
    assert_eq!(
        request(json!({ "profile_layout": null })).profile_layout,
        Some(None)
    );
    let set = request(json!({ "profile_layout": { "tabs": [] }, "public_profile": false }));
    assert_eq!(set.public_profile, Some(false));
    assert_eq!(
        set.profile_layout.unwrap().unwrap().tabs.map(|t| t.len()),
        Some(ProfileTabId::ALL.len())
    );
}

#[tokio::test]
async fn settings_need_a_signed_in_subject() {
    let (ctx, db) = common::context().await;
    assert!(matches!(
        update_settings(&ctx, &Subject::Anonymous, &request(json!({}))).await,
        Err(AppError::Unauthorized)
    ));
    db.close().await;
}

#[tokio::test]
async fn a_layout_save_merges_keys_and_null_resets() {
    let (ctx, db) = common::context().await;
    let owner = db.user(1, Server::EN).await;
    with_settings(&ctx, owner).await;

    save(
        &ctx,
        owner,
        json!({ "profile_layout": { "tabs": [{ "id": "roster", "visible": false }] } }),
    )
    .await;
    save(
        &ctx,
        owner,
        json!({ "profile_layout": { "background": { "kind": "operator", "id": " char_003_kalts " } } }),
    )
    .await;
    let stored = stored_layout(&ctx, owner).await.unwrap();
    assert_eq!(
        stored["background"],
        json!({ "kind": "operator", "id": "char_003_kalts" })
    );
    assert_eq!(
        stored["tabs"].as_array().unwrap().len(),
        ProfileTabId::ALL.len()
    );
    assert!(!ProfileLayout::normalize(&stored).is_visible(ProfileTabId::Roster));

    save(&ctx, owner, json!({ "store_gacha": false })).await;
    assert_eq!(stored_layout(&ctx, owner).await, Some(stored.clone()));

    save(
        &ctx,
        owner,
        json!({ "profile_layout": { "background": null } }),
    )
    .await;
    let after = stored_layout(&ctx, owner).await.unwrap();
    assert!(after.get("background").is_none());
    assert_eq!(after["tabs"], stored["tabs"]);

    save(&ctx, owner, json!({ "profile_layout": null })).await;
    assert_eq!(stored_layout(&ctx, owner).await, None);
    db.close().await;
}

#[tokio::test]
async fn a_save_without_a_settings_row_changes_nothing_and_succeeds() {
    let (ctx, db) = common::context().await;
    let owner = db.user(1, Server::EN).await;
    save(&ctx, owner, json!({ "public_profile": true })).await;
    let mut conn = ctx.db.conn().await.unwrap();
    assert_eq!(
        users::profile(&mut conn, owner)
            .await
            .unwrap()
            .unwrap()
            .public_profile,
        None
    );
    drop(conn);
    db.close().await;
}

const RANKED: [&str; 3] = ["search:x", "leaderboard:x", "music:stats:listeners:x"];

async fn seed_cache(ctx: &AppContext) {
    for key in RANKED.iter().chain(&["user:1", "user:2", "other:x"]) {
        ctx.cache.set(key, "v", Duration::from_mins(5)).await;
    }
}

async fn cached(ctx: &AppContext, key: &str) -> bool {
    ctx.cache.get(key).await.is_some()
}

#[tokio::test]
async fn a_save_invalidates_the_profile_and_a_ranking_change_the_rankings() {
    let (ctx, db) = common::context().await;
    let owner = db.user(1, Server::EN).await;
    with_settings(&ctx, owner).await;

    seed_cache(&ctx).await;
    save(&ctx, owner, json!({ "public_profile": false })).await;
    assert!(!cached(&ctx, "user:1").await);
    assert!(cached(&ctx, "user:2").await);
    for key in RANKED {
        assert!(cached(&ctx, key).await, "{key}");
    }

    seed_cache(&ctx).await;
    save(
        &ctx,
        owner,
        json!({ "profile_layout": { "showcase": { "blocks": [] } } }),
    )
    .await;
    for key in RANKED {
        assert!(cached(&ctx, key).await, "{key}");
    }

    for body in [
        json!({ "profile_layout": { "tabs": [] } }),
        json!({ "profile_layout": { "header": null } }),
        json!({ "profile_layout": null }),
    ] {
        seed_cache(&ctx).await;
        save(&ctx, owner, body.clone()).await;
        for key in RANKED {
            assert!(!cached(&ctx, key).await, "{body} {key}");
        }
        assert!(!cached(&ctx, "user:1").await);
        assert!(cached(&ctx, "other:x").await);
    }
    db.close().await;
}

#[test]
fn masks_identifiers() {
    assert_eq!(mask_identifier("13800138000"), "1*********0");
    assert_eq!(mask_identifier("abcd"), "****");
    assert_eq!(mask_identifier(""), "<empty>");
    assert_eq!(mask_identifier("日本語です"), "日***す");
}

#[test]
fn export_tables_are_ordered_and_scoped() {
    let names: Vec<&str> = USER_TABLES.iter().map(|t| t.name).collect();
    assert_eq!(names.iter().collect::<HashSet<_>>().len(), names.len());
    let at = |name: &str| names.iter().position(|n| *n == name).unwrap();
    assert!(at("user_operators") < at("user_operator_skills"));
    assert!(at("user_operators") < at("user_operator_modules"));
    assert!(at("plan_groups") < at("plan_group_members"));
    assert!(!names.contains(&CREDENTIALS_TABLE.name));
    assert!(!names.contains(&"users"));
    for table in USER_TABLES {
        let join = table.name == "plan_group_members";
        assert_eq!(table.scope == Scope::PlanGroup, join, "{}", table.name);
        assert_eq!(
            table.owner_column(),
            (!join).then_some("user_id"),
            "{}",
            table.name
        );
    }
    let gacha = USER_TABLES
        .iter()
        .find(|t| t.name == "gacha_records")
        .unwrap();
    assert_eq!(gacha.regenerate, ["id"]);
    assert_eq!(
        USER_TABLES
            .iter()
            .filter(|t| t.guard.is_some())
            .map(|t| t.name)
            .collect::<Vec<_>>(),
        ["tier_list_favorites"]
    );
}

#[tokio::test]
async fn every_export_table_exists_and_counts_a_users_rows() {
    let (ctx, db) = common::context().await;
    let owner = db.user(1, Server::EN).await;
    with_settings(&ctx, owner).await;
    let mut conn = ctx.db.conn().await.unwrap();
    for entry in USER_TABLES.iter().chain([&CREDENTIALS_TABLE]) {
        let table = tools::table(&mut conn, entry.name).await.unwrap().unwrap();
        let rows = tools::count_scoped(&mut conn, &table, entry.scope, owner)
            .await
            .unwrap();
        assert_eq!(
            rows,
            i64::from(entry.name == "user_settings"),
            "{}",
            entry.name
        );
        if let Some(column) = entry.owner_column() {
            let columns = tools::columns(&mut conn, &table).await.unwrap();
            assert!(columns.iter().any(|c| c.name() == column), "{}", entry.name);
        }
    }
    drop(conn);
    db.close().await;
}
