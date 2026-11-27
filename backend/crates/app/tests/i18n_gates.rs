#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::AppContext;
use myrtle_app::AppError;
use myrtle_app::policy::Subject;
use myrtle_app::services::i18n::{
    self, ClearMessageRequest, SourceEntry, TranslationGrantRequest, UpdateMessageRequest,
    UpsertLocaleRequest, UpsertOverrideRequest,
};
use myrtle_db::repos::i18n as repo;
use myrtle_types::{Role, Server, UserId};

struct Cast {
    root: UserId,
    super_admin: Subject,
    fr_editor: Subject,
    fr_lead: Subject,
    ja_editor: Subject,
    plain: Subject,
    plain_id: UserId,
}

async fn cast(ctx: &AppContext, db: &myrtle_testkit::db::TestDb) -> Cast {
    let mut conn = ctx.db.conn().await.unwrap();
    for (code, order) in [("fr", 10), ("ja", 20)] {
        repo::upsert_locale(&mut conn, code, code, code, Some("en"), "en", true, order)
            .await
            .unwrap();
    }
    let root = db.user(1, Server::EN).await;
    let fr_editor = db.user(2, Server::EN).await;
    let fr_lead = db.user(3, Server::EN).await;
    let ja_editor = db.user(4, Server::EN).await;
    let plain = db.user(5, Server::EN).await;
    for (locale, user, permission) in [
        ("fr", fr_editor, "edit"),
        ("fr", fr_lead, "admin"),
        ("ja", ja_editor, "edit"),
    ] {
        repo::grant_permission(&mut conn, locale, user, permission, root)
            .await
            .unwrap();
    }
    drop(conn);
    let super_admin = common::user(ctx, root, Role::SuperAdmin).await;
    i18n::sync_source_catalog(
        ctx,
        &super_admin,
        &[SourceEntry {
            key: "roster.count".into(),
            namespace: "roster".into(),
            source_text: "{count} operators".into(),
            description: None,
            placeholders: vec!["count".into()],
        }],
    )
    .await
    .unwrap();
    Cast {
        root,
        super_admin,
        fr_editor: common::user(ctx, fr_editor, Role::Translator).await,
        fr_lead: common::user(ctx, fr_lead, Role::User).await,
        ja_editor: common::user(ctx, ja_editor, Role::Translator).await,
        plain: common::user(ctx, plain, Role::User).await,
        plain_id: plain,
    }
}

fn update(locale: &str, value: &str) -> UpdateMessageRequest {
    UpdateMessageRequest {
        locale: locale.into(),
        key: "roster.count".into(),
        value: value.into(),
    }
}

fn grant_req(locale: &str, user: UserId, permission: &str) -> TranslationGrantRequest {
    TranslationGrantRequest {
        locale: locale.into(),
        user_id: user,
        permission: permission.into(),
    }
}

#[tokio::test]
async fn writing_a_locale_needs_super_admin_or_a_grant_for_that_locale() {
    let (ctx, db) = common::context().await;
    let c = cast(&ctx, &db).await;

    for allowed in [&c.super_admin, &c.fr_editor, &c.fr_lead] {
        i18n::update_message(&ctx, allowed, &update("fr", "{count} opérateurs"))
            .await
            .unwrap();
    }
    for refused in [&c.ja_editor, &c.plain] {
        let err = i18n::update_message(&ctx, refused, &update("fr", "{count} x"))
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Forbidden), "{err:?}");
        let err = i18n::clear_message(
            &ctx,
            refused,
            &ClearMessageRequest {
                locale: "fr".into(),
                key: "roster.count".into(),
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::Forbidden));
    }
    i18n::update_message(&ctx, &c.ja_editor, &update("ja", "{count} 人"))
        .await
        .unwrap();
    let err = i18n::update_message(&ctx, &Subject::Anonymous, &update("fr", "{count}"))
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::Unauthorized));

    let ov = UpsertOverrideRequest {
        locale: "fr".into(),
        kind: "character".into(),
        entity_id: "char_002_amiya".into(),
        field: "name".into(),
        value: "Amiya".into(),
    };
    i18n::upsert_override(&ctx, &c.fr_editor, &ov)
        .await
        .unwrap();
    let err = i18n::upsert_override(&ctx, &c.ja_editor, &ov)
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::Forbidden));
    db.close().await;
}

#[tokio::test]
async fn a_new_grant_works_on_the_next_call_without_a_new_token() {
    let (ctx, db) = common::context().await;
    let c = cast(&ctx, &db).await;
    assert!(
        i18n::update_message(&ctx, &c.plain, &update("fr", "{count} op"))
            .await
            .is_err()
    );
    i18n::grant(&ctx, &c.super_admin, &grant_req("fr", c.plain_id, "edit"))
        .await
        .unwrap();
    i18n::update_message(&ctx, &c.plain, &update("fr", "{count} op"))
        .await
        .unwrap();
    db.close().await;
}

#[tokio::test]
async fn granting_needs_super_admin_or_an_admin_grant_for_that_locale() {
    let (ctx, db) = common::context().await;
    let c = cast(&ctx, &db).await;

    i18n::grant(&ctx, &c.super_admin, &grant_req("ja", c.plain_id, "edit"))
        .await
        .unwrap();
    i18n::grant(&ctx, &c.fr_lead, &grant_req("fr", c.plain_id, "EDIT"))
        .await
        .unwrap();
    for (refused, locale) in [
        (&c.fr_lead, "ja"),
        (&c.fr_editor, "fr"),
        (&c.ja_editor, "fr"),
        (&c.plain, "fr"),
    ] {
        let err = i18n::grant(&ctx, refused, &grant_req(locale, c.plain_id, "edit"))
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Forbidden), "{locale}: {err:?}");
        let err = i18n::revoke(&ctx, refused, &grant_req(locale, c.plain_id, "edit"))
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Forbidden));
    }

    let mine: Vec<(String, String)> = i18n::my_grants(&ctx, &c.plain)
        .await
        .unwrap()
        .into_iter()
        .map(|g| (g.locale, g.permission))
        .collect();
    assert_eq!(
        mine,
        [("fr".into(), "edit".into()), ("ja".into(), "edit".into())]
    );

    i18n::revoke(&ctx, &c.fr_lead, &grant_req("fr", c.plain_id, "edit"))
        .await
        .unwrap();
    i18n::revoke(&ctx, &c.fr_lead, &grant_req("fr", c.plain_id, "edit"))
        .await
        .unwrap();
    let mine = i18n::my_grants(&ctx, &c.plain).await.unwrap();
    assert_eq!(mine.len(), 1);
    assert_eq!(mine[0].locale, "ja");
    assert_eq!(mine[0].granted_by, Some(c.root.0));
    db.close().await;
}

#[tokio::test]
async fn a_grant_validates_permission_locale_and_account() {
    let (ctx, db) = common::context().await;
    let c = cast(&ctx, &db).await;
    let err = i18n::grant(&ctx, &c.super_admin, &grant_req("fr", c.plain_id, "owner"))
        .await
        .unwrap_err();
    assert!(matches!(&err, AppError::BadRequest(m) if m == "Unknown permission: owner"));
    let err = i18n::grant(&ctx, &c.super_admin, &grant_req("xx", c.plain_id, "edit"))
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::NotFound(None)));
    let ghost = UserId(uuid::Uuid::new_v4());
    let err = i18n::grant(&ctx, &c.super_admin, &grant_req("fr", ghost, "edit"))
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::NotFound(None)));
    db.close().await;
}

#[tokio::test]
async fn the_workspace_opens_for_staff_and_grant_holders_only() {
    let (ctx, db) = common::context().await;
    let c = cast(&ctx, &db).await;
    let editor = common::user(&ctx, db.user(9, Server::EN).await, Role::TierListEditor).await;

    for allowed in [&c.super_admin, &c.fr_editor, &c.fr_lead, &editor] {
        assert_eq!(i18n::list_locales(&ctx, allowed).await.unwrap().len(), 3);
        i18n::list_namespaces(&ctx, allowed).await.unwrap();
        i18n::locale_progress(&ctx, allowed).await.unwrap();
    }
    let err = i18n::list_locales(&ctx, &c.plain).await.unwrap_err();
    assert!(matches!(err, AppError::Forbidden));
    let err = i18n::locale_progress(&ctx, &Subject::Anonymous)
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::Unauthorized));
    db.close().await;
}

#[tokio::test]
async fn writable_locales_are_every_locale_for_a_super_admin_else_the_grants() {
    let (ctx, db) = common::context().await;
    let c = cast(&ctx, &db).await;
    assert_eq!(
        i18n::writable_locales(&ctx, &c.super_admin).await.unwrap(),
        ["en", "fr", "ja"]
    );
    assert_eq!(
        i18n::writable_locales(&ctx, &c.fr_editor).await.unwrap(),
        ["fr"]
    );
    assert_eq!(
        i18n::writable_locales(&ctx, &c.fr_lead).await.unwrap(),
        ["fr"]
    );
    assert_eq!(
        i18n::writable_locales(&ctx, &c.ja_editor).await.unwrap(),
        ["ja"]
    );
    assert!(matches!(
        i18n::writable_locales(&ctx, &c.plain).await,
        Err(AppError::Forbidden)
    ));
    db.close().await;
}

#[tokio::test]
async fn a_granted_translator_gets_the_codes_sorted_not_in_locale_order() {
    let (ctx, db) = common::context().await;
    let c = cast(&ctx, &db).await;
    let both = db.user(8, Server::EN).await;
    let mut conn = ctx.db.conn().await.unwrap();
    repo::upsert_locale(&mut conn, "de", "de", "de", Some("en"), "en", true, 30)
        .await
        .unwrap();
    for locale in ["ja", "de"] {
        repo::grant_permission(&mut conn, locale, both, "edit", c.root)
            .await
            .unwrap();
    }
    drop(conn);
    assert_eq!(
        i18n::writable_locales(&ctx, &common::user(&ctx, both, Role::Translator).await)
            .await
            .unwrap(),
        ["de", "ja"]
    );
    assert_eq!(
        i18n::writable_locales(&ctx, &c.super_admin).await.unwrap(),
        ["en", "fr", "ja", "de"]
    );
    db.close().await;
}

#[tokio::test]
async fn the_grant_list_names_everyone_only_for_tier_list_admins() {
    let (ctx, db) = common::context().await;
    let c = cast(&ctx, &db).await;
    let list_admin = common::user(&ctx, db.user(9, Server::EN).await, Role::TierListAdmin).await;

    assert_eq!(
        i18n::list_grants(&ctx, &c.super_admin, None)
            .await
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        i18n::list_grants(&ctx, &list_admin, Some("fr"))
            .await
            .unwrap()
            .len(),
        2
    );
    let own = i18n::list_grants(&ctx, &c.fr_editor, None).await.unwrap();
    assert_eq!(own.len(), 1);
    assert_eq!(own[0].permission, "edit");
    assert!(matches!(
        i18n::list_grants(&ctx, &c.plain, None).await,
        Err(AppError::Forbidden)
    ));
    db.close().await;
}

#[tokio::test]
async fn locales_and_key_sync_are_super_admin_only() {
    let (ctx, db) = common::context().await;
    let c = cast(&ctx, &db).await;
    let req = UpsertLocaleRequest {
        code: "de".into(),
        english_name: "German".into(),
        native_name: "Deutsch".into(),
        fallback_locale: Some("en".into()),
        gamedata_server: "en".into(),
        enabled: false,
        sort_order: None,
    };
    for refused in [&c.fr_lead, &c.fr_editor, &c.plain] {
        assert!(matches!(
            i18n::upsert_locale(&ctx, refused, &req).await,
            Err(AppError::Forbidden)
        ));
        let entries = [SourceEntry {
            key: "a".into(),
            namespace: "n".into(),
            source_text: "A".into(),
            description: None,
            placeholders: vec![],
        }];
        assert!(matches!(
            i18n::sync_source_catalog(&ctx, refused, &entries).await,
            Err(AppError::Forbidden)
        ));
    }
    let de = i18n::upsert_locale(&ctx, &c.super_admin, &req)
        .await
        .unwrap();
    assert_eq!(de.sort_order, 100);
    assert!(!de.enabled && !de.is_source);
    db.close().await;
}
