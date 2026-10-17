// `bun run dev:trevor`: the Trevor reader panel and the stories "Ask Trevor" tab in the dev
// server, answered by the real Trevor worker against a mock backend.
//
// Starts a mock backend on 127.0.0.1:3999 (MOCK_PORT) and `vite dev` on 127.0.0.1:3005
// (VITE_PORT) with VITE_TREVOR=1 pointed at it. MOCK_ONLY=1 starts the mock alone. Both bind
// 127.0.0.1, not localhost, so the demo sign-in cookie never lands on the localhost dev server
// (:3000) or replaces a real session there.
//
// Real answers (contract v1.1 section 7). The mock speaks the worker side of
// trevor/design/trevor-contract-v1.md section 3 (worker/next long-poll of 20 s, heartbeat,
// jobs/<id>/delta, result with a 64 KB cap; header x-trevor-worker-key with a key made per run
// and handed to the child as TREVOR_WORKER_KEY) and starts
// `trevor/target/release/ask --worker <mock> --profile <p> --model <m> --no-wait` from trevor/.
// - Model: TREVOR_PREVIEW_MODEL, else Gemma 4 E2B (models/llm/gemma-4-e2b-it-q4_k_m.gguf) when
//   it exists, else the 12B (gemma-4-12b-it-qat-q4_0.gguf). The 26B is refused.
// - Profile: TREVOR_PREVIEW_PROFILE, else `mid` with E2B and `full` with the 12B. The profiles
//   the binary knows are read from its own argument error at start; `mid` falls back to `slim`,
//   with a log line, while the binary has no `mid`.
// - The worker starts llama-server itself (port 8081, its own slot size) under the model lock
//   /private/tmp/claude-501/trevor-llm.lock. Real mode is refused, and the canned answers served,
//   when any llama-server is already running, when a live process holds the lock (a lock naming
//   a dead pid is removed), when `memory_pressure` reports under 30% free, or when
//   `pmset -g therm` records a thermal warning.
// - Every ask queues a job for the worker: rung cpu, horizonPos = the job's era position (as the
//   backend sends it). The horizon is contract v1 section 1: pos = min(gatePos(horizonStory),
//   profilePos), eraPos = the last era at or before pos, gatePos and eras from the panels
//   manifest. The demo profile has read up to STORY; /demo-profile/<storyId> moves it and
//   /demo-profile/none empties it (gatePos 0). An unknown horizonStory is a 400, as in the backend.
// - The worker's deltas go out on the job's SSE stream in seq order, and its result becomes the
//   done view. Claiming is fair (the user with the fewest running jobs, then the oldest job), and
//   deferred jobs are claimed only when no normal job waits.
// - Quick look (contract v1.1 section 1b): when TREVOR_PREVIEW_QUEUE_MAX (default 2) jobs are
//   already queued or running, or in mode `quick`, the ask gets a quick passages job (always
//   canned: the closest prepared bank answer inside the era and its cited passages, done within
//   a second) and a deferred job for the full answer, which then streams like any other. The
//   response carries the quick job as `jobId`, the deferred one as `deferredJobId`, and, when
//   this run answered the same question in an earlier era, that answer as `quickLook`.
// - Stopping the preview (Ctrl+C, SIGTERM, SIGHUP, or vite exiting) stops vite, the worker (SIGTERM,
//   SIGKILL after 15 s) and the llama-server the worker started (its child pid, recorded every
//   2 s), and removes the model lock only when it names the worker. No other llama-server is
//   touched.
//
// Canned answers, labelled, as before: with TREVOR_PREVIEW_REAL=0, or when the binary or the
// model is missing, or any refusal above. The startup line says which and why. Each canned ask is
// its own job with its own latency (0.8 to 4 s in the queue, 50 to 150 ms a word). The answer
// depends on the question: a canned one for "Hibiscus" or "Ebenholz", the story's recap for
// "recap"/"summary"/"so far", a stream that fails half way for "fail", and otherwise a reply
// marked "Demo answer (mock backend)".
//
// In both: at most MOCK_MAX_OPEN (default 3) open jobs (a further ask is a 409), a twin (the
// same normalized question in the same era while open) joins the open job, and a stream opened
// late carries only the deltas still to come. Other routes:
// - /trevor/status (adds workers, guard and mode), /trevor/panels/<story> from
//   trevor/artifacts/panels, /trevor/jobs/<id> and its stream, feedback;
// - GET /api/trevor/popular?limit=8: this run's answers inside the reader's era (each counted as
//   2 askers) and up to 5 bank answers inside it; anonymous gets an empty list;
// - GET /api/trevor/me/jobs?limit=20&before=<createdAt>: this run's asks, newest first, a bare
//   array of the backend's rows (a deferred ask is listed under its deferred job);
// - a signed-in session for the cookie `site_token=trevor-demo` (/demo-login sets it,
//   /demo-logout clears it);
// - every other GET is proxied to the real backend (MOCK_UPSTREAM, default
//   http://localhost:3060) without the demo token; any other method is refused.
//
// MOCK_MODE picks what an ask returns: stream (default), quick (quick look plus a deferred job),
// passages (a cached passages-only answer), busy (503), offline (queued, then nobody answers).
// http://127.0.0.1:3999/demo-mode/<mode> switches it while the preview runs.
import { spawn, spawnSync } from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const ROOT = path.resolve(import.meta.dir, "../..");
const TREVOR = path.join(ROOT, "trevor");
const PANELS = path.join(TREVOR, "artifacts/panels");
const PORT = Number(process.env.MOCK_PORT ?? 3999);
const VITE_PORT = Number(process.env.VITE_PORT ?? 3005);
const UPSTREAM = process.env.MOCK_UPSTREAM ?? "http://localhost:3060";
const MODES = ["stream", "quick", "passages", "busy", "offline"];
let MODE = process.env.MOCK_MODE ?? "stream";
const DEMO = "trevor-demo";
const DEMO_UID = process.env.MOCK_UID ?? "55699327";
const STORY = "act18side_level_act18side_05_beg";
// Backend MAX_OPEN_PER_USER.
const MAX_OPEN = Number(process.env.MOCK_MAX_OPEN ?? 3);
// Jobs queued or running past which an ask gets a quick look and a deferred job.
const QUEUE_MAX = Number(process.env.TREVOR_PREVIEW_QUEUE_MAX ?? 2);
const ASK_BIN = path.join(TREVOR, "target/release/ask");
const E2B = path.join(TREVOR, "models/llm/gemma-4-e2b-it-q4_k_m.gguf");
const G12B = path.join(TREVOR, "models/llm/gemma-4-12b-it-qat-q4_0.gguf");
const LOCK = "/private/tmp/claude-501/trevor-llm.lock";
const MIN_FREE_PCT = 30;
const WORKER_KEY = crypto.randomBytes(24).toString("hex");

if (!fs.existsSync(path.join(PANELS, "manifest.json"))) {
    console.error(`no panel artifacts at ${PANELS}; build or copy them first`);
    process.exit(1);
}
const MANIFEST = JSON.parse(fs.readFileSync(path.join(PANELS, "manifest.json"), "utf8"));
const VERSION = MANIFEST.version;

// --- the gatePos scale (contract v1 section 1) ---------------------------------
const GATE_POS = new Map(MANIFEST.storyOrder.map((id, i) => [id, i + 1]));
const ERAS = [...MANIFEST.eras].sort((a, b) => a.position - b.position);
const eraAt = (pos) => ERAS.filter((e) => e.position > 0 && e.position <= pos).at(-1) ?? null;
/** The story the demo profile has read up to (null: nothing read). */
let profileStory = STORY;
const profilePos = () => (profileStory ? (GATE_POS.get(profileStory) ?? 0) : 0);
/** The job horizon: pos = min(gatePos(horizonStory), profilePos), its era, or { error } for an unknown story. */
function horizonOf(storyId) {
    let pos = profilePos();
    if (storyId) {
        const s = GATE_POS.get(storyId);
        if (s === undefined) return { error: `horizonStory ${storyId} is not in the active Trevor version` };
        pos = Math.min(s, pos);
    }
    const era = eraAt(pos);
    return { pos, eraPos: era?.position ?? 0, spoiler: era ? { eraId: era.id, name: era.name, position: era.position } : null };
}
/** The contract v1.1 normQ: NFKC, lowercase, punctuation folded, whitespace collapsed (a trailing "?" goes with it). */
const normQ = (q) =>
    q
        .normalize("NFKC")
        .toLowerCase()
        .replace(/[\p{P}\p{S}]+/gu, " ")
        .replace(/\s+/g, " ")
        .trim();

// --- prepared answers from the bank (quick look, popular seeds) ----------------
/** A passage cut to at most 600 characters at a line break, like the worker's citations. */
function cut600(text) {
    if (text.length <= 600) return text;
    const at = text.lastIndexOf("\n", 600);
    return at > 300 ? text.slice(0, at) : text.slice(0, 600);
}
const DECLINES = /^(the (provided|given) (text|passages?)|i cannot|i can't|there is no|it is not)/i;
const STOP = new Set("what when where which while with from that this they them their there does have been were will would could should about into after before trevor".split(" "));
const words = (s) => new Set((s.toLowerCase().match(/[a-z0-9'’]{4,}/g) ?? []).filter((w) => !STOP.has(w)));
/** Bank answers whose every passage is placed on the gatePos scale: { qid, question, text, gatePos, citations, words }. */
const BANK = (() => {
    const file = path.join(TREVOR, "artifacts/bank/answers.jsonl");
    const chunksFile = path.join(TREVOR, "artifacts/p3a/chunks.jsonl");
    if (!fs.existsSync(file) || !fs.existsSync(chunksFile)) return [];
    const rows = [];
    for (const line of fs.readFileSync(file, "utf8").split("\n")) {
        if (!line.trim()) continue;
        try {
            const r = JSON.parse(line);
            // Declines and questions still holding a template ("Dr. {@nickname}") are never offered.
            if (!r.answer || !r.cited?.length || !r.passages?.length || DECLINES.test(r.answer) || /\{@/.test(r.question + r.answer)) continue;
            const pos = r.passages.map((c) => GATE_POS.get(c.split("#")[0]));
            if (pos.some((p) => p === undefined)) continue;
            rows.push({ ...r, gatePos: Math.max(...pos) });
        } catch {}
    }
    const need = new Set(rows.flatMap((r) => r.cited));
    const chunks = new Map();
    for (const line of fs.readFileSync(chunksFile, "utf8").split("\n")) {
        const id = line.match(/^\{"chunkId":"([^"]+)"/)?.[1];
        if (id && need.has(id)) chunks.set(id, JSON.parse(line));
    }
    return rows.map((r) => ({
        qid: r.qid,
        question: r.question,
        text: r.answer,
        gatePos: r.gatePos,
        words: words(r.question),
        citations: r.cited
            .filter((id) => chunks.has(id))
            .map((id) => {
                const c = chunks.get(id);
                return { n: r.passages.indexOf(id) + 1, storyId: c.storyId, chunkId: id, lineStart: c.lineStart, lineEnd: c.lineEnd, text: cut600(c.text) };
            })
            .sort((a, b) => a.n - b.n),
    }));
})();

/** The quick look (contract v1.1 section 1b): the closest bank answer inside the era, as a passages answer. */
function quickLook(question, horizon) {
    const qw = words(question);
    // A name (a capitalized word after the first) counts three words.
    const names = new Set((question.match(/(?<=\S\s+)[A-Z][\p{L}'’-]{2,}/gu) ?? []).map((w) => w.toLowerCase()));
    let best = null;
    let score = -1;
    for (const e of BANK) {
        if (e.gatePos > horizon.eraPos) continue;
        let s = 0;
        for (const w of e.words) if (qw.has(w)) s += names.has(w) ? 3 : 1;
        if (s > score || (s === score && e.gatePos > best.gatePos)) [best, score] = [e, s];
    }
    // A prepared answer that shares one plain word with the question is no quick look at all.
    if (score < 2) best = null;
    const text = best
        ? `Trevor is busy, so this is the closest prepared answer, to "${best.question}":\n\n${best.text}\n\nThe full answer to your question is on its way.`
        : "Trevor is busy, and no prepared answer within your reading so far matches this question. The full answer is on its way.";
    return { id: crypto.randomUUID(), version: VERSION, server: "en", eraPos: horizon.eraPos, rung: "passages", source: "passages", answer: { answer: text, citations: best?.citations ?? [] } };
}

/** Bank answers inside an era as cached answers, newest gatePos first (the popular seeds). */
const seedIds = new Map();
function seeds(eraPos, n) {
    return BANK.filter((e) => e.gatePos <= eraPos && e.citations.length > 0)
        .sort((a, b) => b.gatePos - a.gatePos)
        .slice(0, n)
        .map((e) => {
            if (!seedIds.has(e.qid)) seedIds.set(e.qid, crypto.randomUUID());
            const body = { answer: e.text, rung: "cpu", profile: "full", source: "live", citations: e.citations, route: null, form: null, corpusVersion: MANIFEST.corpusVersion };
            return { question: e.question, answer: { id: seedIds.get(e.qid), version: VERSION, server: "en", eraPos, rung: "cpu", source: "live", answer: body }, askers: 2 };
        });
}

// --- real or canned ---------------------------------------------------------------
const sh = (cmd) => {
    const r = spawnSync("/bin/sh", ["-c", cmd], { encoding: "utf8" });
    return `${r.stdout ?? ""}${r.stderr ?? ""}`;
};
const alive = (pid) => {
    try {
        process.kill(pid, 0);
        return true;
    } catch (e) {
        return e.code === "EPERM";
    }
};
/** Whether the preview can run the real worker: { model, profile, notes } or { why } for canned answers. */
function decideReal() {
    const notes = [];
    if (process.env.TREVOR_PREVIEW_REAL === "0") return { why: "TREVOR_PREVIEW_REAL=0" };
    if (!fs.existsSync(ASK_BIN)) return { why: `${ASK_BIN} is missing (cargo build --release --features embed in trevor/)` };
    const model = process.env.TREVOR_PREVIEW_MODEL ? path.resolve(process.env.TREVOR_PREVIEW_MODEL) : fs.existsSync(E2B) ? E2B : G12B;
    if (/26b/i.test(path.basename(model))) return { why: `${model} is the 26B, which the preview never loads` };
    if (!fs.existsSync(model)) return { why: `${model} is missing` };
    const want = process.env.TREVOR_PREVIEW_PROFILE ?? (model === E2B ? "mid" : "full");
    // clap names the profiles the binary knows when it refuses one.
    const known = sh(`'${ASK_BIN}' --profile zz-preview-probe`)
        .match(/possible values: ([^\]]+)\]/)?.[1]
        ?.split(/,\s*/) ?? ["full", "slim"];
    let profile = want;
    if (!known.includes(want)) {
        if (want !== "mid") return { why: `profile ${want} is not one of ${known.join(", ")}` };
        profile = "slim";
        notes.push(`profile mid is not in this ask binary (it knows ${known.join(", ")}); using slim`);
    }
    const llamas = sh("pgrep -x llama-server").match(/\d+/g);
    if (llamas) return { why: `a llama-server this preview did not start is running (pid ${llamas.join(", ")}); only one runs at a time` };
    let held = "";
    try {
        held = fs.readFileSync(LOCK, "utf8").trim();
    } catch {}
    if (held) {
        const pid = Number(held.split(/\s+/)[0]);
        if (pid > 0 && alive(pid)) return { why: `the model lock ${LOCK} is held by "${held}"` };
        fs.rmSync(LOCK, { force: true });
        notes.push(`removed a stale model lock ("${held}": that pid is gone)`);
    }
    const free = Number(sh("memory_pressure | tail -1").match(/(\d+)%/)?.[1] ?? Number.NaN);
    if (!(free >= MIN_FREE_PCT)) return { why: `memory_pressure reports ${free}% free, under ${MIN_FREE_PCT}%` };
    if (!sh("pmset -g therm").includes("No thermal warning level has been recorded")) return { why: "pmset -g therm records a thermal warning" };
    notes.push(`memory ${free}% free`);
    return { model, profile, notes };
}
const DECIDED = decideReal();
const REAL = !DECIDED.why;
const PROFILE = DECIDED.profile ?? "slim";

const cors = { "access-control-allow-origin": `http://127.0.0.1:${VITE_PORT}`, "access-control-allow-credentials": "true", "access-control-allow-headers": "*", "access-control-allow-methods": "GET, POST, OPTIONS" };
const json = (body, status = 200, headers = {}) => new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json", ...cors, ...headers } });
const iso = (ms) => (ms ? new Date(ms).toISOString() : null);

// --- canned answers ------------------------------------------------------------------
const ANSWER_TEXT =
    "**Hibiscus** is a nurse working in the Afterglow district of Leithanien [1]. She treats the Infected who come to the clinic [1, 2] and argues with Czerny about cancelling the duet [3].\n\n*   She is Kreide's sister [2].\n*   Andante worries she is in no shape to help anyone [3].\n\nOn this evidence, she is the one keeping the clinic running.";
const CITATIONS = [
    { n: 1, storyId: "act18side_level_act18side_04_end", chunkId: "act18side_level_act18side_04_end#0003", lineStart: 41, lineEnd: 58, text: "Hibiscus: Hold still. The crystals have spread past your elbow, and I need to see how far." },
    { n: 2, storyId: STORY, chunkId: `${STORY}#0001`, lineStart: 12, lineEnd: 30, text: "Kreide: You always do this, Hibiscus. Somebody walks in coughing and you forget you were ever tired." },
    { n: 3, storyId: "profile_hibiscus", chunkId: "profile_hibiscus#0000", lineStart: 0, lineEnd: 0, text: "An operator file excerpt." },
];
const SPOILER = { eraId: "main_10", name: "Shatterpoint", position: 722 };
let jobs = 0;
const answer = (source, text = ANSWER_TEXT, citations = CITATIONS) => ({
    id: `11111111-2222-3333-4444-${String(jobs).padStart(12, "0")}`,
    version: VERSION,
    server: "en",
    eraPos: 722,
    rung: source === "live" ? "cf" : "passages",
    source,
    answer: { answer: source === "live" ? text : "", rung: "cf", profile: "full", source, citations, corpusVersion: VERSION },
});
const queuedView = (id) => ({ jobId: id, status: "queued", rung: "cpu", position: 1, etaSecs: null, workerOnline: false, answer: null, error: null });

/**
 * Every job of this run, in creation order. Canned: { id, uid, question, nq, horizon, deferred, quickJob, created,
 * words, sent, fail, deltaMs, start, doneAt, answer }. Real (real: true): { ..., status, started, finished, firstDelta,
 * answer, error, subs, nextSeq, pending }. A deferred job's quick passages job (quick: true) is always canned.
 */
const JOBS = new Map();
const rand = (lo, hi) => lo + Math.random() * (hi - lo);

function panelOf(storyId) {
    const file = path.join(PANELS, "stories", `${path.basename(storyId ?? STORY)}.json`);
    if (!fs.existsSync(file)) return null;
    try {
        return JSON.parse(fs.readFileSync(file, "utf8"));
    } catch {
        return null;
    }
}

/** What the mock answers to a question: text and citations, chosen by keyword. */
function compose(question, storyId) {
    const q = question.toLowerCase();
    const panel = panelOf(storyId) ?? panelOf(STORY);
    const sid = panel?.storyId ?? STORY;
    const summary = panel?.recap?.thisStory?.summary ?? "";
    const synopsis = panel?.recap?.thisStory?.synopsis ?? "";
    const people = (panel?.whosHere?.characters ?? []).map((c) => c.name);
    const own = [
        { n: 1, storyId: sid, chunkId: `${sid}#0002`, lineStart: 18, lineEnd: 40, text: synopsis || "A passage from this story." },
        { n: 2, storyId: sid, chunkId: `${sid}#0005`, lineStart: 77, lineEnd: 96, text: summary.split(". ").slice(0, 2).join(". ") || "Another passage from this story." },
    ];
    if (q.includes("hibiscus")) return { text: ANSWER_TEXT, citations: CITATIONS };
    if (q.includes("ebenholz"))
        return {
            text: "**Ebenholz** is a Leithanien commoner who caught Oripathy in the Vyseheim incident [1]. In this story he leaves Kreide's rehearsal on an urgent errand and follows a rumour into the sewers [2], where he corners Lachmann.\n\nHe does not learn who hired Lachmann before someone appears behind them [2].",
            citations: own,
        };
    if (/recap|summar|so far|what happen/.test(q) && summary) {
        const sentences = summary.split(/(?<=\.) /);
        const half = Math.ceil(sentences.length / 2);
        return {
            text: `${sentences.slice(0, half).join(" ")} [1]\n\n${sentences.slice(half).join(" ")} [2]\n\n*   Who is on stage: ${people.slice(0, 6).join(", ")} [1].\n*   What is unresolved: who sent Lachmann [2].\n\nOn this evidence, the concert is the thing everyone is circling.`,
            citations: own,
        };
    }
    return {
        text: `**Demo answer (mock backend)** to: "${question}"\n\nThe preview has no Trevor behind it, so this reply only repeats your question. The real answer would cite passages like these from ${panel?.title ?? "this story"} [1, 2].`,
        citations: own,
        fail: q.includes("fail"),
    };
}

const isOpen = (j, now = Date.now()) => (j.real ? j.status === "queued" || j.status === "running" : now < j.doneAt);
const openJobs = (now = Date.now()) => [...JOBS.values()].filter((j) => isOpen(j, now));

function jobView(j, now = Date.now()) {
    if (now >= j.doneAt) {
        if (j.fail) return { jobId: j.id, status: "failed", rung: j.rung ?? "cf", position: 0, etaSecs: null, workerOnline: true, answer: null, error: "the mock was asked to fail" };
        return { jobId: j.id, status: "done", rung: j.rung ?? "cf", position: 0, etaSecs: null, workerOnline: true, answer: j.answer, error: null };
    }
    if (now >= j.start) return { jobId: j.id, status: "running", rung: "cf", position: 0, etaSecs: Math.ceil((j.doneAt - now) / 1000), workerOnline: true, answer: null, error: null };
    const ahead = openJobs(now).filter((o) => !o.real && o.start < j.start).length;
    return { jobId: j.id, status: "queued", rung: "cf", position: ahead + 1, etaSecs: Math.ceil((j.doneAt - now) / 1000), workerOnline: true, answer: null, error: null };
}

function demoSession(req) {
    return req.headers.get("authorization") === `Bearer ${DEMO}` || (req.headers.get("cookie") ?? "").includes(`site_token=${DEMO}`);
}

// --- the real worker (contract v1 section 3) ----------------------------------------
/** The worker's last heartbeat: { at, workerId, rungs, busyJobId, corpusVersion, guard }. */
let beat = null;
const workerOnline = (now = Date.now()) => beat !== null && now - beat.at < 75_000;
const realJobs = () => [...JOBS.values()].filter((j) => j.real);
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
const enqueued = () => {
    for (const w of [...waiters]) w();
};

function realView(j) {
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

function finish(j, body) {
    j.finished = Date.now();
    if (body.ok && body.result) {
        const r = body.result;
        j.status = "done";
        j.answer = { id: crypto.randomUUID(), version: VERSION, server: "en", eraPos: j.horizon.eraPos, rung: ["cf", "cpu", "passages"].includes(r.rung) ? r.rung : "cpu", source: r.source === "live" ? "live" : "passages", answer: r };
        if (r.source === "live") took.push((j.finished - j.started) / 1000);
        fanOut(j, `event: done\ndata: ${JSON.stringify(realView(j))}\n\n`);
    } else {
        j.status = "failed";
        j.error = String(body.error ?? "the worker failed");
        fanOut(j, `event: error\ndata: ${JSON.stringify({ message: j.error })}\n\n`);
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
            console.log(`${j.id} claimed after ${((j.started - j.created) / 1000).toFixed(1)} s: horizonPos ${j.horizon.eraPos}, profile ${PROFILE}${j.deferred ? ", deferred" : ""}`);
            return json({ jobId: j.id, question: j.question, server: "en", horizonPos: j.horizon.eraPos, rung: "cpu", profile: PROFILE, flags: [] });
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

async function workerRoute(req, p) {
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
            fanOut(j, `event: delta\ndata: ${JSON.stringify({ seq: j.nextSeq, text })}\n\n`);
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
function realStream(j) {
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
            if (j.status === "done") put(`event: done\ndata: ${JSON.stringify(realView(j))}\n\n`);
            if (j.status === "failed") put(`event: error\ndata: ${JSON.stringify({ message: j.error })}\n\n`);
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

/** One row of "My questions" (contract v1.1 section 5). */
function jobRow(j) {
    const v = j.real ? realView(j) : jobView(j);
    const ended = v.status === "done" || v.status === "failed";
    return {
        jobId: j.id,
        question: j.question,
        status: v.status,
        rung: v.rung,
        source: v.answer?.source ?? null,
        createdAt: iso(j.created),
        finishedAt: ended ? iso(j.real ? j.finished : j.doneAt) : null,
        answer: v.answer ?? null,
    };
}

/** The quick passages job of a deferred ask: canned in both modes, done within a second, its answer the bank's quick look. */
function newQuickJob(id, question, horizon) {
    const start = Date.now() + rand(300, 900);
    const j = { id, uid: DEMO_UID, question, nq: normQ(question), horizon, deferred: false, quick: true, created: Date.now(), rung: "passages", words: [], sent: 0, fail: false, deltaMs: 0, start, doneAt: start, answer: quickLook(question, horizon) };
    JOBS.set(id, j);
    return j;
}

/** A new job for an ask: real (queued for the worker) or canned (on its own clock). A deferred one comes with its quick passages job. */
function newJob(question, body, horizon, deferred) {
    const now = Date.now();
    const id = `job-${jobs}`;
    const common = { id, uid: DEMO_UID, question, nq: normQ(question), horizon, deferred, quickJob: deferred ? newQuickJob(`${id}-quick`, question, horizon).id : null, created: now };
    if (REAL) {
        const j = { ...common, real: true, status: "queued", subs: new Set(), nextSeq: 0, pending: new Map() };
        JOBS.set(id, j);
        enqueued();
        return j;
    }
    const c = compose(question, body.horizonStory);
    const words = c.text.split(/(?<= )/);
    const deltaMs = Number(process.env.MOCK_DELTA_MS ?? rand(50, 150));
    // A deferred canned job waits as if behind a full queue.
    const start = now + (deferred ? rand(5000, 9000) : Number(process.env.MOCK_QUEUE_MS ?? rand(800, 4000)));
    // A failing job stops half way through its words.
    const sent = c.fail ? Math.floor(words.length / 2) : words.length;
    const j = { ...common, words, sent, fail: !!c.fail, deltaMs, start, doneAt: start + sent * deltaMs, answer: answer("live", c.text, c.citations) };
    JOBS.set(id, j);
    return j;
}

/** This run's answer to the same question from an earlier era, the backend's `quickLook`; null when there is none. */
function earlierAnswer(j) {
    for (const o of JOBS.values()) {
        if (o.quick || o.nq !== j.nq || o.horizon.eraPos >= j.horizon.eraPos) continue;
        const v = o.real ? realView(o) : jobView(o);
        if (v.status === "done" && v.answer) return v.answer;
    }
    return null;
}

/** POST /api/trevor/ask's answer for a job: queued, or a deferred job's quick passages job (contract v1.1 section 1b). */
function accepted(j) {
    const spoiler = REAL ? j.horizon.spoiler : (j.horizon.spoiler ?? SPOILER);
    const shown = j.deferred ? JOBS.get(j.quickJob) : j;
    const v = shown.real ? realView(shown) : jobView(shown);
    const out = { jobId: shown.id, status: v.status === "running" ? "running" : "queued", rung: shown.real ? "cpu" : "cf", spoilerSafeUpTo: spoiler, etaSecs: v.etaSecs, streamUrl: `/api/trevor/jobs/${shown.id}/stream` };
    if (!j.deferred) return json(out, 202);
    const look = earlierAnswer(j);
    return json({ ...out, rung: "passages", deferredJobId: j.id, ...(look ? { quickLook: look } : {}) }, 202);
}

const server = Bun.serve({
    hostname: "127.0.0.1",
    port: PORT,
    idleTimeout: 120,
    async fetch(req) {
        const url = new URL(req.url);
        const p = url.pathname;
        if (req.method === "OPTIONS") return new Response(null, { status: 204, headers: cors });
        const demo = demoSession(req);

        if (p === "/demo-login" || p === "/demo-logout") {
            const cookie = p === "/demo-login" ? `site_token=${DEMO}; Path=/; SameSite=Lax` : "site_token=; Path=/; Max-Age=0";
            return new Response(null, { status: 302, headers: { "set-cookie": cookie, location: `http://127.0.0.1:${VITE_PORT}/stories/${STORY}` } });
        }
        const mode = p.match(/^\/demo-mode\/(\w+)$/);
        if (mode) {
            if (!MODES.includes(mode[1] ?? "")) return json({ error: `mode is one of ${MODES.join(", ")}` }, 400);
            MODE = mode[1] ?? MODE;
            return json({ mode: MODE });
        }
        const prof = p.match(/^\/demo-profile\/([^/]+)$/);
        if (prof) {
            const id = decodeURIComponent(prof[1] ?? "");
            if (id !== "none" && !GATE_POS.has(id)) return json({ error: `${id} is not in the active Trevor version (or use none)` }, 400);
            profileStory = id === "none" ? null : id;
            const h = horizonOf(null);
            return json({ profileStory, profilePos: h.pos, spoilerSafeUpTo: h.spoiler });
        }
        if (p === "/api/auth/verify" && demo) return json({ valid: true, uid: DEMO_UID, role: "user", canAccessAdminPanel: false });
        if (p === "/api/trevor/status") {
            if (REAL && (MODE === "stream" || MODE === "quick")) {
                const online = workerOnline();
                const cpu = online && !!beat.rungs.cpu;
                return json({
                    online,
                    rungs: { cf: false, cpu },
                    activeVersion: VERSION,
                    queueDepth: { cf: 0, cpu: openJobs().filter((j) => j.real).length },
                    cfBudget: "ok",
                    workers: { cf: 0, cpu: cpu ? 1 : 0 },
                    guard: beat?.guard?.state === "tripped" ? "tripped" : "ok",
                    mode: "on",
                });
            }
            return json({ online: MODE !== "offline", rungs: { cf: true, cpu: true }, activeVersion: VERSION, queueDepth: { cf: 0, cpu: 1 }, cfBudget: "ok", workers: { cf: 0, cpu: MODE === "offline" ? 0 : 1 }, guard: "ok", mode: "on" });
        }
        const worker = p.match(/^\/api\/trevor\/worker\/(.+)$/);
        if (worker && req.method === "POST") return REAL ? workerRoute(req, worker[1] ?? "") : json({ error: { code: "SERVICE_UNAVAILABLE", message: "the preview serves canned answers" } }, 503);
        const panel = p.match(/^\/api\/trevor\/panels\/(.+)$/);
        if (panel) {
            const file = path.join(PANELS, "stories", `${path.basename(decodeURIComponent(panel[1] ?? ""))}.json`);
            if (!fs.existsSync(file)) return json({ error: { code: "NOT_FOUND", message: "no Trevor panels" } }, 404);
            return json({ storyId: panel[1], server: "en", version: VERSION, panels: JSON.parse(fs.readFileSync(file, "utf8")) });
        }
        if (p === "/api/trevor/popular") {
            if (!demo) return json({ spoilerSafeUpTo: null, items: [] });
            const h = horizonOf(null);
            const limit = Math.min(Math.max(Number(url.searchParams.get("limit") ?? 8) || 8, 1), 20);
            const seen = new Set();
            const asked = [];
            for (const j of [...JOBS.values()].reverse()) {
                const v = j.real ? realView(j) : jobView(j);
                // A decline ("The provided passages do not ...") is not worth suggesting.
                const text = v.answer?.answer?.answer;
                if (j.quick || v.status !== "done" || !v.answer || v.answer.eraPos > h.eraPos || seen.has(j.nq) || (typeof text === "string" && DECLINES.test(text))) continue;
                seen.add(j.nq);
                asked.push({ question: j.question, answer: v.answer, askers: 2 });
            }
            const seeded = seeds(h.eraPos, 5).filter((s) => !seen.has(normQ(s.question)));
            return json({ spoilerSafeUpTo: h.spoiler, items: [...asked, ...seeded].slice(0, limit) });
        }
        if (p === "/api/trevor/me/jobs") {
            if (!demo) return json({ error: { code: "UNAUTHORIZED", message: "sign in" } }, 401);
            const limit = Math.min(Math.max(Number(url.searchParams.get("limit") ?? 20) || 20, 1), 50);
            const before = Date.parse(url.searchParams.get("before") ?? "");
            const rows = [...JOBS.values()].filter((j) => j.uid === DEMO_UID && !j.quick && !(j.created >= before)).reverse();
            return json(rows.slice(0, limit).map(jobRow));
        }
        if (p === "/api/trevor/ask" && req.method === "POST") {
            if (!demo) return json({ error: { code: "UNAUTHORIZED", message: "sign in" } }, 401);
            jobs += 1;
            if (MODE === "busy") return json({ error: { code: "SERVICE_UNAVAILABLE", message: "Trevor is busy" } }, 503);
            if (MODE === "passages") return json({ jobId: null, status: "done", rung: "cache", spoilerSafeUpTo: SPOILER, answer: answer("passages"), etaSecs: null, streamUrl: null });
            if (MODE === "offline") {
                const id = `job-${jobs}`;
                return json({ jobId: id, status: "queued", rung: "cf", spoilerSafeUpTo: SPOILER, etaSecs: null, streamUrl: `/api/trevor/jobs/${id}/stream` }, 202);
            }
            const body = await req.json().catch(() => ({}));
            const question = String(body.question ?? "").trim();
            if (!question) return json({ error: { code: "BAD_REQUEST", message: "empty question" } }, 400);
            let horizon = horizonOf(body.horizonStory);
            if (horizon.error) {
                if (REAL) return json({ error: { code: "BAD_REQUEST", message: horizon.error } }, 400);
                horizon = horizonOf(null);
            }
            const open = openJobs();
            const nq = normQ(question);
            const twin = open.find((j) => !j.quick && j.nq === nq && j.horizon.eraPos === horizon.eraPos);
            if (twin) return accepted(twin);
            if (open.length >= MAX_OPEN) return json({ error: { code: "CONFLICT", message: `at most ${MAX_OPEN} questions may wait at once; wait for one to finish` } }, 409);
            const full = REAL && open.filter((j) => j.real).length >= QUEUE_MAX;
            return accepted(newJob(question, body, horizon, MODE === "quick" || full));
        }
        const stream = p.match(/^\/api\/trevor\/jobs\/(job-\d+(?:-quick)?)\/stream$/);
        if (stream) {
            const j = JOBS.get(stream[1] ?? "");
            if (MODE === "offline" || !j) return json({ error: { code: "NOT_FOUND", message: "no stream" } }, 404);
            if (j.real) return realStream(j);
            const enc = new TextEncoder();
            let closed = false;
            const body = new ReadableStream({
                async start(c) {
                    const put = (s) => {
                        if (!closed) c.enqueue(enc.encode(s));
                    };
                    put(": keepalive\n\n");
                    // Deltas go out on the job's own clock: a late stream only gets the ones still to come.
                    for (let i = 0; i < j.sent && !closed; i++) {
                        const at = j.start + i * j.deltaMs;
                        if (Date.now() > at + j.deltaMs) continue;
                        await Bun.sleep(Math.max(0, at - Date.now()));
                        put(`event: delta\ndata: ${JSON.stringify({ seq: i, text: j.words[i] })}\n\n`);
                    }
                    await Bun.sleep(Math.max(0, j.doneAt - Date.now()));
                    // Sleep may wake a millisecond early: the job is over once its last delta is out.
                    const v = jobView(j, Math.max(Date.now(), j.doneAt));
                    put(v.status === "failed" ? `event: error\ndata: ${JSON.stringify({ message: v.error })}\n\n` : `event: done\ndata: ${JSON.stringify(v)}\n\n`);
                    if (!closed) c.close();
                },
                cancel() {
                    closed = true;
                },
            });
            return new Response(body, { headers: { "content-type": "text/event-stream", "cache-control": "no-cache", ...cors } });
        }
        const job = p.match(/^\/api\/trevor\/jobs\/(job-\d+(?:-quick)?)$/);
        if (job) {
            const j = JOBS.get(job[1] ?? "");
            if (MODE === "offline") return json(queuedView(job[1] ?? ""));
            return j ? json(j.real ? realView(j) : jobView(j)) : json({ error: { code: "NOT_FOUND", message: "no such job" } }, 404);
        }
        if (p.startsWith("/api/trevor/answers/") && req.method === "POST") return new Response(null, { status: 204, headers: cors });

        if (req.method !== "GET" && req.method !== "HEAD") return json({ error: { code: "MOCK", message: "the Trevor preview refuses writes" } }, 405);
        const headers = new Headers(req.headers);
        headers.delete("host");
        if (demo) {
            headers.delete("authorization");
            headers.delete("cookie");
        }
        const res = await fetch(`${UPSTREAM}${p}${url.search}`, { headers });
        const out = new Headers(res.headers);
        out.delete("content-encoding");
        out.delete("content-length");
        for (const [k, v] of Object.entries(cors)) out.set(k, v);
        return new Response(await res.arrayBuffer(), { status: res.status, headers: out });
    },
});

const mockUrl = `http://127.0.0.1:${PORT}`;
console.log(`trevor mock on ${mockUrl} (mode ${MODE}), upstream GETs to ${UPSTREAM}; ${BANK.length} bank answers placed on the gatePos scale`);

// --- the worker process and the llama-server it starts -------------------------
let worker = null;
let workerPid = null;
/** llama-server pids seen as children of the worker: the only ones this preview ever stops. */
const llamaPids = new Set();
const spool = REAL ? fs.mkdtempSync(path.join(os.tmpdir(), "trevor-preview-spool-")) : null;
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
const signal = (pid, sig) => {
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

if (REAL) {
    for (const n of DECIDED.notes) console.log(`real answers: ${n}`);
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
} else {
    console.log(`canned answers (mock): ${DECIDED.why}`);
}

/** Stop the worker (it finishes a job in flight first), then the llama-server it started, by pid. */
async function stopWorker() {
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
    if (worker) signal(worker.pid, "SIGKILL");
    for (const pid of llamaPids) if (isLlama(pid)) signal(pid, "SIGKILL");
    releaseLock(workerPid);
    if (spool) fs.rmSync(spool, { recursive: true, force: true });
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
