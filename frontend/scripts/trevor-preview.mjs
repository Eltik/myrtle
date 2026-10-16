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
//   SSE stream (a canned answer, word by word), and feedback;
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
const answer = (source) => ({
    id: `11111111-2222-3333-4444-${String(jobs).padStart(12, "0")}`,
    version: VERSION,
    server: "en",
    eraPos: 722,
    rung: source === "live" ? "cf" : "passages",
    source,
    answer: { answer: source === "live" ? ANSWER_TEXT : "", rung: "cf", profile: "full", source, citations: CITATIONS, corpusVersion: VERSION },
});
const doneView = (id) => ({ jobId: id, status: "done", rung: "cf", position: 0, etaSecs: null, workerOnline: true, answer: answer("live"), error: null });
const queuedView = (id) => ({ jobId: id, status: "queued", rung: "cpu", position: 1, etaSecs: null, workerOnline: false, answer: null, error: null });

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
            const id = `job-${jobs}`;
            return json({ jobId: id, status: "queued", rung: "cf", spoilerSafeUpTo: SPOILER, etaSecs: MODE === "offline" ? null : 15, streamUrl: `/api/trevor/jobs/${id}/stream` }, 202);
        }
        const stream = p.match(/^\/api\/trevor\/jobs\/(job-\d+)\/stream$/);
        if (stream) {
            if (MODE === "offline") return json({ error: { code: "NOT_FOUND", message: "no stream" } }, 404);
            const id = stream[1] ?? "job-0";
            const enc = new TextEncoder();
            const words = ANSWER_TEXT.split(/(?<= )/);
            const delay = Number(process.env.MOCK_DELTA_MS ?? 120);
            const body = new ReadableStream({
                async start(c) {
                    await Bun.sleep(Number(process.env.MOCK_QUEUE_MS ?? 2500));
                    c.enqueue(enc.encode(": keepalive\n\n"));
                    for (let i = 0; i < words.length; i++) {
                        c.enqueue(enc.encode(`event: delta\ndata: ${JSON.stringify({ seq: i, text: words[i] })}\n\n`));
                        await Bun.sleep(delay);
                    }
                    c.enqueue(enc.encode(`event: done\ndata: ${JSON.stringify(doneView(id))}\n\n`));
                    c.close();
                },
            });
            return new Response(body, { headers: { "content-type": "text/event-stream", "cache-control": "no-cache", ...cors } });
        }
        const job = p.match(/^\/api\/trevor\/jobs\/(job-\d+)$/);
        if (job) return json(MODE === "offline" ? queuedView(job[1] ?? "") : doneView(job[1] ?? ""));
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
