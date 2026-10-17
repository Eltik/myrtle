//! Trevor's Discord alerts (contract v1.1 section 2.2), through the existing
//! `DISCORD_ALERT_WEBHOOK` of [`crate::core::alerts`].
//!
//! Each kind fires at most once per `TREVOR_ALERT_COOLDOWN_SECS` and sends one
//! "recovered" message when it clears (only after an announced firing). A
//! background evaluator ([`spawn_evaluator`], every `TREVOR_ALERT_EVERY_SECS`)
//! checks every kind; a heartbeat that reports a tripped guard checks
//! `guard_tripped` at once. Sending never blocks the caller: the Discord sink
//! posts from a spawned task, and a failed webhook is logged and swallowed.
//! `TREVOR_ALERTS=0` sends nothing.
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use chrono::{Timelike, Utc};

use super::{ONLINE_SECS, cf_serving_budget};
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::queries::trevor as q;

/// `worker_offline`: no worker online this long while jobs are queued ...
pub const OFFLINE_QUEUED_SECS: i64 = 180;
/// ... or this long in any case.
pub const OFFLINE_ANY_SECS: i64 = 900;
/// `backlog`: the oldest queued normal job is older than this.
pub const BACKLOG_SECS: i64 = 300;
/// `failure_rate`: over this share of the last [`FAILURE_WINDOW`] finished
/// jobs failed or fell back to passages after a model failure ...
pub const FAILURE_SHARE: f64 = 0.25;
pub const FAILURE_WINDOW: i64 = 40;
/// ... judged once at least this many finished (a lone failure of two is
/// not a rate).
pub const FAILURE_MIN_SAMPLES: i64 = 10;
/// `cf_budget`: spent before this UTC hour.
pub const CF_SPENT_BEFORE_HOUR: u32 = 18;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlertKind {
    GuardTripped,
    WorkerOffline,
    Backlog,
    FailureRate,
    CfBudget,
}

impl AlertKind {
    pub const ALL: [Self; 5] = [
        Self::GuardTripped,
        Self::WorkerOffline,
        Self::Backlog,
        Self::FailureRate,
        Self::CfBudget,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GuardTripped => "guard_tripped",
            Self::WorkerOffline => "worker_offline",
            Self::Backlog => "backlog",
            Self::FailureRate => "failure_rate",
            Self::CfBudget => "cf_budget",
        }
    }

    const fn title(self) -> &'static str {
        match self {
            Self::GuardTripped => "a worker's power guard tripped",
            Self::WorkerOffline => "no worker online",
            Self::Backlog => "questions are waiting",
            Self::FailureRate => "answers are failing",
            Self::CfBudget => "the Cloudflare budget is spent early",
        }
    }
}

/// One message to send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alert {
    pub kind: AlertKind,
    pub recovered: bool,
    pub title: String,
    pub detail: String,
}

/// Where alerts go: Discord in the server, a recorder in tests.
pub trait AlertSink: Send + Sync {
    /// Must not block: the Discord sink spawns the post.
    fn send(&self, alert: Alert);
}

/// Posts through [`crate::core::alerts::report_trevor`] from a spawned task.
pub struct DiscordSink {
    client: reqwest::Client,
}

impl DiscordSink {
    pub const fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

impl AlertSink for DiscordSink {
    fn send(&self, alert: Alert) {
        let client = self.client.clone();
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.spawn(async move {
                crate::core::alerts::report_trevor(
                    &client,
                    alert.kind.as_str(),
                    &alert.title,
                    &alert.detail,
                    alert.recovered,
                )
                .await;
            });
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct KindState {
    /// The condition held at the last observation.
    active: bool,
    /// The current episode was announced (so its clearing is too).
    announced: bool,
    /// Unix secs of the last firing message.
    last_fired: Option<i64>,
}

/// The per-kind state machine.
#[derive(Debug, Default)]
pub struct Alerter {
    kinds: Mutex<HashMap<AlertKind, KindState>>,
}

impl Alerter {
    /// Record whether `kind`'s condition holds at `now` (unix secs) and say
    /// what to send, if anything: a firing when it holds, is not yet
    /// announced and the kind last fired `cooldown` or more ago; a recovered
    /// message when it stops holding after an announced firing.
    pub fn observe(
        &self,
        kind: AlertKind,
        holds: bool,
        detail: &str,
        now: i64,
        cooldown: i64,
    ) -> Option<Alert> {
        let mut kinds = self
            .kinds
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let st = kinds.entry(kind).or_default();
        let alert = |recovered: bool| Alert {
            kind,
            recovered,
            title: kind.title().to_owned(),
            detail: detail.to_owned(),
        };
        if holds {
            st.active = true;
            let cooled = st.last_fired.is_none_or(|t| now - t >= cooldown);
            if !st.announced && cooled {
                st.announced = true;
                st.last_fired = Some(now);
                return Some(alert(false));
            }
            None
        } else {
            let was = st.active && st.announced;
            st.active = false;
            st.announced = false;
            was.then(|| alert(true))
        }
    }

    pub fn is_active(&self, kind: AlertKind) -> bool {
        self.kinds
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&kind)
            .is_some_and(|s| s.active)
    }
}

/// Observe one kind and send what it yields. Off under `TREVOR_ALERTS=0`.
pub fn observe(state: &AppState, kind: AlertKind, holds: bool, detail: &str) -> Option<Alert> {
    let cfg = state.trevor.config();
    if !cfg.alerts {
        return None;
    }
    let alert = state.trevor.alerter.observe(
        kind,
        holds,
        detail,
        Utc::now().timestamp(),
        cfg.alert_cooldown_secs,
    );
    if let Some(a) = &alert {
        state.trevor.sink().send(a.clone());
    }
    alert
}

/// What the evaluator measured, pure so the conditions are testable.
#[derive(Debug, Clone, Default)]
pub struct Signals {
    /// Seconds since any worker's last heartbeat (or since this process
    /// started, when none ever beat).
    pub secs_since_seen: i64,
    pub online: i64,
    /// Jobs queued (normal and deferred).
    pub queued: i64,
    /// Age in seconds of the oldest queued normal job.
    pub oldest_queued_normal_secs: Option<i64>,
    /// Of the last [`FAILURE_WINDOW`] finished jobs: how many, and how many
    /// failed or fell back to passages after a model failure.
    pub finished: i64,
    pub failed: i64,
    pub neurons_today: i64,
    pub cf_budget: i64,
    pub utc_hour: u32,
    /// Online workers whose guard is tripped, with their reasons.
    pub tripped: Vec<(String, Option<String>)>,
}

/// Each kind's condition and its message.
pub fn conditions(s: &Signals, cf_active: bool) -> Vec<(AlertKind, bool, String)> {
    let offline_secs = s.secs_since_seen - ONLINE_SECS;
    let offline = s.online == 0
        && ((s.queued > 0 && offline_secs >= OFFLINE_QUEUED_SECS)
            || offline_secs >= OFFLINE_ANY_SECS);
    let backlog = s
        .oldest_queued_normal_secs
        .is_some_and(|age| age > BACKLOG_SECS);
    #[allow(clippy::cast_precision_loss)]
    let failing = s.finished >= FAILURE_MIN_SAMPLES
        && (s.failed as f64) > FAILURE_SHARE * (s.finished as f64);
    let spent = s.cf_budget > 0 && s.neurons_today >= s.cf_budget;
    // Once fired, it holds until the budget resets (the next UTC day), not
    // until 18:00 passes.
    let cf = spent && (s.utc_hour < CF_SPENT_BEFORE_HOUR || cf_active);
    let reasons: Vec<String> = s
        .tripped
        .iter()
        .map(|(w, r)| format!("{w}: {}", r.as_deref().unwrap_or("no reason given")))
        .collect();
    vec![
        (
            AlertKind::GuardTripped,
            !s.tripped.is_empty(),
            format!(
                "The worker stopped claiming model jobs and answers from passages.\n{}",
                reasons.join("\n")
            ),
        ),
        (
            AlertKind::WorkerOffline,
            offline,
            format!(
                "No worker heartbeat for {} s; {} job(s) queued.",
                s.secs_since_seen.max(0),
                s.queued
            ),
        ),
        (
            AlertKind::Backlog,
            backlog,
            format!(
                "The oldest queued question has waited {} s.",
                s.oldest_queued_normal_secs.unwrap_or(0)
            ),
        ),
        (
            AlertKind::FailureRate,
            failing,
            format!(
                "{} of the last {} finished jobs failed or fell back to passages after a model failure.",
                s.failed, s.finished
            ),
        ),
        (
            AlertKind::CfBudget,
            cf,
            format!(
                "{} of {} Neurons spent at {:02}:00 UTC; answers go to the CPU rung until 00:00 UTC.",
                s.neurons_today, s.cf_budget, s.utc_hour
            ),
        ),
    ]
}

/// Measure the signals.
pub async fn signals(state: &AppState) -> Result<Signals, ApiError> {
    let m = q::alert_signals(&state.db, ONLINE_SECS, FAILURE_WINDOW).await?;
    let cfg = state.trevor.config();
    let now = Utc::now();
    let tripped = q::tripped_workers(&state.db, ONLINE_SECS).await?;
    Ok(Signals {
        secs_since_seen: m
            .secs_since_seen
            .map_or_else(|| now.timestamp() - state.trevor.started_at, |s| s as i64),
        online: m.online,
        queued: m.queued,
        oldest_queued_normal_secs: m.oldest_queued_normal_secs.map(|s| s as i64),
        finished: m.finished,
        failed: m.failed,
        neurons_today: m.neurons_today,
        cf_budget: cf_serving_budget(&cfg),
        utc_hour: now.hour(),
        tripped,
    })
}

/// One evaluator pass: measure, observe every kind, return what was sent.
pub async fn evaluate(state: &AppState) -> Result<Vec<Alert>, ApiError> {
    if !state.trevor.config().alerts {
        return Ok(Vec::new());
    }
    let s = signals(state).await?;
    let cf_active = state.trevor.alerter.is_active(AlertKind::CfBudget);
    Ok(conditions(&s, cf_active)
        .into_iter()
        .filter_map(|(kind, holds, detail)| observe(state, kind, holds, &detail))
        .collect())
}

/// After a heartbeat: `guard_tripped` at once, from the online workers'
/// guards (so one worker recovering does not clear another's trip).
pub async fn after_heartbeat(state: &AppState) -> Result<(), ApiError> {
    if !state.trevor.config().alerts {
        return Ok(());
    }
    let tripped = q::tripped_workers(&state.db, ONLINE_SECS).await?;
    let s = Signals {
        tripped,
        ..Signals::default()
    };
    if let Some((kind, holds, detail)) = conditions(&s, false)
        .into_iter()
        .find(|(k, _, _)| *k == AlertKind::GuardTripped)
    {
        observe(state, kind, holds, &detail);
    }
    Ok(())
}

/// The evaluator's background tick.
pub fn spawn_evaluator(state: AppState) {
    tokio::spawn(async move {
        let every = state.trevor.config().alert_every_secs;
        let mut tick = tokio::time::interval(Duration::from_secs(every));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            tick.tick().await;
            if let Err(e) = evaluate(&state).await {
                tracing::warn!(error = %e, "trevor: alert evaluation failed");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_kind_fires_once_per_cooldown_and_recovers_once() {
        let a = Alerter::default();
        let k = AlertKind::Backlog;
        let fired = a.observe(k, true, "x", 0, 1800).expect("fires");
        assert!(!fired.recovered);
        // Still holding: silent.
        assert!(a.observe(k, true, "x", 30, 1800).is_none());
        assert!(a.observe(k, true, "x", 5000, 1800).is_none());
        let rec = a.observe(k, false, "x", 5030, 1800).expect("recovers");
        assert!(rec.recovered);
        // Cleared and quiet: nothing more.
        assert!(a.observe(k, false, "x", 5060, 1800).is_none());
        // A new episode inside the cooldown of the last firing (t=0 + 1800
        // is 1800; at 5090 it has passed) fires again.
        assert!(a.observe(k, true, "x", 5090, 1800).is_some());
        assert!(a.observe(k, false, "x", 5120, 1800).is_some());
        // Inside the cooldown: suppressed, and so is its recovery.
        assert!(a.observe(k, true, "x", 5150, 1800).is_none());
        assert!(a.observe(k, false, "x", 5180, 1800).is_none());
        // Still holding once the cooldown passes: announced then.
        assert!(a.observe(k, true, "x", 6000, 1800).is_none());
        assert!(a.observe(k, true, "x", 5090 + 1800, 1800).is_some());
    }

    #[test]
    fn kinds_are_independent() {
        let a = Alerter::default();
        assert!(a.observe(AlertKind::Backlog, true, "", 0, 1800).is_some());
        assert!(
            a.observe(AlertKind::WorkerOffline, true, "", 1, 1800)
                .is_some()
        );
        assert!(a.is_active(AlertKind::Backlog));
        assert!(!a.is_active(AlertKind::CfBudget));
    }

    fn holds(s: &Signals, cf_active: bool, kind: AlertKind) -> bool {
        conditions(s, cf_active)
            .into_iter()
            .find(|(k, _, _)| *k == kind)
            .unwrap()
            .1
    }

    #[test]
    fn the_conditions_follow_the_contract_thresholds() {
        let base = Signals {
            online: 1,
            cf_budget: 8000,
            utc_hour: 12,
            ..Signals::default()
        };
        // worker_offline: 3 min offline with a queue, 15 min without.
        let off = |since: i64, queued: i64| Signals {
            online: 0,
            secs_since_seen: since,
            queued,
            ..base.clone()
        };
        assert!(!holds(&off(75 + 179, 1), false, AlertKind::WorkerOffline));
        assert!(holds(&off(75 + 180, 1), false, AlertKind::WorkerOffline));
        assert!(!holds(&off(75 + 899, 0), false, AlertKind::WorkerOffline));
        assert!(holds(&off(75 + 900, 0), false, AlertKind::WorkerOffline));
        // backlog: older than 5 min.
        let bl = |age| Signals {
            oldest_queued_normal_secs: Some(age),
            ..base.clone()
        };
        assert!(!holds(&bl(300), false, AlertKind::Backlog));
        assert!(holds(&bl(301), false, AlertKind::Backlog));
        // failure_rate: over 25% of 40; 10 of 40 is exactly 25%, not over.
        let fr = |failed, finished| Signals {
            failed,
            finished,
            ..base.clone()
        };
        assert!(!holds(&fr(10, 40), false, AlertKind::FailureRate));
        assert!(holds(&fr(11, 40), false, AlertKind::FailureRate));
        assert!(!holds(&fr(5, 9), false, AlertKind::FailureRate));
        // cf_budget: spent before 18:00 UTC, held after once fired.
        let cf = |n, hour| Signals {
            neurons_today: n,
            utc_hour: hour,
            ..base.clone()
        };
        assert!(!holds(&cf(7999, 12), false, AlertKind::CfBudget));
        assert!(holds(&cf(8000, 17), false, AlertKind::CfBudget));
        assert!(!holds(&cf(8000, 18), false, AlertKind::CfBudget));
        assert!(holds(&cf(8000, 19), true, AlertKind::CfBudget));
        // guard_tripped: any online tripped worker.
        let g = Signals {
            tripped: vec![("w1".into(), Some("psi cpu 71".into()))],
            ..base.clone()
        };
        assert!(holds(&g, false, AlertKind::GuardTripped));
        assert!(!holds(&base, false, AlertKind::GuardTripped));
    }

    #[test]
    fn the_embed_says_fired_or_recovered() {
        let e =
            crate::core::alerts::trevor_embed("backlog", "questions are waiting", "a `b`", false);
        assert_eq!(e["title"], "Trevor: questions are waiting");
        assert_eq!(e["description"], "a 'b'");
        let r = crate::core::alerts::trevor_embed("backlog", "questions are waiting", "", true);
        assert_eq!(r["title"], "Trevor recovered: questions are waiting");
        assert_ne!(e["color"], r["color"]);
    }
}
