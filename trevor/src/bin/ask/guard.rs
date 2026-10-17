//! The worker's power guard (design/trevor-contract-v1.1.md section 2.1): every `TREVOR_GUARD_SECS` (5) seconds it
//! samples the host and trips when any one signal stays over its threshold for 3 samples in a row; it recovers once
//! every signal has stayed under 80% of its threshold for `TREVOR_GUARD_COOLDOWN_SECS` (300). `TREVOR_GUARD=0` turns it
//! off: no sampling, and the heartbeat carries no `guard` field, as before.
//!
//! Signals (a signal that cannot be read is left out, never counted as bad):
//! - Linux: PSI cpu and memory `some avg10` (/proc/pressure), MemAvailable (/proc/meminfo), the largest llama-server
//!   RSS (/proc/<pid>/status of each process named llama-server), the 1-minute load average per CPU, CPU steal between
//!   two samples (/proc/stat).
//! - macOS (dev): `kern.memorystatus_vm_pressure_level` (4 = critical trips it), free memory
//!   (`kern.memorystatus_level` percent of `hw.memsize`, the number `memory_pressure` prints), `pmset -g therm` warnings
//!   (any recorded warning trips it), the llama-server RSS (`ps`).
//!
//! The decision ([`Guard::observe`]) is a pure function of the samples and the clock, so it is unit-tested with
//! injected samples; [`sample`] is the only part that reads the host.

use std::time::Duration;

use serde::Serialize;

/// The thresholds (contract defaults; each overridable by its env variable).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GuardConfig {
    /// PSI cpu `some avg10` above it (percent).
    pub(crate) psi_cpu: f64,
    /// PSI memory `some avg10` above it (percent).
    pub(crate) psi_mem: f64,
    /// Available memory below it (MiB).
    pub(crate) mem_mb: f64,
    /// llama-server RSS above it (MiB).
    pub(crate) rss_mb: f64,
    /// 1-minute load average per CPU above it.
    pub(crate) load: f64,
    /// CPU steal percent above it; the contract names the signal without a threshold, so it trips nothing unless
    /// `TREVOR_GUARD_STEAL` is set (it is sampled and logged either way).
    pub(crate) steal: Option<f64>,
    /// Consecutive bad samples of one signal that trip the guard.
    pub(crate) need: u32,
    /// Seconds every signal must stay under 80% of its threshold before the guard resumes.
    pub(crate) cooldown_secs: u64,
    /// Time between samples.
    pub(crate) every: Duration,
}

impl Default for GuardConfig {
    fn default() -> Self {
        Self { psi_cpu: 60.0, psi_mem: 20.0, mem_mb: 800.0, rss_mb: 4500.0, load: 1.5, steal: None, need: 3,
               cooldown_secs: 300, every: Duration::from_secs(5) }
    }
}

impl GuardConfig {
    /// The guard from the environment; None under `TREVOR_GUARD=0`.
    pub(crate) fn from_env() -> Option<Self> {
        Self::from_lookup(trevor::env::var)
    }

    pub(crate) fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Option<Self> {
        if get("TREVOR_GUARD").as_deref().map(str::trim) == Some("0") {
            return None;
        }
        let num = |k: &str| get(k).and_then(|v| v.trim().parse::<f64>().ok()).filter(|v| v.is_finite() && *v >= 0.0);
        let d = Self::default();
        Some(Self {
            psi_cpu: num("TREVOR_GUARD_PSI_CPU").unwrap_or(d.psi_cpu),
            psi_mem: num("TREVOR_GUARD_PSI_MEM").unwrap_or(d.psi_mem),
            mem_mb: num("TREVOR_GUARD_MEM_MB").unwrap_or(d.mem_mb),
            rss_mb: num("TREVOR_GUARD_RSS_MB").unwrap_or(d.rss_mb),
            load: num("TREVOR_GUARD_LOAD").unwrap_or(d.load),
            steal: num("TREVOR_GUARD_STEAL"),
            need: d.need,
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            cooldown_secs: num("TREVOR_GUARD_COOLDOWN_SECS").map_or(d.cooldown_secs, |v| v as u64),
            every: num("TREVOR_GUARD_SECS").filter(|v| *v >= 0.05).map_or(d.every, Duration::from_secs_f64),
        })
    }
}

/// One reading of the host; None = not available on this platform or not readable now.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Sample {
    pub(crate) psi_cpu: Option<f64>,
    pub(crate) psi_mem: Option<f64>,
    pub(crate) mem_avail_mb: Option<f64>,
    pub(crate) rss_mb: Option<f64>,
    pub(crate) load_per_cpu: Option<f64>,
    pub(crate) steal_pct: Option<f64>,
    /// macOS `kern.memorystatus_vm_pressure_level`: 1 normal, 2 warning, 4 critical.
    pub(crate) mac_pressure: Option<u32>,
    /// macOS: a thermal or performance warning recorded by `pmset -g therm`.
    pub(crate) thermal: Option<bool>,
}

impl Sample {
    /// One line for the log.
    pub(crate) fn brief(&self) -> String {
        let f = |name: &str, v: Option<f64>| v.map(|v| format!("{name}={v:.1}"));
        [f("psi_cpu", self.psi_cpu), f("psi_mem", self.psi_mem), f("mem_avail_mb", self.mem_avail_mb),
         f("rss_mb", self.rss_mb), f("load_per_cpu", self.load_per_cpu), f("steal", self.steal_pct),
         self.mac_pressure.map(|v| format!("vm_pressure={v}")), self.thermal.map(|v| format!("thermal_warning={v}"))]
            .into_iter().flatten().collect::<Vec<_>>().join(" ")
    }
}

/// The signals, in the order a trip reason names them.
const SIGNALS: [Signal; 8] = [Signal::MemAvail, Signal::Rss, Signal::PsiMem, Signal::MacPressure, Signal::PsiCpu,
                              Signal::Load, Signal::Steal, Signal::Thermal];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Signal {
    PsiCpu,
    PsiMem,
    MemAvail,
    Rss,
    Load,
    Steal,
    MacPressure,
    Thermal,
}

/// A signal's reading against its threshold.
struct Reading {
    /// Over the threshold.
    bad: bool,
    /// Under 80% of the threshold (for a floor: above the threshold / 0.8).
    calm: bool,
    text: String,
}

impl Signal {
    fn name(self) -> &'static str {
        match self {
            Self::PsiCpu => "psi_cpu",
            Self::PsiMem => "psi_mem",
            Self::MemAvail => "mem_avail_mb",
            Self::Rss => "llama_rss_mb",
            Self::Load => "load_per_cpu",
            Self::Steal => "cpu_steal",
            Self::MacPressure => "vm_pressure_level",
            Self::Thermal => "thermal_warning",
        }
    }

    /// A memory signal: tripping on it stops the llama-server.
    fn memory(self) -> bool {
        matches!(self, Self::PsiMem | Self::MemAvail | Self::Rss | Self::MacPressure)
    }

    fn read(self, s: &Sample, c: &GuardConfig) -> Option<Reading> {
        let above = |v: f64, t: f64, name: &str| Reading { bad: v > t, calm: v < 0.8 * t,
                                                           text: format!("{name} {v:.1} > {t}") };
        match self {
            Self::PsiCpu => s.psi_cpu.map(|v| above(v, c.psi_cpu, self.name())),
            Self::PsiMem => s.psi_mem.map(|v| above(v, c.psi_mem, self.name())),
            Self::Rss => s.rss_mb.map(|v| above(v, c.rss_mb, self.name())),
            Self::Load => s.load_per_cpu.map(|v| above(v, c.load, self.name())),
            Self::Steal => s.steal_pct.zip(c.steal).map(|(v, t)| above(v, t, self.name())),
            Self::MemAvail => s.mem_avail_mb.map(|v| Reading {
                bad: v < c.mem_mb, calm: v > c.mem_mb / 0.8, text: format!("{} {v:.0} < {}", self.name(), c.mem_mb)
            }),
            Self::MacPressure => s.mac_pressure.map(|v| Reading { bad: v >= 4, calm: f64::from(v) < 0.8 * 4.0,
                                                                  text: format!("{} {v} (critical)", self.name()) }),
            Self::Thermal => s.thermal.map(|w| Reading { bad: w, calm: !w,
                                                         text: format!("{} recorded (pmset -g therm)", self.name()) }),
        }
    }
}

/// A trip: why, since when (unix seconds), and whether a memory signal tripped it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Trip {
    pub(crate) reason: String,
    pub(crate) since: u64,
    pub(crate) memory: bool,
}

/// What one sample changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Change {
    /// The guard tripped, or (while tripped) a memory signal now trips it too.
    Tripped(Trip),
    Recovered,
}

/// The guard's decision state.
#[derive(Debug)]
pub(crate) struct Guard {
    cfg: GuardConfig,
    /// Consecutive bad samples per signal, in [`SIGNALS`] order.
    runs: [u32; SIGNALS.len()],
    trip: Option<Trip>,
    /// The first of the current run of calm samples while tripped.
    calm_since: Option<u64>,
}

impl Guard {
    pub(crate) fn new(cfg: GuardConfig) -> Self {
        Self { cfg, runs: [0; SIGNALS.len()], trip: None, calm_since: None }
    }

    pub(crate) fn trip(&self) -> Option<&Trip> {
        self.trip.as_ref()
    }

    /// Take one sample at `now` (unix seconds).
    pub(crate) fn observe(&mut self, s: &Sample, now: u64) -> Option<Change> {
        let readings: Vec<Option<Reading>> = SIGNALS.iter().map(|sig| sig.read(s, &self.cfg)).collect();
        let mut held: Vec<(Signal, &Reading)> = Vec::new();
        for (i, r) in readings.iter().enumerate() {
            self.runs[i] = match r {
                Some(r) if r.bad => self.runs[i] + 1,
                _ => 0,
            };
            if let Some(r) = r.as_ref().filter(|_| self.runs[i] >= self.cfg.need) {
                held.push((SIGNALS[i], r));
            }
        }
        let need = self.cfg.need;
        let reason = |h: &[(Signal, &Reading)]| h.iter().map(|(_, r)| format!("{} for {need} samples", r.text))
            .collect::<Vec<_>>().join("; ");
        match &mut self.trip {
            None => {
                if held.is_empty() {
                    return None;
                }
                let t = Trip { reason: reason(&held), since: now, memory: held.iter().any(|(sig, _)| sig.memory()) };
                self.trip = Some(t.clone());
                self.calm_since = None;
                Some(Change::Tripped(t))
            }
            Some(t) => {
                let mem: Vec<(Signal, &Reading)> = held.iter().copied().filter(|(sig, _)| sig.memory()).collect();
                if !t.memory && !mem.is_empty() {
                    t.memory = true;
                    t.reason = format!("{}; then {}", t.reason, reason(&mem));
                    self.calm_since = None;
                    return Some(Change::Tripped(t.clone()));
                }
                if readings.iter().flatten().all(|r| r.calm) {
                    let start = *self.calm_since.get_or_insert(now);
                    if now.saturating_sub(start) >= self.cfg.cooldown_secs {
                        self.trip = None;
                        self.calm_since = None;
                        self.runs = [0; SIGNALS.len()];
                        return Some(Change::Recovered);
                    }
                } else {
                    self.calm_since = None;
                }
                None
            }
        }
    }
}

/// The guard as the worker shares it: the heartbeat and the job loop read it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GuardView {
    pub(crate) trip: Option<Trip>,
}

/// The heartbeat's `guard` object (contract v1.1 section 2.1).
#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct GuardStatus {
    pub(crate) state: &'static str,
    pub(crate) reason: Option<String>,
    pub(crate) since: Option<u64>,
}

impl GuardView {
    pub(crate) fn tripped(&self) -> bool {
        self.trip.is_some()
    }

    pub(crate) fn status(&self) -> GuardStatus {
        match &self.trip {
            None => GuardStatus { state: "ok", reason: None, since: None },
            Some(t) => GuardStatus { state: "tripped", reason: Some(t.reason.clone()), since: Some(t.since) },
        }
    }
}

/// Run the guard: sample every `cfg.every`, publish each change on `tx`. Returns when every receiver is gone.
pub(crate) async fn run(cfg: GuardConfig, tx: tokio::sync::watch::Sender<GuardView>) {
    let log = trevor::env::is("TREVOR_GUARD_LOG", "1");
    let every = cfg.every;
    let mut guard = Guard::new(cfg);
    let mut sampler = Sampler::default();
    loop {
        let (s, back) = tokio::task::spawn_blocking(move || {
            let s = sampler.sample();
            (s, sampler)
        }).await.unwrap_or_default();
        sampler = back;
        let now = trevor::util::unix_now();
        let change = guard.observe(&s, now);
        if log {
            eprintln!("guard: {} -> {}", s.brief(), if guard.trip().is_some() { "tripped" } else { "ok" });
        }
        match &change {
            Some(Change::Tripped(t)) => eprintln!("guard: TRIPPED ({}){}", t.reason,
                                                  if t.memory { "; memory: the llama-server stops" } else { "" }),
            Some(Change::Recovered) => eprintln!("guard: recovered after {} s calm", guard.cfg.cooldown_secs),
            None => {}
        }
        if change.is_some() && tx.send(GuardView { trip: guard.trip().cloned() }).is_err() {
            return;
        }
        if tx.is_closed() {
            return;
        }
        tokio::time::sleep(every).await;
    }
}


/// The host reader; keeps the previous /proc/stat totals for the steal rate.
#[derive(Default)]
pub(crate) struct Sampler {
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    prev_stat: Option<(u64, u64)>,
}

impl Sampler {
    #[cfg(target_os = "linux")]
    pub(crate) fn sample(&mut self) -> Sample {
        let read = |p: &str| std::fs::read_to_string(p).ok();
        let cpus = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        let steal = read("/proc/stat").as_deref().and_then(trevor::host::parse_stat_steal).and_then(|now| {
            let prev = self.prev_stat.replace(now);
            prev.and_then(|p| trevor::host::steal_pct(p, now))
        });
        Sample {
            psi_cpu: read("/proc/pressure/cpu").as_deref().and_then(trevor::host::parse_psi_some_avg10),
            psi_mem: read("/proc/pressure/memory").as_deref().and_then(trevor::host::parse_psi_some_avg10),
            mem_avail_mb: read("/proc/meminfo").as_deref()
                .and_then(|t| trevor::host::parse_kb_field(t, "MemAvailable:")).map(|kb| kb / 1024.0),
            rss_mb: linux_llama_rss_mb(),
            #[allow(clippy::cast_precision_loss)]
            load_per_cpu: read("/proc/loadavg").as_deref()
                .and_then(|t| t.split_whitespace().next()?.parse::<f64>().ok())
                .map(|l| l / cpus as f64),
            steal_pct: steal,
            mac_pressure: None,
            thermal: None,
        }
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn sample(&mut self) -> Sample {
        let sysctl = |k: &str| trevor::host::run_cmd("sysctl", &["-n", k]);
        let level = sysctl("kern.memorystatus_level").and_then(|t| t.trim().parse::<f64>().ok());
        let mem = sysctl("hw.memsize").and_then(|t| t.trim().parse::<f64>().ok());
        Sample {
            mac_pressure: sysctl("kern.memorystatus_vm_pressure_level").and_then(|t| t.trim().parse().ok()),
            mem_avail_mb: level.zip(mem).map(|(pct, bytes)| pct / 100.0 * bytes / 1_048_576.0),
            thermal: trevor::host::run_cmd("pmset", &["-g", "therm"]).map(|t| trevor::host::pmset_warning(&t)),
            rss_mb: mac_llama_rss_mb(),
            ..Sample::default()
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    pub(crate) fn sample(&mut self) -> Sample {
        Sample::default()
    }
}

/// The largest RSS of the processes named exactly `llama-server` (`pgrep -x`, never a pattern), MiB.
#[cfg(target_os = "macos")]
fn mac_llama_rss_mb() -> Option<f64> {
    let pids = trevor::host::run_cmd("pgrep", &["-x", "llama-server"])?;
    pids.split_whitespace()
        .filter_map(|p| trevor::host::run_cmd("ps", &["-o", "rss=", "-p", p])?.trim().parse::<f64>().ok())
        .map(|kb| kb / 1024.0).reduce(f64::max)
}

#[cfg(target_os = "linux")]
fn linux_llama_rss_mb() -> Option<f64> {
    std::fs::read_dir("/proc").ok()?.filter_map(Result::ok)
        .filter(|e| e.file_name().to_str().is_some_and(|n| n.bytes().all(|b| b.is_ascii_digit())))
        .filter(|e| std::fs::read_to_string(e.path().join("comm")).is_ok_and(|c| c.trim() == "llama-server"))
        .filter_map(|e| trevor::host::parse_kb_field(&std::fs::read_to_string(e.path().join("status")).ok()?, "VmRSS:"))
        .map(|kb| kb / 1024.0).reduce(f64::max)
}
