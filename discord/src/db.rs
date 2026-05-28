use std::str::FromStr;

use serenity::all::{GuildId, RoleId};
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;

use crate::types::Error;

pub const DEFAULT_DATABASE_URL: &str = "sqlite:database.sqlite";

/// Open the `SQLite` pool at `url` and apply embedded migrations.
///
/// Creates the database file if it doesn't exist so first-run setups don't need a
/// separate `sqlx database create` step.
pub async fn init_pool(url: &str) -> Result<SqlitePool, Error> {
    let options = SqliteConnectOptions::from_str(url)
        .map_err(|e| format!("Invalid DATABASE_URL '{url}': {e}"))?
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(options)
        .await
        .map_err(|e| format!("Failed to connect to {url}: {e}"))?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| format!("Failed to run migrations: {e}"))?;
    Ok(pool)
}

/// Look up the configured auto-role for `guild_id`, if any.
pub async fn get_auto_role(pool: &SqlitePool, guild_id: GuildId) -> Result<Option<RoleId>, Error> {
    let row: Option<(Option<i64>,)> =
        sqlx::query_as("SELECT auto_role_id FROM guild_settings WHERE guild_id = ?")
            .bind(guild_id.get().cast_signed())
            .fetch_optional(pool)
            .await?;
    Ok(row
        .and_then(|(id,)| id)
        .map(|id| RoleId::new(id.cast_unsigned())))
}

/// Set the auto-role for `guild_id`, inserting the row if it doesn't exist.
pub async fn set_auto_role(
    pool: &SqlitePool,
    guild_id: GuildId,
    role_id: RoleId,
) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO guild_settings (guild_id, auto_role_id) VALUES (?, ?) \
         ON CONFLICT(guild_id) DO UPDATE SET auto_role_id = excluded.auto_role_id",
    )
    .bind(guild_id.get().cast_signed())
    .bind(role_id.get().cast_signed())
    .execute(pool)
    .await?;
    Ok(())
}

/// Clear the auto-role for `guild_id`, leaving the row in place with a NULL value.
pub async fn clear_auto_role(pool: &SqlitePool, guild_id: GuildId) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO guild_settings (guild_id, auto_role_id) VALUES (?, NULL) \
         ON CONFLICT(guild_id) DO UPDATE SET auto_role_id = NULL",
    )
    .bind(guild_id.get().cast_signed())
    .execute(pool)
    .await?;
    Ok(())
}
