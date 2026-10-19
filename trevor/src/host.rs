//! Readers of what the host OS publishes about load: /proc files on Linux, `pmset` and `sysctl` output on macOS.
//! Two guards use them with their own policies: the embedding run's pause guard (`corpus::embed::Guard`) and the
//! worker's power guard (`ask`'s guard.rs). Each guard keeps the readers it was written with, so where two readers
//! look alike they are kept apart on purpose:
//!
//! - [`probe`] passes the command's stderr through; [`run_cmd`] discards it.
//! - [`therm_warning`] (embed) counts any "warning level" line; [`pmset_warning`] (worker) only a level above 0.
//! - [`cpu_some_avg10`] (embed) takes the first `avg10=` field that parses; [`parse_psi_some_avg10`] (worker) the
//!   first `avg10=` field.
//! - [`mem_available_gib`] (embed) reads the second whitespace field of the `MemAvailable:` line;
//!   [`parse_kb_field`] (worker) the first field after the prefix.

// ---- the embedding run's pause guard ----

/// The stdout of `prog args` when it exits 0 (stderr passes through), else None.
#[must_use]
pub fn probe(prog: &str, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new(prog).args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// A recorded warning in `pmset -g therm` output, or None. Apple Silicon
/// prints "Note: No thermal warning level has been recorded" (and the same for
/// performance warnings and CPU power status) when nothing is recorded; any
/// other line naming a warning level or power status, or a `*_Limit` below
/// 100 (Intel's CPU_Speed_Limit), is a reason to pause.
#[must_use]
pub fn therm_warning(text: &str) -> Option<String> {
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with("Note: No") {
            continue;
        }
        let lower = line.to_ascii_lowercase();
        if lower.contains("warning level") || lower.contains("power status") {
            return Some(line.to_owned());
        }
        if let Some((k, v)) = line.split_once('=')
            && k.trim().ends_with("_Limit")
            && v.trim().parse::<f64>().is_ok_and(|x| x < 100.0)
        {
            return Some(line.to_owned());
        }
    }
    None
}

/// `MemAvailable` from /proc/meminfo, in GiB.
#[must_use]
pub fn mem_available_gib(meminfo: &str) -> Option<f64> {
    let line = meminfo.lines().find(|l| l.starts_with("MemAvailable:"))?;
    let kb: f64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb / (1024.0 * 1024.0))
}

/// `some avg10` from /proc/pressure/cpu, in percent.
#[must_use]
pub fn cpu_some_avg10(psi: &str) -> Option<f64> {
    let line = psi.lines().find(|l| l.starts_with("some "))?;
    line.split_whitespace()
        .find_map(|f| f.strip_prefix("avg10=")?.parse().ok())
}

/// The online CPUs counted in /proc/stat (`cpu0`, `cpu1`, ...).
#[must_use]
pub fn online_cpus(stat: &str) -> usize {
    stat.lines()
        .filter(|l| {
            l.strip_prefix("cpu")
                .is_some_and(|r| r.starts_with(|c: char| c.is_ascii_digit()))
        })
        .count()
}

// ---- the worker's power guard ----

/// The stdout of `cmd args` when it exits 0 (stderr discarded), else None.
#[must_use]
pub fn run_cmd(cmd: &str, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new(cmd)
        .args(args)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// `some avg10=` of a /proc/pressure file.
pub fn parse_psi_some_avg10(text: &str) -> Option<f64> {
    let line = text.lines().find(|l| l.starts_with("some "))?;
    line.split_whitespace()
        .find_map(|f| f.strip_prefix("avg10="))?
        .parse()
        .ok()
}

/// A `Name:   123 kB` field of /proc/meminfo or /proc/<pid>/status, in kB.
pub fn parse_kb_field(text: &str, field: &str) -> Option<f64> {
    text.lines()
        .find_map(|l| l.strip_prefix(field))?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

/// (steal, total) jiffies of the aggregate `cpu` line of /proc/stat.
pub fn parse_stat_steal(text: &str) -> Option<(u64, u64)> {
    let line = text.lines().find(|l| l.starts_with("cpu "))?;
    let v: Vec<u64> = line.split_whitespace().skip(1).filter_map(|f| f.parse().ok()).collect();
    // user nice system idle iowait irq softirq steal [guest guest_nice]; guest time is already in user and nice.
    let total: u64 = v.iter().take(8).sum();
    Some((*v.get(7)?, total))
}

/// Steal percent between two /proc/stat readings.
pub fn steal_pct(prev: (u64, u64), now: (u64, u64)) -> Option<f64> {
    let dt = now.1.checked_sub(prev.1).filter(|d| *d > 0)?;
    let ds = now.0.saturating_sub(prev.0);
    #[allow(clippy::cast_precision_loss)]
    Some(ds as f64 * 100.0 / dt as f64)
}

/// Whether `pmset -g therm` recorded a warning: a `*_Limit = N` line under 100 (100 = no throttling), a "warning level"
/// line above 0, or another line naming a warning; the "Note: No ... has been recorded" lines, the "CPU Power notify"
/// header and the other `key = value` lines (CPU_Available_CPUs) are not.
pub fn pmset_warning(text: &str) -> bool {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("Note:"))
        .any(|l| {
            let num = l
                .trim_end_matches('.')
                .rsplit([' ', '=', '\t'])
                .next()
                .and_then(|n| n.trim().parse::<i64>().ok());
            let low = l.to_lowercase();
            if let Some((key, _)) = l.split_once('=') {
                return key.trim().ends_with("_Limit") && num.is_some_and(|n| n < 100);
            }
            if low.contains("level")
                && let Some(n) = num
            {
                return n > 0;
            }
            low.contains("warning")
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_parsers() {
        let none = "Note: No thermal warning level has been recorded\n\
                    Note: No performance warning level has been recorded\n\
                    Note: No CPU power status has been recorded\n";
        assert_eq!(therm_warning(none), None);
        assert!(therm_warning("Thermal warning level: 2\n").is_some());
        assert!(therm_warning("CPU_Speed_Limit \t= 80\n").is_some());
        assert_eq!(therm_warning("CPU_Speed_Limit \t= 100\n"), None);
        let mi = "MemTotal:       11534336 kB\nMemFree:  100 kB\nMemAvailable:    1572864 kB\n";
        assert!((mem_available_gib(mi).unwrap() - 1.5).abs() < 1e-9);
        let psi = "some avg10=41.50 avg60=3.00 avg300=1.00 total=1\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=0\n";
        assert!((cpu_some_avg10(psi).unwrap() - 41.5).abs() < 1e-9);
        let stat = "cpu  1 2 3\ncpu0 1 2 3\ncpu1 1 2 3\ncpu2 1 2 3\nintr 5\nctxt 6\n";
        assert_eq!(online_cpus(stat), 3);
    }

    #[test]
    fn guard_parses_proc_and_pmset() {
        let psi = "some avg10=12.34 avg60=5.00 avg300=1.00 total=123\n\
                   full avg10=99.00 avg60=0.00 avg300=0.00 total=0\n";
        assert_eq!(parse_psi_some_avg10(psi), Some(12.34));
        let meminfo = "MemTotal:        8000000 kB\nMemFree:          100000 kB\nMemAvailable:     819200 kB\n";
        assert_eq!(parse_kb_field(meminfo, "MemAvailable:"), Some(819_200.0));
        assert_eq!(
            parse_kb_field("Name:\tllama-server\nVmRSS:\t 4718592 kB\n", "VmRSS:"),
            Some(4_718_592.0)
        );
        let a = parse_stat_steal("cpu  100 0 50 800 10 0 0 40 0 0\ncpu0 1 2 3\n").unwrap();
        let b = parse_stat_steal("cpu  150 0 70 860 10 0 0 60 0 0\n").unwrap();
        assert_eq!((a, b), ((40, 1000), (60, 1150)));
        assert_eq!(steal_pct(a, b), Some(20.0 * 100.0 / 150.0));
        assert_eq!(steal_pct(a, a), None);
        // An idle Mac: three Notes, no warning.
        let quiet = "Note: No thermal warning level has been recorded\n\
                     Note: No performance warning level has been recorded\n\
                     Note: No CPU power status has been recorded\n";
        assert!(!pmset_warning(quiet));
        assert!(pmset_warning("Thermal warning level set to 1."));
        assert!(
            !pmset_warning(
                "CPU Power notify\n\tCPU_Scheduler_Limit \t= 100\n\t\
                                CPU_Available_CPUs \t= 15\n\tCPU_Speed_Limit \t= 100"
            ),
            "limits of 100 are no throttling, but CPU_Available_CPUs = 15 is not a limit"
        );
        assert!(pmset_warning("CPU_Speed_Limit \t= 70"));
        assert!(!pmset_warning("Thermal warning level set to 0."));
    }
}
