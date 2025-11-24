use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Instant;

/// Progress tracker for batch operations
pub struct ProgressTracker {
    multi: Arc<MultiProgress>,
    main_bar: ProgressBar,
    start_time: Instant,
}

impl ProgressTracker {
    pub fn new(total: u64, message: &str) -> Self {
        let multi = Arc::new(MultiProgress::new());

        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap()
            .progress_chars("#>-");

        let main_bar = multi.add(ProgressBar::new(total));
        main_bar.set_style(style);
        main_bar.set_message(message.to_string());

        ProgressTracker {
            multi,
            main_bar,
            start_time: Instant::now(),
        }
    }

    /// Increment progress
    pub fn inc(&self, delta: u64) {
        self.main_bar.inc(delta);
    }

    /// Set current position
    pub fn set_position(&self, pos: u64) {
        self.main_bar.set_position(pos);
    }

    /// Update message
    pub fn set_message(&self, msg: &str) {
        self.main_bar.set_message(msg.to_string());
    }

    /// Finish with message
    pub fn finish(&self, msg: &str) {
        self.main_bar.finish_with_message(msg.to_string());
    }

    /// Get elapsed time in seconds
    pub fn elapsed_secs(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// Get the MultiProgress for adding sub-bars
    pub fn multi(&self) -> Arc<MultiProgress> {
        Arc::clone(&self.multi)
    }
}

/// Simple counter for tracking operations
pub struct Counter {
    count: std::sync::atomic::AtomicUsize,
}

impl Counter {
    pub fn new() -> Self {
        Counter {
            count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub fn inc(&self) {
        self.count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get(&self) -> usize {
        self.count.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn reset(&self) {
        self.count.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}
