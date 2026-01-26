//! Memory profiling middleware and utilities
//!
//! Tracks memory usage over time and per-request to help identify leaks.

use axum::{
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::Response,
};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    net::SocketAddr,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use sysinfo::System;

use crate::app::middleware::rate_limit::RateLimitStore;

const LOG_FILE: &str = "memory_profile.log";
const SNAPSHOT_INTERVAL_SECS: u64 = 30;
const HIGH_MEMORY_DELTA_KB: u64 = 1024; // Log requests that increase memory by 1MB+

/// Global memory stats
pub struct MemoryStats {
    /// Total requests since startup
    pub total_requests: AtomicU64,
    /// Requests that caused high memory delta
    pub high_memory_requests: AtomicU64,
    /// Log file handle
    log_file: Mutex<File>,
}

impl MemoryStats {
    pub fn new() -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(LOG_FILE)?;

        Ok(Self {
            total_requests: AtomicU64::new(0),
            high_memory_requests: AtomicU64::new(0),
            log_file: Mutex::new(file),
        })
    }

    pub fn log(&self, message: &str) {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
        let line = format!("[{timestamp}] {message}\n");

        if let Ok(mut file) = self.log_file.lock() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }

        // Also print to stderr for immediate visibility
        eprint!("{line}");
    }

    pub fn increment_requests(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_high_memory(&self) {
        self.high_memory_requests.fetch_add(1, Ordering::Relaxed);
    }
}

/// Get current process memory usage in KB
pub fn get_memory_usage_kb() -> u64 {
    let pid = sysinfo::get_current_pid().ok();
    let mut sys = System::new();

    if let Some(pid) = pid {
        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
        if let Some(process) = sys.process(pid) {
            return process.memory() / 1024; // Convert bytes to KB
        }
    }

    0
}

/// Get detailed memory info
pub fn get_memory_info() -> MemoryInfo {
    let pid = sysinfo::get_current_pid().ok();
    let mut sys = System::new_all();

    let (rss, virtual_mem) = if let Some(pid) = pid {
        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
        if let Some(process) = sys.process(pid) {
            (process.memory(), process.virtual_memory())
        } else {
            (0, 0)
        }
    } else {
        (0, 0)
    };

    MemoryInfo {
        rss_mb: rss as f64 / 1024.0 / 1024.0,
        virtual_mb: virtual_mem as f64 / 1024.0 / 1024.0,
        system_total_mb: sys.total_memory() as f64 / 1024.0 / 1024.0,
        system_used_mb: sys.used_memory() as f64 / 1024.0 / 1024.0,
    }
}

#[derive(Debug)]
pub struct MemoryInfo {
    pub rss_mb: f64,
    pub virtual_mb: f64,
    pub system_total_mb: f64,
    pub system_used_mb: f64,
}

/// Spawn background task to periodically log memory stats
pub fn spawn_memory_monitor(stats: Arc<MemoryStats>, rate_store: RateLimitStore) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(SNAPSHOT_INTERVAL_SECS));
        let start_time = std::time::Instant::now();

        // Log startup
        let info = get_memory_info();
        stats.log(&format!(
            "STARTUP | RSS: {:.2}MB | Virtual: {:.2}MB",
            info.rss_mb, info.virtual_mb
        ));

        loop {
            interval.tick().await;

            let info = get_memory_info();
            let total_reqs = stats.total_requests.load(Ordering::Relaxed);
            let high_mem_reqs = stats.high_memory_requests.load(Ordering::Relaxed);
            let uptime_secs = start_time.elapsed().as_secs();
            let rate_limit_entries = rate_store.entry_count();

            stats.log(&format!(
                "SNAPSHOT | RSS: {:.2}MB | Virtual: {:.2}MB | Uptime: {}s | Requests: {} | HighMemReqs: {} | RateLimitEntries: {}",
                info.rss_mb,
                info.virtual_mb,
                uptime_secs,
                total_reqs,
                high_mem_reqs,
                rate_limit_entries
            ));

            // Warn if memory is getting high
            if info.rss_mb > 1024.0 {
                stats.log(&format!(
                    "WARNING | Memory exceeds 1GB: {:.2}MB",
                    info.rss_mb
                ));
            }
            if info.rss_mb > 4096.0 {
                stats.log(&format!(
                    "CRITICAL | Memory exceeds 4GB: {:.2}MB",
                    info.rss_mb
                ));
            }
        }
    });
}

/// Middleware to track per-request memory usage
pub async fn memory_tracking_middleware(
    State(stats): State<Arc<MemoryStats>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().to_string();

    // Get memory before request
    let mem_before = get_memory_usage_kb();

    stats.increment_requests();

    // Process request
    let response = next.run(request).await;

    // Get memory after request
    let mem_after = get_memory_usage_kb();

    // Calculate delta (handle underflow from GC/memory release)
    let delta = mem_after.saturating_sub(mem_before);

    // Log if memory increased significantly
    if delta > HIGH_MEMORY_DELTA_KB {
        stats.increment_high_memory();
        stats.log(&format!(
            "HIGH_MEM_REQUEST | {} {} | IP: {} | Delta: +{}KB | Before: {}KB | After: {}KB",
            method,
            path,
            addr.ip(),
            delta,
            mem_before,
            mem_after
        ));
    }

    // Sample log every 1000th request for baseline
    let total = stats.total_requests.load(Ordering::Relaxed);
    if total % 1000 == 0 {
        stats.log(&format!(
            "SAMPLE | {} {} | Mem: {}KB | TotalRequests: {}",
            method, path, mem_after, total
        ));
    }

    response
}

/// Log a custom memory event
pub fn log_memory_event(stats: &MemoryStats, event: &str) {
    let info = get_memory_info();
    stats.log(&format!(
        "EVENT | {} | RSS: {:.2}MB | Virtual: {:.2}MB",
        event, info.rss_mb, info.virtual_mb
    ));
}

/// Initialize memory profiling - call this at startup
pub fn init_memory_profiling() -> std::io::Result<Arc<MemoryStats>> {
    // Rotate log file if it's too large (>50MB)
    let log_path = Path::new(LOG_FILE);
    if log_path.exists()
        && let Ok(metadata) = std::fs::metadata(log_path)
        && metadata.len() > 50 * 1024 * 1024
    {
        let backup_name = format!(
            "memory_profile_{}.log",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        let _ = std::fs::rename(LOG_FILE, &backup_name);
    }

    let stats = Arc::new(MemoryStats::new()?);

    // Log initial state
    stats.log("=== Memory Profiling Started ===");
    let info = get_memory_info();
    stats.log(&format!(
        "INIT | RSS: {:.2}MB | Virtual: {:.2}MB | System Total: {:.2}MB | System Used: {:.2}MB",
        info.rss_mb, info.virtual_mb, info.system_total_mb, info.system_used_mb
    ));

    Ok(stats)
}
