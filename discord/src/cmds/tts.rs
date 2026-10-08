//! `/tts`: bring the reader into a voice channel, control it, and set how names are read.
//!
//! Speech itself needs no command once the bot has joined: see `crate::tts`. A member in the
//! bot's voice channel types in its text chat and the bot reads it out.

use std::sync::Arc;

use ::serenity::model::Timestamp;
use poise::CreateReply;
use poise::serenity_prelude as serenity;
use serenity::model::id::GuildId;
use serenity::model::permissions::Permissions;

use crate::checks::{passes_elevated, require_guild};
use crate::db;
use crate::tts::{self, JoinRefused, session::Session, text, voices};
use crate::types::{Context, Error};

/// The one public line posted in the voice channel's chat when the bot starts reading it.
const JOINED_NOTICE: &str =
    "Reading this channel's chat aloud (voice by Google Translate). `/tts leave` to stop.";

/// Text-to-speech in voice channels.
///
/// `/tts join` while you are in a voice channel, and the bot reads that channel's chat aloud for
/// the members in it, with Google Translate's voices. Subcommands: `join`, `leave`, `skip`,
/// `nickname`, `voice`.
#[poise::command(
    slash_command,
    guild_only,
    subcommands("tts_join", "tts_leave", "tts_skip", "tts_nickname", "tts_voice"),
    subcommand_required
)]
pub async fn tts(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Join your voice channel and read its chat aloud.
///
/// Messages typed in the channel's chat by members in the channel are read as "name said: text".
/// The bot leaves on `/tts leave`, when the channel empties, or after a few minutes of quiet.
#[poise::command(slash_command, guild_only, rename = "join")]
pub async fn tts_join(ctx: Context<'_>) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    let (channel, stage) = tts::voice_channel_of(ctx.serenity_context(), guild, ctx.author().id)
        .ok_or("Join a voice channel first, then run `/tts join`.")?;
    if stage {
        return Err("Text-to-speech doesn't read stage channels.".into());
    }
    let (session, joined) =
        ctx.data()
            .tts
            .start_session(guild, channel)
            .map_err(|refused| -> Error {
                match refused {
                JoinRefused::Unavailable => "Text-to-speech is switched off on this bot.".into(),
                JoinRefused::AlreadyHere => format!("I'm already reading <#{channel}>.").into(),
                JoinRefused::Elsewhere(other) => format!(
                    "I'm already reading <#{other}> in this server. Run `/tts leave` there first."
                )
                .into(),
                JoinRefused::TooSoon => {
                    "I joined or left voice here a moment ago. Try again in a few seconds.".into()
                }
            }
            })?;
    // Joining waits on Discord's voice server, which can take longer than an interaction allows.
    ctx.defer_ephemeral().await?;
    if !joined.await.unwrap_or(false) {
        return Err(format!(
            "I couldn't join <#{channel}>. Check that I can connect and speak there, and that it \
             isn't full."
        )
        .into());
    }
    if let Err(e) = session.channel.say(ctx.http(), JOINED_NOTICE).await {
        tracing::debug!("TTS join notice in {channel} failed: {e}");
    }
    reply(
        ctx,
        &format!(
            "Reading <#{channel}> aloud. Messages typed in its chat by members in the channel \
             are read as \"name said: text\"."
        ),
    )
    .await
}

/// Make the bot leave its voice channel.
#[poise::command(slash_command, guild_only, rename = "leave")]
pub async fn tts_leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    let session = in_session(ctx, guild).await?;
    session.stop();
    reply(ctx, &format!("Leaving <#{}>.", session.channel)).await
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

/// How your name is read before your messages.
///
/// Subcommands: `set`, `clear`, `show`. Setting or clearing someone else's needs Manage
/// Nicknames or this server's mod role.
#[poise::command(
    slash_command,
    guild_only,
    rename = "nickname",
    subcommands("tts_nickname_set", "tts_nickname_clear", "tts_nickname_show"),
    subcommand_required
)]
pub async fn tts_nickname(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Set the name read before your messages (or a member's, as a moderator).
#[poise::command(slash_command, guild_only, rename = "set")]
pub async fn tts_nickname_set(
    ctx: Context<'_>,
    #[description = "How the name should be read, up to 32 characters"] name: String,
    #[description = "Whose name (moderators only; default you)"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    let target = target_user(ctx, user.as_ref()).await?;
    let nickname = text::validate_nickname(&name)?;
    db::set_tts_nickname(
        &ctx.data().pool,
        guild,
        target,
        &nickname,
        ctx.author().id,
        Timestamp::now().unix_timestamp(),
    )
    .await
    .map_err(|e| format!("Couldn't save the TTS nickname: {e}"))?;
    ctx.data()
        .tts
        .cache_nickname(guild, target, Some(nickname.clone()));
    let whose = if target == ctx.author().id {
        "Your messages".to_string()
    } else {
        format!("<@{target}>'s messages")
    };
    reply(
        ctx,
        &format!("{whose} will be read as \"{nickname} said: ...\"."),
    )
    .await
}

/// Go back to reading your display name (or a member's, as a moderator).
#[poise::command(slash_command, guild_only, rename = "clear")]
pub async fn tts_nickname_clear(
    ctx: Context<'_>,
    #[description = "Whose name (moderators only; default you)"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    let target = target_user(ctx, user.as_ref()).await?;
    let removed = db::clear_tts_nickname(&ctx.data().pool, guild, target)
        .await
        .map_err(|e| format!("Couldn't clear the TTS nickname: {e}"))?;
    ctx.data().tts.cache_nickname(guild, target, None);
    let content = match (removed, target == ctx.author().id) {
        (true, true) => "TTS nickname cleared. Your display name is read again.".to_string(),
        (true, false) => format!("<@{target}>'s TTS nickname cleared."),
        (false, true) => "You have no TTS nickname.".to_string(),
        (false, false) => format!("<@{target}> has no TTS nickname."),
    };
    reply(ctx, &content).await
}

/// Show the name read before someone's messages.
#[poise::command(slash_command, guild_only, rename = "show")]
pub async fn tts_nickname_show(
    ctx: Context<'_>,
    #[description = "Whose name (default you)"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    let target = user.as_ref().map_or_else(|| ctx.author().id, |u| u.id);
    let content = match ctx.data().tts.nickname(guild, target) {
        Some(nickname) => format!("<@{target}> is read as \"{nickname}\"."),
        None => format!("<@{target}> has no TTS nickname; their display name is read."),
    };
    reply(ctx, &content).await
}

/// The voice your messages are read in, in every server.
///
/// Subcommands: `set`, `show`, `clear`.
#[poise::command(
    slash_command,
    rename = "voice",
    subcommands("tts_voice_set", "tts_voice_show", "tts_voice_clear"),
    subcommand_required
)]
pub async fn tts_voice(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Pick the voice your messages are read in.
#[poise::command(slash_command, rename = "set")]
pub async fn tts_voice_set(
    ctx: Context<'_>,
    #[description = "Language and accent"]
    #[autocomplete = "autocomplete_voice"]
    voice: String,
) -> Result<(), Error> {
    let picked = voices::find(&voice).ok_or_else(|| {
        format!("There's no voice called {voice:?}. Pick one from the list as you type.")
    })?;
    db::set_tts_voice(
        &ctx.data().pool,
        ctx.author().id,
        picked.id,
        Timestamp::now().unix_timestamp(),
    )
    .await
    .map_err(|e| format!("Couldn't save your voice: {e}"))?;
    ctx.data()
        .tts
        .cache_voice(ctx.author().id, Some(picked.id.to_string()));
    reply(
        ctx,
        &format!("Your messages will be read in {}.", picked.label),
    )
    .await
}

/// Show the voice your messages are read in.
#[poise::command(slash_command, rename = "show")]
pub async fn tts_voice_show(ctx: Context<'_>) -> Result<(), Error> {
    let tts = &ctx.data().tts;
    let voice = tts.voice_of(ctx.author().id);
    let content = if tts.voice_pick(ctx.author().id).is_some() && voice != tts.default_voice {
        format!("Your messages are read in {}.", voice.label)
    } else {
        format!(
            "Your messages are read in {}, the default. `/tts voice set` picks another.",
            voice.label
        )
    };
    reply(ctx, &content).await
}

/// Go back to the default voice.
#[poise::command(slash_command, rename = "clear")]
pub async fn tts_voice_clear(ctx: Context<'_>) -> Result<(), Error> {
    let removed = db::clear_tts_voice(&ctx.data().pool, ctx.author().id)
        .await
        .map_err(|e| format!("Couldn't clear your voice: {e}"))?;
    ctx.data().tts.cache_voice(ctx.author().id, None);
    let default = ctx.data().tts.default_voice.label;
    let content = if removed {
        format!("Voice cleared. Your messages are read in {default}, the default.")
    } else {
        format!("You hadn't picked a voice; your messages are read in {default}.")
    };
    reply(ctx, &content).await
}

/// Every voice whose id or label contains what has been typed, as `label` -> `id` choices.
async fn autocomplete_voice(
    _ctx: Context<'_>,
    partial: &str,
) -> Vec<poise::serenity_prelude::AutocompleteChoice> {
    let needle = partial.trim().to_lowercase();
    voices::VOICES
        .iter()
        .filter(|v| {
            needle.is_empty() || v.id.contains(&needle) || v.label.to_lowercase().contains(&needle)
        })
        .map(|v| poise::serenity_prelude::AutocompleteChoice::new(v.label, v.id))
        .collect()
}

/// The member a nickname command acts on: the invoker, or `user` when the invoker may manage
/// nicknames (Manage Nicknames, or the mod role).
async fn target_user(
    ctx: Context<'_>,
    user: Option<&serenity::User>,
) -> Result<serenity::UserId, Error> {
    match user {
        Some(user) if user.id != ctx.author().id => {
            crate::checks::manage_nicknames_check(ctx).await?;
            Ok(user.id)
        }
        _ => Ok(ctx.author().id),
    }
}

/// The guild's session, if the invoker may control it: they are in its channel, or they could
/// move members out of voice themselves (Move Members, or the mod role).
async fn in_session(ctx: Context<'_>, guild: GuildId) -> Result<Arc<Session>, Error> {
    let session = ctx
        .data()
        .tts
        .session(guild)
        .ok_or("I'm not in a voice channel in this server.")?;
    let here = tts::voice_channel_of(ctx.serenity_context(), guild, ctx.author().id)
        .map(|(channel, _)| channel);
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
