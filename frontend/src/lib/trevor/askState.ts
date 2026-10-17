/**
 * THE ASK BOX'S STATE, as a pure reducer so every transition is tested
 * without a DOM or a network.
 *
 * held -> sending                    (a slot freed: see thread.ts, MAX_OPEN)
 * idle -> sending -> (done | queued | refused)
 * queued -> streaming -> done        (the stream carries the text)
 * queued -> done | failed | offline   (polling, or a stream that ended early)
 * offline -> queued                   (the reader resumes; the question never left the queue)
 *
 * Quick look (contract v1.1 section 1b): when no model can take the question
 * now, the backend answers at once from passages AND queues a deferred job for
 * the full answer. The ask's response then carries both an answer and an open
 * job; the turn is `queued` with that answer as `quick`, so the quick look is
 * shown at once and the full answer replaces it when the job finishes. A
 * deferred job that fails, or a watch that is refused, settles on the quick
 * look (`done`, `fullFailed`), never on an error: the reader already has an
 * answer.
 */
import type { AskResponse } from "#/types/generated/AskResponse";
import type { JobView } from "#/types/generated/JobView";
import type { TrevorAnswer } from "#/types/generated/TrevorAnswer";
import type { TrevorRung } from "#/types/generated/TrevorRung";
import type { TrevorSpoilerSafe } from "#/types/generated/TrevorSpoilerSafe";
import { type IParsedAnswer, parseAnswer } from "./answer";
import { type TrevorFailureKind, trevorFailureKind } from "./errors";

interface IAsked {
    question: string;
}
interface IJob extends IAsked {
    jobId: string;
    rung: TrevorRung;
    spoiler: TrevorSpoilerSafe | null;
    /** The quick look shown while the full answer is made, if any. */
    quick?: IParsedAnswer;
    /** The deferred full answer's job, followed once this (quick passages) job settles. */
    next?: string;
}

export type AskState =
    | { kind: "idle" }
    | ({ kind: "held" } & IAsked)
    | ({ kind: "sending" } & IAsked)
    | ({ kind: "queued"; etaSecs: number | null; position: number | null; workerOnline: boolean } & IJob)
    | ({ kind: "streaming"; text: string } & IJob)
    | ({ kind: "offline" } & IJob)
    | ({ kind: "done"; answer: IParsedAnswer; rung: TrevorRung; spoiler: TrevorSpoilerSafe | null; cached: boolean; quick?: boolean; fullFailed?: boolean } & IAsked)
    | ({ kind: "failed"; message: string } & IAsked)
    | ({ kind: "refused"; failure: TrevorFailureKind; error: unknown } & IAsked);

export type AskEvent =
    | { type: "send"; question: string }
    | { type: "hold"; question: string }
    | { type: "accepted"; response: AskResponse }
    | { type: "text"; text: string }
    | { type: "progress"; view: JobView }
    | { type: "done"; view: JobView }
    | { type: "jobFailed"; message: string }
    | { type: "requestError"; error: unknown }
    | { type: "offline" }
    | { type: "resume" }
    | { type: "reset" };

export const ASK_IDLE: AskState = { kind: "idle" };

/** The job fields of a state that has a job, or null. */
function jobOf(s: AskState): IJob | null {
    if (s.kind !== "queued" && s.kind !== "streaming" && s.kind !== "offline") return null;
    const job: IJob = { question: s.question, jobId: s.jobId, rung: s.rung, spoiler: s.spoiler };
    if (s.quick) job.quick = s.quick;
    if (s.next) job.next = s.next;
    return job;
}

/** The job's quick passages job settled: follow the deferred full answer, with the passages answer (if any) as the quick look. */
function toDeferred(job: IJob, answer: IParsedAnswer | null): AskState {
    const quick = answer ?? job.quick;
    const next: AskState = { kind: "queued", question: job.question, jobId: job.next as string, rung: job.rung, spoiler: job.spoiler, etaSecs: null, position: null, workerOnline: true };
    return quick ? { ...next, quick } : next;
}

/** The quick look a deferred job settles on when its full answer cannot be had. */
function quickFallback(job: IJob): AskState | null {
    return job.quick ? { kind: "done", question: job.question, answer: job.quick, rung: job.quick.rung, spoiler: job.spoiler, cached: false, quick: true, fullFailed: true } : null;
}

/** The v1.1 fields of an ask's response, read whether or not the generated type has them yet. */
type AskResponseV11 = AskResponse & { deferredJobId?: string | null; quickLook?: TrevorAnswer | null; deferred?: boolean };

/**
 * The job of the deferred full answer, or null. The backend names it
 * `deferredJobId` beside the quick passages job; the dev preview's mock
 * (`scripts/trevor-preview.mjs`) answers the passages inline and flags the
 * ask's own job `deferred`.
 */
export function deferredJob(r: AskResponse): string | null {
    const v = r as AskResponseV11;
    if (v.deferredJobId) return v.deferredJobId;
    return v.deferred === true && r.answer && r.jobId ? r.jobId : null;
}

/** The response is a quick look with its full answer deferred. */
export function isDeferred(r: AskResponse): boolean {
    return deferredJob(r) !== null;
}

export function askReducer(s: AskState, e: AskEvent): AskState {
    switch (e.type) {
        case "reset":
            return ASK_IDLE;
        case "send":
            return { kind: "sending", question: e.question };
        case "hold":
            return { kind: "held", question: e.question };
        case "accepted": {
            if (s.kind !== "sending") return s;
            const r = e.response;
            const deferred = deferredJob(r);
            const looked = (r as AskResponseV11).quickLook;
            const quick = r.answer ? parseAnswer(r.answer) : looked ? parseAnswer(looked) : undefined;
            const base = { question: s.question, rung: r.rung, spoiler: r.spoilerSafeUpTo, etaSecs: r.etaSecs, position: null, workerOnline: r.etaSecs !== null };
            if (deferred) {
                // The quick look in hand: follow the full answer. Not yet: follow the passages job first, then the full answer.
                if (quick || !r.jobId || r.jobId === deferred) return { kind: "queued", ...base, jobId: deferred, ...(quick ? { quick } : {}) };
                return { kind: "queued", ...base, jobId: r.jobId, next: deferred };
            }
            // A passages-only ask still running may carry an earlier era's answer to show meanwhile.
            if (!r.answer && r.jobId && quick) return { kind: "queued", ...base, jobId: r.jobId, quick };
            if (r.answer) return { kind: "done", question: s.question, answer: parseAnswer(r.answer), rung: r.rung, spoiler: r.spoilerSafeUpTo, cached: r.jobId === null };
            if (!r.jobId) return { kind: "failed", question: s.question, message: "" };
            return { kind: "queued", question: s.question, jobId: r.jobId, rung: r.rung, spoiler: r.spoilerSafeUpTo, etaSecs: r.etaSecs, position: null, workerOnline: r.etaSecs !== null };
        }
        case "text": {
            const job = jobOf(s);
            return job ? { kind: "streaming", ...job, text: e.text } : s;
        }
        case "progress": {
            const job = jobOf(s);
            if (!job || s.kind === "streaming") return s;
            return { kind: "queued", ...job, rung: e.view.rung, etaSecs: e.view.etaSecs, position: e.view.position > 0 ? e.view.position : null, workerOnline: e.view.workerOnline };
        }
        case "done": {
            const job = jobOf(s);
            if (!job) return s;
            if (job.next) return toDeferred(job, e.view.answer ? parseAnswer(e.view.answer) : null);
            if (!e.view.answer) return quickFallback(job) ?? { kind: "failed", question: job.question, message: e.view.error ?? "" };
            return { kind: "done", question: job.question, answer: parseAnswer(e.view.answer), rung: e.view.rung, spoiler: job.spoiler, cached: false };
        }
        case "jobFailed": {
            const job = jobOf(s);
            if (job?.next) return toDeferred(job, null);
            return job ? (quickFallback(job) ?? { kind: "failed", question: job.question, message: e.message }) : s;
        }
        case "requestError": {
            if (s.kind === "sending") return { kind: "refused", question: s.question, failure: trevorFailureKind(e.error), error: e.error };
            const job = jobOf(s);
            return job ? (quickFallback(job) ?? { kind: "refused", question: job.question, failure: trevorFailureKind(e.error), error: e.error }) : s;
        }
        case "offline": {
            const job = jobOf(s);
            return job ? { kind: "offline", ...job } : s;
        }
        case "resume": {
            if (s.kind !== "offline") return s;
            const job = jobOf(s);
            return job ? { kind: "queued", ...job, etaSecs: null, position: null, workerOnline: false } : s;
        }
    }
}

/** The question is not settled yet: waiting to be sent, sent, queued or streaming. */
export function askPending(s: AskState): boolean {
    return s.kind === "held" || s.kind === "sending" || s.kind === "queued" || s.kind === "streaming";
}
