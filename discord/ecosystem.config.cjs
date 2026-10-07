// PM2 config for the Discord bot.
//
// Runs the compiled binary directly, not `cargo run`. Under `cargo run` pm2
// monitors cargo (a wrapper), every restart re-checks the build and recompiles
// whatever changed (1m31s logged on one start), and a pull that broke the build
// takes the bot down at restart time instead of at deploy time.
//
// `cwd` must be the discord dir: the binary reads `.env` (DISCORD_TOKEN) via
// dotenv, `config.json` (or $DISCORD_CONFIG_PATH), and opens
// `sqlite:database.sqlite`, all relative to the working directory.
//
// The binary comes from `cargo build --release` in discord/ or from the CI
// bundle (scripts/fetch-release.sh), both of which write
// target/release/discord.
module.exports = {
    apps: [
        {
            name: "myrtle-discord",
            script: "target/release/discord",
            cwd: "/var/www/myrtle.moe/discord",
            // Not a node script: run the file itself.
            interpreter: "none",
            // A guardrail, not a budget. The bot's RSS on the VPS was not measured for
            // this file: read it from `pm2 ls` and tighten this once it is known.
            max_memory_restart: "512M",
            // No `env` block on purpose. dotenv never overrides a variable that is
            // already set, so a RUST_LOG here would silently win over discord/.env.
        },
    ],
};
