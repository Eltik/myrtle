//! `/tts`: the per-guild off switch for text-to-speech, and playback controls.
//!
//! Speech itself needs no command: see `crate::tts`. A member in a voice channel types in its
//! text chat and the bot reads it out.

use ::serenity::model::Timestamp;
use poise::CreateReply;
use serenity::model::id::{ChannelId, GuildId};
use serenity::model::permissions::Permissions;

use crate::checks::{passes_elevated, require_guild};
use crate::db;
use crate::tts::{self, session::Session};
use crate::types::{Context, Error};

/// Text-to-speech in voice channels.
///
/// Type in a voice channel's chat while you are in it and the bot reads it out. Subcommands:
/// `enable`, `disable`, `status` (Manage Server), `skip`, `leave`.
#[poise::command(
    slash_command,
    guild_only,
    subcommands("tts_enable", "tts_disable", "tts_status", "tts_skip", "tts_leave"),
    subcommand_required
)]
pub async fn tts(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Turn text-to-speech back on in this server.
#[poise::command(
    slash_command,
    guild_only,
    rename = "enable",
    check = "crate::checks::manage_guild_check"
)]
pub async fn tts_enable(ctx: Context<'_>) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    let was_off = db::clear_tts_disabled(&ctx.data().pool, guild)
        .await
        .map_err(|e| format!("Couldn't save the TTS setting: {e}"))?;
    ctx.data().tts.set_disabled(guild, false).await;
    let content = if was_off {
        "Text-to-speech is on. Members in a voice channel are read out when they type in its chat."
    } else {
        "Text-to-speech was already on."
    };
    reply(ctx, content).await
}

/// Turn text-to-speech off in this server. The bot leaves voice if it is speaking.
#[poise::command(
    slash_command,
    guild_only,
    rename = "disable",
    check = "crate::checks::manage_guild_check"
)]
pub async fn tts_disable(ctx: Context<'_>) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    let was_off = ctx.data().tts.is_disabled(guild).await;
    db::set_tts_disabled(&ctx.data().pool, guild, Timestamp::now().unix_timestamp())
        .await
        .map_err(|e| format!("Couldn't save the TTS setting: {e}"))?;
    ctx.data().tts.set_disabled(guild, true).await;
    let content = if was_off {
        "Text-to-speech was already off."
    } else {
        "Text-to-speech is off. `/tts enable` turns it back on."
    };
    reply(ctx, content).await
}

/// Whether text-to-speech is on here, and where the bot is speaking.
#[poise::command(
    slash_command,
    guild_only,
    rename = "status",
    check = "crate::checks::manage_guild_check"
)]
pub async fn tts_status(ctx: Context<'_>) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    let data = ctx.data();
    let disabled_at = db::get_tts_disabled(&data.pool, guild)
        .await
        .map_err(|e| format!("Couldn't read the TTS setting: {e}"))?;
    let mut lines = vec![match disabled_at {
        Some(at) => format!("Text-to-speech is **off** in this server, since <t:{at}:f>."),
        None => "Text-to-speech is **on** in this server.".to_string(),
    }];
    if !data.tts.synth.available() {
        lines.push("The voice isn't installed on the bot, so nothing is spoken anywhere.".into());
    }
    if let Some(session) = data.tts.session(guild) {
        lines.push(format!(
            "Speaking in <#{}>, {} message(s) waiting.",
            session.channel,
            session.queued()
        ));
    }
    reply(ctx, &lines.join("\n")).await
}

/// Skip the message being read out.
#[poise::command(slash_command, guild_only, rename = "skip")]
pub async fn tts_skip(ctx: Context<'_>) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    let session = in_session(ctx, guild).await?;
    let content = if session.skip() {
        "Skipped."
    } else {
        "Nothing is being read out right now."
    };
    reply(ctx, content).await
}

/// Make the bot leave its voice channel.
#[poise::command(slash_command, guild_only, rename = "leave")]
pub async fn tts_leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    let session = in_session(ctx, guild).await?;
    session.stop();
    reply(ctx, &format!("Leaving <#{}>.", session.channel)).await
}

/// The guild's session, if the invoker may control it: they are in its channel, or they could
/// move members out of voice themselves (Move Members, or the mod role).
async fn in_session(ctx: Context<'_>, guild: GuildId) -> Result<std::sync::Arc<Session>, Error> {
    let session = ctx
        .data()
        .tts
        .session(guild)
        .ok_or("I'm not in a voice channel in this server.")?;
    let here: Option<ChannelId> =
        tts::voice_channel_of(ctx.serenity_context(), guild, ctx.author().id);
    if here == Some(session.channel) || passes_elevated(ctx, Permissions::MOVE_MEMBERS).await? {
        Ok(session)
    } else {
        Err(format!(
            "Join <#{}> to use this, or ask someone with the Move Members permission.",
            session.channel
        )
        .into())
    }
}

async fn reply(ctx: Context<'_>, content: &str) -> Result<(), Error> {
    ctx.send(CreateReply::default().content(content).ephemeral(true))
        .await?;
    Ok(())
}
