/**
 * THE STORIES TAB'S DATA (contract v1.1 section 5): the popular questions of
 * the reader's era and the reader's own past questions ("My questions").
 *
 * The popular list comes as `{spoilerSafeUpTo, items}` (the generated
 * `TrevorPopular`). My questions is read defensively: the list may come bare
 * or under `jobs` / `items`, a time as an ISO string or unix seconds. In both,
 * anything that does not hold a question is dropped instead of breaking the
 * tab. The backend pages My questions by `before=<createdAt of the last row>`
 * and sends no cursor; the mock sends `nextBefore`. A
 * backend without the routes (404) reads as `null`, and the tab works without
 * them.
 *
 * A past question becomes a thread turn (`toTurnState`): a finished one is
 * re-shown with its stored answer and no request; one still queued or running
 * is followed again by the hook's watcher; a deferred one shows its quick look
 * until the full answer lands.
 */
import type { JobStatus } from "#/types/generated/JobStatus";
import type { TrevorAnswer } from "#/types/generated/TrevorAnswer";
import type { TrevorRung } from "#/types/generated/TrevorRung";
import type { TrevorSpoilerSafe } from "#/types/generated/TrevorSpoilerSafe";
import { parseAnswer } from "./answer";
import type { TurnState } from "./thread";

export interface IPopularQuestion {
    question: string;
    answer: TrevorAnswer | null;
    askers: number | null;
}

export interface IPopular {
    questions: IPopularQuestion[];
    /** The era the list was drawn from: null before the first era or signed out, undefined when the body is malformed. */
    spoiler: TrevorSpoilerSafe | null | undefined;
}

export interface IMyJob {
    /** Null for a question answered from the cache (no job ran). */
    jobId: string | null;
    /** The row's key in the list: the job, else the time and question. */
    key: string;
    question: string;
    status: JobStatus;
    rung: TrevorRung;
    source: "live" | "passages" | null;
    deferred: boolean;
    /** Unix milliseconds. */
    createdAt: number | null;
    /** `createdAt` as the backend sent it: the backend's paging cursor (`before`). */
    createdAtRaw: string | null;
    finishedAt: number | null;
    answer: TrevorAnswer | null;
    /** The passages answer given at once for a deferred job, when the backend keeps it. */
    quickLook: TrevorAnswer | null;
    error: string | null;
}

export interface IMyJobs {
    jobs: IMyJob[];
    nextBefore: string | null;
}

type Obj = Record<string, unknown>;

function isObj(v: unknown): v is Obj {
    return typeof v === "object" && v !== null && !Array.isArray(v);
}

function str(v: unknown): string | null {
    return typeof v === "string" && v.trim() !== "" ? v : null;
}

function list(body: unknown, keys: string[]): unknown[] {
    if (Array.isArray(body)) return body;
    if (!isObj(body)) return [];
    for (const k of keys) {
        const v = body[k];
        if (Array.isArray(v)) return v;
    }
    return [];
}

/** A stored answer, when the value is shaped like one (`answer` holds the worker's result). */
function answerOf(v: unknown): TrevorAnswer | null {
    if (!isObj(v) || !("answer" in v)) return null;
    return { id: str(v.id) ?? "", version: str(v.version) ?? "", server: str(v.server) ?? "en", eraPos: typeof v.eraPos === "number" ? v.eraPos : 0, rung: (str(v.rung) ?? "passages") as TrevorRung, source: v.source === "live" ? "live" : "passages", answer: v.answer as TrevorAnswer["answer"] };
}

function spoilerOf(v: unknown): TrevorSpoilerSafe | null | undefined {
    if (v === null) return null;
    if (!isObj(v) || typeof v.position !== "number") return undefined;
    return { eraId: str(v.eraId) ?? "", name: str(v.name) ?? "", position: v.position };
}

/** Unix ms from an ISO string, unix seconds or unix ms. */
export function timeOf(v: unknown): number | null {
    if (typeof v === "number" && Number.isFinite(v)) return v < 1e12 ? v * 1000 : v;
    if (typeof v === "string") {
        const ms = Date.parse(v);
        return Number.isNaN(ms) ? null : ms;
    }
    return null;
}

const STATUSES: readonly JobStatus[] = ["queued", "running", "done", "failed"];

/** `GET /trevor/popular`: `{spoilerSafeUpTo, items: [{question, askers, answer}]}`. A question listed twice is kept once. */
export function parsePopular(body: unknown): IPopular {
    if (!isObj(body)) return { questions: [], spoiler: undefined };
    const questions: IPopularQuestion[] = [];
    const seen = new Set<string>();
    for (const item of Array.isArray(body.items) ? body.items : []) {
        const question = isObj(item) ? str(item.question) : null;
        if (!isObj(item) || !question || seen.has(question.trim().toLowerCase())) continue;
        seen.add(question.trim().toLowerCase());
        questions.push({ question: question.trim(), answer: answerOf(item.answer), askers: typeof item.askers === "number" ? item.askers : null });
    }
    return { questions, spoiler: spoilerOf(body.spoilerSafeUpTo) };
}

export function parseMyJobs(body: unknown, limit: number | null = null): IMyJobs {
    const jobs: IMyJob[] = [];
    for (const item of list(body, ["jobs", "items"])) {
        if (!isObj(item)) continue;
        const jobId = str(item.jobId) ?? str(item.id);
        const question = str(item.question);
        if (!question) continue;
        const createdAtRaw = typeof item.createdAt === "string" ? item.createdAt : typeof item.createdAt === "number" ? String(item.createdAt) : null;
        const status = STATUSES.includes(item.status as JobStatus) ? (item.status as JobStatus) : "queued";
        if (!jobId && createdAtRaw === null) continue;
        jobs.push({
            jobId,
            key: jobId ?? `${createdAtRaw}|${question}`,
            question,
            status,
            rung: (str(item.rung) ?? "passages") as TrevorRung,
            source: item.source === "live" || item.source === "passages" ? item.source : null,
            deferred: item.deferred === true,
            createdAt: timeOf(item.createdAt),
            createdAtRaw,
            finishedAt: timeOf(item.finishedAt),
            answer: answerOf(item.answer),
            quickLook: answerOf(item.quickLook),
            error: str(item.error),
        });
    }
    const named = isObj(body) ? str(body.nextBefore) : null;
    const last = jobs[jobs.length - 1];
    // A bare list has no cursor: a full page continues before its last row's time.
    const derived = Array.isArray(body) && limit !== null && jobs.length >= limit && last ? last.createdAtRaw : null;
    return { jobs, nextBefore: named ?? derived };
}

/** A past question as a thread turn: answered, failed, or open and to be followed again. */
export function toTurnState(job: IMyJob): TurnState {
    const question = job.question;
    if (job.status === "done" && job.answer) return { kind: "done", question, answer: parseAnswer(job.answer), rung: job.answer.rung, spoiler: null, cached: true };
    if (job.status === "done" || job.status === "failed") {
        if (job.quickLook) return { kind: "done", question, answer: parseAnswer(job.quickLook), rung: job.quickLook.rung, spoiler: null, cached: true, quick: true, fullFailed: true };
        return { kind: "failed", question, message: job.error ?? "" };
    }
    if (!job.jobId) return { kind: "failed", question, message: job.error ?? "" };
    const base = { kind: "queued" as const, question, jobId: job.jobId, rung: job.rung, spoiler: null, etaSecs: null, position: null, workerOnline: true };
    return job.quickLook ? { ...base, quick: parseAnswer(job.quickLook) } : base;
}

/** The job is still open (queued or running). */
export function jobOpen(job: IMyJob): boolean {
    return job.status === "queued" || job.status === "running";
}
