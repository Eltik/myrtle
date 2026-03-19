pub mod migrations;
pub mod models;
pub mod pool;
pub mod queries;

pub use migrations::run_migrations;
pub use pool::create_pool;
use sqlx::PgPool;

/// Initialize database: create pool, run migrations, return pool
pub async fn init(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = create_pool(database_url).await?;
    run_migrations(&pool).await?;
    Ok(pool)
}
