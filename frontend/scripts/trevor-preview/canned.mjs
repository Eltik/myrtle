// Canned answers: what the mock says when no worker runs, each job on its own clock.
import fs from "node:fs";
import path from "node:path";
import { cors, PANELS, STORY, sseEvent, VERSION } from "./config.mjs";
import { asks, openJobs } from "./store.mjs";

export const ANSWER_TEXT =
    "**Hibiscus** is a nurse working in the Afterglow district of Leithanien [1]. She treats the Infected who come to the clinic [1, 2] and argues with Czerny about cancelling the duet [3].\n\n*   She is Kreide's sister [2].\n*   Andante worries she is in no shape to help anyone [3].\n\nOn this evidence, she is the one keeping the clinic running.";
const CITATIONS = [
    { n: 1, storyId: "act18side_level_act18side_04_end", chunkId: "act18side_level_act18side_04_end#0003", lineStart: 41, lineEnd: 58, text: "Hibiscus: Hold still. The crystals have spread past your elbow, and I need to see how far." },
    { n: 2, storyId: STORY, chunkId: `${STORY}#0001`, lineStart: 12, lineEnd: 30, text: "Kreide: You always do this, Hibiscus. Somebody walks in coughing and you forget you were ever tired." },
    { n: 3, storyId: "profile_hibiscus", chunkId: "profile_hibiscus#0000", lineStart: 0, lineEnd: 0, text: "An operator file excerpt." },
];
export const SPOILER = { eraId: "main_10", name: "Shatterpoint", position: 722 };
export const answer = (source, text = ANSWER_TEXT, citations = CITATIONS) => ({
    id: `11111111-2222-3333-4444-${String(asks).padStart(12, "0")}`,
    version: VERSION,
    server: "en",
    eraPos: 722,
    rung: source === "live" ? "cpu" : "passages",
    source,
    answer: { answer: source === "live" ? text : "", rung: "cpu", profile: "mid", source, citations, corpusVersion: VERSION },
});
export const queuedView = (id) => ({ jobId: id, status: "queued", rung: "cpu", position: 1, etaSecs: null, workerOnline: false, answer: null, error: null });

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
export function compose(question, storyId) {
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

export function jobView(j, now = Date.now()) {
    if (now >= j.doneAt) {
        if (j.fail) return { jobId: j.id, status: "failed", rung: j.rung ?? "cpu", position: 0, etaSecs: null, workerOnline: true, answer: null, error: "the mock was asked to fail" };
        return { jobId: j.id, status: "done", rung: j.rung ?? "cpu", position: 0, etaSecs: null, workerOnline: true, answer: j.answer, error: null };
    }
    if (now >= j.start) return { jobId: j.id, status: "running", rung: "cpu", position: 0, etaSecs: Math.ceil((j.doneAt - now) / 1000), workerOnline: true, answer: null, error: null };
    const ahead = openJobs(now).filter((o) => !o.real && o.start < j.start).length;
    return { jobId: j.id, status: "queued", rung: "cpu", position: ahead + 1, etaSecs: Math.ceil((j.doneAt - now) / 1000), workerOnline: true, answer: null, error: null };
}

/** The SSE stream of a canned job: its deltas on the job's own clock (a late stream only gets the ones still to come), then done or error. */
export function cannedStream(j) {
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
                put(sseEvent("delta", { seq: i, text: j.words[i] }));
            }
            await Bun.sleep(Math.max(0, j.doneAt - Date.now()));
            // Sleep may wake a millisecond early: the job is over once its last delta is out.
            const v = jobView(j, Math.max(Date.now(), j.doneAt));
            put(v.status === "failed" ? sseEvent("error", { message: v.error }) : sseEvent("done", v));
            if (!closed) c.close();
        },
        cancel() {
            closed = true;
        },
    });
    return new Response(body, { headers: { "content-type": "text/event-stream", "cache-control": "no-cache", ...cors } });
}
