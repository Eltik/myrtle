#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::repos::operator_notes;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_upsert_replaces_only_the_fields_it_is_given(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();

    let first = operator_notes::upsert(
        &mut conn,
        "char_002_amiya",
        Some("pros"),
        Some("cons"),
        None,
        None,
        Some("short"),
        Some(&json!(["caster"])),
    )
    .await
    .unwrap();
    assert_eq!(first.notes, None);

    let second = operator_notes::upsert(
        &mut conn,
        "char_002_amiya",
        None,
        Some("new cons"),
        Some("notes"),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(second.id, first.id);
    assert_eq!(second.created_at, first.created_at);
    assert_eq!(second.pros.as_deref(), Some("pros"));
    assert_eq!(second.cons.as_deref(), Some("new cons"));
    assert_eq!(second.notes.as_deref(), Some("notes"));
    assert_eq!(second.summary.as_deref(), Some("short"));
    assert_eq!(second.tags, Some(json!(["caster"])));

    let fresh = operator_notes::upsert(&mut conn, "char_001_a", None, None, None, None, None, None)
        .await
        .unwrap();
    assert_eq!(fresh.tags, None);

    let ids: Vec<String> = operator_notes::get_all(&mut conn)
        .await
        .unwrap()
        .into_iter()
        .map(|n| n.operator_id)
        .collect();
    assert_eq!(ids, ["char_001_a", "char_002_amiya"]);
    let amiya = operator_notes::get_by_operator(&mut conn, "char_002_amiya")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(amiya.cons.as_deref(), Some("new cons"));
    assert!(
        operator_notes::get_by_operator(&mut conn, "char_999_none")
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_audit_log_reads_per_operator_and_as_a_filtered_newest_first_feed(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let alice = common::user(&mut conn, 1).await;
    let bob = common::user(&mut conn, 2).await;
    let amiya = operator_notes::upsert(
        &mut conn,
        "char_002_amiya",
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    let kalts = operator_notes::upsert(
        &mut conn,
        "char_003_kalts",
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    for (note, field, by) in [
        (amiya.id, "pros", alice),
        (kalts.id, "cons", bob),
        (amiya.id, "notes", bob),
    ] {
        operator_notes::insert_audit(&mut conn, note, field, None, Some("new"), by)
            .await
            .unwrap();
    }
    sqlx::query(
        "UPDATE operator_notes_audit_log
         SET changed_at = timestamptz '2026-01-01' + id * interval '1 hour'",
    )
    .execute(&mut *conn)
    .await
    .unwrap();

    let history = operator_notes::get_audit_log(&mut conn, "char_002_amiya")
        .await
        .unwrap();
    let fields: Vec<&str> = history.iter().map(|r| r.field_name.as_str()).collect();
    assert_eq!(fields, ["notes", "pros"]);
    assert_eq!(history[0].changed_by, bob.0);
    assert_eq!(history[0].actor_nickname.as_deref(), Some("user2"));
    assert_eq!(history[1].old_value, None);
    assert_eq!(history[1].note_id, amiya.id);

    let feed = operator_notes::get_audit_log_global(&mut conn, 10, None, None)
        .await
        .unwrap();
    let operators: Vec<&str> = feed.iter().map(|r| r.operator_id.as_str()).collect();
    assert_eq!(
        operators,
        ["char_002_amiya", "char_003_kalts", "char_002_amiya"]
    );
    assert_eq!(feed[1].actor_uid.as_deref(), Some("2"));

    let page = operator_notes::get_audit_log_global(&mut conn, 1, Some(feed[0].changed_at), None)
        .await
        .unwrap();
    assert_eq!(page.len(), 1);
    assert_eq!(page[0].id, feed[1].id);
    let by_alice = operator_notes::get_audit_log_global(&mut conn, 10, None, Some(alice))
        .await
        .unwrap();
    assert_eq!(by_alice.len(), 1);
    assert_eq!(by_alice[0].actor_user_id, alice.0);

    assert_eq!(
        operator_notes::count_audit_log(&mut conn, None)
            .await
            .unwrap(),
        3
    );
    assert_eq!(
        operator_notes::count_audit_log(&mut conn, Some(bob))
            .await
            .unwrap(),
        2
    );
}
