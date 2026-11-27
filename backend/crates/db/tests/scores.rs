#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::Conn;
use myrtle_db::repos::scores::{self, ScoreWrite};
use myrtle_types::{Server, UserId};
use sqlx::PgPool;

const WRITE: ScoreWrite<'static> = ScoreWrite {
    total_score: 0.5,
    operator_score: 0.6,
    stage_score: 0.4,
    roguelike_score: 0.3,
    sandbox_score: 0.2,
    medal_score: 0.1,
    base_score: 0.7,
    base_utilization: Some(0.8),
    base_infrastructure: Some(0.9),
    skin_score: 0.0,
    grade: Some("B"),
};

async fn public(conn: &mut Conn, user: UserId) {
    sqlx::query("INSERT INTO user_settings (user_id, public_profile) VALUES ($1, true)")
        .bind(user.0)
        .execute(&mut *conn)
        .await
        .unwrap();
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_upsert_replaces_every_section_and_restamps_the_row(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    assert!(scores::get(&mut conn, user).await.unwrap().is_none());

    scores::upsert(&mut conn, user, &WRITE).await.unwrap();
    sqlx::query("UPDATE user_scores SET calculated_at = NOW() - interval '1 day'")
        .execute(&mut *conn)
        .await
        .unwrap();
    let first = scores::get(&mut conn, user).await.unwrap().unwrap();
    assert_eq!(first.user_id, user.0);
    assert_eq!(first.grade.as_deref(), Some("B"));
    assert_eq!(first.base_utilization, Some(0.8));

    let rewrite = ScoreWrite {
        total_score: 0.9,
        base_utilization: None,
        base_infrastructure: None,
        grade: Some("S+"),
        ..WRITE
    };
    scores::upsert(&mut conn, user, &rewrite).await.unwrap();
    let second = scores::get_by_uid(&mut conn, "1", None)
        .await
        .unwrap()
        .unwrap();
    assert!((second.total_score - 0.9).abs() < f64::EPSILON);
    assert!((second.operator_score - 0.6).abs() < f64::EPSILON);
    assert_eq!(second.base_utilization, None);
    assert_eq!(second.base_infrastructure, None);
    assert_eq!(second.grade.as_deref(), Some("S+"));
    assert!(second.calculated_at > first.calculated_at);
    assert!(
        scores::get_by_uid(&mut conn, "2", None)
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_sandbox_rewrite_touches_only_sandbox_total_and_grade(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    assert!(
        scores::stored_sections(&mut conn, user)
            .await
            .unwrap()
            .is_none()
    );
    scores::write_sandbox(&mut conn, user, 1.0, 1.0, "S+")
        .await
        .unwrap();
    assert!(scores::get(&mut conn, user).await.unwrap().is_none());

    scores::upsert(&mut conn, user, &WRITE).await.unwrap();
    let stored = scores::stored_sections(&mut conn, user)
        .await
        .unwrap()
        .unwrap();
    let read = [
        stored.operator_score,
        stored.base_score,
        stored.roguelike_score,
        stored.medal_score,
        stored.stage_score,
    ];
    assert_eq!(
        read.map(f64::to_bits),
        [0.6, 0.7, 0.3, 0.1, 0.4].map(f64::to_bits)
    );

    scores::write_sandbox(&mut conn, user, 0.95, 0.55, "A")
        .await
        .unwrap();
    let row = scores::get(&mut conn, user).await.unwrap().unwrap();
    let changed = [
        row.sandbox_score,
        row.total_score,
        row.operator_score,
        row.stage_score,
    ];
    assert_eq!(
        changed.map(f64::to_bits),
        [0.95, 0.55, 0.6, 0.4].map(f64::to_bits)
    );
    assert_eq!(row.grade.as_deref(), Some("A"));
    assert_eq!(row.base_utilization, Some(0.8));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn history_lists_a_players_snapshots_oldest_first(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    for (days, rank) in [(1, 3), (5, 7), (3, 4)] {
        sqlx::query(
            "WITH s AS (
                 INSERT INTO leaderboard_snapshots (taken_at)
                 VALUES (NOW() - make_interval(days => $3)) RETURNING id
             )
             INSERT INTO leaderboard_snapshot_entries
                 (snapshot_id, user_id, server_id, rank_global, rank_server, total_score)
             SELECT s.id, u, 0, $4, $4 + 1, NULL FROM s, UNNEST(ARRAY[$1, $2]::uuid[]) u",
        )
        .bind(user.0)
        .bind(other.0)
        .bind(days)
        .bind(rank)
        .execute(&mut *conn)
        .await
        .unwrap();
    }

    let points = scores::history(&mut conn, "1", None).await.unwrap();
    let ranks: Vec<(i32, i32)> = points
        .iter()
        .map(|p| (p.rank_global, p.rank_server))
        .collect();
    assert_eq!(ranks, [(7, 8), (4, 5), (3, 4)]);
    assert!(points.windows(2).all(|w| w[0].taken_at < w[1].taken_at));
    assert!(points.iter().all(|p| p.total_score.is_none()));
    assert!(
        scores::history(&mut conn, "3", None)
            .await
            .unwrap()
            .is_empty()
    );
}

/// uid `1` on EN, then on JP with an earlier `created_at`: JP is the account a bare uid picks
async fn shared_uid(conn: &mut Conn) -> (UserId, UserId) {
    let en = common::user_on(conn, 1, Server::EN).await;
    let jp = common::user_on(conn, 1, Server::JP).await;
    sqlx::query("UPDATE users SET created_at = NOW() - interval '1 day' WHERE id = $1")
        .bind(jp.0)
        .execute(&mut *conn)
        .await
        .unwrap();
    (en, jp)
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_score_by_uid_reads_the_named_server_or_only_the_oldest_account(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let (en, jp) = shared_uid(&mut conn).await;
    scores::upsert(&mut conn, en, &WRITE).await.unwrap();

    let on_en = scores::get_by_uid(&mut conn, "1", Some(Server::EN))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(on_en.user_id, en.0);
    assert!(
        scores::get_by_uid(&mut conn, "1", None)
            .await
            .unwrap()
            .is_none(),
        "the oldest account has no score, the other account's is not borrowed"
    );

    let jp_write = ScoreWrite {
        total_score: 0.1,
        ..WRITE
    };
    scores::upsert(&mut conn, jp, &jp_write).await.unwrap();
    let bare = scores::get_by_uid(&mut conn, "1", None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(bare.user_id, jp.0);
    assert!(
        scores::get_by_uid(&mut conn, "1", Some(Server::KR))
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn history_by_uid_never_mixes_two_accounts(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let (en, jp) = shared_uid(&mut conn).await;
    for (days, user, rank) in [(2, en, 10), (1, jp, 20), (3, jp, 30)] {
        sqlx::query(
            "WITH s AS (
                 INSERT INTO leaderboard_snapshots (taken_at)
                 VALUES (NOW() - make_interval(days => $2)) RETURNING id
             )
             INSERT INTO leaderboard_snapshot_entries
                 (snapshot_id, user_id, server_id, rank_global, rank_server, total_score)
             SELECT s.id, $1, 0, $3, $3, NULL FROM s",
        )
        .bind(user.0)
        .bind(days)
        .bind(rank)
        .execute(&mut *conn)
        .await
        .unwrap();
    }

    let ranks = |rows: Vec<scores::ScoreHistoryRow>| -> Vec<i32> {
        rows.iter().map(|r| r.rank_global).collect()
    };
    let bare = scores::history(&mut conn, "1", None).await.unwrap();
    assert_eq!(ranks(bare), [30, 20]);
    let on_en = scores::history(&mut conn, "1", Some(Server::EN))
        .await
        .unwrap();
    assert_eq!(ranks(on_en), [10]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_last_calculation_time_is_per_server_or_overall(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    assert_eq!(
        scores::last_calculated_at(&mut conn, None).await.unwrap(),
        None
    );
    let en = common::user_on(&mut conn, 1, Server::EN).await;
    let jp = common::user_on(&mut conn, 2, Server::JP).await;
    for user in [en, jp] {
        scores::upsert(&mut conn, user, &WRITE).await.unwrap();
    }
    sqlx::query(
        "UPDATE user_scores SET calculated_at = NOW() - interval '1 hour' WHERE user_id = $1",
    )
    .bind(en.0)
    .execute(&mut *conn)
    .await
    .unwrap();
    let at = |user: UserId| {
        sqlx::query_scalar::<_, chrono::DateTime<chrono::Utc>>(
            "SELECT calculated_at FROM user_scores WHERE user_id = $1",
        )
        .bind(user.0)
    };
    let en_at = at(en).fetch_one(&mut *conn).await.unwrap();
    let jp_at = at(jp).fetch_one(&mut *conn).await.unwrap();

    assert_eq!(
        scores::last_calculated_at(&mut conn, Some(Server::EN))
            .await
            .unwrap(),
        Some(en_at)
    );
    assert_eq!(
        scores::last_calculated_at(&mut conn, None).await.unwrap(),
        Some(jp_at)
    );
    assert_eq!(
        scores::last_calculated_at(&mut conn, Some(Server::KR))
            .await
            .unwrap(),
        None
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn server_shares_count_the_top_public_players_by_server(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    for (n, server, total) in [
        (1, Server::EN, 0.9),
        (2, Server::JP, 0.8),
        (3, Server::JP, 0.7),
        (4, Server::EN, 0.6),
        (5, Server::EN, 0.5),
    ] {
        let user = common::user_on(&mut conn, n, server).await;
        public(&mut conn, user).await;
        scores::upsert(
            &mut conn,
            user,
            &ScoreWrite {
                total_score: total,
                ..WRITE
            },
        )
        .await
        .unwrap();
    }
    let hidden = common::user_on(&mut conn, 6, Server::KR).await;
    scores::upsert(
        &mut conn,
        hidden,
        &ScoreWrite {
            total_score: 1.0,
            ..WRITE
        },
    )
    .await
    .unwrap();

    let top3: Vec<(String, i64)> = scores::server_shares(&mut conn, 3)
        .await
        .unwrap()
        .into_iter()
        .map(|s| (s.server, s.players))
        .collect();
    assert_eq!(top3, [("jp".to_owned(), 2), ("en".to_owned(), 1)]);
    let all = scores::server_shares(&mut conn, 10).await.unwrap();
    assert_eq!((all[0].server.as_str(), all[0].players), ("en", 3));
    assert_eq!(all.len(), 2);
}
