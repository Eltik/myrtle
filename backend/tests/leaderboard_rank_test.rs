//! The score leaderboard ranks an ungraded public player after every graded
//! one, on a database built by the real migrations.
//!
//! Each run creates its own empty database on the server `DATABASE_URL` points
//! at (default: the local Postgres), applies every migration in order, seeds
//! three public players, checks the view, the page query, a player's standing
//! and the snapshot job, then drops the database. Nothing touches the database
//! named in the URL beyond `CREATE DATABASE` / `DROP DATABASE`, so the exact
//! ranks hold whatever data the local database holds. Needs a role that may
//! create databases, so it is ignored by default:
//! `cargo test --test leaderboard_rank_test -- --ignored`

use std::panic::AssertUnwindSafe;
use std::str::FromStr;

use backend::database::queries::score::{
    get_leaderboard, get_player_standing, take_leaderboard_snapshot,
};
use backend::database::run_migrations;
use futures_util::FutureExt;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Connection, PgConnection, PgPool};
use uuid::Uuid;

fn base_options() -> PgConnectOptions {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    PgConnectOptions::from_str(&url).expect("DATABASE_URL parses")
}

/// A fresh database with every migration applied, and its name for the drop.
async fn throwaway_database() -> (PgPool, String) {
    let name = format!(
        "myrtle_lbrank_{}",
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
    // `database::init` seeds the servers after migrating; the view joins them.
    sqlx::query("INSERT INTO servers (id, code, name) VALUES (0, 'en', 'Global')")
        .execute(&pool)
        .await
        .expect("seed server");
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

/// A public player on EN; `score` None means never graded (no `user_scores` row).
async fn player(pool: &PgPool, uid: &str, score: Option<f64>) -> Uuid {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (uid, server_id, nickname) VALUES ($1, 0, $1) RETURNING id",
    )
    .bind(uid)
    .fetch_one(pool)
    .await
    .expect("user");
    sqlx::query("INSERT INTO user_settings (user_id, public_profile) VALUES ($1, true)")
        .bind(id)
        .execute(pool)
        .await
        .expect("settings");
    if let Some(total) = score {
        sqlx::query("INSERT INTO user_scores (user_id, total_score) VALUES ($1, $2)")
            .bind(id)
            .bind(total)
            .execute(pool)
            .await
            .expect("score");
    }
    id
}

#[tokio::test]
#[ignore = "creates and drops a throwaway database on the local Postgres"]
async fn an_ungraded_player_ranks_after_every_graded_one() {
    let (pool, name) = throwaway_database().await;
    // Drop the database even when an assertion fails, then fail.
    let outcome = AssertUnwindSafe(ranks(&pool)).catch_unwind().await;
    drop_database(pool, &name).await;
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}

async fn ranks(pool: &PgPool) {
    let pool = pool.clone();

    player(&pool, "100000001", Some(900.0)).await;
    player(&pool, "100000002", Some(500.0)).await;
    player(&pool, "100000003", None).await;

    // The view itself: before v041 the ungraded player was 1 and the others 2, 3.
    let ranks: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT uid, rank_global, rank_server FROM v_leaderboard ORDER BY rank_global, uid",
    )
    .fetch_all(&pool)
    .await
    .expect("view");
    assert_eq!(
        ranks,
        vec![
            ("100000001".to_string(), 1, 1),
            ("100000002".to_string(), 2, 2),
            ("100000003".to_string(), 3, 3),
        ]
    );

    // The page the leaderboard route serves agrees.
    let page = get_leaderboard(&pool, None, "total_score", None, false, None, 50, 0)
        .await
        .expect("page");
    let page: Vec<(&str, Option<i64>)> = page
        .iter()
        .map(|e| (e.uid.as_str(), e.rank_global))
        .collect();
    assert_eq!(
        page,
        vec![
            ("100000001", Some(1)),
            ("100000002", Some(2)),
            ("100000003", Some(3))
        ]
    );

    // The top graded player's own standing is rank 1.
    let top = get_player_standing(&pool, "100000001", "en", 5, "1 day")
        .await
        .expect("standing")
        .expect("public graded player has a standing");
    assert_eq!(top.player.rank_global, Some(1));

    // The snapshot job copies the view's ranks into history.
    let snapshot = take_leaderboard_snapshot(&pool).await.expect("snapshot");
    let copied: Vec<(String, i32, Option<f64>)> = sqlx::query_as(
        r"
        SELECT u.uid, e.rank_global, e.total_score
        FROM leaderboard_snapshot_entries e JOIN users u ON u.id = e.user_id
        WHERE e.snapshot_id = $1 ORDER BY e.rank_global
        ",
    )
    .bind(snapshot)
    .fetch_all(&pool)
    .await
    .expect("snapshot entries");
    assert_eq!(
        copied,
        vec![
            ("100000001".to_string(), 1, Some(900.0)),
            ("100000002".to_string(), 2, Some(500.0)),
            ("100000003".to_string(), 3, None),
        ]
    );
}
