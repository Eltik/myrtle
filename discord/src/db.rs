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
///
/// Note this is the raw binding: it says nothing about which categories the guild has switched
/// off. Anything deciding whether to *log* something must go through `audit::audit_channel`,
/// which consults the cached `AuditEventFilter` as well.
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

/// Every configured audit-log binding. Used to hydrate the in-memory cache at startup so the
/// hot path (every logged event) skips a DB roundtrip.
pub async fn list_audit_log_settings(
    pool: &SqlitePool,
) -> Result<Vec<(GuildId, AuditSettings)>, Error> {
    let rows: Vec<(i64, i64, i64)> =
        sqlx::query_as("SELECT guild_id, channel_id, disabled_events FROM guild_audit_log")
            .fetch_all(pool)
            .await?;
    Ok(rows
        .into_iter()
        .map(|(g, c, disabled)| {
            (
                GuildId::new(g.cast_unsigned()),
                AuditSettings {
                    channel_id: ChannelId::new(c.cast_unsigned()),
                    events: AuditEventFilter::from_disabled_mask(mask_from_db(disabled)),
                },
            )
        })
        .collect())
}

/// Persist `filter` for `guild_id`. Returns `false` when the guild has no audit-log binding
/// yet - the filter hangs off that row, so there is nowhere to store it until `/auditlog set`
/// has run.
pub async fn set_audit_log_events(
    pool: &SqlitePool,
    guild_id: GuildId,
    filter: AuditEventFilter,
) -> Result<bool, Error> {
    let res = sqlx::query("UPDATE guild_audit_log SET disabled_events = ? WHERE guild_id = ?")
        .bind(i64::from(filter.disabled_mask()))
        .bind(guild_id.get().cast_signed())
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

/// A guild's audit-log destination plus its per-event filter.
#[derive(Debug, Clone, Copy)]
pub struct AuditSettings {
    pub channel_id: ChannelId,
    pub events: AuditEventFilter,
}

/// One switchable category of audit-log output.
///
/// Lives here (not in `audit.rs`) alongside `AntiSpamAction` so both the event handler and the
/// slash command can name a category without depending on the other's module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, poise::ChoiceParameter)]
pub enum AuditEvent {
    #[name = "Message edited"]
    MessageEdit,
    #[name = "Message deleted"]
    MessageDelete,
    #[name = "Bulk message delete"]
    MessageBulkDelete,
    #[name = "Reaction added"]
    ReactionAdd,
    #[name = "Reaction removed"]
    ReactionRemove,
    #[name = "Reactions cleared"]
    ReactionClear,
    #[name = "Member joined"]
    MemberJoin,
    #[name = "Member left"]
    MemberLeave,
    #[name = "Member banned"]
    MemberBan,
    #[name = "Member unbanned"]
    MemberUnban,
    #[name = "Moderator actions"]
    ModAction,
    #[name = "Server changes"]
    ServerChange,
}

impl AuditEvent {
    /// Every category, in the order `/auditlog show` lists them.
    pub const ALL: [Self; 12] = [
        Self::MessageEdit,
        Self::MessageDelete,
        Self::MessageBulkDelete,
        Self::ReactionAdd,
        Self::ReactionRemove,
        Self::ReactionClear,
        Self::MemberJoin,
        Self::MemberLeave,
        Self::MemberBan,
        Self::MemberUnban,
        Self::ModAction,
        Self::ServerChange,
    ];

    /// This category's bit in `guild_audit_log.disabled_events`.
    ///
    /// The values are persisted, so they are written out explicitly rather than derived from
    /// declaration order: reordering or removing a variant must never silently repoint an
    /// existing guild's filter at a different category. A retired category's bit stays retired.
    #[must_use]
    pub const fn bit(self) -> u32 {
        match self {
            Self::MessageEdit => 1 << 0,
            Self::MessageDelete => 1 << 1,
            Self::MessageBulkDelete => 1 << 2,
            Self::ReactionAdd => 1 << 3,
            Self::ReactionRemove => 1 << 4,
            Self::MemberJoin => 1 << 5,
            Self::MemberLeave => 1 << 6,
            Self::MemberBan => 1 << 7,
            Self::MemberUnban => 1 << 8,
            Self::ModAction => 1 << 9,
            Self::ServerChange => 1 << 10,
            // Added after the first ten, so it takes the next free bit rather than slotting in
            // beside the other reaction categories and shifting everything after it.
            Self::ReactionClear => 1 << 11,
        }
    }
}

/// Which audit-log categories a guild has switched off.
///
/// Held as the *disabled* mask so the default (`0`) logs everything - see the migration note in
/// `20260827000000_audit_log_events.sql`. Nothing outside this type should touch the raw bits.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AuditEventFilter(u32);

impl AuditEventFilter {
    #[must_use]
    pub const fn from_disabled_mask(mask: u32) -> Self {
        Self(mask)
    }

    #[must_use]
    pub const fn disabled_mask(self) -> u32 {
        self.0
    }

    #[must_use]
    pub const fn is_enabled(self, event: AuditEvent) -> bool {
        self.0 & event.bit() == 0
    }

    /// Switch `event` on or off. Returns whether this actually changed anything, so the command
    /// can tell the admin "already off" instead of reporting a no-op as a change.
    pub const fn set(&mut self, event: AuditEvent, enabled: bool) -> bool {
        let before = self.0;
        if enabled {
            self.0 &= !event.bit();
        } else {
            self.0 |= event.bit();
        }
        before != self.0
    }
}

/// Narrow a stored mask back to `u32`, dropping any bits `SQLite` hands back that no live
/// category claims. A negative or oversized value can only come from hand-editing the DB;
/// treating the unknown bits as "not disabled" keeps logging on rather than silently off.
fn mask_from_db(raw: i64) -> u32 {
    let known: u32 = AuditEvent::ALL.iter().fold(0, |acc, e| acc | e.bit());
    u32::try_from(raw).unwrap_or(0) & known
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
