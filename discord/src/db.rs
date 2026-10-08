use std::str::FromStr;

use serenity::all::{GuildId, RoleId, UserId};
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

/// Run `sql`, a `DELETE ... WHERE guild_id = ?`, for `guild_id`. Returns the rows removed.
async fn delete_guild_rows(
    pool: &SqlitePool,
    sql: &'static str,
    guild_id: GuildId,
) -> Result<u64, Error> {
    let result = sqlx::query(sql)
        .bind(guild_id.get().cast_signed())
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// A stored count or duration back as `u32`, saturating: only hand-editing the DB stores one
/// out of range.
fn u32_from_db(v: i64) -> u32 {
    v.try_into().unwrap_or(u32::MAX)
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

/// Look up the moderator role for `guild_id`, if any.
///
/// Read on the denial path of every elevated command (see `checks::elevated`), so it stays a
/// plain query: unlike the antispam and audit-log caches it is not on an event hot path, and a
/// cache here would only add an invalidation surface.
pub async fn get_mod_role(pool: &SqlitePool, guild_id: GuildId) -> Result<Option<RoleId>, Error> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT mod_role_id FROM guild_mod_role WHERE guild_id = ?")
            .bind(guild_id.get().cast_signed())
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(id,)| RoleId::new(id.cast_unsigned())))
}

/// Set the moderator role for `guild_id`, replacing any previous one.
pub async fn set_mod_role(
    pool: &SqlitePool,
    guild_id: GuildId,
    role_id: RoleId,
) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO guild_mod_role (guild_id, mod_role_id) VALUES (?, ?) \
         ON CONFLICT(guild_id) DO UPDATE SET mod_role_id = excluded.mod_role_id",
    )
    .bind(guild_id.get().cast_signed())
    .bind(role_id.get().cast_signed())
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove the moderator role for `guild_id`. Returns whether a row was removed.
pub async fn clear_mod_role(pool: &SqlitePool, guild_id: GuildId) -> Result<bool, Error> {
    Ok(delete_guild_rows(
        pool,
        "DELETE FROM guild_mod_role WHERE guild_id = ?",
        guild_id,
    )
    .await?
        > 0)
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
    delete_guild_rows(
        pool,
        "DELETE FROM guild_reaction_roles WHERE guild_id = ?",
        guild_id,
    )
    .await
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

/// A guild's asset-announcement binding.
#[derive(Debug, Clone)]
pub struct AssetChannel {
    pub channel_id: ChannelId,
    /// Server labels this guild hears from, as stored ("EN,JP"). `None` means every server.
    pub servers: Option<Vec<String>>,
}

impl AssetChannel {
    fn from_row(channel_id: i64, servers: Option<String>) -> Self {
        Self {
            channel_id: ChannelId::new(channel_id.cast_unsigned()),
            servers: servers.map(|s| {
                s.split(',')
                    .map(str::trim)
                    .filter(|l| !l.is_empty())
                    .map(str::to_string)
                    .collect()
            }),
        }
    }

    /// Whether announcements from the server labelled `label` go to this guild.
    #[must_use]
    pub fn wants(&self, label: &str) -> bool {
        self.servers
            .as_ref()
            .is_none_or(|s| s.iter().any(|l| l.eq_ignore_ascii_case(label)))
    }
}

/// Set the asset-announcement channel for `guild_id`, with the server labels it hears from
/// (`None` for all of them). Re-running replaces both.
pub async fn set_assets_channel(
    pool: &SqlitePool,
    guild_id: GuildId,
    channel_id: ChannelId,
    servers: Option<&[String]>,
) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO guild_asset_channel (guild_id, channel_id, servers) VALUES (?, ?, ?) \
         ON CONFLICT(guild_id) DO UPDATE SET \
            channel_id = excluded.channel_id, \
            servers = excluded.servers",
    )
    .bind(guild_id.get().cast_signed())
    .bind(channel_id.get().cast_signed())
    .bind(servers.map(|s| s.join(",")))
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove the asset-announcement binding for `guild_id`.
pub async fn clear_assets_channel(pool: &SqlitePool, guild_id: GuildId) -> Result<u64, Error> {
    delete_guild_rows(
        pool,
        "DELETE FROM guild_asset_channel WHERE guild_id = ?",
        guild_id,
    )
    .await
}

/// Look up the asset-announcement binding for `guild_id`, if any.
pub async fn get_assets_channel(
    pool: &SqlitePool,
    guild_id: GuildId,
) -> Result<Option<AssetChannel>, Error> {
    let row: Option<(i64, Option<String>)> =
        sqlx::query_as("SELECT channel_id, servers FROM guild_asset_channel WHERE guild_id = ?")
            .bind(guild_id.get().cast_signed())
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(c, servers)| AssetChannel::from_row(c, servers)))
}

/// Every asset-announcement binding. Used by the watcher to fan an event out to all
/// subscribers in one pass, skipping guilds whose server filter leaves the event's label out.
pub async fn list_assets_channels(
    pool: &SqlitePool,
) -> Result<Vec<(GuildId, AssetChannel)>, Error> {
    let rows: Vec<(i64, i64, Option<String>)> =
        sqlx::query_as("SELECT guild_id, channel_id, servers FROM guild_asset_channel")
            .fetch_all(pool)
            .await?;
    Ok(rows
        .into_iter()
        .map(|(g, c, servers)| {
            (
                GuildId::new(g.cast_unsigned()),
                AssetChannel::from_row(c, servers),
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
    delete_guild_rows(
        pool,
        "DELETE FROM guild_audit_log WHERE guild_id = ?",
        guild_id,
    )
    .await
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
    #[name = "Warnings"]
    Warning,
}

impl AuditEvent {
    /// Every category, in the order `/auditlog show` lists them.
    pub const ALL: [Self; 13] = [
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
        Self::Warning,
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
            // `/warn add`, mirrored by the bot itself rather than read from Discord's audit log.
            Self::Warning => 1 << 12,
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
        max_per_message: u32_from_db(max_per_message),
        window_secs: window_secs.map(u32_from_db),
        window_max_pings: window_max_pings.map(u32_from_db),
        action,
        timeout_secs: timeout_secs.map(u32_from_db),
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
    delete_guild_rows(
        pool,
        "DELETE FROM guild_max_ping WHERE guild_id = ?",
        guild_id,
    )
    .await
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

/// One row of `guild_warnings`.
#[derive(Debug, Clone)]
pub struct Warning {
    pub id: i64,
    pub moderator_id: UserId,
    pub reason: String,
    /// Unix seconds.
    pub created_at: i64,
}

/// Record a warning and return its id.
pub async fn add_warning(
    pool: &SqlitePool,
    guild_id: GuildId,
    user_id: UserId,
    moderator_id: UserId,
    reason: &str,
    created_at: i64,
) -> Result<i64, Error> {
    let result = sqlx::query(
        "INSERT INTO guild_warnings (guild_id, user_id, moderator_id, reason, created_at) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(guild_id.get().cast_signed())
    .bind(user_id.get().cast_signed())
    .bind(moderator_id.get().cast_signed())
    .bind(reason)
    .bind(created_at)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

/// Every warning `user_id` holds in `guild_id`, newest first.
pub async fn list_warnings(
    pool: &SqlitePool,
    guild_id: GuildId,
    user_id: UserId,
) -> Result<Vec<Warning>, Error> {
    let rows: Vec<(i64, i64, String, i64)> = sqlx::query_as(
        "SELECT id, moderator_id, reason, created_at FROM guild_warnings \
         WHERE guild_id = ? AND user_id = ? ORDER BY created_at DESC, id DESC",
    )
    .bind(guild_id.get().cast_signed())
    .bind(user_id.get().cast_signed())
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, m, reason, created_at)| Warning {
            id,
            moderator_id: UserId::new(m.cast_unsigned()),
            reason,
            created_at,
        })
        .collect())
}

/// How many warnings `user_id` holds in `guild_id` created at or after `since` (unix seconds),
/// or in total when `since` is `None`.
pub async fn count_warnings(
    pool: &SqlitePool,
    guild_id: GuildId,
    user_id: UserId,
    since: Option<i64>,
) -> Result<i64, Error> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM guild_warnings \
         WHERE guild_id = ? AND user_id = ? AND created_at >= ?",
    )
    .bind(guild_id.get().cast_signed())
    .bind(user_id.get().cast_signed())
    .bind(since.unwrap_or(i64::MIN))
    .fetch_one(pool)
    .await?;
    Ok(count)
}

/// Delete warning `id`, but only if it belongs to `guild_id`, so one server can't remove
/// another's warnings by guessing ids. Returns the removed warning's user, if any.
pub async fn remove_warning(
    pool: &SqlitePool,
    guild_id: GuildId,
    id: i64,
) -> Result<Option<UserId>, Error> {
    let row: Option<(i64,)> = sqlx::query_as(
        "DELETE FROM guild_warnings WHERE id = ? AND guild_id = ? RETURNING user_id",
    )
    .bind(id)
    .bind(guild_id.get().cast_signed())
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(u,)| UserId::new(u.cast_unsigned())))
}

/// Delete every warning `user_id` holds in `guild_id`. Returns how many were removed.
pub async fn clear_warnings(
    pool: &SqlitePool,
    guild_id: GuildId,
    user_id: UserId,
) -> Result<u64, Error> {
    let result = sqlx::query("DELETE FROM guild_warnings WHERE guild_id = ? AND user_id = ?")
        .bind(guild_id.get().cast_signed())
        .bind(user_id.get().cast_signed())
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// What happens to a member who reaches the warning threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, poise::ChoiceParameter)]
pub enum WarnAction {
    #[name = "Timeout user"]
    Timeout,
    #[name = "Kick user"]
    Kick,
    #[name = "Ban user"]
    Ban,
}

impl WarnAction {
    /// Serialized form stored in `guild_warn_policy.action`.
    #[must_use]
    pub const fn as_db_str(self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::Kick => "kick",
            Self::Ban => "ban",
        }
    }

    fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "timeout" => Some(Self::Timeout),
            "kick" => Some(Self::Kick),
            "ban" => Some(Self::Ban),
            _ => None,
        }
    }
}

/// A guild's warning escalation policy. No row means no escalation.
#[derive(Debug, Clone, Copy)]
pub struct WarnPolicy {
    pub threshold: u32,
    /// Only warnings this recent count; `None` counts every warning ever given.
    pub window_days: Option<u32>,
    pub action: WarnAction,
    /// Used only when `action == Timeout`.
    pub timeout_secs: Option<u32>,
}

/// Insert or replace the warning policy for `guild_id`.
pub async fn set_warn_policy(
    pool: &SqlitePool,
    guild_id: GuildId,
    policy: &WarnPolicy,
) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO guild_warn_policy (guild_id, threshold, window_days, action, timeout_secs) \
         VALUES (?, ?, ?, ?, ?) \
         ON CONFLICT(guild_id) DO UPDATE SET \
            threshold = excluded.threshold, \
            window_days = excluded.window_days, \
            action = excluded.action, \
            timeout_secs = excluded.timeout_secs",
    )
    .bind(guild_id.get().cast_signed())
    .bind(i64::from(policy.threshold))
    .bind(policy.window_days.map(i64::from))
    .bind(policy.action.as_db_str())
    .bind(policy.timeout_secs.map(i64::from))
    .execute(pool)
    .await?;
    Ok(())
}

/// Look up the warning policy for `guild_id`, if one is configured.
pub async fn get_warn_policy(
    pool: &SqlitePool,
    guild_id: GuildId,
) -> Result<Option<WarnPolicy>, Error> {
    let row: Option<(i64, Option<i64>, String, Option<i64>)> = sqlx::query_as(
        "SELECT threshold, window_days, action, timeout_secs FROM guild_warn_policy \
         WHERE guild_id = ?",
    )
    .bind(guild_id.get().cast_signed())
    .fetch_optional(pool)
    .await?;
    row.map(|(threshold, window_days, action, timeout_secs)| {
        Ok(WarnPolicy {
            threshold: u32_from_db(threshold),
            window_days: window_days.map(u32_from_db),
            action: WarnAction::from_db_str(&action)
                .ok_or_else(|| format!("Invalid warn action '{action}' for guild {guild_id}"))?,
            timeout_secs: timeout_secs.map(u32_from_db),
        })
    })
    .transpose()
}

/// Drop the warning policy for `guild_id`. Returns the number of rows removed.
pub async fn clear_warn_policy(pool: &SqlitePool, guild_id: GuildId) -> Result<u64, Error> {
    delete_guild_rows(
        pool,
        "DELETE FROM guild_warn_policy WHERE guild_id = ?",
        guild_id,
    )
    .await
}

/// Bind birthday announcements for `guild_id` to `channel_id`.
///
/// On a fresh binding `posted_date` (the current game day, `YYYY-MM-DD`) is written as already
/// posted, so a channel bound after today's reset starts with tomorrow's birthdays instead of
/// firing at once. Moving an existing binding only changes the channel and keeps its
/// `last_posted_date`, so a move can neither repeat nor skip a day.
pub async fn set_birthday_channel(
    pool: &SqlitePool,
    guild_id: GuildId,
    channel_id: ChannelId,
    posted_date: &str,
) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO guild_birthday_channel (guild_id, channel_id, last_posted_date) \
         VALUES (?, ?, ?) \
         ON CONFLICT(guild_id) DO UPDATE SET channel_id = excluded.channel_id",
    )
    .bind(guild_id.get().cast_signed())
    .bind(channel_id.get().cast_signed())
    .bind(posted_date)
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove the birthday binding for `guild_id`. Returns the number of rows removed.
pub async fn clear_birthday_channel(pool: &SqlitePool, guild_id: GuildId) -> Result<u64, Error> {
    delete_guild_rows(
        pool,
        "DELETE FROM guild_birthday_channel WHERE guild_id = ?",
        guild_id,
    )
    .await
}

/// The birthday channel for `guild_id` and the last game day posted there, if bound.
pub async fn get_birthday_channel(
    pool: &SqlitePool,
    guild_id: GuildId,
) -> Result<Option<(ChannelId, Option<String>)>, Error> {
    let row: Option<(i64, Option<String>)> = sqlx::query_as(
        "SELECT channel_id, last_posted_date FROM guild_birthday_channel WHERE guild_id = ?",
    )
    .bind(guild_id.get().cast_signed())
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(c, d)| (ChannelId::new(c.cast_unsigned()), d)))
}

/// Every binding that hasn't posted `date` yet.
pub async fn list_birthday_channels_due(
    pool: &SqlitePool,
    date: &str,
) -> Result<Vec<(GuildId, ChannelId)>, Error> {
    let rows: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT guild_id, channel_id FROM guild_birthday_channel \
         WHERE last_posted_date IS NULL OR last_posted_date <> ?",
    )
    .bind(date)
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

/// Record that `guild_id` has had `date`'s birthdays.
pub async fn mark_birthday_posted(
    pool: &SqlitePool,
    guild_id: GuildId,
    date: &str,
) -> Result<(), Error> {
    sqlx::query("UPDATE guild_birthday_channel SET last_posted_date = ? WHERE guild_id = ?")
        .bind(date)
        .bind(guild_id.get().cast_signed())
        .execute(pool)
        .await?;
    Ok(())
}

/// Set `user_id`'s TTS nickname in `guild_id`, replacing any previous one.
pub async fn set_tts_nickname(
    pool: &SqlitePool,
    guild_id: GuildId,
    user_id: UserId,
    nickname: &str,
    set_by: UserId,
    at: i64,
) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO guild_tts_nicknames (guild_id, user_id, nickname, set_by, set_at) \
         VALUES (?, ?, ?, ?, ?) \
         ON CONFLICT(guild_id, user_id) DO UPDATE SET nickname = excluded.nickname, \
         set_by = excluded.set_by, set_at = excluded.set_at",
    )
    .bind(guild_id.get().cast_signed())
    .bind(user_id.get().cast_signed())
    .bind(nickname)
    .bind(set_by.get().cast_signed())
    .bind(at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove `user_id`'s TTS nickname in `guild_id`. Returns whether one was set.
pub async fn clear_tts_nickname(
    pool: &SqlitePool,
    guild_id: GuildId,
    user_id: UserId,
) -> Result<bool, Error> {
    let result = sqlx::query("DELETE FROM guild_tts_nicknames WHERE guild_id = ? AND user_id = ?")
        .bind(guild_id.get().cast_signed())
        .bind(user_id.get().cast_signed())
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Every TTS nickname. Hydrates the cache the message hot path reads.
pub async fn list_tts_nicknames(
    pool: &SqlitePool,
) -> Result<Vec<(GuildId, UserId, String)>, Error> {
    let rows: Vec<(i64, i64, String)> =
        sqlx::query_as("SELECT guild_id, user_id, nickname FROM guild_tts_nicknames")
            .fetch_all(pool)
            .await?;
    Ok(rows
        .into_iter()
        .map(|(g, u, n)| {
            (
                GuildId::new(g.cast_unsigned()),
                UserId::new(u.cast_unsigned()),
                n,
            )
        })
        .collect())
}

/// Set `user_id`'s TTS voice, replacing any previous pick.
pub async fn set_tts_voice(
    pool: &SqlitePool,
    user_id: UserId,
    voice: &str,
    at: i64,
) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO user_tts_voice (user_id, voice, set_at) VALUES (?, ?, ?) \
         ON CONFLICT(user_id) DO UPDATE SET voice = excluded.voice, set_at = excluded.set_at",
    )
    .bind(user_id.get().cast_signed())
    .bind(voice)
    .bind(at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove `user_id`'s TTS voice pick. Returns whether one was set.
pub async fn clear_tts_voice(pool: &SqlitePool, user_id: UserId) -> Result<bool, Error> {
    let result = sqlx::query("DELETE FROM user_tts_voice WHERE user_id = ?")
        .bind(user_id.get().cast_signed())
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Every TTS voice pick. Hydrates the cache the message hot path reads.
pub async fn list_tts_voices(pool: &SqlitePool) -> Result<Vec<(UserId, String)>, Error> {
    let rows: Vec<(i64, String)> = sqlx::query_as("SELECT user_id, voice FROM user_tts_voice")
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|(u, v)| (UserId::new(u.cast_unsigned()), v))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u32_from_db_saturates() {
        assert_eq!(u32_from_db(0), 0);
        assert_eq!(u32_from_db(42), 42);
        assert_eq!(u32_from_db(i64::from(u32::MAX)), u32::MAX);
        assert_eq!(u32_from_db(i64::from(u32::MAX) + 1), u32::MAX);
        // A negative value also fails `try_into` and saturates high, not low.
        assert_eq!(u32_from_db(-1), u32::MAX);
    }

    #[test]
    fn audit_event_bits_are_distinct_single_bits() {
        let mut seen = 0u32;
        for event in AuditEvent::ALL {
            let bit = event.bit();
            assert_eq!(bit.count_ones(), 1, "{event:?}");
            assert_eq!(seen & bit, 0, "{event:?} reuses a bit");
            seen |= bit;
        }
        // Thirteen categories on bits 0..=12.
        assert_eq!(seen, 0x1FFF);
    }

    #[test]
    fn persisted_bits_never_move() {
        assert_eq!(AuditEvent::MessageEdit.bit(), 1);
        assert_eq!(AuditEvent::ServerChange.bit(), 1 << 10);
        assert_eq!(AuditEvent::ReactionClear.bit(), 1 << 11);
        assert_eq!(AuditEvent::Warning.bit(), 1 << 12);
    }

    #[test]
    fn mask_from_db_keeps_only_known_bits() {
        assert_eq!(mask_from_db(0), 0);
        assert_eq!(mask_from_db(0b101), 0b101);
        assert_eq!(mask_from_db(0x1FFF), 0x1FFF);
        assert_eq!(mask_from_db((1 << 20) | 0b11), 0b11);
        // Out of u32 range or negative: nothing disabled, logging stays on.
        assert_eq!(mask_from_db(-1), 0);
        assert_eq!(mask_from_db(i64::from(u32::MAX) + 1), 0);
    }
}
