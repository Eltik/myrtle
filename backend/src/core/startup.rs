//! Startup progress: which phase the server is in, and how much of the boot is
//! left.
//!
//! A cold start runs tens of seconds - game data per server (EN's
//! `activity_table` alone is 2.3 GB), ~2800 level files, migrations, a config
//! fetch per server - and put almost nothing on stdout, so it read as a hang.
//!
//! Each step's duration goes to `startup_timings.json` and is read back on the
//! next boot, so a step that took 9s of a 40s boot gets 9/40 of the bar. Steps
//! that
//! can't count themselves - `serde_json` on a 2.3 GB table reports nothing - are
//! interpolated against that duration and scaled by the drift so far; the level
//! walks report exactly, through [`step_progress`]. A first boot has no history
//! and says `estimating`.
//!
//! Off a terminal (pm2, Docker) the bars become one log line per phase.
//!
//! [`step`] reads a global rather than taking a reporter, because the code that
//! reports it - `init_game_data`, the table loader - also runs in the tests, the
//! `src/bin` tools and the hot-reload watcher, none of which draw bars.

use std::borrow::Cow;
use std::collections::HashMap;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use arc_swap::ArcSwapOption;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Per-step durations, remembered between boots. `STARTUP_TIMINGS_FILE`
/// overrides the path.
const DEFAULT_TIMINGS_FILE: &str = "startup_timings.json";
const TIMING_FILE_VERSION: u32 = 1;

/// EWMA weight on the new measurement, so one cold boot doesn't set the next
/// estimate by itself.
const EWMA_ALPHA: f64 = 0.35;

/// Assumed cost of a step with no history and no size hint, and of any step
/// appended off-plan.
const UNMEASURED_MS: f64 = 400.0;

/// Nominal read+parse throughput, used to weight table steps by file size until
/// they have been measured (see [`table_hint_ms`]). A placeholder, not a
/// measurement.
const NOMINAL_PARSE_MB_PER_S: f64 = 55.0;

/// How far a step may be interpolated from elapsed time. A step completes only
/// on being succeeded, never on the clock.
const INTERP_CAP: f64 = 0.94;

/// Bounds on the drift correction; without them one fast early step predicts an
/// absurd total.
const DRIFT_MIN: f64 = 0.4;
const DRIFT_MAX: f64 = 4.0;

const TICK: Duration = Duration::from_millis(100);

/// Column the bars line up on; a longer label pushes its own bar right.
const LABEL_WIDTH: usize = 22;

// ── the plan ────────────────────────────────────────────────────────────────

/// One step of a phase: a stable key, which is also the display label, and what
/// to assume it costs with no history for it.
#[derive(Clone)]
pub struct StepSpec {
    key: Cow<'static, str>,
    hint_ms: f64,
}

impl StepSpec {
    pub fn new(key: impl Into<Cow<'static, str>>) -> Self {
        Self {
            key: key.into(),
            hint_ms: UNMEASURED_MS,
        }
    }

    /// Cost hint in milliseconds, used until the step has been measured once.
    #[must_use]
    pub const fn hint(mut self, ms: f64) -> Self {
        self.hint_ms = ms;
        self
    }
}

impl From<&'static str> for StepSpec {
    fn from(key: &'static str) -> Self {
        Self::new(key)
    }
}

impl From<String> for StepSpec {
    fn from(key: String) -> Self {
        Self::new(key)
    }
}

/// A phase of the boot and the steps it is expected to take.
///
/// The steps are declared up front so the phase has an estimate before any of it
/// runs: their remembered durations sum to it.
pub struct PhaseSpec {
    key: String,
    label: String,
    steps: Vec<StepSpec>,
}

impl PhaseSpec {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            steps: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_steps<I, S>(mut self, steps: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<StepSpec>,
    {
        self.steps = steps.into_iter().map(Into::into).collect();
        self
    }
}

/// Cost hint for a JSON table, from its size on disk. `None` when the file is
/// absent, which is normal - the loader warns and carries on - leaving the step
/// at [`UNMEASURED_MS`].
pub fn table_hint_ms(path: &std::path::Path) -> Option<f64> {
    let bytes = std::fs::metadata(path).ok()?.len() as f64;
    Some(bytes / (NOMINAL_PARSE_MB_PER_S * 1024.0 * 1024.0) * 1000.0)
}

// ── remembered timings ──────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct TimingFile {
    /// Bumped when the keys change meaning, so an old file is discarded rather
    /// than mixed into the new estimates.
    version: u32,
    /// `"phase/step"` -> smoothed duration in milliseconds.
    steps: HashMap<String, f64>,
}

fn timings_path() -> PathBuf {
    PathBuf::from(
        std::env::var("STARTUP_TIMINGS_FILE").unwrap_or_else(|_| DEFAULT_TIMINGS_FILE.into()),
    )
}

fn read_timings(path: &std::path::Path) -> HashMap<String, f64> {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return HashMap::new();
    };
    match serde_json::from_str::<TimingFile>(&raw) {
        Ok(file) if file.version == TIMING_FILE_VERSION => file.steps,
        Ok(_) => {
            debug!(path = %path.display(), "startup timings use an older format, discarding");
            HashMap::new()
        }
        Err(e) => {
            debug!(error = %e, path = %path.display(), "startup timings unreadable, discarding");
            HashMap::new()
        }
    }
}

/// Atomic write via temp file + rename, as with `device_ids.json`. Failure is
/// `debug` only: the cost of losing this is a vaguer bar next boot.
fn write_timings(path: &std::path::Path, steps: &HashMap<String, f64>) {
    let file = TimingFile {
        version: TIMING_FILE_VERSION,
        steps: steps.clone(),
    };
    let Ok(body) = serde_json::to_vec_pretty(&file) else {
        return;
    };
    let tmp = path.with_extension("json.tmp");
    if let Err(e) = std::fs::write(&tmp, body).and_then(|()| std::fs::rename(&tmp, path)) {
        debug!(error = %e, path = %path.display(), "failed to persist startup timings");
    }
}

// ── formatting ──────────────────────────────────────────────────────────────

fn secs(ms: f64) -> String {
    let s = ms / 1000.0;
    if s >= 60.0 {
        format!("{}m{:02.0}s", (s / 60.0).floor(), s % 60.0)
    } else if s >= 10.0 {
        format!("{s:.1}s")
    } else {
        format!("{s:.2}s")
    }
}

// ── live state ──────────────────────────────────────────────────────────────

/// The in-flight boot, if any. Set for the lifetime of a [`Boot`].
static ACTIVE: ArcSwapOption<Inner> = ArcSwapOption::const_empty();

struct Step {
    key: String,
    est_ms: f64,
}

/// The mutable half of the running phase. Touched a few dozen times per phase
/// plus once per [`TICK`] by the painter, so one mutex is enough.
struct PhaseInner {
    steps: Vec<Step>,
    total_est_ms: f64,
    /// Estimated ms accounted for by steps before `current`.
    done_est_ms: f64,
    /// Index of the running step, or `steps.len()` once the phase is closed.
    current: usize,
    current_started: Instant,
    /// Exact progress within the step, for the steps that can count themselves.
    explicit: Option<(u64, u64)>,
    /// Completed `(key, ms)`, folded into the timing file when the boot ends.
    measured: Vec<(String, f64)>,
}

struct PhaseState {
    key: String,
    label: String,
    bar: Option<ProgressBar>,
    started: Instant,
    inner: Mutex<PhaseInner>,
}

impl PhaseState {
    /// Estimated ms done, including the running step's reported or interpolated
    /// fraction.
    fn done_est_ms(&self) -> f64 {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::done_locked(&inner)
    }

    fn done_locked(inner: &PhaseInner) -> f64 {
        let Some(step) = inner.steps.get(inner.current) else {
            return inner.total_est_ms;
        };
        let frac = match inner.explicit {
            Some((done, total)) if total > 0 => (done as f64 / total as f64).min(1.0),
            _ => {
                let elapsed = inner.current_started.elapsed().as_secs_f64() * 1000.0;
                (elapsed / step.est_ms.max(1.0)).min(INTERP_CAP)
            }
        };
        inner.done_est_ms + step.est_ms * frac
    }
}

struct Inner {
    multi: Option<MultiProgress>,
    total_bar: Option<ProgressBar>,
    started: Instant,
    total_est_ms: f64,
    /// Estimated ms of *finished* phases.
    done_est_ms: AtomicU64,
    have_history: bool,
    label_width: usize,
    phase: ArcSwapOption<PhaseState>,
    /// Durations gathered from every finished phase.
    collected: Mutex<HashMap<String, f64>>,
    /// Finished phase bars. `MultiProgress` wipes a dropped bar's line at the
    /// next redraw, so releasing these erases the completed phases as soon as
    /// anything logs.
    finished_bars: Mutex<Vec<ProgressBar>>,
    running: AtomicBool,
}

impl Inner {
    fn done_est_ms(&self) -> f64 {
        let phases = self.done_est_ms.load(Ordering::Relaxed) as f64;
        phases + self.phase.load().as_ref().map_or(0.0, |p| p.done_est_ms())
    }

    /// `elapsed / ~total`, or `elapsed · estimating` with no history to go on.
    fn total_msg(&self) -> String {
        let elapsed = self.started.elapsed().as_secs_f64() * 1000.0;
        if !self.have_history {
            return format!("{} · estimating", secs(elapsed));
        }
        let done = self.done_est_ms();
        // Scale what's left by how far this boot is running ahead of or behind
        // its history, so a cold start lengthens the ETA instead of pinning the
        // bar just short of the end.
        let drift = if done > 250.0 {
            (elapsed / done).clamp(DRIFT_MIN, DRIFT_MAX)
        } else {
            1.0
        };
        let remaining = ((self.total_est_ms - done).max(0.0)) * drift;
        format!("{} / ~{}", secs(elapsed), secs(elapsed + remaining))
    }

    /// Repaint both bars. Called every [`TICK`] by the painter, and again
    /// whenever a phase opens or closes.
    fn paint(&self) {
        if let Some(phase) = self.phase.load().as_ref()
            && let Some(bar) = &phase.bar
        {
            let (total_est_ms, done, name, counted) = {
                let inner = phase
                    .inner
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let done = PhaseState::done_locked(&inner);
                let name = inner
                    .steps
                    .get(inner.current)
                    .map_or_else(String::new, |s| s.key.clone());
                let counted = match inner.explicit {
                    Some((d, t)) if t > 0 => format!(" {d}/{t}"),
                    _ => String::new(),
                };
                (inner.total_est_ms, done, name, counted)
            };
            bar.set_length(total_est_ms.max(1.0) as u64);
            bar.set_position(done as u64);
            bar.set_message(format!("{name}{counted}"));
        }
        if let Some(bar) = &self.total_bar {
            bar.set_length(self.total_est_ms.max(1.0) as u64);
            bar.set_position(self.done_est_ms() as u64);
            bar.set_message(self.total_msg());
        }
    }
}

// ── reporting from the work ─────────────────────────────────────────────────

/// Current resident set size in MiB, or `None` if it can't be read.
///
/// Linux: `/proc/self/statm`'s second field is the resident page count; the
/// page size is assumed to be 4096 bytes, which holds on every target we ship
/// to. macOS: `libc` is not a dependency of this crate, so shell out to
/// `ps -o rss=`, which reports KiB. The macOS path forks a process per call and
/// is only ever reached under `STEP_RSS=1`.
#[must_use]
pub fn rss_mib() -> Option<f64> {
    #[cfg(target_os = "linux")]
    {
        let statm = std::fs::read_to_string("/proc/self/statm").ok()?;
        let resident_pages: f64 = statm.split_whitespace().nth(1)?.parse().ok()?;
        Some(resident_pages * 4096.0 / (1024.0 * 1024.0))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let out = std::process::Command::new("ps")
            .args(["-o", "rss=", "-p"])
            .arg(std::process::id().to_string())
            .output()
            .ok()?;
        let kib: f64 = String::from_utf8_lossy(&out.stdout).trim().parse().ok()?;
        Some(kib / 1024.0)
    }
}

/// Whether `STEP_RSS=1` asked for the per-step memory trace. Read once.
fn step_rss_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("STEP_RSS").is_ok_and(|v| v == "1"))
}

/// Wall clock for the `step_rss` trace, started at the first [`step`] call.
fn step_rss_epoch() -> Instant {
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    *EPOCH.get_or_init(Instant::now)
}

/// One `step_rss` line on stderr, before the step's own bookkeeping.
///
/// Why this ships rather than living in a scratch patch: the CN load once
/// peaked at 5.3 GiB of RSS against a 542 MiB result, and because the peak is
/// transient and inside a step nobody could say *which* step allocated it -
/// reproducing it meant rebuilding an instrumented binary. Every load step
/// already calls [`step`], so hanging the measurement here turns "where did the
/// boot spend 5 GiB?" into a one-env-var question, in production, on the real
/// data, at no cost when the variable is unset.
fn trace_step_rss(key: &str) {
    if !step_rss_enabled() {
        return;
    }
    let elapsed_ms = step_rss_epoch().elapsed().as_millis();
    let rss = rss_mib().unwrap_or(f64::NAN);
    eprintln!("step_rss key={key} elapsed_ms={elapsed_ms} rss_mib={rss:.1}");
}

/// Enter a named step of the running phase, banking the previous step's
/// duration.
///
/// An undeclared key is appended at the default weight, so instrumenting a new
/// step without touching the plan costs accuracy for one boot rather than
/// breaking the bar. Does nothing when no boot is in flight.
pub fn step(key: &str) {
    trace_step_rss(key);
    let Some(inner) = ACTIVE.load_full() else {
        return;
    };
    let Some(phase) = inner.phase.load_full() else {
        return;
    };
    {
        let mut p = phase
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        close_current(&mut p, &phase.key);
        let at = p
            .steps
            .iter()
            .position(|s| s.key == key)
            .unwrap_or(usize::MAX);
        if at == usize::MAX {
            // Undeclared: append, growing the phase estimate rather than letting
            // the bar jump backwards.
            p.total_est_ms += UNMEASURED_MS;
            p.current = p.steps.len();
            p.steps.push(Step {
                key: key.to_owned(),
                est_ms: UNMEASURED_MS,
            });
        } else {
            // Recomputed, not accumulated, so steps jumped over - an optional
            // table that wasn't on disk - are credited and the bar doesn't stall
            // behind work that will never run.
            p.done_est_ms = p.steps[..at].iter().map(|s| s.est_ms).sum();
            p.current = at;
        }
        p.current_started = Instant::now();
        p.explicit = None;
    }
    inner.paint();
}

/// Report exact progress within the running step. Replaces the elapsed-time
/// interpolation until the next [`step`].
pub fn step_progress(done: u64, total: u64) {
    let Some(inner) = ACTIVE.load_full() else {
        return;
    };
    let Some(phase) = inner.phase.load_full() else {
        return;
    };
    phase
        .inner
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .explicit = Some((done, total));
}

/// Mark the running step finished and record what it took.
fn close_current(p: &mut PhaseInner, phase_key: &str) {
    let Some(step) = p.steps.get(p.current) else {
        return;
    };
    let ms = p.current_started.elapsed().as_secs_f64() * 1000.0;
    p.measured.push((format!("{phase_key}/{}", step.key), ms));
    p.done_est_ms += step.est_ms;
    p.current += 1;
}

// ── log routing ─────────────────────────────────────────────────────────────

/// A `tracing` writer that clears the bars before a log line goes out.
///
/// Without it the startup `info!`/`warn!` lines - the game data warnings, the
/// applied migrations - print over a half-drawn bar and shred the terminal.
/// Install with `tracing_subscriber::fmt().with_writer(startup::log_writer())`;
/// once the boot is over it costs one atomic load per line.
#[derive(Clone, Copy, Default)]
pub struct ProgressWriter;

pub const fn log_writer() -> ProgressWriter {
    ProgressWriter
}

impl Write for ProgressWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let multi = ACTIVE.load_full().and_then(|inner| inner.multi.clone());
        match multi {
            Some(multi) => multi.suspend(|| io::stdout().write(buf)),
            None => io::stdout().write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for ProgressWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        *self
    }
}

// ── the boot ────────────────────────────────────────────────────────────────

/// A boot in progress. Hold it for the length of startup, open one
/// [`PhaseGuard`] per phase, and [`finish`](Boot::finish) before serving.
pub struct Boot {
    inner: Arc<Inner>,
    plan: HashMap<String, PhaseSpec>,
    order: Vec<String>,
    timings: HashMap<String, f64>,
    timings_path: PathBuf,
    painter: Option<std::thread::JoinHandle<()>>,
    finished: bool,
}

impl Boot {
    /// Start reporting. Bars are drawn only on a terminal; pm2, Docker without
    /// `-t` and CI get one `tracing` line per phase, and `NO_PROGRESS=1` takes
    /// that path on a terminal too.
    pub fn start(phases: Vec<PhaseSpec>) -> Self {
        Self::start_with_timings(phases, timings_path())
    }

    /// As [`start`](Boot::start), with the timing file named rather than taken
    /// from `STARTUP_TIMINGS_FILE`. For callers that must not disturb the
    /// server's own history: the `boot_progress` example, the tests.
    pub fn start_with_timings(phases: Vec<PhaseSpec>, timings_path: PathBuf) -> Self {
        let timings = read_timings(&timings_path);
        let have_history = !timings.is_empty();

        let phase_est = |spec: &PhaseSpec| -> f64 {
            spec.steps
                .iter()
                .map(|s| est_for(&timings, &spec.key, s))
                .sum()
        };
        let total_est_ms: f64 = phases.iter().map(phase_est).sum();
        let label_width = phases
            .iter()
            .map(|p| p.label.chars().count())
            .max()
            .unwrap_or(LABEL_WIDTH)
            .max(LABEL_WIDTH);

        let interactive = io::stderr().is_terminal()
            && !std::env::var("NO_PROGRESS").is_ok_and(|v| v != "0" && !v.is_empty());

        let (multi, total_bar) = if interactive {
            let multi = MultiProgress::with_draw_target(ProgressDrawTarget::stderr());
            let header = if have_history {
                format!(
                    "myrtle backend · {} phases · ~{} by last boot",
                    phases.len(),
                    secs(total_est_ms)
                )
            } else {
                format!(
                    "myrtle backend · {} phases · no timings yet, this boot writes them",
                    phases.len()
                )
            };
            // One `println` per line: it counts the lines it must scroll past,
            // and an embedded newline corrupts that count for the rest of the
            // boot.
            let _ = multi.println("");
            let _ = multi.println(header);
            let _ = multi.println("");
            let bar = multi.add(
                ProgressBar::new(total_est_ms.max(1.0) as u64).with_style(total_style(label_width)),
            );
            bar.set_prefix("total");
            (Some(multi), Some(bar))
        } else {
            (None, None)
        };

        let inner = Arc::new(Inner {
            multi,
            total_bar,
            started: Instant::now(),
            total_est_ms,
            done_est_ms: AtomicU64::new(0),
            have_history,
            label_width,
            phase: ArcSwapOption::empty(),
            collected: Mutex::new(HashMap::new()),
            finished_bars: Mutex::new(Vec::new()),
            running: AtomicBool::new(true),
        });
        ACTIVE.store(Some(inner.clone()));

        // A painter thread rather than indicatif's steady tick, which only
        // redraws: most steps have nothing to count, so their share of the bar
        // has to be recomputed from elapsed time on every frame.
        let painter = interactive.then(|| {
            let inner = inner.clone();
            std::thread::spawn(move || {
                while inner.running.load(Ordering::Relaxed) {
                    inner.paint();
                    std::thread::sleep(TICK);
                }
            })
        });

        let order = phases.iter().map(|p| p.key.clone()).collect();
        let plan = phases.into_iter().map(|p| (p.key.clone(), p)).collect();
        Self {
            inner,
            plan,
            order,
            timings,
            timings_path,
            painter,
            finished: false,
        }
    }

    /// Open a phase, which runs until the returned guard is dropped. An
    /// unplanned key still works: it gets a bar, and all of its steps are
    /// appended as they are reported.
    pub fn phase(&self, key: &str) -> PhaseGuard {
        let (label, steps) = match self.plan.get(key) {
            Some(spec) => (
                spec.label.clone(),
                spec.steps
                    .iter()
                    .map(|s| Step {
                        key: s.key.to_string(),
                        est_ms: est_for(&self.timings, key, s),
                    })
                    .collect::<Vec<_>>(),
            ),
            None => (key.to_owned(), Vec::new()),
        };
        let total_est_ms = steps.iter().map(|s| s.est_ms).sum::<f64>();

        let bar = self.inner.multi.as_ref().map(|multi| {
            let bar = multi.insert_before(
                self.inner.total_bar.as_ref().expect("bars come in pairs"),
                ProgressBar::new(total_est_ms.max(1.0) as u64)
                    .with_style(phase_style(self.inner.label_width)),
            );
            bar.set_prefix(label.clone());
            bar.enable_steady_tick(TICK);
            bar
        });

        if bar.is_none() {
            info!(phase = %label, "starting");
        }

        self.inner.phase.store(Some(Arc::new(PhaseState {
            key: key.to_owned(),
            label,
            bar,
            started: Instant::now(),
            inner: Mutex::new(PhaseInner {
                steps,
                total_est_ms,
                done_est_ms: 0.0,
                current: 0,
                current_started: Instant::now(),
                explicit: None,
                measured: Vec::new(),
            }),
        })));

        PhaseGuard {
            inner: self.inner.clone(),
            expected_ms: total_est_ms,
        }
    }

    /// Stop painting, leave the phase list on screen with its real durations,
    /// and fold this boot's measurements into the timing file.
    pub fn finish(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        if std::mem::replace(&mut self.finished, true) {
            return;
        }
        self.inner.phase.store(None);
        self.inner.running.store(false, Ordering::Relaxed);
        if let Some(painter) = self.painter.take() {
            let _ = painter.join();
        }

        let elapsed = self.inner.started.elapsed().as_secs_f64() * 1000.0;
        if let Some(bar) = &self.inner.total_bar {
            bar.set_position(bar.length().unwrap_or(1));
            bar.set_style(done_style(self.inner.label_width));
            bar.finish_with_message(format!("ready in {}", secs(elapsed)));
            if let Some(multi) = &self.inner.multi {
                let _ = multi.println("");
            }
        } else {
            info!(elapsed = %secs(elapsed), "startup complete");
        }

        let collected = std::mem::take(
            &mut *self
                .inner
                .collected
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        for (key, ms) in collected {
            let smoothed = match self.timings.get(&key) {
                Some(prev) => prev.mul_add(1.0 - EWMA_ALPHA, EWMA_ALPHA * ms),
                None => ms,
            };
            self.timings.insert(key, smoothed);
        }
        // Drop keys the plan no longer declares, or a renamed step lingers and
        // inflates every later estimate. Only for phases in this plan, so
        // `SERVERS=en` keeps what is known about CN.
        let live: std::collections::HashSet<String> = self
            .order
            .iter()
            .filter_map(|k| self.plan.get(k))
            .flat_map(|spec| {
                spec.steps
                    .iter()
                    .map(move |s| format!("{}/{}", spec.key, s.key))
            })
            .collect();
        let planned: std::collections::HashSet<&str> =
            self.order.iter().map(String::as_str).collect();
        self.timings.retain(|k, _| {
            live.contains(k)
                || !k
                    .split_once('/')
                    .is_some_and(|(phase, _)| planned.contains(phase))
        });
        write_timings(&self.timings_path, &self.timings);

        ACTIVE.store(None);
    }
}

impl Drop for Boot {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// What a step is expected to cost: its last measurement, else its hint.
fn est_for(timings: &HashMap<String, f64>, phase: &str, step: &StepSpec) -> f64 {
    timings
        .get(&format!("{phase}/{}", step.key))
        .copied()
        .unwrap_or(step.hint_ms)
        .max(1.0)
}

/// Closes its phase on drop, so `let _phase = boot.phase(..)` is the whole call
/// site and an early return still ends the phase.
pub struct PhaseGuard {
    inner: Arc<Inner>,
    expected_ms: f64,
}

impl Drop for PhaseGuard {
    fn drop(&mut self) {
        let Some(phase) = self.inner.phase.swap(None) else {
            return;
        };
        let elapsed = phase.started.elapsed().as_secs_f64() * 1000.0;

        let measured = {
            let mut p = phase
                .inner
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            close_current(&mut p, &phase.key);
            std::mem::take(&mut p.measured)
        };
        {
            let mut collected = self
                .inner
                .collected
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for (key, ms) in measured {
                *collected.entry(key).or_insert(0.0) += ms;
            }
        }

        self.inner
            .done_est_ms
            .fetch_add(self.expected_ms as u64, Ordering::Relaxed);

        if let Some(bar) = &phase.bar {
            bar.disable_steady_tick();
            bar.set_style(done_style(self.inner.label_width));
            bar.finish_with_message(secs(elapsed));
            self.inner
                .finished_bars
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(bar.clone());
        } else {
            info!(
                phase = %phase.label,
                elapsed = %secs(elapsed),
                expected = %secs(self.expected_ms),
                "done"
            );
        }
        self.inner.paint();
    }
}

// ── styles ──────────────────────────────────────────────────────────────────

fn phase_style(width: usize) -> ProgressStyle {
    ProgressStyle::with_template(&format!(
        "  {{spinner:.cyan}} {{prefix:<{width}}} {{bar:28.cyan/blue}} {{percent:>3}}%  {{msg}}"
    ))
    .expect("static template")
    .progress_chars("━━╾╴")
    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
}

fn total_style(width: usize) -> ProgressStyle {
    ProgressStyle::with_template(&format!(
        "    {{prefix:<{width}}} {{bar:28.green/blue}} {{percent:>3}}%  {{msg}}"
    ))
    .expect("static template")
    .progress_chars("━━╾╴")
}

fn done_style(width: usize) -> ProgressStyle {
    ProgressStyle::with_template(&format!("  ✔ {{prefix:<{width}}} {{msg}}"))
        .expect("static template")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `ACTIVE` is process-global, so these tests run one at a time.
    static SERIAL: Mutex<()> = Mutex::new(());

    fn temp_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "myrtle_startup_timings_{}_{name}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&p);
        p
    }

    fn plan() -> Vec<PhaseSpec> {
        vec![PhaseSpec::new("p", "phase").with_steps(["a", "b", "c"])]
    }

    #[test]
    fn measured_steps_become_the_next_boots_estimate() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let path = temp_path("measured");

        let first = Boot::start_with_timings(plan(), path.clone());
        assert!(!first.inner.have_history, "nothing to go on yet");
        {
            let _phase = first.phase("p");
            step("a");
            std::thread::sleep(Duration::from_millis(30));
            step("b");
            step("c");
        }
        first.finish();

        let stored = read_timings(&path);
        assert_eq!(stored.len(), 3, "every declared step recorded: {stored:?}");
        assert!(
            stored["p/a"] >= 25.0,
            "a's 30ms sleep was timed: {stored:?}"
        );

        let second = Boot::start_with_timings(plan(), path.clone());
        assert!(second.inner.have_history);
        assert!(
            second.inner.total_est_ms >= 25.0,
            "the estimate is the measurement, not the hint: {}",
            second.inner.total_est_ms
        );
        second.finish();
        let _ = std::fs::remove_file(&path);
    }

    /// Instrumenting a step without adding it to the plan costs accuracy for one
    /// boot; it must not break the bar.
    #[test]
    fn an_undeclared_step_is_appended() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let path = temp_path("undeclared");

        let boot = Boot::start_with_timings(plan(), path.clone());
        {
            let _phase = boot.phase("p");
            step("a");
            step("surprise");
            let phase = boot.inner.phase.load_full().expect("phase is open");
            let inner = phase.inner.lock().unwrap();
            assert_eq!(inner.steps.len(), 4);
            assert_eq!(inner.steps[3].key, "surprise");
            assert_eq!(inner.current, 3);
        }
        boot.finish();

        // Dropped on the way out rather than left to inflate later estimates.
        let stored = read_timings(&path);
        assert!(!stored.contains_key("p/surprise"), "{stored:?}");
        let _ = std::fs::remove_file(&path);
    }

    /// An optional table that isn't on disk skips its step; the bar must jump
    /// over it rather than sit behind work that will never run.
    #[test]
    fn skipped_steps_are_credited() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let path = temp_path("skipped");

        let boot = Boot::start_with_timings(plan(), path.clone());
        {
            let _phase = boot.phase("p");
            step("a");
            step("c");
            let phase = boot.inner.phase.load_full().expect("phase is open");
            let inner = phase.inner.lock().unwrap();
            assert_eq!(inner.current, 2);
            assert!(
                inner.done_est_ms >= 2.0 * UNMEASURED_MS,
                "a and the skipped b are both banked: {}",
                inner.done_est_ms
            );
        }
        boot.finish();
        let _ = std::fs::remove_file(&path);
    }

    /// The reporting calls sit in code the tests, the `src/bin` tools and the
    /// hot-reload watcher all run, none of which install a boot.
    #[test]
    fn reporting_with_no_boot_is_inert() {
        let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        assert!(ACTIVE.load().is_none());
        step("unheard");
        step_progress(3, 7);
    }

    #[test]
    fn durations_format_by_magnitude() {
        assert_eq!(secs(432.0), "0.43s");
        assert_eq!(secs(12_340.0), "12.3s");
        assert_eq!(secs(95_000.0), "1m35s");
    }

    #[test]
    fn table_hint_scales_with_file_size() {
        let big = temp_path("hint");
        std::fs::write(&big, vec![0u8; 4 * 1024 * 1024]).expect("write");
        let ms = table_hint_ms(&big).expect("file exists");
        assert!(
            (50.0..200.0).contains(&ms),
            "4 MB at the nominal rate: {ms}"
        );
        assert!(table_hint_ms(std::path::Path::new("/nonexistent/x.json")).is_none());
        let _ = std::fs::remove_file(&big);
    }
}
