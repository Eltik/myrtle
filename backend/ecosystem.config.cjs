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
            },
        },
    ],
};
