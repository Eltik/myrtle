/**
 * THE ASK BOX'S STATE, as a pure reducer so every transition is tested
 * without a DOM or a network.
 *
 * held -> sending                    (a slot freed: see thread.ts, MAX_OPEN)
 * idle -> sending -> (done | queued | refused)
 * queued -> streaming -> done        (the stream carries the text)
 * queued -> done | failed | offline   (polling, or a stream that ended early)
 * offline -> queued                   (the reader resumes; the question never left the queue)
 */
import type { AskResponse } from "#/types/generated/AskResponse";
import type { JobView } from "#/types/generated/JobView";
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
}

export type AskState =
    | { kind: "idle" }
    | ({ kind: "held" } & IAsked)
    | ({ kind: "sending" } & IAsked)
    | ({ kind: "queued"; etaSecs: number | null; position: number | null; workerOnline: boolean } & IJob)
    | ({ kind: "streaming"; text: string } & IJob)
    | ({ kind: "offline" } & IJob)
    | ({ kind: "done"; answer: IParsedAnswer; rung: TrevorRung; spoiler: TrevorSpoilerSafe | null; cached: boolean } & IAsked)
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
    return s.kind === "queued" || s.kind === "streaming" || s.kind === "offline" ? { question: s.question, jobId: s.jobId, rung: s.rung, spoiler: s.spoiler } : null;
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
            if (!e.view.answer) return { kind: "failed", question: job.question, message: e.view.error ?? "" };
            return { kind: "done", question: job.question, answer: parseAnswer(e.view.answer), rung: e.view.rung, spoiler: job.spoiler, cached: false };
        }
        case "jobFailed": {
            const job = jobOf(s);
            return job ? { kind: "failed", question: job.question, message: e.message } : s;
        }
        case "requestError": {
            if (s.kind === "sending") return { kind: "refused", question: s.question, failure: trevorFailureKind(e.error), error: e.error };
            const job = jobOf(s);
            return job ? { kind: "refused", question: job.question, failure: trevorFailureKind(e.error), error: e.error } : s;
        }
        case "offline": {
            const job = jobOf(s);
            return job ? { kind: "offline", ...job } : s;
        }
        case "resume": {
            if (s.kind !== "offline") return s;
            return { kind: "queued", question: s.question, jobId: s.jobId, rung: s.rung, spoiler: s.spoiler, etaSecs: null, position: null, workerOnline: false };
        }
    }
}

/** The question is not settled yet: waiting to be sent, sent, queued or streaming. */
export function askPending(s: AskState): boolean {
    return s.kind === "held" || s.kind === "sending" || s.kind === "queued" || s.kind === "streaming";
}
