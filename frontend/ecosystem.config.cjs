// PM2 config for the frontend SSR server.
//
// Runs the built server directly (not via `bun start`, which forks a
// `bash -c bun start` -> `bun start` -> `bun .output/server/index.mjs` chain)
module.exports = {
    apps: [
        {
            name: "myrtle-frontend",
            script: ".output/server/index.mjs",
            interpreter: "bun",
            cwd: "/var/www/myrtle.moe/frontend",
            max_memory_restart: "1500M",
            env: {
                NODE_ENV: "production",
                PORT: "3000",
            },
        },
    ],
};
