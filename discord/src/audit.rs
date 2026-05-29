//! Audit-log embed builders.
//!
//! Each `log_*` helper short-circuits when the guild has no configured channel, builds a
//! `CreateEmbed`, and dispatches it through `dispatch`. Send failures are warn-logged and
//! swallowed - a broken audit-log destination must never break the event handler or other
//! features.

use std::fmt::Write as _;

use serenity::all::{Context, Http, MessageUpdateEvent, Timestamp};
use serenity::builder::{
    CreateAttachment, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage,
};
use serenity::model::channel::Message;
use serenity::model::guild::Member;
use serenity::model::guild::audit_log::{
    Action, AuditLogEntry, ChannelAction, ChannelOverwriteAction, EmojiAction, IntegrationAction,
    InviteAction, MemberAction, MessageAction, RoleAction, StickerAction, ThreadAction,
};
use serenity::model::id::{ChannelId, GuildId, MessageId};
use serenity::model::user::User;

use crate::types::Data;

#[allow(clippy::unreadable_literal)]
const COLOR_EDIT: u32 = 0xFEE75C;
#[allow(clippy::unreadable_literal)]
const COLOR_DELETE: u32 = 0xED4245;
#[allow(clippy::unreadable_literal)]
const COLOR_JOIN: u32 = 0x57F287;
#[allow(clippy::unreadable_literal)]
const COLOR_LEAVE: u32 = 0xFEE75C;
#[allow(clippy::unreadable_literal)]
const COLOR_BAN: u32 = 0xED4245;
#[allow(clippy::unreadable_literal)]
const COLOR_UNBAN: u32 = 0x57F287;
#[allow(clippy::unreadable_literal)]
const COLOR_MOD: u32 = 0xEB459E;
#[allow(clippy::unreadable_literal)]
const COLOR_STRUCT: u32 = 0x5865F2;

/// Discord embed field-value limit.
const FIELD_LIMIT: usize = 1024;
/// Discord message-content limit (used as a soft cap for the bulk-delete summary).
const MESSAGE_LIMIT: usize = 2000;
/// Max cached messages to render inline in a bulk-delete summary before we attach a file.
const BULK_INLINE_CAP: usize = 25;

/// Log a message edit. Skips bot authors, edits in the audit channel itself, and updates
/// that didn't change `.content` (embed-load updates fire `MessageUpdate` too).
pub async fn log_message_update(
    ctx: &Context,
    data: &Data,
    old: Option<&Message>,
    new: Option<&Message>,
    event: &MessageUpdateEvent,
) {
    let Some(guild_id) = event.guild_id.or_else(|| new.and_then(|m| m.guild_id)) else {
        return;
    };
    let Some(channel) = audit_channel(data, guild_id).await else {
        return;
    };
    if event.channel_id == channel {
        return;
    }

    let new_content = new
        .map(|m| m.content.as_str())
        .or(event.content.as_deref())
        .unwrap_or("");
    let old_content = old.map_or("", |m| m.content.as_str());

    if old.is_some() && old_content == new_content {
        return;
    }

    if new.is_some_and(|m| m.author.bot) || event.author.as_ref().is_some_and(|u| u.bot) {
        return;
    }

    let author = new
        .map(|m| (m.author.id, m.author.tag(), m.author.face()))
        .or_else(|| event.author.as_ref().map(|u| (u.id, u.tag(), u.face())));

    let jump = message_link(Some(guild_id), event.channel_id, event.id);

    let mut embed = CreateEmbed::new()
        .title("Message edited")
        .colour(COLOR_EDIT)
        .field("Channel", format!("<#{}>", event.channel_id), true)
        .field("Jump", format!("[link]({jump})"), true)
        .timestamp(Timestamp::now());

    let mut attachments: Vec<CreateAttachment> = Vec::new();
    let (before_field, before_attach) = format_content_field(
        if old.is_some() { old_content } else { "" },
        "before.txt",
        old.is_some(),
    );
    let (after_field, after_attach) = format_content_field(new_content, "after.txt", true);
    embed = embed.field("Before", before_field, false);
    embed = embed.field("After", after_field, false);
    attachments.extend(before_attach);
    attachments.extend(after_attach);

    if let Some((user_id, tag, face)) = author {
        embed = embed.author(CreateEmbedAuthor::new(tag).icon_url(face));
        embed = embed.footer(CreateEmbedFooter::new(format!(
            "Author ID: {user_id} • Message ID: {}",
            event.id
        )));
    } else {
        embed = embed.footer(CreateEmbedFooter::new(format!("Message ID: {}", event.id)));
    }

    dispatch(&ctx.http, channel, embed, attachments).await;
}

/// Log a single message delete. With the cache feature on, `ctx.cache.message` returns the
/// pre-delete `Message`; on cache miss we still emit a stub embed with the IDs.
pub async fn log_message_delete(
    ctx: &Context,
    data: &Data,
    channel_id: ChannelId,
    message_id: MessageId,
    guild_id: Option<GuildId>,
) {
    let Some(guild_id) = guild_id else {
        return;
    };
    let Some(audit_channel_id) = audit_channel(data, guild_id).await else {
        return;
    };
    if channel_id == audit_channel_id {
        return;
    }

    let cached = ctx.cache.message(channel_id, message_id).map(|m| m.clone());

    if cached.as_ref().is_some_and(|m| m.author.bot) {
        return;
    }

    let mut embed = CreateEmbed::new()
        .title("Message deleted")
        .colour(COLOR_DELETE)
        .field("Channel", format!("<#{channel_id}>"), true)
        .timestamp(Timestamp::now());

    let mut attachments: Vec<CreateAttachment> = Vec::new();
    if let Some(msg) = cached.as_ref() {
        embed = embed.author(CreateEmbedAuthor::new(msg.author.tag()).icon_url(msg.author.face()));
        let (content_field, content_attach) =
            format_content_field(&msg.content, "deleted.txt", true);
        embed = embed.field("Content", content_field, false);
        attachments.extend(content_attach);
        if !msg.attachments.is_empty() {
            let names = msg
                .attachments
                .iter()
                .map(|a| format!("`{}` ({} B)", a.filename, a.size))
                .collect::<Vec<_>>()
                .join("\n");
            embed = embed.field("Attachments", truncate(&names, FIELD_LIMIT), false);
        }
        embed = embed.footer(CreateEmbedFooter::new(format!(
            "Author ID: {} • Message ID: {message_id}",
            msg.author.id
        )));
    } else {
        embed = embed.description(
            "*(no cached content - message was sent before the bot restarted or fell out of cache)*",
        );
        embed = embed.footer(CreateEmbedFooter::new(format!("Message ID: {message_id}")));
    }

    dispatch(&ctx.http, audit_channel_id, embed, attachments).await;
}

/// Log a bulk delete. Renders cached messages inline (≤25), then a `+M uncached` tail line;
/// summaries that overflow the 2000-char message limit are sent as `bulk_delete.txt`.
pub async fn log_message_bulk_delete(
    ctx: &Context,
    data: &Data,
    channel_id: ChannelId,
    ids: &[MessageId],
    guild_id: Option<GuildId>,
) {
    let Some(guild_id) = guild_id else {
        return;
    };
    let Some(audit_channel_id) = audit_channel(data, guild_id).await else {
        return;
    };
    if channel_id == audit_channel_id {
        return;
    }

    let mut cached: Vec<Message> = Vec::new();
    let mut uncached: usize = 0;
    for &id in ids {
        match ctx.cache.message(channel_id, id) {
            Some(m) => cached.push(m.clone()),
            None => uncached += 1,
        }
    }

    let mut body = String::new();
    for msg in cached.iter().take(BULK_INLINE_CAP) {
        let snippet = truncate_oneline(&msg.content, 80);
        let _ = writeln!(
            body,
            "• <@{}> at <t:{}:f>: {snippet}",
            msg.author.id,
            msg.timestamp.unix_timestamp(),
        );
    }
    if cached.len() > BULK_INLINE_CAP {
        let _ = writeln!(body, "…and {} more cached", cached.len() - BULK_INLINE_CAP);
    }
    if uncached > 0 {
        let _ = writeln!(body, "+{uncached} uncached (older than cache TTL)");
    }
    if body.is_empty() {
        body.push_str("*(no cached content)*");
    }

    let mut embed = CreateEmbed::new()
        .title(format!("Bulk delete: {} messages", ids.len()))
        .colour(COLOR_DELETE)
        .field("Channel", format!("<#{channel_id}>"), true)
        .timestamp(Timestamp::now())
        .footer(CreateEmbedFooter::new(format!(
            "Total: {} • Cached: {} • Uncached: {uncached}",
            ids.len(),
            cached.len(),
        )));

    let mut attachments: Vec<CreateAttachment> = Vec::new();
    if body.len() <= MESSAGE_LIMIT && body.len() <= 4000 {
        embed = embed.description(body);
    } else {
        embed = embed.description("Summary too large for an embed - see attachment.");
        let full = render_bulk_full(&cached, uncached, ids.len());
        attachments.push(CreateAttachment::bytes(
            full.into_bytes(),
            "bulk_delete.txt",
        ));
    }

    dispatch(&ctx.http, audit_channel_id, embed, attachments).await;
}

async fn audit_channel(data: &Data, guild_id: GuildId) -> Option<ChannelId> {
    data.audit_log_channels.read().await.get(&guild_id).copied()
}

async fn dispatch(
    http: &Http,
    channel: ChannelId,
    embed: CreateEmbed,
    attachments: Vec<CreateAttachment>,
) {
    let mut msg = CreateMessage::new().embed(embed);
    for a in attachments {
        msg = msg.add_file(a);
    }
    if let Err(e) = channel.send_message(http, msg).await {
        tracing::warn!("audit: send to {channel} failed: {e}");
    }
}

/// Render `content` for an embed field. Returns `(field_value, attachments_to_add)`.
/// Long content is truncated and the full text spilled into an attachment.
fn format_content_field(
    content: &str,
    filename: &'static str,
    available: bool,
) -> (String, Vec<CreateAttachment>) {
    if !available {
        return ("*(unavailable - not in cache)*".to_string(), Vec::new());
    }
    if content.is_empty() {
        return ("*(empty)*".to_string(), Vec::new());
    }
    if content.len() <= FIELD_LIMIT {
        return (content.to_string(), Vec::new());
    }
    let truncated = format!(
        "{}…\n*(full content attached as `{filename}`)*",
        truncate(content, FIELD_LIMIT - 64),
    );
    let attach = CreateAttachment::bytes(content.to_owned().into_bytes(), filename);
    (truncated, vec![attach])
}

fn render_bulk_full(cached: &[Message], uncached: usize, total: usize) -> String {
    let mut out = format!(
        "Bulk delete summary\nTotal: {total}\nCached: {}\nUncached: {uncached}\n\n",
        cached.len()
    );
    for msg in cached {
        let _ = writeln!(
            out,
            "[{}] {} ({}): {}",
            msg.timestamp,
            msg.author.tag(),
            msg.author.id,
            msg.content,
        );
        for a in &msg.attachments {
            let _ = writeln!(out, "    attachment: {} ({} B)", a.filename, a.size);
        }
    }
    out
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    // Walk to the closest char boundary at or below `max` to avoid splitting a UTF-8 sequence.
    let mut cut = max;
    while cut > 0 && !s.is_char_boundary(cut) {
        cut -= 1;
    }
    s[..cut].to_string()
}

fn truncate_oneline(s: &str, max: usize) -> String {
    let single = s.replace('\n', " ");
    if single.chars().count() <= max {
        return single;
    }
    let cut: String = single.chars().take(max).collect();
    format!("{cut}…")
}

fn message_link(guild: Option<GuildId>, channel: ChannelId, message: MessageId) -> String {
    match guild {
        Some(g) => format!("https://discord.com/channels/{g}/{channel}/{message}"),
        None => format!("https://discord.com/channels/@me/{channel}/{message}"),
    }
}

// ---------------------------------------------------------------------------
// Membership: join / leave / ban / unban
// ---------------------------------------------------------------------------

/// Log a guild join. Joins are gateway-only - Discord does not write an audit log entry for
/// them, so the gateway event is the canonical source.
pub async fn log_member_join(ctx: &Context, data: &Data, member: &Member) {
    let Some(channel) = audit_channel(data, member.guild_id).await else {
        return;
    };
    if member.user.bot {
        return;
    }

    let created = member.user.id.created_at();
    let mut embed = CreateEmbed::new()
        .title("Member joined")
        .colour(COLOR_JOIN)
        .author(CreateEmbedAuthor::new(member.user.tag()).icon_url(member.user.face()))
        .field("User", format!("<@{}>", member.user.id), true)
        .field(
            "Account created",
            format!("<t:{}:R>", created.unix_timestamp()),
            true,
        )
        .timestamp(Timestamp::now())
        .footer(CreateEmbedFooter::new(format!(
            "User ID: {}",
            member.user.id
        )));

    // Highlight suspiciously young accounts - common bot signal.
    let age_days = (Timestamp::now().unix_timestamp() - created.unix_timestamp()) / 86_400;
    if age_days < 7 {
        embed = embed.description(format!("*New account: {age_days} day(s) old.*"));
    }

    dispatch(&ctx.http, channel, embed, Vec::new()).await;
}

/// Log a guild leave with neutral wording.
///
/// Gateway fires for both voluntary leaves and kicks; if it was a kick or ban, the matching
/// `GuildAuditLogEntryCreate` embed appears adjacent with the actor.
pub async fn log_member_leave(
    ctx: &Context,
    data: &Data,
    guild_id: GuildId,
    user: &User,
    member_data: Option<&Member>,
) {
    let Some(channel) = audit_channel(data, guild_id).await else {
        return;
    };
    if user.bot {
        return;
    }

    let mut embed = CreateEmbed::new()
        .title("Member left")
        .colour(COLOR_LEAVE)
        .author(CreateEmbedAuthor::new(user.tag()).icon_url(user.face()))
        .field("User", format!("<@{}>", user.id), true)
        .timestamp(Timestamp::now())
        .footer(CreateEmbedFooter::new(format!("User ID: {}", user.id)));

    if let Some(m) = member_data {
        if let Some(joined) = m.joined_at {
            let stayed_days =
                (Timestamp::now().unix_timestamp() - joined.unix_timestamp()) / 86_400;
            embed = embed.field(
                "Joined",
                format!(
                    "<t:{}:R> ({} day(s) ago)",
                    joined.unix_timestamp(),
                    stayed_days
                ),
                true,
            );
        }
        if !m.roles.is_empty() {
            let roles = m
                .roles
                .iter()
                .map(|r| format!("<@&{r}>"))
                .collect::<Vec<_>>()
                .join(" ");
            embed = embed.field("Roles", truncate(&roles, FIELD_LIMIT), false);
        }
    }

    dispatch(&ctx.http, channel, embed, Vec::new()).await;
}

/// Log a ban (gateway-side, no actor). The `GuildAuditLogEntryCreate` for `MemberBanAdd` will
/// carry the actor in its own embed.
pub async fn log_member_ban(ctx: &Context, data: &Data, guild_id: GuildId, user: &User) {
    let Some(channel) = audit_channel(data, guild_id).await else {
        return;
    };
    let embed = CreateEmbed::new()
        .title("Member banned")
        .colour(COLOR_BAN)
        .author(CreateEmbedAuthor::new(user.tag()).icon_url(user.face()))
        .field("User", format!("<@{}>", user.id), true)
        .timestamp(Timestamp::now())
        .footer(CreateEmbedFooter::new(format!("User ID: {}", user.id)));
    dispatch(&ctx.http, channel, embed, Vec::new()).await;
}

/// Log an unban (gateway-side, no actor). The `GuildAuditLogEntryCreate` for `MemberBanRemove`
/// will carry the actor.
pub async fn log_member_unban(ctx: &Context, data: &Data, guild_id: GuildId, user: &User) {
    let Some(channel) = audit_channel(data, guild_id).await else {
        return;
    };
    let embed = CreateEmbed::new()
        .title("Member unbanned")
        .colour(COLOR_UNBAN)
        .author(CreateEmbedAuthor::new(user.tag()).icon_url(user.face()))
        .field("User", format!("<@{}>", user.id), true)
        .timestamp(Timestamp::now())
        .footer(CreateEmbedFooter::new(format!("User ID: {}", user.id)));
    dispatch(&ctx.http, channel, embed, Vec::new()).await;
}

// ---------------------------------------------------------------------------
// Audit log entries: the canonical "who did what" source.
// ---------------------------------------------------------------------------

/// Log a `GuildAuditLogEntryCreate` event. This is the source of truth for moderator
/// attribution on bans, kicks, timeouts, role/channel/guild edits, ownership transfer, etc.
pub async fn log_audit_entry(ctx: &Context, data: &Data, guild_id: GuildId, entry: &AuditLogEntry) {
    let Some(channel) = audit_channel(data, guild_id).await else {
        return;
    };
    let Some((title, color)) = action_title(entry.action) else {
        // Action variants we intentionally skip (we already log via gateway, or out of scope).
        return;
    };

    let mut embed = CreateEmbed::new()
        .title(title)
        .colour(color)
        .field("Moderator", format!("<@{}>", entry.user_id), true)
        .timestamp(Timestamp::now())
        .footer(CreateEmbedFooter::new(format!("Entry ID: {}", entry.id)));

    if let Some(target) = entry.target_id {
        let target_field = render_target(entry.action, target.get());
        embed = embed.field("Target", target_field, true);
    }

    if let Some(reason) = entry.reason.as_deref()
        && !reason.is_empty()
    {
        embed = embed.field("Reason", truncate(reason, FIELD_LIMIT), false);
    }

    if let Some(opts) = entry.options.as_ref() {
        let opts_field = render_options(opts);
        if !opts_field.is_empty() {
            embed = embed.field("Details", opts_field, false);
        }
    }

    if let Some(changes) = entry.changes.as_ref()
        && !changes.is_empty()
    {
        let changes_field = render_changes(changes);
        if !changes_field.is_empty() {
            embed = embed.field("Changes", truncate(&changes_field, FIELD_LIMIT), false);
        }
    }

    dispatch(&ctx.http, channel, embed, Vec::new()).await;
}

/// Map an `Action` to an embed title + color. Returns `None` for variants we deliberately
/// skip (e.g. `MemberBanAdd` - already covered by the gateway `GuildBanAddition` embed).
const fn action_title(action: Action) -> Option<(&'static str, u32)> {
    match action {
        Action::GuildUpdate => Some(("Server updated", COLOR_STRUCT)),
        Action::Channel(a) => match a {
            ChannelAction::Create => Some(("Channel created", COLOR_STRUCT)),
            ChannelAction::Update => Some(("Channel updated", COLOR_STRUCT)),
            ChannelAction::Delete => Some(("Channel deleted", COLOR_DELETE)),
            _ => Some(("Channel changed", COLOR_STRUCT)),
        },
        Action::ChannelOverwrite(a) => match a {
            ChannelOverwriteAction::Create => Some(("Permission overwrite added", COLOR_STRUCT)),
            ChannelOverwriteAction::Update => Some(("Permission overwrite updated", COLOR_STRUCT)),
            ChannelOverwriteAction::Delete => Some(("Permission overwrite removed", COLOR_DELETE)),
            _ => Some(("Permission overwrite changed", COLOR_STRUCT)),
        },
        Action::Member(a) => match a {
            // Kick/Ban gateway events are already logged; suppress the audit-log duplicate
            // for the "what happened" part but keep an actor-attributed embed so admins
            // can see *who* did it.
            MemberAction::Kick => Some(("Member kicked", COLOR_MOD)),
            MemberAction::BanAdd => Some(("Ban issued", COLOR_MOD)),
            MemberAction::BanRemove => Some(("Ban lifted", COLOR_MOD)),
            MemberAction::Prune => Some(("Members pruned", COLOR_MOD)),
            MemberAction::Update => Some(("Member updated", COLOR_MOD)),
            MemberAction::RoleUpdate => Some(("Member roles changed", COLOR_MOD)),
            MemberAction::MemberMove => Some(("Member moved", COLOR_MOD)),
            MemberAction::MemberDisconnect => Some(("Member disconnected", COLOR_MOD)),
            MemberAction::BotAdd => Some(("Bot added", COLOR_STRUCT)),
            _ => Some(("Member changed", COLOR_MOD)),
        },
        Action::Role(a) => match a {
            RoleAction::Create => Some(("Role created", COLOR_STRUCT)),
            RoleAction::Update => Some(("Role updated", COLOR_STRUCT)),
            RoleAction::Delete => Some(("Role deleted", COLOR_DELETE)),
            _ => Some(("Role changed", COLOR_STRUCT)),
        },
        Action::Invite(a) => match a {
            InviteAction::Create => Some(("Invite created", COLOR_STRUCT)),
            InviteAction::Update => Some(("Invite updated", COLOR_STRUCT)),
            InviteAction::Delete => Some(("Invite deleted", COLOR_DELETE)),
            _ => Some(("Invite changed", COLOR_STRUCT)),
        },
        Action::Webhook(_) => Some(("Webhook changed", COLOR_STRUCT)),
        Action::Emoji(a) => match a {
            EmojiAction::Create => Some(("Emoji created", COLOR_STRUCT)),
            EmojiAction::Update => Some(("Emoji updated", COLOR_STRUCT)),
            EmojiAction::Delete => Some(("Emoji deleted", COLOR_DELETE)),
            _ => Some(("Emoji changed", COLOR_STRUCT)),
        },
        Action::Sticker(a) => match a {
            StickerAction::Create => Some(("Sticker created", COLOR_STRUCT)),
            StickerAction::Update => Some(("Sticker updated", COLOR_STRUCT)),
            StickerAction::Delete => Some(("Sticker deleted", COLOR_DELETE)),
            _ => Some(("Sticker changed", COLOR_STRUCT)),
        },
        Action::Message(a) => match a {
            // Author-side deletes already go through `log_message_delete`; this entry
            // adds the moderator attribution when a mod did it.
            MessageAction::Delete => Some(("Message deleted (by moderator)", COLOR_MOD)),
            MessageAction::BulkDelete => Some(("Bulk delete (by moderator)", COLOR_MOD)),
            MessageAction::Pin => Some(("Message pinned", COLOR_STRUCT)),
            MessageAction::Unpin => Some(("Message unpinned", COLOR_STRUCT)),
            _ => Some(("Message changed", COLOR_MOD)),
        },
        Action::Integration(a) => match a {
            IntegrationAction::Create => Some(("Integration added", COLOR_STRUCT)),
            IntegrationAction::Update => Some(("Integration updated", COLOR_STRUCT)),
            IntegrationAction::Delete => Some(("Integration removed", COLOR_DELETE)),
            _ => Some(("Integration changed", COLOR_STRUCT)),
        },
        Action::Thread(a) => match a {
            ThreadAction::Create => Some(("Thread created", COLOR_STRUCT)),
            ThreadAction::Update => Some(("Thread updated", COLOR_STRUCT)),
            ThreadAction::Delete => Some(("Thread deleted", COLOR_DELETE)),
            _ => Some(("Thread changed", COLOR_STRUCT)),
        },
        Action::ScheduledEvent(_) => Some(("Scheduled event changed", COLOR_STRUCT)),
        Action::StageInstance(_) => Some(("Stage instance changed", COLOR_STRUCT)),
        Action::AutoMod(_) => Some(("AutoMod action", COLOR_MOD)),
        // Variants we don't render (yet): CreatorMonetization, VoiceChannelStatus, Unknown.
        _ => None,
    }
}

/// Render the `target_id` field with a context-appropriate mention shape.
fn render_target(action: Action, target_id: u64) -> String {
    match action {
        Action::Channel(_) | Action::ChannelOverwrite(_) | Action::Thread(_) => {
            format!("<#{target_id}> (`{target_id}`)")
        }
        Action::Role(_) => format!("<@&{target_id}> (`{target_id}`)"),
        Action::Member(_) => format!("<@{target_id}> (`{target_id}`)"),
        Action::Message(_) => format!("Message `{target_id}`"),
        _ => format!("`{target_id}`"),
    }
}

/// Render an `Options` payload as a multi-line field.
fn render_options(opts: &serenity::model::guild::audit_log::Options) -> String {
    let mut lines: Vec<String> = Vec::new();
    if let Some(c) = opts.count {
        lines.push(format!("Count: {c}"));
    }
    if let Some(c) = opts.channel_id {
        lines.push(format!("Channel: <#{c}>"));
    }
    if let Some(m) = opts.message_id {
        lines.push(format!("Message: `{m}`"));
    }
    if let Some(days) = opts.delete_member_days {
        lines.push(format!("Inactive days: {days}"));
    }
    if let Some(removed) = opts.members_removed {
        lines.push(format!("Members removed: {removed}"));
    }
    if let Some(role) = opts.role_name.as_ref() {
        lines.push(format!("Role: {role}"));
    }
    if let Some(kind) = opts.kind.as_ref() {
        lines.push(format!("Type: {kind:?}"));
    }
    if let Some(status) = opts.status.as_ref() {
        lines.push(format!("Status: {status}"));
    }
    lines.join("\n")
}

/// Render a `Vec<Change>` generically. Serde round-trips each variant to `{"key": "...",
/// "old_value": ..., "new_value": ...}` so we can pull old/new without exhaustively matching
/// every Change variant (there are ~80).
fn render_changes(changes: &[serenity::model::guild::audit_log::Change]) -> String {
    let mut lines: Vec<String> = Vec::new();
    for change in changes {
        let key = change.key();
        let line = match serde_json::to_value(change) {
            Ok(v) => {
                let old = v.get("old_value").map(value_to_str);
                let new = v.get("new_value").map(value_to_str);
                match (old, new) {
                    (Some(o), Some(n)) => format!("• `{key}`: `{o}` → `{n}`"),
                    (None, Some(n)) => format!("• `{key}`: → `{n}`"),
                    (Some(o), None) => format!("• `{key}`: `{o}` →"),
                    (None, None) => format!("• `{key}` changed"),
                }
            }
            Err(_) => format!("• `{key}` changed"),
        };
        lines.push(line);
    }
    lines.join("\n")
}

/// Compact one-line rendering of an arbitrary JSON value for change diffs. Long values
/// are truncated; objects/arrays collapse to their JSON form.
fn value_to_str(v: &serde_json::Value) -> String {
    let s = match v {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    truncate_oneline(&s, 80)
}
