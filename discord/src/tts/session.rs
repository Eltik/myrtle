//! One guild's stay in a voice channel: join, speak the queue in order, leave.
//!
//! A session is created by `/tts join` and owns a worker task for its whole life. The worker
//! joins, reports the outcome to the command, waits [`TtsConfig::settle_ms`] for the voice
//! connection to finish its DAVE handshake, then speaks one message at a time until it is stopped
//! (`/tts leave`, the channel emptying, the bot being moved or disconnected) or goes
//! [`TtsConfig::idle_secs`] without speaking. It always ends by leaving the channel and removing
//! itself.
//!
//! A message is spoken a chunk at a time ([`text::chunks`]: sentences, split further to fit
//! Google's 200-unit limit): a producer fetches each chunk from Google in turn and hands it to
//! the player, so the first chunk plays while the next is fetched. A chunk whose fetch fails is
//! skipped with a warning in the log and nothing in chat. `/tts skip` drops the rest of the
//! message.
//!
//! DAVE: songbird 0.6.0 encrypts with the end-to-end session only once that session reports
//! ready, and sends packets without it until then (`mixer/mod.rs`, the `is_ready()` branch);
//! clients discard those (songbird issue #310). Songbird raises no event when the session turns
//! ready: `CoreEvent` stops at `DriverConnect`, which `Songbird::join` already waits for. The only
//! lever left is time, so the first playback waits `settle_ms` after the connect. ⚠️ TTS is
//! reported working live at 1500 ms, but whether the first sentence after every join is heard
//! has not been checked on its own; a silent first sentence would say the wait is too short.
//!
//! [`TtsConfig::settle_ms`]: crate::config::TtsConfig::settle_ms
//! [`TtsConfig::idle_secs`]: crate::config::TtsConfig::idle_secs

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serenity::model::id::{ChannelId, GuildId, UserId};
use songbird::error::JoinError;
use songbird::input::Input;
use songbird::tracks::TrackHandle;
use songbird::{Call, Event, EventContext, EventHandler, TrackEvent};
use tokio::sync::{Mutex as TokioMutex, Notify, mpsc, oneshot};
use tokio::time::Instant;

use super::Tts;
use super::engine::MAX_UNITS;
use super::gate::SpeechQueue;
use super::text;
use super::voices::Voice;

/// Added to a chunk's estimated playing time before the worker stops waiting for its end event.
const PLAYBACK_SLACK: Duration = Duration::from_secs(5);

/// MP3 bytes per second of Google's speech: 64 kbps CBR mono at 24 kHz (the fixture measures
/// 15,744 B for 1.968 s). Halved here, so the end-event deadline errs long.
const MP3_BYTES_PER_SEC_FLOOR: u64 = 4000;

/// Tries at leaving voice, 2 s apart and doubling: 62 s in all, the span of a shard reconnect.
const LEAVE_ATTEMPTS: u32 = 6;

/// Chunks fetched ahead of the one playing. One keeps the next ready without one guild holding
/// both of the global request permits for a whole message.
const CHUNKS_AHEAD: usize = 1;

/// One message waiting to be spoken, in the voice its author picked.
#[derive(Debug, Clone)]
pub struct Utterance {
    pub text: String,
    pub voice: Voice,
}

/// A guild's voice session, shared between its worker, the message handler and `/tts`.
pub struct Session {
    pub guild: GuildId,
    pub channel: ChannelId,
    queue: Mutex<SpeechQueue<Utterance>>,
    wake: Notify,
    stopping: AtomicBool,
    /// Set once `Songbird::join` has returned. Until then the bot's own voice-state updates may be
    /// a late replay from the guild's previous session and are ignored.
    joined: AtomicBool,
    /// A message is being spoken (fetching or playing any of its chunks).
    in_message: AtomicBool,
    /// `/tts skip` asked to drop the rest of the message being spoken.
    skip_message: AtomicBool,
    current: Mutex<Option<TrackHandle>>,
    /// Set while a "did the channel empty" re-check is pending, so a burst of voice-state updates
    /// schedules one check, not one each.
    pub(super) empty_check: AtomicBool,
}

impl Session {
    pub(super) fn new(
        guild: GuildId,
        channel: ChannelId,
        queue_max: usize,
        user_queue_max: usize,
    ) -> Self {
        Self {
            guild,
            channel,
            queue: Mutex::new(SpeechQueue::new(queue_max, user_queue_max)),
            wake: Notify::new(),
            stopping: AtomicBool::new(false),
            joined: AtomicBool::new(false),
            in_message: AtomicBool::new(false),
            skip_message: AtomicBool::new(false),
            current: Mutex::new(None),
            empty_check: AtomicBool::new(false),
        }
    }

    /// Queue `text` from `user`. `false` when the guild's queue is full, `user` already has their
    /// share waiting, or the session is ending; the text is dropped without a reply.
    pub fn push(&self, user: UserId, utterance: Utterance) -> bool {
        if self.is_stopping() {
            return false;
        }
        let queued = lock(&self.queue).push(user.get(), utterance);
        if queued {
            self.wake.notify_one();
        }
        queued
    }

    /// Messages waiting behind the one being spoken.
    #[must_use]
    pub fn queued(&self) -> usize {
        lock(&self.queue).len()
    }

    /// Drop the rest of the message being spoken. `false` when nothing was being spoken.
    pub fn skip(&self) -> bool {
        if !self.in_message.load(Ordering::SeqCst) {
            return false;
        }
        self.skip_message.store(true, Ordering::SeqCst);
        let playing = lock(&self.current).take();
        if let Some(handle) = playing {
            let _ = handle.stop();
        }
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

    /// The message being spoken should be abandoned: skipped, or the session is ending.
    fn abandoned(&self) -> bool {
        self.skip_message.load(Ordering::SeqCst) || self.is_stopping()
    }
}

/// A poisoned lock still holds a usable queue or handle; a panic elsewhere shouldn't silence TTS.
fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The worker: join, tell `joined` how it went, speak until stopped or idle, leave, deregister.
pub(super) async fn run(tts: Arc<Tts>, session: Arc<Session>, joined: oneshot::Sender<bool>) {
    let (guild, channel) = (session.guild, session.channel);
    let reason = match tts.songbird.join(guild, channel).await {
        Ok(call) => {
            session.joined.store(true, Ordering::SeqCst);
            let _ = joined.send(true);
            tokio::time::sleep(Duration::from_millis(tts.config.settle_ms())).await;
            serve(&tts, &session, &call).await
        }
        Err(e) => {
            tracing::warn!("TTS couldn't join {channel} in guild {guild}: {e}");
            let _ = joined.send(false);
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
    let idle = Duration::from_secs(tts.config.idle_secs());
    let mut deadline = Instant::now() + idle;
    loop {
        if session.is_stopping() {
            return "stopped";
        }
        let next = lock(&session.queue).pop();
        if let Some(message) = next {
            session.skip_message.store(false, Ordering::SeqCst);
            session.in_message.store(true, Ordering::SeqCst);
            speak_message(tts, session, call, &message).await;
            session.in_message.store(false, Ordering::SeqCst);
            deadline = Instant::now() + idle;
            continue;
        }
        tokio::select! {
            () = session.wake.notified() => {}
            () = tokio::time::sleep_until(deadline) => return "idle",
        }
    }
}

/// Speak one message a chunk at a time, fetching the next while the current one plays.
async fn speak_message(tts: &Tts, session: &Session, call: &TokioMutex<Call>, message: &Utterance) {
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(CHUNKS_AHEAD);
    let produce = async move {
        for chunk in text::chunks(&message.text, MAX_UNITS) {
            if session.abandoned() {
                break;
            }
            // A failed chunk is skipped, not the message: the rest may still come through.
            let Some(mp3) = tts.google.fetch(message.voice, &chunk).await else {
                continue;
            };
            if tx.send(mp3).await.is_err() {
                break;
            }
        }
    };
    let play = async {
        while let Some(mp3) = rx.recv().await {
            if session.abandoned() {
                break;
            }
            play_one(session, call, mp3).await;
        }
        // Dropping the receiver stops the producer at its next send.
        drop(rx);
    };
    tokio::join!(produce, play);
}

/// Play one chunk's MP3, returning once it has finished, been skipped, or failed.
async fn play_one(session: &Session, call: &TokioMutex<Call>, mp3: Vec<u8>) {
    let estimate = u64::try_from(mp3.len()).unwrap_or(u64::MAX) / MP3_BYTES_PER_SEC_FLOOR;
    let wait = Duration::from_secs(estimate) + PLAYBACK_SLACK;
    let input = Input::from(mp3);

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
    // A skip or stop between the caller's check and storing the handle found nothing to stop; it
    // set its flag first, so it is seen here.
    if session.abandoned() {
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

    fn said(text: &str) -> Utterance {
        Utterance {
            text: text.into(),
            voice: crate::tts::voices::FALLBACK,
        }
    }

    /// The live bug: a member's second message, 100 ms after the first, was dropped by a 2 s
    /// per-member cooldown before it reached the queue. Both now queue, in order.
    #[tokio::test]
    async fn back_to_back_messages_from_one_member_both_queue() {
        let session = Session::new(GuildId::new(1), ChannelId::new(2), 10, 3);
        let member = UserId::new(3);
        assert!(session.push(member, said("Amiya said: first")));
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(session.push(member, said("Amiya said: second")));
        assert_eq!(session.queued(), 2);
        let mut queue = lock(&session.queue);
        assert_eq!(
            queue.pop().map(|u| u.text).as_deref(),
            Some("Amiya said: first")
        );
        assert_eq!(
            queue.pop().map(|u| u.text).as_deref(),
            Some("Amiya said: second")
        );
    }

    /// The format path a spoken chunk takes, minus Discord: a real MP3 fetched from Google
    /// (`tests/fixtures/hello-en-us.mp3`, "Hello there, Doctor." in en-US) must probe through
    /// songbird's own registry and decode to 24 kHz mono, 1.968 s long, the length macOS
    /// `afinfo` reports for the file.
    #[tokio::test]
    async fn google_mp3_decodes_through_songbird() {
        let mp3: &'static [u8] = include_bytes!("../../tests/fixtures/hello-en-us.mp3");
        let mut input = Input::from(mp3)
            .make_playable_async(get_codec_registry(), get_probe())
            .await
            .expect("songbird probes the MP3");
        let parsed = input.parsed_mut().expect("a playable input is parsed");

        let mut frames = 0;
        let mut rate = 0;
        while let Ok(packet) = parsed.format.next_packet() {
            let audio = parsed.decoder.decode(&packet).expect("mp3 decodes");
            rate = audio.spec().rate;
            assert_eq!(audio.spec().channels.count(), 1);
            frames += audio.frames();
        }
        assert_eq!(rate, 24_000);
        let seconds = f64::from(u32::try_from(frames).unwrap()) / f64::from(rate);
        assert!((seconds - 1.968).abs() < 0.05, "decoded {seconds} s");
        assert_eq!(mp3.len(), 15_744);
    }
}
