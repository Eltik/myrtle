//! Fresh databases, databases tracked by `_migrations`, and the error classes.

#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::migrate::{self, LEGACY_HEAD, MIGRATOR};
use myrtle_db::{Db, DbError};
use sqlx::PgPool;

const BASELINE_SQL: &str = include_str!("../migrations/0001_baseline.sql");

async fn count(db: &Db, sql: &'static str) -> i64 {
    sqlx::query_scalar(sql)
        .fetch_one(&mut *db.conn().await.unwrap())
        .await
        .unwrap()
}

#[sqlx::test(migrations = false)]
async fn a_fresh_database_gets_the_schema_and_the_servers(pool: PgPool) {
    let db = Db::from_pool(pool);
    assert!(matches!(
        migrate::check(&db).await,
        Err(DbError::SchemaBehind { version: 1 })
    ));

    migrate::run(&db).await.unwrap();
    migrate::check(&db).await.unwrap();
    assert_eq!(count(&db, "SELECT count(*) FROM servers").await, 6);
    assert_eq!(count(&db, "SELECT count(*) FROM _migrations").await, 44);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM locales WHERE code = 'en' AND is_source AND enabled"
        )
        .await,
        1
    );
    assert_eq!(count(&db, "SELECT count(*) FROM locales").await, 1);
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM information_schema.tables
             WHERE table_schema = 'public' AND table_type = 'BASE TABLE'"
        )
        .await,
        76
    );

    migrate::run(&db).await.unwrap();
    assert_eq!(count(&db, "SELECT count(*) FROM servers").await, 6);
}

#[sqlx::test(migrations = false)]
async fn the_migrator_leaves_no_session_settings_behind(pool: PgPool) {
    let db = Db::from_pool(pool);
    let mut conn = db.conn().await.unwrap();
    MIGRATOR.run_direct(None, &mut *conn, false).await.unwrap();
    let path: String = sqlx::query_scalar("SHOW search_path")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    assert_eq!(path, "\"$user\", public");
    let bodies: String = sqlx::query_scalar("SHOW check_function_bodies")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    assert_eq!(bodies, "on");
}

#[sqlx::test(migrations = false)]
async fn a_database_at_the_legacy_head_is_adopted_not_rebuilt(pool: PgPool) {
    sqlx::raw_sql(BASELINE_SQL).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO servers (id, code, name) VALUES (0, 'en', 'kept')")
        .execute(&pool)
        .await
        .unwrap();
    let db = Db::from_pool(pool);

    migrate::run(&db).await.unwrap();
    migrate::check(&db).await.unwrap();

    let recorded: Vec<u8> =
        sqlx::query_scalar("SELECT checksum FROM _sqlx_migrations WHERE version = 1")
            .fetch_one(&mut *db.conn().await.unwrap())
            .await
            .unwrap();
    let baseline = MIGRATOR.iter().find(|m| m.version == 1).unwrap();
    assert_eq!(recorded, baseline.checksum.as_ref());
    assert_eq!(
        count(&db, "SELECT count(*) FROM servers WHERE name = 'kept'").await,
        1
    );
    assert_eq!(count(&db, "SELECT count(*) FROM servers").await, 6);

    migrate::run(&db).await.unwrap();
}

#[sqlx::test(migrations = false)]
async fn a_database_behind_the_legacy_head_is_refused_untouched(pool: PgPool) {
    sqlx::raw_sql(
        "CREATE TABLE _migrations (name VARCHAR(100) PRIMARY KEY, applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW());
         INSERT INTO _migrations (name) VALUES ('v045_sandbox_keep_unloaded_topics');",
    )
    .execute(&pool)
    .await
    .unwrap();
    let db = Db::from_pool(pool);

    let refused = migrate::run(&db).await;
    assert!(matches!(refused, Err(DbError::LegacyBehind { expected }) if expected == LEGACY_HEAD));
    assert_eq!(
        count(
            &db,
            "SELECT count(*)::bigint FROM pg_tables WHERE schemaname = 'public'"
        )
        .await,
        1
    );
}

#[sqlx::test(migrations = false)]
async fn a_baseline_applied_from_other_sql_fails_and_releases_the_lock(pool: PgPool) {
    let db = Db::from_pool(pool);
    migrate::run(&db).await.unwrap();
    sqlx::query("UPDATE _sqlx_migrations SET checksum = '\\x00' WHERE version = 1")
        .execute(&mut *db.conn().await.unwrap())
        .await
        .unwrap();

    let mismatch = |r: Result<(), DbError>| {
        matches!(
            r,
            Err(DbError::Migrate(
                sqlx::migrate::MigrateError::VersionMismatch(1)
            ))
        )
    };
    assert!(mismatch(migrate::check(&db).await));
    assert!(mismatch(migrate::run(&db).await));
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM pg_locks WHERE locktype = 'advisory'
             AND database = (SELECT oid FROM pg_database WHERE datname = current_database())"
        )
        .await,
        0
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn database_errors_are_classified(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let run = async |conn: &mut myrtle_db::Conn, sql: &'static str| -> DbError {
        sqlx::query(sql).fetch_one(conn).await.unwrap_err().into()
    };

    assert!(matches!(
        run(&mut conn, "SELECT 1 WHERE false").await,
        DbError::NotFound
    ));
    assert!(matches!(
        run(&mut conn, "INSERT INTO servers (id, code, name) VALUES (0, 'zz', 'x') RETURNING id").await,
        DbError::Conflict { constraint } if constraint == "servers_pkey"
    ));
    assert!(matches!(
        run(&mut conn, "INSERT INTO users (uid, server_id) VALUES ('1', 99) RETURNING id").await,
        DbError::ForeignKey { constraint } if constraint == "users_server_id_fkey"
    ));
    assert!(matches!(
        run(
            &mut conn,
            "INSERT INTO servers (id, code, name) VALUES (9, 'toolong', 'x') RETURNING id"
        )
        .await,
        DbError::TooLong
    ));
    assert!(matches!(
        run(&mut conn, "SELECT 1/0").await,
        DbError::Other(_)
    ));
}
