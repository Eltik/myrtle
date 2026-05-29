use std::str::FromStr;

use serenity::all::{GuildId, RoleId};
use serenity::model::id::{ChannelId, MessageId};
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
        sqlx::query_as("SELECT auto_role_id FROM guild_auto_role WHERE guild_id = ?")
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
        "INSERT INTO guild_auto_role (guild_id, auto_role_id) VALUES (?, ?) \
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
        "INSERT INTO guild_auto_role (guild_id, auto_role_id) VALUES (?, NULL) \
         ON CONFLICT(guild_id) DO UPDATE SET auto_role_id = NULL",
    )
    .bind(guild_id.get().cast_signed())
    .execute(pool)
    .await?;
    Ok(())
}

pub struct ReactionRoleRow {
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub emoji: String,
    pub role_id: RoleId,
}

/// Insert or replace the role assigned when `emoji` is reacted on `message`.
///
/// Re-running with the same `(message, emoji)` swaps the role rather than erroring, so admins
/// can rebind an emoji without first removing the old mapping.
pub async fn add_reaction_role(
    pool: &SqlitePool,
    guild_id: GuildId,
    channel_id: ChannelId,
    message_id: MessageId,
    emoji: &str,
    role_id: RoleId,
) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO guild_reaction_roles (guild_id, channel_id, message_id, emoji, role_id) \
         VALUES (?, ?, ?, ?, ?) \
         ON CONFLICT(message_id, emoji) DO UPDATE SET role_id = excluded.role_id",
    )
    .bind(guild_id.get().cast_signed())
    .bind(channel_id.get().cast_signed())
    .bind(message_id.get().cast_signed())
    .bind(emoji)
    .bind(role_id.get().cast_signed())
    .execute(pool)
    .await?;
    Ok(())
}

/// Delete a single `(message, emoji)` mapping. Returns whether a row was removed.
pub async fn remove_reaction_role(
    pool: &SqlitePool,
    message_id: MessageId,
    emoji: &str,
) -> Result<bool, Error> {
    let result = sqlx::query("DELETE FROM guild_reaction_roles WHERE message_id = ? AND emoji = ?")
        .bind(message_id.get().cast_signed())
        .bind(emoji)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Delete every mapping attached to `message_id`. Used for `/reactionrole delete <message>`
/// and as the response to a `MessageDelete` event for a tracked message.
pub async fn remove_reaction_message(
    pool: &SqlitePool,
    message_id: MessageId,
) -> Result<u64, Error> {
    let result = sqlx::query("DELETE FROM guild_reaction_roles WHERE message_id = ?")
        .bind(message_id.get().cast_signed())
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Delete every mapping owned by `guild_id`. Used when the bot is kicked from a guild
/// (`GuildDelete`)
pub async fn remove_reaction_roles_for_guild(
    pool: &SqlitePool,
    guild_id: GuildId,
) -> Result<u64, Error> {
    let result = sqlx::query("DELETE FROM guild_reaction_roles WHERE guild_id = ?")
        .bind(guild_id.get().cast_signed())
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Resolve a single `(message, emoji)` to the role it grants, if any.
pub async fn get_role_for_reaction(
    pool: &SqlitePool,
    message_id: MessageId,
    emoji: &str,
) -> Result<Option<RoleId>, Error> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT role_id FROM guild_reaction_roles WHERE message_id = ? AND emoji = ?",
    )
    .bind(message_id.get().cast_signed())
    .bind(emoji)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(id,)| RoleId::new(id.cast_unsigned())))
}

/// All distinct tracked message IDs. Used to hydrate the in-memory `tracked_messages` cache
/// on startup so the reaction handler can short-circuit untracked messages without a DB hit.
pub async fn list_tracked_message_ids(pool: &SqlitePool) -> Result<Vec<MessageId>, Error> {
    let rows: Vec<(i64,)> = sqlx::query_as("SELECT DISTINCT message_id FROM guild_reaction_roles")
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|(id,)| MessageId::new(id.cast_unsigned()))
        .collect())
}

/// Set the asset-announcement channel for `guild_id`.
pub async fn set_assets_channel(
    pool: &SqlitePool,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO guild_asset_channel (guild_id, channel_id) VALUES (?, ?) \
         ON CONFLICT(guild_id) DO UPDATE SET channel_id = excluded.channel_id",
    )
    .bind(guild_id.get().cast_signed())
    .bind(channel_id.get().cast_signed())
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove the asset-announcement binding for `guild_id`.
pub async fn clear_assets_channel(pool: &SqlitePool, guild_id: GuildId) -> Result<u64, Error> {
    let result = sqlx::query("DELETE FROM guild_asset_channel WHERE guild_id = ?")
        .bind(guild_id.get().cast_signed())
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Look up the asset-announcement channel for `guild_id`, if any.
pub async fn get_assets_channel(
    pool: &SqlitePool,
    guild_id: GuildId,
) -> Result<Option<ChannelId>, Error> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT channel_id FROM guild_asset_channel WHERE guild_id = ?")
            .bind(guild_id.get().cast_signed())
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(id,)| ChannelId::new(id.cast_unsigned())))
}

/// Every configured `(guild, channel)` pair for asset announcements. Used by the watcher
/// to fan an event out to all subscribers in one pass.
pub async fn list_assets_channels(pool: &SqlitePool) -> Result<Vec<(GuildId, ChannelId)>, Error> {
    let rows: Vec<(i64, i64)> =
        sqlx::query_as("SELECT guild_id, channel_id FROM guild_asset_channel")
            .fetch_all(pool)
            .await?;
    Ok(rows
        .into_iter()
        .map(|(g, c)| {
            (
                GuildId::new(g.cast_unsigned()),
                ChannelId::new(c.cast_unsigned()),
            )
        })
        .collect())
}

/// All mappings in `guild_id`, ordered by message then emoji for stable `/reactionrole list` output.
pub async fn list_reaction_roles_for_guild(
    pool: &SqlitePool,
    guild_id: GuildId,
) -> Result<Vec<ReactionRoleRow>, Error> {
    let rows: Vec<(i64, i64, i64, String, i64)> = sqlx::query_as(
        "SELECT guild_id, channel_id, message_id, emoji, role_id \
         FROM guild_reaction_roles WHERE guild_id = ? ORDER BY message_id, emoji",
    )
    .bind(guild_id.get().cast_signed())
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(g, c, m, e, r)| ReactionRoleRow {
            guild_id: GuildId::new(g.cast_unsigned()),
            channel_id: ChannelId::new(c.cast_unsigned()),
            message_id: MessageId::new(m.cast_unsigned()),
            emoji: e,
            role_id: RoleId::new(r.cast_unsigned()),
        })
        .collect())
}
