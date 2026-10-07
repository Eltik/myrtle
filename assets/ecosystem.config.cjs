// PM2 config for the asset watcher: ONE process serving every region.
//
// The backend contract is unchanged: each region keeps its own WebSocket port
// (backend ASSET_WS_URLS, en=9160 / cn=9161 / jp=9162 / kr=9163) and the same
// message shapes. What changed is that the four regions share one scheduler, so
// exactly one download or extract runs on the box at a time, a manual
// force_update included. The previous four-app layout is in
// ecosystem.legacy.config.cjs and still works (`run.mjs ws --server en`).
//
// The VPS is 3 cores and 10 GiB against one disk holding a ~113 GB asset tree. It
// has logged WRITE DMA timeouts with the SATA link frozen, soft lockups in the
// writeback kworkers, and RCU stalls when extracts overlapped. The phases below
// (EN :00/:30, JP :07/:37, CN :15/:45, KR :22/:52) are the ones the four processes
// had; they still spread the checks, but it is the queue, not the phase, that now
// guarantees no overlap. A check that wakes while another region extracts waits
// its turn (state "queued" on its socket) instead of running alongside.
//
// Resource guard (each has a kill switch, see ws/config.mjs):
//   WS_NICE=10, WS_IONICE_CLASS=2 / WS_IONICE_LEVEL=7: the downloader and unpacker
//     run under `ionice -c 2 -n 7 nice -n 10`, so the backend wins the CPU and disk.
//   WS_MAX_UNPACK_JOBS: caps -j across regions (default cores-1; each region asks for 1).
//   WS_MIN_FREE_MB=1536: a job does not start below this much MemAvailable; it is
//     re-tried every WS_MEM_RETRY_MIN and skipped after WS_MEM_MAX_DEFER_MIN. The
//     extract re-checks after the download and waits up to WS_MEM_WAIT_MIN. The
//     1536 is a TRADE, not a measured unpacker peak: about 15% of the box.
const SERVERS = [
    // EN holds the interval phase; the others are offsets from it.
    { server: "en", port: 9160, profile: "full", startDelayMin: 0 },
    // `release` adds the event / banner / skin-brand art the Release Planner
    // serves for CN-only content; without it the CN preview has names and dates
    // but no pictures.
    { server: "cn", port: 9161, profile: "operators,release", startDelayMin: 15 },
    // JP and KR exist for their TEXT: the game-data picker and the `ja`/`ko`
    // locales read their excel and story tables, never their art, so `gamedata`
    // is all they pull (74.6 to 77.6 MB of bundles, an extract of 16 to 40 s
    // locally on 2026-10-06).
    { server: "jp", port: 9162, profile: "gamedata", startDelayMin: 7 },
    { server: "kr", port: 9163, profile: "gamedata", startDelayMin: 22 },
];

module.exports = {
    apps: [
        {
            name: "myrtle-ws",
            script: "run.mjs",
            interpreter: "node",
            cwd: "/var/www/myrtle.moe/assets",
            args: "ws",
            // The node process itself idles near 50 MB; the unpacker is a child
            // process and does not count toward this.
            max_memory_restart: "2G",
            // A restart mid-extract kills the child (SIGKILL); the region's backoff
            // and stamps are on disk, so the next tick resumes cleanly.
            kill_timeout: 5000,
            env: {
                WS_SERVERS: JSON.stringify(SERVERS),
                WS_THREADS: "1",
                WS_INTERVAL: "30",
                WS_MIN_FREE_MB: "1536",
                WS_NICE: "10",
                WS_IONICE_CLASS: "2",
            },
        },
    ],
};
