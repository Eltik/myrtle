// The real worker: the `ask --worker` process, the llama-server it starts, and stopping both by pid.
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { ASK_BIN, alive, LOCK, sh, TREVOR, WORKER_KEY } from "./config.mjs";
import { DECIDED, PROFILE } from "./decide.mjs";
import { realJobs } from "./store.mjs";
import { finish } from "./worker.mjs";

let worker = null;
let workerPid = null;
/** llama-server pids seen as children of the worker: the only ones this preview ever stops. */
const llamaPids = new Set();
let spool = null;
const isLlama = (pid) => alive(pid) && sh(`ps -o comm= -p ${pid}`).trim().endsWith("llama-server");
function trackLlama() {
    if (!worker) return;
    const out = spawnSync("pgrep", ["-P", String(worker.pid), "-x", "llama-server"], { encoding: "utf8" }).stdout ?? "";
    for (const pid of (out.match(/\d+/g) ?? []).map(Number)) {
        if (!llamaPids.has(pid)) console.log(`worker started llama-server (pid ${pid})`);
        llamaPids.add(pid);
    }
}
/** The model lock, removed only while it still names the worker (a SIGKILLed worker cannot remove its own). */
function releaseLock(pid) {
    if (!pid) return;
    try {
        if (fs.readFileSync(LOCK, "utf8").split(/\s+/)[0] === String(pid)) {
            fs.rmSync(LOCK, { force: true });
            console.log(`removed the model lock the worker (pid ${pid}) left`);
        }
    } catch {}
}
export const signal = (pid, sig) => {
    try {
        process.kill(pid, sig);
    } catch {}
};
async function stopLlamas() {
    const mine = [...llamaPids].filter(isLlama);
    for (const pid of mine) signal(pid, "SIGTERM");
    for (let i = 0; i < 50 && mine.some(alive); i++) await Bun.sleep(100);
    for (const pid of mine.filter(alive)) signal(pid, "SIGKILL");
    for (const pid of mine) console.log(`stopped llama-server pid ${pid}: ${alive(pid) ? "STILL ALIVE" : "gone"}`);
    llamaPids.clear();
}

/** Starts the real worker against the mock at `mockUrl` (contract v1 section 3); it starts llama-server itself. */
export function startWorker(mockUrl) {
    for (const n of DECIDED.notes) console.log(`real answers: ${n}`);
    spool = fs.mkdtempSync(path.join(os.tmpdir(), "trevor-preview-spool-"));
    const args = ["--worker", mockUrl, "--profile", PROFILE, "--model", DECIDED.model, "--no-wait"];
    worker = spawn(ASK_BIN, args, {
        cwd: TREVOR,
        stdio: ["ignore", "inherit", "inherit"],
        env: {
            ...process.env,
            TREVOR_WORKER_KEY: WORKER_KEY,
            TREVOR_WORKER_ID: "trevor-preview",
            TREVOR_HEARTBEAT_SECS: process.env.TREVOR_HEARTBEAT_SECS ?? "10",
            // A result the preview could not take is never re-sent to a real backend later.
            TREVOR_SPOOL_DIR: spool,
        },
    });
    workerPid = worker.pid;
    console.log(`real answers: ask ${args.join(" ")} (pid ${worker.pid}), cwd trevor/; it loads its indexes and starts llama-server on :8081 under ${LOCK}`);
    setInterval(trackLlama, 2000).unref();
    worker.on("exit", (code, sig) => {
        trackLlama();
        worker = null;
        console.log(`worker exited (${sig ?? code}); its running jobs fail, queued ones wait, and the status reads offline once its heartbeat is 75 s old`);
        for (const j of realJobs()) if (j.status === "running") finish(j, { ok: false, error: "the worker stopped" });
        // A worker that died without its own cleanup leaves its llama-server and lock behind.
        stopLlamas().then(() => releaseLock(workerPid));
    });
}

/** Stop the worker (it finishes a job in flight first), then the llama-server it started, by pid. */
export async function stopWorker() {
    const w = worker;
    if (w) {
        trackLlama();
        w.kill("SIGTERM");
        for (let i = 0; i < 150 && worker; i++) await Bun.sleep(100);
        if (worker) {
            console.log("worker still running 15 s after SIGTERM; SIGKILL");
            w.kill("SIGKILL");
            for (let i = 0; i < 30 && worker; i++) await Bun.sleep(100);
        }
    }
    await stopLlamas();
    releaseLock(workerPid);
}

/** The last word on exit: SIGKILL the worker and its llama-server, remove the lock it left and the spool. */
export function killWorkerNow() {
    if (worker) signal(worker.pid, "SIGKILL");
    for (const pid of llamaPids) if (isLlama(pid)) signal(pid, "SIGKILL");
    releaseLock(workerPid);
    if (spool) fs.rmSync(spool, { recursive: true, force: true });
}
