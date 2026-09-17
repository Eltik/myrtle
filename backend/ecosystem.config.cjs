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
            },
        },
    ],
};
