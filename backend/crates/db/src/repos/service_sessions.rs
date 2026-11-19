use myrtle_types::Server;

use crate::{Conn, DbError};

const GET: &str = "SELECT nonce, ciphertext FROM service_account_sessions WHERE server_id = $1";

const LOCK: &str =
    "SELECT nonce, ciphertext FROM service_account_sessions WHERE server_id = $1 FOR UPDATE";

const UPSERT: &str = "
INSERT INTO service_account_sessions (server_id, nonce, ciphertext)
VALUES ($1, $2, $3)
ON CONFLICT (server_id) DO UPDATE
   SET nonce = EXCLUDED.nonce,
       ciphertext = EXCLUDED.ciphertext,
       updated_at = now()";

const INSERT_NEW: &str = "
INSERT INTO service_account_sessions (server_id, nonce, ciphertext)
VALUES ($1, $2, $3)
ON CONFLICT (server_id) DO NOTHING";

const DELETE: &str = "DELETE FROM service_account_sessions WHERE server_id = $1";

#[derive(Debug, Clone)]
pub struct SealedSessionRow {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

async fn fetch(
    conn: &mut Conn,
    sql: &'static str,
    server: Server,
) -> Result<Option<SealedSessionRow>, DbError> {
    let row: Option<(Vec<u8>, Vec<u8>)> = sqlx::query_as(sql)
        .bind(server.index())
        .fetch_optional(conn)
        .await?;
    Ok(row.map(|(nonce, ciphertext)| SealedSessionRow { nonce, ciphertext }))
}

pub async fn get(conn: &mut Conn, server: Server) -> Result<Option<SealedSessionRow>, DbError> {
    fetch(conn, GET, server).await
}

pub async fn lock(conn: &mut Conn, server: Server) -> Result<Option<SealedSessionRow>, DbError> {
    fetch(conn, LOCK, server).await
}

pub async fn upsert(
    conn: &mut Conn,
    server: Server,
    nonce: &[u8],
    ciphertext: &[u8],
) -> Result<(), DbError> {
    sqlx::query(UPSERT)
        .bind(server.index())
        .bind(nonce)
        .bind(ciphertext)
        .execute(conn)
        .await?;
    Ok(())
}

/// False when the server already has a row
pub async fn insert_new(
    conn: &mut Conn,
    server: Server,
    nonce: &[u8],
    ciphertext: &[u8],
) -> Result<bool, DbError> {
    let result = sqlx::query(INSERT_NEW)
        .bind(server.index())
        .bind(nonce)
        .bind(ciphertext)
        .execute(conn)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete(conn: &mut Conn, server: Server) -> Result<bool, DbError> {
    let result = sqlx::query(DELETE)
        .bind(server.index())
        .execute(conn)
        .await?;
    Ok(result.rows_affected() > 0)
}
