use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LazyLock, Mutex, PoisonError};
use std::time::{Duration, Instant};

use indicatif::{ProgressBar, ProgressStyle};

#[expect(
    clippy::literal_string_with_formatting_args,
    reason = "an indicatif template"
)]
static BAR: LazyLock<ProgressBar> = LazyLock::new(|| {
    let style = ProgressStyle::with_template("{spinner:.cyan} {msg} {elapsed:.dim}")
        .unwrap_or_else(|_| ProgressStyle::default_spinner());
    ProgressBar::new_spinner().with_style(style)
});
static CURRENT: Mutex<Option<(String, Instant)>> = Mutex::new(None);
static STARTED: LazyLock<Instant> = LazyLock::new(Instant::now);
static TOTAL: AtomicUsize = AtomicUsize::new(0);
static COUNT: AtomicUsize = AtomicUsize::new(0);

pub struct LogWriter;

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        BAR.suspend(|| io::stdout().write_all(buf))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()
    }
}

pub fn begin(steps: usize) {
    LazyLock::force(&STARTED);
    TOTAL.store(steps, Ordering::Relaxed);
}

pub fn step(label: impl Into<String>) {
    finish();
    let label = label.into();
    let count = COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    let total = TOTAL.load(Ordering::Relaxed);
    BAR.reset_elapsed();
    BAR.set_message(format!("[{count}/{total}] {label}"));
    BAR.enable_steady_tick(Duration::from_millis(100));
    *current() = Some((label, Instant::now()));
}

pub fn done() -> Duration {
    finish();
    clear();
    STARTED.elapsed()
}

pub fn clear() {
    current().take();
    BAR.finish_and_clear();
}

fn finish() {
    let finished = current().take();
    if let Some((step, at)) = finished {
        let took = format!("{:.1}s", at.elapsed().as_secs_f64());
        tracing::info!(step, took, "boot step done");
    }
}

fn current() -> std::sync::MutexGuard<'static, Option<(String, Instant)>> {
    CURRENT.lock().unwrap_or_else(PoisonError::into_inner)
}
