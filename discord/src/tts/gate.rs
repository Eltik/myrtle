//! Whether a message is spoken, as pure functions over facts the caller gathers.
//!
//! The order of the checks in [`decide`] is the order of their cost to the caller: everything
//! here is read from the message or the cache, and the per-member rate limit, which records a
//! send, is consulted only after every other check has passed.

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::time::{Duration, Instant};

use serenity::model::id::ChannelId;

/// What the message handler knows about one message when it asks whether to speak it.
//
// The bools are independent yes/no facts read off one message, not states of one machine; an
// enum per flag would only rename `true`.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct Facts {
    /// The author is a bot, a webhook, or the message is a system message (joins, pins, boosts).
    pub automated: bool,
    /// The message starts with the bot's prefix, so it is a prefix command, not speech.
    pub prefixed: bool,
    /// The message carries stickers, which have nothing to read.
    pub has_stickers: bool,
    /// A moderator switched TTS off for the guild.
    pub guild_disabled: bool,
    /// The voice engine loaded, or has not been tried yet with its files present.
    pub engine_available: bool,
    /// The channel the message was sent in.
    pub channel: ChannelId,
    /// The channel is a voice channel (not a stage, not a text channel).
    pub channel_is_voice: bool,
    /// The author's current voice channel in this guild, from the cache.
    pub author_voice: Option<ChannelId>,
    /// The voice channel the bot is speaking in for this guild, if any.
    pub bot_voice: Option<ChannelId>,
}

/// Why a message is not spoken. Every skip is silent; the reason is for logs and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skip {
    Automated,
    Prefixed,
    Stickers,
    Disabled,
    EngineUnavailable,
    NotVoiceChannel,
    AuthorNotInChannel,
    BusyElsewhere,
}

/// The verdict on one message, before cleanup and the rate limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Speak it in the bot's current channel.
    Speak,
    /// Speak it, joining the channel first.
    JoinAndSpeak,
    Skip(Skip),
}

/// Decide whether `facts` describe a message the bot speaks.
///
/// The author must be in the voice channel whose text chat they typed in, and the bot must be in
/// that same channel or in none of the guild's. Being in another one is not an error and gets no
/// reply: a reply per message would be spam in a busy chat.
#[must_use]
pub fn decide(facts: &Facts) -> Verdict {
    let skip = if facts.automated {
        Some(Skip::Automated)
    } else if facts.prefixed {
        Some(Skip::Prefixed)
    } else if facts.has_stickers {
        Some(Skip::Stickers)
    } else if !facts.channel_is_voice {
        Some(Skip::NotVoiceChannel)
    } else if facts.author_voice != Some(facts.channel) {
        Some(Skip::AuthorNotInChannel)
    } else if facts.guild_disabled {
        Some(Skip::Disabled)
    } else if !facts.engine_available {
        Some(Skip::EngineUnavailable)
    } else {
        None
    };
    if let Some(skip) = skip {
        return Verdict::Skip(skip);
    }
    match facts.bot_voice {
        None => Verdict::JoinAndSpeak,
        Some(bot) if bot == facts.channel => Verdict::Speak,
        Some(_) => Verdict::Skip(Skip::BusyElsewhere),
    }
}

/// At most one event per key per `interval`.
///
/// Used per member (one spoken message per two seconds by default) and per guild (one voice join
/// per [`super::JOIN_COOLDOWN`]). Entries older than `interval` are pruned once the map passes
/// [`RateLimiter::PRUNE_AT`] keys, so a member who speaks once never stays in memory for good.
#[derive(Debug)]
pub struct RateLimiter<K> {
    interval: Duration,
    last: HashMap<K, Instant>,
}

impl<K: Eq + Hash + Copy> RateLimiter<K> {
    const PRUNE_AT: usize = 1024;

    #[must_use]
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last: HashMap::new(),
        }
    }

    /// Whether `key` may act at `now`; when it may, `now` is recorded as its last action.
    pub fn allow(&mut self, key: K, now: Instant) -> bool {
        if let Some(&t) = self.last.get(&key)
            && now.saturating_duration_since(t) < self.interval
        {
            return false;
        }
        if self.last.len() >= Self::PRUNE_AT {
            let interval = self.interval;
            self.last
                .retain(|_, t| now.saturating_duration_since(*t) < interval);
        }
        self.last.insert(key, now);
        true
    }

    /// Record `now` for `key` without asking, so the next [`allow`](Self::allow) waits a full
    /// interval from here.
    pub fn touch(&mut self, key: K, now: Instant) {
        self.last.insert(key, now);
    }
}

/// A FIFO of text waiting to be spoken, dropping what arrives past `cap`.
#[derive(Debug)]
pub struct SpeechQueue {
    cap: usize,
    items: VecDeque<String>,
}

impl SpeechQueue {
    #[must_use]
    pub const fn new(cap: usize) -> Self {
        Self {
            cap,
            items: VecDeque::new(),
        }
    }

    /// Queue `text`. Returns `false`, keeping the queue as it was, when it is already full.
    pub fn push(&mut self, text: String) -> bool {
        if self.items.len() >= self.cap {
            return false;
        }
        self.items.push_back(text);
        true
    }

    pub fn pop(&mut self) -> Option<String> {
        self.items.pop_front()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VC: ChannelId = ChannelId::new(10);
    const OTHER_VC: ChannelId = ChannelId::new(11);

    /// A message a member in `VC` typed in `VC`'s chat, with the bot nowhere.
    fn speakable() -> Facts {
        Facts {
            automated: false,
            prefixed: false,
            has_stickers: false,
            guild_disabled: false,
            engine_available: true,
            channel: VC,
            channel_is_voice: true,
            author_voice: Some(VC),
            bot_voice: None,
        }
    }

    #[test]
    fn author_in_channel_joins_then_speaks() {
        assert_eq!(decide(&speakable()), Verdict::JoinAndSpeak);
        let here = Facts {
            bot_voice: Some(VC),
            ..speakable()
        };
        assert_eq!(decide(&here), Verdict::Speak);
    }

    #[test]
    fn author_must_be_in_this_channel() {
        let elsewhere = Facts {
            author_voice: Some(OTHER_VC),
            ..speakable()
        };
        assert_eq!(decide(&elsewhere), Verdict::Skip(Skip::AuthorNotInChannel));
        let nowhere = Facts {
            author_voice: None,
            ..speakable()
        };
        assert_eq!(decide(&nowhere), Verdict::Skip(Skip::AuthorNotInChannel));
    }

    #[test]
    fn only_voice_channel_chat() {
        let text = Facts {
            channel_is_voice: false,
            ..speakable()
        };
        assert_eq!(decide(&text), Verdict::Skip(Skip::NotVoiceChannel));
    }

    #[test]
    fn bots_and_prefix_commands_are_silent() {
        let bot = Facts {
            automated: true,
            ..speakable()
        };
        assert_eq!(decide(&bot), Verdict::Skip(Skip::Automated));
        let prefixed = Facts {
            prefixed: true,
            ..speakable()
        };
        assert_eq!(decide(&prefixed), Verdict::Skip(Skip::Prefixed));
        let sticker = Facts {
            has_stickers: true,
            ..speakable()
        };
        assert_eq!(decide(&sticker), Verdict::Skip(Skip::Stickers));
    }

    #[test]
    fn disabled_guild_and_missing_engine() {
        let disabled = Facts {
            guild_disabled: true,
            ..speakable()
        };
        assert_eq!(decide(&disabled), Verdict::Skip(Skip::Disabled));
        let no_engine = Facts {
            engine_available: false,
            ..speakable()
        };
        assert_eq!(decide(&no_engine), Verdict::Skip(Skip::EngineUnavailable));
    }

    #[test]
    fn busy_in_another_channel_is_silent() {
        let busy = Facts {
            bot_voice: Some(OTHER_VC),
            ..speakable()
        };
        assert_eq!(decide(&busy), Verdict::Skip(Skip::BusyElsewhere));
    }

    #[test]
    fn rate_limit_per_key() {
        let mut limit = RateLimiter::new(Duration::from_secs(2));
        let t0 = Instant::now();
        assert!(limit.allow(1u64, t0));
        assert!(!limit.allow(1, t0 + Duration::from_millis(1999)));
        assert!(
            limit.allow(2, t0 + Duration::from_millis(1)),
            "keys are independent"
        );
        assert!(limit.allow(1, t0 + Duration::from_secs(2)));
        // A refused attempt does not restart the window.
        assert!(!limit.allow(1, t0 + Duration::from_secs(3)));
        assert!(limit.allow(1, t0 + Duration::from_secs(4)));
    }

    #[test]
    fn rate_limit_prunes_stale_keys() {
        let mut limit = RateLimiter::new(Duration::from_secs(1));
        let t0 = Instant::now();
        for k in 0..RateLimiter::<u64>::PRUNE_AT as u64 {
            assert!(limit.allow(k, t0));
        }
        assert!(limit.allow(u64::MAX, t0 + Duration::from_secs(5)));
        assert_eq!(limit.last.len(), 1);
    }

    #[test]
    fn queue_caps_and_keeps_order() {
        let mut q = SpeechQueue::new(10);
        for i in 0..10 {
            assert!(q.push(format!("m{i}")));
        }
        assert!(!q.push("m10".into()), "the eleventh is dropped");
        assert_eq!(q.len(), 10);
        assert_eq!(q.pop().as_deref(), Some("m0"));
        assert!(q.push("m11".into()), "room again after one is spoken");
        let rest: Vec<_> = std::iter::from_fn(|| q.pop()).collect();
        assert_eq!(rest.first().map(String::as_str), Some("m1"));
        assert_eq!(rest.last().map(String::as_str), Some("m11"));
        assert!(q.is_empty());
    }
}
