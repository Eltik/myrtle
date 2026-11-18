#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::Conn;
use myrtle_db::repos::user_search::{self, Direction, OwnsAll, Rank, Search, SearchRow};
use myrtle_types::{Server, UserId};
use serde_json::{Value, json};
use sqlx::PgPool;

/// A public profile with uid `n`, an optional score and layout
async fn player(conn: &mut Conn, n: u32, score: Option<f64>, layout: Option<Value>) -> UserId {
    player_on(conn, n, Server::EN, score, layout).await
}

async fn player_on(
    conn: &mut Conn,
    n: u32,
    server: Server,
    score: Option<f64>,
    layout: Option<Value>,
) -> UserId {
    let user = common::user_on(conn, n, server).await;
    sqlx::query("INSERT INTO user_settings (user_id, profile_layout) VALUES ($1, $2)")
        .bind(user.0)
        .bind(layout)
        .execute(&mut *conn)
        .await
        .unwrap();
    if let Some(score) = score {
        sqlx::query("INSERT INTO user_scores (user_id, total_score) VALUES ($1, $2)")
            .bind(user.0)
            .bind(score)
            .execute(&mut *conn)
            .await
            .unwrap();
    }
    user
}

async fn own(conn: &mut Conn, user: UserId, operators: &[(&str, i16)]) {
    for (id, potential) in operators {
        sqlx::query(
            "INSERT INTO user_operators (user_id, operator_id, potential) VALUES ($1, $2, $3)",
        )
        .bind(user.0)
        .bind(*id)
        .bind(*potential)
        .execute(&mut *conn)
        .await
        .unwrap();
    }
}

async fn run(conn: &mut Conn, sql: &'static str, user: UserId) {
    sqlx::query(sql).bind(user.0).execute(conn).await.unwrap();
}

const fn by(rank: Rank<'_>, direction: Direction) -> Search<'_> {
    Search {
        q: None,
        rank,
        direction,
        has: &[],
        support: None,
        owns_all: None,
    }
}

async fn ranked(conn: &mut Conn, search: &Search<'_>) -> Vec<(String, Option<i64>)> {
    let rows = user_search::page(conn, search, 100, 0).await.unwrap();
    let total = user_search::count(conn, search).await.unwrap();
    assert_eq!(total, rows.len() as i64, "count agrees with the page");
    rows.into_iter()
        .map(|r: SearchRow| (r.profile.uid, r.metric))
        .collect()
}

fn uids(rows: &[(String, Option<i64>)]) -> Vec<&str> {
    rows.iter().map(|(uid, _)| uid.as_str()).collect()
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_score_order_ranks_public_profiles_with_a_private_score_tab_last(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    player(&mut conn, 1, Some(10.0), None).await;
    player(&mut conn, 2, Some(30.0), None).await;
    player(&mut conn, 3, Some(20.0), None).await;
    player(
        &mut conn,
        4,
        Some(100.0),
        Some(json!({"tabs": [{"id": "score", "visible": false}]})),
    )
    .await;
    player(&mut conn, 5, None, None).await;
    let private = player(&mut conn, 6, Some(999.0), None).await;
    sqlx::query("UPDATE user_settings SET public_profile = false WHERE user_id = $1")
        .bind(private.0)
        .execute(&mut *conn)
        .await
        .unwrap();
    common::user(&mut conn, 7).await;

    let desc = ranked(&mut conn, &by(Rank::Score, Direction::Desc)).await;
    assert_eq!(uids(&desc), ["2", "3", "1", "4", "5"]);
    assert!(
        desc.iter().all(|(_, m)| m.is_none()),
        "score rows carry no metric"
    );
    let row = &user_search::page(&mut conn, &by(Rank::Score, Direction::Desc), 1, 0)
        .await
        .unwrap()[0];
    assert_eq!(row.profile.total_score, Some(30.0));

    let asc = ranked(&mut conn, &by(Rank::Score, Direction::Asc)).await;
    assert_eq!(uids(&asc), ["1", "3", "2", "4", "5"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn each_roster_rank_carries_its_aggregate_and_a_missing_one_reads_zero(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let a = player(&mut conn, 1, Some(1.0), None).await;
    let b = player(&mut conn, 2, Some(2.0), None).await;
    player(&mut conn, 3, Some(3.0), None).await;

    own(&mut conn, a, &[("char_a", 5), ("char_b", 0), ("char_c", 2)]).await;
    own(&mut conn, b, &[("char_a", 1)]).await;
    run(
        &mut conn,
        "INSERT INTO user_operator_skills (user_id, operator_id, skill_index, specialize_level)
         VALUES ($1, 'char_a', 0, 3), ($1, 'char_a', 1, 3), ($1, 'char_b', 0, 2)",
        a,
    )
    .await;
    run(
        &mut conn,
        "INSERT INTO user_operator_modules (user_id, operator_id, module_id, module_level, locked)
         VALUES ($1, 'char_a', 'uniequip_x', 3, false), ($1, 'char_a', 'uniequip_y', 3, true),
                ($1, 'char_a', 'uniequip_z', 2, false)",
        b,
    )
    .await;
    run(
        &mut conn,
        "INSERT INTO user_skins (user_id, skin_id) VALUES ($1, 's1'), ($1, 's2')",
        b,
    )
    .await;
    run(
        &mut conn,
        r#"INSERT INTO user_enemy_progress (user_id, enemies) VALUES ($1, '{"e1": 1, "e2": 1, "e3": 0}')"#,
        a,
    )
    .await;

    let scope = vec!["char_b".to_owned(), "char_c".to_owned()];
    for (rank, expected) in [
        (Rank::Operators, [("1", 3), ("2", 1), ("3", 0)]),
        (Rank::Potentials, [("1", 7), ("2", 1), ("3", 0)]),
        (Rank::Masteries, [("1", 2), ("3", 0), ("2", 0)]),
        (Rank::Modules, [("2", 1), ("3", 0), ("1", 0)]),
        (Rank::Skins, [("2", 2), ("3", 0), ("1", 0)]),
        (Rank::Enemies, [("1", 3), ("3", 0), ("2", 0)]),
        (Rank::OwnedOf(&scope), [("1", 2), ("3", 0), ("2", 0)]),
    ] {
        let got = ranked(&mut conn, &by(rank, Direction::Desc)).await;
        let expected: Vec<(String, Option<i64>)> = expected
            .iter()
            .map(|(uid, v)| ((*uid).to_owned(), Some(*v)))
            .collect();
        assert_eq!(got, expected, "{rank:?}, ties by score");
    }

    let asc = ranked(&mut conn, &by(Rank::Operators, Direction::Asc)).await;
    assert_eq!(uids(&asc), ["3", "2", "1"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_join_date_rank_reads_register_ts_and_skips_a_hidden_date(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let old = player(&mut conn, 1, None, None).await;
    let new = player(&mut conn, 2, None, None).await;
    player(&mut conn, 3, None, None).await;
    let hidden = player(
        &mut conn,
        4,
        None,
        Some(json!({"header": {"hidden": ["registered"]}})),
    )
    .await;
    for (user, ts) in [(old, 100_i64), (new, 200), (hidden, 50)] {
        sqlx::query("INSERT INTO user_status (user_id, register_ts) VALUES ($1, $2)")
            .bind(user.0)
            .bind(ts)
            .execute(&mut *conn)
            .await
            .unwrap();
    }

    let asc = ranked(&mut conn, &by(Rank::Joined, Direction::Asc)).await;
    let expected = [("1", Some(100)), ("2", Some(200)), ("3", None)];
    let expected: Vec<(String, Option<i64>)> = expected
        .iter()
        .map(|(uid, v)| ((*uid).to_owned(), *v))
        .collect();
    assert_eq!(asc, expected, "a missing date sorts last");

    let desc = ranked(&mut conn, &by(Rank::Joined, Direction::Desc)).await;
    assert_eq!(uids(&desc), ["2", "1", "3"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_hidden_tab_or_header_number_drops_the_player_from_what_reads_it(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let open = player(&mut conn, 1, Some(1.0), None).await;
    let no_roster = player(
        &mut conn,
        2,
        Some(2.0),
        Some(json!({"tabs": [{"id": "roster", "visible": false}]})),
    )
    .await;
    let no_enemies = player(
        &mut conn,
        3,
        Some(3.0),
        Some(json!({"tabs": [{"id": "enemies", "visible": false}]})),
    )
    .await;
    let no_count = player(
        &mut conn,
        4,
        Some(4.0),
        Some(json!({"header": {"hidden": ["operators"]}})),
    )
    .await;
    let no_skins = player(
        &mut conn,
        5,
        Some(5.0),
        Some(json!({"header": {"hidden": ["skins"]}})),
    )
    .await;
    for user in [open, no_roster, no_enemies, no_count, no_skins] {
        own(&mut conn, user, &[("char_a", 0)]).await;
    }

    let rank = |rank| by(rank, Direction::Desc);
    assert_eq!(
        uids(&ranked(&mut conn, &rank(Rank::Operators)).await),
        ["5", "3", "1"]
    );
    assert_eq!(
        uids(&ranked(&mut conn, &rank(Rank::Skins)).await),
        ["4", "3", "1"]
    );
    assert_eq!(
        uids(&ranked(&mut conn, &rank(Rank::Masteries)).await),
        ["5", "4", "3", "1"]
    );
    assert_eq!(
        uids(&ranked(&mut conn, &rank(Rank::Enemies)).await),
        ["5", "4", "2", "1"]
    );
    assert_eq!(
        uids(&ranked(&mut conn, &rank(Rank::Score)).await),
        ["5", "4", "3", "2", "1"],
        "the score order reads no tab"
    );

    let has = vec!["char_a".to_owned()];
    let filtered = Search {
        has: &has,
        ..rank(Rank::Score)
    };
    assert_eq!(
        uids(&ranked(&mut conn, &filtered).await),
        ["5", "4", "3", "1"],
        "an ownership filter reads the roster"
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_filters_admit_only_rows_that_satisfy_them(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let a = player(&mut conn, 1, Some(1.0), None).await;
    let b = player(&mut conn, 2, Some(2.0), None).await;
    let c = player_on(&mut conn, 3, Server::JP, Some(3.0), None).await;
    for (user, nickname) in [(a, "Amiya_fan"), (b, "AmiyaXfan"), (c, "kalts")] {
        sqlx::query("UPDATE users SET nickname = $2 WHERE id = $1")
            .bind(user.0)
            .bind(nickname)
            .execute(&mut *conn)
            .await
            .unwrap();
    }
    own(&mut conn, a, &[("char_a", 0), ("char_b", 0)]).await;
    own(&mut conn, b, &[("char_a", 0)]).await;
    own(&mut conn, c, &[("char_a", 0), ("char_b", 0)]).await;
    run(
        &mut conn,
        "INSERT INTO user_support_units (user_id, slot, operator_id) VALUES ($1, 0, 'char_b')",
        c,
    )
    .await;
    let base = by(Rank::Score, Direction::Desc);

    let q = Search {
        q: Some("amiya_"),
        ..base
    };
    assert_eq!(uids(&ranked(&mut conn, &q).await), ["1"], "`_` is literal");

    let both = vec!["char_a".to_owned(), "char_b".to_owned()];
    let has = Search { has: &both, ..base };
    assert_eq!(uids(&ranked(&mut conn, &has).await), ["3", "1"]);
    let repeated = vec![
        "char_a".to_owned(),
        "char_b".to_owned(),
        "char_a".to_owned(),
    ];
    let has = Search {
        has: &repeated,
        ..base
    };
    assert_eq!(
        uids(&ranked(&mut conn, &has).await),
        ["3", "1"],
        "a repeated id counts once"
    );

    let support = Search {
        support: Some("char_b"),
        ..base
    };
    assert_eq!(uids(&ranked(&mut conn, &support).await), ["3"]);

    let required = vec![("en".to_owned(), 2_i64), ("jp".to_owned(), 1)];
    let all = Search {
        owns_all: Some(OwnsAll {
            ids: &both,
            required: &required,
        }),
        ..base
    };
    assert_eq!(uids(&ranked(&mut conn, &all).await), ["3", "1"]);

    let en_only = vec![("en".to_owned(), 1_i64)];
    let unjudged = Search {
        owns_all: Some(OwnsAll {
            ids: &both,
            required: &en_only,
        }),
        ..base
    };
    assert_eq!(
        uids(&ranked(&mut conn, &unjudged).await),
        ["2", "1"],
        "a server missing from `required` drops out"
    );

    let empty: Vec<String> = Vec::new();
    let zero = vec![("en".to_owned(), 0_i64), ("jp".to_owned(), 0)];
    let vacuous = Search {
        owns_all: Some(OwnsAll {
            ids: &empty,
            required: &zero,
        }),
        ..base
    };
    assert!(
        ranked(&mut conn, &vacuous).await.is_empty(),
        "an empty scope matches nobody"
    );

    let combined = Search {
        q: Some("amiya"),
        rank: Rank::Operators,
        direction: Direction::Desc,
        has: &both,
        support: None,
        owns_all: Some(OwnsAll {
            ids: &both,
            required: &required,
        }),
    };
    assert_eq!(
        ranked(&mut conn, &combined).await,
        [("1".to_owned(), Some(2))]
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn pages_split_the_order_without_overlap(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    for n in 1..=5 {
        player(&mut conn, n, Some(1.0), None).await;
    }
    let search = by(Rank::Score, Direction::Desc);

    let mut seen = Vec::new();
    for offset in [0, 2, 4] {
        let page = user_search::page(&mut conn, &search, 2, offset)
            .await
            .unwrap();
        seen.extend(page.into_iter().map(|r| r.profile.uid));
    }
    assert_eq!(seen, ["1", "2", "3", "4", "5"], "equal scores split by uid");
    assert_eq!(user_search::count(&mut conn, &search).await.unwrap(), 5);
}
