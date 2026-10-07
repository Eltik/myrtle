//! Resident memory of game-data loads, unloads and reloads, measured without
//! starting the server.
//!
//! ```text
//! cargo run --release --example memory_bench [-- <assets/output/en>]
//! _RJEM_MALLOC_CONF=dirty_decay_ms:5000,muzzy_decay_ms:5000 cargo run --release --example memory_bench
//! ```
//!
//! Installs jemalloc with the same compiled-in options as the `backend`
//! binary, prints the options in effect, then walks: load -> full reload held
//! next to the live copy (the old sidecar-job path) -> sidecar patch held next
//! to the live copy (the new path) -> unload (a lazy server going idle) ->
//! load again. RSS is read with `ps` after each step and again after a settle
//! wait, so the decay settings show.

use std::alloc::{GlobalAlloc, Layout};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use backend::app::memory::{jemalloc_opts, rss_bytes};
use backend::core::gamedata::{init_game_data, load_sidecar_parts};

/// jemalloc behind a counter of LIVE heap bytes and their high-water mark.
/// RSS on macOS barely moves on free (no background purge thread there, and
/// freed pages linger), so the live-byte count is the exact measure of how
/// many copies are held; RSS is reported beside it.
struct Counting;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

// SAFETY: forwards every call to jemalloc unchanged; only counts sizes.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { tikv_jemallocator::Jemalloc.alloc(layout) };
        if !p.is_null() {
            let now = LIVE.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            PEAK.fetch_max(now, Ordering::Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { tikv_jemallocator::Jemalloc.dealloc(ptr, layout) };
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let p = unsafe { tikv_jemallocator::Jemalloc.realloc(ptr, layout, new_size) };
        if !p.is_null() {
            LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
            let now = LIVE.fetch_add(new_size, Ordering::Relaxed) + new_size;
            PEAK.fetch_max(now, Ordering::Relaxed);
        }
        p
    }
}

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Counting = Counting;

#[cfg(not(target_env = "msvc"))]
#[allow(non_upper_case_globals)]
#[unsafe(export_name = "_rjem_malloc_conf")]
pub static MALLOC_CONF: &[u8] = backend::app::memory::JEMALLOC_CONF;

#[allow(clippy::cast_precision_loss)] // MiB to one decimal
fn mib(b: Option<u64>) -> String {
    b.map_or_else(|| "?".into(), |b| format!("{:.1}", b as f64 / 1_048_576.0))
}

fn report(step: &str, settle: Duration) {
    let live = LIVE.load(Ordering::Relaxed) as u64;
    let peak = PEAK.swap(LIVE.load(Ordering::Relaxed), Ordering::Relaxed) as u64;
    let now = rss_bytes();
    std::thread::sleep(settle);
    let later = rss_bytes();
    println!(
        "{step:<40} live_mib={:>7} peak_since_last={:>7} rss_mib={:>7} after_{}s={:>7}",
        mib(Some(live)),
        mib(Some(peak)),
        mib(now),
        settle.as_secs(),
        mib(later)
    );
}

fn main() {
    let base: PathBuf = std::env::args()
        .nth(1)
        .map_or_else(|| PathBuf::from("../assets/output/en"), PathBuf::from);
    let excel = base.join("gamedata/excel");
    let settle = Duration::from_secs(
        std::env::var("BENCH_SETTLE_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3),
    );
    println!(
        "jemalloc: {}",
        jemalloc_opts().unwrap_or_else(|| "unavailable".into())
    );
    println!(
        "env _RJEM_MALLOC_CONF={:?} MALLOC_CONF={:?}",
        std::env::var("_RJEM_MALLOC_CONF").ok(),
        std::env::var("MALLOC_CONF").ok()
    );
    let load = |label: &str| {
        let t = Instant::now();
        let (gd, ai) = init_game_data(&excel, Path::new(&base)).expect("load EN game data");
        println!(
            "  {label}: {} operators in {:.2} s",
            gd.operators.len(),
            t.elapsed().as_secs_f64()
        );
        (Arc::new(gd), Arc::new(ai))
    };

    report("0 baseline", settle);
    let live = load("load");
    report("1 one copy resident", settle);

    // Old sidecar-job path: a full rebuild held next to the live copy until
    // the swap, then the old copy dropped.
    let rebuilt = load("full reload");
    report("2 full reload, both copies held", Duration::ZERO);
    drop(live);
    report("3 full reload swapped, old dropped", settle);
    let live = rebuilt;

    // New path: only the sidecar-fed parts rebuilt, tables shared.
    let t = Instant::now();
    let parts = load_sidecar_parts(&excel, &base);
    let patched = Arc::new(live.0.with_sidecars(parts.gacha, parts.event_shops));
    println!(
        "  sidecar patch: {} banners, {} shops in {:.3} s, tables shared={}",
        patched.gacha.gacha_pool_client.len(),
        patched.event_shops.len(),
        t.elapsed().as_secs_f64(),
        patched.shares_tables(&live.0)
    );
    report("4 sidecar patch, both held", Duration::ZERO);
    let (superseded, assets) = live;
    let live = (patched, assets);
    drop(superseded);
    report("5 sidecar patch swapped", settle);

    // A lazy server going idle, then requested again.
    drop(live);
    report("6 unloaded", settle);
    let live = load("reload after unload");
    report("7 loaded again", settle);
    drop(live);
    report("8 unloaded again", settle);
}
