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
#[derive(Debug, Clone)]
pub struct Facts {
    /// The author is a bot, a webhook, or the message is a system message (joins, pins, boosts).
    pub automated: bool,
    /// The message starts with the bot's prefix, so it is a prefix command, not speech.
    pub prefixed: bool,
    /// The message carries stickers, which have nothing to read.
    pub has_stickers: bool,
    /// The channel the message was sent in.
    pub channel: ChannelId,
    /// The author's current voice channel in this guild, from the cache.
    pub author_voice: Option<ChannelId>,
    /// The voice channel `/tts join` put the bot in for this guild, if any.
    pub bot_voice: Option<ChannelId>,
}

/// Why a message is not spoken. Every skip is silent; the reason is for logs and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skip {
    Automated,
    Prefixed,
    Stickers,
    /// The bot is not reading this channel: it is in none, or in another one.
    NotReadingHere,
    AuthorNotInChannel,
}

/// Decide whether `facts` describe a message the bot speaks: one typed in the chat of the voice
/// channel the bot was asked to join, by a member who is in that channel now.
pub fn decide(facts: &Facts) -> Result<(), Skip> {
    if facts.automated {
        Err(Skip::Automated)
    } else if facts.prefixed {
        Err(Skip::Prefixed)
    } else if facts.has_stickers {
        Err(Skip::Stickers)
    } else if facts.bot_voice != Some(facts.channel) {
        Err(Skip::NotReadingHere)
    } else if facts.author_voice != Some(facts.channel) {
        Err(Skip::AuthorNotInChannel)
    } else {
        Ok(())
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

/// A FIFO of text waiting to be spoken, with a cap for the guild and a cap per member.
///
/// A message past either cap is dropped and the ones already waiting are kept, so a member who
/// floods the chat is heard for their first few messages and then not at all until those are
/// spoken. Keys are user ids.
#[derive(Debug)]
pub struct SpeechQueue<T> {
    cap: usize,
    per_user: usize,
    items: VecDeque<(u64, T)>,
}

impl<T> SpeechQueue<T> {
    #[must_use]
    pub const fn new(cap: usize, per_user: usize) -> Self {
        Self {
            cap,
            per_user,
            items: VecDeque::new(),
        }
    }

    /// Queue `item` from `user`. Returns `false`, keeping the queue as it was, when the guild's
    /// queue is full or `user` already has `per_user` messages waiting.
    pub fn push(&mut self, user: u64, item: T) -> bool {
        if self.items.len() >= self.cap
            || self.items.iter().filter(|(u, _)| *u == user).count() >= self.per_user
        {
            return false;
        }
        self.items.push_back((user, item));
        true
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop_front().map(|(_, item)| item)
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

    /// A message a member in `VC` typed in `VC`'s chat, with the bot reading `VC`.
    fn speakable() -> Facts {
        Facts {
            automated: false,
            prefixed: false,
            has_stickers: false,
            channel: VC,
            author_voice: Some(VC),
            bot_voice: Some(VC),
        }
    }

    #[test]
    fn member_in_the_bots_channel_is_spoken() {
        assert_eq!(decide(&speakable()), Ok(()));
    }

    #[test]
    fn author_must_be_in_this_channel() {
        let elsewhere = Facts {
            author_voice: Some(OTHER_VC),
            ..speakable()
        };
        assert_eq!(decide(&elsewhere), Err(Skip::AuthorNotInChannel));
        let nowhere = Facts {
            author_voice: None,
            ..speakable()
        };
        assert_eq!(decide(&nowhere), Err(Skip::AuthorNotInChannel));
    }

    #[test]
    fn nothing_is_read_without_a_join() {
        let unjoined = Facts {
            bot_voice: None,
            ..speakable()
        };
        assert_eq!(decide(&unjoined), Err(Skip::NotReadingHere));
        let busy = Facts {
            bot_voice: Some(OTHER_VC),
            ..speakable()
        };
        assert_eq!(decide(&busy), Err(Skip::NotReadingHere));
        // A text channel's id is never the bot's voice channel.
        let text = Facts {
            channel: ChannelId::new(12),
            ..speakable()
        };
        assert_eq!(decide(&text), Err(Skip::NotReadingHere));
    }

    #[test]
    fn bots_and_prefix_commands_are_silent() {
        let bot = Facts {
            automated: true,
            ..speakable()
        };
        assert_eq!(decide(&bot), Err(Skip::Automated));
        let prefixed = Facts {
            prefixed: true,
            ..speakable()
        };
        assert_eq!(decide(&prefixed), Err(Skip::Prefixed));
        let sticker = Facts {
            has_stickers: true,
            ..speakable()
        };
        assert_eq!(decide(&sticker), Err(Skip::Stickers));
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
        let mut q = SpeechQueue::<String>::new(10, 10);
        for i in 0..10 {
            assert!(q.push(i, format!("m{i}")));
        }
        assert!(!q.push(10, "m10".into()), "the eleventh is dropped");
        assert_eq!(q.len(), 10);
        assert_eq!(q.pop().as_deref(), Some("m0"));
        assert!(q.push(11, "m11".into()), "room again after one is spoken");
        let rest: Vec<_> = std::iter::from_fn(|| q.pop()).collect();
        assert_eq!(rest.first().map(String::as_str), Some("m1"));
        assert_eq!(rest.last().map(String::as_str), Some("m11"));
        assert!(q.is_empty());
    }

    #[test]
    fn queue_caps_each_member_keeping_the_oldest() {
        let mut q = SpeechQueue::<String>::new(10, 3);
        assert!(q.push(1, "a1".into()));
        assert!(q.push(1, "a2".into()));
        assert!(q.push(2, "b1".into()));
        assert!(q.push(1, "a3".into()));
        assert!(
            !q.push(1, "a4".into()),
            "a fourth waiting message from one member is dropped"
        );
        assert!(q.push(2, "b2".into()), "other members are unaffected");
        assert_eq!(q.pop().as_deref(), Some("a1"));
        assert!(q.push(1, "a5".into()), "room again once one is spoken");
        let rest: Vec<_> = std::iter::from_fn(|| q.pop()).collect();
        assert_eq!(rest, ["a2", "b1", "a3", "b2", "a5"]);
    }
}
