#![allow(clippy::unwrap_used)]

mod common;

use std::collections::BTreeMap;
use std::time::Duration;

use myrtle_app::AppContext;
use myrtle_app::AppError;
use myrtle_app::policy::Subject;
use myrtle_app::services::i18n::{
    self, ALL_NAMESPACES, AuditQuery, ClearMessageRequest, DeleteOverrideRequest, EntryAuditQuery,
    I18nManifest, ListMessagesQuery, SourceEntry, UpdateMessageRequest, UpsertLocaleRequest,
    UpsertOverrideRequest,
};
use myrtle_db::repos::i18n as repo;
use myrtle_types::{Role, Server, UserId};

fn entry(key: &str, namespace: &str, text: &str, placeholders: &[&str]) -> SourceEntry {
    SourceEntry {
        key: key.into(),
        namespace: namespace.into(),
        source_text: text.into(),
        description: None,
        placeholders: placeholders.iter().map(|p| (*p).to_owned()).collect(),
    }
}

/// Locale `fr` falling back to `en`, three keys over two namespaces, and a super admin
async fn seed(ctx: &AppContext, db: &myrtle_testkit::db::TestDb) -> (Subject, UserId) {
    let id = db.user(1, Server::EN).await;
    let admin = common::user(ctx, id, Role::SuperAdmin).await;
    repo::upsert_locale(
        &mut ctx.db.conn().await.unwrap(),
        "fr",
        "French",
        "Français",
        Some("en"),
        "en",
        true,
        10,
    )
    .await
    .unwrap();
    i18n::sync_source_catalog(
        ctx,
        &admin,
        &[
            entry("nav.home", "nav", "Home", &[]),
            entry("nav.back", "nav", "Back", &[]),
            entry("roster.count", "roster", "{count} operators", &["count"]),
        ],
    )
    .await
    .unwrap();
    (admin, id)
}

async fn manifest(ctx: &AppContext) -> I18nManifest {
    serde_json::from_str(&i18n::manifest(ctx).await.unwrap().body).unwrap()
}

fn fr_hash(m: &I18nManifest, ns: &str) -> String {
    m.locales
        .iter()
        .find(|l| l.code == "fr")
        .unwrap()
        .namespaces[ns]
        .clone()
}

fn update(key: &str, value: &str) -> UpdateMessageRequest {
    UpdateMessageRequest {
        locale: "fr".into(),
        key: key.into(),
        value: value.into(),
    }
}

#[tokio::test]
async fn the_manifest_lists_enabled_locales_with_a_hash_per_namespace() {
    let (ctx, db) = common::context().await;
    seed(&ctx, &db).await;
    let mut conn = ctx.db.conn().await.unwrap();
    repo::upsert_locale(&mut conn, "ja", "Japanese", "日本語", None, "jp", false, 20)
        .await
        .unwrap();
    drop(conn);

    let m = manifest(&ctx).await;
    assert_eq!(m.default_locale, "en");
    let codes: Vec<&str> = m.locales.iter().map(|l| l.code.as_str()).collect();
    assert_eq!(codes, ["en", "fr"]);
    let en = &m.locales[0];
    assert_eq!(
        en.namespaces.keys().collect::<Vec<_>>(),
        [ALL_NAMESPACES, "nav", "roster"]
    );
    assert_eq!((en.translated, en.total), (3, 3));
    let fr = &m.locales[1];
    assert_eq!((fr.translated, fr.total), (0, 3));
    assert!(m.gamedata_servers.is_empty());
    db.close().await;
}

#[tokio::test]
async fn an_edit_changes_the_catalog_hash_and_flushes_the_cached_manifest() {
    let (ctx, db) = common::context().await;
    let (admin, id) = seed(&ctx, &db).await;
    let before = manifest(&ctx).await;

    repo::upsert_message(
        &mut ctx.db.conn().await.unwrap(),
        "nav.back",
        "fr",
        "Retour",
        "h",
        "Back",
        id,
    )
    .await
    .unwrap(); // behind the service's back: hidden until the manifest expires
    assert_eq!(
        manifest(&ctx).await.locales[1].namespaces,
        before.locales[1].namespaces
    );

    i18n::update_message(&ctx, &admin, &update("nav.home", "Accueil"))
        .await
        .unwrap();
    let after = manifest(&ctx).await;
    assert_ne!(fr_hash(&after, "nav"), fr_hash(&before, "nav"));
    assert_ne!(
        fr_hash(&after, ALL_NAMESPACES),
        fr_hash(&before, ALL_NAMESPACES)
    );
    assert_eq!(fr_hash(&after, "roster"), fr_hash(&before, "roster"));
    db.close().await;
}

#[tokio::test]
async fn a_catalog_is_cached_under_its_content_hash_and_resolves_the_fallback() {
    let (ctx, db) = common::context().await;
    let (admin, id) = seed(&ctx, &db).await;
    let mut conn = ctx.db.conn().await.unwrap();
    for (key, value) in [("nav.home", "Home"), ("nav.back", "Back")] {
        repo::upsert_message(&mut conn, key, "en", value, "h", value, id)
            .await
            .unwrap();
    }
    drop(conn);
    i18n::update_message(&ctx, &admin, &update("nav.home", "Accueil"))
        .await
        .unwrap();

    let hash = fr_hash(&manifest(&ctx).await, "nav");
    let current = i18n::catalog(&ctx, "fr", "nav", &hash).await.unwrap();
    assert!(current.is_current);
    let body: BTreeMap<String, String> = serde_json::from_str(&current.json.body).unwrap();
    assert_eq!(body["nav.home"], "Accueil");
    assert_eq!(body["nav.back"], "Back");
    let key = format!("i18n:catalog:fr:nav:{hash}");
    assert!(ctx.cache.get(&key).await.is_some());

    let all = i18n::catalog(&ctx, "fr", ALL_NAMESPACES, "latest")
        .await
        .unwrap();
    assert!(!all.is_current);
    let all_hash = fr_hash(&manifest(&ctx).await, ALL_NAMESPACES);
    assert!(
        ctx.cache
            .get(&format!("i18n:catalog:fr::{all_hash}"))
            .await
            .is_some()
    );

    let stale = i18n::catalog(&ctx, "fr", "nav", "000000000000")
        .await
        .unwrap();
    assert!(!stale.is_current);
    assert_eq!(stale.json, current.json);

    i18n::update_message(&ctx, &admin, &update("nav.home", "Maison"))
        .await
        .unwrap();
    assert!(ctx.cache.get(&key).await.is_none());

    assert!(matches!(
        i18n::catalog(&ctx, "xx", "nav", &hash).await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[tokio::test]
async fn an_update_validates_stamps_the_source_and_audits_only_real_changes() {
    let (ctx, db) = common::context().await;
    let (admin, id) = seed(&ctx, &db).await;

    let err = i18n::update_message(&ctx, &admin, &update("roster.count", "{n} opérateurs"))
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
    let err = i18n::update_message(&ctx, &admin, &update("nope", "x"))
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::NotFound(None)));

    let saved = i18n::update_message(&ctx, &admin, &update("roster.count", "{count} opérateurs"))
        .await
        .unwrap();
    assert_eq!(saved.value.as_deref(), Some("{count} opérateurs"));
    assert_eq!(
        saved.translated_hash.as_deref(),
        Some(saved.source_hash.as_str())
    );
    assert_eq!(
        saved.translated_source_text.as_deref(),
        Some("{count} operators")
    );
    assert_eq!(saved.updated_by, Some(id.0));
    assert!(!saved.is_stale);

    i18n::sync_source_catalog(
        &ctx,
        &admin,
        &[
            entry("nav.home", "nav", "Home", &[]),
            entry("nav.back", "nav", "Back", &[]),
            entry("roster.count", "roster", "{count} ops", &["count"]),
        ],
    )
    .await
    .unwrap();
    let q = ListMessagesQuery {
        locale: "fr".into(),
        namespace: None,
        search: None,
        filter: Some("stale".into()),
        limit: None,
        offset: None,
    };
    let stale = i18n::list_messages(&ctx, &admin, &q).await.unwrap();
    assert_eq!(stale.total, 1);
    assert!(stale.entries[0].is_stale);

    let resaved = i18n::update_message(&ctx, &admin, &update("roster.count", "{count} opérateurs"))
        .await
        .unwrap();
    assert!(!resaved.is_stale);

    let history = i18n::entry_history(
        &ctx,
        &admin,
        &EntryAuditQuery {
            locale: "fr".into(),
            key: "roster.count".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].old_value, None);
    assert_eq!(history[0].actor.id, id.0);

    let clear = ClearMessageRequest {
        locale: "fr".into(),
        key: "roster.count".into(),
    };
    i18n::clear_message(&ctx, &admin, &clear).await.unwrap();
    i18n::clear_message(&ctx, &admin, &clear).await.unwrap();
    let feed = i18n::audit_feed(
        &ctx,
        &admin,
        &AuditQuery {
            locale: Some("fr".into()),
            actor: Some(id),
            limit: Some(0),
            before: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(feed.total, 2);
    assert_eq!(feed.entries.len(), 1);
    assert_eq!(feed.entries[0].new_value, None);
    assert_eq!(feed.entries[0].actor.user_id, id.0);
    db.close().await;
}

#[tokio::test]
async fn a_sync_refuses_an_empty_catalog_and_deactivates_vanished_keys() {
    let (ctx, db) = common::context().await;
    let (admin, _) = seed(&ctx, &db).await;
    assert!(matches!(
        i18n::sync_source_catalog(&ctx, &admin, &[]).await,
        Err(AppError::BadRequest(_))
    ));
    let before = manifest(&ctx).await;
    let result = i18n::sync_source_catalog(&ctx, &admin, &[entry("nav.home", "nav", "Home", &[])])
        .await
        .unwrap();
    assert_eq!((result.upserted, result.deactivated), (1, 2));
    assert_eq!(i18n::list_namespaces(&ctx, &admin).await.unwrap(), ["nav"]);
    assert_ne!(
        manifest(&ctx).await.locales[0].namespaces,
        before.locales[0].namespaces
    );
    db.close().await;
}

#[tokio::test]
async fn locale_and_override_writes_flush_their_caches() {
    let (ctx, db) = common::context().await;
    let (admin, _) = seed(&ctx, &db).await;
    manifest(&ctx).await;
    ctx.cache
        .set("static:operators", "{}", Duration::from_secs(60))
        .await;

    let ov = UpsertOverrideRequest {
        locale: "fr".into(),
        kind: "character".into(),
        entity_id: "char_002_amiya".into(),
        field: "name".into(),
        value: "Amiya".into(),
    };
    i18n::upsert_override(&ctx, &admin, &ov).await.unwrap();
    assert!(ctx.cache.get("static:operators").await.is_none());
    assert!(ctx.cache.get("i18n:manifest").await.is_none());
    let listed = i18n::list_overrides(&ctx, &admin, "fr").await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].value, "Amiya");

    ctx.cache
        .set("static:operators", "{}", Duration::from_secs(60))
        .await;
    let del = DeleteOverrideRequest {
        locale: "fr".into(),
        kind: "character".into(),
        entity_id: "char_002_amiya".into(),
        field: "name".into(),
    };
    i18n::delete_override(&ctx, &admin, &del).await.unwrap();
    assert!(ctx.cache.get("static:operators").await.is_none());
    assert!(
        i18n::list_overrides(&ctx, &admin, "fr")
            .await
            .unwrap()
            .is_empty()
    );

    manifest(&ctx).await;
    let req = UpsertLocaleRequest {
        code: "fr".into(),
        english_name: "French".into(),
        native_name: "Français".into(),
        fallback_locale: Some("en".into()),
        gamedata_server: "en".into(),
        enabled: false,
        sort_order: Some(10),
    };
    i18n::upsert_locale(&ctx, &admin, &req).await.unwrap();
    let codes: Vec<String> = manifest(&ctx)
        .await
        .locales
        .into_iter()
        .map(|l| l.code)
        .collect();
    assert_eq!(codes, ["en"]);
    db.close().await;
}

#[tokio::test]
async fn list_messages_clamps_the_page() {
    let (ctx, db) = common::context().await;
    let (admin, _) = seed(&ctx, &db).await;
    let q = |limit, offset| ListMessagesQuery {
        locale: "fr".into(),
        namespace: Some("nav".into()),
        search: None,
        filter: None,
        limit: Some(limit),
        offset: Some(offset),
    };
    let page = i18n::list_messages(&ctx, &admin, &q(0, -5)).await.unwrap();
    assert_eq!((page.entries.len(), page.total), (1, 2));
    assert_eq!(page.entries[0].key, "nav.back");
    let page = i18n::list_messages(&ctx, &admin, &q(10_000, 1))
        .await
        .unwrap();
    assert_eq!(page.entries.len(), 1);
    assert_eq!(page.entries[0].key, "nav.home");
    db.close().await;
}
