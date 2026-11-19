#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::repos::data_generations::{self, Generation};
use myrtle_types::Server;
use sqlx::PgPool;

fn at(generation: i64, res_version: Option<&str>) -> Generation {
    Generation {
        generation,
        res_version: res_version.map(str::to_owned),
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
        at(1, None)
    );
    assert_eq!(
        data_generations::bump(&mut conn, Server::EN, None)
            .await
            .unwrap(),
        at(2, None)
    );
    assert_eq!(
        data_generations::bump(&mut conn, Server::KR, None)
            .await
            .unwrap(),
        at(1, None)
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
    assert_eq!(bump(Some("v1")).await, at(1, Some("v1")));
    assert_eq!(bump(None).await, at(2, Some("v1")), "a sidecar bump");
    assert_eq!(bump(Some("v2")).await, at(3, Some("v2")));
    assert_eq!(
        data_generations::all(&mut conn).await.unwrap(),
        [(Server::CN, at(3, Some("v2")))]
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
