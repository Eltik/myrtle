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
            // `bun --smol` (interpreter_args) was measured and NOT adopted: on 588
            // requests over every operator page it moved neither peak RSS (580.2/613.4
            // MB default vs 596.4/578.7 smol) nor footprint after load (212/230 vs
            // 225/232 MB). BUN_JSC_forceRAMSize=256M moved footprint after load
            // 212 -> 211 and 230 -> 201 MB in two runs and cost +26% and +37% CPU per
            // request (16.39 -> 20.61, 13.79 -> 18.91 ms). Local macOS numbers, 2026-10-07.
            env: {
                NODE_ENV: "production",
                PORT: "3000",
                // Anonymous HTML micro-cache for /operators and /operators/:id
                // (src/lib/ssr-cache.ts). "0" is the kill switch and restores the stock
                // server exactly. Measured 15.65..15.91 -> 3.62..3.83 ms CPU per request
                // at an 88% hit rate, for up to SSR_CACHE_MAX_MB of resident HTML.
                SSR_CACHE: "1",
                SSR_CACHE_TTL_MS: "45000",
                SSR_CACHE_MAX_MB: "48",
            },
        },
    ],
};
