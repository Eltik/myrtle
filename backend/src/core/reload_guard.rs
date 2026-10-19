//! The memory gate in front of every full game-data build.
//!
//! A hot reload (`asset_watcher::perform_reload`, on an asset pipeline
//! `update_complete` or a sidecar job's fallback) and a lazy server's load
//! (`app::residency`) each build a second `GameData`-sized allocation while
//! the old one is still served: measured peaks of 0.84 GiB on EN and 1.76 GiB
//! on CN (memory notes, 2026-09). On the 11 GB VPS the nightly Trevor window
//! holds the 12B at 7.3 to 7.6 GB resident with 0.6 to 1.5 GB of margin, so a
//! reload inside the window pushes the box into swap or the OOM killer.
//!
//! The rule: a build does not START while Linux `MemAvailable` is below
//! `RELOAD_MIN_FREE_MB` for that server, or while the Trevor model lock
//! (`TREVOR_LLM_LOCK`, default `/run/trevor/llm.lock`, ask's `<pid> <label>`
//! convention) names a live process. A refused hot reload is retried every
//! `RELOAD_RETRY_SECS` until it runs (`asset_watcher::defer_reload`); a refused
//! lazy load answers the 503 plus `Retry-After` the lazy path already answers
//! while loading. Every refusal logs one line; a wait longer than
//! `RELOAD_MAX_WAIT_SECS` posts one Discord alert through `core::alerts`.
//!
//! Where `/proc/meminfo` cannot be read (macOS, the development machine) the
//! memory half is inert and says so once in the log; the lock half still runs.
//! `RELOAD_GUARD=0` admits every build without reading anything, as before.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, Once};
use std::time::{Duration, Instant};

use crate::core::hypergryph::constants::Server;

/// The model lock's path when `TREVOR_LLM_LOCK` is unset or empty: the VPS's
/// (`trevor/deploy/trevor.env.example`).
pub const DEFAULT_LOCK_PATH: &str = "/run/trevor/llm.lock";

/// How often a refused hot reload is tried again.
const DEFAULT_RETRY_SECS: u64 = 300;

/// How long a refused build may wait before one Discord alert: 6 hours, the
/// length of the nightly window, so a reload held only by the window never
/// alerts and one held past it does.
const DEFAULT_MAX_WAIT_SECS: u64 = 6 * 3600;

/// `RELOAD_MIN_FREE_MB`'s default per server: the measured peak of the second
/// copy a build holds, rounded up (EN 0.84 GiB -> 900 MiB, CN 1.76 GiB -> 1800
/// MiB). JP, KR and TW were not measured; 1200 sits between the two, a TRADE
/// rather than a measurement. Bilibili shares CN's cell.
#[must_use]
pub const fn default_min_free_mb(server: Server) -> u64 {
    match server {
        Server::EN => 900,
        Server::CN | Server::Bilibili => 1800,
        Server::JP | Server::KR | Server::TW => 1200,
    }
}

/// Which build is asking, for the log line and the waiting clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadKind {
    /// A hot reload of a resident (or failed) server.
    Reload,
    /// A lazy server's load, started by a request.
    LazyLoad,
}

impl LoadKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reload => "reload",
            Self::LazyLoad => "lazy load",
        }
    }
}

/// The guard's variables, read once at boot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardConfig {
    /// `RELOAD_GUARD`; `0` turns the guard off.
    pub enabled: bool,
    /// A bare `RELOAD_MIN_FREE_MB` value: every server's floor.
    all_mb: Option<u64>,
    /// `server=mb` items of `RELOAD_MIN_FREE_MB`, which win over `all_mb`.
    per_server_mb: Vec<(Server, u64)>,
    /// The model lock (`TREVOR_LLM_LOCK`).
    pub lock_path: PathBuf,
    /// `RELOAD_RETRY_SECS`.
    pub retry: Duration,
    /// `RELOAD_MAX_WAIT_SECS`; `None` (`0`) never alerts.
    pub max_wait: Option<Duration>,
}

impl GuardConfig {
    #[must_use]
    pub fn from_env() -> Self {
        Self::parse(|key| std::env::var(key).ok())
    }

    /// Pure core of [`GuardConfig::from_env`]. `RELOAD_MIN_FREE_MB` is either
    /// one number for every server (`1500`), `server=mb` items (`cn=2000`), or
    /// both (`1500,cn=2000`); `0` turns the memory half off. An unparseable
    /// item is ignored rather than failing the boot.
    #[must_use]
    pub fn parse(get: impl Fn(&str) -> Option<String>) -> Self {
        let enabled = get("RELOAD_GUARD").is_none_or(|v| v.trim() != "0");
        let mut all_mb = None;
        let mut per_server_mb = Vec::new();
        for item in get("RELOAD_MIN_FREE_MB").unwrap_or_default().split(',') {
            let item = item.trim();
            if let Some((server, mb)) = item.split_once('=') {
                if let (Some(server), Ok(mb)) = (Server::parse(server), mb.trim().parse()) {
                    per_server_mb.push((server, mb));
                }
            } else if let Ok(mb) = item.parse() {
                all_mb = Some(mb);
            }
        }
        let lock_path = get("TREVOR_LLM_LOCK")
            .filter(|p| !p.trim().is_empty())
            .map_or_else(|| PathBuf::from(DEFAULT_LOCK_PATH), PathBuf::from);
        let secs = |key: &str| get(key).and_then(|v| v.trim().parse::<u64>().ok());
        let retry = secs("RELOAD_RETRY_SECS")
            .filter(|s| *s > 0)
            .unwrap_or(DEFAULT_RETRY_SECS);
        let max_wait = secs("RELOAD_MAX_WAIT_SECS").unwrap_or(DEFAULT_MAX_WAIT_SECS);
        Self {
            enabled,
            all_mb,
            per_server_mb,
            lock_path,
            retry: Duration::from_secs(retry),
            max_wait: (max_wait > 0).then(|| Duration::from_secs(max_wait)),
        }
    }

    /// The `MemAvailable` floor for `server`, in MiB; `0` = no memory check.
    #[must_use]
    pub fn min_free_mb(&self, server: Server) -> u64 {
        let canonical = |s: Server| if s == Server::Bilibili { Server::CN } else { s };
        self.per_server_mb
            .iter()
            .rev()
            .find(|(s, _)| canonical(*s) == canonical(server))
            .map(|(_, mb)| *mb)
            .or(self.all_mb)
            .unwrap_or_else(|| default_min_free_mb(server))
    }
}

static CONFIG: LazyLock<GuardConfig> = LazyLock::new(GuardConfig::from_env);

/// A test's stand-in for the environment; `None` reads [`CONFIG`].
#[cfg(test)]
pub(crate) static TEST_CONFIG: Mutex<Option<GuardConfig>> = Mutex::new(None);

/// The process's guard configuration.
#[must_use]
pub fn config() -> GuardConfig {
    #[cfg(test)]
    if let Some(c) = TEST_CONFIG.lock().ok().and_then(|g| g.clone()) {
        return c;
    }
    CONFIG.clone()
}

/// Why a build was not started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    LowMemory { available_mb: u64, need_mb: u64 },
    ModelBusy { pid: u32, label: String },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LowMemory {
                available_mb,
                need_mb,
            } => write!(
                f,
                "only {available_mb} MiB available, need {need_mb} MiB (RELOAD_MIN_FREE_MB)"
            ),
            Self::ModelBusy { pid, label } => {
                let label = if label.is_empty() {
                    "a model job"
                } else {
                    label
                };
                write!(f, "the Trevor model lock is held by {label} (pid {pid})")
            }
        }
    }
}

/// `MemAvailable` from the text of `/proc/meminfo`, in MiB.
#[must_use]
pub fn mem_available_mb(meminfo: &str) -> Option<u64> {
    meminfo.lines().find_map(|line| {
        let rest = line.strip_prefix("MemAvailable:")?;
        let mut parts = rest.split_whitespace();
        let value: u64 = parts.next()?.parse().ok()?;
        match parts.next() {
            Some("kB") | None => Some(value / 1024),
            Some(_) => None,
        }
    })
}

/// The model lock's text as (pid, label): pid first, then a label, the
/// convention ask and the update script write. `None` for anything else.
#[must_use]
pub fn parse_lock(text: &str) -> Option<(u32, String)> {
    let text = text.trim();
    let (pid, label) = text.split_once(char::is_whitespace).unwrap_or((text, ""));
    Some((pid.parse().ok()?, label.trim().to_owned()))
}

/// The lock's holder when the file exists, parses, and names a live process.
/// A missing, unreadable or unparseable file, or a dead pid (a job that died
/// without its cleanup), means the model is free.
pub fn lock_holder(path: &Path, alive: impl Fn(u32) -> bool) -> Option<(u32, String)> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            static WARNED: Once = Once::new();
            WARNED.call_once(|| {
                tracing::warn!(
                    path = %path.display(),
                    error = %e,
                    "the Trevor model lock exists but cannot be read; the reload guard treats the model as free"
                );
            });
            return None;
        }
    };
    parse_lock(&text).filter(|(pid, _)| alive(*pid))
}

/// Whether process `pid` exists. `/proc/<pid>` where there is a `/proc`
/// (Linux, no fork); else `ps -p`, as `trevor/src/bin/ask/llama_server.rs`
/// does, which also sees another user's process and is true when `ps` cannot
/// run.
#[must_use]
pub fn pid_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    if Path::new("/proc/self").exists() {
        return Path::new(&format!("/proc/{pid}")).exists();
    }
    std::process::Command::new("ps")
        .args(["-p", &pid.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()
        .is_none_or(|s| s.success())
}

/// `MemAvailable` of this machine in MiB; `None` where `/proc/meminfo` is
/// missing or has no such line, which is logged once.
fn read_mem_available() -> Option<u64> {
    let found = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .as_deref()
        .and_then(mem_available_mb);
    if found.is_none() {
        static LOGGED: Once = Once::new();
        LOGGED.call_once(|| {
            tracing::info!(
                "MemAvailable cannot be read here (no /proc/meminfo); the reload guard checks only the Trevor model lock"
            );
        });
    }
    found
}

/// The decision, pure: refuse while the model lock names a live holder, or
/// while `MemAvailable` is known and below a non-zero floor; admit otherwise.
/// A disabled guard admits everything.
///
/// # Errors
/// The [`Refusal`] that holds the build back.
pub fn evaluate(
    cfg: &GuardConfig,
    server: Server,
    available_mb: Option<u64>,
    holder: Option<(u32, String)>,
) -> Result<(), Refusal> {
    if !cfg.enabled {
        return Ok(());
    }
    if let Some((pid, label)) = holder {
        return Err(Refusal::ModelBusy { pid, label });
    }
    let need_mb = cfg.min_free_mb(server);
    match available_mb {
        Some(available_mb) if need_mb > 0 && available_mb < need_mb => Err(Refusal::LowMemory {
            available_mb,
            need_mb,
        }),
        _ => Ok(()),
    }
}

/// [`evaluate`] against this machine: the lock file and `/proc/meminfo`.
/// Reads nothing when the guard is off.
///
/// # Errors
/// The [`Refusal`] that holds the build back.
pub fn check(cfg: &GuardConfig, server: Server) -> Result<(), Refusal> {
    if !cfg.enabled {
        return Ok(());
    }
    let holder = lock_holder(&cfg.lock_path, pid_alive);
    let available = if cfg.min_free_mb(server) > 0 {
        read_mem_available()
    } else {
        None
    };
    evaluate(cfg, server, available, holder)
}

/// One build's waiting clock: since the first refusal after its last
/// admission.
#[derive(Debug, Clone, Copy)]
struct Waiting {
    since: Instant,
    alerted: bool,
}

/// What [`note_refusal`] decided: how long the build has waited and whether
/// this refusal is the one that alerts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Noted {
    waited: Duration,
    alert: bool,
}

type Clocks = HashMap<(Server, LoadKind), Waiting>;

static WAITING: LazyLock<Mutex<Clocks>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// Start or advance the waiting clock of `key`; the first refusal past
/// `max_wait` alerts, once per wait.
fn note_refusal(
    clocks: &mut Clocks,
    key: (Server, LoadKind),
    now: Instant,
    max_wait: Option<Duration>,
) -> Noted {
    let w = clocks.entry(key).or_insert(Waiting {
        since: now,
        alerted: false,
    });
    let waited = now.saturating_duration_since(w.since);
    let alert = !w.alerted && max_wait.is_some_and(|max| waited > max);
    if alert {
        w.alerted = true;
    }
    Noted { waited, alert }
}

/// Stop the waiting clock of `key`; how long it ran, if it was running.
fn note_admit(clocks: &mut Clocks, key: (Server, LoadKind), now: Instant) -> Option<Duration> {
    clocks
        .remove(&key)
        .map(|w| now.saturating_duration_since(w.since))
}

/// Admit or refuse a build of `server`, with the bookkeeping: one log line
/// per refusal, one Discord alert (spawned, never awaited) when the build has
/// waited past `RELOAD_MAX_WAIT_SECS`, and one log line when a build that
/// waited is admitted. Call it from inside the Tokio runtime.
///
/// # Errors
/// The [`Refusal`] that holds the build back.
pub fn admit(client: &reqwest::Client, server: Server, kind: LoadKind) -> Result<(), Refusal> {
    let cfg = config();
    let now = Instant::now();
    let key = (server, kind);
    let Err(why) = check(&cfg, server) else {
        if cfg.enabled
            && let Some(waited) = WAITING
                .lock()
                .ok()
                .and_then(|mut c| note_admit(&mut c, key, now))
        {
            tracing::info!(
                server = server.as_str(),
                kind = kind.as_str(),
                waited_secs = waited.as_secs(),
                "the reload guard admits a game data build that waited"
            );
        }
        return Ok(());
    };
    let noted = WAITING
        .lock()
        .ok()
        .map(|mut c| note_refusal(&mut c, key, now, cfg.max_wait))
        .unwrap_or(Noted {
            waited: Duration::ZERO,
            alert: false,
        });
    match kind {
        LoadKind::Reload => tracing::warn!(
            server = server.as_str(),
            kind = kind.as_str(),
            waited_secs = noted.waited.as_secs(),
            retry_secs = cfg.retry.as_secs(),
            reason = %why,
            "game data reload held back by the reload guard; retrying"
        ),
        LoadKind::LazyLoad => tracing::warn!(
            server = server.as_str(),
            kind = kind.as_str(),
            waited_secs = noted.waited.as_secs(),
            reason = %why,
            "lazy load held back by the reload guard; answering 503"
        ),
    }
    if noted.alert {
        let client = client.clone();
        let reason = why.to_string();
        tokio::spawn(async move {
            crate::core::alerts::report_reload_waiting(
                &client,
                server.as_str(),
                kind.as_str(),
                noted.waited,
                &reason,
            )
            .await;
        });
    }
    Err(why)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> GuardConfig {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        GuardConfig::parse(|k| map.get(k).cloned())
    }

    /// A fixture shaped like a Linux 6.x `/proc/meminfo`.
    const MEMINFO: &str = "MemTotal:       11138432 kB\n\
                           MemFree:          412312 kB\n\
                           MemAvailable:    1536000 kB\n\
                           Buffers:           88120 kB\n\
                           Cached:          1702244 kB\n";

    #[test]
    fn meminfo_parses_mem_available_in_mib() {
        assert_eq!(mem_available_mb(MEMINFO), Some(1500));
        assert_eq!(mem_available_mb("MemTotal: 1 kB\nMemFree: 2 kB\n"), None);
        assert_eq!(mem_available_mb("MemAvailable: lots kB\n"), None);
        assert_eq!(mem_available_mb("MemAvailable: 2048 MB\n"), None);
        assert_eq!(mem_available_mb(""), None);
    }

    #[test]
    fn lock_parses_pid_then_label() {
        assert_eq!(
            parse_lock("4242 stages window-gen\n"),
            Some((4242, "stages window-gen".to_owned()))
        );
        assert_eq!(parse_lock("17"), Some((17, String::new())));
        assert_eq!(parse_lock(""), None);
        assert_eq!(parse_lock("summaries 4242"), None);
    }

    /// A lock naming a live pid holds the model; a dead pid, a missing file or
    /// garbage leaves it free.
    #[test]
    fn lock_holder_needs_a_live_pid() {
        let dir = std::env::temp_dir().join(format!("myrtle-reload-guard-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let lock = dir.join("llm.lock");

        std::fs::write(&lock, format!("{} canary\n", std::process::id())).expect("write");
        assert_eq!(
            lock_holder(&lock, pid_alive),
            Some((std::process::id(), "canary".to_owned())),
            "this test's own pid is alive"
        );

        let mut child = std::process::Command::new("true").spawn().expect("spawn");
        let dead = child.id();
        child.wait().expect("wait");
        std::fs::write(&lock, format!("{dead} summaries\n")).expect("write");
        assert!(!pid_alive(dead), "a reaped child is dead");
        assert_eq!(lock_holder(&lock, pid_alive), None, "a dead pid is free");

        std::fs::write(&lock, "not a lock").expect("write");
        assert_eq!(lock_holder(&lock, |_| true), None);
        std::fs::remove_file(&lock).expect("rm");
        assert_eq!(lock_holder(&lock, |_| true), None, "no file is free");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn defaults_are_the_measured_peaks() {
        let c = env(&[]);
        assert!(c.enabled);
        assert_eq!(c.min_free_mb(Server::EN), 900);
        assert_eq!(c.min_free_mb(Server::CN), 1800);
        assert_eq!(c.min_free_mb(Server::Bilibili), 1800);
        assert_eq!(c.min_free_mb(Server::JP), 1200);
        assert_eq!(c.min_free_mb(Server::KR), 1200);
        assert_eq!(c.lock_path, PathBuf::from(DEFAULT_LOCK_PATH));
        assert_eq!(c.retry, Duration::from_secs(300));
        assert_eq!(c.max_wait, Some(Duration::from_secs(21_600)));
    }

    #[test]
    fn thresholds_and_paths_are_overridable() {
        let c = env(&[
            ("RELOAD_MIN_FREE_MB", "1500, cn=2500, bogus=9, kr=x"),
            ("TREVOR_LLM_LOCK", "/tmp/x.lock"),
            ("RELOAD_RETRY_SECS", "0"),
            ("RELOAD_MAX_WAIT_SECS", "0"),
        ]);
        assert_eq!(c.min_free_mb(Server::EN), 1500);
        assert_eq!(c.min_free_mb(Server::KR), 1500, "a bad item is ignored");
        assert_eq!(c.min_free_mb(Server::CN), 2500);
        assert_eq!(c.min_free_mb(Server::Bilibili), 2500, "bili follows cn");
        assert_eq!(c.lock_path, PathBuf::from("/tmp/x.lock"));
        assert_eq!(c.retry, Duration::from_secs(300), "0 s would spin");
        assert_eq!(c.max_wait, None, "0 never alerts");
        assert_eq!(
            env(&[("TREVOR_LLM_LOCK", " ")]).lock_path,
            PathBuf::from(DEFAULT_LOCK_PATH)
        );
        assert_eq!(
            env(&[("RELOAD_MIN_FREE_MB", "0")]).min_free_mb(Server::CN),
            0
        );
    }

    #[test]
    fn decision_refuses_below_the_floor_and_under_the_lock() {
        let c = env(&[]);
        assert_eq!(evaluate(&c, Server::EN, Some(900), None), Ok(()));
        assert_eq!(
            evaluate(&c, Server::EN, Some(899), None),
            Err(Refusal::LowMemory {
                available_mb: 899,
                need_mb: 900
            })
        );
        assert_eq!(
            evaluate(&c, Server::CN, Some(1500), None),
            Err(Refusal::LowMemory {
                available_mb: 1500,
                need_mb: 1800
            }),
            "1500 MiB clears EN's floor but not CN's"
        );
        assert_eq!(
            evaluate(&c, Server::EN, None, None),
            Ok(()),
            "unknown memory (macOS) admits"
        );
        let held = Some((4242, "stages".to_owned()));
        assert_eq!(
            evaluate(&c, Server::EN, Some(8000), held.clone()),
            Err(Refusal::ModelBusy {
                pid: 4242,
                label: "stages".to_owned()
            }),
            "the lock refuses whatever the memory"
        );
        let mem_off = env(&[("RELOAD_MIN_FREE_MB", "0")]);
        assert_eq!(evaluate(&mem_off, Server::CN, Some(1), None), Ok(()));
        assert!(evaluate(&mem_off, Server::CN, Some(1), held.clone()).is_err());
    }

    /// `RELOAD_GUARD=0` admits a build that every other setting refuses, and
    /// reads neither the lock nor `/proc/meminfo`.
    #[test]
    fn the_switch_off_admits_everything() {
        let off = env(&[
            ("RELOAD_GUARD", " 0 "),
            ("TREVOR_LLM_LOCK", "/proc/self/status"),
        ]);
        assert!(!off.enabled);
        assert_eq!(
            evaluate(&off, Server::CN, Some(1), Some((1, "stages".to_owned()))),
            Ok(())
        );
        assert_eq!(check(&off, Server::CN), Ok(()));
        assert!(env(&[("RELOAD_GUARD", "1")]).enabled);
    }

    #[test]
    fn one_alert_per_wait_past_the_bound() {
        let mut clocks = Clocks::new();
        let key = (Server::EN, LoadKind::Reload);
        let t0 = Instant::now();
        let max = Some(Duration::from_secs(600));
        let at = |s| t0 + Duration::from_secs(s);
        assert_eq!(
            note_refusal(&mut clocks, key, t0, max),
            Noted {
                waited: Duration::ZERO,
                alert: false
            }
        );
        assert!(
            !note_refusal(&mut clocks, key, at(600), max).alert,
            "not past yet"
        );
        let past = note_refusal(&mut clocks, key, at(900), max);
        assert_eq!(past.waited, Duration::from_secs(900));
        assert!(past.alert);
        assert!(!note_refusal(&mut clocks, key, at(1200), max).alert, "once");
        assert!(
            !note_refusal(&mut clocks, (Server::CN, LoadKind::Reload), at(1200), max).alert,
            "each build has its own clock"
        );
        assert_eq!(
            note_admit(&mut clocks, key, at(1500)),
            Some(Duration::from_secs(1500))
        );
        assert_eq!(note_admit(&mut clocks, key, at(1500)), None);
        assert!(
            !note_refusal(&mut clocks, key, at(1600), max).alert,
            "a new wait"
        );
        assert!(
            !note_refusal(&mut clocks, key, at(99_999), None).alert,
            "0 never alerts"
        );
    }

    #[test]
    fn the_alert_has_the_embed_shape() {
        let e = crate::core::alerts::reload_waiting_embed(
            "cn",
            "reload",
            Duration::from_secs(6 * 3600 + 5 * 60),
            "the Trevor model lock is held by `stages` (pid 7)",
        );
        assert_eq!(e["title"], "Game-data reload waiting (cn)");
        let d = e["description"].as_str().expect("description");
        assert!(
            d.contains("held back for 6h05m: the Trevor model lock is held by 'stages' (pid 7)"),
            "{d}"
        );
        assert!(e["color"].is_u64());
        assert!(e["footer"]["text"].is_string());
    }

    #[test]
    fn refusals_read_plainly() {
        assert_eq!(
            Refusal::LowMemory {
                available_mb: 700,
                need_mb: 900
            }
            .to_string(),
            "only 700 MiB available, need 900 MiB (RELOAD_MIN_FREE_MB)"
        );
        assert_eq!(
            Refusal::ModelBusy {
                pid: 7,
                label: String::new()
            }
            .to_string(),
            "the Trevor model lock is held by a model job (pid 7)"
        );
    }
}
