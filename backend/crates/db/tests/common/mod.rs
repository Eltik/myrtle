//! A seeded `Db` over the database `#[sqlx::test]` hands each test, and row fixtures.

#![allow(dead_code, clippy::unwrap_used)]

use myrtle_db::{Conn, Db, migrate};
use myrtle_types::{Server, UserId};
use sqlx::PgPool;

pub async fn db(pool: PgPool) -> Db {
    let db = Db::from_pool(pool);
    migrate::seed(&mut db.conn().await.unwrap()).await.unwrap();
    db
}

/// A `users` row on EN with uid `n`
pub async fn user(conn: &mut Conn, n: u32) -> UserId {
    user_on(conn, n, Server::EN).await
}

pub async fn user_on(conn: &mut Conn, n: u32, server: Server) -> UserId {
    let id = sqlx::query_scalar(
        "INSERT INTO users (uid, server_id, nickname) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(n.to_string())
    .bind(server.index())
    .bind(format!("user{n}"))
    .fetch_one(conn)
    .await
    .unwrap();
    UserId(id)
}
