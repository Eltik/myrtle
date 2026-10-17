// `bun run dev:trevor`: the Trevor reader panel and the stories "Ask Trevor" tab in the dev server, answered by the
// real Trevor worker against a mock backend, or by canned answers.
//
// Starts the mock backend on 127.0.0.1:3999 (MOCK_PORT) and `vite dev` on 127.0.0.1:3005 (VITE_PORT) with VITE_TREVOR=1
// pointed at it; MOCK_ONLY=1 starts the mock alone. Both bind 127.0.0.1, not localhost, so the demo sign-in cookie
// never lands on the localhost dev server (:3000) or replaces a real session there.
//
// The modules under trevor-preview/:
// - config.mjs: the settings below, the panel manifest, the gatePos scale (contract v1 section 1) and the demo profile.
// - routes.mjs: the mock routes. /trevor/status, /panels/<story> from trevor/artifacts/panels, ask, jobs/<id> and its
//   stream, feedback, popular (this run's answers inside the reader's era plus up to 5 bank answers), me/jobs (a bare
//   array, newest first, paged by before=<createdAt>); /demo-login and /demo-logout (cookie site_token=trevor-demo),
//   /demo-mode/<mode>, /demo-profile/<storyId|none>; every other GET is proxied to MOCK_UPSTREAM
//   (http://localhost:3060) without the demo token, and any other method is refused. At most MOCK_MAX_OPEN (3) open
//   jobs (a further ask is a 409), and a twin (the same normQ in the same era while open) joins the open job.
// - jobs.mjs: the jobs an ask makes. Past TREVOR_PREVIEW_QUEUE_MAX (2) queued or running real jobs, or in mode `quick`,
//   an ask gets a quick passages job (always canned, its answer the closest bank answer inside the era) as `jobId`, a
//   deferred job for the full answer as `deferredJobId`, and this run's earlier-era answer as `quickLook` when there is
//   one (contract v1.1 section 1b).
// - worker.mjs: the worker side of contract v1 section 3 for the real worker (20 s next long-poll, heartbeat, deltas
//   streamed in seq order, result with a 64 KB cap, the x-trevor-worker-key made per run). Claims are fair (normal
//   before deferred, then the user with the fewest running, then the oldest).
// - decide.mjs and spawn.mjs: whether to run `trevor/target/release/ask --worker <mock> --profile <p> --model <m>
//   --no-wait`, and running it. Model: TREVOR_PREVIEW_MODEL, else Gemma 4 E2B when present, else the 12B; the 26B is
//   refused. Profile: TREVOR_PREVIEW_PROFILE, else `mid` with E2B (`slim` when the binary has no `mid`) and `full`
//   with the 12B. Canned answers instead with TREVOR_PREVIEW_REAL=0, a missing binary or model, a llama-server
//   already running, a live holder of the model lock, under 30% free memory, or a thermal warning.
// - canned.mjs: the canned answers, chosen by keyword ("Hibiscus", "Ebenholz", "recap"/"summary"/"so far", "fail"
//   fails half way, anything else a labelled demo answer), each job with its own latency.
// - bank.mjs: the prepared bank answers placed on the gatePos scale (quick looks and popular seeds).
//
// MOCK_MODE picks what an ask returns: stream (default), quick, passages (a cached passages answer), busy (503) or
// offline (queued, then nobody answers); http://127.0.0.1:3999/demo-mode/<mode> switches it while the preview runs.
//
// Stopping the preview (Ctrl+C, SIGTERM, SIGHUP, or vite exiting) stops vite, the worker (SIGTERM, SIGKILL after 15 s)
// and the llama-server the worker started, by pid, and removes the model lock only when it names the worker.
import { spawn, spawnSync } from "node:child_process";
import path from "node:path";
import { BANK } from "./trevor-preview/bank.mjs";
import { PORT, ROOT, STORY, UPSTREAM, VITE_PORT } from "./trevor-preview/config.mjs";
import { DECIDED, REAL } from "./trevor-preview/decide.mjs";
import { MODE, route } from "./trevor-preview/routes.mjs";
import { killWorkerNow, signal, startWorker, stopWorker } from "./trevor-preview/spawn.mjs";

const server = Bun.serve({ hostname: "127.0.0.1", port: PORT, idleTimeout: 120, fetch: route });
const mockUrl = `http://127.0.0.1:${PORT}`;
console.log(`trevor mock on ${mockUrl} (mode ${MODE}), upstream GETs to ${UPSTREAM}; ${BANK.length} bank answers placed on the gatePos scale`);

if (REAL) startWorker(mockUrl);
else console.log(`canned answers (mock): ${DECIDED.why}`);

let viteProc = null;
/** vite and the children bunx started for it. */
function stopVite(sig) {
    if (!viteProc?.pid) return;
    const kids = (spawnSync("pgrep", ["-P", String(viteProc.pid)], { encoding: "utf8" }).stdout?.match(/\d+/g) ?? []).map(Number);
    for (const pid of [viteProc.pid, ...kids]) signal(pid, sig);
}
let stopping = false;
const stopAll = async (code = 0) => {
    if (stopping) return;
    stopping = true;
    stopVite("SIGTERM");
    await stopWorker();
    server.stop(true);
    process.exit(code);
};
process.on("SIGINT", () => stopAll(0));
process.on("SIGTERM", () => stopAll(0));
process.on("SIGHUP", () => stopAll(0));
process.on("uncaughtException", (e) => {
    console.error(e);
    stopAll(1);
});
// The last word on any exit path: nothing this preview started outlives it.
process.on("exit", () => {
    stopVite("SIGKILL");
    killWorkerNow();
});

if (process.env.MOCK_ONLY === "1") {
    console.log("MOCK_ONLY=1: not starting vite");
} else {
    viteProc = spawn("bunx", ["vite", "dev", "--host", "127.0.0.1", "--port", String(VITE_PORT), "--strictPort"], {
        cwd: path.join(ROOT, "frontend"),
        stdio: "inherit",
        env: { ...process.env, VITE_TREVOR: "1", BACKEND_URL: mockUrl, VITE_BACKEND_URL: mockUrl },
    });
    console.log(`reader with Trevor: http://127.0.0.1:${VITE_PORT}/stories/${STORY}  (sign in as the demo: ${mockUrl}/demo-login)`);
    viteProc.on("exit", (code) => stopAll(code ?? 0));
}
