//! Trevor on the backend: the light plumbing between readers and a worker
//! (design: `trevor/design/trevor-integration.md`; binding interface:
//! `trevor/design/trevor-contract-v1.md` and `trevor-contract-v1.1.md`).
//!
//! The backend never runs a model. It normalizes a question into a cache key,
//! answers from the Postgres cache when it can, and otherwise queues a job with
//! a rung (Cloudflare, local CPU, or passages only) and a spoiler horizon that
//! a worker pulls (`worker/next`), answers, streams (`worker/jobs/{id}/delta`)
//! and posts back (`worker/result`). Workers connect out with the scoped
//! `TREVOR_WORKER_KEY`, never the global service key.
//!
//! - [`ask`]: the reader side (ask, job status, panels, feedback, status).
//! - [`worker`]: the worker side (claim, heartbeat, delta, result, publish,
//!   the stale sweep, the warm-up).
//! - [`alerts`]: the Discord alerts on the queue and the workers.
//! - [`stream`]: the per-job SSE fan-out.
//! - [`runtime`]: the in-process state (config, wake-up, broadcast registry).
//! - [`turnstile`]: the Cloudflare Turnstile check on ask.
//!
//! The pure rules, re-exported here: [`normq`] (the cache key of a question),
//! [`rung`] (rungs, admission, the Neuron budget), [`guard`] (the master switch
//! and a worker's guard), [`horizon`] (the spoiler horizon and its era),
//! [`answer`] (a worker's answer and its citations) and [`wire`] (the
//! reader-facing request and response bodies).

pub mod alerts;
pub mod answer;
pub mod ask;
pub mod guard;
pub mod horizon;
pub mod normq;
pub mod rung;
pub mod runtime;
pub mod stream;
pub mod turnstile;
pub mod wire;
pub mod worker;

pub use answer::*;
pub use guard::*;
pub use horizon::*;
pub use normq::*;
pub use rung::*;
pub use runtime::TrevorConfig;
pub use wire::*;

use crate::app::error::ApiError;

/// Free questions a signed-in user may ask per clock hour.
pub const ASK_PER_HOUR: u64 = 30;
/// Open (queued or running) jobs one user may hold at once.
pub const MAX_OPEN_PER_USER: i64 = 3;
/// Queued jobs per server before ask answers "Trevor is busy" (503); only
/// under `TREVOR_ADMISSION=0`. Admission control replaces it with
/// `TREVOR_GLOBAL_QUEUE_MAX` and a passages answer instead of the 503.
pub const MAX_QUEUED: i64 = 200;
/// The longest question accepted, in characters.
pub const MAX_QUESTION_CHARS: usize = 500;
/// A running job whose worker has not answered in this long goes back to the
/// queue once, and fails the second time (`MAX_ATTEMPTS` claims).
pub const RUNNING_STALE_SECS: i64 = 300;
pub const MAX_ATTEMPTS: i16 = 2;
/// How long `worker/next` holds an empty poll open, under the 30 s handler timeout.
pub const LONG_POLL_SECS: u64 = 20;
/// A worker counts as online while its last heartbeat is this recent (two
/// missed 30 s beats plus slack).
pub const ONLINE_SECS: i64 = 75;
/// The stale sweep's tick.
pub const SWEEP_EVERY_SECS: u64 = 60;
/// The SSE keep-alive comment interval (Cloudflare drops a proxied stream
/// that is silent for 100 s).
pub const KEEPALIVE_SECS: u64 = 15;
/// A cited passage's text is trimmed to this many characters, at a line break.
pub const CITATION_TEXT_CHARS: usize = 600;
/// The ETA reads the last this-many finished jobs of a rung ...
pub const ETA_WINDOW: i64 = 50;
/// ... and falls back to [`Rung::fallback_eta_secs`] below this many.
pub const ETA_MIN_SAMPLES: i64 = 5;
/// `trevor_answers.era_pos` of an internal job, whose horizon is unlimited.
/// No reader maps here: every reader's eraPos is a story's gatePos.
pub const NO_LIMIT_ERA: i32 = i32::MAX;

/// The servers Trevor may serve. EN has a corpus today; CN is accepted so a CN
/// version stream needs no code change, and simply has no worker or version
/// until one is published.
pub const SERVERS: &[&str] = &["en", "cn"];

/// `server` as sent, defaulted to EN and checked against [`SERVERS`].
pub fn parse_server(server: Option<&str>) -> Result<&'static str, ApiError> {
    let s = server.unwrap_or("en");
    SERVERS
        .iter()
        .copied()
        .find(|known| known.eq_ignore_ascii_case(s))
        .ok_or_else(|| ApiError::BadRequest(format!("server must be one of {SERVERS:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_defaults_to_en_and_refuses_unknown() {
        assert_eq!(parse_server(None).unwrap(), "en");
        assert_eq!(parse_server(Some("CN")).unwrap(), "cn");
        assert!(parse_server(Some("jp")).is_err());
    }
}
