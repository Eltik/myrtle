// PM2 config for the backend server.
//
// Runs the compiled binary directly, not `cargo run --release`. `cargo run` spawns the
// binary as a child and waits, so pm2 ends up monitoring `cargo` (a wrapper) instead of
// the server.
//
// The binary loads backend/.env via dotenv, so `cwd` must be the backend dir.
module.exports = {
    apps: [
        {
            name: "myrtle-backend",
            script: "target/release/backend",
            cwd: "/var/www/myrtle.moe/backend",
            max_memory_restart: "3G",
            env: {
                RUST_LOG: "backend=info,tower_http=info",
                // The default permit formula is (cores / 2).max(1), which on this
                // 3 vCPU box yields ONE, so a second reader of a CPU-bound route
                // was refused while the first was still computing. Two gives real
                // concurrency and still leaves a core for the async workers,
                // postgres and everything else on the box. A TRADE, not a derived
                // number: raise it only if /admin/stats shows rejections with the
                // box not actually busy.
                CPU_TASK_PERMITS: "2",
                // How long a request waits for a permit before it is refused.
                // Size it from /admin/stats: sum_micros / started for a kind is
                // its mean hold time, and the wait wants to be a small multiple of
                // that so a burst drains instead of shedding. Must stay well under
                // the 30s handler timeout. 0 restores instant refusal.
                CPU_TASK_WAIT_MS: "2500",
                // jemalloc options, overriding the ones compiled into the binary
                // (src/app/memory.rs) option by option. The build is PREFIXED,
                // so the variable is `_RJEM_MALLOC_CONF`: a plain MALLOC_CONF is
                // ignored (measured: MALLOC_CONF=dirty_decay_ms:4321 left the
                // running value at the compiled-in 1000). These values equal the
                // compiled-in ones, so this line is inert until edited. To restore
                // the previous build's decay: dirty_decay_ms:5000,muzzy_decay_ms:5000.
                _RJEM_MALLOC_CONF: "background_thread:true,dirty_decay_ms:1000,muzzy_decay_ms:0",
            },
        },
    ],
};
