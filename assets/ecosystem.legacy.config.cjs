// LEGACY PM2 config: one watcher process per region. Superseded by
// ecosystem.config.cjs (one process, one queue, every region). Kept as the kill
// switch: `pm2 delete myrtle-ws && pm2 start ecosystem.legacy.config.cjs`.
// Each process here still takes the shared lockfile (<savedir>/.watcher.lock), so
// the four of them no longer overlap even on a manual force_update; WS_LOCK=0 in an
// app's env restores the old unlocked behaviour exactly.
//
// One watcher process per region (the backend connects to each over its own WS
// port; see backend ASSET_WS_URLS, default en=9160 / cn=9161; jp=9162 / kr=9163 below). Each polls the
// HG CDN on an interval and re-downloads + re-extracts when a new resVersion or
// a rebuilt unpacker is detected.
//
// `run.mjs ws` runs the WebSocket server non-interactively; the WS_* env vars
// fill in what the interactive prompts otherwise would. `cwd` must be the assets
// dir so the bundled `binaries/`, `./ArkAssets/<region>` and `./output/<region>`
// paths resolve (run.mjs joins the region onto the savedir/output itself).
//
// WS_THREADS and WS_START_DELAY_MIN are sized for the VPS, which is 3 cores and
// 10 GiB against one disk holding a ~113 GB asset tree. Both watchers extracting
// at -j 2 puts four extraction threads on three cores and, more to the point,
// four write streams on one queue: that box has logged WRITE DMA timeouts with
// the SATA link frozen, soft lockups in the writeback kworkers, and RCU stalls.
// One thread each, half an interval apart on the clock, keeps at most one extract
// running at a time under the scheduler. A manual force_update bypasses the phase,
// and before the shared lockfile two triggered by hand at once could overlap.
// Raise them on a box with more cores and faster storage.
module.exports = {
    apps: [
        {
            name: "myrtle-ws-en",
            script: "run.mjs",
            interpreter: "node",
            cwd: "/var/www/myrtle.moe/assets",
            args: "ws",
            max_memory_restart: "2G",
            env: {
                WS_SERVER: "en",
                WS_PORT: "9160",
                WS_PROFILE: "full",
                WS_THREADS: "1",
                // EN holds the interval phase; CN is the one that moves.
                WS_START_DELAY_MIN: "0",
            },
        },
        {
            name: "myrtle-ws-cn",
            script: "run.mjs",
            interpreter: "node",
            cwd: "/var/www/myrtle.moe/assets",
            args: "ws",
            max_memory_restart: "2G",
            env: {
                WS_SERVER: "cn",
                WS_PORT: "9161",
                // `release` adds the event / banner / skin-brand art the
                // Release Planner serves for CN-only content; without it the
                // CN preview has names and dates but no pictures.
                WS_PROFILE: "operators,release",
                WS_THREADS: "1",
                // A PHASE offset, not a one-off delay: checks are anchored to
                // the wall clock, so EN lands on :00 and :30 and CN on :15 and
                // :45 no matter when either process last restarted. Half the 30
                // minute interval is the furthest apart two regions can be.
                // WS_ALIGN=0 falls back to offsetting from process start.
                WS_START_DELAY_MIN: "15",
            },
        },
        // JP and KR exist for their TEXT: the game-data picker and the `ja`/`ko`
        // locales read their excel and story tables, never their art, so the
        // `gamedata` profile is all they pull (74.6 to 77.6 MB of bundles, an
        // extract of 16 to 40 s locally on 2026-10-06, against EN's full tree).
        // Phased at :07/:37 and :22/:52, between EN and CN, because a gamedata
        // extract is short enough to finish before the next region wakes.
        {
            name: "myrtle-ws-jp",
            script: "run.mjs",
            interpreter: "node",
            cwd: "/var/www/myrtle.moe/assets",
            args: "ws",
            max_memory_restart: "2G",
            env: {
                WS_SERVER: "jp",
                WS_PORT: "9162",
                WS_PROFILE: "gamedata",
                WS_THREADS: "1",
                WS_START_DELAY_MIN: "7",
            },
        },
        {
            name: "myrtle-ws-kr",
            script: "run.mjs",
            interpreter: "node",
            cwd: "/var/www/myrtle.moe/assets",
            args: "ws",
            max_memory_restart: "2G",
            env: {
                WS_SERVER: "kr",
                WS_PORT: "9163",
                WS_PROFILE: "gamedata",
                WS_THREADS: "1",
                WS_START_DELAY_MIN: "22",
            },
        },
    ],
};
