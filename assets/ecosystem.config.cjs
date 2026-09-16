// PM2 config for the asset watcher(s).
//
// One watcher process per region (the backend connects to each over its own WS
// port — see backend ASSET_WS_URLS, default en=9160 / cn=9161). Each polls the
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
// running at a time under the scheduler. A manual force_update still bypasses the
// phase, so two triggered by hand at once can still overlap.
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
    ],
};
