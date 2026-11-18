#![allow(clippy::unwrap_used)]

mod common;

use chrono::{DateTime, Utc};
use myrtle_db::repos::story::{self, GameStoryMark};
use myrtle_db::{Conn, DbError};
use myrtle_types::UserId;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

fn mark(id: &str, read_in_game: bool) -> GameStoryMark {
    GameStoryMark {
        story_id: id.to_owned(),
        reread_count: 0,
        unlocked_at: None,
        played: read_in_game,
        archived: false,
        cleared: false,
        read_in_game,
    }
}

async fn settings(conn: &mut Conn, user: UserId, public_profile: bool, share_stats: bool) {
    sqlx::query(
        "INSERT INTO user_settings (user_id, public_profile, share_stats) VALUES ($1, $2, $3)",
    )
    .bind(user.0)
    .bind(public_profile)
    .bind(share_stats)
    .execute(conn)
    .await
    .unwrap();
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_reader_document_is_replaced_whole(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    assert!(story::document(&mut conn, user).await.unwrap().is_none());

    story::put_document(&mut conn, user, &json!({"read": ["a"], "notes": {"a": 1}}))
        .await
        .unwrap();
    let second = story::put_document(&mut conn, user, &json!({"read": ["b"]}))
        .await
        .unwrap();
    assert_eq!(second.progress, json!({"read": ["b"]}));

    let stored = story::document(&mut conn, user).await.unwrap().unwrap();
    assert_eq!(stored.progress, json!({"read": ["b"]}));
    assert_eq!(stored.updated_at, second.updated_at);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn game_marks_are_swapped_for_the_new_set(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    let unlocked: DateTime<Utc> = DateTime::from_timestamp(1_700_000_000, 0).unwrap();
    story::replace_game_marks(&mut conn, other, &[mark("obt_x", true)])
        .await
        .unwrap();

    let first = [
        mark("obt_b", true),
        GameStoryMark {
            reread_count: 2,
            unlocked_at: Some(unlocked),
            archived: true,
            cleared: true,
            ..mark("obt_a", true)
        },
        mark("obt_c", false),
    ];
    assert_eq!(
        story::replace_game_marks(&mut conn, user, &first)
            .await
            .unwrap(),
        3
    );
    let verdicts = story::mark_verdicts(&mut conn, user).await.unwrap();
    let ids: Vec<&str> = verdicts.iter().map(|v| v.story_id.as_str()).collect();
    assert_eq!(ids, ["obt_a", "obt_b", "obt_c"]);
    assert!(verdicts[0].archived && verdicts[0].read_in_game);
    assert!(!verdicts[2].read_in_game);

    let stored = story::stored_marks(&mut conn, user).await.unwrap();
    let a = stored.iter().find(|m| m.story_id == "obt_a").unwrap();
    assert_eq!((a.reread_count, a.unlocked_at), (2, Some(unlocked)));
    assert!(a.played && a.archived);

    assert_eq!(
        story::replace_game_marks(&mut conn, user, &[mark("obt_d", true)])
            .await
            .unwrap(),
        1
    );
    let ids: Vec<String> = story::mark_verdicts(&mut conn, user)
        .await
        .unwrap()
        .into_iter()
        .map(|v| v.story_id)
        .collect();
    assert_eq!(ids, ["obt_d"]);

    assert_eq!(
        story::replace_game_marks(&mut conn, user, &[])
            .await
            .unwrap(),
        0
    );
    assert!(
        story::stored_marks(&mut conn, user)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        story::stored_marks(&mut conn, other).await.unwrap().len(),
        1
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_failed_insert_keeps_the_old_marks(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    story::replace_game_marks(
        &mut conn,
        user,
        &[mark("obt_a", true), mark("obt_b", false)],
    )
    .await
    .unwrap();

    // A repeated story id breaks the (user_id, story_id) key after the delete ran.
    let err =
        story::replace_game_marks(&mut conn, user, &[mark("obt_c", true), mark("obt_c", true)])
            .await
            .unwrap_err();
    assert!(matches!(err, DbError::Conflict { .. }), "{err:?}");

    let mut ids: Vec<String> = story::stored_marks(&mut conn, user)
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.story_id)
        .collect();
    ids.sort();
    assert_eq!(ids, ["obt_a", "obt_b"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_stage_blob_is_read_as_stored(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    sqlx::query("INSERT INTO user_stage_progress (user_id, stages) VALUES ($1, $2)")
        .bind(user.0)
        .bind(json!({"main_00-01": {"state": 3}}))
        .execute(&mut *conn)
        .await
        .unwrap();

    assert_eq!(
        story::stage_blob(&mut conn, user).await.unwrap(),
        Some(json!({"main_00-01": {"state": 3}}))
    );
    assert_eq!(story::stage_blob(&mut conn, other).await.unwrap(), None);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_community_walk_pages_visible_accounts_by_id(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let mut visible = Vec::new();
    for n in 1..=3 {
        let user = common::user(&mut conn, n).await;
        settings(&mut conn, user, true, true).await;
        visible.push(user.0);
    }
    let private = common::user(&mut conn, 4).await;
    settings(&mut conn, private, false, true).await;
    let unshared = common::user(&mut conn, 5).await;
    settings(&mut conn, unshared, true, false).await;
    common::user(&mut conn, 6).await;
    visible.sort();
    sqlx::query("INSERT INTO user_stage_progress (user_id, stages) VALUES ($1, '{\"s\": 1}')")
        .bind(visible[0])
        .execute(&mut *conn)
        .await
        .unwrap();
    story::put_document(&mut conn, UserId(visible[0]), &json!({"r": 1}))
        .await
        .unwrap();

    let first = story::community_accounts(&mut conn, None, 2).await.unwrap();
    let ids: Vec<Uuid> = first.iter().map(|a| a.id).collect();
    assert_eq!(ids, visible[..2]);
    assert_eq!(first[0].stages.as_deref(), Some(r#"{"s": 1}"#));
    assert_eq!(first[0].progress.as_deref(), Some(r#"{"r": 1}"#));
    assert_eq!(
        (first[1].stages.as_deref(), first[1].progress.as_deref()),
        (None, None)
    );

    let rest = story::community_accounts(&mut conn, Some(ids[1]), 2)
        .await
        .unwrap();
    let ids: Vec<Uuid> = rest.iter().map(|a| a.id).collect();
    assert_eq!(ids, visible[2..]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn only_marks_read_in_game_reach_the_community_tally(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let reader = common::user(&mut conn, 1).await;
    let unread = common::user(&mut conn, 2).await;
    let unasked = common::user(&mut conn, 3).await;
    story::replace_game_marks(
        &mut conn,
        reader,
        &[mark("obt_a", true), mark("obt_b", false)],
    )
    .await
    .unwrap();
    story::replace_game_marks(&mut conn, unread, &[mark("obt_a", false)])
        .await
        .unwrap();
    story::replace_game_marks(&mut conn, unasked, &[mark("obt_c", true)])
        .await
        .unwrap();

    let mut with_reads = story::accounts_with_game_reads(&mut conn).await.unwrap();
    with_reads.sort();
    let mut expected = vec![reader.0, unasked.0];
    expected.sort();
    assert_eq!(with_reads, expected);

    let rows = story::game_reads_of(&mut conn, &[reader.0, unread.0])
        .await
        .unwrap();
    let pairs: Vec<(Uuid, &str)> = rows
        .iter()
        .map(|r| (r.user_id, r.story_id.as_str()))
        .collect();
    assert_eq!(pairs, [(reader.0, "obt_a")]);
}
