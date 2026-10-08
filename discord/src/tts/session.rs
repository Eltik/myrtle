//! One guild's stay in a voice channel: join, speak the queue in order, leave.
//!
//! A session is created by the first speakable message and owns a worker task for its whole life.
//! The worker joins, waits [`TtsConfig::settle_ms`](crate::config::TtsConfig) for the voice
//! connection to finish its DAVE handshake, then speaks one message at a time until it is stopped
//! (`/tts leave`, `/tts disable`, the channel emptying, the bot being moved or disconnected) or
//! goes [`TtsConfig::idle_secs`](crate::config::TtsConfig) without speaking. It always ends by
//! leaving the channel and removing itself.
//!
//! DAVE: songbird 0.6.0 encrypts with the end-to-end session only once that session reports
//! ready, and sends packets without it until then (`mixer/mod.rs`, the `is_ready()` branch);
//! clients discard those (songbird issue #310). Songbird raises no event when the session turns
//! ready: `CoreEvent` stops at `DriverConnect`, which `Songbird::join` already waits for. The only
//! lever left is time, so the first playback waits `settle_ms` after the connect. ⚠️ Whether
//! 1500 ms is enough is unmeasured: it is a guess at one MLS round trip, and a channel whose
//! first message comes out silent would say it is not. Later messages are unaffected, and a
//! member joining mid-session triggers a DAVE transition that songbird covers with passthrough.

use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serenity::model::id::{ChannelId, GuildId};
use songbird::error::JoinError;
use songbird::input::{Input, RawAdapter};
use songbird::tracks::TrackHandle;
use songbird::{Call, Event, EventContext, EventHandler, TrackEvent};
use tokio::sync::{Mutex as TokioMutex, Notify};
use tokio::time::Instant;

use super::Tts;
use super::engine::Speech;
use super::gate::SpeechQueue;

/// Added to a message's playing time before the worker stops waiting for its end event.
const PLAYBACK_SLACK: Duration = Duration::from_secs(5);

/// Tries at leaving voice, 2 s apart and doubling: 62 s in all, the span of a shard reconnect.
const LEAVE_ATTEMPTS: u32 = 6;

/// A guild's voice session, shared between its worker, the message handler and `/tts`.
pub struct Session {
    pub guild: GuildId,
    pub channel: ChannelId,
    queue: Mutex<SpeechQueue>,
    wake: Notify,
    stopping: AtomicBool,
    /// Set once `Songbird::join` has returned. Until then the bot's own voice-state updates may be
    /// a late replay from the guild's previous session and are ignored.
    joined: AtomicBool,
    current: Mutex<Option<TrackHandle>>,
    /// Set while a "did the channel empty" re-check is pending, so a burst of voice-state updates
    /// schedules one check, not one each.
    pub(super) empty_check: AtomicBool,
}

impl Session {
    pub(super) fn new(guild: GuildId, channel: ChannelId, queue_max: usize) -> Self {
        Self {
            guild,
            channel,
            queue: Mutex::new(SpeechQueue::new(queue_max)),
            wake: Notify::new(),
            stopping: AtomicBool::new(false),
            joined: AtomicBool::new(false),
            current: Mutex::new(None),
            empty_check: AtomicBool::new(false),
        }
    }

    /// Queue `text`. `false` when the queue is full or the session is ending; the text is dropped
    /// without a reply.
    pub fn push(&self, text: String) -> bool {
        if self.is_stopping() {
            return false;
        }
        let queued = lock(&self.queue).push(text);
        if queued {
            self.wake.notify_one();
        }
        queued
    }

    /// Messages waiting behind the one playing.
    #[must_use]
    pub fn queued(&self) -> usize {
        lock(&self.queue).len()
    }

    /// Stop the message playing now. `false` when nothing was playing.
    pub fn skip(&self) -> bool {
        let Some(handle) = lock(&self.current).take() else {
            return false;
        };
        let _ = handle.stop();
        true
    }

    /// End the session: stop playback and wake the worker, which leaves the channel.
    pub fn stop(&self) {
        self.stopping.store(true, Ordering::SeqCst);
        self.skip();
        self.wake.notify_one();
    }

    #[must_use]
    pub fn is_stopping(&self) -> bool {
        self.stopping.load(Ordering::SeqCst)
    }

    #[must_use]
    pub fn is_joined(&self) -> bool {
        self.joined.load(Ordering::SeqCst)
    }
}

/// A poisoned lock still holds a usable queue or handle; a panic elsewhere shouldn't silence TTS.
fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The worker: join, speak until stopped or idle, leave, deregister.
pub(super) async fn run(tts: Arc<Tts>, session: Arc<Session>) {
    let (guild, channel) = (session.guild, session.channel);
    let reason = match tts.songbird.join(guild, channel).await {
        Ok(call) => {
            session.joined.store(true, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(tts.config.settle_ms)).await;
            serve(&tts, &session, &call).await
        }
        Err(e) => {
            tracing::warn!("TTS couldn't join {channel} in guild {guild}: {e}");
            tts.join_failed(guild, channel);
            "join failed"
        }
    };
    // From here nothing more is queued: an idle ending must refuse late messages too.
    session.stop();
    leave(&tts, guild).await;
    tracing::info!("TTS left {channel} in guild {guild}: {reason}");
    tts.end_session(&session);
}

/// Leave the guild's voice channel, retrying while the gateway can't be reached (a shard
/// reconnecting), so the bot is never left in a channel with no session to time it out.
async fn leave(tts: &Tts, guild: GuildId) {
    let mut wait = Duration::from_secs(2);
    for attempt in 1..=LEAVE_ATTEMPTS {
        match tts.songbird.remove(guild).await {
            Ok(()) | Err(JoinError::NoCall) => return,
            Err(e) if attempt == LEAVE_ATTEMPTS => {
                tracing::warn!("TTS couldn't leave voice in guild {guild}: {e}");
            }
            Err(e) => {
                tracing::debug!("TTS leave in guild {guild} failed, retrying: {e}");
                tokio::time::sleep(wait).await;
                wait *= 2;
            }
        }
    }
}

/// Speak the queue in order. Returns why the session ended.
async fn serve(tts: &Tts, session: &Session, call: &TokioMutex<Call>) -> &'static str {
    let idle = Duration::from_secs(tts.config.idle_secs);
    let mut deadline = Instant::now() + idle;
    loop {
        if session.is_stopping() {
            return "stopped";
        }
        let next = lock(&session.queue).pop();
        if let Some(text) = next {
            speak_one(tts, session, call, text).await;
            deadline = Instant::now() + idle;
            continue;
        }
        tokio::select! {
            () = session.wake.notified() => {}
            () = tokio::time::sleep_until(deadline) => return "idle",
        }
    }
}

/// Synthesize `text` and play it, returning once it has finished, been skipped, or failed.
async fn speak_one(tts: &Tts, session: &Session, call: &TokioMutex<Call>, text: String) {
    let Some(speech) = tts.synth.speak(text).await else {
        if !tts.synth.available() {
            session.stop();
        }
        return;
    };
    if session.is_stopping() {
        return;
    }
    let wait = Duration::from_secs_f64(speech.seconds()) + PLAYBACK_SLACK;
    let input = speech_input(&speech);

    let done = Arc::new(Notify::new());
    let handle = call.lock().await.play_only_input(input);
    for event in [TrackEvent::End, TrackEvent::Error] {
        if let Err(e) = handle.add_event(Event::Track(event), Done(Arc::clone(&done))) {
            tracing::warn!(
                "TTS couldn't watch playback in guild {}: {e}",
                session.guild
            );
        }
    }
    *lock(&session.current) = Some(handle.clone());
    // A `stop()` between the check above and storing the handle found nothing to skip; it set the
    // flag first, so it is seen here.
    if session.is_stopping() {
        let _ = handle.stop();
    }

    if tokio::time::timeout(wait, done.notified()).await.is_err() {
        tracing::warn!(
            "TTS playback in guild {} outlasted {} ms; stopping it",
            session.guild,
            wait.as_millis()
        );
        let _ = handle.stop();
    }
    lock(&session.current).take();
}

/// `speech` as a songbird input: little-endian f32 mono behind `RawAdapter`'s header, which
/// songbird's own probe reads and symphonia's `pcm` codec decodes. Songbird resamples to 48 kHz.
fn speech_input(speech: &Speech) -> Input {
    let bytes: Vec<u8> = speech
        .samples
        .iter()
        .flat_map(|s| s.to_le_bytes())
        .collect();
    RawAdapter::new(Cursor::new(bytes), speech.sample_rate, 1).into()
}

/// Wakes the worker when a track ends or fails.
struct Done(Arc<Notify>);

#[serenity::async_trait]
impl EventHandler for Done {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        self.0.notify_one();
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird::input::codecs::{get_codec_registry, get_probe};

    /// The format path a spoken message takes, minus Discord: the bytes `speech_input` frames
    /// must probe as songbird's raw f32 container and decode back to every sample, at the
    /// voice's rate, mono.
    #[tokio::test]
    async fn speech_input_decodes_back_to_its_samples() {
        let samples: Vec<f32> = (0..22_050_u16)
            .map(|i| (f32::from(i) * 0.05).sin() * 0.5)
            .collect();
        let speech = Speech {
            samples: samples.clone(),
            sample_rate: 22_050,
        };
        let mut input = speech_input(&speech)
            .make_playable_async(get_codec_registry(), get_probe())
            .await
            .expect("songbird parses the raw container");
        let parsed = input.parsed_mut().expect("a playable input is parsed");

        let mut decoded = 0;
        while let Ok(packet) = parsed.format.next_packet() {
            let audio = parsed.decoder.decode(&packet).expect("pcm decodes");
            assert_eq!(audio.spec().rate, 22_050);
            assert_eq!(audio.spec().channels.count(), 1);
            decoded += audio.frames();
        }
        assert_eq!(decoded, samples.len());
    }
}
