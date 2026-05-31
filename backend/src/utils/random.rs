pub fn random_digits(count: usize) -> String {
    if count == 0 {
        return String::new();
    }
    let mut buf = vec![0u8; count];
    fill_random(&mut buf);
    let mut out = String::with_capacity(count);
    for b in buf {
        out.push(char::from_digit(u32::from(b % 10), 10).unwrap());
    }
    out
}

pub fn fill_random(buf: &mut [u8]) {
    if buf.is_empty() {
        return;
    }
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    platform::fill(buf);

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    compile_error!("unsupported platform: only linux, macos, and windows are supported");
}

// ── Linux ────────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
mod platform {
    use std::sync::atomic::{AtomicI32, Ordering};

    // Sentinel values stored in GETRANDOM_STATE:
    //   UNCHECKED (-1) → haven't probed yet
    //   UNAVAILABLE (0) → getrandom syscall not present, fall back to /dev/urandom
    //   AVAILABLE  (1) → use getrandom
    const UNCHECKED: i32 = -1;
    const UNAVAILABLE: i32 = 0;
    const AVAILABLE: i32 = 1;

    static GETRANDOM_STATE: AtomicI32 = AtomicI32::new(UNCHECKED);
    static URANDOM_FD: AtomicI32 = AtomicI32::new(-1);

    pub fn fill(buf: &mut [u8]) {
        if getrandom_available() && try_getrandom(buf) {
            return;
        }
        urandom_fill(buf);
    }

    fn getrandom_available() -> bool {
        match GETRANDOM_STATE.load(Ordering::Acquire) {
            AVAILABLE => true,
            UNAVAILABLE => false,
            _ => {
                // SAFETY: a null buffer with length 0 is a valid getrandom probe -
                // the kernel writes nothing and returns 0 if the syscall exists or
                // -1/ENOSYS if it doesn't.
                let supported = unsafe { sys_getrandom(core::ptr::null_mut(), 0, 0) } != -1;
                let state = if supported { AVAILABLE } else { UNAVAILABLE };
                GETRANDOM_STATE.store(state, Ordering::Release);
                supported
            }
        }
    }

    fn try_getrandom(buf: &mut [u8]) -> bool {
        let mut filled = 0;
        while filled < buf.len() {
            // SAFETY: pointer and length describe the still-unfilled, writable
            // tail of `buf`; getrandom writes at most `len` bytes into it.
            let ret = unsafe { sys_getrandom(buf[filled..].as_mut_ptr(), buf.len() - filled, 0) };
            match ret {
                n if n > 0 => filled += n as usize,
                -1 => {
                    // SAFETY: `errno_ptr` returns the thread-local errno location,
                    // which is always valid to read.
                    let errno = unsafe { *errno_ptr() };
                    match errno {
                        4 => {} // EINTR - retry
                        _ => panic!("getrandom failed: errno {errno}"),
                    }
                }
                _ => panic!("getrandom returned unexpected value: {ret}"),
            }
        }
        true
    }

    fn urandom_fill(buf: &mut [u8]) {
        let fd = match URANDOM_FD.load(Ordering::Acquire) {
            -1 => open_urandom(),
            fd => fd,
        };

        let mut filled = 0;
        while filled < buf.len() {
            // SAFETY: `fd` is an open `/dev/urandom` descriptor; pointer and length
            // describe the still-unfilled, writable tail of `buf`.
            let ret = unsafe { libc_read(fd, buf[filled..].as_mut_ptr(), buf.len() - filled) };
            match ret {
                n if n > 0 => filled += n as usize,
                -1 => {
                    // SAFETY: `errno_ptr` returns the thread-local errno location,
                    // which is always valid to read.
                    let errno = unsafe { *errno_ptr() };
                    match errno {
                        4 => {} // EINTR - retry
                        _ => panic!("/dev/urandom read failed: errno {errno}"),
                    }
                }
                0 => panic!("/dev/urandom returned EOF - this should never happen"),
                _ => unreachable!(),
            }
        }
    }

    fn open_urandom() -> i32 {
        let path = b"/dev/urandom\0";
        // SAFETY: `path` is a NUL-terminated C string; `open` with O_RDONLY (0)
        // returns a new fd or a negative error code.
        let new_fd = unsafe { libc_open(path.as_ptr(), 0) };
        assert!(new_fd >= 0, "failed to open /dev/urandom");
        match URANDOM_FD.compare_exchange(-1, new_fd, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => new_fd,
            Err(existing) => {
                // SAFETY: another thread won the race; `new_fd` is a valid fd we
                // own and no longer need, so closing it exactly once is sound.
                unsafe { libc_close(new_fd) };
                existing
            }
        }
    }

    // ── Syscall / libc bindings ───────────────────────────────────────────

    unsafe fn sys_getrandom(buf: *mut u8, len: usize, flags: u32) -> i64 {
        #[cfg(target_arch = "x86_64")]
        const NR: i64 = 318;
        #[cfg(target_arch = "aarch64")]
        const NR: i64 = 278;
        #[cfg(target_arch = "arm")]
        const NR: i64 = 384;
        #[cfg(target_arch = "riscv64")]
        const NR: i64 = 278;
        #[cfg(target_arch = "x86")]
        const NR: i64 = 355;
        #[cfg(target_arch = "mips")]
        const NR: i64 = 4353;
        #[cfg(target_arch = "mips64")]
        const NR: i64 = 5313;

        unsafe extern "C" {
            fn syscall(num: i64, ...) -> i64;
        }
        // SAFETY: forwards the caller's (buf, len, flags) to the getrandom syscall;
        // the caller guarantees buf/len describe a valid writable region (or null/0).
        unsafe { syscall(NR, buf, len, flags) }
    }

    unsafe fn errno_ptr() -> *mut i32 {
        unsafe extern "C" {
            fn __errno_location() -> *mut i32;
        }
        // SAFETY: glibc's `__errno_location` always returns a valid, thread-local
        // pointer to errno.
        unsafe { __errno_location() }
    }

    unsafe fn libc_open(path: *const u8, flags: i32) -> i32 {
        unsafe extern "C" {
            fn open(path: *const u8, flags: i32, ...) -> i32;
        }
        // SAFETY: the caller passes a NUL-terminated `path`; `open` reads it and
        // returns an fd or negative error code.
        unsafe { open(path, flags) }
    }

    unsafe fn libc_read(fd: i32, buf: *mut u8, count: usize) -> isize {
        unsafe extern "C" {
            fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
        }
        // SAFETY: the caller passes a valid `fd` and a writable (buf, count) region.
        unsafe { read(fd, buf, count) }
    }

    unsafe fn libc_close(fd: i32) {
        unsafe extern "C" {
            fn close(fd: i32) -> i32;
        }
        // SAFETY: the caller passes an fd they own; closing it once is sound.
        unsafe {
            close(fd);
        }
    }
}

// ── macOS ────────────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
mod platform {
    pub fn fill(buf: &mut [u8]) {
        unsafe extern "C" {
            fn getentropy(buf: *mut u8, size: usize) -> i32;
        }
        // getentropy(2) is capped at 256 bytes per call
        for chunk in buf.chunks_mut(256) {
            // SAFETY: (ptr, len) describe a writable chunk of `buf` of at most 256
            // bytes - within getentropy's per-call limit.
            let ret = unsafe { getentropy(chunk.as_mut_ptr(), chunk.len()) };
            assert_eq!(ret, 0, "getentropy failed");
        }
    }
}

// ── Windows ──────────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod platform {
    pub fn fill(buf: &mut [u8]) {
        #[link(name = "bcrypt")]
        unsafe extern "system" {
            fn BCryptGenRandom(algo: *mut u8, buf: *mut u8, size: u32, flags: u32) -> u32;
        }
        const USE_SYSTEM_RNG: u32 = 0x00000002;
        // BCryptGenRandom with USE_SYSTEM_RNG ignores the algorithm handle
        // SAFETY: a null algorithm handle is valid with USE_SYSTEM_RNG; (buf, len)
        // describe a writable region and `len` fits in u32 for realistic sizes.
        let ret = unsafe {
            BCryptGenRandom(
                core::ptr::null_mut(),
                buf.as_mut_ptr(),
                buf.len() as u32,
                USE_SYSTEM_RNG,
            )
        };
        assert_eq!(ret, 0, "BCryptGenRandom failed: NTSTATUS 0x{ret:08x}");
    }
}
