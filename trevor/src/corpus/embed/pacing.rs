//! Pacing a long run: the host-pressure guard, the duty cycle and the time limit.

use std::time::{Duration, Instant};

use anyhow::Result;

use super::RunStats;
use super::checkpoint::Checkpoint;
use crate::host::{cpu_some_avg10, mem_available_gib, online_cpus, probe, therm_warning};

/// Pauses a long embedding run while the machine is under pressure. Every
/// probe reads only what the OS already publishes (no sudo, no
/// `powermetrics`); a probe that cannot run is skipped, never a pause.
#[derive(Debug, Clone)]
pub struct Guard {
    /// Linux: pause while `MemAvailable` (/proc/meminfo) is below this, GiB.
    pub min_available_gib: f64,
    /// Linux: pause while /proc/pressure/cpu `some avg10` is above this, %.
    pub max_cpu_pressure: f64,
    /// Linux: pause while the 1-minute load average is above the number of
    /// online CPUs (the machine's, not this process's affinity mask).
    pub check_load: bool,
    /// macOS: pause while on battery power (`pmset -g batt`).
    pub pause_on_battery: bool,
    /// Seconds between probes while running.
    pub every_secs: f64,
    /// Seconds between probes while paused.
    pub pause_secs: f64,
}

impl Default for Guard {
    fn default() -> Self {
        Self {
            min_available_gib: 1.5,
            max_cpu_pressure: 40.0,
            check_load: true,
            pause_on_battery: true,
            every_secs: 10.0,
            pause_secs: 30.0,
        }
    }
}

impl Guard {
    /// Every reason to pause right now; empty when the run may continue.
    #[must_use]
    pub fn reasons(&self) -> Vec<String> {
        let mut why = Vec::new();
        if cfg!(target_os = "macos") {
            if let Some(t) = probe("/usr/bin/pmset", &["-g", "therm"])
                .as_deref()
                .and_then(therm_warning)
            {
                why.push(format!("pmset therm: {t}"));
            }
            if let Some(level) = probe("/usr/sbin/sysctl", &["-n", "kern.memorystatus_vm_pressure_level"])
                .and_then(|s| s.trim().parse::<u32>().ok())
                && level >= 2
            {
                why.push(format!("memory pressure level {level} (2 warn, 4 critical)"));
            }
            if self.pause_on_battery
                && probe("/usr/bin/pmset", &["-g", "batt"]).is_some_and(|s| s.contains("'Battery Power'"))
            {
                why.push("on battery power".to_owned());
            }
        } else if cfg!(target_os = "linux") {
            let read = |p: &str| std::fs::read_to_string(p).ok();
            if let Some(g) = read("/proc/meminfo").as_deref().and_then(mem_available_gib)
                && g < self.min_available_gib
            {
                why.push(format!("MemAvailable {g:.2} GiB < {:.2}", self.min_available_gib));
            }
            if let Some(p) = read("/proc/pressure/cpu").as_deref().and_then(cpu_some_avg10)
                && p > self.max_cpu_pressure
            {
                why.push(format!("cpu pressure some avg10 {p:.1} > {:.1}", self.max_cpu_pressure));
            }
            if self.check_load {
                let n = read("/proc/stat").as_deref().map_or(0, online_cpus);
                let load =
                    read("/proc/loadavg").and_then(|s| s.split_whitespace().next().and_then(|x| x.parse::<f64>().ok()));
                #[allow(clippy::cast_precision_loss)]
                if let Some(l) = load
                    && n > 0
                    && l > n as f64
                {
                    why.push(format!("load average {l:.2} > {n} CPUs"));
                }
            }
        }
        why
    }
}

/// Paces a run: the guard's pauses, the duty cycle's sleeps and the
/// `max_minutes` stop, with the seconds each took.
pub(super) struct Pacer<'a> {
    pub(super) guard: Option<&'a Guard>,
    pub(super) duty: f64,
    pub(super) deadline: Option<Instant>,
    pub(super) next_probe: Instant,
    pub(super) stats: RunStats,
}

impl Pacer<'_> {
    /// Before an embedding call: Err(reason) when the run must stop now, after
    /// syncing the checkpoint; otherwise waits out any pause and returns.
    pub(super) fn before(
        &mut self,
        ckpt: &mut Option<Checkpoint>,
        done: usize,
        total: usize,
    ) -> Result<std::result::Result<(), String>> {
        let over = |d: Option<Instant>| d.is_some_and(|d| Instant::now() >= d);
        if over(self.deadline) {
            return Ok(Err("max-minutes reached".to_owned()));
        }
        let Some(g) = self.guard else { return Ok(Ok(())) };
        if Instant::now() < self.next_probe {
            return Ok(Ok(()));
        }
        let mut last = String::new();
        let mut since_log = Instant::now();
        let mut paused_at: Option<Instant> = None;
        loop {
            let why = g.reasons();
            if why.is_empty() {
                break;
            }
            if let Some(c) = ckpt.as_mut() {
                c.sync()?;
            }
            let now = why.join("; ");
            if paused_at.is_none() {
                paused_at = Some(Instant::now());
                self.stats.pauses += 1;
            }
            if now != last || since_log.elapsed() >= Duration::from_secs(300) {
                eprintln!("guard: paused at {done}/{total}: {now}");
                last = now;
                since_log = Instant::now();
            }
            if over(self.deadline) {
                self.stats.paused_secs += paused_at.map_or(0.0, |t| t.elapsed().as_secs_f64());
                return Ok(Err("max-minutes reached while paused".to_owned()));
            }
            std::thread::sleep(Duration::from_secs_f64(g.pause_secs.max(0.1)));
        }
        if let Some(t) = paused_at {
            let s = t.elapsed().as_secs_f64();
            self.stats.paused_secs += s;
            eprintln!("guard: resumed after {s:.0}s");
        }
        self.next_probe = Instant::now() + Duration::from_secs_f64(g.every_secs.max(0.0));
        Ok(Ok(()))
    }

    /// After an embedding call that took `work`.
    pub(super) fn after(&mut self, work: Duration) {
        self.stats.busy_secs += work.as_secs_f64();
        if self.duty < 1.0 {
            let s = work.mul_f64((1.0 - self.duty) / self.duty);
            self.stats.duty_secs += s.as_secs_f64();
            std::thread::sleep(s);
        }
    }
}
