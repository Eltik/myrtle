pub mod migrations;
pub mod models;
pub mod pool;
pub mod queries;

pub use migrations::run_migrations;
pub use pool::create_pool;
use sqlx::PgPool;

/// Initialize database: create pool, run migrations, seed data, return pool
pub async fn init(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = create_pool(database_url).await?;
    run_migrations(&pool).await?;
    seed(&pool).await?;
    Ok(pool)
}

/// Seed reference data (idempotent — safe to call on every startup).
/// Derives server list from the `Server` enum in core::hypergryph::constants.
async fn seed(pool: &PgPool) -> Result<(), sqlx::Error> {
    use crate::core::hypergryph::constants::Server;

    for &server in Server::all() {
        sqlx::query(
            "INSERT INTO servers (id, code, name) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING",
        )
        .bind(server.index() as i16)
        .bind(server.as_str())
        .bind(server.display_name())
        .execute(pool)
        .await?;
    }

    Ok(())
}
