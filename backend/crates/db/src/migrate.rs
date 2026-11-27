use std::collections::HashMap;

use myrtle_types::Server;
use sqlx::migrate::{MigrateError, Migrator};

use crate::{Conn, Db, DbError};

pub static MIGRATOR: Migrator = sqlx::migrate!();

const BASELINE: i64 = 1;
pub const LEGACY_HEAD: &str = "v046_oauth";

pub async fn run(db: &Db) -> Result<(), DbError> {
    let mut conn = db.conn().await?;
    let migrated = migrate(&mut conn).await;
    conn.close().await;
    migrated?;
    seed(&mut *db.conn().await?).await
}

async fn migrate(conn: &mut Conn) -> Result<(), DbError> {
    sqlx::query("SET statement_timeout = 0")
        .execute(&mut *conn)
        .await?;
    sqlx::query("SET lock_timeout = 0")
        .execute(&mut *conn)
        .await?;
    adopt(conn).await?;
    Ok(MIGRATOR.run_direct(None, conn, false).await?)
}

pub async fn check(db: &Db) -> Result<(), DbError> {
    let mut conn = db.conn().await?;
    let applied = applied(&mut conn).await?;
    for m in MIGRATOR
        .iter()
        .filter(|m| !m.migration_type.is_down_migration())
    {
        match applied.get(&m.version) {
            None => return Err(DbError::SchemaBehind { version: m.version }),
            Some(checksum) if checksum[..] != m.checksum[..] => {
                return Err(MigrateError::VersionMismatch(m.version).into());
            }
            Some(_) => {}
        }
    }
    Ok(())
}

pub async fn seed(conn: &mut Conn) -> Result<(), DbError> {
    let ids: Vec<i16> = Server::ALL.iter().map(|s| s.index()).collect();
    let codes: Vec<String> = Server::ALL.iter().map(|s| s.as_str().to_owned()).collect();
    let names: Vec<String> = Server::ALL.iter().map(|s| name(*s).to_owned()).collect();
    sqlx::query!(
        "INSERT INTO servers (id, code, name)
         SELECT * FROM UNNEST($1::smallint[], $2::text[], $3::text[])
         ON CONFLICT (id) DO NOTHING",
        &ids,
        &codes,
        &names,
    )
    .execute(conn)
    .await?;
    Ok(())
}

const fn name(server: Server) -> &'static str {
    match server {
        Server::EN => "English",
        Server::JP => "Japanese",
        Server::KR => "Korean",
        Server::CN => "Chinese",
        Server::Bilibili => "Bilibili",
        Server::TW => "Taiwanese",
    }
}

async fn adopt(conn: &mut Conn) -> Result<(), DbError> {
    if !exists(conn, "public._migrations").await? || applied(conn).await?.contains_key(&BASELINE) {
        return Ok(());
    }
    let at_head: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM public._migrations WHERE name = $1)")
            .bind(LEGACY_HEAD)
            .fetch_one(&mut *conn)
            .await?;
    if !at_head {
        return Err(DbError::LegacyBehind {
            expected: LEGACY_HEAD,
        });
    }
    MIGRATOR.run_direct(Some(BASELINE), conn, true).await?;
    tracing::info!(
        legacy_head = LEGACY_HEAD,
        "adopted the existing schema as the baseline"
    );
    Ok(())
}

async fn applied(conn: &mut Conn) -> Result<HashMap<i64, Vec<u8>>, DbError> {
    if !exists(conn, "public._sqlx_migrations").await? {
        return Ok(HashMap::new());
    }
    let rows: Vec<(i64, Vec<u8>)> =
        sqlx::query_as("SELECT version, checksum FROM public._sqlx_migrations WHERE success")
            .fetch_all(conn)
            .await?;
    Ok(rows.into_iter().collect())
}

async fn exists(conn: &mut Conn, table: &str) -> Result<bool, DbError> {
    Ok(sqlx::query_scalar("SELECT to_regclass($1) IS NOT NULL")
        .bind(table)
        .fetch_one(conn)
        .await?)
}
