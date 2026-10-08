//! Text-to-speech in voice channels.
//!
//! A member in voice channel X who types in X's built-in text chat is read out in X: the bot
//! joins if it is in no other channel of the guild, speaks each message in order, and leaves when
//! the channel has no humans left or after [`TtsConfig::idle_secs`] of quiet. It is on in every
//! guild unless a moderator runs `/tts disable`.
//!
//! [`gate`] decides whether a message is spoken, [`text`] turns it into words, [`engine`] makes
//! the audio, and [`session`] holds the guild's place in the channel.

pub mod engine;
pub mod gate;
pub mod session;
pub mod text;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use serenity::all::{ChannelType, Context, Guild, Message, MessageType, VoiceState};
use serenity::model::id::{ChannelId, GuildId, RoleId, UserId};
use songbird::Songbird;
use tokio::sync::RwLock;

use crate::config::TtsConfig;
use engine::{ModelPaths, Synth};
use gate::{Facts, RateLimiter, Verdict};
use session::Session;
use text::Mention;

/// Least time between two voice joins in one guild, counted from the last join or leave.
///
/// Each join and each leave is one gateway send, and a shard may make 120 of those a minute;
/// this holds one guild to 12 joins and 12 leaves a minute however its members come and go.
pub const JOIN_COOLDOWN: Duration = Duration::from_secs(5);

/// How long the bot's channel must stay without humans before it leaves, so a member dropping
/// and rejoining doesn't cost a leave and a join.
const EMPTY_GRACE: Duration = Duration::from_secs(5);

/// How long a voice channel the bot failed to join is left alone. A join with no Connect
/// permission, or into a full channel, is ignored by Discord and only times out after 10 s; without
/// this every few seconds of chat there would cost a gateway send and a warning.
const JOIN_FAILURE_BACKOFF: Duration = Duration::from_mins(5);

/// The bot's prefix-command marker; a message starting with it is a command, not speech.
const PREFIX: &str = "-";

/// The TTS subsystem: config, the voice, per-guild sessions and the per-guild off switch.
pub struct Tts {
    pub config: TtsConfig,
    pub synth: Synth,
    songbird: Arc<Songbird>,
    /// Guilds where a moderator turned TTS off, mirroring `guild_tts_disabled`. Hydrated at
    /// startup and kept in sync by `/tts enable` and `/tts disable`, so the message hot path never
    /// reads `SQLite`.
    disabled: RwLock<HashSet<GuildId>>,
    sessions: Mutex<HashMap<GuildId, Arc<Session>>>,
    members: Mutex<RateLimiter<(GuildId, UserId)>>,
    joins: Mutex<RateLimiter<GuildId>>,
    /// The last channel per guild the bot failed to join, and when.
    failed_joins: Mutex<HashMap<GuildId, (ChannelId, Instant)>>,
}

impl Tts {
    /// Set TTS up from `config`. Finds the voice's files now, so a missing model is one warning at
    /// startup; the model itself loads on the first message that needs it.
    #[must_use]
    pub fn new(config: TtsConfig, songbird: Arc<Songbird>) -> Self {
        let paths = if config.enabled {
            match ModelPaths::resolve(&config) {
                Ok(paths) => {
                    tracing::info!("TTS voice found at {}", paths.model.display());
                    Some(paths)
                }
                Err(e) => {
                    tracing::warn!("TTS disabled: {e}");
                    None
                }
            }
        } else {
            tracing::info!("TTS disabled in config");
            None
        };
        let cooldown = Duration::from_millis(config.user_cooldown_ms);
        Self {
            config,
            synth: Synth::new(paths),
            songbird,
            disabled: RwLock::new(HashSet::new()),
            sessions: Mutex::new(HashMap::new()),
            members: Mutex::new(RateLimiter::new(cooldown)),
            joins: Mutex::new(RateLimiter::new(JOIN_COOLDOWN)),
            failed_joins: Mutex::new(HashMap::new()),
        }
    }

    /// Replace the cached off-switch set with `guilds`, read from the database at startup.
    pub async fn hydrate_disabled(&self, guilds: impl IntoIterator<Item = GuildId>) {
        *self.disabled.write().await = guilds.into_iter().collect();
    }

    /// Mirror a `/tts enable` or `/tts disable`. Disabling also ends the guild's session.
    pub async fn set_disabled(&self, guild: GuildId, disabled: bool) {
        if disabled {
            self.disabled.write().await.insert(guild);
            if let Some(session) = self.session(guild) {
                session.stop();
            }
        } else {
            self.disabled.write().await.remove(&guild);
        }
    }

    pub async fn is_disabled(&self, guild: GuildId) -> bool {
        self.disabled.read().await.contains(&guild)
    }

    /// The guild's live session, if the bot is in (or joining) one of its voice channels.
    #[must_use]
    pub fn session(&self, guild: GuildId) -> Option<Arc<Session>> {
        lock(&self.sessions).get(&guild).cloned()
    }

    /// Start a session in `channel`, unless the guild joined or left too recently or another
    /// message started one first. Returns the session to queue on, if any.
    fn start_session(self: &Arc<Self>, guild: GuildId, channel: ChannelId) -> Option<Arc<Session>> {
        let session = {
            let mut sessions = lock(&self.sessions);
            if let Some(existing) = sessions.get(&guild) {
                return (existing.channel == channel).then(|| Arc::clone(existing));
            }
            // `/tts disable` may have landed since the caller read the set; it holds the write
            // lock while it runs, which also reads as disabled.
            if self
                .disabled
                .try_read()
                .map_or(true, |d| d.contains(&guild))
            {
                return None;
            }
            if lock(&self.failed_joins)
                .get(&guild)
                .is_some_and(|&(c, at)| c == channel && at.elapsed() < JOIN_FAILURE_BACKOFF)
            {
                return None;
            }
            if !lock(&self.joins).allow(guild, Instant::now()) {
                tracing::debug!("TTS join in guild {guild} skipped: joined or left too recently");
                return None;
            }
            let session = Arc::new(Session::new(guild, channel, self.config.queue_max));
            sessions.insert(guild, Arc::clone(&session));
            session
        };
        tracing::info!("TTS joining {channel} in guild {guild}");
        tokio::spawn(session::run(Arc::clone(self), Arc::clone(&session)));
        Some(session)
    }

    /// Remember that joining `channel` failed, so it isn't retried for [`JOIN_FAILURE_BACKOFF`].
    fn join_failed(&self, guild: GuildId, channel: ChannelId) {
        lock(&self.failed_joins).insert(guild, (channel, Instant::now()));
    }

    /// Deregister `session` once its worker has left. A newer session for the guild is kept.
    fn end_session(&self, session: &Arc<Session>) {
        let mut sessions = lock(&self.sessions);
        if sessions
            .get(&session.guild)
            .is_some_and(|s| Arc::ptr_eq(s, session))
        {
            sessions.remove(&session.guild);
        }
        drop(sessions);
        lock(&self.joins).touch(session.guild, Instant::now());
    }
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Speak `msg` if it was typed in the text chat of the voice channel its author is in.
///
/// Every refusal is silent. A message that passes every check but finds the queue full is
/// dropped, also silently. Order across members holds from the queue on: serenity runs each
/// event in its own task, so two messages sent milliseconds apart can reach the queue swapped.
pub async fn on_message(tts: &Arc<Tts>, ctx: &Context, bot_id: UserId, msg: &Message) {
    let Some(guild_id) = msg.guild_id else {
        return;
    };
    let automated = msg.author.bot
        || msg.author.id == bot_id
        || msg.webhook_id.is_some()
        || !matches!(msg.kind, MessageType::Regular | MessageType::InlineReply);
    // The cheapest refusals first: most messages are in text channels and never reach the cache.
    if automated || !tts.synth.available() {
        return;
    }
    let guild_disabled = tts.is_disabled(guild_id).await;
    let bot_voice = tts.session(guild_id).map(|s| s.channel);

    // Everything read from the cache happens inside this block: the guild handle is a map guard
    // and must not be held across an await.
    let spoken = {
        let Some(guild) = ctx.cache.guild(guild_id) else {
            return;
        };
        let facts = Facts {
            automated,
            prefixed: msg.content.trim_start().starts_with(PREFIX),
            has_stickers: !msg.sticker_items.is_empty(),
            guild_disabled,
            engine_available: tts.synth.available(),
            channel: msg.channel_id,
            channel_is_voice: guild
                .channels
                .get(&msg.channel_id)
                .is_some_and(|c| c.kind == ChannelType::Voice),
            author_voice: guild
                .voice_states
                .get(&msg.author.id)
                .and_then(|v| v.channel_id),
            bot_voice,
        };
        let verdict = gate::decide(&facts);
        if let Verdict::Skip(why) = verdict {
            if facts.channel_is_voice {
                tracing::trace!("TTS skipped message {}: {why:?}", msg.id);
            }
            return;
        }
        let resolve = |m: Mention| mention_name(&guild, msg, m);
        let Some(words) = text::clean(&msg.content, &resolve, tts.config.max_chars) else {
            return;
        };
        (verdict, words)
    };
    let (verdict, words) = spoken;

    if !lock(&tts.members).allow((guild_id, msg.author.id), Instant::now()) {
        return;
    }
    let session = match verdict {
        Verdict::JoinAndSpeak => tts.start_session(guild_id, msg.channel_id),
        _ => tts
            .session(guild_id)
            .filter(|s| s.channel == msg.channel_id),
    };
    if let Some(session) = session
        && !session.push(words)
    {
        tracing::debug!(
            "TTS queue full in guild {guild_id}; dropped message {}",
            msg.id
        );
    }
}

/// The spoken name for a mention in `msg`: a member's server nickname, else their display name;
/// a role's or channel's name. `None` when the cache doesn't have it.
fn mention_name(guild: &Guild, msg: &Message, m: Mention) -> Option<String> {
    match m {
        Mention::User(id) => {
            let id = UserId::new(id);
            if let Some(member) = guild.members.get(&id) {
                return Some(member.display_name().to_string());
            }
            let user = msg.mentions.iter().find(|u| u.id == id)?;
            Some(
                user.member
                    .as_ref()
                    .and_then(|m| m.nick.clone())
                    .unwrap_or_else(|| user.display_name().to_string()),
            )
        }
        Mention::Role(id) => guild.roles.get(&RoleId::new(id)).map(|r| r.name.clone()),
        Mention::Channel(id) => guild
            .channels
            .get(&ChannelId::new(id))
            .map(|c| c.name.clone()),
    }
}

/// Follow a voice-state change: end the session when the bot is moved or disconnected, and
/// leave once the bot's channel has had no humans for [`EMPTY_GRACE`].
pub fn on_voice_state(tts: &Arc<Tts>, ctx: &Context, bot_id: UserId, new: &VoiceState) {
    let Some(guild) = new.guild_id else {
        return;
    };
    let Some(session) = tts.session(guild) else {
        return;
    };
    if new.user_id == bot_id {
        // Before the join is confirmed, a bot update may be a late replay from the guild's last
        // session (its leave) and says nothing about this one.
        if !session.is_joined() {
            return;
        }
        if new.channel_id.is_some_and(|c| c != session.channel) {
            tracing::info!("TTS moved out of {} in guild {guild}", session.channel);
            session.stop();
        } else if new.channel_id.is_none() && !session.is_stopping() {
            tracing::info!("TTS disconnected from {} in guild {guild}", session.channel);
            session.stop();
        }
        return;
    }
    if humans_in(ctx, guild, session.channel, bot_id) > 0
        || session
            .empty_check
            .swap(true, std::sync::atomic::Ordering::Relaxed)
    {
        return;
    }
    let ctx = ctx.clone();
    tokio::spawn(async move {
        tokio::time::sleep(EMPTY_GRACE).await;
        session
            .empty_check
            .store(false, std::sync::atomic::Ordering::Relaxed);
        if humans_in(&ctx, guild, session.channel, bot_id) == 0 {
            session.stop();
        }
    });
}

/// Humans in `channel`, from the cache's voice states. An uncached guild counts as occupied, so
/// a cache gap never makes the bot leave.
fn humans_in(ctx: &Context, guild: GuildId, channel: ChannelId, bot_id: UserId) -> usize {
    let Some(guild) = ctx.cache.guild(guild) else {
        return 1;
    };
    guild
        .voice_states
        .values()
        .filter(|v| v.channel_id == Some(channel) && v.user_id != bot_id)
        .filter(|v| {
            let bot = v
                .member
                .as_ref()
                .map(|m| m.user.bot)
                .or_else(|| guild.members.get(&v.user_id).map(|m| m.user.bot));
            !bot.unwrap_or(false)
        })
        .count()
}

/// The voice channel `user` is in, from the cache.
#[must_use]
pub fn voice_channel_of(ctx: &Context, guild: GuildId, user: UserId) -> Option<ChannelId> {
    ctx.cache
        .guild(guild)?
        .voice_states
        .get(&user)
        .and_then(|v| v.channel_id)
}
