use std::fmt::Write as _;
use std::sync::Arc;
use std::time::Duration;

use poise::CreateReply;
use poise::serenity_prelude as serenity;
use serde_json::json;
use tokio::sync::oneshot;

use crate::db;
use crate::types::{Context, Error};
use crate::watcher::AssetsState;

/// Resolve a server label (case-insensitive) to its `AssetsState`, or error with
/// the list of configured servers.
fn assets_state_for(ctx: &Context<'_>, server: &str) -> Result<Arc<AssetsState>, Error> {
    let states = &ctx.data().assets;
    if let Some(s) = states.get(server) {
        return Ok(s.clone());
    }
    if let Some((_, s)) = states.iter().find(|(k, _)| k.eq_ignore_ascii_case(server)) {
        return Ok(s.clone());
    }
    let mut labels: Vec<&str> = states.keys().map(String::as_str).collect();
    labels.sort_unstable();
    let list = if labels.is_empty() {
        "(none configured)".to_string()
    } else {
        labels.join(", ")
    };
    Err(format!("Unknown server `{server}`. Configured: {list}").into())
}

/// Manage the Arknights asset-pipeline integration.
///
/// Subcommands: `channel`, `status`, `resources`. The `channel` group binds
/// announcements to a specific channel; the pipeline daemon (`assets/run.mjs ws`) emits
/// version/error events which the bot forwards as embeds.
#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "MANAGE_GUILD",
    subcommands("assets_channel", "assets_status", "assets_resources"),
    subcommand_required
)]
pub async fn assets(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Configure which channel receives asset announcements for this guild.
///
/// Subcommands: `set`, `clear`, `show`. Requires the Manage Server permission.
#[poise::command(
    slash_command,
    guild_only,
    rename = "channel",
    subcommands("assets_channel_set", "assets_channel_clear", "assets_channel_show"),
    subcommand_required
)]
pub async fn assets_channel(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Bind asset announcements to a channel.
#[poise::command(
    slash_command,
    guild_only,
    rename = "set",
    required_permissions = "MANAGE_GUILD"
)]
pub async fn assets_channel_set(
    ctx: Context<'_>,
    #[description = "Channel to announce updates in"] channel: serenity::ChannelId,
) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;
    db::set_assets_channel(&ctx.data().pool, guild, channel)
        .await
        .map_err(|e| format!("Couldn't save assets channel: {e}"))?;
    ctx.send(
        CreateReply::default()
            .content(format!("Asset announcements will be sent to <#{channel}>."))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Stop sending asset announcements for this guild.
#[poise::command(
    slash_command,
    guild_only,
    rename = "clear",
    required_permissions = "MANAGE_GUILD"
)]
pub async fn assets_channel_clear(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;
    let removed = db::clear_assets_channel(&ctx.data().pool, guild)
        .await
        .map_err(|e| format!("Couldn't clear assets channel: {e}"))?;
    let content = if removed > 0 {
        "Asset announcements disabled."
    } else {
        "No asset channel was configured."
    };
    ctx.send(CreateReply::default().content(content).ephemeral(true))
        .await?;
    Ok(())
}

/// Show the configured asset-announcements channel for this guild.
#[poise::command(
    slash_command,
    guild_only,
    rename = "show",
    required_permissions = "MANAGE_GUILD"
)]
pub async fn assets_channel_show(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;
    let content = match db::get_assets_channel(&ctx.data().pool, guild)
        .await
        .map_err(|e| format!("Couldn't read assets channel: {e}"))?
    {
        Some(c) => format!("Asset announcements: <#{c}>"),
        None => "No asset channel configured.".to_string(),
    };
    ctx.send(CreateReply::default().content(content).ephemeral(true))
        .await?;
    Ok(())
}

/// Show the last-known state of the asset pipeline(s), mirrored from WS events.
#[poise::command(slash_command, guild_only, rename = "status")]
pub async fn assets_status(
    ctx: Context<'_>,
    #[description = "Server label (e.g. EN, CN). Omit for all."] server: Option<String>,
) -> Result<(), Error> {
    let states = &ctx.data().assets;
    if states.is_empty() {
        ctx.send(
            CreateReply::default()
                .content("No asset pipelines are configured.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let mut labels: Vec<&String> = states.keys().collect();
    labels.sort();

    let mut body = String::new();
    for label in labels {
        if let Some(ref want) = server
            && !label.eq_ignore_ascii_case(want)
        {
            continue;
        }
        let snapshot = states
            .get(label)
            .expect("label came from keys()")
            .status
            .read()
            .await
            .clone();
        let state = snapshot.state.as_deref().unwrap_or("disconnected");
        let version = snapshot.current_version.as_deref().unwrap_or("(unknown)");
        let _ = writeln!(body, "**{label}** — state `{state}`, version `{version}`");
    }

    if body.is_empty() {
        return Err(format!("Unknown server `{}`.", server.unwrap_or_default()).into());
    }

    ctx.send(CreateReply::default().content(body).ephemeral(true))
        .await?;
    Ok(())
}

/// Ask the asset pipeline for its current resource listing and reply with a summary.
#[poise::command(
    slash_command,
    guild_only,
    rename = "resources",
    check = "crate::checks::owner_check"
)]
pub async fn assets_resources(
    ctx: Context<'_>,
    #[description = "Server label (e.g. EN, CN)"] server: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let state = assets_state_for(&ctx, &server)?;

    let (tx, rx) = oneshot::channel();
    *state.pending_resource_list.lock().await = Some(tx);

    let req = serde_json::to_string(&json!({ "type": "list_resources" }))?;
    state
        .tx
        .send(req)
        .map_err(|_| "Asset watcher channel is closed.")?;

    let payload = match tokio::time::timeout(Duration::from_secs(10), rx).await {
        Ok(Ok(p)) => p,
        Ok(Err(_)) => return Err("Resource-list listener was cancelled.".into()),
        Err(_) => {
            state.pending_resource_list.lock().await.take();
            return Err("Timed out waiting for the asset pipeline to respond.".into());
        }
    };

    let total = payload
        .total_size_formatted
        .clone()
        .unwrap_or_else(|| payload.total_size.to_string());
    let mut body = format!(
        "**Resources:** {} entries, total **{}**\n\n",
        payload.files.len(),
        total
    );
    for f in payload.files.iter().take(25) {
        let _ = writeln!(body, "• `{}` ({}) — {} B", f.name, f.kind, f.size);
    }
    if payload.files.len() > 25 {
        let _ = writeln!(body, "…and {} more", payload.files.len() - 25);
    }

    let reply = if body.len() <= 2000 {
        CreateReply::default().content(body)
    } else {
        CreateReply::default().attachment(serenity::CreateAttachment::bytes(
            body.into_bytes(),
            "resources.txt",
        ))
    };
    ctx.send(reply.ephemeral(true)).await?;
    Ok(())
}
