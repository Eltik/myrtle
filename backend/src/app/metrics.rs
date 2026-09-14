//! Process-wide metrics, rendered as Prometheus text at `GET /metrics`.
//!
//! | series | what it measures |
//! |---|---|
//! | `myrtle_db_pool_connections` | pool connections idle / in use / total |
//! | `myrtle_http_requests_total` | finished requests by matched route and status class |
//! | `myrtle_http_request_duration_seconds` | request latency histogram by route |
//! | `myrtle_cache_events_total` | cache hit / miss / error by key namespace |
//! | `myrtle_cpu_task_*` | CPU-bound work admitted, refused and in flight |
//! | `myrtle_upstream_*` | outbound requests and their latency, by host |
//! | `myrtle_rate_limited_total` | requests refused by the limiter, by bucket |
//! | `myrtle_regrade_*` | regrade pass duration and users remaining |
//!
//! Built on `dashmap` and atomics rather than the `metrics` crate: no recorder
//! to install, no exporter version to track, and every label under the control
//! of this module.

use std::fmt::Write as _;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};

use dashmap::DashMap;

pub static METRICS: LazyLock<Metrics> = LazyLock::new(Metrics::default);

/// Histogram upper edges, in seconds. The lower edges resolve ordinary JSON
/// responses; the upper ones resolve the compute-bound routes.
const BUCKET_BOUNDS: [f64; 9] = [0.005, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 5.0, 30.0];
const BUCKET_COUNT: usize = BUCKET_BOUNDS.len() + 1; // + "+Inf"

/// Ceiling on distinct label values held per map.
///
/// Route labels come from axum's matched path and are bounded by the router,
/// but callers may supply other label values. Past this ceiling a new label is
/// simply not recorded, so no caller can grow these maps without limit.
const MAX_SERIES: usize = 512;

#[derive(Default)]
pub struct Metrics {
    http: DashMap<String, RouteStats>,
    cache: DashMap<String, CacheStats>,
    upstream: DashMap<String, UpstreamStats>,
    cpu: DashMap<String, CpuStats>,
    /// Requests refused by the rate limiter, by bucket class.
    rate_limited: DashMap<String, AtomicU64>,
    regrade: RegradeStats,
}

#[derive(Default)]
struct RouteStats {
    /// Indexed by status class - 1xx..5xx at 0..4.
    by_class: [AtomicU64; 5],
    buckets: [AtomicU64; BUCKET_COUNT],
    sum_micros: AtomicU64,
    max_micros: AtomicU64,
}

#[derive(Default)]
struct CacheStats {
    hit: AtomicU64,
    miss: AtomicU64,
    error: AtomicU64,
}

#[derive(Default)]
struct UpstreamStats {
    ok: AtomicU64,
    failed: AtomicU64,
    sum_micros: AtomicU64,
}

#[derive(Default)]
struct CpuStats {
    started: AtomicU64,
    rejected: AtomicU64,
    in_flight: AtomicU64,
    sum_micros: AtomicU64,
}

#[derive(Default)]
struct RegradeStats {
    /// Seconds the last completed pass took. 0 when none has finished.
    last_duration_secs: AtomicU64,
    /// Users left in the pass currently running. 0 when idle.
    remaining: AtomicU64,
    passes: AtomicU64,
}

/// Insert-with-a-ceiling. Returns `None` once `MAX_SERIES` distinct labels are
/// held, so a caller with an unbounded label simply stops being recorded rather
/// than growing the map.
fn slot<'a, V: Default>(
    map: &'a DashMap<String, V>,
    key: &str,
) -> Option<dashmap::mapref::one::Ref<'a, String, V>> {
    if let Some(existing) = map.get(key) {
        return Some(existing);
    }
    if map.len() >= MAX_SERIES {
        return None;
    }
    map.entry(key.to_owned()).or_default();
    map.get(key)
}

impl Metrics {
    /// One finished HTTP request, labelled by matched route
    /// (`/api/operators/{id}`) rather than the concrete path, which would give
    /// one series per id.
    pub fn record_http(&self, route: &str, status: u16, micros: u64) {
        let Some(stats) = slot(&self.http, route) else {
            return;
        };
        let class = (status / 100).clamp(1, 5) as usize - 1;
        stats.by_class[class].fetch_add(1, Ordering::Relaxed);
        stats.sum_micros.fetch_add(micros, Ordering::Relaxed);
        stats.max_micros.fetch_max(micros, Ordering::Relaxed);

        let secs = micros as f64 / 1_000_000.0;
        let idx = BUCKET_BOUNDS
            .iter()
            .position(|&b| secs <= b)
            .unwrap_or(BUCKET_COUNT - 1);
        // Prometheus histograms are cumulative: a sample counts in its own
        // bucket and in every wider one.
        for bucket in &stats.buckets[idx..] {
            bucket.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_cache(&self, key_class: &str, outcome: CacheOutcome) {
        let Some(stats) = slot(&self.cache, key_class) else {
            return;
        };
        match outcome {
            CacheOutcome::Hit => &stats.hit,
            CacheOutcome::Miss => &stats.miss,
            CacheOutcome::Error => &stats.error,
        }
        .fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_upstream(&self, host: &str, ok: bool, micros: u64) {
        let Some(stats) = slot(&self.upstream, host) else {
            return;
        };
        if ok {
            stats.ok.fetch_add(1, Ordering::Relaxed);
        } else {
            stats.failed.fetch_add(1, Ordering::Relaxed);
        }
        stats.sum_micros.fetch_add(micros, Ordering::Relaxed);
    }

    pub fn cpu_started(&self, kind: &str) {
        if let Some(stats) = slot(&self.cpu, kind) {
            stats.started.fetch_add(1, Ordering::Relaxed);
            stats.in_flight.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn cpu_finished(&self, kind: &str, micros: u64) {
        if let Some(stats) = slot(&self.cpu, kind) {
            stats.in_flight.fetch_sub(1, Ordering::Relaxed);
            stats.sum_micros.fetch_add(micros, Ordering::Relaxed);
        }
    }

    /// A request turned away because no CPU permit was free.
    pub fn cpu_rejected(&self, kind: &str) {
        if let Some(stats) = slot(&self.cpu, kind) {
            stats.rejected.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn rate_limited(&self, class: &str) {
        if let Some(counter) = slot(&self.rate_limited, class) {
            counter.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn regrade_remaining(&self, remaining: u64) {
        self.regrade.remaining.store(remaining, Ordering::Relaxed);
    }

    pub fn regrade_pass_finished(&self, duration_secs: u64) {
        self.regrade
            .last_duration_secs
            .store(duration_secs, Ordering::Relaxed);
        self.regrade.remaining.store(0, Ordering::Relaxed);
        self.regrade.passes.fetch_add(1, Ordering::Relaxed);
    }

    /// Prometheus text exposition. Pool figures are read from sqlx at scrape
    /// time rather than tracked incrementally.
    pub fn render(&self, pool: Option<&sqlx::PgPool>) -> String {
        let mut out = String::with_capacity(4096);

        if let Some(pool) = pool {
            let size = u64::from(pool.size());
            let idle = pool.num_idle() as u64;
            out.push_str("# HELP myrtle_db_pool_connections Connections held by the sqlx pool.\n");
            out.push_str("# TYPE myrtle_db_pool_connections gauge\n");
            let _ = writeln!(out, "myrtle_db_pool_connections{{state=\"idle\"}} {idle}");
            let _ = writeln!(
                out,
                "myrtle_db_pool_connections{{state=\"in_use\"}} {}",
                size.saturating_sub(idle)
            );
            let _ = writeln!(out, "myrtle_db_pool_connections{{state=\"total\"}} {size}");
        }

        out.push_str("# HELP myrtle_http_requests_total Finished HTTP requests.\n");
        out.push_str("# TYPE myrtle_http_requests_total counter\n");
        for entry in &self.http {
            let route = escape(entry.key());
            for (i, count) in entry.by_class.iter().enumerate() {
                let n = count.load(Ordering::Relaxed);
                if n > 0 {
                    let _ = writeln!(
                        out,
                        "myrtle_http_requests_total{{route=\"{route}\",status=\"{}xx\"}} {n}",
                        i + 1
                    );
                }
            }
        }

        out.push_str("# HELP myrtle_http_request_duration_seconds Request latency by route.\n");
        out.push_str("# TYPE myrtle_http_request_duration_seconds histogram\n");
        for entry in &self.http {
            let route = escape(entry.key());
            for (i, bucket) in entry.buckets.iter().enumerate() {
                let le = BUCKET_BOUNDS
                    .get(i)
                    .map_or_else(|| "+Inf".to_owned(), ToString::to_string);
                let _ = writeln!(
                    out,
                    "myrtle_http_request_duration_seconds_bucket{{route=\"{route}\",le=\"{le}\"}} {}",
                    bucket.load(Ordering::Relaxed)
                );
            }
            let total: u64 = entry
                .by_class
                .iter()
                .map(|c| c.load(Ordering::Relaxed))
                .sum();
            let _ = writeln!(
                out,
                "myrtle_http_request_duration_seconds_sum{{route=\"{route}\"}} {:.6}",
                entry.sum_micros.load(Ordering::Relaxed) as f64 / 1_000_000.0
            );
            let _ = writeln!(
                out,
                "myrtle_http_request_duration_seconds_count{{route=\"{route}\"}} {total}"
            );
            let _ = writeln!(
                out,
                "myrtle_http_request_duration_seconds_max{{route=\"{route}\"}} {:.6}",
                entry.max_micros.load(Ordering::Relaxed) as f64 / 1_000_000.0
            );
        }

        out.push_str("# HELP myrtle_cache_events_total Cache lookups by key class.\n");
        out.push_str("# TYPE myrtle_cache_events_total counter\n");
        for entry in &self.cache {
            let class = escape(entry.key());
            for (name, counter) in [
                ("hit", &entry.hit),
                ("miss", &entry.miss),
                ("error", &entry.error),
            ] {
                let _ = writeln!(
                    out,
                    "myrtle_cache_events_total{{key_class=\"{class}\",result=\"{name}\"}} {}",
                    counter.load(Ordering::Relaxed)
                );
            }
        }

        out.push_str("# HELP myrtle_upstream_requests_total Outbound requests by host.\n");
        out.push_str("# TYPE myrtle_upstream_requests_total counter\n");
        for entry in &self.upstream {
            let host = escape(entry.key());
            let _ = writeln!(
                out,
                "myrtle_upstream_requests_total{{host=\"{host}\",outcome=\"ok\"}} {}",
                entry.ok.load(Ordering::Relaxed)
            );
            let _ = writeln!(
                out,
                "myrtle_upstream_requests_total{{host=\"{host}\",outcome=\"failed\"}} {}",
                entry.failed.load(Ordering::Relaxed)
            );
            let _ = writeln!(
                out,
                "myrtle_upstream_duration_seconds_sum{{host=\"{host}\"}} {:.6}",
                entry.sum_micros.load(Ordering::Relaxed) as f64 / 1_000_000.0
            );
        }

        out.push_str("# HELP myrtle_cpu_task_in_flight CPU-bound tasks on the blocking pool.\n");
        out.push_str("# TYPE myrtle_cpu_task_in_flight gauge\n");
        for entry in &self.cpu {
            let kind = escape(entry.key());
            let _ = writeln!(
                out,
                "myrtle_cpu_task_in_flight{{kind=\"{kind}\"}} {}",
                entry.in_flight.load(Ordering::Relaxed)
            );
        }
        out.push_str("# HELP myrtle_cpu_task_total CPU-bound tasks admitted or refused.\n");
        out.push_str("# TYPE myrtle_cpu_task_total counter\n");
        for entry in &self.cpu {
            let kind = escape(entry.key());
            let _ = writeln!(
                out,
                "myrtle_cpu_task_total{{kind=\"{kind}\",outcome=\"started\"}} {}",
                entry.started.load(Ordering::Relaxed)
            );
            let _ = writeln!(
                out,
                "myrtle_cpu_task_total{{kind=\"{kind}\",outcome=\"rejected\"}} {}",
                entry.rejected.load(Ordering::Relaxed)
            );
            let _ = writeln!(
                out,
                "myrtle_cpu_task_duration_seconds_sum{{kind=\"{kind}\"}} {:.6}",
                entry.sum_micros.load(Ordering::Relaxed) as f64 / 1_000_000.0
            );
        }

        out.push_str("# HELP myrtle_rate_limited_total Requests refused by the rate limiter.\n");
        out.push_str("# TYPE myrtle_rate_limited_total counter\n");
        for entry in &self.rate_limited {
            let _ = writeln!(
                out,
                "myrtle_rate_limited_total{{bucket=\"{}\"}} {}",
                escape(entry.key()),
                entry.value().load(Ordering::Relaxed)
            );
        }

        out.push_str("# HELP myrtle_regrade_users_remaining Users left in the running pass.\n");
        out.push_str("# TYPE myrtle_regrade_users_remaining gauge\n");
        let _ = writeln!(
            out,
            "myrtle_regrade_users_remaining {}",
            self.regrade.remaining.load(Ordering::Relaxed)
        );
        out.push_str("# HELP myrtle_regrade_pass_duration_seconds Duration of the last pass.\n");
        out.push_str("# TYPE myrtle_regrade_pass_duration_seconds gauge\n");
        let _ = writeln!(
            out,
            "myrtle_regrade_pass_duration_seconds {}",
            self.regrade.last_duration_secs.load(Ordering::Relaxed)
        );
        out.push_str("# HELP myrtle_regrade_passes_total Completed regrade passes.\n");
        out.push_str("# TYPE myrtle_regrade_passes_total counter\n");
        let _ = writeln!(
            out,
            "myrtle_regrade_passes_total {}",
            self.regrade.passes.load(Ordering::Relaxed)
        );

        out
    }
}

#[derive(Clone, Copy)]
pub enum CacheOutcome {
    Hit,
    Miss,
    Error,
}

/// Prometheus label values escape backslash, double-quote and newline.
fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::{BUCKET_BOUNDS, BUCKET_COUNT, CacheOutcome, MAX_SERIES, Metrics, escape};

    #[test]
    fn histogram_is_cumulative_and_counts_match() {
        let m = Metrics::default();
        m.record_http("/api/health", 200, 1_000); // 1 ms
        m.record_http("/api/health", 200, 300_000); // 300 ms
        m.record_http("/api/health", 500, 40_000_000); // 40 s
        let text = m.render(None);

        // 1 ms lands in le=0.005 and every wider bucket; +Inf holds all three.
        assert!(
            text.contains(
                r#"myrtle_http_request_duration_seconds_bucket{route="/api/health",le="0.005"} 1"#
            ),
            "{text}"
        );
        assert!(
            text.contains(
                r#"myrtle_http_request_duration_seconds_bucket{route="/api/health",le="0.5"} 2"#
            ),
            "{text}"
        );
        assert!(
            text.contains(
                r#"myrtle_http_request_duration_seconds_bucket{route="/api/health",le="+Inf"} 3"#
            ),
            "{text}"
        );
        assert!(
            text.contains(r#"myrtle_http_request_duration_seconds_count{route="/api/health"} 3"#),
            "{text}"
        );
        assert!(
            text.contains(r#"myrtle_http_requests_total{route="/api/health",status="2xx"} 2"#),
            "{text}"
        );
        assert!(
            text.contains(r#"myrtle_http_requests_total{route="/api/health",status="5xx"} 1"#),
            "{text}"
        );
    }

    #[test]
    fn bucket_array_matches_its_bounds() {
        assert_eq!(BUCKET_COUNT, BUCKET_BOUNDS.len() + 1);
    }

    #[test]
    fn status_outside_1xx_5xx_does_not_panic() {
        let m = Metrics::default();
        m.record_http("/api/x", 0, 10);
        m.record_http("/api/x", 999, 10);
        assert!(m.render(None).contains("/api/x"));
    }

    #[test]
    fn label_cardinality_is_capped() {
        let m = Metrics::default();
        for i in 0..(MAX_SERIES + 50) {
            m.record_http(&format!("/scan/{i}"), 404, 100);
        }
        assert_eq!(m.http.len(), MAX_SERIES);
    }

    #[test]
    fn cache_outcomes_are_counted_separately() {
        let m = Metrics::default();
        m.record_cache("roster", CacheOutcome::Hit);
        m.record_cache("roster", CacheOutcome::Hit);
        m.record_cache("roster", CacheOutcome::Miss);
        let text = m.render(None);
        assert!(
            text.contains(r#"key_class="roster",result="hit"} 2"#),
            "{text}"
        );
        assert!(
            text.contains(r#"key_class="roster",result="miss"} 1"#),
            "{text}"
        );
    }

    #[test]
    fn labels_are_escaped() {
        assert_eq!(escape(r#"a"b\c"#), r#"a\"b\\c"#);
    }
}
