use std::fmt::Write as _;

use poise::CreateReply;
use poise::serenity_prelude as serenity;
// `.name()` (the `#[name = "..."]` label on each `AuditEvent`) comes from this trait.
use poise::ChoiceParameter as _;

use crate::db::{self, AuditEvent, AuditSettings};
use crate::types::{Context, Error};

/// Configure the audit-log channel for this guild.
///
/// Subcommands: `set`, `clear`, `show`, `enable`, `disable`. When bound, the bot mirrors
/// message, reaction, membership and moderation events from every channel into the chosen log
/// channel; `enable`/`disable` switch individual categories off. Requires the Manage Server
/// permission.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "MANAGE_GUILD",
    subcommands("set", "clear", "show", "enable", "disable"),
    subcommand_required
)]
pub async fn auditlog(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Bind audit-log events to a channel.
#[poise::command(
    slash_command,
    guild_only,
    rename = "set",
    required_permissions = "MANAGE_GUILD"
)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "Channel to receive audit-log events"] channel: serenity::ChannelId,
) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;
    db::set_audit_log_channel(&ctx.data().pool, guild, channel)
        .await
        .map_err(|e| format!("Couldn't save audit-log channel: {e}"))?;
    {
        // Re-binding only moves the destination - the DB keeps `disabled_events` on conflict,
        // so the cache has to carry the guild's event filter over too.
        let mut cache = ctx.data().audit_log_settings.write().await;
        let events = cache
            .get(&guild)
            .map_or_else(Default::default, |s| s.events);
        cache.insert(
            guild,
            AuditSettings {
                channel_id: channel,
                events,
            },
        );
    }
    ctx.send(
        CreateReply::default()
            .content(format!("Audit log will be sent to <#{channel}>."))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Stop sending audit-log events for this guild.
#[poise::command(
    slash_command,
    guild_only,
    rename = "clear",
    required_permissions = "MANAGE_GUILD"
)]
pub async fn clear(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;
    let removed = db::clear_audit_log_channel(&ctx.data().pool, guild)
        .await
        .map_err(|e| format!("Couldn't clear audit-log channel: {e}"))?;
    ctx.data().audit_log_settings.write().await.remove(&guild);
    let content = if removed > 0 {
        "Audit log disabled."
    } else {
        "No audit-log channel was configured."
    };
    ctx.send(CreateReply::default().content(content).ephemeral(true))
        .await?;
    Ok(())
}

/// Show the configured audit-log channel for this guild.
#[poise::command(
    slash_command,
    guild_only,
    rename = "show",
    required_permissions = "MANAGE_GUILD"
)]
pub async fn show(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;
    let settings = ctx
        .data()
        .audit_log_settings
        .read()
        .await
        .get(&guild)
        .copied();
    let content = match settings {
        Some(s) => {
            let mut out = format!("Audit log: <#{}>\n\n", s.channel_id);
            for event in AuditEvent::ALL {
                let mark = if s.events.is_enabled(event) {
                    "\u{2705}"
                } else {
                    "\u{274c}"
                };
                let _ = writeln!(out, "{mark} {}", event.name());
            }
            out.push_str("\nUse `/auditlog enable` or `/auditlog disable` to change these.");
            out
        }
        None => "No audit-log channel configured.".to_string(),
    };
    ctx.send(CreateReply::default().content(content).ephemeral(true))
        .await?;
    Ok(())
}

/// Start logging an event category (or every category, if none is named).
#[poise::command(
    slash_command,
    guild_only,
    rename = "enable",
    required_permissions = "MANAGE_GUILD"
)]
pub async fn enable(
    ctx: Context<'_>,
    #[description = "Category to log; omit for all of them"] event: Option<AuditEvent>,
) -> Result<(), Error> {
    set_events(ctx, event, true).await
}

/// Stop logging an event category (or every category, if none is named).
#[poise::command(
    slash_command,
    guild_only,
    rename = "disable",
    required_permissions = "MANAGE_GUILD"
)]
pub async fn disable(
    ctx: Context<'_>,
    #[description = "Category to stop logging; omit for all of them"] event: Option<AuditEvent>,
) -> Result<(), Error> {
    set_events(ctx, event, false).await
}

/// Shared body of `/auditlog enable` and `/auditlog disable`. `None` means every category.
///
/// The filter is stored on the audit-log row, so a guild has to bind a channel first; the DB
/// write is what decides whether the binding is still there, since `/auditlog clear` can land
/// between our cache read and the update.
async fn set_events(
    ctx: Context<'_>,
    event: Option<AuditEvent>,
    enabled: bool,
) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;

    let Some(settings) = ctx
        .data()
        .audit_log_settings
        .read()
        .await
        .get(&guild)
        .copied()
    else {
        ctx.send(
            CreateReply::default()
                .content("No audit-log channel configured. Run `/auditlog set` first.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };

    let mut filter = settings.events;
    let changed = match event {
        Some(e) => usize::from(filter.set(e, enabled)),
        None => AuditEvent::ALL
            .into_iter()
            .filter(|&e| filter.set(e, enabled))
            .count(),
    };

    if changed > 0 {
        let bound = db::set_audit_log_events(&ctx.data().pool, guild, filter)
            .await
            .map_err(|e| format!("Couldn't save audit-log events: {e}"))?;
        if !bound {
            ctx.data().audit_log_settings.write().await.remove(&guild);
            ctx.send(
                CreateReply::default()
                    .content("The audit-log channel was just cleared - nothing to configure.")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
        // Only touch the entry if it's still there; a concurrent `/auditlog clear` wins.
        if let Some(cached) = ctx.data().audit_log_settings.write().await.get_mut(&guild) {
            cached.events = filter;
        }
    }

    let subject = match event {
        Some(e) => format!("**{}**", e.name()),
        None => format!("all {} categories", AuditEvent::ALL.len()),
    };
    let content = if changed == 0 {
        let state = if enabled { "enabled" } else { "disabled" };
        format!("No change: {subject} already {state}.")
    } else if enabled {
        format!("Now logging {subject}.")
    } else {
        format!("No longer logging {subject}.")
    };
    ctx.send(CreateReply::default().content(content).ephemeral(true))
        .await?;
    Ok(())
}
