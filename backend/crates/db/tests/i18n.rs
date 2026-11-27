#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::repos::i18n;
use myrtle_db::{Conn, DbError};
use myrtle_types::UserId;
use serde_json::json;
use sqlx::PgPool;

/// Replaces a seeded locale of the same code
async fn locale(conn: &mut Conn, code: &str, fallback: Option<&str>, is_source: bool) {
    sqlx::query(
        "INSERT INTO locales (code, english_name, native_name, fallback_locale, enabled, is_source)
         VALUES ($1, $1, $1, $2, true, $3)
         ON CONFLICT (code) DO UPDATE SET english_name = $1, native_name = $1,
             fallback_locale = $2, enabled = true, is_source = $3, sort_order = 0",
    )
    .bind(code)
    .bind(fallback)
    .bind(is_source)
    .execute(conn)
    .await
    .unwrap();
}

/// Active keys, each `(key, namespace, source_hash)` with source text `<key> source`
async fn keys(conn: &mut Conn, rows: &[(&str, &str, &str)]) {
    let keys: Vec<String> = rows.iter().map(|r| r.0.to_owned()).collect();
    let namespaces: Vec<String> = rows.iter().map(|r| r.1.to_owned()).collect();
    let hashes: Vec<String> = rows.iter().map(|r| r.2.to_owned()).collect();
    let texts: Vec<String> = rows.iter().map(|r| format!("{} source", r.0)).collect();
    i18n::sync_keys(
        conn,
        &keys,
        &namespaces,
        &texts,
        &hashes,
        &vec![None; rows.len()],
        &vec![json!([]); rows.len()],
    )
    .await
    .unwrap();
}

async fn translate(conn: &mut Conn, key: &str, locale: &str, value: &str, hash: &str, by: UserId) {
    i18n::upsert_message(conn, key, locale, value, hash, "old source", by)
        .await
        .unwrap();
}

fn entry_keys(rows: &[i18n::EntryRow]) -> Vec<&str> {
    rows.iter().map(|r| r.key.as_str()).collect()
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn locales_list_by_sort_order_and_the_enabled_filter_drops_disabled_ones(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    i18n::upsert_locale(&mut conn, "ko", "Korean", "한국어", None, "kr", true, 20)
        .await
        .unwrap();
    i18n::upsert_locale(&mut conn, "ja", "Japanese", "日本語", None, "jp", false, 10)
        .await
        .unwrap();
    let fr = i18n::upsert_locale(&mut conn, "fr", "French", "Français", None, "en", true, 20)
        .await
        .unwrap();
    assert!(!fr.is_source);
    assert_eq!(fr.gamedata_server, "en");

    let all: Vec<String> = i18n::list_locales(&mut conn, false)
        .await
        .unwrap()
        .into_iter()
        .map(|l| l.code)
        .collect();
    assert_eq!(all, ["en", "ja", "fr", "ko"]);
    let enabled: Vec<String> = i18n::list_locales(&mut conn, true)
        .await
        .unwrap()
        .into_iter()
        .map(|l| l.code)
        .collect();
    assert_eq!(enabled, ["en", "fr", "ko"]);

    let ja = i18n::get_locale(&mut conn, "ja").await.unwrap().unwrap();
    assert_eq!((ja.native_name.as_str(), ja.enabled), ("日本語", false));
    assert!(i18n::get_locale(&mut conn, "zz").await.unwrap().is_none());
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_source_locale_stays_enabled_without_a_fallback_through_an_upsert(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    locale(&mut conn, "en", None, true).await;
    locale(&mut conn, "fr", None, false).await;

    let en = i18n::upsert_locale(
        &mut conn,
        "en",
        "English",
        "English",
        Some("fr"),
        "en",
        false,
        1,
    )
    .await
    .unwrap();
    assert_eq!(en.fallback_locale, None);
    assert!(en.enabled && en.is_source);
    assert_eq!((en.english_name.as_str(), en.sort_order), ("English", 1));

    let fr = i18n::upsert_locale(
        &mut conn,
        "fr",
        "French",
        "Français",
        Some("en"),
        "en",
        false,
        5,
    )
    .await
    .unwrap();
    assert_eq!(fr.fallback_locale.as_deref(), Some("en"));
    assert!(!fr.enabled);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_key_the_extractor_stops_seeing_is_retired_and_comes_back_on_the_next_sync(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    locale(&mut conn, "fr", None, false).await;
    keys(&mut conn, &[("a.one", "a", "h1"), ("b.one", "b", "h1")]).await;

    assert_eq!(
        i18n::deactivate_missing_keys(&mut conn, &["a.one".to_owned()])
            .await
            .unwrap(),
        1
    );
    assert_eq!(i18n::list_namespaces(&mut conn).await.unwrap(), ["a"]);
    assert!(
        i18n::get_entry(&mut conn, "fr", "b.one")
            .await
            .unwrap()
            .is_none()
    );

    let synced = i18n::sync_keys(
        &mut conn,
        &["b.one".to_owned()],
        &["c".to_owned()],
        &["new text".to_owned()],
        &["h2".to_owned()],
        &[Some("a note".to_owned())],
        &[json!(["count"])],
    )
    .await
    .unwrap();
    assert_eq!(synced, 1);
    assert_eq!(i18n::list_namespaces(&mut conn).await.unwrap(), ["a", "c"]);
    let back = i18n::get_entry(&mut conn, "fr", "b.one")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(back.namespace, "c");
    assert_eq!(back.source_text, "new text");
    assert_eq!(back.source_hash, "h2");
    assert_eq!(back.description.as_deref(), Some("a note"));
    assert_eq!(back.placeholders, json!(["count"]));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_catalog_falls_back_per_key_and_its_hash_moves_with_an_inherited_value(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    locale(&mut conn, "fr", None, false).await;
    locale(&mut conn, "fr-ca", Some("fr"), false).await;
    keys(
        &mut conn,
        &[("a", "x", "h"), ("b", "y", "h"), ("c", "x", "h")],
    )
    .await;
    translate(&mut conn, "a", "fr", "A fr", "h", user).await;
    translate(&mut conn, "b", "fr", "B fr", "h", user).await;
    translate(&mut conn, "a", "fr-ca", "A ca", "h", user).await;

    let catalog = i18n::get_catalog(&mut conn, "fr-ca", "").await.unwrap();
    assert_eq!(
        catalog,
        [
            ("a".to_owned(), "A ca".to_owned()),
            ("b".to_owned(), "B fr".to_owned())
        ]
    );
    let x = i18n::get_catalog(&mut conn, "fr-ca", "x").await.unwrap();
    assert_eq!(x, [("a".to_owned(), "A ca".to_owned())]);

    let before = i18n::catalog_hash(&mut conn, "fr-ca", "").await.unwrap();
    assert_eq!(before.len(), 12);
    assert_eq!(
        i18n::catalog_hash(&mut conn, "fr-ca", "").await.unwrap(),
        before
    );
    translate(&mut conn, "b", "fr", "B fr 2", "h", user).await;
    assert_ne!(
        i18n::catalog_hash(&mut conn, "fr-ca", "").await.unwrap(),
        before
    );

    assert_eq!(
        i18n::catalog_hash(&mut conn, "zz", "").await.unwrap(),
        "d41d8cd98f00"
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn entry_filters_split_translated_stale_and_untranslated_keys(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    locale(&mut conn, "fr", None, false).await;
    keys(
        &mut conn,
        &[("a", "n", "h1"), ("b", "n", "h1"), ("c", "n", "h1")],
    )
    .await;
    translate(&mut conn, "a", "fr", "A", "h1", user).await;
    translate(&mut conn, "b", "fr", "B", "h0", user).await;

    let cases = [
        ("untranslated", vec!["c"]),
        ("stale", vec!["b"]),
        ("todo", vec!["b", "c"]),
        ("translated", vec!["a"]),
        ("all", vec!["a", "b", "c"]),
        ("unknown", vec!["a", "b", "c"]),
    ];
    for (filter, expected) in cases {
        let rows = i18n::list_entries(&mut conn, "fr", "", "", filter, 50, 0)
            .await
            .unwrap();
        assert_eq!(entry_keys(&rows), expected, "{filter}");
        let count = i18n::count_entries(&mut conn, "fr", "", "", filter)
            .await
            .unwrap();
        assert_eq!(count, i64::try_from(expected.len()).unwrap(), "{filter}");
    }

    let all = i18n::list_entries(&mut conn, "fr", "", "", "all", 50, 0)
        .await
        .unwrap();
    assert!(!all[0].is_stale && all[1].is_stale && !all[2].is_stale);
    assert_eq!(all[1].value.as_deref(), Some("B"));
    assert_eq!(all[1].translated_hash.as_deref(), Some("h0"));
    assert_eq!(all[1].translated_source_text.as_deref(), Some("old source"));
    assert_eq!(all[1].updated_by, Some(user.0));
    assert_eq!(all[2].value, None);
    assert_eq!(all[2].updated_at, None);

    let page = i18n::list_entries(&mut conn, "fr", "", "", "all", 1, 1)
        .await
        .unwrap();
    assert_eq!(entry_keys(&page), ["b"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_source_locale_reads_as_fully_translated_and_never_stale(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    locale(&mut conn, "en", None, true).await;
    keys(&mut conn, &[("a", "n", "h1"), ("b", "n", "h1")]).await;
    translate(&mut conn, "a", "en", "A override", "h0", user).await;

    let rows = i18n::list_entries(&mut conn, "en", "", "", "translated", 50, 0)
        .await
        .unwrap();
    assert_eq!(entry_keys(&rows), ["a", "b"]);
    assert_eq!(rows[0].value.as_deref(), Some("A override"));
    assert_eq!(rows[1].value.as_deref(), Some("b source"));
    assert!(rows.iter().all(|r| !r.is_stale));
    for filter in ["untranslated", "stale", "todo"] {
        assert_eq!(
            i18n::count_entries(&mut conn, "en", "", "", filter)
                .await
                .unwrap(),
            0
        );
    }
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_locale_without_a_locales_row_reads_as_a_translation_target(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    keys(&mut conn, &[("a", "n", "h1"), ("b", "n", "h1")]).await;

    for (filter, expected) in [
        ("untranslated", vec!["a", "b"]),
        ("todo", vec!["a", "b"]),
        ("stale", vec![]),
        ("translated", vec![]),
    ] {
        let rows = i18n::list_entries(&mut conn, "xx", "", "", filter, 50, 0)
            .await
            .unwrap();
        assert_eq!(entry_keys(&rows), expected, "{filter}");
        let count = i18n::count_entries(&mut conn, "xx", "", "", filter)
            .await
            .unwrap();
        assert_eq!(count, i64::try_from(expected.len()).unwrap(), "{filter}");
    }
    let entry = i18n::get_entry(&mut conn, "xx", "a")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(entry.value, None);
    assert!(!entry.is_stale);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn entry_search_matches_key_or_source_literally_within_a_namespace(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    locale(&mut conn, "fr", None, false).await;
    keys(
        &mut conn,
        &[("ui_x", "n", "h"), ("uiAx", "n", "h"), ("other", "m", "h")],
    )
    .await;

    let underscore = i18n::list_entries(&mut conn, "fr", "", "ui_", "all", 50, 0)
        .await
        .unwrap();
    assert_eq!(entry_keys(&underscore), ["ui_x"]);
    let by_source = i18n::list_entries(&mut conn, "fr", "", "OTHER SOURCE", "all", 50, 0)
        .await
        .unwrap();
    assert_eq!(entry_keys(&by_source), ["other"]);
    let in_namespace = i18n::list_entries(&mut conn, "fr", "n", "source", "all", 50, 0)
        .await
        .unwrap();
    let mut found = entry_keys(&in_namespace);
    found.sort_unstable();
    assert_eq!(found, ["uiAx", "ui_x"]);
    assert_eq!(
        i18n::count_entries(&mut conn, "fr", "m", "", "all")
            .await
            .unwrap(),
        1
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_rewritten_translation_replaces_the_stamp_and_a_deleted_one_is_untranslated(
    pool: PgPool,
) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let first = common::user(&mut conn, 1).await;
    let second = common::user(&mut conn, 2).await;
    locale(&mut conn, "fr", None, false).await;
    keys(&mut conn, &[("a", "n", "h1")]).await;
    translate(&mut conn, "a", "fr", "old", "h0", first).await;
    i18n::upsert_message(&mut conn, "a", "fr", "new", "h1", "a source", second)
        .await
        .unwrap();

    let entry = i18n::get_entry(&mut conn, "fr", "a")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(entry.value.as_deref(), Some("new"));
    assert_eq!(entry.translated_source_text.as_deref(), Some("a source"));
    assert_eq!(entry.updated_by, Some(second.0));
    assert!(!entry.is_stale);

    i18n::delete_message(&mut conn, "a", "fr").await.unwrap();
    let entry = i18n::get_entry(&mut conn, "fr", "a")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(entry.value, None);
    assert!(
        i18n::get_entry(&mut conn, "fr", "missing")
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_audit_log_reads_per_key_and_as_a_filtered_newest_first_feed(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let alice = common::user(&mut conn, 1).await;
    let bob = common::user(&mut conn, 2).await;
    for (key, locale, by) in [
        ("a", "fr", alice),
        ("a", "fr", bob),
        ("a", "ja", alice),
        ("b", "fr", bob),
    ] {
        i18n::insert_audit(&mut conn, key, locale, Some("old"), Some("new"), by)
            .await
            .unwrap();
    }
    sqlx::query(
        "UPDATE ui_message_audit_log SET changed_at = timestamptz '2026-01-01' + id * interval '1 hour'",
    )
    .execute(&mut *conn)
    .await
    .unwrap();

    let history = i18n::get_audit_log(&mut conn, "a", "fr").await.unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].changed_by, bob.0);
    assert_eq!(history[0].actor_nickname.as_deref(), Some("user2"));
    assert_eq!(history[1].changed_by, alice.0);
    assert_eq!(history[1].old_value.as_deref(), Some("old"));

    let feed = i18n::get_audit_log_global(&mut conn, None, None, 10, None)
        .await
        .unwrap();
    let ids: Vec<i64> = feed.iter().map(|r| r.id).collect();
    assert_eq!(ids.len(), 4);
    assert!(ids.windows(2).all(|w| w[0] > w[1]));
    assert_eq!(feed[0].actor_uid.as_deref(), Some("2"));

    let fr = i18n::get_audit_log_global(&mut conn, Some("fr"), None, 10, None)
        .await
        .unwrap();
    assert_eq!(fr.len(), 3);
    let fr_by_alice = i18n::get_audit_log_global(&mut conn, Some("fr"), Some(alice), 10, None)
        .await
        .unwrap();
    assert_eq!(fr_by_alice.len(), 1);
    assert_eq!(fr_by_alice[0].actor_user_id, alice.0);
    let older = i18n::get_audit_log_global(&mut conn, None, None, 1, Some(feed[1].changed_at))
        .await
        .unwrap();
    assert_eq!(older.len(), 1);
    assert_eq!(older[0].id, feed[2].id);

    assert_eq!(
        i18n::count_audit_log(&mut conn, None, None).await.unwrap(),
        4
    );
    assert_eq!(
        i18n::count_audit_log(&mut conn, Some("fr"), Some(bob))
            .await
            .unwrap(),
        2
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn locale_progress_counts_active_keys_only_and_stale_rows_where_completion_does_not(
    pool: PgPool,
) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    locale(&mut conn, "en", None, true).await;
    locale(&mut conn, "fr", None, false).await;
    keys(
        &mut conn,
        &[("a", "n", "h1"), ("b", "n", "h1"), ("c", "n", "h1")],
    )
    .await;
    translate(&mut conn, "a", "fr", "A", "h1", user).await;
    translate(&mut conn, "b", "fr", "B", "h0", user).await;
    translate(&mut conn, "c", "fr", "C", "h1", user).await;
    i18n::deactivate_missing_keys(&mut conn, &["a".to_owned(), "b".to_owned()])
        .await
        .unwrap();

    let mut completion: Vec<(String, i64, i64)> = i18n::locale_completion(&mut conn)
        .await
        .unwrap()
        .into_iter()
        .map(|r| (r.code, r.current, r.total))
        .collect();
    completion.sort();
    assert_eq!(
        completion,
        [("en".to_owned(), 2, 2), ("fr".to_owned(), 1, 2)]
    );

    let progress: Vec<(String, i64, i64, i64)> = i18n::locale_progress(&mut conn)
        .await
        .unwrap()
        .into_iter()
        .map(|r| (r.code, r.total, r.translated, r.stale))
        .collect();
    assert_eq!(
        progress,
        [("en".to_owned(), 2, 2, 0), ("fr".to_owned(), 2, 2, 1)],
        "the retired key's message counts toward neither total nor translated"
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn grants_list_with_names_and_a_repeat_grant_refreshes_the_granter(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let admin = common::user(&mut conn, 1).await;
    let other_admin = common::user(&mut conn, 2).await;
    let translator = common::user(&mut conn, 3).await;
    let outsider = common::user(&mut conn, 4).await;
    locale(&mut conn, "fr", None, false).await;
    locale(&mut conn, "ja", None, false).await;

    i18n::grant_permission(&mut conn, "ja", translator, "edit", admin)
        .await
        .unwrap();
    i18n::grant_permission(&mut conn, "fr", translator, "edit", admin)
        .await
        .unwrap();
    i18n::grant_permission(&mut conn, "fr", translator, "edit", other_admin)
        .await
        .unwrap();
    i18n::grant_permission(&mut conn, "fr", admin, "review", admin)
        .await
        .unwrap();

    let all = i18n::list_permissions(&mut conn, None, None).await.unwrap();
    let shape: Vec<(&str, &str)> = all
        .iter()
        .map(|g| (g.locale.as_str(), g.permission.as_str()))
        .collect();
    assert_eq!(shape, [("fr", "edit"), ("fr", "review"), ("ja", "edit")]);
    assert_eq!(all[0].granted_by, Some(other_admin.0));
    assert_eq!(all[0].granted_by_nickname.as_deref(), Some("user2"));
    assert_eq!(all[0].user_uid.as_deref(), Some("3"));
    assert_eq!(all[0].user_nickname.as_deref(), Some("user3"));

    let fr_translator = i18n::list_permissions(&mut conn, Some("fr"), Some(translator))
        .await
        .unwrap();
    assert_eq!(fr_translator.len(), 1);
    let mine: Vec<String> = i18n::list_permissions_for_user(&mut conn, translator)
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.locale)
        .collect();
    assert_eq!(mine, ["fr", "ja"]);

    assert!(
        i18n::has_any_permission(&mut conn, translator)
            .await
            .unwrap()
    );
    assert!(!i18n::has_any_permission(&mut conn, outsider).await.unwrap());

    i18n::revoke_permission(&mut conn, "ja", translator, "edit")
        .await
        .unwrap();
    i18n::revoke_permission(&mut conn, "fr", translator, "edit")
        .await
        .unwrap();
    assert!(
        !i18n::has_any_permission(&mut conn, translator)
            .await
            .unwrap()
    );
    assert_eq!(
        i18n::list_permissions(&mut conn, None, None)
            .await
            .unwrap()
            .len(),
        1
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_grant_on_an_unknown_locale_or_account_violates_a_foreign_key_and_writes_nothing(
    pool: PgPool,
) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let admin = common::user(&mut conn, 1).await;
    locale(&mut conn, "fr", None, false).await;

    let err = i18n::grant_permission(&mut conn, "zz-not-a-locale", admin, "edit", admin)
        .await
        .unwrap_err();
    assert!(
        matches!(&err, DbError::ForeignKey { constraint } if constraint == "translation_permissions_locale_fkey"),
        "{err:?}"
    );
    let nobody = UserId(uuid::Uuid::new_v4());
    let err = i18n::grant_permission(&mut conn, "fr", nobody, "edit", admin)
        .await
        .unwrap_err();
    assert!(
        matches!(&err, DbError::ForeignKey { constraint } if constraint == "translation_permissions_user_id_fkey"),
        "{err:?}"
    );
    assert!(
        i18n::list_permissions(&mut conn, None, None)
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn gamedata_overrides_are_kept_per_locale_and_replaced_by_field(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    locale(&mut conn, "fr", None, false).await;
    locale(&mut conn, "ja", None, false).await;
    for (kind, entity, field, value) in [
        ("skill", "skchr_1", "name", "S"),
        ("char", "char_2", "name", "B"),
        ("char", "char_1", "name", "A"),
        ("char", "char_1", "desc", "D"),
        ("char", "char_1", "name", "A2"),
    ] {
        i18n::upsert_override(&mut conn, "fr", kind, entity, field, value, user)
            .await
            .unwrap();
    }
    i18n::upsert_override(&mut conn, "ja", "char", "char_1", "name", "J", user)
        .await
        .unwrap();

    let fr: Vec<(String, String, String, String)> = i18n::list_overrides(&mut conn, "fr")
        .await
        .unwrap()
        .into_iter()
        .map(|o| (o.kind, o.entity_id, o.field, o.value))
        .collect();
    let expected = [
        ("char", "char_1", "desc", "D"),
        ("char", "char_1", "name", "A2"),
        ("char", "char_2", "name", "B"),
        ("skill", "skchr_1", "name", "S"),
    ]
    .map(|(a, b, c, d)| (a.to_owned(), b.to_owned(), c.to_owned(), d.to_owned()));
    assert_eq!(fr, expected);

    i18n::delete_override(&mut conn, "fr", "char", "char_1", "name")
        .await
        .unwrap();
    assert_eq!(
        i18n::list_overrides(&mut conn, "fr").await.unwrap().len(),
        3
    );
    assert_eq!(
        i18n::list_overrides(&mut conn, "ja").await.unwrap().len(),
        1
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_document_is_replaced_in_its_slug_and_locale(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    locale(&mut conn, "fr", None, false).await;
    locale(&mut conn, "ja", None, false).await;
    assert!(
        i18n::get_document(&mut conn, "privacy", "fr")
            .await
            .unwrap()
            .is_none()
    );

    i18n::upsert_document(&mut conn, "privacy", "fr", "v1", "body 1", user)
        .await
        .unwrap();
    i18n::upsert_document(&mut conn, "privacy", "fr", "v2", "body 2", user)
        .await
        .unwrap();
    let doc = i18n::get_document(&mut conn, "privacy", "fr")
        .await
        .unwrap()
        .unwrap();
    assert_eq!((doc.title.as_str(), doc.body.as_str()), ("v2", "body 2"));
    assert_eq!((doc.slug.as_str(), doc.locale.as_str()), ("privacy", "fr"));
    assert!(
        i18n::get_document(&mut conn, "privacy", "ja")
            .await
            .unwrap()
            .is_none()
    );
}
