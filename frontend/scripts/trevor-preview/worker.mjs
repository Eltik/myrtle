// The worker side of contract v1 section 3, for the real worker: claims, heartbeats, deltas, results and the job streams.
import crypto from "node:crypto";
import { cors, GATE_POS, json, sseEvent, VERSION, WORKER_KEY } from "./config.mjs";
import { PROFILE } from "./decide.mjs";
import { JOBS, realJobs } from "./store.mjs";

/** The worker's last heartbeat: { at, workerId, rungs, busyJobId, corpusVersion, guard }. */
export let beat = null;
export const workerOnline = (now = Date.now()) => beat !== null && now - beat.at < 75_000;
/** Queued real jobs in claim order: normal before deferred, then the user with the fewest running, then the oldest. */
function claimOrder() {
    const running = new Map();
    for (const j of realJobs()) if (j.status === "running") running.set(j.uid, (running.get(j.uid) ?? 0) + 1);
    return realJobs()
        .filter((j) => j.status === "queued")
        .sort((a, b) => Number(a.deferred) - Number(b.deferred) || (running.get(a.uid) ?? 0) - (running.get(b.uid) ?? 0) || a.created - b.created);
}
/** Seconds a finished live job took, for the ETA; the fallback is a guess per profile. */
const took = [];
const perJob = () => (took.length ? took.reduce((a, b) => a + b, 0) / took.length : PROFILE === "full" ? 90 : 30);
const waiters = new Set();
export const enqueued = () => {
    for (const w of [...waiters]) w();
};

export function realView(j) {
    const now = Date.now();
    const base = { jobId: j.id, rung: "cpu", workerOnline: workerOnline(now), error: null };
    if (j.status === "done") return { ...base, status: "done", rung: j.answer.rung, position: 0, etaSecs: null, answer: j.answer };
    if (j.status === "failed") return { ...base, status: "failed", position: 0, etaSecs: null, answer: null, error: j.error };
    if (j.status === "running") return { ...base, status: "running", position: 0, etaSecs: Math.max(1, Math.ceil(perJob() - (now - j.started) / 1000)), answer: null };
    const position = claimOrder().indexOf(j) + 1;
    const running = realJobs().some((o) => o.status === "running") ? 1 : 0;
    return { ...base, status: "queued", position, etaSecs: workerOnline(now) ? Math.ceil(perJob() * (position + running)) : null, answer: null };
}

/** Send an SSE frame to every open stream of a job (null closes them). */
function fanOut(j, frame) {
    for (const put of [...j.subs]) put(frame);
}

/** The latest cited gatePos against the job's horizon, for the log: "3 citations, latest gatePos 701 <= 722". */
function citeCheck(j) {
    const cites = j.answer?.answer?.citations ?? [];
    const pos = cites.map((c) => GATE_POS.get(c.storyId));
    const placed = pos.filter((p) => p !== undefined);
    const latest = placed.length ? Math.max(...placed) : 0;
    const unplaced = pos.length - placed.length;
    return `${cites.length} citations, latest gatePos ${latest} ${latest <= j.horizon.eraPos ? "<=" : "> (OUTSIDE)"} horizon ${j.horizon.eraPos}${unplaced ? `, ${unplaced} not on the story scale` : ""}`;
}

export function finish(j, body) {
    j.finished = Date.now();
    if (body.ok && body.result) {
        const r = body.result;
        j.status = "done";
        j.answer = { id: crypto.randomUUID(), version: VERSION, server: "en", eraPos: j.horizon.eraPos, rung: ["cf", "cpu", "passages"].includes(r.rung) ? r.rung : "cpu", source: r.source === "live" ? "live" : "passages", answer: r };
        if (r.source === "live") took.push((j.finished - j.started) / 1000);
        fanOut(j, sseEvent("done", realView(j)));
    } else {
        j.status = "failed";
        j.error = String(body.error ?? "the worker failed");
        fanOut(j, sseEvent("error", { message: j.error }));
    }
    fanOut(j, null);
}

/** Constant-time compare of the worker key (both sides hashed to one length first). */
function workerKeyOk(req) {
    const got = crypto.createHash("sha256").update(req.headers.get("x-trevor-worker-key") ?? "").digest();
    return crypto.timingSafeEqual(got, crypto.createHash("sha256").update(WORKER_KEY).digest());
}

async function nextJob(req) {
    const until = Date.now() + 20_000;
    for (;;) {
        if (req.signal?.aborted) return new Response(null, { status: 204 });
        const j = claimOrder()[0];
        if (j) {
            j.status = "running";
            j.started = Date.now();
            // A spoiler-mode job has no limit: the worker gets horizonPos null (contract v1.1 section 8).
            const horizonPos = j.horizon.all ? null : j.horizon.eraPos;
            console.log(`${j.id} claimed after ${((j.started - j.created) / 1000).toFixed(1)} s: horizonPos ${horizonPos}, profile ${PROFILE}${j.deferred ? ", deferred" : ""}`);
            return json({ jobId: j.id, question: j.question, server: "en", horizonPos, rung: "cpu", profile: PROFILE, flags: [] });
        }
        const left = until - Date.now();
        if (left <= 0) return new Response(null, { status: 204 });
        await new Promise((resolve) => {
            const done = () => {
                clearTimeout(t);
                waiters.delete(done);
                req.signal?.removeEventListener?.("abort", done);
                resolve();
            };
            const t = setTimeout(done, left);
            waiters.add(done);
            req.signal?.addEventListener?.("abort", done);
        });
    }
}

export async function workerRoute(req, p) {
    if (!workerKeyOk(req)) return json({ error: { code: "UNAUTHORIZED", message: "worker key" } }, 401);
    if (p === "next") return nextJob(req);
    const raw = await req.text();
    if (p === "result" && Buffer.byteLength(raw) > 64 * 1024) return json({ error: { code: "PAYLOAD_TOO_LARGE", message: "over 64 KB" } }, 413);
    let body = {};
    try {
        body = JSON.parse(raw || "{}");
    } catch {
        return json({ error: { code: "BAD_REQUEST", message: "not JSON" } }, 400);
    }
    if (p === "heartbeat") {
        const prev = beat;
        const guard = body.guard ?? null;
        beat = { at: Date.now(), workerId: body.workerId, rungs: body.rungs ?? {}, busyJobId: body.busyJobId ?? null, corpusVersion: body.corpusVersion, guard };
        if (!prev) console.log(`worker online: ${body.workerId}, rungs ${JSON.stringify(body.rungs)}, corpus ${body.corpusVersion}`);
        else if (prev.rungs?.cpu !== beat.rungs.cpu) console.log(`worker rungs now ${JSON.stringify(beat.rungs)}`);
        if (guard?.state === "tripped" && prev?.guard?.state !== "tripped") console.log(`worker power guard TRIPPED: ${guard.reason} (since ${guard.since})`);
        if (guard?.state === "ok" && prev?.guard?.state === "tripped") console.log("worker power guard recovered");
        return new Response(null, { status: 204 });
    }
    const delta = p.match(/^jobs\/([^/]+)\/delta$/);
    if (delta) {
        const j = JOBS.get(decodeURIComponent(delta[1] ?? ""));
        if (!j?.real || j.status !== "running") return json({ error: { code: "NOT_FOUND", message: "no running job" } }, 404);
        const seq = Number(body.seq);
        if (!Number.isInteger(seq) || typeof body.text !== "string") return json({ error: { code: "BAD_REQUEST", message: "seq and text" } }, 400);
        j.firstDelta ??= Date.now();
        // Appended in seq order: an early one waits for the ones before it, a repeat is dropped.
        if (seq >= j.nextSeq) j.pending.set(seq, body.text);
        while (j.pending.has(j.nextSeq)) {
            const text = j.pending.get(j.nextSeq);
            j.pending.delete(j.nextSeq);
            fanOut(j, sseEvent("delta", { seq: j.nextSeq, text }));
            j.nextSeq += 1;
        }
        return new Response(null, { status: 204 });
    }
    if (p === "result") {
        const j = JOBS.get(String(body.jobId ?? ""));
        if (!j?.real || j.status !== "running") return json({ error: { code: "NOT_FOUND", message: "no running job" } }, 404);
        finish(j, body);
        const r = body.result ?? {};
        console.log(
            `${j.id} ${j.status} (${r.rung ?? "-"}, ${r.source ?? "-"}): first delta ${j.firstDelta ? j.firstDelta - j.created : "-"} ms, done ${j.finished - j.created} ms after the ask; ${j.status === "done" ? citeCheck(j) : j.error}`,
        );
        return new Response(null, { status: 204 });
    }
    return json({ error: { code: "NOT_FOUND", message: "no such worker route" } }, 404);
}

/** The SSE stream of a real job: deltas from now on (the backend does not keep them), then done or error. */
export function realStream(j) {
    const enc = new TextEncoder();
    let put = null;
    let tick = null;
    const body = new ReadableStream({
        start(c) {
            put = (s) => {
                if (s === null) {
                    clearInterval(tick);
                    j.subs.delete(put);
                    try {
                        c.close();
                    } catch {}
                } else {
                    try {
                        c.enqueue(enc.encode(s));
                    } catch {}
                }
            };
            put(": keepalive\n\n");
            if (j.status === "done") put(sseEvent("done", realView(j)));
            if (j.status === "failed") put(sseEvent("error", { message: j.error }));
            if (j.status === "done" || j.status === "failed") return put(null);
            j.subs.add(put);
            tick = setInterval(() => put(": keepalive\n\n"), 15_000);
        },
        cancel() {
            clearInterval(tick);
            j.subs.delete(put);
        },
    });
    return new Response(body, { headers: { "content-type": "text/event-stream", "cache-control": "no-cache", ...cors } });
}
