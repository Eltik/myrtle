//! The allocator configuration the server binaries compile in, and a way to
//! read back what jemalloc is actually running with.
//!
//! The build is PREFIXED (`tikv-jemalloc-sys` without
//! `unprefixed_malloc_on_supported_platforms`), so jemalloc's symbols carry
//! `_rjem_`: the compiled-in options are read from the `_rjem_malloc_conf`
//! symbol, and the runtime override is the environment variable
//! `_RJEM_MALLOC_CONF`. A plain `MALLOC_CONF` is NOT read by this build (see
//! the `memory_bench` example, which prints the options under both).
//!
//! jemalloc applies the compiled-in string first and the environment second,
//! option by option, so `_RJEM_MALLOC_CONF` overrides any option it names.

/// The options compiled into `backend` (and the bench example), as the
/// `_rjem_malloc_conf` symbol expects: NUL-terminated.
///
/// `background_thread:true` purges on a jemalloc thread instead of on the
/// next allocation, so an idle process still gives memory back.
/// `dirty_decay_ms:1000` returns freed pages within about a second instead of
/// five. `muzzy_decay_ms:0` skips the muzzy stage: on Linux a muzzy page is
/// `MADV_FREE`d, which the kernel takes back only under memory pressure and
/// which keeps counting in RSS until then, and RSS is what pm2's
/// `max_memory_restart` reads. With 0 a decayed page is `MADV_DONTNEED`ed and
/// leaves RSS at once.
///
/// The previous string was `background_thread:true,dirty_decay_ms:5000,muzzy_decay_ms:5000`;
/// `_RJEM_MALLOC_CONF=dirty_decay_ms:5000,muzzy_decay_ms:5000` restores it
/// exactly (`background_thread` is unchanged).
pub const JEMALLOC_CONF: &[u8] = b"background_thread:true,dirty_decay_ms:1000,muzzy_decay_ms:0\0";

/// The options jemalloc is running with, as `key=value` pairs, or `None`
/// where this platform has no jemalloc.
#[must_use]
pub fn jemalloc_opts() -> Option<String> {
    #[cfg(not(target_env = "msvc"))]
    {
        let background: bool = read(b"opt.background_thread\0")?;
        let dirty: isize = read(b"opt.dirty_decay_ms\0")?;
        let muzzy: isize = read(b"opt.muzzy_decay_ms\0")?;
        let narenas: u32 = read(b"opt.narenas\0")?;
        Some(format!(
            "background_thread={background} dirty_decay_ms={dirty} muzzy_decay_ms={muzzy} narenas={narenas}"
        ))
    }
    #[cfg(target_env = "msvc")]
    {
        None
    }
}

#[cfg(not(target_env = "msvc"))]
fn read<T: Copy + Default>(name: &[u8]) -> Option<T> {
    let mut value = T::default();
    let mut len = std::mem::size_of::<T>();
    // SAFETY: `name` is NUL-terminated, `value` is a valid `T` of `len` bytes
    // for the option read (each name above is read as its documented C type),
    // and no new value is written.
    let rc = unsafe {
        tikv_jemalloc_sys::mallctl(
            name.as_ptr().cast(),
            std::ptr::from_mut(&mut value).cast(),
            &raw mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    (rc == 0 && len == std::mem::size_of::<T>()).then_some(value)
}

/// This process's resident set size in bytes, from `ps` (portable across the
/// macOS dev box and the Linux VPS). `None` if `ps` is unavailable.
#[must_use]
pub fn rss_bytes() -> Option<u64> {
    let out = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;
    let kib: u64 = String::from_utf8_lossy(&out.stdout).trim().parse().ok()?;
    Some(kib * 1024)
}
