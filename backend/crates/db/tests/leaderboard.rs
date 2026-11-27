#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::Conn;
use myrtle_db::repos::leaderboard::{self, LeaderboardSort, Movement, MoverDirection};
use myrtle_types::{Server, UserId};
use sqlx::PgPool;

/// A public player; `total` None means never graded
async fn player(conn: &mut Conn, n: u32, server: Server, total: Option<f64>) -> UserId {
    let user = common::user_on(conn, n, server).await;
    sqlx::query("INSERT INTO user_settings (user_id, public_profile) VALUES ($1, true)")
        .bind(user.0)
        .execute(&mut *conn)
        .await
        .unwrap();
    if let Some(total) = total {
        sqlx::query("INSERT INTO user_scores (user_id, total_score) VALUES ($1, $2)")
            .bind(user.0)
            .bind(total)
            .execute(&mut *conn)
            .await
            .unwrap();
    }
    user
}

/// A snapshot taken `days_ago`, each entry `(user, rank_global, total_score)` on EN
async fn snapshot(conn: &mut Conn, days_ago: i32, entries: &[(UserId, i32, f64)]) {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO leaderboard_snapshots (taken_at)
         VALUES (NOW() - make_interval(days => $1)) RETURNING id",
    )
    .bind(days_ago)
    .fetch_one(&mut *conn)
    .await
    .unwrap();
    for (user, rank, total) in entries {
        sqlx::query(
            "INSERT INTO leaderboard_snapshot_entries
                 (snapshot_id, user_id, server_id, rank_global, rank_server, total_score)
             VALUES ($1, $2, 0, $3, $3, $4)",
        )
        .bind(id)
        .bind(user.0)
        .bind(rank)
        .bind(total)
        .execute(&mut *conn)
        .await
        .unwrap();
    }
}

fn uids(rows: &[leaderboard::LeaderboardRow]) -> Vec<&str> {
    rows.iter().map(|r| r.uid.as_str()).collect()
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_ungraded_player_ranks_after_every_graded_one(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    player(&mut conn, 100_000_003, Server::EN, None).await;
    player(&mut conn, 100_000_001, Server::EN, Some(900.0)).await;
    player(&mut conn, 100_000_002, Server::EN, Some(500.0)).await;

    let page = leaderboard::page(&mut conn, LeaderboardSort::Total, None, None, None, 50, 0)
        .await
        .unwrap();
    let ranks: Vec<(&str, Option<i64>)> = page
        .iter()
        .map(|e| (e.uid.as_str(), e.rank_global))
        .collect();
    assert_eq!(
        ranks,
        [
            ("100000001", Some(1)),
            ("100000002", Some(2)),
            ("100000003", Some(3))
        ]
    );
    assert!(page.iter().all(|e| e.rank_delta.is_none()));
    assert_eq!(page[2].total_score, None);

    let top = leaderboard::standing(&mut conn, "100000001", Server::EN, 5, "1 day")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(top.player.rank_global, Some(1));

    let id = leaderboard::take_snapshot(&mut conn).await.unwrap();
    let copied: Vec<(String, i32, Option<f64>)> = sqlx::query_as(
        "SELECT u.uid, e.rank_global, e.total_score
         FROM leaderboard_snapshot_entries e JOIN users u ON u.id = e.user_id
         WHERE e.snapshot_id = $1 ORDER BY e.rank_global",
    )
    .bind(id)
    .fetch_all(&mut *conn)
    .await
    .unwrap();
    assert_eq!(
        copied,
        [
            ("100000001".to_owned(), 1, Some(900.0)),
            ("100000002".to_owned(), 2, Some(500.0)),
            ("100000003".to_owned(), 3, None),
        ]
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn each_sort_orders_by_its_own_column_with_unscored_players_last(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    player(&mut conn, 3, Server::EN, None).await;
    player(&mut conn, 1, Server::EN, Some(0.9)).await;
    let b = player(&mut conn, 2, Server::EN, Some(0.5)).await;
    let columns = [
        LeaderboardSort::Operator,
        LeaderboardSort::Stage,
        LeaderboardSort::Roguelike,
        LeaderboardSort::Sandbox,
        LeaderboardSort::Medal,
        LeaderboardSort::Base,
        LeaderboardSort::Skin,
    ];
    for sort in columns {
        let raise = format!("UPDATE user_scores SET {} = 0.1", sort.column());
        sqlx::query(sqlx::AssertSqlSafe(raise))
            .execute(&mut *conn)
            .await
            .unwrap();
        let bump = format!(
            "UPDATE user_scores SET {} = 0.8 WHERE user_id = $1",
            sort.column()
        );
        sqlx::query(sqlx::AssertSqlSafe(bump))
            .bind(b.0)
            .execute(&mut *conn)
            .await
            .unwrap();

        let page = leaderboard::page(&mut conn, sort, None, None, None, 50, 0)
            .await
            .unwrap();
        assert_eq!(uids(&page), ["2", "1", "3"], "{sort:?}");

        let moved = Movement {
            interval: "1 day",
            moved_only: false,
        };
        let page = leaderboard::page(&mut conn, sort, None, None, Some(moved), 50, 0)
            .await
            .unwrap();
        assert_eq!(uids(&page), ["2", "1", "3"], "{sort:?} with movement");

        let reset = format!("UPDATE user_scores SET {} = 0", sort.column());
        sqlx::query(sqlx::AssertSqlSafe(reset))
            .execute(&mut *conn)
            .await
            .unwrap();
    }
    let page = leaderboard::page(&mut conn, LeaderboardSort::Total, None, None, None, 50, 0)
        .await
        .unwrap();
    assert_eq!(uids(&page), ["1", "2", "3"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_page_filters_by_server_and_literal_search_text_and_pages(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    for (n, nick, server, total) in [
        (1, "a_b", Server::EN, 0.9),
        (2, "axb", Server::EN, 0.8),
        (3, "a_b2", Server::JP, 0.7),
        (4, "zzz", Server::EN, 0.6),
    ] {
        let user = player(&mut conn, n, server, Some(total)).await;
        sqlx::query("UPDATE users SET nickname = $2 WHERE id = $1")
            .bind(user.0)
            .bind(nick)
            .execute(&mut *conn)
            .await
            .unwrap();
    }
    let private = common::user(&mut conn, 5).await;
    sqlx::query("INSERT INTO user_settings (user_id, public_profile) VALUES ($1, false)")
        .bind(private.0)
        .execute(&mut *conn)
        .await
        .unwrap();

    let sort = LeaderboardSort::Total;
    let en = leaderboard::page(&mut conn, sort, Some(Server::EN), None, None, 50, 0)
        .await
        .unwrap();
    assert_eq!(uids(&en), ["1", "2", "4"]);
    assert_eq!(en[2].rank_server, Some(3));
    assert_eq!(en[2].rank_global, Some(4));

    let found = leaderboard::page(&mut conn, sort, None, Some("a_b"), None, 50, 0)
        .await
        .unwrap();
    assert_eq!(uids(&found), ["1", "3"]);
    let found = leaderboard::page(&mut conn, sort, Some(Server::JP), Some("A_B"), None, 50, 0)
        .await
        .unwrap();
    assert_eq!(uids(&found), ["3"]);

    let second = leaderboard::page(&mut conn, sort, None, None, None, 2, 2)
        .await
        .unwrap();
    assert_eq!(uids(&second), ["3", "4"]);

    assert_eq!(
        leaderboard::count(&mut conn, None, None, None)
            .await
            .unwrap(),
        4
    );
    assert_eq!(
        leaderboard::count(&mut conn, Some(Server::EN), Some("a_b"), None)
            .await
            .unwrap(),
        1
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn movement_compares_against_the_latest_snapshot_old_enough(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let a = player(&mut conn, 1, Server::EN, Some(900.0)).await;
    let b = player(&mut conn, 2, Server::EN, Some(500.0)).await;
    player(&mut conn, 3, Server::JP, Some(100.0)).await;
    snapshot(&mut conn, 30, &[(a, 3, 100.0), (b, 3, 100.0)]).await;
    snapshot(&mut conn, 8, &[(a, 2, 800.0), (b, 1, 950.0)]).await;
    snapshot(&mut conn, 1, &[(a, 3, 0.0), (b, 3, 0.0)]).await;

    let sort = LeaderboardSort::Total;
    let all = Movement {
        interval: "7 days",
        moved_only: false,
    };
    let page = leaderboard::page(&mut conn, sort, None, None, Some(all), 50, 0)
        .await
        .unwrap();
    let deltas: Vec<(&str, Option<i64>)> = page
        .iter()
        .map(|r| (r.uid.as_str(), r.rank_delta))
        .collect();
    assert_eq!(deltas, [("1", Some(1)), ("2", Some(-1)), ("3", None)]);
    assert_eq!(
        leaderboard::count(&mut conn, None, None, Some(all))
            .await
            .unwrap(),
        3
    );

    let moved = Movement {
        interval: "7 days",
        moved_only: true,
    };
    let page = leaderboard::page(&mut conn, sort, None, None, Some(moved), 50, 0)
        .await
        .unwrap();
    assert_eq!(uids(&page), ["1", "2"]);
    assert_eq!(
        leaderboard::count(&mut conn, None, None, Some(moved))
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        leaderboard::count(&mut conn, Some(Server::JP), None, Some(moved))
            .await
            .unwrap(),
        0
    );

    let up = leaderboard::top_movers(&mut conn, "7 days", MoverDirection::Up, None, 10)
        .await
        .unwrap();
    let moves: Vec<(&str, i64, i64, i64, Option<f64>)> = up
        .iter()
        .map(|m| {
            let uid = m.uid.as_str();
            (
                uid,
                m.current_rank,
                m.previous_rank,
                m.rank_delta,
                m.score_delta,
            )
        })
        .collect();
    assert_eq!(
        moves,
        [("1", 1, 2, 1, Some(100.0)), ("2", 2, 1, -1, Some(-450.0))]
    );
    let down = leaderboard::top_movers(&mut conn, "7 days", MoverDirection::Down, None, 1)
        .await
        .unwrap();
    assert_eq!(down.len(), 1);
    assert_eq!(down[0].uid, "2");
    assert!(
        leaderboard::top_movers(
            &mut conn,
            "7 days",
            MoverDirection::Up,
            Some(Server::JP),
            10
        )
        .await
        .unwrap()
        .is_empty()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn movement_reads_one_baseline_snapshot_so_a_player_absent_from_it_has_no_delta(
    pool: PgPool,
) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let a = player(&mut conn, 1, Server::EN, Some(900.0)).await;
    let b = player(&mut conn, 2, Server::EN, Some(500.0)).await;
    snapshot(&mut conn, 30, &[(a, 5, 100.0), (b, 1, 950.0)]).await;
    snapshot(&mut conn, 8, &[(a, 2, 800.0)]).await;

    let all = Movement {
        interval: "7 days",
        moved_only: false,
    };
    let page = leaderboard::page(
        &mut conn,
        LeaderboardSort::Total,
        None,
        None,
        Some(all),
        50,
        0,
    )
    .await
    .unwrap();
    let deltas: Vec<(&str, Option<i64>)> = page
        .iter()
        .map(|r| (r.uid.as_str(), r.rank_delta))
        .collect();
    assert_eq!(deltas, [("1", Some(1)), ("2", None)]);

    let moved = Movement {
        interval: "7 days",
        moved_only: true,
    };
    let page = leaderboard::page(
        &mut conn,
        LeaderboardSort::Total,
        None,
        None,
        Some(moved),
        50,
        0,
    )
    .await
    .unwrap();
    assert_eq!(uids(&page), ["1"]);
    assert_eq!(
        leaderboard::count(&mut conn, None, None, Some(moved))
            .await
            .unwrap(),
        1
    );
    for direction in [MoverDirection::Up, MoverDirection::Down] {
        let movers = leaderboard::top_movers(&mut conn, "7 days", direction, None, 10)
            .await
            .unwrap();
        let movers: Vec<&str> = movers.iter().map(|m| m.uid.as_str()).collect();
        assert_eq!(movers, ["1"]);
    }
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_standing_carries_the_rank_window_and_the_rank_an_interval_ago(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let mut users = Vec::new();
    for n in 1..=5 {
        users.push(player(&mut conn, n, Server::EN, Some(f64::from(10 - n))).await);
    }
    snapshot(&mut conn, 2, &[(users[2], 5, 0.0)]).await;
    snapshot(&mut conn, 10, &[(users[2], 1, 0.0)]).await;

    let standing = leaderboard::standing(&mut conn, "3", Server::EN, 1, "1 day")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(standing.player.rank_global, Some(3));
    assert_eq!(uids(&standing.neighbours), ["2", "4"]);
    assert_eq!(standing.ranked, 5);
    assert_eq!(standing.previous_rank, Some(5));

    let edge = leaderboard::standing(&mut conn, "1", Server::EN, 2, "7 days")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(uids(&edge.neighbours), ["2", "3"]);
    assert_eq!(edge.previous_rank, None);

    assert!(
        leaderboard::standing(&mut conn, "3", Server::JP, 1, "1 day")
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_negative_window_returns_the_player_without_neighbours(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    for n in 1..=3 {
        player(&mut conn, n, Server::EN, Some(f64::from(10 - n))).await;
    }

    let standing = leaderboard::standing(&mut conn, "2", Server::EN, -1, "1 day")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(standing.player.uid, "2");
    assert_eq!(standing.player.rank_global, Some(2));
    assert!(standing.neighbours.is_empty());
    assert_eq!(standing.ranked, 3);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_standings_window_keeps_the_players_uid_twin_on_another_server(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    player(&mut conn, 1, Server::EN, Some(0.9)).await;
    player(&mut conn, 2, Server::EN, Some(0.8)).await;
    player(&mut conn, 2, Server::JP, Some(0.7)).await;

    let standing = leaderboard::standing(&mut conn, "2", Server::EN, 1, "1 day")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(standing.player.rank_global, Some(2));
    let window: Vec<(&str, &str)> = standing
        .neighbours
        .iter()
        .map(|r| (r.uid.as_str(), r.server.as_str()))
        .collect();
    assert_eq!(window, [("1", "en"), ("2", "jp")]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_count_with_movement_matches_the_page_with_the_same_filters(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let a = player(&mut conn, 1, Server::EN, Some(900.0)).await;
    player(&mut conn, 2, Server::EN, None).await;
    player(&mut conn, 3, Server::JP, Some(100.0)).await;
    snapshot(&mut conn, 8, &[(a, 2, 800.0)]).await;

    for moved_only in [false, true] {
        let movement = Movement {
            interval: "7 days",
            moved_only,
        };
        for (server, q) in [(None, None), (Some(Server::EN), None), (None, Some("user"))] {
            let page = leaderboard::page(
                &mut conn,
                LeaderboardSort::Total,
                server,
                q,
                Some(movement),
                50,
                0,
            )
            .await
            .unwrap();
            let count = leaderboard::count(&mut conn, server, q, Some(movement))
                .await
                .unwrap();
            assert_eq!(
                usize::try_from(count).unwrap(),
                page.len(),
                "{moved_only} {server:?} {q:?}"
            );
        }
    }
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_latest_snapshot_time_is_none_until_one_is_taken(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    assert_eq!(
        leaderboard::latest_snapshot_at(&mut conn).await.unwrap(),
        None
    );
    snapshot(&mut conn, 3, &[]).await;
    let id = leaderboard::take_snapshot(&mut conn).await.unwrap();
    let taken: chrono::DateTime<chrono::Utc> =
        sqlx::query_scalar("SELECT taken_at FROM leaderboard_snapshots WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *conn)
            .await
            .unwrap();
    assert_eq!(
        leaderboard::latest_snapshot_at(&mut conn).await.unwrap(),
        Some(taken)
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn tied_players_come_back_in_id_order_so_pages_never_overlap(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let mut tied = Vec::new();
    for n in 1..=6 {
        let user = player(&mut conn, n, Server::EN, Some(500.0)).await;
        tied.push((user, n.to_string()));
    }
    let baseline: Vec<_> = tied.iter().map(|(user, _)| (*user, 3, 500.0)).collect();
    snapshot(&mut conn, 8, &baseline).await;
    tied.sort_by_key(|(user, _)| user.0);
    let by_id: Vec<&str> = tied.iter().map(|(_, uid)| uid.as_str()).collect();

    let movement = Movement {
        interval: "7 days",
        moved_only: true,
    };
    for movement in [None, Some(movement)] {
        let mut paged = Vec::new();
        for offset in [0, 2, 4] {
            let page = leaderboard::page(
                &mut conn,
                LeaderboardSort::Total,
                None,
                None,
                movement,
                2,
                offset,
            )
            .await
            .unwrap();
            paged.extend(page.into_iter().map(|r| r.uid));
        }
        assert_eq!(paged, by_id);
    }

    let movers = leaderboard::top_movers(&mut conn, "7 days", MoverDirection::Up, None, 10)
        .await
        .unwrap();
    let movers: Vec<&str> = movers.iter().map(|m| m.uid.as_str()).collect();
    assert_eq!(movers, by_id);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_standing_reads_the_same_baseline_snapshot_as_the_page(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let a = player(&mut conn, 1, Server::EN, Some(900.0)).await;
    let b = player(&mut conn, 2, Server::EN, Some(500.0)).await;
    snapshot(&mut conn, 30, &[(a, 5, 100.0), (b, 1, 950.0)]).await;
    snapshot(&mut conn, 8, &[(a, 2, 800.0)]).await;

    let present = leaderboard::standing(&mut conn, "1", Server::EN, 1, "7 days")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(present.previous_rank, Some(2));
    let absent = leaderboard::standing(&mut conn, "2", Server::EN, 1, "7 days")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(absent.previous_rank, None);
}
