#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::repos::data_generations::{self, Bump, Generation};
use myrtle_types::Server;
use sqlx::PgPool;

fn at(generation: i64, res_version: Option<&str>) -> Generation {
    at_seq(generation, res_version, 0)
}

fn published(generation: i64, res_version: Option<&str>, pipeline_seq: i64) -> Bump {
    Bump::Published(at_seq(generation, res_version, pipeline_seq))
}

fn at_seq(generation: i64, res_version: Option<&str>, pipeline_seq: i64) -> Generation {
    Generation {
        generation,
        res_version: res_version.map(str::to_owned),
        pipeline_seq,
    }
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_bump_counts_up_from_zero_per_server(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    assert!(data_generations::all(&mut conn).await.unwrap().is_empty());

    assert_eq!(
        data_generations::bump(&mut conn, Server::EN, None)
            .await
            .unwrap(),
        published(1, None, 0)
    );
    assert_eq!(
        data_generations::bump(&mut conn, Server::EN, None)
            .await
            .unwrap(),
        published(2, None, 0)
    );
    assert_eq!(
        data_generations::bump(&mut conn, Server::KR, None)
            .await
            .unwrap(),
        published(1, None, 0)
    );

    let mut all = data_generations::all(&mut conn).await.unwrap();
    all.sort_by_key(|(server, _)| server.index());
    let mut expected = vec![(Server::EN, at(2, None)), (Server::KR, at(1, None))];
    expected.sort_by_key(|(server, _)| server.index());
    assert_eq!(all, expected);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_bump_without_a_version_keeps_the_last_pipeline_version(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let mut bump = async |version| {
        data_generations::bump(&mut conn, Server::CN, version)
            .await
            .unwrap()
    };
    assert_eq!(bump(Some("v1")).await, published(1, Some("v1"), 1));
    assert_eq!(
        bump(None).await,
        published(2, Some("v1"), 1),
        "a sidecar bump"
    );
    assert_eq!(bump(Some("v2")).await, published(3, Some("v2"), 2));
    assert_eq!(
        data_generations::all(&mut conn).await.unwrap(),
        [(Server::CN, at_seq(3, Some("v2"), 2))]
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn concurrent_bumps_never_hand_out_one_generation_twice(pool: PgPool) {
    let db = common::db(pool).await;
    let bumps: Vec<_> = (0..8)
        .map(|_| {
            let db = db.clone();
            tokio::spawn(async move {
                let mut conn = db.conn().await.unwrap();
                data_generations::bump(&mut conn, Server::CN, None)
                    .await
                    .unwrap()
                    .current()
                    .generation
            })
        })
        .collect();
    let mut seen = Vec::new();
    for bump in bumps {
        seen.push(bump.await.unwrap());
    }
    seen.sort_unstable();
    assert_eq!(seen, (1..=8).collect::<Vec<i64>>());
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_bump_moves_updated_at(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    data_generations::bump(&mut conn, Server::JP, None)
        .await
        .unwrap();
    sqlx::query("UPDATE data_generations SET updated_at = now() - interval '1 day'")
        .execute(&mut *conn)
        .await
        .unwrap();
    data_generations::bump(&mut conn, Server::JP, None)
        .await
        .unwrap();
    let fresh: bool = sqlx::query_scalar(
        "SELECT updated_at > now() - interval '1 minute' FROM data_generations WHERE server_id = $1",
    )
    .bind(Server::JP.index())
    .fetch_one(&mut *conn)
    .await
    .unwrap();
    assert!(fresh);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_ledger_cursor_starts_at_zero_and_serialises_its_recorders(pool: PgPool) {
    let db = common::db(pool).await;
    let mut first = db.begin().await.unwrap();
    assert_eq!(
        data_generations::lock_recorded(&mut first, Server::EN)
            .await
            .unwrap(),
        0,
        "a missing row is created at zero"
    );
    let second = tokio::spawn({
        let db = db.clone();
        async move {
            let mut tx = db.begin().await.unwrap();
            let seen = data_generations::lock_recorded(&mut tx, Server::EN)
                .await
                .unwrap();
            tx.commit().await.unwrap();
            seen
        }
    });
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    assert!(
        !second.is_finished(),
        "the second recorder waits on the lock"
    );
    data_generations::set_recorded(&mut first, Server::EN, 3)
        .await
        .unwrap();
    first.commit().await.unwrap();
    assert_eq!(second.await.unwrap(), 3);

    let mut conn = db.conn().await.unwrap();
    assert_eq!(
        data_generations::bump(&mut conn, Server::EN, Some("v1"))
            .await
            .unwrap(),
        published(1, Some("v1"), 1),
        "the cursor's row bumps like any other"
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_version_already_held_is_not_published_again(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let mut bump = async |version| {
        data_generations::bump(&mut conn, Server::EN, version)
            .await
            .unwrap()
    };
    assert_eq!(bump(Some("v1")).await, published(1, Some("v1"), 1));
    assert_eq!(
        bump(Some("v1")).await,
        Bump::AlreadyPublished(at_seq(1, Some("v1"), 1)),
        "the same version again"
    );
    assert_eq!(
        bump(None).await,
        published(2, Some("v1"), 1),
        "no version always goes through"
    );
    assert_eq!(
        bump(Some("v1")).await,
        Bump::AlreadyPublished(at_seq(2, Some("v1"), 1)),
        "a versionless bump keeps the held version"
    );
    assert_eq!(bump(Some("v2")).await, published(3, Some("v2"), 2));
    assert_eq!(
        data_generations::all(&mut conn).await.unwrap(),
        [(Server::EN, at_seq(3, Some("v2"), 2))]
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_cursor_row_without_a_version_takes_the_first_one(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    data_generations::lock_recorded(&mut conn, Server::JP)
        .await
        .unwrap();
    assert_eq!(
        data_generations::bump(&mut conn, Server::JP, Some("v1"))
            .await
            .unwrap(),
        published(1, Some("v1"), 1)
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn concurrent_bumps_of_one_version_publish_it_once(pool: PgPool) {
    let db = common::db(pool).await;
    let bumps: Vec<_> = (0..8)
        .map(|_| {
            let db = db.clone();
            tokio::spawn(async move {
                let mut tx = db.begin().await.unwrap();
                let bumped = data_generations::bump(&mut tx, Server::CN, Some("v1"))
                    .await
                    .unwrap();
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                tx.commit().await.unwrap();
                bumped
            })
        })
        .collect();
    let mut published = 0;
    for bump in bumps {
        let bumped = bump.await.unwrap();
        assert_eq!(bumped.current(), &at_seq(1, Some("v1"), 1));
        published += usize::from(matches!(bumped, Bump::Published(_)));
    }
    assert_eq!(published, 1);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_bump_that_rolls_back_leaves_the_version_to_the_next(pool: PgPool) {
    let db = common::db(pool).await;
    let mut first = db.begin().await.unwrap();
    let abandoned = data_generations::bump(&mut first, Server::CN, Some("v1"))
        .await
        .unwrap();
    assert_eq!(abandoned, published(1, Some("v1"), 1));

    let waiting = {
        let db = db.clone();
        tokio::spawn(async move {
            let mut tx = db.begin().await.unwrap();
            let bumped = data_generations::bump(&mut tx, Server::CN, Some("v1"))
                .await
                .unwrap();
            tx.commit().await.unwrap();
            bumped
        })
    };
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert!(!waiting.is_finished(), "the second bump waits on the first");
    first.rollback().await.unwrap();

    assert_eq!(waiting.await.unwrap(), published(1, Some("v1"), 1));
}
