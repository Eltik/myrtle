// The mock backend's routes: Trevor's reader and worker routes, the demo session and switches, and a read-only proxy for the rest.
import fs from "node:fs";
import path from "node:path";
import { DECLINES, seeds } from "./bank.mjs";
import { answer, cannedStream, queuedView, SPOILER } from "./canned.mjs";
import { cors, DEMO, DEMO_UID, GATE_POS, horizonOf, json, MAX_OPEN, MODES, normQ, PANELS, profileStory, QUEUE_MAX, STORY, setProfileStory, UPSTREAM, VERSION, VITE_PORT } from "./config.mjs";
import { REAL } from "./decide.mjs";
import { accepted, jobRow, newJob, viewOf } from "./jobs.mjs";
import { asks, countAsk, JOBS, openJobs } from "./store.mjs";
import { beat, realStream, workerOnline, workerRoute } from "./worker.mjs";

/** What an ask returns (MOCK_MODE, switched at /demo-mode/<mode>): stream, quick, passages, busy or offline. */
export let MODE = process.env.MOCK_MODE ?? "stream";
function setMode(mode) {
    MODE = mode;
}

function demoSession(req) {
    return req.headers.get("authorization") === `Bearer ${DEMO}` || (req.headers.get("cookie") ?? "").includes(`site_token=${DEMO}`);
}

/** Answers one request to the mock backend. */
export async function route(req) {
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
        setMode(mode[1] ?? MODE);
        return json({ mode: MODE });
    }
    const prof = p.match(/^\/demo-profile\/([^/]+)$/);
    if (prof) {
        const id = decodeURIComponent(prof[1] ?? "");
        if (id !== "none" && !GATE_POS.has(id)) return json({ error: `${id} is not in the active Trevor version (or use none)` }, 400);
        setProfileStory(id === "none" ? null : id);
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
            const v = viewOf(j);
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
        countAsk();
        if (MODE === "busy") return json({ error: { code: "SERVICE_UNAVAILABLE", message: "Trevor is busy" } }, 503);
        if (MODE === "passages") return json({ jobId: null, status: "done", rung: "cache", spoilerSafeUpTo: SPOILER, answer: answer("passages"), etaSecs: null, streamUrl: null });
        if (MODE === "offline") {
            const id = `job-${asks}`;
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
        return cannedStream(j);
    }
    const job = p.match(/^\/api\/trevor\/jobs\/(job-\d+(?:-quick)?)$/);
    if (job) {
        const j = JOBS.get(job[1] ?? "");
        if (MODE === "offline") return json(queuedView(job[1] ?? ""));
        return j ? json(viewOf(j)) : json({ error: { code: "NOT_FOUND", message: "no such job" } }, 404);
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
}
