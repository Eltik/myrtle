use poise::CreateReply;
use poise::serenity_prelude as serenity;

use crate::db;
use crate::types::{Context, Error};

/// Configure the audit-log channel for this guild.
///
/// Subcommands: `set`, `clear`, `show`. When bound, the bot mirrors message edits, deletes,
/// and bulk deletes from every channel into the chosen log channel. Requires the Manage
/// Server permission.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "MANAGE_GUILD",
    subcommands("set", "clear", "show"),
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
    ctx.data()
        .audit_log_channels
        .write()
        .await
        .insert(guild, channel);
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
    ctx.data().audit_log_channels.write().await.remove(&guild);
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
    let content = match ctx.data().audit_log_channels.read().await.get(&guild) {
        Some(c) => format!("Audit log: <#{c}>"),
        None => "No audit-log channel configured.".to_string(),
    };
    ctx.send(CreateReply::default().content(content).ephemeral(true))
        .await?;
    Ok(())
}
