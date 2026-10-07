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

/// Autocomplete a server label from the configured asset servers.
async fn autocomplete_server(ctx: Context<'_>, partial: &str) -> Vec<String> {
    let mut labels: Vec<String> = ctx
        .data()
        .assets
        .keys()
        .filter(|l| l.to_lowercase().starts_with(&partial.trim().to_lowercase()))
        .cloned()
        .collect();
    labels.sort_unstable();
    labels
}

/// Parse a comma list of server labels ("EN, jp") against the configured servers.
///
/// Returns the labels in their configured spelling, deduplicated, in configured-label order.
/// Errors name the unknown label and list the valid ones.
fn parse_server_filter(ctx: &Context<'_>, raw: &str) -> Result<Vec<String>, Error> {
    let mut configured: Vec<&String> = ctx.data().assets.keys().collect();
    configured.sort_unstable();
    let mut wanted: Vec<String> = Vec::new();
    for token in raw.split(',').map(str::trim).filter(|t| !t.is_empty()) {
        let Some(label) = configured.iter().find(|l| l.eq_ignore_ascii_case(token)) else {
            let list: Vec<&str> = configured.iter().map(|l| l.as_str()).collect();
            return Err(format!(
                "Unknown server `{token}`. Configured: {}",
                if list.is_empty() {
                    "(none configured)".to_string()
                } else {
                    list.join(", ")
                }
            )
            .into());
        };
        if !wanted.contains(label) {
            wanted.push((*label).clone());
        }
    }
    if wanted.is_empty() {
        return Err("`servers` needs at least one server label, e.g. `EN,JP`.".into());
    }
    wanted.sort_unstable();
    Ok(wanted)
}

/// Manage the Arknights asset-pipeline integration.
///
/// Subcommands: `channel`, `status`, `resources`. The `channel` group binds
/// announcements to a specific channel; the pipeline daemon (`assets/run.mjs ws`) emits
/// version/error events which the bot forwards as embeds.
///
/// Bot-owner only: this drives the export pipeline the bot operators run, not anything a
/// server configures for itself, so neither the server owner nor the mod role opens it.
///
/// The check sits on this parent command, which is enough: poise runs every parent's checks
/// before the invoked subcommand's, so the whole tree is gated here.
#[poise::command(
    slash_command,
    guild_only,
    check = "crate::checks::owner_check",
    subcommands("assets_channel", "assets_status", "assets_resources"),
    subcommand_required
)]
pub async fn assets(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Configure which channel receives asset announcements for this guild.
///
/// Subcommands: `set`, `clear`, `show`. Server-owner only, via the `/assets` check.
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
///
/// `servers` limits the announcements to some regions (comma list, e.g. `EN,JP`). Leave it out
/// to hear from every configured server; re-running `set` replaces both the channel and the list.
#[poise::command(slash_command, guild_only, rename = "set")]
pub async fn assets_channel_set(
    ctx: Context<'_>,
    #[description = "Channel to announce updates in"] channel: serenity::ChannelId,
    #[description = "Only these servers, comma-separated (e.g. EN,JP). Omit for all."]
    servers: Option<String>,
) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;
    let filter = servers
        .as_deref()
        .map(|raw| parse_server_filter(&ctx, raw))
        .transpose()?;
    db::set_assets_channel(&ctx.data().pool, guild, channel, filter.as_deref())
        .await
        .map_err(|e| format!("Couldn't save assets channel: {e}"))?;
    let scope = filter.map_or_else(
        || "every server".to_string(),
        |f| format!("{} only", f.join(", ")),
    );
    ctx.send(
        CreateReply::default()
            .content(format!(
                "Asset announcements will be sent to <#{channel}>, from {scope}."
            ))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Stop sending asset announcements for this guild.
#[poise::command(slash_command, guild_only, rename = "clear")]
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
#[poise::command(slash_command, guild_only, rename = "show")]
pub async fn assets_channel_show(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;
    let content = match db::get_assets_channel(&ctx.data().pool, guild)
        .await
        .map_err(|e| format!("Couldn't read assets channel: {e}"))?
    {
        Some(binding) => format!(
            "Asset announcements: <#{}>, from {}.",
            binding.channel_id,
            binding.servers.map_or_else(
                || "every server".to_string(),
                |s| format!("{} only", s.join(", "))
            )
        ),
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
    #[description = "Server label (EN, CN, JP, KR). Omit for all."]
    #[autocomplete = "autocomplete_server"]
    server: Option<String>,
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
#[poise::command(slash_command, guild_only, rename = "resources")]
pub async fn assets_resources(
    ctx: Context<'_>,
    #[description = "Server label (EN, CN, JP, KR)"]
    #[autocomplete = "autocomplete_server"]
    server: String,
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
