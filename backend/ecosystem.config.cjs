// PM2 config for the backend: the HTTP API and the job worker.
//
// Both run the compiled binaries directly. Both read config/myrtle.toml relative
// to the working directory, and resolve its env: secrets from the process env,
// then the first .env found walking up from it (backend/.env, or the repo-root
// .env only when backend/.env is missing), so `cwd` must be the backend dir.
// Start the API first: it runs the database migrations at boot and the worker
// does not.
//
// Run exactly ONE worker. Each worker holds its own in-memory game
// service-account sessions and re-mints them on its own, so a second one signs
// the first out; schedules, the regrade and the stats rebuilds also assume a
// single consumer. The old backend's jobs must not run beside it either.
//
// max_memory_restart: the old single backend ran under 3G (steady 0.2 to 0.6 GB,
// boot peak 1.76 GiB with EN and CN loaded). Each of these two loads all game
// data, so each gets the same cap; neither has been measured on the VPS yet.
module.exports = {
    apps: [
        {
            name: "myrtle-api",
            script: "target/release/myrtle-api",
            args: "config/myrtle.toml",
            cwd: "/var/www/myrtle.moe/backend",
            max_memory_restart: "3G",
            exp_backoff_restart_delay: 200,
            // SHUTDOWN_GRACE (5s, crates/api/src/lib.rs) plus the 2s runtime shutdown
            kill_timeout: 10000,
            env: {
                RUST_LOG: "info",
            },
        },
        {
            name: "myrtle-worker",
            script: "target/release/myrtle-worker",
            args: "config/myrtle.toml",
            cwd: "/var/www/myrtle.moe/backend",
            instances: 1,
            exec_mode: "fork",
            max_memory_restart: "3G",
            exp_backoff_restart_delay: 200,
            // DRAIN (30s, crates/worker/src/runtime.rs) plus the 2s runtime
            // shutdown; a SIGKILL before that leaves jobs for the reaper to requeue
            kill_timeout: 40000,
            env: {
                RUST_LOG: "info",
            },
        },
    ],
};
