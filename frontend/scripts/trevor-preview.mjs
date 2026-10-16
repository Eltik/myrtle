// `bun run dev:trevor`: the Trevor reader panel in the dev server with no Trevor backend.
//
// Starts a mock backend on 127.0.0.1:3999 and `vite dev` on 127.0.0.1:3005 with
// VITE_TREVOR=1 pointed at it, and stops vite when this process stops (Ctrl+C).
// Both bind 127.0.0.1, not localhost, so the demo sign-in cookie never lands on
// the localhost dev server (:3000) or replaces a real session there.
//
// The mock:
// - fakes the Trevor routes: /trevor/status, /trevor/panels/<story> from the
//   published artifacts in trevor/artifacts/panels, /trevor/ask, the job and its
//   SSE stream, and feedback. Each ask is its own job with its own latency
//   (0.8 to 4 s in the queue, 50 to 150 ms a word). The answer depends on the
//   question: a canned one for "Hibiscus" or "Ebenholz", the story's recap as
//   a long answer for "recap"/"summary"/"so far", a stream that fails half way
//   for "fail", and otherwise a reply marked "Demo answer (mock backend)" that
//   repeats the question and cites the story's own panel. Like the backend, it
//   holds at most 3 open jobs (a 4th ask is a 409), joins a twin (the same
//   question while it is open) to the open job, and a stream opened late only
//   carries the deltas still to come;
// - fakes a signed-in session for the cookie `site_token=trevor-demo`
//   (open http://127.0.0.1:3999/demo-login to set it, /demo-logout to clear it);
// - proxies every other GET to the real backend (MOCK_UPSTREAM, default
//   http://localhost:3060) without the demo token, and refuses any other method,
//   so nothing the preview does writes anywhere.
//
// MOCK_MODE picks what an ask returns: stream (default), passages (a cached
// passages-only answer), busy (503), offline (queued, then nobody answers).
// http://127.0.0.1:3999/demo-mode/<mode> switches it while the preview runs.
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const ROOT = path.resolve(import.meta.dir, "../..");
const PANELS = path.join(ROOT, "trevor/artifacts/panels");
const PORT = Number(process.env.MOCK_PORT ?? 3999);
const VITE_PORT = Number(process.env.VITE_PORT ?? 3005);
const UPSTREAM = process.env.MOCK_UPSTREAM ?? "http://localhost:3060";
const MODES = ["stream", "passages", "busy", "offline"];
let MODE = process.env.MOCK_MODE ?? "stream";
const DEMO = "trevor-demo";
const DEMO_UID = process.env.MOCK_UID ?? "55699327";
const STORY = "act18side_level_act18side_05_beg";

if (!fs.existsSync(path.join(PANELS, "manifest.json"))) {
    console.error(`no panel artifacts at ${PANELS}; build or copy them first`);
    process.exit(1);
}
const VERSION = JSON.parse(fs.readFileSync(path.join(PANELS, "manifest.json"), "utf8")).version;

const cors = { "access-control-allow-origin": `http://127.0.0.1:${VITE_PORT}`, "access-control-allow-credentials": "true", "access-control-allow-headers": "*", "access-control-allow-methods": "GET, POST, OPTIONS" };
const json = (body, status = 200, headers = {}) => new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json", ...cors, ...headers } });

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

// Backend MAX_OPEN_PER_USER.
const MAX_OPEN = 3;
/** id -> { id, question, text, citations, fail, start (first delta), deltaMs, words, doneAt, answer } */
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

function openJobs(now = Date.now()) {
    return [...JOBS.values()].filter((j) => now < j.doneAt);
}

function jobView(j, now = Date.now()) {
    if (now >= j.doneAt) {
        if (j.fail) return { jobId: j.id, status: "failed", rung: "cf", position: 0, etaSecs: null, workerOnline: true, answer: null, error: "the mock was asked to fail" };
        return { jobId: j.id, status: "done", rung: "cf", position: 0, etaSecs: null, workerOnline: true, answer: j.answer, error: null };
    }
    if (now >= j.start) return { jobId: j.id, status: "running", rung: "cf", position: 0, etaSecs: Math.ceil((j.doneAt - now) / 1000), workerOnline: true, answer: null, error: null };
    const ahead = openJobs(now).filter((o) => o.start < j.start).length;
    return { jobId: j.id, status: "queued", rung: "cf", position: ahead + 1, etaSecs: Math.ceil((j.doneAt - now) / 1000), workerOnline: true, answer: null, error: null };
}

function demoSession(req) {
    return req.headers.get("authorization") === `Bearer ${DEMO}` || (req.headers.get("cookie") ?? "").includes(`site_token=${DEMO}`);
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
        if (p === "/api/auth/verify" && demo) return json({ valid: true, uid: DEMO_UID, role: "user", canAccessAdminPanel: false });
        if (p === "/api/trevor/status") return json({ online: MODE !== "offline", rungs: { cf: true, cpu: true }, activeVersion: VERSION, queueDepth: { cf: 0, cpu: 1 }, cfBudget: "ok" });
        const panel = p.match(/^\/api\/trevor\/panels\/(.+)$/);
        if (panel) {
            const file = path.join(PANELS, "stories", `${path.basename(decodeURIComponent(panel[1] ?? ""))}.json`);
            if (!fs.existsSync(file)) return json({ error: { code: "NOT_FOUND", message: "no Trevor panels" } }, 404);
            return json({ storyId: panel[1], server: "en", version: VERSION, panels: JSON.parse(fs.readFileSync(file, "utf8")) });
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
            const now = Date.now();
            const open = openJobs(now);
            const twin = open.find((j) => j.question.toLowerCase() === question.toLowerCase());
            if (twin) return json({ jobId: twin.id, status: "queued", rung: "cf", spoilerSafeUpTo: SPOILER, etaSecs: Math.ceil((twin.doneAt - now) / 1000), streamUrl: `/api/trevor/jobs/${twin.id}/stream` }, 202);
            if (open.length >= MAX_OPEN) return json({ error: { code: "CONFLICT", message: `at most ${MAX_OPEN} questions may wait at once; wait for one to finish` } }, 409);
            const id = `job-${jobs}`;
            const c = compose(question, body.horizonStory);
            const words = c.text.split(/(?<= )/);
            const deltaMs = Number(process.env.MOCK_DELTA_MS ?? rand(50, 150));
            const start = now + Number(process.env.MOCK_QUEUE_MS ?? rand(800, 4000));
            // A failing job stops half way through its words.
            const sent = c.fail ? Math.floor(words.length / 2) : words.length;
            JOBS.set(id, { id, question, words, sent, fail: !!c.fail, deltaMs, start, doneAt: start + sent * deltaMs, answer: answer("live", c.text, c.citations) });
            return json({ jobId: id, status: "queued", rung: "cf", spoilerSafeUpTo: SPOILER, etaSecs: Math.ceil((start + sent * deltaMs - now) / 1000), streamUrl: `/api/trevor/jobs/${id}/stream` }, 202);
        }
        const stream = p.match(/^\/api\/trevor\/jobs\/(job-\d+)\/stream$/);
        if (stream) {
            const j = JOBS.get(stream[1] ?? "");
            if (MODE === "offline" || !j) return json({ error: { code: "NOT_FOUND", message: "no stream" } }, 404);
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
        const job = p.match(/^\/api\/trevor\/jobs\/(job-\d+)$/);
        if (job) {
            const j = JOBS.get(job[1] ?? "");
            if (MODE === "offline") return json(queuedView(job[1] ?? ""));
            return j ? json(jobView(j)) : json({ error: { code: "NOT_FOUND", message: "no such job" } }, 404);
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
console.log(`trevor mock on ${mockUrl} (mode ${MODE}), upstream GETs to ${UPSTREAM}`);
if (process.env.MOCK_ONLY === "1") {
    console.log("MOCK_ONLY=1: not starting vite");
} else {
    const vite = spawn("bunx", ["vite", "dev", "--host", "127.0.0.1", "--port", String(VITE_PORT), "--strictPort"], {
        cwd: path.join(ROOT, "frontend"),
        stdio: "inherit",
        env: { ...process.env, VITE_TREVOR: "1", BACKEND_URL: mockUrl, VITE_BACKEND_URL: mockUrl },
    });
    console.log(`reader with Trevor: http://127.0.0.1:${VITE_PORT}/stories/${STORY}  (sign in as the demo: ${mockUrl}/demo-login)`);
    const stop = () => {
        vite.kill("SIGTERM");
        server.stop(true);
        process.exit(0);
    };
    process.on("SIGINT", stop);
    process.on("SIGTERM", stop);
    vite.on("exit", (code) => {
        server.stop(true);
        process.exit(code ?? 0);
    });
}
