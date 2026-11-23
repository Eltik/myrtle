use std::fmt::Write as _;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::routing::get;
use dashmap::DashMap;
use myrtle_app::AppContext;
use tokio::net::TcpListener;

pub static METRICS: LazyLock<Metrics> = LazyLock::new(Metrics::default);

const BUCKET_BOUNDS: [f64; 9] = [0.005, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 5.0, 30.0];
const BUCKET_COUNT: usize = BUCKET_BOUNDS.len() + 1;
const MAX_SERIES: usize = 512;
const STATUS_CLASSES: [&str; 5] = ["1xx", "2xx", "3xx", "4xx", "5xx"];

#[derive(Default)]
struct RouteStats {
    by_class: [AtomicU64; 5],
    buckets: [AtomicU64; BUCKET_COUNT],
    micros: AtomicU64,
}

#[derive(Default)]
pub struct Metrics {
    http: DashMap<String, RouteStats>,
    rate_limited: DashMap<&'static str, AtomicU64>,
}

impl Metrics {
    pub fn record_http(&self, route: &str, status: u16, elapsed: Duration) {
        if !self.http.contains_key(route) {
            if self.http.len() >= MAX_SERIES {
                return;
            }
            self.http.entry(route.to_owned()).or_default();
        }
        let Some(stats) = self.http.get(route) else {
            return;
        };
        let class = usize::from(status / 100).clamp(1, 5) - 1;
        stats.by_class[class].fetch_add(1, Ordering::Relaxed);
        let secs = elapsed.as_secs_f64();
        let bucket = BUCKET_BOUNDS
            .iter()
            .position(|bound| secs <= *bound)
            .unwrap_or(BUCKET_BOUNDS.len());
        stats.buckets[bucket].fetch_add(1, Ordering::Relaxed);
        stats
            .micros
            .fetch_add(elapsed.as_micros() as u64, Ordering::Relaxed);
    }

    pub fn rate_limited(&self, bucket: &'static str) {
        self.rate_limited
            .entry(bucket)
            .or_default()
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "# TYPE myrtle_http_requests_total counter");
        for entry in &self.http {
            for (class, hits) in STATUS_CLASSES.iter().zip(&entry.by_class) {
                let hits = hits.load(Ordering::Relaxed);
                if hits > 0 {
                    let _ = writeln!(
                        out,
                        "myrtle_http_requests_total{{route=\"{}\",status=\"{class}\"}} {hits}",
                        entry.key()
                    );
                }
            }
        }
        let _ = writeln!(out, "# TYPE myrtle_http_request_duration_seconds histogram");
        for entry in &self.http {
            let route = entry.key();
            let mut cumulative = 0;
            for (i, hits) in entry.buckets.iter().enumerate() {
                cumulative += hits.load(Ordering::Relaxed);
                let le = BUCKET_BOUNDS
                    .get(i)
                    .map_or_else(|| "+Inf".to_owned(), f64::to_string);
                let _ = writeln!(
                    out,
                    "myrtle_http_request_duration_seconds_bucket{{route=\"{route}\",le=\"{le}\"}} {cumulative}"
                );
            }
            let sum = entry.micros.load(Ordering::Relaxed) as f64 / 1e6;
            let _ = writeln!(
                out,
                "myrtle_http_request_duration_seconds_sum{{route=\"{route}\"}} {sum}"
            );
            let _ = writeln!(
                out,
                "myrtle_http_request_duration_seconds_count{{route=\"{route}\"}} {cumulative}"
            );
        }
        let _ = writeln!(out, "# TYPE myrtle_rate_limited_total counter");
        for entry in &self.rate_limited {
            let _ = writeln!(
                out,
                "myrtle_rate_limited_total{{bucket=\"{}\"}} {}",
                entry.key(),
                entry.load(Ordering::Relaxed)
            );
        }
        out
    }
}

pub async fn serve(ctx: AppContext) {
    let bind = ctx.config.server.metrics_bind.clone();
    if bind.is_empty() {
        return;
    }
    let listener = match TcpListener::bind(&bind).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::warn!(bind, error = %e, "metrics listener did not start");
            return;
        }
    };
    let app = Router::new()
        .route(
            "/metrics",
            get(|State(ctx): State<AppContext>| async move {
                let mut out = METRICS.render();
                let _ = writeln!(out, "# TYPE myrtle_cpu_task_permits gauge");
                let _ = writeln!(out, "myrtle_cpu_task_permits {}", ctx.cpu.permits());
                let _ = writeln!(out, "# TYPE myrtle_cpu_task_waiting gauge");
                let _ = writeln!(out, "myrtle_cpu_task_waiting {}", ctx.cpu.waiting());
                out
            }),
        )
        .with_state(ctx);
    if let Err(e) = axum::serve(listener, app).await {
        tracing::warn!(error = %e, "metrics listener stopped");
    }
}
