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

/// Set the audit-log channel for `guild_id`, inserting the row if it doesn't exist.
pub async fn set_audit_log_channel(
    pool: &SqlitePool,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO guild_audit_log (guild_id, channel_id) VALUES (?, ?) \
         ON CONFLICT(guild_id) DO UPDATE SET channel_id = excluded.channel_id",
    )
    .bind(guild_id.get().cast_signed())
    .bind(channel_id.get().cast_signed())
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove the audit-log binding for `guild_id`.
pub async fn clear_audit_log_channel(pool: &SqlitePool, guild_id: GuildId) -> Result<u64, Error> {
    let result = sqlx::query("DELETE FROM guild_audit_log WHERE guild_id = ?")
        .bind(guild_id.get().cast_signed())
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Look up the audit-log channel for `guild_id`, if any.
pub async fn get_audit_log_channel(
    pool: &SqlitePool,
    guild_id: GuildId,
) -> Result<Option<ChannelId>, Error> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT channel_id FROM guild_audit_log WHERE guild_id = ?")
            .bind(guild_id.get().cast_signed())
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(id,)| ChannelId::new(id.cast_unsigned())))
}

/// Every configured `(guild, channel)` pair for audit logs. Used to hydrate the in-memory
/// cache at startup so the hot path (every logged event) skips a DB roundtrip.
pub async fn list_audit_log_channels(
    pool: &SqlitePool,
) -> Result<Vec<(GuildId, ChannelId)>, Error> {
    let rows: Vec<(i64, i64)> = sqlx::query_as("SELECT guild_id, channel_id FROM guild_audit_log")
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

/// Moderation action taken when a user trips an antispam check.
///
/// Lives here (not in `cmds/admin.rs`) so the event handler can also pattern-match on it
/// without taking a dependency on the command module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, poise::ChoiceParameter)]
pub enum AntiSpamAction {
    #[name = "Delete message"]
    Delete,
    #[name = "Warn user"]
    Warn,
    #[name = "Timeout user"]
    Timeout,
    #[name = "Kick user"]
    Kick,
    #[name = "Ban user"]
    Ban,
}

impl AntiSpamAction {
    /// Serialized form stored in `guild_max_ping.action`
    #[must_use]
    pub const fn as_db_str(self) -> &'static str {
        match self {
            Self::Delete => "delete",
            Self::Warn => "warn",
            Self::Timeout => "timeout",
            Self::Kick => "kick",
            Self::Ban => "ban",
        }
    }
    fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "delete" => Some(Self::Delete),
            "warn" => Some(Self::Warn),
            "timeout" => Some(Self::Timeout),
            "kick" => Some(Self::Kick),
            "ban" => Some(Self::Ban),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AntiSpamPolicy {
    pub max_per_message: u32,
    /// Rolling-window length; `None` disables the window check.
    pub window_secs: Option<u32>,
    /// Max pings allowed inside the rolling window; paired with `window_secs`.
    pub window_max_pings: Option<u32>,
    pub action: AntiSpamAction,
    /// Used only when `action == Timeout`.
    pub timeout_secs: Option<u32>,
    /// Members holding this role bypass antispam entirely.
    pub exempt_role_id: Option<RoleId>,
}

type AntiSpamRow = (
    i64,
    Option<i64>,
    Option<i64>,
    String,
    Option<i64>,
    Option<i64>,
);

fn row_to_policy(guild_id: GuildId, row: AntiSpamRow) -> Result<AntiSpamPolicy, Error> {
    let (max_per_message, window_secs, window_max_pings, action_str, timeout_secs, exempt_role_id) =
        row;
    let action = AntiSpamAction::from_db_str(&action_str)
        .ok_or_else(|| format!("Invalid action '{action_str}' for guild {guild_id}"))?;
    Ok(AntiSpamPolicy {
        max_per_message: max_per_message.try_into().unwrap_or(u32::MAX),
        window_secs: window_secs.map(|v| v.try_into().unwrap_or(u32::MAX)),
        window_max_pings: window_max_pings.map(|v| v.try_into().unwrap_or(u32::MAX)),
        action,
        timeout_secs: timeout_secs.map(|v| v.try_into().unwrap_or(u32::MAX)),
        exempt_role_id: exempt_role_id.map(|id| RoleId::new(id.cast_unsigned())),
    })
}

/// Insert or replace the antispam policy for `guild_id`.
pub async fn set_antispam_policy(
    pool: &SqlitePool,
    guild_id: GuildId,
    policy: &AntiSpamPolicy,
) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO guild_max_ping (guild_id, max_ping_per_message, window_secs, window_max_pings, action, timeout_secs, exempt_role_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(guild_id) DO UPDATE SET \
            max_ping_per_message = excluded.max_ping_per_message, \
            window_secs = excluded.window_secs, \
            window_max_pings = excluded.window_max_pings, \
            action = excluded.action, \
            timeout_secs = excluded.timeout_secs, \
            exempt_role_id = excluded.exempt_role_id",
    )
    .bind(guild_id.get().cast_signed())
    .bind(i64::from(policy.max_per_message))
    .bind(policy.window_secs.map(i64::from))
    .bind(policy.window_max_pings.map(i64::from))
    .bind(policy.action.as_db_str())
    .bind(policy.timeout_secs.map(i64::from))
    .bind(policy.exempt_role_id.map(|r| r.get().cast_signed()))
    .execute(pool)
    .await?;
    Ok(())
}

/// Drop the antispam policy for `guild_id`. Returns the number of rows removed.
pub async fn clear_antispam_policy(pool: &SqlitePool, guild_id: GuildId) -> Result<u64, Error> {
    let res = sqlx::query("DELETE FROM guild_max_ping WHERE guild_id = ?")
        .bind(guild_id.get().cast_signed())
        .execute(pool)
        .await?;
    Ok(res.rows_affected())
}

/// Look up the antispam policy for `guild_id`, if one is configured.
pub async fn get_antispam_policy(
    pool: &SqlitePool,
    guild_id: GuildId,
) -> Result<Option<AntiSpamPolicy>, Error> {
    let row: Option<AntiSpamRow> = sqlx::query_as(
        "SELECT max_ping_per_message, window_secs, window_max_pings, action, timeout_secs, exempt_role_id \
         FROM guild_max_ping WHERE guild_id = ?",
    )
    .bind(guild_id.get().cast_signed())
    .fetch_optional(pool)
    .await?;
    row.map(|r| row_to_policy(guild_id, r)).transpose()
}

/// Every configured antispam policy, used to hydrate the in-memory cache on startup.
pub async fn list_antispam_policies(
    pool: &SqlitePool,
) -> Result<Vec<(GuildId, AntiSpamPolicy)>, Error> {
    // sqlx::FromRow doesn't auto-flatten a `(i64, AntiSpamRow)` nesting, so the row tuple
    // has to spell out every column. The complexity warning is unavoidable here.
    #[allow(clippy::type_complexity)]
    let rows: Vec<(i64, i64, Option<i64>, Option<i64>, String, Option<i64>, Option<i64>)> =
        sqlx::query_as(
            "SELECT guild_id, max_ping_per_message, window_secs, window_max_pings, action, timeout_secs, exempt_role_id \
             FROM guild_max_ping",
        )
        .fetch_all(pool)
        .await?;
    rows.into_iter()
        .map(|(g, max, ws, wmax, act, ts, exempt)| {
            let guild = GuildId::new(g.cast_unsigned());
            row_to_policy(guild, (max, ws, wmax, act, ts, exempt)).map(|p| (guild, p))
        })
        .collect()
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
