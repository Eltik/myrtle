//! The per-server known-stage set is read once and reused, and a sync's
//! `forget_known_stage_ids` makes the next read see what the sync brought.
//!
//! Each run creates its own empty database on the server `DATABASE_URL` points
//! at (default: the local Postgres), applies every migration in order, seeds
//! two players on EN and one on JP, checks the set per server, then drops the
//! database. Nothing touches the database named in the URL beyond
//! `CREATE DATABASE` / `DROP DATABASE`. Needs a role that may create
//! databases, so it is ignored by default:
//! `cargo test --test known_stages_cache_test -- --ignored`

use std::collections::HashSet;
use std::panic::AssertUnwindSafe;
use std::str::FromStr;

use backend::database::queries::stages::{forget_known_stage_ids, get_known_stage_ids_for_server};
use backend::database::run_migrations;
use futures_util::FutureExt;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Connection, PgConnection, PgPool};
use uuid::Uuid;

const EN: i16 = 0;
const JP: i16 = 1;

fn base_options() -> PgConnectOptions {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    PgConnectOptions::from_str(&url).expect("DATABASE_URL parses")
}

/// A fresh database with every migration applied, and its name for the drop.
async fn throwaway_database() -> (PgPool, String) {
    let name = format!(
        "myrtle_knownstages_{}",
        &Uuid::new_v4().simple().to_string()[..12]
    );
    let mut admin = PgConnection::connect_with(&base_options())
        .await
        .expect("connect to postgres");
    sqlx::raw_sql(&format!("CREATE DATABASE {name}"))
        .execute(&mut admin)
        .await
        .expect("create throwaway database");
    admin.close().await.ok();

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(base_options().database(&name))
        .await
        .expect("connect to throwaway database");
    run_migrations(&pool)
        .await
        .expect("every migration applies");
    // `database::init` seeds the servers after migrating; `users` references them.
    sqlx::query(
        "INSERT INTO servers (id, code, name) VALUES (0, 'en', 'Global'), (1, 'jp', 'Japan')",
    )
    .execute(&pool)
    .await
    .expect("seed servers");
    (pool, name)
}

async fn drop_database(pool: PgPool, name: &str) {
    pool.close().await;
    let mut admin = PgConnection::connect_with(&base_options())
        .await
        .expect("connect to postgres");
    sqlx::raw_sql(&format!("DROP DATABASE IF EXISTS {name} WITH (FORCE)"))
        .execute(&mut admin)
        .await
        .expect("drop throwaway database");
}

/// A player on `server` whose dungeon JSON holds `stages` (the value is what a
/// sync stores: presence of the key is all the set reads).
async fn player(pool: &PgPool, uid: &str, server: i16, stages: &[&str]) -> Uuid {
    let id: Uuid =
        sqlx::query_scalar("INSERT INTO users (uid, server_id) VALUES ($1, $2) RETURNING id")
            .bind(uid)
            .bind(server)
            .fetch_one(pool)
            .await
            .expect("insert user");
    write_stages(pool, id, stages).await;
    id
}

async fn write_stages(pool: &PgPool, user_id: Uuid, stages: &[&str]) {
    let json: serde_json::Map<String, serde_json::Value> = stages
        .iter()
        .map(|s| ((*s).to_owned(), serde_json::json!({ "state": 0 })))
        .collect();
    sqlx::query(
        "INSERT INTO user_stage_progress (user_id, stages) VALUES ($1, $2)
         ON CONFLICT (user_id) DO UPDATE SET stages = EXCLUDED.stages",
    )
    .bind(user_id)
    .bind(serde_json::Value::Object(json))
    .execute(pool)
    .await
    .expect("write stage progress");
}

fn set(ids: &[&str]) -> HashSet<String> {
    ids.iter().map(|s| (*s).to_owned()).collect()
}

async fn checks(pool: &PgPool) {
    // Entries another test in this process left behind would answer for this
    // database's servers.
    forget_known_stage_ids(EN);
    forget_known_stage_ids(JP);

    let ada = player(pool, "10000001", EN, &["main_00-01", "main_00-02"]).await;
    let bea = player(pool, "10000002", EN, &["main_00-02", "act1side_01"]).await;
    let cho = player(pool, "20000001", JP, &["main_00-01", "jp_only_01"]).await;

    // The union of everyone on the server, and only that server.
    let en = set(&["main_00-01", "main_00-02", "act1side_01"]);
    let jp = set(&["main_00-01", "jp_only_01"]);
    assert_eq!(
        *get_known_stage_ids_for_server(pool, ada).await.expect("en"),
        en
    );
    assert_eq!(
        *get_known_stage_ids_for_server(pool, bea).await.expect("en"),
        en
    );
    assert_eq!(
        *get_known_stage_ids_for_server(pool, cho).await.expect("jp"),
        jp
    );

    // A user that does not exist is on no server.
    let nobody = get_known_stage_ids_for_server(pool, Uuid::new_v4())
        .await
        .expect("no such user");
    assert!(nobody.is_empty());

    // New rows on EN are not seen while the set is cached...
    write_stages(pool, bea, &["main_00-02", "act1side_01", "act2side_01"]).await;
    assert_eq!(
        *get_known_stage_ids_for_server(pool, ada)
            .await
            .expect("en cached"),
        en,
        "the cached set is reused inside its TTL"
    );
    // ...until the sync that wrote them drops it, which is what `roster::refresh` does.
    forget_known_stage_ids(EN);
    let mut en_after = en.clone();
    en_after.insert("act2side_01".to_owned());
    assert_eq!(
        *get_known_stage_ids_for_server(pool, ada)
            .await
            .expect("en fresh"),
        en_after
    );
    // Dropping EN's set left JP's alone.
    assert_eq!(
        *get_known_stage_ids_for_server(pool, cho).await.expect("jp"),
        jp
    );

    forget_known_stage_ids(EN);
    forget_known_stage_ids(JP);
}

#[tokio::test]
#[ignore = "creates and drops a throwaway database on the local Postgres"]
async fn the_known_stage_set_is_cached_per_server_and_dropped_by_a_sync() {
    let (pool, name) = throwaway_database().await;
    let outcome = AssertUnwindSafe(checks(&pool)).catch_unwind().await;
    drop_database(pool, &name).await;
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}
