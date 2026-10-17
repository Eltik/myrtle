// The jobs an ask makes, and how the mock reports them: the ask's response and the My questions rows.
import { DEMO_UID, iso, json, normQ, rand } from "./config.mjs";
import { quickLook } from "./bank.mjs";
import { answer, compose, jobView, SPOILER } from "./canned.mjs";
import { REAL } from "./decide.mjs";
import { asks, JOBS } from "./store.mjs";
import { enqueued, realView } from "./worker.mjs";

/** A job's view (`GET /trevor/jobs/{id}`), real or canned. */
export const viewOf = (j) => (j.real ? realView(j) : jobView(j));

/** One row of "My questions" (contract v1.1 section 5). */
export function jobRow(j) {
    const v = viewOf(j);
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
export function newJob(question, body, horizon, deferred) {
    const now = Date.now();
    const id = `job-${asks}`;
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
        const v = viewOf(o);
        if (v.status === "done" && v.answer) return v.answer;
    }
    return null;
}

/** POST /api/trevor/ask's answer for a job: queued, or a deferred job's quick passages job (contract v1.1 section 1b). */
export function accepted(j) {
    const spoiler = REAL ? j.horizon.spoiler : (j.horizon.spoiler ?? SPOILER);
    const shown = j.deferred ? JOBS.get(j.quickJob) : j;
    const v = viewOf(shown);
    const out = { jobId: shown.id, status: v.status === "running" ? "running" : "queued", rung: shown.real ? "cpu" : "cf", spoilerSafeUpTo: spoiler, etaSecs: v.etaSecs, streamUrl: `/api/trevor/jobs/${shown.id}/stream` };
    if (!j.deferred) return json(out, 202);
    const look = earlierAnswer(j);
    return json({ ...out, rung: "passages", deferredJobId: j.id, ...(look ? { quickLook: look } : {}) }, 202);
}
