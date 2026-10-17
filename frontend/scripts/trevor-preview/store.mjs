// Every job of this run, and the ask counter.
/**
 * Every job of this run, in creation order. Canned: { id, uid, question, nq, horizon, deferred, quickJob, created,
 * words, sent, fail, deltaMs, start, doneAt, answer }. Real (real: true): { ..., status, started, finished, firstDelta,
 * answer, error, subs, nextSeq, pending }. A deferred job's quick passages job (quick: true) is always canned.
 */
export const JOBS = new Map();
/** Asks made this run (refused ones included): numbers the jobs and the canned answers. */
export let asks = 0;
export function countAsk() {
    asks += 1;
    return asks;
}

/** A job still waiting for its answer. */
export const isOpen = (j, now = Date.now()) => (j.real ? j.status === "queued" || j.status === "running" : now < j.doneAt);
export const openJobs = (now = Date.now()) => [...JOBS.values()].filter((j) => isOpen(j, now));
export const realJobs = () => [...JOBS.values()].filter((j) => j.real);
