//! Text-to-speech in voice channels.
//!
//! `/tts join` puts the bot in the invoker's voice channel. From then on, a member in that
//! channel who types in its built-in text chat is read out there, as "<name> said: <text>", in
//! the voice they picked with `/tts voice`. The bot leaves on `/tts leave`, when the channel has
//! no humans left, or after [`TtsConfig::idle_secs`] of quiet. It never joins on its own.
//!
//! The audio comes from Google Translate's speech endpoint, so every message read aloud is sent
//! to Google.
//!
//! [`gate`] decides whether a message is spoken, [`text`] turns it into words, [`engine`] fetches
//! the audio, [`voices`] lists the voices, and [`session`] holds the guild's place in the channel.

pub mod engine;
pub mod gate;
pub mod session;
pub mod text;
pub mod voices;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use serenity::all::{Context, Guild, Message, MessageType, VoiceState};
use serenity::model::id::{ChannelId, GuildId, RoleId, UserId};
use songbird::Songbird;
use tokio::sync::oneshot;

use crate::config::TtsConfig;
use engine::Google;
use gate::{Facts, RateLimiter};
use session::{Session, Utterance};
use text::{Mention, NameSources};
use voices::Voice;

/// Least time between two voice joins in one guild, counted from the last join or leave.
///
/// Each join and each leave is one gateway send, and a shard may make 120 of those a minute;
/// this holds one guild to 12 joins and 12 leaves a minute however often `/tts join` is run.
pub const JOIN_COOLDOWN: Duration = Duration::from_secs(5);

/// How long the bot's channel must stay without humans before it leaves, so a member dropping
/// and rejoining doesn't end the session.
const EMPTY_GRACE: Duration = Duration::from_secs(5);

/// The bot's prefix-command marker; a message starting with it is a command, not speech.
const PREFIX: &str = "-";

/// Why `/tts join` didn't start a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinRefused {
    /// TTS is switched off in the config.
    Unavailable,
    /// The bot is already reading this channel.
    AlreadyHere,
    /// The bot is reading another channel of the guild.
    Elsewhere(ChannelId),
    /// The guild joined or left voice less than [`JOIN_COOLDOWN`] ago.
    TooSoon,
}

/// The TTS subsystem: config, the Google client, per-guild sessions, TTS nicknames and voices.
pub struct Tts {
    pub config: TtsConfig,
    pub google: Google,
    /// The voice for members who haven't picked one: `tts.default_voice`, or [`voices::FALLBACK`].
    pub default_voice: Voice,
    songbird: Arc<Songbird>,
    sessions: Mutex<HashMap<GuildId, Arc<Session>>>,
    /// TTS nicknames, mirroring `guild_tts_nicknames`. Hydrated at startup and kept in sync by
    /// `/tts nickname`, so the message hot path never reads `SQLite`.
    nicknames: Mutex<HashMap<(GuildId, UserId), String>>,
    /// Voice picks, mirroring `user_tts_voice`, kept in sync by `/tts voice`. Global per user.
    voice_picks: Mutex<HashMap<UserId, String>>,
    joins: Mutex<RateLimiter<GuildId>>,
}

impl Tts {
    /// Set TTS up from `config`, fetching through the bot's shared `http` client. Warns once
    /// about deprecated config fields and an unknown default voice.
    #[must_use]
    pub fn new(config: TtsConfig, songbird: Arc<Songbird>, http: reqwest::Client) -> Self {
        let deprecated = config.deprecated_fields();
        if !deprecated.is_empty() {
            tracing::warn!(
                "Ignoring deprecated tts config field(s): {}. Speech comes from Google Translate \
                 now; per-member limits are tts.user_queue_max",
                deprecated.join(", ")
            );
        }
        let default_voice = match config.default_voice.as_deref() {
            None => voices::FALLBACK,
            Some(id) => voices::find(id).unwrap_or_else(|| {
                tracing::warn!(
                    "tts.default_voice {id:?} is not a known voice; using {}",
                    voices::FALLBACK.id
                );
                voices::FALLBACK
            }),
        };
        if !config.enabled() {
            tracing::info!("TTS disabled in config");
        }
        Self {
            config,
            google: Google::new(http),
            default_voice,
            songbird,
            sessions: Mutex::new(HashMap::new()),
            nicknames: Mutex::new(HashMap::new()),
            voice_picks: Mutex::new(HashMap::new()),
            joins: Mutex::new(RateLimiter::new(JOIN_COOLDOWN)),
        }
    }

    /// Replace the cached voice picks with `rows`, read from the database at startup.
    pub fn hydrate_voices(&self, rows: impl IntoIterator<Item = (UserId, String)>) {
        *lock(&self.voice_picks) = rows.into_iter().collect();
    }

    /// Mirror a `/tts voice set` (`Some`) or `clear` (`None`).
    pub fn cache_voice(&self, user: UserId, voice: Option<String>) {
        let mut picks = lock(&self.voice_picks);
        match voice {
            Some(v) => picks.insert(user, v),
            None => picks.remove(&user),
        };
    }

    /// The voice `user` speaks in: their pick if it is still a known voice, else the default.
    #[must_use]
    pub fn voice_of(&self, user: UserId) -> Voice {
        voices::resolve(
            lock(&self.voice_picks).get(&user).map(String::as_str),
            self.default_voice,
        )
    }

    /// `user`'s stored pick, as stored (it may name a voice that no longer exists).
    #[must_use]
    pub fn voice_pick(&self, user: UserId) -> Option<String> {
        lock(&self.voice_picks).get(&user).cloned()
    }

    /// Replace the cached TTS nicknames with `rows`, read from the database at startup.
    pub fn hydrate_nicknames(&self, rows: impl IntoIterator<Item = (GuildId, UserId, String)>) {
        *lock(&self.nicknames) = rows.into_iter().map(|(g, u, n)| ((g, u), n)).collect();
    }

    /// Mirror a `/tts nickname set` (`Some`) or `clear` (`None`).
    pub fn cache_nickname(&self, guild: GuildId, user: UserId, nickname: Option<String>) {
        let mut nicknames = lock(&self.nicknames);
        match nickname {
            Some(n) => nicknames.insert((guild, user), n),
            None => nicknames.remove(&(guild, user)),
        };
    }

    /// `user`'s TTS nickname in `guild`, from the cache.
    #[must_use]
    pub fn nickname(&self, guild: GuildId, user: UserId) -> Option<String> {
        lock(&self.nicknames).get(&(guild, user)).cloned()
    }

    /// The guild's live session, if the bot is in (or joining) one of its voice channels.
    #[must_use]
    pub fn session(&self, guild: GuildId) -> Option<Arc<Session>> {
        lock(&self.sessions).get(&guild).cloned()
    }

    /// Start reading `channel` (`/tts join`). On success the receiver says whether the voice
    /// connection came up; the session ends itself if it didn't.
    pub fn start_session(
        self: &Arc<Self>,
        guild: GuildId,
        channel: ChannelId,
    ) -> Result<(Arc<Session>, oneshot::Receiver<bool>), JoinRefused> {
        if !self.config.enabled() {
            return Err(JoinRefused::Unavailable);
        }
        let session = {
            let mut sessions = lock(&self.sessions);
            if let Some(existing) = sessions.get(&guild) {
                return Err(if existing.channel == channel {
                    JoinRefused::AlreadyHere
                } else {
                    JoinRefused::Elsewhere(existing.channel)
                });
            }
            if !lock(&self.joins).allow(guild, Instant::now()) {
                return Err(JoinRefused::TooSoon);
            }
            let session = Arc::new(Session::new(
                guild,
                channel,
                self.config.queue_max(),
                self.config.user_queue_max(),
            ));
            sessions.insert(guild, Arc::clone(&session));
            session
        };
        tracing::info!("TTS joining {channel} in guild {guild}");
        let (tx, rx) = oneshot::channel();
        tokio::spawn(session::run(Arc::clone(self), Arc::clone(&session), tx));
        Ok((session, rx))
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

/// Speak `msg` if it was typed in the chat of the voice channel the bot is reading, by a member
/// who is in that channel.
///
/// Every refusal is silent. A message that passes every check but finds the queue full is
/// dropped, also silently. Order across members holds from the queue on: serenity runs each
/// event in its own task, so two messages sent milliseconds apart can reach the queue swapped.
pub fn on_message(tts: &Arc<Tts>, ctx: &Context, bot_id: UserId, msg: &Message) {
    let Some(guild_id) = msg.guild_id else {
        return;
    };
    // The cheapest refusal first: in a guild the bot isn't reading, nothing else is looked at.
    let Some(session) = tts.session(guild_id) else {
        return;
    };
    let automated = msg.author.bot
        || msg.author.id == bot_id
        || msg.webhook_id.is_some()
        || !matches!(msg.kind, MessageType::Regular | MessageType::InlineReply);

    let spoken = {
        let Some(guild) = ctx.cache.guild(guild_id) else {
            return;
        };
        let facts = Facts {
            automated,
            prefixed: msg.content.trim_start().starts_with(PREFIX),
            has_stickers: !msg.sticker_items.is_empty(),
            channel: msg.channel_id,
            author_voice: guild
                .voice_states
                .get(&msg.author.id)
                .and_then(|v| v.channel_id),
            bot_voice: Some(session.channel),
        };
        if let Err(why) = gate::decide(&facts) {
            if msg.channel_id == session.channel {
                tracing::trace!("TTS skipped message {}: {why:?}", msg.id);
            }
            return;
        }
        let resolve = |m: Mention| mention_name(&guild, msg, m);
        let Some(words) = text::clean(&msg.content, &resolve, tts.config.max_chars()) else {
            return;
        };
        let tts_nickname = tts.nickname(guild_id, msg.author.id);
        let guild_nick = msg
            .member
            .as_ref()
            .and_then(|m| m.nick.clone())
            .or_else(|| {
                guild
                    .members
                    .get(&msg.author.id)
                    .and_then(|m| m.nick.clone())
            });
        let name = text::speaker_name(
            &NameSources {
                tts_nickname: tts_nickname.as_deref(),
                guild_nick: guild_nick.as_deref(),
                global_name: msg.author.global_name.as_deref(),
                username: &msg.author.name,
            },
            &resolve,
        );
        Utterance {
            text: text::attributed(&name, &words),
            voice: tts.voice_of(msg.author.id),
        }
    };

    if !session.push(msg.author.id, spoken) {
        tracing::debug!(
            "TTS queue full in guild {guild_id} (or for {}); dropped message {}",
            msg.author.id,
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

/// The voice channel `user` is in, and whether it is a stage, from the cache.
#[must_use]
pub fn voice_channel_of(ctx: &Context, guild: GuildId, user: UserId) -> Option<(ChannelId, bool)> {
    let guild = ctx.cache.guild(guild)?;
    let channel = guild.voice_states.get(&user)?.channel_id?;
    let stage = guild
        .channels
        .get(&channel)
        .is_some_and(|c| c.kind == serenity::all::ChannelType::Stage);
    drop(guild);
    Some((channel, stage))
}
